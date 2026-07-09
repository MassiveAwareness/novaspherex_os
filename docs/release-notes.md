# NovasphereX Release Notes

This document records versioned development history for NovasphereX OS.

The roadmap describes planned and completed technical capabilities. This file records what changed in each release.

---

## v0.0.6 - Memory Map and Physical Frame Allocator Groundwork

### Summary

This release introduced the first real memory-management foundation for NovasphereX.

The kernel now requests and reads the Limine memory map, logs physical memory regions, summarizes usable memory, initializes an early physical frame allocator and uses that allocator to provide page-table frames for paging/MMIO work.

This is an important architectural step: the paging subsystem no longer depends on a static kernel-embedded early page-table pool. New page tables used during Local APIC MMIO mapping are now backed by real physical frames discovered through the bootloader-provided memory map.

### Added

* Limine memory map request
* Limine memory map response and entry structures
* Memory map entry type constants
* Readable memory region type names
* Memory map response helper
* Memory map entry lookup helper
* `memory` subsystem module
* Early memory initialization path
* Memory map logging
* Usable memory summary logging
* Early physical frame allocator
* 4 KiB physical frame allocator
* Low-memory avoidance below the first 1 MiB
* Frame allocator state diagnostics
* Frame allocator initialization check
* Allocator-backed page-table frame allocation
* Page-table frame zeroing before use
* Paging error cases for allocator availability and physical frame exhaustion
* Memory initialization moved before architecture initialization

### Changed

* Boot order now initializes memory before the x86_64 architecture baseline
* `paging.rs` now requests new page-table frames from the physical frame allocator
* The previous static early page-table pool has been replaced for Local APIC MMIO mapping
* Local APIC MMIO mapping now uses allocator-provided physical frames for missing intermediate page tables
* Timer diagnostic logging can be throttled more aggressively to reduce serial output noise

### Validated

* Limine memory map is available during boot
* Memory map entries are readable and logged correctly
* Usable memory regions are detected
* Usable memory byte and MiB summaries are produced
* The early physical frame allocator initializes successfully
* The allocator starts from the first suitable usable frame above low memory
* Sequential 4 KiB frame allocation was validated
* Paging can allocate new page-table frames from the frame allocator
* Local APIC MMIO mapping still succeeds after replacing the static page-table pool
* Local APIC ID/version register reads still works
* Local APIC OEI backend still works
* Local APIC timer delivery remains stable
* Timer interrupts continue ater the kernel reaches the halt loop
* Retro16 background selection and rendering remain functional

### Notes

The physical frame allocator is still intentionally simple. It is a bootstrap allocator with no deallocation, no bitmap, no buddy allocator, and no reclamation of bootloader memory yet.

The allocator currently uses only `Usable` memory regions and skips memory below the first 1 MiB. This avoids low legacy memory while the memory subsystem is still primitive.

The Local APIC timer is still not calibrated. Timer interrupts are delivered successfully, but the current APIC timer initial count is still a delivery-proof value rather than a stable wall-clock frequency.

This release does not introduce a heap allocator yet. The next memory milestones should focus on a more complete physical frame allocator, dedicated MMIO mapping region, page-table abstraction cleanup and kernel heap support.

---

## v0.0.5 - Retro16 Boot Background Pipeline

### Summary

This release introduced the first real Retro16 visual boot experience. The previous simple framebuffer banner was replaced by an embedded boot background system that selects one of five Retro16-style images at startup.

### Added

* Source image asset directory under `assets/images`
* Five Retro16 boot background source images
* `prepare-assets.ps1` asset preparation script
* PNG-to-RGBX conversion pipeline
* Generated RGBX asset validation
* Conversion skipping when generated assets are already valid
* Optional forced asset regeneration
* Embedded boot background assets through `include_bytes!`
* Lightweight boot-time random background selection
* Inclusive background selection range `1..=5`
* Serial logging for selected boot background
* Raw RGBX framebuffer drawing
* Nearest-neighbor framebuffer scaling
* Automatic asset preparation before QEMU launch

### Validated

* Selected Retro16 background appears successfully during boot
* Kernel continues into CPU initialization after drawing the background
* Local APIC timer delivery remains functional
* Timer interrupts continue after reaching the halt loop

### Notes

The boot background selection currently uses a lightweight TSC-based seed. This is suitable for harmless visual variation, but it is not cryptographically secure randomness.

Runtime PNG loading is not implemented yet. The current design intentionally converts PNG assets into raw RGBX buffers before kernel build time.

---

## v0.0.4 - Local APIC Timer Delivery

### Summary

This release delivered a major hardware milestone: periodic hardware timer interrupts through the Local APIC.

Before this release, the timer vector path had been verified through a software interrupt and the PIT/PIC path had shown that IRQ0 reached the PIC IRR. However, hardware timer delivery to the CPU was not working through the legacy PIC path in the tested UEFI/QEMU environment.

v0.0.4 introduced minimal paging/MMIO mapping support so that the Local APIC MMIO page could be mapped and used directly.

### Added

* `read_cr3()` CPU helper
* `invlpg()` CPU helper
* Minimal paging helper module
* Active PML4 physical address detection
* Active page table walking
* Existing virtual-to-physical translation helper
* Tiny static early page-table pool
* 4 KiB page mapping helper
* MMIO page mapping helper
* Local APIC MMIO page mapping
* Local APIC ID/version register validation
* Local APIC software enable path
* Local APIC EOI backend
* Local APIC timer configuration
* Legacy PIC masking when APIC is active
* Timer backend selection between legacy PIC and Local APIC

### Validated

* Local APIC physical base is detected from `IA32_APIC_BASE`
* Local APIC MMIO page is mapped successfully
* APIC ID register can be read
* APIC version register can be read
* Local APIC EOI write path works
* Periodic Local APIC timer interrupts are delivered
* Timer interrupts wake the CPU from `hlt`
* Timer ticks continue after the kernel reaches the halt loop

### Notes

The Local APIC timer is not calibrated yet. The initial count is currently an experimental delivery-proof value, not a stable 100 Hz clock source.

---

## v0.0.3 - HHDM and Local APIC Groundwork

### Summary

This release introduced Limine HHDM support and prepared the kernel for Local APIC development.

The main purpose was to obtain a higher-half direct-map offset, compute physical-to-virtual address candidates, probe the Local APIC base MSR and prepare the timer code for multiple EOI backends.

### Added

* Limine HHDM request
* HHDM offset helper
* HHDM diagnostics
* Physical-to-virtual address helper
* Local APIC base MSR probe
* Local APIC physical base logging
* Local APIC virtual base candidate calculation
* Timer EOI backend abstraction
* APIC MMIO safety gate

### Validated

* Limine provides a valid HHDM offset
* Local APIC is present and enabled according to `IA32_APIC_BASE`
* Local APIC physical base is detected as `0xfee00000` in QEMU
* HHDM address calculation produces a virtual base candidate
* Direct APIC MMIO access through HHDM alone is not valid in the tested environment

### Notes

This release intentionally did not enable APIC MMIO writes by default. A later experiment confirmed that accessing `hhdm_offset + 0xfee00000` caused a not-present page fault, which motivated the minimal MMIO mapping work in v0.0.4.

---

## v0.0.2 - CPU Baseline and Exception Handling

### Summary

This release established the first serious x86_64 CPU baseline.

It introduced the kernel's own descriptor-table setup, recoverable breakpoint exceptions, fatal page fault diagnostics, CPU state logging and a documented panic halt path.

### Added

* Own GDT
* Limine-compatible temporary GDT selectors
* Own IDT
* Breakpoint exception handler
* Page fault handler
* Basic exception logging
* CPU state diagnostics
* Segment selector diagnostics
* `RFLAGS` diagnostics
* `CR2` diagnostics
* Halt-safe panic path
* Centralized CPU instruction helpers
* PIT/PIC timer draft
* Timer software-vector test
* Legacy PIC diagnostics
* Local APIC MSR probe groundwork
* Documentation baseline for architecture modules

### Validated

* The kernel can load its own GDT without breaking Limine-provided selectors
* The kernel can load its own IDT
* Breakpoint exception handler returns successfully
* Page fault handler logs fault address, error code, instruction pointer, code segment and flags
* Timer vector 32 works through a software interrupt
* PIT can raise IRQ0 into the legacy PIC IRR
* Legacy PIC delivery does not reach the CPU in the tested UEFI/QEMU configuration

### Notes

The GDT intentionally keeps Limine-compatible selectors valid. A full controlled transition to NovasphereX-owned selectors remains future work.

---

## v0.0.1 - Initial Boot Proof

### Summary

This release established the first bootable NovasphereX kernel.

The goal was to prove that a Rust `no_std` kernel could be built, loaded by Limine, run under QEMU/OVMF, log through serial, draw to the framebuffer and reach a stable halt loop.

### Added

* Rust `no_std` kernel structure
* Rust `no_main` entry point
* Custom linker script
* Limine boot protocol request markers
* Limine base revision check
* Limine bootloader info request
* Limine framebuffer request
* Serial COM1 logging
* Basic framebuffer banner
* Windows 11 PowerShell workflow
* QEMU launch script
* OVMF/EDK2 firmware discovery
* ESP preparation workflow

### Validated

* Kernel builds for `x86_64-unknown-none`
* Kernel boots through Limine
* Serial logging works
* Framebuffer drawing works
* Kernel reaches halt loop in QEMU

### Notes

This version was a boot proof, not a CPU or memory-management milestone.
