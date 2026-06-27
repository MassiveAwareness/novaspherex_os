#![allow(dead_code)]

use core::arch::asm;
use core::mem::size_of;

pub const KERNEL_CODE_SELECTOR: u16 = 0x08;
pub const KERNEL_DATA_SELECTOR: u16 = 0x10;

pub const LIMINE_COMPAT_CODE_SELECTOR: u16 = 0x28;
pub const LIMINE_COMPAT_DATA_SELECTOR: u16 = 0x30;

#[repr(C, packed)]
struct DescriptorTablePointer {
    limit: u16,
    base: u64,
}

#[repr(align(8))]
struct Gdt([u64; 7]);

const NULL_DESCRIPTOR: u64 = 0x0000000000000000;
const KERNEL_CODE_DESCRIPTOR: u64 = 0x00af9a000000ffff;
const KERNEL_DATA_DESCRIPTOR: u64 = 0x00cf92000000ffff;

static GDT: Gdt = Gdt([
    NULL_DESCRIPTOR,        // 0x00
    KERNEL_CODE_DESCRIPTOR, // 0x08 - our kernel code
    KERNEL_DATA_DESCRIPTOR, // 0x10 - our kernel data
    NULL_DESCRIPTOR,        // 0x18
    NULL_DESCRIPTOR,        // 0x20
    KERNEL_CODE_DESCRIPTOR, // 0x28 - Limine-compatible code selector
    KERNEL_DATA_DESCRIPTOR, // 0x30 - Limine-compatible data/stack selector
]);

pub fn init() {
    crate::kprintln!("[NX][GDT] loading");

    unsafe {
        load_gdt_only();
    }

    crate::kprintln!("[NX][GDT] loaded");
    crate::kprintln!("[NX][GDT] CS reload skipped for now");
    crate::kprintln!("[NX][GDT] selector 0x28 kept valid for Limine CS compatibility");
    crate::kprintln!("[NX][GDT] selector 0x30 kept valid for Limine data/stack compatibility");
}

unsafe fn load_gdt_only() {
    let gdt_ptr = DescriptorTablePointer {
        limit: (size_of::<Gdt>() - 1) as u16,
        base: &GDT as *const Gdt as u64,
    };

    asm!(
        "lgdt [{gdt_ptr}]",
        gdt_ptr = in(reg) &gdt_ptr,
        options(readonly, nostack, preserves_flags)
    );
}