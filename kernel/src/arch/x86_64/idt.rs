use core::arch::asm;
use core::mem::size_of;
use core::ptr;

use super::gdt::KERNEL_CODE_SELECTOR;

const IDT_ENTRY_COUNT: usize = 256;
const INTERRUPT_GATE: u16 = 0x8e00;

unsafe extern "C" {
    fn nx_isr_breakpoint();
    fn nx_isr_page_fault();
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    options: u16,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32
}

impl IdtEntry {
    const fn missing() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            options: 0,
            offset_mid: 0,
            offset_high: 0,
            reserved: 0
        }
    }

    fn set_handler(&mut self, handler: u64) {
        self.offset_low = handler as u16;
        self.selector = KERNEL_CODE_SELECTOR;
        self.options = INTERRUPT_GATE;
        self.offset_mid = (handler >> 16) as u16;
        self.offset_high = (handler >> 32) as u32;
        self.reserved = 0;
    }
}

#[repr(C, packed)]
struct DescriptorTablePointer {
    limit: u16,
    base: u64
}

static mut IDT: [IdtEntry; IDT_ENTRY_COUNT] = [IdtEntry::missing(); IDT_ENTRY_COUNT];

pub fn init() {
    unsafe {
        IDT[3].set_handler(nx_isr_breakpoint as *const () as u64);
        IDT[14].set_handler(nx_isr_page_fault as *const () as u64);

        load_idt();
    }

    crate::kprintln!("[NX][IDT] loaded");
    crate::kprintln!("[NX][IDT] breakpoint handler installed at vector 3");
    crate::kprintln!("[NX][IDT] page fault handler installed at vector 14");
}

unsafe fn load_idt() {
    let idt_ptr = DescriptorTablePointer {
        limit: (size_of::<[IdtEntry; IDT_ENTRY_COUNT]>() - 1) as u16,
        base: ptr::addr_of!(IDT) as u64
    };

    asm!(
        "lidt [{idt_ptr}]",
        idt_ptr = in(reg) &idt_ptr,
        options(readonly, nostack, preserves_flags)
    );
}