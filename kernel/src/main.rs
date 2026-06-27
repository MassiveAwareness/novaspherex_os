#![no_std]
#![no_main]

mod arch;
mod limine;
mod serial;

use core::panic::PanicInfo;

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

    kprintln!("[NX] triggering breakpoint exception test");
    arch::test_breakpoint();
    kprintln!("[NX] breakpoint exception returned successfully");

    kprintln!("[NX] triggering page fault test");
    arch::test_page_fault();

    kprintln!("[NX] reached halt loop");
    halt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kprintln!("[NX][PANIC] {}", info);
    halt_loop();
}

pub fn halt_loop() -> ! {
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}