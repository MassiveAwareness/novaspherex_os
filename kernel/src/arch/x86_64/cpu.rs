//! Low-level x86_64 CPU helper functions
//!
//! This module contains small, focused wrappers around priviledged or
//! architecture-specific CPU instructions.
//! 
//! Higher-level kernel code should use these helpers instead of embedding
//! inline assembly directly. This keeps unsafe CPU operations centralized and
//! easier to audit.

#![allow(dead_code)]

use core::arch::asm;

/// Snapshot of a small subset of the current CPU state.
/// 
/// This is mainly focused for early boot diagnostics and exception debugging.
/// It is not intended to be a complete CPU context structure.
#[derive(Clone, Copy)]
pub struct CpuState {
    /// Current code segment selector
    pub cs: u16,

    /// Current stack segment selector
    pub ss: u16,

    /// Current data segment selector
    pub ds: u16,

    /// Current extra segment selector
    pub es: u16,

    /// Current RFLAGS register value
    pub rflags: u64,

    /// Current CR2 value
    /// 
    /// On x86_64, CR2 contains the faulting virtual address after a page fault
    pub cr2: u64
}

/// Disables maskable interrupts on the current CPU
/// 
/// This clears the interrup flag in `RFLAGS`. Non-maskable interrupts and
/// some machine-check conditions are not affected.
pub fn cli() {
    // SAFETY: `cli` is a priviledged instruction and must only be executed in
    // ring 0. The kernel is already running in ring 0 after Limine transfers
    // control to the user. The isntruction does not access memory on the stack.
    unsafe {
        asm!("cli", options(nomem, nostack, preserves_flags));
    }
}

/// Enables maskable interrupts on the current CPU
/// 
/// This should only be called after the IDT and interrupt controller state are
/// ready to handle incoming inpterrupts safely.
pub fn sti() {
    // SAFETY: `sti` is a priviledged instruction and must only be executed in
    // ring 0. The caller is responsible for ensuring that interrupt handlers
    // and interrupt controller state are initialized before enabling
    // interrupts.
    unsafe {
        asm!("sti", options(nomem, nostack, preserves_flags));
    }
}

/// Halts the current CPU until the next external interrupt
/// 
/// If interrupts are disabled, this may stop the CPU indefinitely unless a
/// non-maskable event occurs.
pub fn hlt() {
    // SAFETY: `hlt` is a priviledged instruction and must only be executed in
    // ring 0. It does not read or write memory and does not modify the stack.
    unsafe {
        asm!("hlt", options(nomem, nostack, preserves_flags));
    }
}

/// Enters an infinite halt loop
/// 
/// This is used when the kernel has no more work to do, but the system is not
/// necessarily in an error state.
pub fn halt_loop() -> ! {
    loop {
        hlt();
    }
}

/// Enters a halt loop suitable for panic or fatal exception paths
/// 
/// Maskable interrupts are disabled first so the CPU does not continue
/// handling timer, device, or external interrupts after a fatal condition.
pub fn panic_halt_loop() -> ! {
    cli();

    loop {
        hlt();
    }
}

/// Reads the current code segment selector
pub fn read_cs() -> u16 {
    let value: u16;

    // SAFETY: Reading a segment selector into `ax` does not access memory and
    // does not modify the stack. The output register is fully controlled by
    // the inline assembly operand.
    unsafe {
        asm!(
            "mov {0:x}, cs",
            out(reg) value,
            options(nomem, nostack, preserves_flags)
        );
    }

    value
}

/// Reads the current stack segment selector
pub fn read_ss() -> u16 {
    let value: u16;

    // SAFETY: Reading `ss` is a non-memory CPU register read. The instruction
    // does not alter control flow, memory, or the stack.
    unsafe {
        asm!(
            "mov {0:x}, ss",
            out(reg) value,
            options(nomem, nostack, preserves_flags)
        );
    }

    value
}

/// Reads the current data segment selector
pub fn read_ds() -> u16 {
    let value: u16;

    // SAFETY: Reading `ds` is a non-memory CPU register read. The instruction
    // does not alter control flow, memory, or the stack.
    unsafe {
        asm!(
            "mov {0:x}, ds",
            out(reg) value,
            options(nomem, nostack, preserves_flags)
        );
    }

    value
}

/// Reads the current extra segment selector
pub fn read_es() -> u16 {
    let value: u16;

    // SAFETY: Reading `es` is a non-memory CPU register read. The instruction
    // does not alter control flow, memory, or the stack.
    unsafe {
        asm!(
            "mov {0:x}, es",
            out(reg) value,
            options(nomem, nostack, preserves_flags)
        );
    }

    value
}

/// Reads the current `RFLAGS` register
/// 
/// This uses `pushfq` and `pop`, so it intentionally does not use the
/// `nostack` assembly option.
pub fn read_rflags() -> u64 {
    let value: u64;

    // SAFETY: `pushfq` pushed the current flags value and `pop` immediately
    // restores it into a general-purpose output register. The stack is used in
    // a balanced way: one push and one pop.
    unsafe {
        asm!(
            "pushfq",
            "pop {}",
            out(reg) value,
            options(nomem, preserves_flags)
        );
    }

    value
}

/// Reads the current `CR2` register
/// 
/// On x86_64, `CR2` contains the faulting virtual address after a page fault.
/// Outside of a page fault path, the value is still readable but may not be
/// meaningful.
pub fn read_cr2() -> u64 {
    let value: u64;

    // SAFETY: Reading `cr2` is a priviledged CPU register read. The kernel runs
    // in ring 0, and this isntruction does not access memory or the stack.
    unsafe {
        asm!(
            "mov {}, cr2",
            out(reg) value,
            options(nomem, nostack, preserves_flags)
        );
    }

    value
}

/// Reads a small diagnostic snapshot of the current CPU state
pub fn read_state() -> CpuState {
    CpuState {
        cs: read_cs(),
        ss: read_ss(),
        ds: read_ds(),
        es: read_es(),
        rflags: read_rflags(),
        cr2: read_cr2()
    }
}

/// Logs a small diagnostic snapshot of the current CPU state.
/// 
/// This is primarily used during early boot, GDT/IDT setup, and exception
/// debugging.
pub fn log_state(label: &str) {
    let state = read_state();

    crate::kprintln!("[NX][CPU] state: {}", label);
    crate::kprintln!(
        "[NX][CPU] cs={:#06x} ss={:#06x} ds={:#06x} es={:#06x}",
        state.cs,
        state.ss,
        state.ds,
        state.es
    );
    crate::kprintln!(
        "[NX][CPU] rflags={:#018x} cr2={:#018x}",
        state.rflags,
        state.cr2
    );
}