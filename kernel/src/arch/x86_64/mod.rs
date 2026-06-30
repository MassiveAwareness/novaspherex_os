//! x86_64 architecture support
//! 
//! This module coordinates early x86_64 CPU initialization, including the GDT,
//! IDT, exception smoke tests, and CPU helper wrappers.

#![allow(dead_code)]

pub mod gdt;
pub mod idt;
pub mod cpu;
pub mod pic;
pub mod pit;
pub mod apic;
pub mod port;
pub mod timer;
pub mod interrupts;

/// Initializes the early x86_64 CPU baseline
/// 
/// Current initialization order:
/// 
/// 1. log inherited bootloader CPU state
/// 2. load the early GDT
/// 3. load the early IDT
/// 4. remap and mask the legacy PIC
/// 5. configure the PIT timer
/// 6. enable maskable interrupts
pub fn init() {
    crate::kprintln!("[NX][ARCH] x86_64 init begin");

    cpu::log_state("before GDT");

    gdt::init();
    cpu::log_state("after GDT");

    idt::init();
    cpu::log_state("after IDT");

    apic::init_probe();

    pic::init();
    pit::init(timer::TIMER_FREQUENCY_HZ);

    crate::kprintln!("[NX][CPU] enabling interrupts");
    cpu::sti();
    cpu::log_state("after sti");

    pic::log_irq_state("after sti");
    timer::debug_wait_for_hardware_ticks();

    crate::kprintln!("[NX][ARCH] x86_64 init complete");
}

/// Triggers the breakpoint exception smoke test
pub fn test_breakpoint() {
    interrupts::trigger_breakpoint();
}

/// Triggers the page fault smoke test
/// 
/// This test is fatal by design.
pub fn test_page_fault() {
    interrupts::trigger_page_fault();
}

/// Enters the normal halt loop
pub fn halt_loop() -> ! {
    cpu::halt_loop();
}

/// Enters a fatal halt loop with maskable interrupts disabled
pub fn panic_halt_loop() -> ! {
    cpu::panic_halt_loop();
}

/// Triggers the timer interrupt vector through a software interrupt
/// 
/// This is a debug-only test for the vector 32 ISR path. It does not prove that
/// the PIT/PIC hardware path is working.
pub fn test_timer_interrupt() {
    interrupts::trigger_timer_interrupt_test();
}

/// Logs the current CPU state
pub fn log_cpu_state(label: &str) {
    cpu::log_state(label);
}