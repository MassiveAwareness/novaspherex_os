//! Architecture abstraction layer
//! 
//! This module exposes architecture-neutral entry points used by the rest of
//! the kernel. The current implementation targets x86_64, but higher-level
//! code should call this module instead of directly depending on
//! `arch::x86_64`.

#![allow(dead_code)]

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

/// Initializes architecture-specific CPU state
pub fn init() {
    x86_64::init();
}

/// Runs the architecture-specific breakpoint exception smoke test
pub fn test_breakpoint() {
    x86_64::test_breakpoint();
}

/// Runst the architecture-specific page fault smoke test
/// 
/// This test is fatal and should only be enabled during controller debugging.
pub fn test_page_fault() {
    x86_64::test_page_fault();
}

/// Enters the architecture-specific halt loop
pub fn halt_loop() -> ! {
    x86_64::halt_loop();
}

/// Enters a fatal halt loop after disabling maskable interrupts
pub fn panic_halt_loop() -> ! {
    x86_64::panic_halt_loop();
}

/// Runs the architecture-specific timer interrupt vector smoke test
/// 
/// This test invokes vector 32 through a software interrupt. Is does not test
/// physical PIT/PIC delivery.
pub fn test_timer_interrupt() {
    x86_64::test_timer_interrupt();
}

/// Logs architecture-specific CPU state for diagnostics
pub fn log_cpu_state(label: &str) {
    x86_64::log_cpu_state(label);
}