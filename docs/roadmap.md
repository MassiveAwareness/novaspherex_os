# NovasphereX Roadmap

This roadmap tracks the incremental development of NovasphereX OS.

The project currently follows a deliberately staged approach: first establish a stable bootable kernel, then validate CPU and interrupt-controller fundamentals, then build memory management, execution, devices, graphics, and higher-level user interface systems.

---

## Current Release State

```text
v0.0.6 — Memory Map and Physical Frame Allocator Groundwork
```

Current validated capabilities:

* Rust `no_std` kernel boots through Limine.
* Serial diagnostics work through COM1.
* Framebuffer output is available.
* Retro16 boot background assets are embedded into the kernel.
* One of five boot backgrounds is selected at boot using lightweight TSC-based variation.
* GDT and IDT are installed.
* Breakpoint and page fault exception paths work.
* Limine HHDM is available.
* Physical-to-virtual address helpers work through HHDM.
* Active `CR3` can be inspected.
* Minimal paging helpers can walk active page tables.
* Local APIC MMIO page can be mapped.
* Local APIC MMIO access is validated.
* Local APIC EOI backend works.
* Periodic Local APIC timer interrupts are delivered.
* The CPU wakes from `hlt` through timer interrupts.
* Limine memory map is requested and logged.
* Usable memory regions are detected and summarized.
* Early physical frame allocator initializes from the memory map.
* 4 KiB physical frames can be allocated.
* Paging can allocate page-table frames through the physical frame allocator.
* Local APIC MMIO mapping no longer depends on the old static page-table pool.

---

## Milestone 0 — Boot Proof

* [x] Rust `no_std` kernel entry point
* [x] Rust `no_main` bare-metal startup
* [x] Custom linker script
* [x] Limine boot protocol request markers
* [x] Limine base revision check
* [x] Limine bootloader info request
* [x] Limine framebuffer request
* [x] Serial COM1 logging
* [x] Framebuffer presence check
* [x] Initial framebuffer banner drawing
* [x] Verified build on local Rust toolchain
* [x] Verified boot in QEMU
* [x] Windows 11 PowerShell development workflow
* [x] QEMU + OVMF UEFI boot workflow
* [x] ESP directory preparation workflow

---

## Milestone 1 — CPU Baseline

* [x] Own GDT
* [x] Limine-compatible temporary GDT selectors
* [x] Own IDT
* [x] Breakpoint exception handler
* [x] Page fault handler
* [x] Basic exception logging
* [x] CPU state diagnostics
* [x] Segment selector diagnostics
* [x] `RFLAGS` diagnostics
* [x] `CR2` diagnostics
* [x] Halt-safe panic path
* [x] Centralized CPU instruction helpers
* [x] `cli`
* [x] `sti`
* [x] `hlt`
* [x] `read_cr2`
* [x] `read_cr3`
* [x] `invlpg`
* [x] `rdmsr`
* [x] `wrmsr`
* [x] `rdtsc`
* [x] Software interrupt test for breakpoint exception
* [x] Software interrupt test for timer vector
* [x] PIT/PIC initialization draft
* [x] PIT reaches PIC IRR bit 0
* [x] Legacy PIC diagnostics
* [x] Local APIC base MSR probe
* [x] Local APIC physical base detection
* [x] Local APIC virtual base candidate calculation
* [x] Local APIC MMIO mapping
* [x] Local APIC ID/version register validation
* [x] Local APIC software enable path
* [x] Local APIC EOI backend
* [x] Legacy PIC masking when APIC is active
* [x] Periodic Local APIC timer configuration
* [x] Hardware timer interrupt delivery through Local APIC
* [x] Timer interrupts wake CPU from `hlt`
* [ ] Calibrated timer frequency
* [ ] Separate interrupt stacks through TSS/IST
* [ ] Controlled transition from Limine selectors to NovasphereX selectors

---

## Milestone 2 — Memory

* [x] Limine HHDM request
* [x] HHDM offset helper
* [x] HHDM diagnostics
* [x] Physical-to-virtual address helper
* [x] Active `CR3` inspection
* [x] Active PML4 physical address detection
* [x] Minimal page table walk
* [x] Existing virtual-to-physical translation helper
* [x] Initial static page-table pool experiment
* [x] Single-page 4 KiB mapping helper
* [x] MMIO page mapping helper
* [x] Local APIC MMIO mapping proof
* [x] Read Limine memory map
* [x] Log Limine memory map entries
* [x] Classify memory map regions by Limine type
* [x] Summarize usable memory
* [x] Skip low memory below the first 1 MiB for early allocation
* [x] Early physical frame allocator
* [x] 4 KiB physical frame allocation
* [x] Frame allocator state diagnostics
* [x] Frame allocator smoke test
* [x] Memory subsystem initialization before architecture initialization
* [x] Allocator-backed page-table frame allocation
* [x] Page-table frame zeroing before use
* [x] Replace static page-table pool for Local APIC MMIO mapping
* [x] Paging error cases for missing allocator or exhausted frames
* [x] Preserve bootloader memory map diagnostics
* [x] Allocation failure diagnostics for early frame allocation
* [x] Basic memory statistics logging
* [ ] General page table abstraction
* [ ] Page table entry flag abstraction
* [ ] Kernel virtual memory map layout
* [ ] Dedicated MMIO virtual region
* [ ] Proper MMIO mapping API
* [ ] Higher-half direct map wrapper
* [ ] Frame deallocation strategy
* [ ] Physical frame allocator bookkeeping structure
* [ ] Bootloader-reclaimable memory policy
* [ ] Kernel heap allocator
* [ ] Heap smoke test
* [ ] Allocation failure path for heap allocations
* [ ] Memory statistics expansion

---

## Milestone 3 — Execution

* [ ] Cooperative task abstraction
* [ ] Basic executor
* [ ] Kernel task structure
* [ ] Kernel stack allocation strategy
* [ ] Context switch prototype
* [ ] Timer-driven scheduler hook
* [ ] Preemptive scheduler draft
* [ ] Basic synchronization primitives
* [ ] Syscall ABI draft
* [ ] User/kernel privilege boundary plan
* [ ] First userspace ELF loader
* [ ] Minimal init process
* [ ] Userspace panic/exit path
* [ ] Process table draft

---

## Milestone 4 — Retro16 Graphics

* [x] Long-term Retro16 visual direction defined
* [x] Sega Genesis / Mega Drive inspired aesthetic recorded
* [x] Pixel-art-first graphical philosophy documented
* [x] Source asset directory: `assets/images`
* [x] Retro16 boot background source images
* [x] Asset preparation script
* [x] PNG-to-RGBX conversion pipeline
* [x] Generated RGBX asset validation
* [x] Skip conversion when generated assets are valid
* [x] Optional forced asset regeneration
* [x] QEMU run workflow prepares assets before kernel build
* [x] Embedded boot background assets through `include_bytes!`
* [x] Boot-time random background selection in inclusive range `1..=5`
* [x] Selected background serial logging
* [x] Raw RGBX framebuffer drawing
* [x] Nearest-neighbor framebuffer scaling
* [x] Boot banner replaced by Retro16 background image
* [ ] Define internal logical resolution strategy
* [ ] Add framebuffer abstraction layer
* [ ] Add pixel format conversion module
* [ ] Add rectangle fill primitive
* [ ] Add line drawing primitive
* [ ] Add nearest-neighbor integer scaling module
* [ ] Add bitmap font renderer
* [ ] Add first 8×16 debug font
* [ ] Add Retro16 system palette
* [ ] Add palette remapping experiment
* [ ] Add sprite blitting
* [ ] Add tile blitting
* [ ] Add simple UI panel primitive
* [ ] Add status bar primitive
* [ ] Add mouse cursor sprite
* [ ] Draft `gfx-core` module
* [ ] Draft `gfx-font` module
* [ ] Draft `gfx-blit` module
* [ ] Draft `gfx-ui` module
* [ ] Draft `theme-retro16` module
* [ ] Create first NovasphereX pixel logo

---

## Milestone 5 — Files and Devices

* [ ] Basic device model
* [ ] Basic block device abstraction
* [ ] RAM disk support
* [ ] Simple read-only filesystem experiment
* [ ] File loading API
* [ ] Runtime asset loading experiment
* [ ] Keyboard input driver
* [ ] Mouse input driver
* [ ] PS/2 controller investigation
* [ ] PCI enumeration draft
* [ ] PCI device configuration space access
* [ ] Storage device discovery draft
* [ ] AHCI investigation
* [ ] VirtIO device investigation

---

## Milestone 6 — User Interface

* [ ] Text console over framebuffer
* [ ] Kernel debug shell
* [ ] Basic command parser
* [ ] Keyboard-driven shell input
* [ ] Retro16 UI widgets
* [ ] Window primitive
* [ ] Panel primitive
* [ ] Menu primitive
* [ ] Simple compositor
* [ ] First graphical system monitor
* [ ] First graphical launcher
* [ ] Boot diagnostics screen
* [ ] Runtime theme switch experiment

---

## Milestone 7 — Build, Tooling, and Release Workflow

* [x] Windows-first PowerShell workflow
* [x] Kernel build script
* [x] Limine fetch/reuse script
* [x] ESP preparation script
* [x] QEMU launch script
* [x] Asset preparation script
* [x] Automatic asset validation before QEMU run
* [x] Generated asset size validation
* [x] QEMU path discovery
* [x] OVMF/EDK2 firmware discovery
* [x] `OVMF_CODE` override support
* [ ] Release checklist document
* [ ] Version bump checklist
* [ ] Automated boot smoke-test notes
* [ ] Debug/release profile distinction
* [ ] CI feasibility investigation
* [ ] Source archive script
* [ ] Generated asset policy document

---

## Release Notes

Release history is maintained separately in:

```text
docs/release-notes.md
```

The roadmap describes planned and completed technical capabilities, while release notes describe versioned project history.

---

## Near-Term Direction

The next major technical direction remains the memory subsystem, but the first foundation is now complete.

Completed in v0.0.6:

* parse and log the Limine memory map,
* classify memory regions,
* summarize usable memory,
* initialize an early physical frame allocator,
* allocate 4 KiB physical frames,
* use allocator-provided frames for page-table allocation,
* remove the static page-table pool from the Local APIC MMIO mapping path.

Recommended next focus:

* formalize page-table abstractions,
* introduce page-table entry flag types,
* define a kernel virtual memory map layout,
* create a dedicated MMIO virtual address region,
* turn the Local APIC mapping proof into a proper MMIO mapping API,
* improve physical frame allocator bookkeeping,
* decide how and when to reclaim bootloader-reclaimable memory,
* introduce a kernel heap allocator,
* add a heap smoke test.

This will turn the current early memory foundation into a more complete kernel memory-management subsystem.

---

## Long-Term Direction

The NovasphereX graphical interface is pixel art based and inspired by the 16-bit console era, especially the Sega Genesis / Mega Drive aesthetic.

The system should not become a modern desktop with a retro skin. It should be designed from the ground up around pixel-perfect rendering, bitmap fonts, palette-conscious graphics, sprite-like UI objects, and a low-resolution-first interface model.

The long-term vision is:

```text
A native Rust operating system designed as if the pixel-art era never ended.
```
