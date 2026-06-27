//! Early serial logging through COM1
//! 
//! The serial driver is the kernel's first reliable debugging channel. It is
//! available before the framebuffer console, allocator, scheduler, or device
//! model exist.
//! 
//! QEMU maps `-serial stdio` to this COM1 port, so anything written here shows
//! up in the host PowerShell terminal.

use core::fmt::{self, Write};

/// Base I/O port address for COM1
const COM1: u16 = 0x3f8;

/// Initializes the COM1 serial port
/// 
/// Configuration:
/// 
/// - interrupts disabled at the UART level
/// - baud divisor set for 38400 baud
/// - 8 data bits
/// - no parity
/// - one stop bit
/// - FIFO enabled
pub fn init() {
    // SAFETY: Port I/O is inherently unsafe and only valid in ring 0. Limine
    // transfers control to the kernel in ring 0, and COM1 is the conventional
    // debug serial port used by QEMU.
    unsafe {
        outb(COM1 + 1, 0x00);   // Disable UART interrupts
        outb(COM1 + 3, 0x80);   // Enable DLAB
        outb(COM1, 0x03);       // Divisor low byte: 38400 baud
        outb(COM1 + 1, 0x00);   // Divisor high byte
        outb(COM1 + 3, 0x03);   // 8 bits, no parity, one stop bit
        outb(COM1 + 2, 0xC7);   // Enable FIFO, clear it, 14-byte threshold
        outb(COM1 + 4, 0x08);   // IRQs enabled, RTS/DSR set
    }
}

/// Writes a single byte to COM1
/// 
/// Newline bytes are expanded to CRLF because many serial terminals expect
/// `\r\n` line endings.
pub fn write_byte(byte: u8) {
    match byte {
        b'\n' => {
            raw_write_byte(b'\r');
            raw_write_byte(b'\n');
        }
        byte => {
            raw_write_byte(byte);
        }
    }
}

/// Writes one byte directly to COM1 withut newline translation
fn raw_write_byte(byte: u8) {
    // SAFETY: COM1 has been initialized by `serial::init`. The transmit loop
    // waits until the UART reports that the transmit holding register is empty
    // before writing the byte.
    unsafe {
        while(inb(COM1 + 5) & 0x20) == 0 {}
        outb(COM1, byte);
    }
}

/// Writes a null-terminated C string to the serial port
/// 
/// This is used for Limine-provided bootloader strings.
/// 
/// # Safety
/// `ptr` must either be null or point to a valid null-terminated byte string.
/// The memory must remain readable for the duration of this function call.
pub unsafe fn write_cstr(mut ptr: *const u8) {
    if ptr.is_null() {
        return;
    }

    loop {
        // SAFETY: The caller guarantees that `ptr` points to a readable
        // null-terminated byte string.
        let byte = unsafe {
            core::ptr::read_volatile(ptr)
        };

        if byte == 0 {
            break;
        }

        write_byte(byte);

        // SAFETY: Advancing inside the caller-provided C string is valid until
        // the terminating zero byte is reached.
        ptr = unsafe {
            ptr.add(1)
        };
    }
}

/// `core::fmt::Write` adapter for the serial port
struct SerialWriter;

impl Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            write_byte(byte);
        }

        Ok(())
    }
}

/// Internal formatting entry point used by `kprint!` and `kprintln!`
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    let _ = SerialWriter.write_fmt(args);
}

/// Kernel print macro
/// 
/// This writes formatted text to the early serial logger.
#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {
        $crate::serial::_print(core::format_args!($($arg)*));
    };
}

/// Kernel println macro
/// 
/// This writes formatted text followed by a newline to the early serial logger.
#[macro_export]
macro_rules! kprintln {
    () => {
        $crate::kprint!("\n");
    };
    ($fmt:expr) => {
        $crate::kprint!(concat!($fmt, "\n"));
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::kprint!(concat!($fmt, "\n"), $($arg)*);
    };
}

/// Writes one byte to an I/O port
unsafe fn outb(port: u16, value: u8) {
    // SAFETY: The caller must ensure that `port` is valid for byte output. This
    // module only uses legacy COM1 UART ports.
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
            options(nomem, nostack, preserves_flags)
        );
    }
}

/// Reads one byte from an I/O port
unsafe fn inb(port: u16) -> u8 {
    let value: u8;

    // SAFETY: The caller must ensure that `port` is valid for byte input. This
    // module only uses legacy COM1 UART ports.
    unsafe {
        core::arch::asm!(
            "in al, dx",
            out("al") value,
            in("dx") port,
            options(nomem, nostack, preserves_flags)
        );
    }

    value
}