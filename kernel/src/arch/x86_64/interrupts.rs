#![allow(dead_code)]

use core::arch::{asm, global_asm};

use crate::halt_loop;

#[repr(C)]
pub struct InterruptFrame {
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub cpu_flags: u64,
}

global_asm!(
    r#"
.global nx_isr_breakpoint
nx_isr_breakpoint:
    cld

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

    sub rsp, 8

    // For int3, CPU pushed:
    //   RIP
    //   CS
    //   RFLAGS
    //
    // After 15 register pushes and 8 bytes alignment padding,
    // the interrupt frame starts at rsp + 128.
    lea rdi, [rsp + 128]
    call nx_breakpoint_handler

    add rsp, 8

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

    iretq


.global nx_isr_page_fault
nx_isr_page_fault:
    cld

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

    sub rsp, 8

    // For page fault, CPU pushed:
    //   ERROR CODE
    //   RIP
    //   CS
    //   RFLAGS
    //
    // After 15 register pushes and 8 bytes alignment padding:
    //   error code is at rsp + 128
    //   interrupt frame starts at rsp + 136
    lea rdi, [rsp + 136]
    mov rsi, [rsp + 128]
    call nx_page_fault_handler

    // nx_page_fault_handler never returns.
    hlt
"#
);

pub fn trigger_breakpoint() {
    unsafe {
        asm!("int3", options(nomem));
    }
}

pub fn trigger_page_fault() {
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

#[no_mangle]
extern "C" fn nx_breakpoint_handler(frame: &InterruptFrame) {
    crate::kprintln!(
        "[NX][INT] breakpoint exception at rip={:#018x}, cs={:#06x}, flags={:#018x}",
        frame.instruction_pointer,
        frame.code_segment,
        frame.cpu_flags
    );
}

#[no_mangle]
extern "C" fn nx_page_fault_handler(frame: &InterruptFrame, error_code: u64) -> ! {
    let fault_address: u64;

    unsafe {
        asm!(
            "mov {}, cr2",
            out(reg) fault_address,
            options(nomem, nostack, preserves_flags)
        );
    }

    crate::kprintln!("[NX][INT] PAGE FAULT");
    crate::kprintln!("[NX][INT] fault address: {:#018x}", fault_address);
    crate::kprintln!("[NX][INT] error code:    {:#018x}", error_code);
    crate::kprintln!("[NX][INT] rip:           {:#018x}", frame.instruction_pointer);
    crate::kprintln!("[NX][INT] cs:            {:#06x}", frame.code_segment);
    crate::kprintln!("[NX][INT] flags:         {:#018x}", frame.cpu_flags);
    crate::kprintln!("[NX][INT] halting after page fault");

    halt_loop();
}