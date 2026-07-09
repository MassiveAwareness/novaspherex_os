//! Minimal x86_64 paging helpers
//! 
//! This module is not a full memory manager. It currently supports the v0.0.6
//! transition from a static early page-table pool to real physical frames
//! provided by the early frame allocator.
//! 
//! The implementation assumes 4-level paging and uses Limine's HHDM offset to
//! access physical page tables through virtual addresses.
//! 
//! Current responsibilities:
//! - read active CR3
//! - walk active page tables
//! - translate existing virtual addresses to physical addresses
//! - allocate new page-table frames through the physical frame allocator
//! - map one 4 KiB page
//! - map one MMIO page for the Local APIC

#![allow(dead_code)]

use core::ptr;

use super::{addr, apic, cpu};
use crate::memory::frame_allocator;

/// Size of one x86_64 page
const PAGE_SIZE: usize = 4096;

/// Page table entry present bit
const PTE_PRESENT: u64 = 1 << 0;

/// Page table entry writable bit
const PTE_WRITABLE: u64 = 1 << 1;

/// Page table entry write-through bit
const PTE_WRITE_THROUGH: u64 = 1 << 3;

/// Page-level cache-disable bit
/// 
/// For MMIO, this prevents ordinary cacheable memory semantics
const PTE_CACHE_DISABLE: u64 = 1 << 4;

/// Page size bit used by 1 GiB and 2 MiB mappings
const PTE_HUGE_PAGE: u64 = 1 << 7;

/// Address mask for page table entries
const PTE_ADDR_MASK: u64 = 0x000f_ffff_ffff_f000;

/// Physical address mask for CR3
const CR3_ADDR_MASK: u64 = 0x000f_ffff_ffff_f000;

/// Flags used for intermediate page-table entries
const TABLE_FLAGS: u64 = PTE_PRESENT | PTE_WRITABLE;

/// Flags used for the Local APIC MMIO page
const MMIO_PAGE_FLAGS: u64 = PTE_PRESENT | PTE_WRITABLE | PTE_WRITE_THROUGH | PTE_CACHE_DISABLE;

/// One 4 KiB x86_64 page table
#[repr(C, align(4096))]
struct PageTable {
    entries: [u64; 512]
}

/// Errors produced by the minimal paging helpers
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PagingError {
    /// Limine did not provide an HHDM offset
    NoHhdm,

    /// The frame allocator was not initialized before paging needed a frame
    FrameAllocatorUnavailable,

    /// The frame allocator could not provide another physical frame
    OutOfPhysicalFrames,

    /// A page table entry required for a walk was not present
    NotMapped,

    /// The walk encountered a huge page where a normal page table was expected
    HugePageConflict
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
/// 
/// ```text
/// physical 0xfee00000 -> virtual hhdm_offset + 0xfee00000
/// ```
/// 
/// The mapping uses present, writable, write_through and cache-disable flags.
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
/// This function updates the active page tables directly.
pub fn map_mmio_page(virtual_address: u64, physical_address: u64) -> Result<(), PagingError> {
    let virtual_page = align_down_4k(virtual_address);
    let physical_page = align_down_4k(physical_address);

    // SAFETY: This mutates the active page tables during controlled early boot.
    // Missing intermediate page tables are allocated from the early physical
    // frame allocator and zeroed before being linked into the hierarchy.
    unsafe {
        map_page_4k(virtual_page, physical_page, MMIO_PAGE_FLAGS)?;
    }

    cpu::invlpg(virtual_page);

    crate::kprintln!("[NX][PAGING] MMIO page mapped");

    Ok(())
}

/// Translates a virtual address to a physical address using the active tables
/// 
/// This supports 4 KiB, 2 MiB and 1 GiB mappings.
pub fn virtual_to_physical_address(virtual_address: u64) -> Result<u64, PagingError> {
    let pml4 = active_pml4_mut()?;

    // SAFETY: `pml4` is derived from the active from the active CR3 physical address and
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

    let pdpt = unsafe { ensure_next_table(pml4, pml4_index)? };
    let pd = unsafe { ensure_next_table(pdpt, pdpt_index)? };
    let pt = unsafe { ensure_next_table(pd, pd_index)? };

    // SAFETY: `pt` points to a valid page table ensured above.
    let entry = unsafe {
        &mut (*pt).entries[pt_index]
    };

    if (*entry & PTE_PRESENT) != 0 {
        crate::kprintln!("[NX][PAGING] replacing existing PTE for virtual={:#018x}", virtual_address);
    }

    *entry = (physical_address & PTE_ADDR_MASK) | flags;

    Ok(())
}

/// Ensures that `table[index]` points to a normal next-level page table
/// 
/// If the entry is missing, a new page-table frame is allocated from the early
/// physical frame allocator.
/// 
/// # Safety
/// `table` must point to a valid mutable page table in the active paging hierarchy.
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

    let (new_table, new_table_physical) = allocate_page_table_frame()?;

    *entry = (new_table_physical & PTE_ADDR_MASK) | TABLE_FLAGS;

    Ok(new_table)
}

/// Allocates and zeroes one physical frame for use as a page table
fn allocate_page_table_frame() -> Result<(*mut PageTable, u64), PagingError> {
    if !frame_allocator::is_initialized() {
        return Err(PagingError::FrameAllocatorUnavailable);
    }

    let Some(frame) = frame_allocator::allocate_frame() else {
        return Err(PagingError::OutOfPhysicalFrames);
    };

    let table = page_table_from_physical_mut(frame.start)?;

    // SAFETY: The physical frame was just handed out by the frame allocator and
    // is intended to become a page table. HHDM is used to access the frame as
    // memory before linking it into the active page-table hierarchy.
    unsafe {
        ptr::write_bytes(table as *mut u8, 0, PAGE_SIZE);
    }

    crate::kprintln!(
        "[NX][PAGING] allocated page table frame physical={:#018x} virtual={:#018x}",
        frame.start,
        table as u64
    );

    Ok((table, frame.start))
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
/// `pml4` must point to a valid active PML4 table.
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

    if (pdpt_entry &PTE_HUGE_PAGE) != 0 {
        let base = pdpt_entry & PTE_ADDR_MASK;
        let offset = virtual_address & 0x3fff_ffff;

        return Ok(base + offset);
    }

    let pd = page_table_from_physical_mut(pdpt_entry & PTE_ADDR_MASK)?;

    // SAFETY: `pd` was obtained from a present page-table entry.
    let pd_entry = unsafe {
        (*pd).entries[pd_index]
    };

    if (pd_entry & PTE_PRESENT) == 0 {
        return Err(PagingError::NotMapped);
    }

    if (pd_entry & PTE_HUGE_PAGE) != 0 {
        let base = pd_entry & PTE_ADDR_MASK;
        let offset = virtual_address & 0x1f_ffff;

        return Ok(base + offset);
    }

    let pt = page_table_from_physical_mut(pd_entry & PTE_ADDR_MASK)?;

    // SAFETY: `pt` was obtained from a present page-table entry.
    let pt_entry = unsafe {
        (*pt).entries[pt_index]
    };

    if (pt_entry & PTE_PRESENT) == 0 {
        return Err(PagingError::NotMapped);
    }

    let base = pt_entry & PTE_ADDR_MASK;
    let offset = virtual_address & 0xfff;

    Ok(base + offset)
}

/// Returns a page-table index for a virtual address
fn page_table_index(virtual_address: u64, shift: u8) -> usize {
    ((virtual_address >> shift) & 0x1ff) as usize
}

/// Aligns an address down to a 4 KiB boundary
fn align_down_4k(address: u64) -> u64 {
    address & !0xfff
}