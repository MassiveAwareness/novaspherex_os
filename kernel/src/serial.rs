use core::fmt::{self, Write};

const COM1: u16 = 0x3f8;

pub fn init() {
    unsafe {
        outb(COM1 + 1, 0x00);   // Disable interrupts
        outb(COM1 + 3, 0x80);   // Enable DLAB
        outb(COM1, 0x03);       // Divisor low: 38400 baud
        outb(COM1 + 1, 0x00);   // Divisor high
        outb(COM1 + 3, 0x03);   // 8 bits, no parity, one stop bit
        outb(COM1 + 2, 0xC7);   // FIFO on, clear, 14-byte threshold
        outb(COM1 + 4, 0x08);   // IRQs enabled, RTS/DSR set
    }
}

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

fn raw_write_byte(byte: u8) {
    unsafe {
        while(inb(COM1 + 5) & 0x20) == 0 {}
        outb(COM1, byte);
    }
}

pub unsafe fn write_cstr(mut ptr: *const u8) {
    if ptr.is_null() {
        return;
    }

    while core::ptr::read_volatile(ptr) != 0 {
        write_byte(core::ptr::read_volatile(ptr));
        ptr = ptr.add(1);
    }
}

struct SerialWriter;

impl Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            write_byte(byte);
        }

        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    let _ = SerialWriter.write_fmt(args);
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {
        $crate::serial::_print(core::format_args!($($arg)*));
    };
}

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

unsafe fn outb(port: u16, value: u8) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nomem, nostack, preserves_flags)
    );
}

unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!(
        "in al, dx",
        out("al") value,
        in("dx") port,
        options(nomem, nostack, preserves_flags)
    );

    value
}