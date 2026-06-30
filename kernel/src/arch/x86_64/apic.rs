//! Local APIC support for x86_64
//! 
//! The local APIC is the modern per-CPU interrupt controller used for local
//! interrupts, inter-processor interrupts, and APIC timer interrupts.
//! 
//! This module starts conservatively:
//! - it reads the `IA32_APIC_BASE` MSR
//! - logs the Local APIC physical base address
//! - records whether the APIC appears enabled
//! - provides an explicit EOI helper for later APIC interrupt handling
//! 
//! We do not automatically touch Local APIC MMIO registers during early boot
//! yet, because the kernel does not have a full paging/MMIO mapping layer.
//! Directly dereferencing the physical APIC base saddress can page fault unless
//! that physical region is mapped into the current virtual address space.

#![allow(dead_code)]

use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use super::cpu;

/// IA32_APIC_BASE model-specific register
const IA32_APIC_BASE_MSR: u32 = 0x1b;

/// Bit indicating that the Local APIC is globally enabled
const IA32_APIC_BASE_ENABLE: u64 = 1 << 11;

/// Bit indicating that this CPU is the bootstrap processor
const IA32_APIC_BASE_BSP: u64 = 1 << 8;

/// Physical base mask for the APIC MMIO base address
/// 
/// The base address is page-aligned and stored in the upper physical layer
/// bits if `IA32_APIC_BASE`.
const IA32_APIC_BASE_ADDR_MASK: u64 = 0x000f_ffff_ffff_f000;

/// Local APIC End-of-Interrupt register offset
const APIC_EOI_OFFSET: usize = 0x0b0;

/// Local APIC Spurious Interrupt Vector Register offset
const APIC_SPURIOUS_INTERRUPT_VECTOR_OFFSET: usize = 0x0f0;

/// Software enable bit inside the Spurious Interrupt Vector Register
const APIC_SPURIOUS_SOFTWARE_ENABLE: u32 = 1 << 8;

/// Spurious interrupt vector reserved for future APIC setup
/// 
/// We keep this away from CPU exception vectors and away from the legacy PIC
/// range.
pub const APIC_SPURIOUS_VECTOR: u8 = 0xff;

/// Cached Local APIC physical base address
static LOCAL_APIC_BASE_PHYSICAL: AtomicU64 = AtomicU64::new(0);

/// Whether the IA32_APIC_BASE MSR reported the APIC as globally enabled
static LOCAL_APIC_ENABLED_BY_MSR: AtomicBool = AtomicBool::new(false);

/// Minimal decoded APIC base information
#[derive(Clone, Copy)]
pub struct ApicBaseInfo {
    /// Raw IA32_APIC_BASE MSR value
    pub raw: u64,

    /// Physical Local APIC MMIO base address
    pub physical_base: u64,

    /// Whether the CPU reports that the Local APIC is globally enabled
    pub enabled: bool,

    /// Whether this CPU is the bootstrap processor
    pub bootstrap_processor: bool
}

/// Reads and decodes the Local APIC base MSR
/// 
/// # Safety
/// This reads `IA32_APIC_BASE`, which is expected to exist on x86_64 CPUs used
/// by our target environment. If run on a CPU without this MSR, a general
/// protection fault could occur.
pub unsafe fn read_base_info() -> ApicBaseInfo {
    // SAFETY: The caller accepts the MSR availability requirement. x86_64 PC
    // environments used by QEMU expose IA32_APIC_BASE.
    let raw = unsafe {
        cpu::read_msr(IA32_APIC_BASE_MSR)
    };

    ApicBaseInfo {
        raw,
        physical_base: raw & IA32_APIC_BASE_ADDR_MASK,
        enabled: (raw & IA32_APIC_BASE_ENABLE) != 0,
        bootstrap_processor: (raw & IA32_APIC_BASE_BSP) != 0
    }
}

/// Probes the Local APIC base state without touching APIC MMIO registers
/// 
/// This is dafe to call during the current early boot phase because it only
/// reads the APIC base MSR and logs decoded state.
pub fn init_probe() {
    crate::kprintln!("[NX][APIC] probing Local APIC");

    // SAFETY: We are running in the x86_64 kernel on QEMU/PC-like hardware,
    // where IA32_APIC_BASE is expected to exist. This function only reads the
    // MSR and does not dereference the returned MMIO base.
    let info = unsafe {
        read_base_info()
    };

    LOCAL_APIC_BASE_PHYSICAL.store(info.physical_base, Ordering::Relaxed);
    LOCAL_APIC_ENABLED_BY_MSR.store(info.enabled, Ordering::Relaxed);

    crate::kprintln!("[NX][APIC] IA32_APIC_BASE={:#018x}", info.raw);
    crate::kprintln!(
        "[NX][APIC] physical_base={:#018x} enabled={} bsp={}",
        info.physical_base,
        info.enabled,
        info.bootstrap_processor
    );

    crate::kprintln!("[NX][APIC] MMIO register access deferred until mapping is available");
}

/// Returns the cached Local APIC physical base address
pub fn physical_base() -> u64 {
    LOCAL_APIC_BASE_PHYSICAL.load(Ordering::Relaxed)
}

/// Returns whether the APIC was reported as enabled by `IA32_APIC_BASE`
pub fn enabled_by_msr() -> bool {
    LOCAL_APIC_ENABLED_BY_MSR.load(Ordering::Relaxed)
}

/// Writes a Local APIC register through an already-mapped virtual base address
/// 
/// # Safety
/// `apic_virtual_base` must point to a valid virtual mapping of the Local APIC
/// MMIO page. The mapping must be uncached pr otherwhise configured according to
/// CPU/APIC requirements. The caller must ensure the offset is a valid APIC
/// register offset.
pub unsafe fn write_register(apic_virtual_base: *mut u8, offset: usize, value: u32) {
    // SAFETY: The caller guarantees that `apic_virtual_base + offset` is a
    // valid mapped Local APIC register address
    unsafe {
        ptr::write_volatile(apic_virtual_base.add(offset) as *mut u32, value)
    }
}

/// Reads a Local APIC register through an already-mapped virtual base address
/// 
/// # Safety
/// `apic_virtual_base` must point to a valid virtual mapping of the Local APIC
/// MMIO page. The caller must ensure the offset is a valid APIC register
/// offset.
pub unsafe fn read_register(apic_virtual_base: *mut u8, offset: usize) -> u32 {
    // SAFETY: The caller guarantees that `apic_virtual_base + offset` is a
    // valid mapped Local APIC register address.
    unsafe {
        ptr::read_volatile(apic_virtual_base.add(offset) as *const u32)
    }
}

/// Sends a Local APIC End-of-Interrupt
/// 
/// This requires a valid virtual mapping of the Local APIC MMIO page.
/// 
/// # Safety
/// `apic_virtual_base` must be a valid mapped Local APIC base. Calling this
/// without a valid mapping will page fault.
pub unsafe fn send_eoi_mapped(apic_virtual_base: *mut u8) {
    // SAFETY: The caller guarantees that the APIC MMIO base is mapped
    unsafe {
        write_register(apic_virtual_base, APIC_EOI_OFFSET, 0);
    }
}

/// Enables the Local APIC through the Spurious Interrupt Vector Register
/// 
/// This helper is intentionally not called automatically yet. It requires an
/// already-mapped Local APIC MMIO page.
/// 
/// # Safety
/// `apic_virtual_base` must point to a valid mapped Local APIC MMIO page.
pub unsafe fn enable_software_mapped(apic_virtual_base: *mut u8) {
    // SAFETY: The caller guarantees that the APIC MMIO base is mapped
    let current = unsafe {
        read_register(apic_virtual_base, APIC_SPURIOUS_INTERRUPT_VECTOR_OFFSET)
    };

    let new_value =
        current |
        APIC_SPURIOUS_SOFTWARE_ENABLE |
        APIC_SPURIOUS_VECTOR as u32;

    // SAFETY: The caller guarantees that the APIC MMIO base is mapped.
    unsafe {
        write_register(
            apic_virtual_base,
            APIC_SPURIOUS_INTERRUPT_VECTOR_OFFSET,
            new_value
        );
    }
}