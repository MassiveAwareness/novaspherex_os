//! x86_64 exception and interrupt stubs
//! 
//! This module contains the low-level assembly entry points used by the IDT,
//! plus small Rust handlers for early exception diagnostics.
//! 
//! The assembly stubs are responsible for:
//! 
//! - saving general-purpose registers
//! - preserving stack layout expectations
//! - aligning the stack before calling Rust
//! - passing an `InterruptFrame` pointer to Rust
//! - returning with `iretq` when the exception is recoverable
//! 
//! The current milestone implements:
//! 
//! - breakpoint exception handling
//! - page fault diagnostics

#![allow(dead_code)]

use core::arch::{asm, global_asm};

use super::cpu;

/// Minimal interrupt frame pushed by the CPU for same-priviledge exceptions
/// 
/// For the current early kernel state, exceptions happen in ring 0 and return
/// to ring 0, so the CPU pushes:
/// 
/// RIP
/// CS
/// RFLAGS
/// 
/// Exceptions that include an error code, such as page fault, push the error
/// code in addition to this frame.
#[repr(C)]
pub struct InterruptFrame {
    /// Instruction pointer to resume at or report
    pub instruction_pointer: u64,

    /// Code segment selector saved by the CPU
    pub code_segment: u64,

    /// Saved RFLAGS value
    pub cpu_flags: u64
}

global_asm!(
    r#"
.global nx_isr_breakpoint
nx_isr_breakpoint:
    cld

    // Save general-purpose registers
    //
    // We currently save 15 registers. `rsp` is not saved explicitly because
    // the CPU-created interrupt frame and the current stack pointer define the
    // active exception stack layout.
    push rax
    push rbx
    push rcx
    push rdx
    push rbp
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15

    // Align stack before calling Rust
    //
    // The Rust ABI expects the stack to be suitably aligned at function call
    // boundaries. The extra 8-byte padding keeps calls from this hand-written
    // interrupt stub stable.
    sub rsp, 8

    // Breakpoint exceptions do not push an error code
    //
    // CPU-pushed frame:
    //
    //      RIP
    //      CS
    //      RFLAGS
    //
    // After 15 registers pushes and 8 vytes of alignment padding, the
    // InterruptFrame starts at rsp + 128.
    lea rdi, [rsp + 128]
    call nx_breakpoint_handler

    // Remove alignment padding.
    add rsp, 8

    // Restore general-purpose registers in reverse order
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rbp
    pop rdx
    pop rcx
    pop rbx
    pop rax

    // Return from the exception
    iretq


.global nx_isr_page_fault
nx_isr_page_fault:
    cld

    // Save general-purpose registers
    push rax
    push rbx
    push rcx
    push rdx
    push rbp
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15

    // Align stack before calling Rust
    sub rsp, 8

    // Page faults push an error code
    //
    // CPU-pushed frame:
    //
    //      ERROR CODE
    //      RIP
    //      CS
    //      RFLAGS
    //
    // After argument in SysV x86_64 ABI:       rdi = &InterruptFrame
    // Second argument in SysV x86_64 ABI:      rsi = error_code
    lea rdi, [rsp + 136]
    mov rsi, [rsp + 128]
    call nx_page_fault_handler

    // The Rust page fault handler is fatal and never returns
    hlt

.global nx_isr_timer
nx_isr_timer:
    cld

    // Save general-purpose registers.
    push rax
    push rbx
    push rcx
    push rdx
    push rbp
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15

    // Align stack before calling Rust.
    sub rsp, 8

    // IRQ0 does not push an error code. The Rust timer handler currently
    // needs no frame pointer, so we call it without arguments.
    call nx_timer_handler

    // Remove alignment padding.
    add rsp, 8

    // Restore general-purpose registers.
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rbp
    pop rdx
    pop rcx
    pop rbx
    pop rax

    // Return from interrupt.
    iretq
"#
);

/// Triggers a breakpoint exception using `int3`
/// 
/// This is used as a smoke test for the IDT and exception return path.
pub fn trigger_breakpoint() {

    // SAFETY: `int3` intentionally triggers vector 3. The IDT must already
    // contain a valid a valid breakpoint handler before this function is called.
    //
    // This instruction uses the stack implicitly because the CPU pushes an
    // interrupt frame. Therefore this block must not use the `nostack` option.
    unsafe {
        asm!("int3", options(nomem));
    }
}

/// Triggers a page fault by reading from an intentionally invalid address
/// 
/// This test is expected to be fatal. The page fault handler logs diagnostics
/// and enters a panic halt loop.
pub fn trigger_page_fault() {
    // SAFETY: This intentionally performs an invalid memory read to exercise
    // the page fault handler. It should only be called in controlled debug
    // scenarios after the page fault handler is installed.
    unsafe {
        let bad_address: u64 = 0xdead_beef;

        asm!(
            "mov al, byte ptr [{addr}]",
            addr = in(reg) bad_address,
            lateout("al") _,
            options(nostack, preserves_flags)
        );
    }
}

/// Rust breakpoint exception handler
/// 
/// Breakpoint exceptions are recoverable in this early kernel. After logging,
/// the assembly stub restores registers and returns with `iretq`.
#[no_mangle]
extern "C" fn nx_breakpoint_handler(frame: &InterruptFrame) {
    crate::kprintln!(
        "[NX][INT] breakpoint exception at rip={:#018x}, cs={:#06x}, flags={:#018x}",
        frame.instruction_pointer,
        frame.code_segment,
        frame.cpu_flags
    );
}

/// Rust page fault handler
/// 
/// Page faults are currently treated as fatal. The handler logs the faulting
/// virtual address from `CR2`, the CPU-provided error code, and the saved
/// instruction frame, then halts the CPU with interrupts disabled.
#[no_mangle]
extern "C" fn nx_page_fault_handler(frame: &InterruptFrame, error_code: u64) -> ! {
    let fault_address = cpu::read_cr2();

    crate::kprintln!("[NX][INT] PAGE FAULT");
    crate::kprintln!("[NX][INT] fault address: {:#018x}", fault_address);
    crate::kprintln!("[NX][INT] error code:    {:#018x}", error_code);
    crate::kprintln!("[NX][INT] rip:           {:#018x}", frame.instruction_pointer);
    crate::kprintln!("[NX][INT] cs:            {:#06x}", frame.code_segment);
    crate::kprintln!("[NX][INT] flags:         {:#018x}", frame.cpu_flags);
    crate::kprintln!("[NX][INT] halting after page fault");

    cpu::panic_halt_loop();
}

/// Triggers the timer interrupt vector using a software interrupt
/// 
/// This does not test the PIT or PIC hardware path. It only verifies that
/// vector 32 is installed in the IDT and that the timer ISR can call the Rust
/// timer handler successfully.
pub fn trigger_timer_interrupt_test() {
    // SAFETY: `int 32` intentionally invokes the timer interrupt vector.
    // The IDT must already contain a valid handler for vector 32.
    //
    // Like all interrupt instructions, this uses the stack implicitly because
    // the CPU pushes an interrupt frame. Therefore this block must not use the
    // `nostack` option.
    unsafe {
        asm!("int 32", options(nomem));
    }
}