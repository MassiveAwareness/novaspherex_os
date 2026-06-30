//! x86_64 I/O port helpers
//! 
//! This module centralizes legacy port I/O operations such as `inb` and
//! `outb`. These instructions are required for early hardware programming,
//! including the 8259 PIC, PIT, and serial ports.
//! 
//! Port I/O is inherently unsafe because Rust cannot verify that a given port
//! number is valid or that the device behind the port expects the value being
//! written.

#![allow(dead_code)]

use core::arch::asm;

/// Writes one byte to an I/O port
/// 
/// # Safety
/// The caller must ensure that `port` is valid for byte output and that writing
/// `value` is meaningful for the device attached to that port.
pub unsafe fn outb(port: u16, value: u8) {
    // SAFETY: The caller upholds the port validity contract. The instruction
    // does not access memory or the stack.
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
            options(nomem, nostack, preserves_flags)
        );
    }
}

/// Reads one byte from an I/O port
/// 
/// # Safety
/// The caller must ensure that `port` is valid byte input and that reading
/// from it has no unexpected side effects for the current hardwware state.
pub unsafe fn inb(port: u16) -> u8 {
    let value: u8;

    // SAFETY: The caller upholds the port validity contract. The instruction
    // does not access memory or the stack.
    unsafe {
        asm!(
            "in al, dx",
            out("al") value,
            in("dx") port,
            options(nomem, nostack, preserves_flags)
        );
    }

    value
}

/// Performs a tiny I/O delay
/// 
/// Historically, writing to port `0x80` has been used as a short delay between
/// programming legacy PC devices such as the PIC. QEMU accepts this pattern,
/// and it is sufficient for our early boot environment.
pub fn io_wait() {
    // SAFETY: Port `0x80` is conventionally used for POST/debug delay writes.
    // We do not depend on any returned value.
    unsafe {
        outb(0x80, 0);
    }
}