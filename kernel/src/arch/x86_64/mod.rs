pub mod gdt;
pub mod idt;
pub mod interrupts;

pub fn init() {
    crate::kprintln!("[NX][ARCH] x86_64 init begin");

    gdt::init();
    idt::init();

    crate::kprintln!("[NX][ARCH] x86_64 init complete");
}

pub fn test_breakpoint() {
    interrupts::trigger_breakpoint();
}

pub fn test_page_fault() {
    interrupts::trigger_page_fault();
}