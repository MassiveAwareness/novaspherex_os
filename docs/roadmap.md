# NovasphereX Roadmap

## Milestone 0 — Boot proof

* [x] Rust `no_std` kernel entry point
* [x] Limine boot protocol request markers
* [x] Serial COM1 logging
* [x] Framebuffer presence check and simple banner drawing
* [x] Verified build on local Rust toolchain
* [x] Verified boot in QEMU

## Milestone 1 — CPU baseline

- [x] Own GDT
- [x] Own IDT
- [x] Breakpoint exception handler
- [ ] Page fault handler
- [ ] Basic exception logging
- [ ] Halt-safe panic path
- [ ] PIT/APIC timer interrupt draft

## Milestone 2 — Memory

* [ ] Read Limine memory map
* [ ] Classify usable and reserved memory regions
* [ ] Physical frame allocator
* [ ] Page table abstraction
* [ ] Higher-half direct map wrapper
* [ ] Kernel heap allocator
* [ ] Heap smoke test

## Milestone 3 — Execution

* [ ] Cooperative task abstraction
* [ ] Basic executor
* [ ] Preemptive scheduler draft
* [ ] Syscall ABI draft
* [ ] First userspace ELF loader
* [ ] Minimal init process

## Milestone 4 — Retro16 Graphics

* [ ] Define internal logical resolution strategy
* [ ] Add framebuffer abstraction layer
* [ ] Add pixel format conversion helpers
* [ ] Add rectangle fill primitive
* [ ] Add line drawing primitive
* [ ] Add nearest-neighbor integer scaling plan
* [ ] Add bitmap font renderer
* [ ] Add first 8×16 debug font
* [ ] Add boot screen with pixel art styling
* [ ] Add Retro16 system palette
* [ ] Add sprite blitting
* [ ] Add tile blitting
* [ ] Add simple UI panel primitive
* [ ] Add status bar primitive
* [ ] Add mouse cursor sprite
* [ ] Draft `gfx-core` module
* [ ] Draft `gfx-font` module
* [ ] Draft `gfx-blit` module
* [ ] Draft `theme-retro16` module
* [ ] Create first NovasphereX pixel logo

## Milestone 5 — Files and Devices

* [ ] Basic block device abstraction
* [ ] RAM disk support
* [ ] Simple read-only filesystem experiment
* [ ] Keyboard input driver
* [ ] Mouse input driver
* [ ] PCI enumeration draft
* [ ] Storage device discovery draft

## Milestone 6 — User Interface

* [ ] Text console over framebuffer
* [ ] Kernel debug shell
* [ ] Basic command parser
* [ ] Retro16 UI widgets
* [ ] Window primitive
* [ ] Simple compositor
* [ ] First graphical system monitor
* [ ] First graphical launcher

## Long-Term Direction

The NovasphereX graphical interface is pixel art based and inspired by the 16-bit console era, especially the Sega Genesis / Mega Drive aesthetic.

The system should not become a modern desktop with a retro skin. It should be designed from the ground up around pixel-perfect rendering, bitmap fonts, palette-conscious graphics, sprite-like UI objects, and a low-resolution-first interface model.