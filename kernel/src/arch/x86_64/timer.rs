//! Early timer interrupt handling
//! 
//! This module implements the Rust side of the timer interrupt path.
//! 
//! The assembly timer interrupt stub calls `nx_timer_handler`, which increments
//! a global tick counter and then acknowledges the interrupt through the
//! currently selected End-of-Interrupt backend.
//! 
//! At the moment, the legacy PIC backend is the default. AOUC EOI support is
//! represented as an explicit backend, but it should only be selected once the
//! Local APIC MMIO page is mapped and the APIC timer is actually used.

#![allow(dead_code)]

use core::sync::atomic::{AtomicU8, AtomicU64, Ordering};

use super::pic;

/// Timer frequency selected for the first timer draft
pub const TIMER_FREQUENCY_HZ: u32 = 100;

/// Legacy PIC End-of-Interrupt backend
const EOI_BACKEND_PIC: u8 = 0;

/// Local APIC End-of-Interrupt backend
/// 
/// This backend is reserved for the APIC timer path. It should not be selected
/// until the Local APIC MMIO region is mapped and APIC EOI writes are wired up.
const EOI_BACKEND_LOCAL_APIC: u8 = 1;

/// Global timer tick counter
/// 
/// This is incremented from the timer interrupt handler. Atomic access keeps
/// the value safe to read from normal kernel code and interrupt context without
/// introducing a lock.
static TIMER_TICKS: AtomicU64 = AtomicU64::new(0);

/// Current timer EOI backend
static TIMER_EOI_BACKEND: AtomicU8 = AtomicU8::new(EOI_BACKEND_PIC);

/// Timer EOI backend selection
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EoiBackend {
    /// Acknowledge timer interrupts through the legacy 8259 PIC
    LegacyPic,

    /// Acknowledge timer interrupts through the Local APIC
    ///
    /// This requires APIC MMIO support before it can be fully implemented.
    LocalApic
}

/// Returns the number of timer ticks since timer interrupts were enabled
pub fn ticks() -> u64 {
    TIMER_TICKS.load(Ordering::Relaxed)
}

/// Selects which interrupt controller receives timer EOI commands
pub fn set_eoi_backend(backend: EoiBackend) {
    let raw = match backend {
        EoiBackend::LegacyPic => EOI_BACKEND_PIC,
        EoiBackend::LocalApic => EOI_BACKEND_LOCAL_APIC
    };

    TIMER_EOI_BACKEND.store(raw, Ordering::Relaxed);
    crate::kprintln!("[NX][TIMER] EOI backend set to {:?}", backend);
}

/// Returns the currently configured timer EOI backend
pub fn eoi_backend() -> EoiBackend {
    match TIMER_EOI_BACKEND.load(Ordering::Relaxed) {
        EOI_BACKEND_LOCAL_APIC => EoiBackend::LocalApic,
        _ => EoiBackend::LegacyPic
    }
}

/// Sends an End-of-Interrupt command for the current timer source
/// 
/// The legacy PIC backend is fully implemented. The Local APIC backend is
/// intentionally a placeholder until APIC MMIO mapping exists.
fn send_timer_eoi() {
    match eoi_backend() {
        EoiBackend::LegacyPic => pic::send_eoi(pic::IRQ_TIMER),
        EoiBackend::LocalApic => {
            // APIC EOI requires writing to the Local APIC EOI MMIO register.
            // We intentionally do not do that here yet because the current
            // kernel has not established a safe APIC MMIO mapping.
            //
            // The Local APIC timer implementation will replace this placeholder
            // once APIC MMIO access is available.
        }
    }
}

/// Rust entry point for timer interrupts
/// 
/// This function is called from the assembly ISR. It must stay short: interrupt
/// handlers run with interrupts disabled because we use interrupt gates.
/// 
/// During early bring-up, we log the first few ticks as proof that the timer
/// path reaches the kernel. After that, we only log once per second.
#[no_mangle]
extern "C" fn nx_timer_handler() {
    let tick = TIMER_TICKS.fetch_add(1, Ordering::Relaxed) + 1;

    if tick <= 5 || tick % TIMER_FREQUENCY_HZ as u64 == 0 {
        crate::kprintln!("[NX][TIMER] tick {}", tick);
    }

    send_timer_eoi();
}

/// Spins briefly to check whether hardware timer interrupts arrive
/// 
/// This is a temporary diagnostic helper. Interrupts must already be enabled
/// before this function is called. If PIT/PIC IRQ0 delivery works, the global
/// tick counter should increase while this function spins.
pub fn debug_wait_for_hardware_ticks() {
    let before = ticks();

    crate::kprintln!("[NX][TIMER] waiting for hardware ticks, start={}", before);
    for _ in 0..20_000_000u64 {
        core::hint::spin_loop();
    }

    let after = ticks();

    crate::kprintln!(
        "[NX][TIMER] hardware tick wait complete, start={}, end={}",
        before,
        after
    );

    super::pic::log_irq_state("after timer wait");
}