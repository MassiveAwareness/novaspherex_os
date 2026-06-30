//! Legacy 8259 Programmable Interrupt Controller support
//! 
//! The 8259 PIC is an old interrupt controller still emulated by QEMU. We use
//! it as the first interrupt controller because it is simple to program and
//! enough to prove that timer interrupts can reach out kernel.
//! 
//! Long term, NovasphereX should move to APIC/IOAPIC, but PIC + PIT is a good
//! Milestone 1 smoke test.

#![allow(dead_code)]

use super::port::{inb, io_wait, outb};

/// First CPU vector used for hardware IRQs after PIC remapping
pub const PIC_1_OFFSET: u8 = 32;

/// First CPU vector used for slave PIC IRQs after PIC remapping
pub const PIC_2_OFFSET: u8 = 40;

/// Timer IRQ line in the master PIC
pub const IRQ_TIMER: u8 = 0;

const PIC_1_COMMAND: u16 = 0x20;
const PIC_1_DATA: u16 = 0x21;
const PIC_2_COMMAND: u16 = 0xa0;
const PIC_2_DATA: u16 = 0xa1;

const PIC_EOI: u8 = 0x20;

const ICW1_INIT: u8 = 0x10;
const ICW1_ICW4: u8 = 0x01;
const ICW4_8086: u8 = 0x01;
const PIC_READ_IRR: u8 = 0x0a;
const PIC_READ_ISR: u8 = 0x0b;

/// Initializes the legacy PIC pair
/// 
/// This remaps hardware IRQs away from CPU exception vectors:
/// 
/// IRQ0..IRQ7      -> vectors 32..39
/// IRQ8..IRQ15     -> vectors 40..47
/// 
/// After remapping, only IRQ0, the PIT timer, is unmasked. All other legacy IRQ
/// lines remain masked until we intentionally add drivers for them.
pub fn init() {
    crate::kprintln!("[NX][PIC] remapping IRQs to vectors 32..47");

    // SAFETY: The port sequence below is the standard 8259 PIC initialization
    // sequence. It is only executed during early single-core boot before
    // interrupts are enabled.
    unsafe {
        remap(PIC_1_OFFSET, PIC_2_OFFSET);

        // Mask all IRQs except IRQ0 on the master PIC.
        //
        // Master mask 0b1111_1110 means:
        //      IRQ0 unmasked
        //      IRQ1..IRQ7 masked
        //
        // Slave mask 0b1111_1111 masks all slave IRQs.
        outb(PIC_1_DATA, 0b1111_1110);
        outb(PIC_2_DATA, 0b1111_1111);
    }

    crate::kprintln!("[NX][PIC] IRQ0 timer unmasked");
    log_masks("after init");
}

/// Sends End-of-Interrupt for a handled IRQ
/// 
/// The PIC will not deliver further interrupts from the same line until it
/// receives an EOI command.
pub fn send_eoi(irq: u8) {
    // SAFETY: EOI commands are written to PIC command ports. If the interrupt
    // came from the slave PIC, the slave must be acknowledged first, then the
    // master.
    unsafe {
        if irq >= 8 {
            outb(PIC_2_COMMAND, PIC_EOI);
        }

        outb(PIC_1_COMMAND, PIC_EOI);
    }
}

/// Remaps the master and slave PICs
/// 
/// # Safety
/// Must only be called during early interrupt-controller initialization. Doing
/// this while interrupts are enabled could race with incoming IRQs.
unsafe fn remap(master_offset: u8, slave_offset: u8) {
    // SAFETY: Reading current masks from PIC data ports is part of the standard
    // remap sequence.
    let saved_master_mask = unsafe { inb(PIC_1_DATA) };
    let saved_slave_mask = unsafe { inb(PIC_2_DATA) };

    // Start initialization sequence in cascade mode
    unsafe {
        outb(PIC_1_COMMAND, ICW1_INIT | ICW1_ICW4);
        io_wait();
        outb(PIC_2_COMMAND, ICW1_INIT | ICW1_ICW4);
        io_wait();

        // Set vector offsets
        outb(PIC_1_DATA, master_offset);
        io_wait();
        outb(PIC_2_DATA, slave_offset);
        io_wait();

        // Tell master that slave is on IRQ2
        outb(PIC_1_DATA, 0x04);
        io_wait();

        // Tell slave its cascade identity
        outb(PIC_2_DATA, 0x02);
        io_wait();

        // Use 8086/88 mode
        outb(PIC_1_DATA, ICW4_8086);
        io_wait();
        outb(PIC_2_DATA, ICW4_8086);
        io_wait();

        // Restore masks temporarily. `init()` will apply the final mask
        outb(PIC_1_DATA, saved_master_mask);
        outb(PIC_2_DATA, saved_slave_mask);
    }
}

/// Current PIC interrupt masks
#[derive(Debug, Clone)]
pub struct PicMasks {
    /// Master PIC interrupt mask register
    pub master: u8,

    /// Slave PIC interrupt mask register
    pub slave: u8
}

/// Reads the current PIC interrupt masks
pub fn read_masks() -> PicMasks {
    // SAFETY: Reading PIC data ports is valid after the legacy PIC exists.
    // This is a diagnostic helper and does not modify controller state.
    unsafe {
        PicMasks {
            master: inb(PIC_1_DATA),
            slave: inb(PIC_2_DATA)
        }
    }
}

/// Logs the current PIC interrupt masks
pub fn log_masks(label: &str) {
    let masks = read_masks();

    crate::kprintln!(
        "[NX][PIC] masks {}: master={:#010b} slave={:#010b}",
        label,
        masks.master,
        masks.slave
    );
}

/// Current PIC interrupt request and in-service state
/// 
/// The IRR bits show which IRQ lines are requesting service.
/// The ISR bits show which IRQ lines are currently considered in service.
#[derive(Debug, Clone, Copy)]
pub struct PicIrqState {
    /// Master PIC interrupt request register
    pub master_irr: u8,

    /// Slave PIC interrupt request register
    pub slave_irr: u8,

    /// Master PIC in-service register
    pub master_isr: u8,

    /// Slave PIC in-service register
    pub slave_isr: u8
}

/// Reads the PIC interrupt request and in-service registers
/// 
/// This is a diagnostic helper for debugging whether a legacy IRQ reaches the
/// PIC and whether the PIC considers it already in service.
pub fn read_irq_state() -> PicIrqState {
    // SAFETY: Reading IRR/ISR uses the standard OCW3 PIC command sequence. This
    // helper is diagnostic and does not change interrupt masks or vector
    // offsets.
    unsafe {
        outb(PIC_1_COMMAND, PIC_READ_IRR);
        outb(PIC_2_COMMAND, PIC_READ_IRR);

        let master_irr = inb(PIC_1_COMMAND);
        let slave_irr = inb(PIC_2_COMMAND);

        outb(PIC_1_COMMAND, PIC_READ_ISR);
        outb(PIC_2_COMMAND, PIC_READ_ISR);

        let master_isr = inb(PIC_1_COMMAND);
        let slave_isr = inb(PIC_2_COMMAND);

        PicIrqState {
            master_irr,
            slave_irr,
            master_isr,
            slave_isr
        }
    }
}

/// Logs PIC IRR/ISR state
pub fn log_irq_state(label: &str) {
    let state = read_irq_state();

    crate::kprintln!(
        "[NX][PIC] irq state: {}: master_irr={:#010b} slave_irr={:#010b} master_isr={:#010b} slave_isr={:#010b}",
        label,
        state.master_irr,
        state.slave_irr,
        state.master_isr,
        state.slave_isr
    );
}