//! Programmable Interval Timer setup
//! 
//! The PIT is a legacy timer device that can periodically raise IRQ0 through
//! the PIC. We use it for the first timer interrupt draft because it is simple
//! and reliably emulated by QEMU.

#![allow(dead_code)]

use super::port::outb;

/// PIT input clock frequence in Hz
const PIT_BASE_FREQUENCY: u32 = 1_193_182;

/// PIT channel 0 data port
const PIT_CHANNEL_0: u16 = 0x40;

/// PIT command port
const PIT_COMMAND: u16 = 0x43;

/// PIT command byte for channel 0, lobyte/hibyte access, mode 3 square wave
const PIT_COMMAND_CHANNEL_0_RATE: u8 = 0x36;

/// Configures PIT channel 0 to generate periodic timer interrupts
/// 
/// `frequency_hz` should be a reasonable non-zero value. For the early kernel,
/// `100 Hz` is a good default because it gives visible ticks without flooding
/// the serial log.
pub fn init(frequency_hz: u32) {
    let divisor = PIT_BASE_FREQUENCY / frequency_hz;

    crate::kprintln!(
        "[NX][PIT] configuring channel 0 at {} Hz, divisor={}",
        frequency_hz,
        divisor
    );

    // SAFETY: The PIT is programmed through legacy I/O ports. This function is
    // called during early boot before interrupts are enabled.
    unsafe {
        outb(PIT_COMMAND, PIT_COMMAND_CHANNEL_0_RATE);
        outb(PIT_CHANNEL_0, (divisor & 0xff) as u8);
        outb(PIT_CHANNEL_0, ((divisor >> 8) & 0xff) as u8);
    }

    crate::kprintln!("[NX][PIT] configured");
}