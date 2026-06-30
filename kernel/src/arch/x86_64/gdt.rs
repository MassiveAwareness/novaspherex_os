//! Global Descriptor Table setup for x86_64
//! 
//! In long mode, segmentation is mostly disabled, but the CPU still requires
//! valid code and data descriptors for priviledge checks, interrupt gates, and
//! certain control-flow transitions.
//! 
//! During early boot, Limine transfers control to the kernel with its own GDT
//! already active. We install our own GDT, but we temporarily keep Limine's
//! observed selectors valid until we implement a stable far jump into our own
//! kernel code selector.

#![allow(dead_code)]

use core::arch::asm;
use core::mem::size_of;

/// Kernel code segment selector in our own GDT
/// 
/// This selector is used by IDT interrupt gates.
pub const KERNEL_CODE_SELECTOR: u16 = 0x08;

/// Kernel data segment selector in our own GDT.
pub const KERNEL_DATA_SELECTOR: u16 = 0x10;

/// Limine-compatible code selector observed during early boot.
/// 
/// In our current QEMU/Limine environment, the kernel starts with `CS=0x28`.
/// Until we reload `CS` into `KERNEL_CODE_SELECTOR`, this selector must remain
/// valid in our GDT so `iretq` can return to the interrupted context.
pub const LIMINE_COMPAT_CODE_SELECTOR: u16 = 0x28;

/// Limine-compatible data/stack selector observed during early boot.
/// 
/// In our current QEMU/Limine environment, the kernel starts with
/// `SS=DS=ES=0x30`. Until we fully switch to our own data selector, this
/// selector must remain valid in our GDT.
pub const LIMINE_COMPAT_DATA_SELECTOR: u16 = 0x30;

/// CPU descriptor-table pointer used by `lgdt`
/// 
/// The layout is defined by the x86_64 architecture:
/// 
/// - `limit`: size of the table in bytes minus one
/// - `base`: linear address of the first table entry
#[repr(C, packed)]
struct DescriptorTablePointer {
    limit: u16,
    base: u64,
}

/// Global Descriptor Table used during eraly kernel initialization
/// 
/// The entries are 8-byte segment descriptors. In long mode, the base and
/// limit fields are ignored for most code/data segments, but access bits and
/// long-mode bits still matter.
#[repr(align(8))]
struct Gdt([u64; 7]);

/// Null descriptor
/// 
/// Selector `0x00` is never a valid segment selector.
const NULL_DESCRIPTOR: u64 = 0x0000000000000000;

/// 64-bit ring-0 executable code descriptor
/// 
/// Descriptor purpose:
/// 
/// - present
/// - descriptor priviledge level 0
/// - executable code segment
/// - readable code segment
/// - long mode enabled
const KERNEL_CODE_DESCRIPTOR: u64 = 0x00af9a000000ffff;

/// Ring-0 writable data descriptor
/// 
/// In x86_64 long mode, data segment base/limit are mostly ignored, but a
/// valid data/stack descriptor is still required for `SS`, `DS`, and `ES`.
const KERNEL_DATA_DESCRIPTOR: u64 = 0x00cf92000000ffff;

/// Early boot GDT
/// 
/// Layout (in order):
/// 
/// 0x00 -> null
/// 0x08 -> NovasphereX kernel code
/// 0x10 -> NovasphereX kernel data
/// 0x18 -> reserved
/// 0x20 -> reserved
/// 0x28 -> Limine-compatible code selector
/// 0x30 -> Limine-compatible data/stack selector
/// 
/// The `0x28` and `0x30` entries are temporary bootstrap compatibility
/// descriptors. They keep the bootloader-provided execution context valid
/// after we load our own GDTR.
static GDT: Gdt = Gdt([
    NULL_DESCRIPTOR,        // 0x00
    KERNEL_CODE_DESCRIPTOR, // 0x08 - our kernel code
    KERNEL_DATA_DESCRIPTOR, // 0x10 - our kernel data
    NULL_DESCRIPTOR,        // 0x18
    NULL_DESCRIPTOR,        // 0x20
    KERNEL_CODE_DESCRIPTOR, // 0x28 - Limine-compatible code selector
    KERNEL_DATA_DESCRIPTOR, // 0x30 - Limine-compatible data/stack selector
]);

/// Loads the early NovasphereX GDT
/// 
/// This function intentionally does not reload `CS` yet. The kernel is still
/// executing with Limine's observed `CS=0x28`, so the GDT keeps selector
/// `0x28` valid.
/// 
/// A later milestone should replace this compatibility approach with a stable
/// far jump or far return into `KERNEL_CODE_SELECTOR`.
pub fn init() {
    crate::kprintln!("[NX][GDT] loading");

    // SAFETY: The GDT is a static, properly aligned descriptor table that
    // remains valid for the lifetime of the kernel. `lgdt` is priviledged and
    // must only be executed in ring 0; Limine already transfers control to us
    // in ring 0.
    unsafe {
        load_gdt_only();
    }

    crate::kprintln!("[NX][GDT] loaded");
    crate::kprintln!("[NX][GDT] CS reload skipped for now");
    crate::kprintln!("[NX][GDT] selector 0x28 kept valid for Limine CS compatibility");
    crate::kprintln!("[NX][GDT] selector 0x30 kept valid for Limine data/stack compatibility");
}

/// Loads the GDT register without reloading segment registers
/// 
/// This is a conservative early-bootstrap step. We update `GDTR`, but do not
/// modify `CS`, `SS`, `DS`, or `ES` here. The compatibility descriptors keep
/// the existing selectors valid.
unsafe fn load_gdt_only() {
    let gdt_ptr = DescriptorTablePointer {
        limit: (size_of::<Gdt>() - 1) as u16,
        base: &GDT as *const Gdt as u64,
    };

    // SAFETY: `gdt_ptr` points to the static `GDT`, which is aligned and valid.
    // The instruction only updates the CPU's GDTR register. It does not access
    // the stack, and the pointer remains valid after this function returns
    // because the underlying table is static.
    asm!(
        "lgdt [{gdt_ptr}]",
        gdt_ptr = in(reg) &gdt_ptr,
        options(readonly, nostack, preserves_flags)
    );
}