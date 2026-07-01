//! Local APIC support for x86_64.
//!
//! The Local APIC is the per-CPU interrupt controller used for local
//! interrupts, inter-processor interrupts, and Local APIC timer interrupts.
//!
//! v0.0.4 attempts the first real Local APIC MMIO validation step:
//!
//! - read `IA32_APIC_BASE`,
//! - compute a HHDM virtual base candidate,
//! - read Local APIC ID/version registers,
//! - enable the Local APIC through the Spurious Interrupt Vector Register,
//! - provide APIC EOI,
//! - configure the Local APIC timer in periodic mode.
//!
//! This is intentionally still experimental. HHDM address calculation does not
//! prove that device MMIO pages are mapped. If the Local APIC MMIO page is not
//! mapped by the current page tables, this module may trigger a page fault.

#![allow(dead_code)]

use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

use super::{addr, cpu};

/// Enables the v0.0.4 HHDM-based Local APIC MMIO experiment.
///
/// If this causes a page fault at the Local APIC virtual base candidate, the
/// next step is a real page-table/MMIO mapping layer.
const ENABLE_HHDM_APIC_MMIO_EXPERIMENT: bool = true;

/// IA32_APIC_BASE model-specific register.
const IA32_APIC_BASE_MSR: u32 = 0x1b;

/// Bit indicating that the Local APIC is globally enabled.
const IA32_APIC_BASE_ENABLE: u64 = 1 << 11;

/// Bit indicating that this CPU is the bootstrap processor.
const IA32_APIC_BASE_BSP: u64 = 1 << 8;

/// Physical base mask for the APIC MMIO base address.
const IA32_APIC_BASE_ADDR_MASK: u64 = 0x000f_ffff_ffff_f000;

/// Local APIC ID register offset.
const APIC_ID_OFFSET: usize = 0x020;

/// Local APIC Version register offset.
const APIC_VERSION_OFFSET: usize = 0x030;

/// Local APIC End-of-Interrupt register offset.
const APIC_EOI_OFFSET: usize = 0x0b0;

/// Local APIC Spurious Interrupt Vector Register offset.
const APIC_SPURIOUS_INTERRUPT_VECTOR_OFFSET: usize = 0x0f0;

/// Local Vector Table Timer register offset.
const APIC_LVT_TIMER_OFFSET: usize = 0x320;

/// Local APIC Timer Initial Count register offset.
const APIC_TIMER_INITIAL_COUNT_OFFSET: usize = 0x380;

/// Local APIC Timer Current Count register offset.
const APIC_TIMER_CURRENT_COUNT_OFFSET: usize = 0x390;

/// Local APIC Timer Divide Configuration register offset.
const APIC_TIMER_DIVIDE_CONFIG_OFFSET: usize = 0x3e0;

/// Software enable bit inside the Spurious Interrupt Vector Register.
const APIC_SPURIOUS_SOFTWARE_ENABLE: u32 = 1 << 8;

/// LVT timer mask bit.
const APIC_LVT_MASKED: u32 = 1 << 16;

/// LVT timer periodic mode bit.
///
/// In xAPIC LVT Timer, timer mode bits are at 17..18. Periodic mode is encoded
/// as `01b` in that field, therefore bit 17 set and bit 18 clear.
const APIC_LVT_TIMER_PERIODIC: u32 = 1 << 17;

/// Divide configuration value for divide-by-16.
///
/// The exact APIC timer rate is platform dependent, so this value is only part
/// of an early bring-up configuration.
const APIC_TIMER_DIVIDE_BY_16: u32 = 0x3;

/// Spurious interrupt vector reserved for APIC setup.
pub const APIC_SPURIOUS_VECTOR: u8 = 0xff;

/// Cached Local APIC physical base address.
static LOCAL_APIC_BASE_PHYSICAL: AtomicU64 = AtomicU64::new(0);

/// Cached Local APIC virtual base candidate.
static LOCAL_APIC_BASE_VIRTUAL: AtomicUsize = AtomicUsize::new(0);

/// Whether the IA32_APIC_BASE MSR reported the APIC as globally enabled.
static LOCAL_APIC_ENABLED_BY_MSR: AtomicBool = AtomicBool::new(false);

/// Whether APIC MMIO access has been validated by the kernel.
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
///
/// This reads `IA32_APIC_BASE`, which is expected to exist on x86_64 CPUs used
/// by the target environment. Reading an unsupported MSR can raise a general
/// protection fault.
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

/// Probes the Local APIC and attempts MMIO validation if enabled.
pub fn init_probe() {
    crate::kprintln!("[NX][APIC] probing Local APIC");

    // SAFETY: We are running in the x86_64 kernel on QEMU/PC-like hardware,
    // where IA32_APIC_BASE is expected to exist.
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

    match local_apic_virtual_base_candidate() {
        Some(virtual_base) => {
            LOCAL_APIC_BASE_VIRTUAL.store(virtual_base as usize, Ordering::Relaxed);
            crate::kprintln!("[NX][APIC] virtual_base_candidate={:#018x}", virtual_base);
        }
        None => {
            crate::kprintln!("[NX][APIC] virtual_base_candidate unavailable");
        }
    }

    crate::kprintln!("[NX][APIC] MMIO validation deferred until paging maps APIC page");
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

/// Returns whether APIC MMIO access is ready for use.
pub fn mmio_ready() -> bool {
    LOCAL_APIC_MMIO_READY.load(Ordering::Relaxed)
}

/// Computes the Local APIC HHDM virtual base candidate.
pub fn local_apic_virtual_base_candidate() -> Option<u64> {
    addr::physical_to_virtual_address(physical_base())
}

/// Attempts to validate and initialize Local APIC MMIO access.
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

    crate::kprintln!("[NX][APIC] attempting HHDM MMIO validation");

    // SAFETY: This is the explicit v0.0.4 experiment. If the HHDM candidate
    // does not map the Local APIC MMIO page, this may page fault. The page fault
    // handler is installed before this function is called.
    unsafe {
        let id = read_register(base, APIC_ID_OFFSET);
        let version = read_register(base, APIC_VERSION_OFFSET);

        crate::kprintln!("[NX][APIC] id_register={:#010x}", id);
        crate::kprintln!("[NX][APIC] version_register={:#010x}", version);

        enable_software_mapped(base);
        send_eoi_mapped(base);
    }

    LOCAL_APIC_MMIO_READY.store(true, Ordering::Relaxed);

    crate::kprintln!("[NX][APIC] MMIO ready");
    true
}

/// Writes a Local APIC register through a mapped virtual base address.
///
/// # Safety
///
/// `apic_virtual_base` must point to a valid virtual mapping of the Local APIC
/// MMIO page. The caller must ensure the offset is a valid APIC register offset.
pub unsafe fn write_register(apic_virtual_base: *mut u8, offset: usize, value: u32) {
    // SAFETY: The caller guarantees that `apic_virtual_base + offset` is valid.
    unsafe {
        ptr::write_volatile(apic_virtual_base.add(offset) as *mut u32, value);
    }
}

/// Reads a Local APIC register through a mapped virtual base address.
///
/// # Safety
///
/// `apic_virtual_base` must point to a valid virtual mapping of the Local APIC
/// MMIO page.
pub unsafe fn read_register(apic_virtual_base: *mut u8, offset: usize) -> u32 {
    // SAFETY: The caller guarantees that `apic_virtual_base + offset` is valid.
    unsafe { ptr::read_volatile(apic_virtual_base.add(offset) as *const u32) }
}

/// Sends a Local APIC End-of-Interrupt command through a mapped APIC base.
///
/// # Safety
///
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
    // validation.
    unsafe {
        send_eoi_mapped(base);
    }
}

/// Enables the Local APIC through the Spurious Interrupt Vector Register.
///
/// # Safety
///
/// `apic_virtual_base` must point to a valid mapped Local APIC MMIO page.
pub unsafe fn enable_software_mapped(apic_virtual_base: *mut u8) {
    // SAFETY: The caller guarantees that the APIC MMIO base is mapped.
    let current = unsafe { read_register(apic_virtual_base, APIC_SPURIOUS_INTERRUPT_VECTOR_OFFSET) };

    let new_value = current | APIC_SPURIOUS_SOFTWARE_ENABLE | APIC_SPURIOUS_VECTOR as u32;

    // SAFETY: The caller guarantees that the APIC MMIO base is mapped.
    unsafe {
        write_register(
            apic_virtual_base,
            APIC_SPURIOUS_INTERRUPT_VECTOR_OFFSET,
            new_value,
        );
    }

    crate::kprintln!(
        "[NX][APIC] spurious vector register: {:#010x} -> {:#010x}",
        current,
        new_value
    );
}

/// Configures the Local APIC timer in periodic mode.
///
/// Returns `true` if the timer was configured, `false` if APIC MMIO is not
/// ready yet.
pub fn init_timer_periodic(vector: u8, initial_count: u32) -> bool {
    if !mmio_ready() {
        crate::kprintln!("[NX][APIC] timer init skipped: MMIO not ready");
        return false;
    }

    let Some(base) = virtual_base_candidate() else {
        crate::kprintln!("[NX][APIC] timer init skipped: no virtual base");
        return false;
    };

    crate::kprintln!(
        "[NX][APIC] configuring Local APIC timer: vector={} initial_count={}",
        vector,
        initial_count
    );

    // SAFETY: `mmio_ready` means the APIC MMIO page was successfully accessed
    // during `try_init_mmio`.
    unsafe {
        write_register(base, APIC_TIMER_INITIAL_COUNT_OFFSET, 0);
        write_register(base, APIC_TIMER_DIVIDE_CONFIG_OFFSET, APIC_TIMER_DIVIDE_BY_16);

        let lvt_timer = vector as u32 | APIC_LVT_TIMER_PERIODIC;
        write_register(base, APIC_LVT_TIMER_OFFSET, lvt_timer);

        send_eoi_mapped(base);
        write_register(base, APIC_TIMER_INITIAL_COUNT_OFFSET, initial_count);

        let current = read_register(base, APIC_TIMER_CURRENT_COUNT_OFFSET);
        crate::kprintln!("[NX][APIC] timer current_count={}", current);
    }

    crate::kprintln!("[NX][APIC] Local APIC timer configured");
    true
}

/// Masks the Local APIC timer.
///
/// This is useful before reconfiguring the timer.
///
/// Returns `false` if APIC MMIO is not available.
pub fn mask_timer() -> bool {
    if !mmio_ready() {
        return false;
    }

    let Some(base) = virtual_base_candidate() else {
        return false;
    };

    // SAFETY: `mmio_ready` means APIC MMIO access was validated.
    unsafe {
        let current = read_register(base, APIC_LVT_TIMER_OFFSET);
        write_register(base, APIC_LVT_TIMER_OFFSET, current | APIC_LVT_MASKED);
    }

    true
}

/// Logs Local APIC timer state.
pub fn log_timer_state(label: &str) {
    if !mmio_ready() {
        crate::kprintln!("[NX][APIC] timer state {}: MMIO not ready", label);
        return;
    }

    let Some(base) = virtual_base_candidate() else {
        crate::kprintln!("[NX][APIC] timer state {}: no virtual base", label);
        return;
    };

    // SAFETY: `mmio_ready` means APIC MMIO access was validated.
    unsafe {
        let lvt = read_register(base, APIC_LVT_TIMER_OFFSET);
        let initial = read_register(base, APIC_TIMER_INITIAL_COUNT_OFFSET);
        let current = read_register(base, APIC_TIMER_CURRENT_COUNT_OFFSET);
        let divide = read_register(base, APIC_TIMER_DIVIDE_CONFIG_OFFSET);

        crate::kprintln!(
            "[NX][APIC] timer state {}: lvt={:#010x} initial={} current={} divide={:#010x}",
            label,
            lvt,
            initial,
            current,
            divide
        );
    }
}