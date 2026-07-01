//! Early timer interrupt handling.
//!
//! This module implements the Rust side of the timer interrupt path.
//!
//! The assembly timer interrupt stub calls `nx_timer_handler`, which increments
//! a global tick counter and then acknowledges the interrupt through the
//! currently selected End-of-Interrupt backend.

#![allow(dead_code)]

use core::sync::atomic::{AtomicU8, AtomicU64, Ordering};

use super::{apic, pic};

/// Timer vector used by both the legacy PIT/PIC draft and the Local APIC timer.
pub const TIMER_VECTOR: u8 = 32;

/// Timer frequency selected for the legacy PIT draft.
pub const TIMER_FREQUENCY_HZ: u32 = 100;

/// Experimental Local APIC timer initial count.
///
/// This is not calibrated yet. The goal is only to prove delivery.
pub const APIC_TIMER_INITIAL_COUNT: u32 = 1_000_000;

/// Legacy PIC End-of-Interrupt backend.
const EOI_BACKEND_PIC: u8 = 0;

/// Local APIC End-of-Interrupt backend.
const EOI_BACKEND_LOCAL_APIC: u8 = 1;

/// Global timer tick counter.
static TIMER_TICKS: AtomicU64 = AtomicU64::new(0);

/// Current timer EOI backend.
static TIMER_EOI_BACKEND: AtomicU8 = AtomicU8::new(EOI_BACKEND_PIC);

/// Timer EOI backend selection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EoiBackend {
    /// Acknowledge timer interrupts through the legacy 8259 PIC.
    LegacyPic,

    /// Acknowledge timer interrupts through the Local APIC.
    LocalApic,
}

/// Returns the number of timer ticks since timer interrupts were enabled.
pub fn ticks() -> u64 {
    TIMER_TICKS.load(Ordering::Relaxed)
}

/// Selects which interrupt controller receives timer EOI commands.
pub fn set_eoi_backend(backend: EoiBackend) {
    let raw = match backend {
        EoiBackend::LegacyPic => EOI_BACKEND_PIC,
        EoiBackend::LocalApic => EOI_BACKEND_LOCAL_APIC,
    };

    TIMER_EOI_BACKEND.store(raw, Ordering::Relaxed);

    crate::kprintln!("[NX][TIMER] EOI backend set to {:?}", backend);
}

/// Selects the best currently available EOI backend.
pub fn select_best_eoi_backend() {
    if apic::mmio_ready() {
        set_eoi_backend(EoiBackend::LocalApic);
    } else {
        set_eoi_backend(EoiBackend::LegacyPic);
    }
}

/// Returns the currently configured timer EOI backend.
pub fn eoi_backend() -> EoiBackend {
    match TIMER_EOI_BACKEND.load(Ordering::Relaxed) {
        EOI_BACKEND_LOCAL_APIC => EoiBackend::LocalApic,
        _ => EoiBackend::LegacyPic,
    }
}

/// Sends an End-of-Interrupt command for the current timer source.
fn send_timer_eoi() {
    match eoi_backend() {
        EoiBackend::LegacyPic => {
            pic::send_eoi(pic::IRQ_TIMER);
        }
        EoiBackend::LocalApic => {
            apic::send_eoi();
        }
    }
}

/// Initializes the best available hardware timer source.
///
/// If Local APIC MMIO is ready, this configures the Local APIC timer. Otherwise
/// the kernel falls back to the legacy PIT/PIC draft.
pub fn init_hardware_timer() {
    if apic::mmio_ready() {
        crate::kprintln!("[NX][TIMER] using Local APIC timer");

        set_eoi_backend(EoiBackend::LocalApic);

        if apic::init_timer_periodic(TIMER_VECTOR, APIC_TIMER_INITIAL_COUNT) {
            apic::log_timer_state("after APIC timer init");
            return;
        }

        crate::kprintln!("[NX][TIMER] APIC timer init failed, falling back to PIT/PIC");
    }

    crate::kprintln!("[NX][TIMER] using legacy PIT/PIC timer draft");

    set_eoi_backend(EoiBackend::LegacyPic);
    pic::init();
    super::pit::init(TIMER_FREQUENCY_HZ);
}

/// Rust entry point for timer interrupts.
#[no_mangle]
extern "C" fn nx_timer_handler() {
    let tick = TIMER_TICKS.fetch_add(1, Ordering::Relaxed) + 1;

    if tick <= 5 || tick % TIMER_FREQUENCY_HZ as u64 == 0 {
        crate::kprintln!("[NX][TIMER] tick {}", tick);
    }

    send_timer_eoi();
}

/// Spins briefly to check whether hardware timer interrupts arrive.
pub fn debug_wait_for_hardware_ticks() {
    let before = ticks();

    crate::kprintln!(
        "[NX][TIMER] waiting for hardware ticks, start={}",
        before
    );

    for _ in 0..20_000_000u64 {
        core::hint::spin_loop();
    }

    let after = ticks();

    crate::kprintln!(
        "[NX][TIMER] hardware tick wait complete, start={}, end={}",
        before,
        after
    );

    match eoi_backend() {
        EoiBackend::LegacyPic => {
            super::pic::log_irq_state("after timer wait");
        }
        EoiBackend::LocalApic => {
            super::apic::log_timer_state("after timer wait");
        }
    }
}