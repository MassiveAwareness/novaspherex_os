//! NovasphereX kernel entry point
//! 
//! This module contains the early boot pipeline after Limine trasfers control
//! to the Rust kernel.
//! 
//! The entry point stays intentionally small. Architecture-specific operations
//! are routed through `arch`, boot protocol access through `limine`, and early
//! logging through `serial`.

#![no_std]
#![no_main]

mod arch;
mod limine;
mod serial;

use core::panic::PanicInfo;

/// Enables the recoverable breakpoint exception smoke test
const RUN_BREAKPOINT_TEST: bool = true;

/// Enables the fatal page fault smoke test
/// 
/// Keep this disabled during normal development boots because the page fault
/// handler intentionally halts the kernel after logging diagnostics.
const RUN_PAGE_FAULT_TEST: bool = false;

/// Kernel entry point called by the bootloader
/// 
/// Limine jumps here after loading the kernel ELF and preparing the requested
/// boot protocol responses.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial::init();

    kprintln!("[NX] NovasphereX kernel booted");
    kprintln!("[NX] target: x86_64-unknown-none");
    kprintln!("[NX] runtime: no_std");
    kprintln!("[NX] boot protocol: Limine");

    if limine::base_revision_supported() {
        kprintln!("[NX] Limine base revision supported");
    } else {
        kprintln!("[NX] WARNING: Limine base revision was not acknowledged");
    }

    // SAFETY: Limine response pointers are valid during early boot if the
    // corresponding request was acknowledged. Each helper performs null checks
    // before using optional responses.
    unsafe {
        if let Some(info) = limine::bootloader_info() {
            kprint!("[NX] bootloader: ");
            serial::write_cstr(info.name as *const u8);
            kprint!(" ");
            serial::write_cstr(info.version as *const u8);
            kprintln!();
        } else {
            kprintln!("[NX] bootloader info unavailable");
        }

        limine::draw_boot_banner();
    }

    kprintln!("[NX] initializing CPU baseline");
    arch::init();

    if RUN_BREAKPOINT_TEST {
        kprintln!("[NX] triggering breakpoint exception test");
        arch::test_breakpoint();
        kprintln!("[NX] breakpoint exception returned successfully");
    }

    /* kprintln!("[NX] triggering timer interrupt vector test");
    arch::test_timer_interrupt();
    kprintln!("[NX] timer interrupt vector returned successfully"); */

    if RUN_PAGE_FAULT_TEST {
        kprintln!("[NX] triggering page fault test");
        arch::test_page_fault();

        kprintln!("[NX] ERROR: page fault test returned unexpectedly");
        arch::panic_halt_loop();
    }

    kprintln!("[NX] reached halt loop");
    arch::halt_loop();
}

/// Kernel panic handler
/// 
/// At this stage of the project, a panic is patal. We log the panic, dump a
/// small CPU state snapshot, disable maskable interrupts, and halt forever.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kprintln!("[NX][PANIC] {}", info);
    arch::log_cpu_state("panic");
    arch::panic_halt_loop();
}