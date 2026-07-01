//! Minimal x86_64 paging helpers
//! 
//! This module is not a full memory manager. It exists to support the v0.0.4
//! Local APIC MMIO experiment by mapping one 4 KiB MMIO page into the active
//! page tables.
//! 
//! The implementation assumes 4-level paging and uses Limine's HHDM offset to
//! access physical page tables through virtual addresses.
//! 
//! This is intentionally narrow:
//! - read active CR3
//! - walk page tables
//! - translate existing virtual addresses to physical addresses
//! - allocate a tiny static pool of page tables
//! map one 4 KiB MMIO page
//! 

#![allow(dead_code)]

use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::arch::x86_64::{addr, apic, cpu};

/// Size of one x86_64 page
const PAGE_SIZE: usize = 4096;

/// Page table entry present bit
const PTE_PRESENT: u64 = 1 << 0;

/// Page table entry writable bit
const PTE_WRITABLE: u64 = 1 << 1;

/// Page-level write-through bit
/// 
/// For MMIO, this is part of a conservative cache-control configuration
const PTE_WRITE_THROUGH: u64 = 1 << 3;

/// Page-level cache-disable bit
/// 
/// For MMIO, this prevents ordinary cacheable memory semantics
const PTE_CACHE_DISABLE: u64 = 1 << 4;

/// Page size bit used by 1 GiB and 2 MiB mappings
const PTE_HUGE_PAGE: u64 = 1 << 7;

/// Address mask for intermediate page-table entries
const PTE_ADDR_MASK: u64 = 0x000f_ffff_ffff_f000;

/// Flags used for intermediate page-table entries
const TABLE_FLAGS: u64 = PTE_PRESENT | PTE_WRITABLE;

/// Flags used for the Local APIC MMIO page
const MMIO_PAGE_FLAGS: u64 = PTE_PRESENT | PTE_WRITABLE | PTE_WRITE_THROUGH | PTE_CACHE_DISABLE;

/// Physical address mask for CR3
const CR3_ADDR_MASK: u64 = 0x000f_ffff_ffff_f000;

/// One 4 KiB x86_64 page table
#[repr(C, align(4096))]
#[derive(Clone, Copy)]
struct PageTable {
    entries: [u64; 512]
}

impl PageTable {
    /// Creates a zero-filled page table
    const fn zero() -> Self {
        Self {
            entries: [0; 512]
        }
    }
}

/// Small static page-table pool for early MMIO mapping
/// 
/// This is not a general allocator. It is only enough to create missing paging
/// levels for the Local APIC virtual address path,
static mut EARLY_TABLE_POOL: [PageTable; 3] = [PageTable::zero(); 3];

/// Number of early page tables already consumed from `EARLY_TABLE_POOL`
static EARLY_TABLES_USED: AtomicUsize = AtomicUsize::new(0);

/// Errors produced by the minimal paging helpers
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PagingError {
    /// Limine did not provide an HHDM offset
    NoHhdm,

    /// A page table entry required for a walk was not present
    NotMapped,

    /// The walk encountered a huge page where a normal page table was expected
    HugePageConflict,

    /// The small static early table pool ran out of entries
    OutOfEarlyTables,

    /// A virtual address could not be translated to a physical address
    TranslationFailed
}

/// Initializes paging diagnostics
pub fn init() {
    let cr3 = cpu::read_cr3();
    let pml4_physical = active_pml4_physical();

    crate::kprintln!("[NX][PAGING] cr3={:#018x}", cr3);
    crate::kprintln!("[NX][PAGING] active_pml4_physical={:#018x}", pml4_physical);
}

/// Returns the physical address of the active PML4
pub fn active_pml4_physical() -> u64 {
    cpu::read_cr3() & CR3_ADDR_MASK
}

/// Maps the Local APIC MMIO page at its HHDM virtual candidate
/// 
/// This maps:
/// physical 0xfee00000 -> virtual hhdm_offset + 0xfee00000
/// 
/// The mapping uses present, writable, write-through and cache-disable falgs
pub fn map_local_apic_mmio() -> Result<(), PagingError> {
    let physical = apic::physical_base();

    let Some(virtual_address) = apic::local_apic_virtual_base_candidate() else {
        return Err(PagingError::NoHhdm);
    };

    crate::kprintln!(
        "[NX][PAGING] mapping Local APIC MMIO physical={:#018x} virtual={:#018x}",
        physical,
        virtual_address
    );

    map_mmio_page(virtual_address, physical)
}

/// Maps a single 4 KiB MMIO page
/// 
/// This function updates the active page tables directly
pub fn map_mmio_page(virtual_address: u64, physical_address: u64) -> Result<(), PagingError> {
    let virtual_page = align_down_4k(virtual_address);
    let physical_page = align_down_4k(physical_address);

    // SAFETY: This modifies the active page tables. The function creates
    // missing intermediate tables from a static early pool and maps exactly one
    // 4 KiB page. It is intended for early single-core boot before general
    // memory management exists.
    unsafe {
        map_page_4k(virtual_page, physical_page, MMIO_PAGE_FLAGS)?;
    }

    cpu::invlpg(virtual_page);

    crate::kprintln!("[NX][PAGING] MMIO page mapped");
    Ok(())
}

/// Translates a virtual address to a physical address using the active tables
/// 
/// This supports 4 KiB, 2 MiB, and 1 GiB mappings.
pub fn virtual_to_physical_address(virtual_address: u64) -> Result<u64, PagingError> {
    let pml4 = active_pml4_mut()?;

    // SAFETY: `pml4` is derived from the active CR3 physical address and
    // accessed through the HHDM. Page table walks only read entries.
    unsafe {
        translate_from_pml4(pml4, virtual_address)
    }
}

/// Maps one 4 KiB page in the active page tables
/// 
/// # Safety
/// This function mutates the active page tables. It must only be called during
/// controlled early boot or with appropriate synchronization.
unsafe fn map_page_4k(virtual_address: u64, physical_address: u64, flags: u64) -> Result<(), PagingError> {
    let pml4 = active_pml4_mut()?;

    let pml4_index = page_table_index(virtual_address, 39);
    let pdpt_index = page_table_index(virtual_address, 30);
    let pd_index = page_table_index(virtual_address, 21);
    let pt_index = page_table_index(virtual_address, 12);

    let pdpt = unsafe {
        ensure_next_table(pml4, pml4_index)?
    };

    let pd = unsafe {
        ensure_next_table(pdpt, pdpt_index)?
    };

    let pt = unsafe {
        ensure_next_table(pd, pd_index)?
    };

    // SAFETY: `pt` points to a valid page table ensured above
    let entry = unsafe {
        &mut (*pt).entries[pt_index]
    };

    if(*entry & PTE_PRESENT) != 0 {
        crate::kprintln!(
            "[NX][PAGING] replacing existing PTE for virtual={:#018x}",
            virtual_address
        );
    }

    *entry = (physical_address & PTE_ADDR_MASK) | flags;
    Ok(())
}

/// Ensures that `table[index]` points to a normal next-level page table
/// 
/// # Safety
/// `table` must point to a valid mutable page table in the active paging
/// hierarchy.
unsafe fn ensure_next_table(table: *mut PageTable, index: usize) -> Result<*mut PageTable, PagingError> {
    // SAFETY: The caller guarantees that `table` points to a valid page table.
    let entry = unsafe {
        &mut (*table).entries[index]
    };

    if (*entry & PTE_PRESENT) != 0 {
        if (*entry & PTE_HUGE_PAGE) != 0 {
            return Err(PagingError::HugePageConflict);
        }

        let next_physical = *entry & PTE_ADDR_MASK;
        return page_table_from_physical_mut(next_physical);
    }

    let (new_table, new_table_physical) = unsafe {
        allocate_early_table()?
    };

    *entry = (new_table_physical & PTE_ADDR_MASK) | TABLE_FLAGS;

    Ok(new_table)
}

/// Allocates one page table from the static early pool
/// 
/// # Safety
/// This is a tiny early-boot allocator. It must not be used as a general
/// physical frame allocator.
unsafe fn allocate_early_table() -> Result<(*mut PageTable, u64), PagingError> {
    let index = EARLY_TABLES_USED.fetch_add(1, Ordering::Relaxed);

    if index >= 3 {
        return Err(PagingError::OutOfEarlyTables);
    }

    // SAFETY: We select a unique table from the static pool using the atomic
    // counter above. Early boot is single-core, but the atomic keeps the access
    // explicit.
    let table = unsafe {
        let pool = ptr::addr_of_mut!(EARLY_TABLE_POOL) as *mut PageTable;
        pool.add(index)
    };

    // SAFETY: `table` points to one 4 KiB page table from the static pool.
    unsafe {
        ptr::write_bytes(table as *mut u8, 0, PAGE_SIZE);
    }

    let table_virtual = table as u64;
    let table_physical = virtual_to_physical_address(table_virtual)
        .map_err(|_| PagingError::TranslationFailed)?;

    crate::kprintln!(
        "[NX][PAGING] allocated early table index={} virtual={:#018x} physical={:#018x}",
        index,
        table_virtual,
        table_physical
    );

    Ok((table, table_physical))
}

/// Returns the active PML4 through the HHDM
fn active_pml4_mut() -> Result<*mut PageTable, PagingError> {
    page_table_from_physical_mut(active_pml4_physical())
}

/// Converts a physical page-table address to a mutable virtual pointer
fn page_table_from_physical_mut(physical: u64) -> Result<*mut PageTable, PagingError> {
    let Some(virtual_address) = addr::physical_to_virtual_address(physical) else {
        return Err(PagingError::NoHhdm);
    };

    Ok(virtual_address as *mut PageTable)
}

/// Translates a virtual address by walking from a PML4 pointer
/// 
/// # Safety
/// `pml4 must point to a valid active PML4 table.
unsafe fn translate_from_pml4(pml4: *mut PageTable, virtual_address: u64) -> Result<u64, PagingError> {
    let pml4_index = page_table_index(virtual_address, 39);
    let pdpt_index = page_table_index(virtual_address, 30);
    let pd_index = page_table_index(virtual_address, 21);
    let pt_index = page_table_index(virtual_address, 12);

    // SAFETY: Caller guarantees `pml4` is valid.
    let pml4_entry = unsafe {
        (*pml4).entries[pml4_index]
    };

    if (pml4_entry & PTE_PRESENT) == 0 {
        return Err(PagingError::NotMapped);
    }

    let pdpt = page_table_from_physical_mut(pml4_entry & PTE_ADDR_MASK)?;

    // SAFETY: `pdpt` was obtained from a present page-table entry.
    let pdpt_entry = unsafe {
        (*pdpt).entries[pdpt_index]
    };

    if (pdpt_entry & PTE_PRESENT) == 0 {
        return Err(PagingError::NotMapped);
    }

    if(pdpt_entry & PTE_HUGE_PAGE) != 0 {
        let base = pdpt_entry & PTE_ADDR_MASK;
        let offset = virtual_address & 0x3fff_ffff;
        return Ok(base + offset);
    }

    let pd = page_table_from_physical_mut(pdpt_entry & PTE_ADDR_MASK)?;

    // SAFETY: `pd` was obtained from a present page-table entry.
    let pd_entry = unsafe {
        (*pd).entries[pd_index]
    };

    if(pd_entry & PTE_HUGE_PAGE) != 0 {
        let base = pd_entry & PTE_ADDR_MASK;
        let offset = virtual_address & 0x1f_ffff;
        return Ok(base + offset);
    }

    let pt = page_table_from_physical_mut(pd_entry & PTE_ADDR_MASK)?;

    // SAFETY: `pt` was obtained from a present page-table entry.
    let pt_entry = unsafe {
        (*pt).entries[pt_index]
    };

    if(pt_entry & PTE_PRESENT) == 0 {
        return Err(PagingError::NotMapped);
    }

    let base = pt_entry & PTE_ADDR_MASK;
    let offset = virtual_address & 0xfff;

    Ok(base + offset)
}

fn page_table_index(virtual_address: u64, shift: u8) -> usize {
    ((virtual_address >> shift) & 0x1ff) as usize
}

/// Aligns an address down to a 4 KiB boundary
fn align_down_4k(address: u64) -> u64 {
    address & !0xfff
}