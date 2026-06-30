//! Local APIC support for x86_64.
//!
//! The Local APIC is the modern per-CPU interrupt controller used for local
//! interrupts, inter-processor interrupts, and APIC timer interrupts.
//!
//! This module currently provides:
//!
//! - IA32_APIC_BASE MSR probing,
//! - Local APIC physical base logging,
//! - HHDM-based virtual base calculation,
//! - optional MMIO software-enable experiment,
//! - APIC EOI helper for the future APIC timer path.
//!
//! The MMIO write path is intentionally guarded by
//! `ENABLE_HHDM_APIC_MMIO_EXPERIMENT`, because Limine's HHDM offset does not by
//! itself prove that the Local APIC MMIO physical page is mapped.

#![allow(dead_code)]

use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

use super::{addr, cpu};

/// Whether to attempt Local APIC MMIO writes through the HHDM-derived address.
///
/// Keep this `false` by default. If the APIC MMIO page is not mapped, enabling
/// this can cause a page fault.
const ENABLE_HHDM_APIC_MMIO_EXPERIMENT: bool = false;

/// IA32_APIC_BASE model-specific register.
const IA32_APIC_BASE_MSR: u32 = 0x1b;

/// Bit indicating that the Local APIC is globally enabled.
const IA32_APIC_BASE_ENABLE: u64 = 1 << 11;

/// Bit indicating that this CPU is the bootstrap processor.
const IA32_APIC_BASE_BSP: u64 = 1 << 8;

/// Physical base mask for the APIC MMIO base address.
const IA32_APIC_BASE_ADDR_MASK: u64 = 0x000f_ffff_ffff_f000;

/// Local APIC End-of-Interrupt register offset.
const APIC_EOI_OFFSET: usize = 0x0b0;

/// Local APIC Spurious Interrupt Vector Register offset.
const APIC_SPURIOUS_INTERRUPT_VECTOR_OFFSET: usize = 0x0f0;

/// Software enable bit inside the Spurious Interrupt Vector Register.
const APIC_SPURIOUS_SOFTWARE_ENABLE: u32 = 1 << 8;

/// Spurious interrupt vector reserved for APIC setup.
pub const APIC_SPURIOUS_VECTOR: u8 = 0xff;

/// Cached Local APIC physical base address.
static LOCAL_APIC_BASE_PHYSICAL: AtomicU64 = AtomicU64::new(0);

/// Cached Local APIC virtual base candidate.
static LOCAL_APIC_BASE_VIRTUAL: AtomicUsize = AtomicUsize::new(0);

/// Whether the IA32_APIC_BASE MSR reported the APIC as globally enabled.
static LOCAL_APIC_ENABLED_BY_MSR: AtomicBool = AtomicBool::new(false);

/// Whether APIC MMIO writes were enabled by our kernel.
static LOCAL_APIC_MMIO_READY: AtomicBool = AtomicBool::new(false);

/// Minimal decoded APIC base information.
#[derive(Clone, Copy)]
pub struct ApicBaseInfo {
    /// Raw IA32_APIC_BASE MSR value.
    pub raw: u64,

    /// Physical Local APIC MMIO base address.
    pub physical_base: u64,

    /// Whether the CPU reports that the Local APIC is globally enabled.
    pub enabled: bool,

    /// Whether this CPU is the bootstrap processor.
    pub bootstrap_processor: bool,
}

/// Reads and decodes the Local APIC base MSR.
///
/// # Safety
/// This reads `IA32_APIC_BASE`, which is expected to exist on x86_64 CPUs used
/// by our target environment. If run on a CPU without this MSR, a general
/// protection fault could occur.
pub unsafe fn read_base_info() -> ApicBaseInfo {
    // SAFETY: The caller accepts the MSR availability requirement.
    let raw = unsafe { cpu::read_msr(IA32_APIC_BASE_MSR) };

    ApicBaseInfo {
        raw,
        physical_base: raw & IA32_APIC_BASE_ADDR_MASK,
        enabled: (raw & IA32_APIC_BASE_ENABLE) != 0,
        bootstrap_processor: (raw & IA32_APIC_BASE_BSP) != 0,
    }
}

/// Probes the Local APIC and prepares cached address information.
pub fn init_probe() {
    crate::kprintln!("[NX][APIC] probing Local APIC");

    // SAFETY: We are running in the x86_64 kernel on QEMU/PC-like hardware,
    // where IA32_APIC_BASE is expected to exist.
    let info = unsafe { read_base_info() };

    LOCAL_APIC_BASE_PHYSICAL.store(info.physical_base, Ordering::Relaxed);
    LOCAL_APIC_ENABLED_BY_MSR.store(info.enabled, Ordering::Relaxed);

    crate::kprintln!("[NX][APIC] IA32_APIC_BASE={:#018x}", info.raw);
    crate::kprintln!(
        "[NX][APIC] physical_base={:#018x} enabled={} bsp={}",
        info.physical_base,
        info.enabled,
        info.bootstrap_processor
    );

    match local_apic_virtual_base_candidate() {
        Some(virtual_base) => {
            LOCAL_APIC_BASE_VIRTUAL.store(virtual_base as usize, Ordering::Relaxed);
            crate::kprintln!("[NX][APIC] virtual_base_candidate={:#018x}", virtual_base);
        }
        None => {
            crate::kprintln!("[NX][APIC] virtual_base_candidate unavailable");
        }
    }

    try_init_mmio();
}

/// Returns the cached Local APIC physical base address.
pub fn physical_base() -> u64 {
    LOCAL_APIC_BASE_PHYSICAL.load(Ordering::Relaxed)
}

/// Returns the cached Local APIC virtual base candidate.
pub fn virtual_base_candidate() -> Option<*mut u8> {
    let value = LOCAL_APIC_BASE_VIRTUAL.load(Ordering::Relaxed);

    if value == 0 {
        None
    } else {
        Some(value as *mut u8)
    }
}

/// Returns whether the APIC was reported as enabled by `IA32_APIC_BASE`.
pub fn enabled_by_msr() -> bool {
    LOCAL_APIC_ENABLED_BY_MSR.load(Ordering::Relaxed)
}

/// Returns whether APIC MMIO access is enabled by our kernel.
pub fn mmio_ready() -> bool {
    LOCAL_APIC_MMIO_READY.load(Ordering::Relaxed)
}

/// Computes the Local APIC HHDM virtual base candidate.
pub fn local_apic_virtual_base_candidate() -> Option<u64> {
    addr::physical_to_virtual_address(physical_base())
}

/// Attempts to initialize Local APIC MMIO support.
///
/// In safe default mode, this only logs that MMIO writes are disabled. If
/// `ENABLE_HHDM_APIC_MMIO_EXPERIMENT` is set to `true`, this attempts to enable
/// the Local APIC through the Spurious Interrupt Vector Register and performs a
/// test EOI write.
pub fn try_init_mmio() -> bool {
    if !enabled_by_msr() {
        crate::kprintln!("[NX][APIC] MMIO init skipped: APIC disabled by MSR");
        return false;
    }

    let Some(base) = virtual_base_candidate() else {
        crate::kprintln!("[NX][APIC] MMIO init skipped: no virtual base candidate");
        return false;
    };

    if !ENABLE_HHDM_APIC_MMIO_EXPERIMENT {
        crate::kprintln!("[NX][APIC] MMIO writes disabled by safety gate");
        crate::kprintln!("[NX][APIC] set ENABLE_HHDM_APIC_MMIO_EXPERIMENT=true to test");
        return false;
    }

    crate::kprintln!("[NX][APIC] attempting HHDM MMIO software enable");

    // SAFETY: This is explicitly gated as an experiment. If the HHDM candidate
    // does not actually map the Local APIC MMIO page, this may page fault.
    unsafe {
        enable_software_mapped(base);
        send_eoi_mapped(base);
    }

    LOCAL_APIC_MMIO_READY.store(true, Ordering::Relaxed);

    crate::kprintln!("[NX][APIC] MMIO ready");
    true
}

/// Writes a Local APIC register through an already-mapped virtual base address.
///
/// # Safety
/// `apic_virtual_base` must point to a valid virtual mapping of the Local APIC
/// MMIO page. The caller must ensure the offset is a valid APIC register offset.
pub unsafe fn write_register(apic_virtual_base: *mut u8, offset: usize, value: u32) {
    // SAFETY: The caller guarantees that `apic_virtual_base + offset` is valid.
    unsafe {
        ptr::write_volatile(apic_virtual_base.add(offset) as *mut u32, value);
    }
}

/// Reads a Local APIC register through an already-mapped virtual base address.
///
/// # Safety
/// `apic_virtual_base` must point to a valid virtual mapping of the Local APIC
/// MMIO page.
pub unsafe fn read_register(apic_virtual_base: *mut u8, offset: usize) -> u32 {
    // SAFETY: The caller guarantees that `apic_virtual_base + offset` is valid.
    unsafe { ptr::read_volatile(apic_virtual_base.add(offset) as *const u32) }
}

/// Sends a Local APIC End-of-Interrupt command through a mapped APIC base.
///
/// # Safety
/// `apic_virtual_base` must be a valid mapped Local APIC base.
pub unsafe fn send_eoi_mapped(apic_virtual_base: *mut u8) {
    // SAFETY: The caller guarantees that the APIC MMIO base is mapped.
    unsafe {
        write_register(apic_virtual_base, APIC_EOI_OFFSET, 0);
    }
}

/// Sends a Local APIC EOI if APIC MMIO support is ready.
pub fn send_eoi() {
    if !mmio_ready() {
        return;
    }

    let Some(base) = virtual_base_candidate() else {
        return;
    };

    // SAFETY: `mmio_ready` is only set after successful experimental APIC MMIO
    // initialization.
    unsafe {
        send_eoi_mapped(base);
    }
}

/// Enables the Local APIC through the Spurious Interrupt Vector Register.
///
/// # Safety
/// `apic_virtual_base` must point to a valid mapped Local APIC MMIO page.
pub unsafe fn enable_software_mapped(apic_virtual_base: *mut u8) {
    // SAFETY: The caller guarantees that the APIC MMIO base is mapped.
    let current = unsafe {
        read_register(apic_virtual_base, APIC_SPURIOUS_INTERRUPT_VECTOR_OFFSET)
    };

    let new_value = current | APIC_SPURIOUS_SOFTWARE_ENABLE | APIC_SPURIOUS_VECTOR as u32;

    // SAFETY: The caller guarantees that the APIC MMIO base is mapped.
    unsafe {
        write_register(
            apic_virtual_base,
            APIC_SPURIOUS_INTERRUPT_VECTOR_OFFSET,
            new_value,
        );
    }
}