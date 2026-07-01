//! Interrupt Descriptor Table setup for x86_64
//! 
//! The IDT maps CPU exception and interrupt vectors to handler stubs.
//! Each entry tells the CPU which code selector and instruction pointer to
//! use when a given vector fires.
//! 
//! This module installs the early exception handlers required by Milestone 1:
//! 
//! - vector 3:     breakpoint exception
//! - vector 14:    page fault exception

use core::arch::asm;
use core::mem::size_of;
use core::ptr;

use crate::arch::x86_64::timer;

use super::gdt::KERNEL_CODE_SELECTOR;

/// Number of entries in the x86_64 IDT
/// 
/// x86_64 supports 256 interrupt vectors.
const IDT_ENTRY_COUNT: usize = 256;

/// Interrupt gate options for a present ring-0 interrupt gate
/// 
/// `0x8e00` encodes:
/// 
/// - present bit set
/// - descriptor priviledge level 0
/// - 64-bit interrupt gate type
/// 
/// Interrupt gates clear the interrupt flag while the handler is running.
const INTERRUPT_GATE: u16 = 0x8e00;

unsafe extern "C" {
    /// Assembly stub for vector 3, breakpoint exception
    fn nx_isr_breakpoint();

    /// Assembly stub for vector 14, page fault exception
    fn nx_isr_page_fault();

    /// Assembly stub for vector 32, PIT timer interrupt
    fn nx_isr_timer();
}

/// One x86_64 IDT entry
/// 
/// The CPU uses this layout directly. The handler address is split across
/// three fields because the IDT entry format is inherited from older x86
/// descriptor formats.
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
    /// Creates a missing/empty IDT entry
    /// 
    /// Triggering a vector with a missing entry will cause a fault.
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

    /// Installs an interrupt handler address into this entry
    /// 
    /// The handler runs using `KERNEL_CODE_SELECTOR`. The selector must refer
    /// to a valid executable 64-bit code segment in the active GDT.
    fn set_handler(&mut self, handler: u64) {
        self.offset_low = handler as u16;
        self.selector = KERNEL_CODE_SELECTOR;
        self.options = INTERRUPT_GATE;
        self.offset_mid = (handler >> 16) as u16;
        self.offset_high = (handler >> 32) as u32;
        self.reserved = 0;
    }
}

/// CPU descriptor-table pointer used by `lidt`
/// 
/// The layout matches the x86_64 `lidt` operand format.
#[repr(C, packed)]
struct DescriptorTablePointer {
    limit: u16,
    base: u64
}

/// Early kernel IDT
/// 
/// It starts with all entries missing. `init()` installs only the exception
/// handlers that are currently implemented.
static mut IDT: [IdtEntry; IDT_ENTRY_COUNT] = [IdtEntry::missing(); IDT_ENTRY_COUNT];

/// Initializes and loads the early IDT
/// 
/// Currently installed:
/// 
/// - vector 3:     breakpoint exception
/// - vector 14:    page fault exception
pub fn init() {
    // SAFETY: During early boot, initialization is single-threaded and
    // interrupts are not enabled yet. Mutating the static IDT here is safe
    // because no other CPU or interrupt handler can concurrency access it.
    unsafe {
        IDT[3].set_handler(nx_isr_breakpoint as *const () as u64);
        IDT[14].set_handler(nx_isr_page_fault as *const () as u64);
        IDT[timer::TIMER_VECTOR as usize].set_handler(nx_isr_timer as *const () as u64);

        load_idt();
    }

    crate::kprintln!("[NX][IDT] loaded");
    crate::kprintln!("[NX][IDT] breakpoint handler installed at vector 3");
    crate::kprintln!("[NX][IDT] page fault handler installed at vector 14");
    crate::kprintln!("[NX][IDT] timer handler installed at vector {}", timer::TIMER_VECTOR);
}

/// Loads the IDT register
unsafe fn load_idt() {
    let idt_ptr = DescriptorTablePointer {
        limit: (size_of::<[IdtEntry; IDT_ENTRY_COUNT]>() - 1) as u16,
        base: ptr::addr_of!(IDT) as u64
    };

    // SAFETY: `idt_ptr` points to the static IDT, which remains valid for the
    // lifetime of the kernel. `lidt` is priviledged and must only be executed in
    // ring 0. Limine has already transferred control to the user in ring 0.
    asm!(
        "lidt [{idt_ptr}]",
        idt_ptr = in(reg) &idt_ptr,
        options(readonly, nostack, preserves_flags)
    );
}