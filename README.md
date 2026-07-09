# NovasphereX OS

**NovasphereX** is an experimental operating system kernel written in Rust.

The project targets native execution on real x86_64 hardware in the long term. The current development and validation environment is based on Windows 11, PowerShell, QEMU, OVMF/EDK2 UEFI firmware, and the Limine boot protocol.

NovasphereX is not a Linux distribution, not a Unix clone, and not a userspace application. It is a from-scratch kernel experiment whose purpose is to construct, examine, and gradually evolve the fundamental components of an operating system: bootstrapping, CPU initialization, interrupt handling, memory discovery, paging, timer infrastructure, low-level diagnostics, graphics output, and eventually userspace facilities.

At the current stage, NovasphereX is best understood as a research-oriented kernel prototype with a deliberately distinctive visual direction.

---

## Current Version

```text
v0.0.5
```

Release title:

```text
v0.0.5 — Retro16 boot background pipeline
```

This version introduces the first real Retro16 visual boot experience. The previous simple framebuffer banner has been replaced by a build-time asset pipeline and an embedded boot background system.

The kernel now selects one of five prepared Retro16-style boot backgrounds at startup and draws it directly to the Limine-provided framebuffer.

Current validated capabilities include:

* Rust `no_std` kernel entry point,
* Limine boot protocol integration,
* serial logging through COM1,
* framebuffer output through Limine,
* embedded Retro16 boot background assets,
* build-time PNG-to-RGBX asset conversion,
* generated asset validation,
* boot-time background selection in the inclusive range `1..=5`,
* custom GDT,
* custom IDT,
* breakpoint exception handling,
* page fault handling,
* CPU state diagnostics,
* halt-safe panic path,
* documented x86_64 architecture layer,
* PIT/PIC legacy timer diagnostics,
* Limine HHDM request support,
* physical-to-virtual address helpers,
* active `CR3` inspection,
* minimal page table walking,
* early static page-table pool,
* Local APIC MMIO page mapping,
* Local APIC ID/version register validation,
* Local APIC EOI backend,
* periodic Local APIC timer delivery,
* timer interrupts waking the CPU from `hlt`.

The system currently boots successfully in QEMU, displays a randomly selected Retro16 boot background, initializes the CPU baseline, validates Local APIC timer delivery, runs a breakpoint exception smoke test, and reaches a stable halt loop while timer interrupts continue.

---

## Project Philosophy

The central goal of NovasphereX is to build an operating system as an explicit, inspectable, low-level system rather than as an opaque stack of abstractions.

The project follows several guiding principles.

First, the kernel should expose hardware reality rather than hide it prematurely. Descriptor tables, interrupt frames, model-specific registers, bootloader requests, page tables, MMIO registers, framebuffer metadata, and I/O ports are represented explicitly so that the system can be reasoned about from first principles.

Second, the implementation should favor correctness, observability, and documented assumptions over early generalization. A small, well-instrumented kernel is preferred over a larger but poorly understood kernel.

Third, unsafe operations are not avoided, because operating-system kernels necessarily require unsafe hardware access. Instead, unsafe operations are centralized, documented, and isolated behind minimal APIs.

Fourth, the project is designed to evolve incrementally. Each milestone should produce a system that still boots, logs useful diagnostics, and preserves previously validated behavior.

Fifth, the operating system should have a coherent visual identity from the earliest graphics work onward. NovasphereX is not intended to become a generic modern desktop with a retro theme applied later. Its graphical direction is part of the system design.

---

## Long-Term Visual Direction

NovasphereX is intended to develop a distinctive graphical identity.

The long-term graphical interface is planned as a pixel-art-oriented, retro-futuristic environment inspired by the 16-bit console era, especially the Sega Genesis / Mega Drive aesthetic.

This is not intended to be a modern desktop with a retro skin. The graphical stack should eventually be designed around:

* pixel-perfect rendering,
* low-resolution-first UI design,
* integer scaling,
* nearest-neighbor scaling,
* bitmap fonts,
* sprite-like UI elements,
* tile-based visual composition,
* limited and deliberate color palettes,
* custom 2D framebuffer rendering,
* Retro16-style themes and system visuals.

The current kernel does not yet implement a full GUI. However, it now includes an early proof of the visual direction: embedded Retro16 boot backgrounds rendered directly to the framebuffer.

The guiding sentence is:

```text
NovasphereX is not a modern operating system with a retro theme.

NovasphereX is an operating system designed as if the pixel-art era never ended.
```

---

## Host Development Environment

The primary host environment is:

```text
Windows 11
PowerShell
Rustup
QEMU for Windows
OVMF/EDK2 UEFI firmware
Limine bootloader
```

The project is deliberately kept compatible with a Windows-first workflow. Build, asset preparation, ESP preparation, and execution scripts are PowerShell-based rather than Bash-based.

---

## Required Tools

### Rust

Install Rust using Rustup:

```powershell
winget install Rustlang.Rustup
```

Then install the bare-metal target used by the kernel:

```powershell
rustup default stable
rustup target add x86_64-unknown-none
```

The kernel currently builds for:

```text
x86_64-unknown-none
```

This target does not provide a standard library, operating-system runtime, process model, allocator, filesystem access, or host APIs. The kernel therefore uses:

```rust
#![no_std]
#![no_main]
```

---

### QEMU

Install QEMU for Windows:

```powershell
winget install -e --id SoftwareFreedomConservancy.QEMU
```

Verify that QEMU is visible from PowerShell:

```powershell
qemu-system-x86_64.exe --version
```

If QEMU is not in `PATH`, either add it manually or ensure that `tools/run-qemu.ps1` can locate it in one of the known installation directories.

---

### OVMF / EDK2 Firmware

NovasphereX boots through UEFI. QEMU therefore requires an OVMF/EDK2 firmware image, typically named one of the following:

```text
edk2-x86_64-code.fd
OVMF_CODE.fd
OVMF.fd
```

The run script attempts to find this firmware automatically.

If automatic detection fails, set the `OVMF_CODE` environment variable:

```powershell
$env:OVMF_CODE = "C:\Path\To\edk2-x86_64-code.fd"
.\tools\run-qemu.ps1
```

For a persistent user-level setting:

```powershell
[Environment]::SetEnvironmentVariable(
  "OVMF_CODE",
  "C:\Path\To\edk2-x86_64-code.fd",
  "User"
)
```

Open a new PowerShell window after setting it persistently.

---

## Running the Kernel

From the project root:

```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
.\tools\run-qemu.ps1
```

The script performs the following actions:

1. validates and prepares embedded Retro16 boot background assets,
2. ensures the Rust target is installed,
3. builds the kernel in release mode,
4. downloads or reuses the Limine binary release,
5. prepares a virtual UEFI ESP directory,
6. locates QEMU,
7. locates OVMF/EDK2 firmware,
8. launches QEMU with the ESP directory exposed as a FAT drive,
9. redirects serial output to the PowerShell terminal.

Expected high-level output includes:

```text
Preparing assets...
Validated bg_1.rgbx
Validated bg_2.rgbx
Validated bg_3.rgbx
Validated bg_4.rgbx
Validated bg_5.rgbx
Background assets ready. Validated: 5, converted: 0.
Installing Rust target: x86_64-unknown-none
Building NovasphereX kernel...
Prepared UEFI ESP at ...
QEMU: ...
OVMF: ...
ESP: ...
[NX] NovasphereX kernel booted
[NX] target: x86_64-unknown-none
[NX] runtime: no_std
[NX] boot protocol: Limine
[NX] Limine base revision supported
[NX] bootloader: Limine 12.3.3
[NX][BOOT] selected Retro16 background bg_3.png
[NX] initializing CPU baseline
...
[NX][APIC] MMIO ready
[NX][TIMER] using Local APIC timer
[NX][TIMER] tick 1
...
[NX] reached halt loop
```

Some OVMF debug output may appear interleaved with NovasphereX serial logs. This is normal because both firmware and the kernel can write to the serial device used by QEMU.

---

## Repository Layout

Current high-level structure:

```text
novaspherex-os/
  .cargo/
    config.toml

  assets/
    images/
      bg_1.png
      bg_2.png
      bg_3.png
      bg_4.png
      bg_5.png

  boot/
    limine.conf

  docs/
    code-style.md
    release-notes.md
    roadmap.md
    vision.md

  kernel/
    Cargo.toml
    linker.ld
    src/
      main.rs
      limine.rs
      serial.rs

      assets/
        mod.rs
        boot_background.rs
        generated/
          bg_1.rgbx
          bg_2.rgbx
          bg_3.rgbx
          bg_4.rgbx
          bg_5.rgbx

      arch/
        mod.rs
        x86_64/
          mod.rs
          addr.rs
          apic.rs
          cpu.rs
          gdt.rs
          idt.rs
          interrupts.rs
          paging.rs
          pic.rs
          pit.rs
          port.rs
          timer.rs

  tools/
    build.ps1
    clean.ps1
    fetch-limine.ps1
    prepare-assets.ps1
    prepare-esp.ps1
    run-qemu.ps1

  Cargo.toml
  rust-toolchain.toml
  README.md
```

The structure separates boot protocol support, serial diagnostics, embedded assets, architecture-independent wrappers, x86_64-specific hardware mechanisms, and host-side development tooling.

---

## Boot Model

NovasphereX currently boots through Limine in UEFI mode.

The boot process is conceptually:

```text
UEFI firmware
  -> Limine bootloader
    -> NovasphereX ELF kernel
      -> _start()
        -> serial initialization
        -> Limine response validation
        -> bootloader info logging
        -> boot-time background selection
        -> Retro16 background framebuffer drawing
        -> x86_64 CPU baseline initialization
        -> HHDM diagnostics
        -> Local APIC probe
        -> minimal paging/MMIO mapping
        -> Local APIC MMIO validation
        -> Local APIC timer initialization
        -> breakpoint exception smoke test
        -> halt loop
```

Limine is responsible for loading the kernel ELF and filling in static boot protocol request responses. The kernel currently requests:

* base revision information,
* bootloader information,
* framebuffer information,
* HHDM information.

The kernel then reads these responses during early boot.

---

## Limine Integration

The file:

```text
kernel/src/limine.rs
```

contains minimal Limine boot protocol bindings.

The module defines request structures and places them into special linker sections:

```text
.limine_requests_start
.limine_requests
.limine_requests_end
```

The compiler must not discard these request objects even if Rust code does not reference them directly, so they are marked with:

```rust
#[used]
```

The current Limine requests include:

* base revision request,
* bootloader information request,
* framebuffer request,
* HHDM request.

The HHDM request allows the kernel to obtain the higher-half direct-map offset supplied by Limine.

Example diagnostic output:

```text
[NX][HHDM] offset=0xffff800000000000
```

A physical address can then be converted into a candidate direct-map virtual address:

```text
virtual = hhdm_offset + physical
```

However, this is an address calculation, not a proof that every physical range is safely mapped or suitable for ordinary memory access. This distinction became important during Local APIC development: the Local APIC MMIO physical page required explicit mapping before it could be accessed safely.

---

## Serial Logging

The first reliable diagnostic output mechanism is COM1 serial logging.

The file:

```text
kernel/src/serial.rs
```

initializes the legacy UART at:

```text
COM1 = 0x3f8
```

The logging macros:

```rust
kprint!()
kprintln!()
```

write formatted text to the serial device.

QEMU is launched with:

```text
-serial stdio
```

so serial logs appear directly in the PowerShell terminal.

The serial driver expands newline bytes to CRLF:

```text
\n -> \r\n
```

This improves readability in terminal environments that expect carriage return and line feed pairs.

Current log prefixes include:

```text
[NX]          general boot log
[NX][BOOT]   boot-stage visual or asset selection log
[NX][ARCH]   architecture initialization
[NX][CPU]    CPU state and CPU helper diagnostics
[NX][GDT]    Global Descriptor Table
[NX][IDT]    Interrupt Descriptor Table
[NX][INT]    interrupt and exception handling
[NX][PANIC]  kernel panic path
[NX][HHDM]   Limine HHDM diagnostics
[NX][PAGING] paging and address translation
[NX][APIC]   Local APIC diagnostics
[NX][PIC]    legacy PIC diagnostics
[NX][PIT]    legacy PIT diagnostics
[NX][TIMER]  timer subsystem
```

---

## Retro16 Boot Background Pipeline

NovasphereX v0.0.5 introduces an embedded Retro16 boot background pipeline.

Source PNG files are stored in:

```text
assets/images/
```

Expected source files:

```text
bg_1.png
bg_2.png
bg_3.png
bg_4.png
bg_5.png
```

The host-side preparation script:

```text
tools/prepare-assets.ps1
```

converts these PNG images into raw RGBX buffers stored under:

```text
kernel/src/assets/generated/
```

Generated files:

```text
bg_1.rgbx
bg_2.rgbx
bg_3.rgbx
bg_4.rgbx
bg_5.rgbx
```

The generated RGBX format is:

```text
byte 0: red
byte 1: green
byte 2: blue
byte 3: unused padding
```

The current generated boot background dimensions are:

```text
640 × 360 × 4 bytes
```

The kernel embeds these generated files with `include_bytes!`, selects one at boot using a lightweight TSC-based seed, and draws it to the framebuffer using nearest-neighbor scaling.

The boot log shows the selected background:

```text
[NX][BOOT] selected Retro16 background bg_3.png
```

The current selection mechanism is intentionally non-cryptographic. It is suitable for harmless visual variation, not for security-sensitive randomness.

---

## Asset Preparation

The asset preparation script validates generated assets before conversion.

If the generated RGBX files already exist and have the expected byte length, conversion is skipped.

Typical output:

```text
Preparing assets...
Validated bg_1.rgbx
Validated bg_2.rgbx
Validated bg_3.rgbx
Validated bg_4.rgbx
Validated bg_5.rgbx
Background assets ready. Validated: 5, converted: 0.
```

If an output file is missing or has the wrong size, it is regenerated.

To force regeneration manually:

```powershell
.\tools\prepare-assets.ps1 -Force
```

This is useful after replacing one or more source PNG files.

The QEMU run workflow invokes asset preparation automatically before building the kernel, because the generated RGBX files must exist before Rust evaluates `include_bytes!`.

---

## Framebuffer Graphics

Early graphics are handled through the framebuffer provided by Limine.

The current framebuffer path can:

* check whether a framebuffer is available,
* draw raw RGBX image data,
* pack source RGB values into the framebuffer pixel format,
* respect framebuffer pitch,
* scale the source image to the physical framebuffer,
* use nearest-neighbor sampling to preserve a pixel-art appearance.

This is not yet a full graphics stack. It is an early boot graphics path.

Future work will move toward:

* framebuffer abstraction,
* primitive drawing,
* bitmap font rendering,
* palette-aware rendering,
* sprite blitting,
* tile blitting,
* UI panel primitives,
* Retro16 system theme modules.

---

## CPU Baseline

The x86_64 architecture layer is located under:

```text
kernel/src/arch/x86_64/
```

The baseline currently initializes and validates:

* GDT,
* IDT,
* exception handlers,
* CPU diagnostic helpers,
* HHDM-based address calculation,
* minimal active page-table walking,
* Local APIC probe,
* Local APIC MMIO mapping,
* Local APIC timer delivery,
* legacy PIT/PIC diagnostics.

The architecture-independent entry points are exposed through:

```text
kernel/src/arch/mod.rs
```

Higher-level code should call architecture-level wrappers where possible rather than directly depending on deep x86_64 modules.

---

## CPU Helper Layer

The file:

```text
kernel/src/arch/x86_64/cpu.rs
```

centralizes low-level CPU instructions.

It currently provides helpers such as:

```rust
cli()
sti()
hlt()
halt_loop()
panic_halt_loop()
read_cs()
read_ss()
read_ds()
read_es()
read_rflags()
read_cr2()
read_cr3()
read_state()
log_state()
read_msr()
write_msr()
read_tsc()
invlpg()
```

The purpose of this module is to keep privileged instructions and inline assembly out of high-level kernel code.

For example, instead of writing:

```rust
unsafe {
    core::arch::asm!("hlt");
}
```

higher-level code uses:

```rust
arch::halt_loop();
```

This is a deliberate design decision: privileged CPU operations should be centralized, documented, and audited.

---

## GDT

The Global Descriptor Table is implemented in:

```text
kernel/src/arch/x86_64/gdt.rs
```

In x86_64 long mode, segmentation is mostly disabled, but the CPU still requires valid descriptors for code, data, stack, privilege transitions, and interrupt gates.

NovasphereX currently installs its own early GDT. It contains:

```text
0x00 -> null descriptor
0x08 -> NovasphereX kernel code descriptor
0x10 -> NovasphereX kernel data descriptor
0x18 -> reserved
0x20 -> reserved
0x28 -> Limine-compatible code descriptor
0x30 -> Limine-compatible data/stack descriptor
```

The `0x28` and `0x30` entries are temporary compatibility descriptors.

During debugging, the observed Limine-provided selectors were:

```text
CS = 0x28
SS = 0x30
DS = 0x30
ES = 0x30
```

Therefore, after installing our own GDT, these selector values must remain valid until the kernel performs a controlled transition into its own `CS=0x08` and `SS=0x10` execution context.

At present, the kernel loads the GDT but intentionally does not reload `CS`.

---

## IDT

The Interrupt Descriptor Table is implemented in:

```text
kernel/src/arch/x86_64/idt.rs
```

The current IDT installs handlers for:

```text
vector 3   -> breakpoint exception
vector 14  -> page fault exception
vector 32  -> timer interrupt vector
```

The breakpoint and page fault handlers validate that CPU exceptions correctly reach the kernel.

The timer vector is now used by the Local APIC timer path. Hardware timer delivery through the Local APIC has been validated.

---

## Interrupt and Exception Handling

The low-level interrupt stubs are implemented in:

```text
kernel/src/arch/x86_64/interrupts.rs
```

This module contains assembly entry points for:

```text
nx_isr_breakpoint
nx_isr_page_fault
nx_isr_timer
```

The assembly stubs are responsible for:

* clearing the direction flag,
* saving general-purpose registers,
* aligning the stack before calling Rust,
* constructing or locating the CPU-pushed interrupt frame,
* calling a Rust handler,
* restoring registers,
* returning with `iretq` when appropriate.

The breakpoint handler is recoverable:

```text
int3 -> handler -> iretq -> kernel continues
```

The page fault handler is currently fatal:

```text
page fault -> log CR2/error code/frame -> disable interrupts -> halt forever
```

A successful breakpoint test produces output similar to:

```text
[NX] triggering breakpoint exception test
[NX][INT] breakpoint exception at rip=0xffffffff80001788, cs=0x0028, flags=0x0000000000000286
[NX] breakpoint exception returned successfully
```

---

## Page Fault Diagnostics

The page fault path reads `CR2` to identify the faulting virtual address.

A controlled page fault test can be enabled through a development flag in `main.rs`.

Expected diagnostic shape:

```text
[NX][INT] PAGE FAULT
[NX][INT] fault address: 0x00000000deadbeef
[NX][INT] error code:    0x0000000000000000
[NX][INT] rip:           ...
[NX][INT] cs:            ...
[NX][INT] flags:         ...
[NX][INT] halting after page fault
```

This test is intentionally fatal and should remain disabled during normal development boots.

During v0.0.4 development, the page fault handler was used to confirm that the Local APIC MMIO page was not mapped by HHDM alone. That result directly motivated the minimal paging/MMIO mapping layer.

---

## Halt-Safe Panic Path

The panic handler logs the panic and CPU state, then enters a halt loop with maskable interrupts disabled.

Conceptually:

```text
panic
  -> serial log
  -> CPU state dump
  -> cli()
  -> hlt loop
```

This prevents the kernel from continuing to service timer or device interrupts after a fatal condition.

---

## HHDM Address Helpers

The file:

```text
kernel/src/arch/x86_64/addr.rs
```

contains early physical-to-virtual address helpers based on Limine HHDM.

It provides:

```rust
hhdm_offset()
phys_to_virt_addr(physical: u64) -> Option<u64>
phys_to_virt<T>(physical: u64) -> Option<*const T>
phys_to_virt_mut<T>(physical: u64) -> Option<*mut T>
```

These helpers perform address calculation only.

They do not prove:

* that the target physical page is mapped,
* that the region is RAM,
* that the region is MMIO,
* that the region is safe to dereference,
* that cache attributes are correct.

For ordinary RAM regions described by the bootloader memory map, HHDM will be central to the upcoming memory subsystem. For device MMIO regions such as the Local APIC, explicit mapping is required.

---

## Minimal Paging and MMIO Mapping

The file:

```text
kernel/src/arch/x86_64/paging.rs
```

contains the current minimal paging helper layer.

This is not yet a full memory manager. It exists to support early boot experiments, especially Local APIC MMIO mapping.

Current capabilities include:

* reading the active `CR3`,
* detecting the active PML4 physical address,
* accessing page tables through HHDM,
* walking existing page tables,
* translating existing virtual addresses to physical addresses,
* allocating from a tiny static early page-table pool,
* mapping a single 4 KiB page,
* mapping a single MMIO page,
* invalidating the relevant TLB entry with `invlpg`.

The Local APIC MMIO page is mapped as:

```text
physical 0xfee00000 -> virtual hhdm_offset + 0xfee00000
```

The mapping uses page-table flags appropriate for MMIO-style access:

```text
present
writable
write-through
cache-disable
```

This temporary paging layer must eventually be replaced or absorbed into the real memory subsystem.

---

## Legacy PIT/PIC Timer Experiment

NovasphereX contains a documented PIT/PIC experiment.

Relevant files:

```text
kernel/src/arch/x86_64/pic.rs
kernel/src/arch/x86_64/pit.rs
kernel/src/arch/x86_64/timer.rs
```

The legacy 8259 PIC is remapped:

```text
IRQ0..IRQ7   -> vectors 32..39
IRQ8..IRQ15  -> vectors 40..47
```

The PIT can be configured to approximately:

```text
100 Hz
```

The timer interrupt vector path was verified using a software interrupt:

```text
int 32 -> nx_isr_timer -> nx_timer_handler -> iretq
```

This proved:

* IDT vector 32 is valid,
* the timer assembly stub works,
* the Rust timer handler works,
* the timer tick counter works,
* the interrupt return path works.

However, hardware PIT/PIC delivery did not reach the CPU in the tested UEFI/QEMU environment. Diagnostics showed that PIT IRQ0 reached the PIC IRR, but was not delivered as a CPU interrupt.

This result motivated the transition to the Local APIC timer path.

---

## Local APIC

The Local APIC implementation is located in:

```text
kernel/src/arch/x86_64/apic.rs
```

The APIC implementation currently:

* reads `IA32_APIC_BASE`,
* detects the Local APIC physical base,
* detects whether the APIC is enabled,
* detects whether the CPU is the bootstrap processor,
* computes a virtual base candidate,
* validates APIC MMIO access after paging maps the APIC page,
* reads APIC ID/version registers,
* enables the APIC through the spurious interrupt vector register,
* sends APIC EOI commands,
* configures the Local APIC timer in periodic mode,
* logs Local APIC timer state.

Observed QEMU output:

```text
[NX][APIC] IA32_APIC_BASE=0x00000000fee00900
[NX][APIC] physical_base=0x00000000fee00000 enabled=true bsp=true
[NX][APIC] virtual_base_candidate=0xffff8000fee00000
[NX][PAGING] MMIO page mapped
[NX][APIC] id_register=0x00000000
[NX][APIC] version_register=0x00050014
[NX][APIC] MMIO ready
```

The physical APIC MMIO base in the tested QEMU environment is:

```text
0xfee00000
```

---

## Local APIC Timer

NovasphereX now has working hardware timer interrupt delivery through the Local APIC.

The timer subsystem is located in:

```text
kernel/src/arch/x86_64/timer.rs
```

The timer handler receives interrupts on vector:

```text
32
```

The timer module supports two EOI backends:

```rust
EoiBackend::LegacyPic
EoiBackend::LocalApic
```

At the current stage, once Local APIC MMIO access is validated, the timer path selects:

```text
LocalApic
```

Observed successful output:

```text
[NX][TIMER] using Local APIC timer
[NX][TIMER] EOI backend set to LocalApic
[NX][APIC] configuring Local APIC timer: vector=32 initial_count=1000000
[NX][APIC] Local APIC timer configured
[NX][CPU] enabling interrupts
[NX][TIMER] tick 1
[NX][TIMER] tick 2
[NX][TIMER] tick 3
[NX][TIMER] tick 4
[NX][TIMER] tick 5
[NX][TIMER] tick 100
[NX][TIMER] tick 200
```

This proves:

* Local APIC MMIO access works,
* APIC EOI works,
* the timer vector works,
* periodic hardware timer interrupts are delivered,
* the CPU wakes from `hlt`,
* timer interrupts continue after the kernel reaches the halt loop.

The Local APIC timer is not calibrated yet. The current initial count is a delivery-proof value, not a stable wall-clock frequency.

---

## Current Boot Output

A successful v0.0.5 boot produces output broadly similar to:

```text
Preparing assets...
Validated bg_1.rgbx
Validated bg_2.rgbx
Validated bg_3.rgbx
Validated bg_4.rgbx
Validated bg_5.rgbx
Background assets ready. Validated: 5, converted: 0.
Installing Rust target: x86_64-unknown-none
Building NovasphereX kernel...
Prepared UEFI ESP at ...
QEMU: ...
OVMF: ...
ESP:  ...
[NX] NovasphereX kernel booted
[NX] target: x86_64-unknown-none
[NX] runtime: no_std
[NX] boot protocol: Limine
[NX] Limine base revision supported
[NX] bootloader: Limine 12.3.3
[NX][BOOT] selected Retro16 background bg_3.png
[NX] initializing CPU baseline
[NX][ARCH] x86_64 init begin
[NX][CPU] state: before GDT
[NX][GDT] loading
[NX][GDT] loaded
[NX][IDT] loaded
[NX][HHDM] offset=0xffff800000000000
[NX][APIC] probing Local APIC
[NX][PAGING] active_pml4_physical=...
[NX][PAGING] mapping Local APIC MMIO physical=0x00000000fee00000 virtual=0xffff8000fee00000
[NX][PAGING] MMIO page mapped
[NX][APIC] MMIO ready
[NX][PIC] masking all legacy IRQs
[NX][TIMER] using Local APIC timer
[NX][TIMER] EOI backend set to LocalApic
[NX][APIC] Local APIC timer configured
[NX][CPU] enabling interrupts
[NX][TIMER] tick 1
[NX][TIMER] tick 2
[NX][ARCH] x86_64 init complete
[NX] triggering breakpoint exception test
[NX][INT] breakpoint exception at rip=..., cs=0x0028, flags=...
[NX] breakpoint exception returned successfully
[NX] reached halt loop
[NX][TIMER] tick 400
[NX][TIMER] tick 500
```

Exact addresses and selected background IDs may differ between boots and builds.

---

## Documentation Policy

New kernel code is expected to be documented at the time it is written.

The project follows these conventions:

* module-level documentation uses `//!`,
* public items use `///`,
* unsafe blocks include `SAFETY:` comments when non-trivial,
* assembly code includes stack-layout comments,
* hardware constants are named and documented,
* temporary bootstrap decisions are explicitly marked,
* debug helpers are labeled as such,
* paging and MMIO assumptions are documented,
* asset pipeline assumptions are documented.

The documentation goal is not to comment every line. The goal is to record why hardware-facing code is correct, necessary, temporary, or intentionally constrained.

See:

```text
docs/code-style.md
```

---

## Project Documentation

Additional documentation is maintained under:

```text
docs/
```

Important documents:

```text
docs/vision.md
```

Describes the long-term identity, Retro16 visual direction, rendering philosophy, boot graphics direction, and system design philosophy.

```text
docs/roadmap.md
```

Tracks planned and completed technical capabilities across boot, CPU, memory, execution, graphics, devices, UI, and tooling.

```text
docs/release-notes.md
```

Records versioned project history.

```text
docs/code-style.md
```

Defines coding and documentation conventions for kernel modules, unsafe blocks, assembly, paging, MMIO, framebuffer code, assets, and tooling.

---

## Known Limitations

The current kernel has several important limitations.

There is no general physical frame allocator yet.

There is no kernel heap allocator.

There is no scheduler.

There is no userspace.

There is no filesystem.

There is no runtime file loader.

There is no runtime PNG decoder.

There is no keyboard or mouse driver.

There is no general device model.

There is no PCI enumeration yet.

There is no calibrated wall-clock timer.

There is no TSS/IST-based separate interrupt stack setup yet.

There is no fully general page table abstraction yet.

There is no dedicated MMIO virtual address region yet.

The current paging code uses a tiny static early page-table pool and should not be treated as a full memory-management subsystem.

The Local APIC timer works, but its frequency is not calibrated.

The GDT still retains Limine-compatible selectors because the kernel has not yet performed a full controlled transition into its own code and stack segment selectors.

The boot background system embeds preconverted raw RGBX assets into the kernel. Runtime asset loading belongs to a later filesystem and memory-management milestone.

---

## Development Commands

Build the kernel:

```powershell
cargo build --release -p novaspherex_kernel
```

Prepare assets manually:

```powershell
.\tools\prepare-assets.ps1
```

Force asset regeneration:

```powershell
.\tools\prepare-assets.ps1 -Force
```

Run in QEMU:

```powershell
.\tools\run-qemu.ps1
```

Clean build artifacts:

```powershell
.\tools\clean.ps1
```

Create a source-only zip archive from the project root:

```powershell
$items = Get-ChildItem -Force |
  Where-Object {
    $_.Name -notin @("target", "third_party", ".git")
  }

Compress-Archive `
  -Path $items.FullName `
  -DestinationPath ..\novaspherex-os.zip `
  -Force
```

---

## Suggested Version Tags

Current suggested tag:

```powershell
git tag -a v0.0.5 -m "Embedded Retro16 boot backgrounds with asset preparation and random boot selection"
```

Suggested commit message:

```powershell
git commit -m "v0.0.5: add Retro16 boot background pipeline"
```

Previous major milestone tag:

```powershell
git tag -a v0.0.4 -m "Local APIC MMIO mapping, APIC EOI backend, and periodic APIC timer delivery"
```

Likely next target:

```text
v0.0.6 — Memory map and physical frame allocator groundwork
```

---

## Troubleshooting

### `Set-ExecutionPolicy` error

If this fails:

```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass .\tools\run-qemu.ps1
```

then the commands were accidentally written as one command.

Use either:

```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
.\tools\run-qemu.ps1
```

or:

```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass; .\tools\run-qemu.ps1
```

---

### OVMF firmware not found

Set the firmware path explicitly:

```powershell
$env:OVMF_CODE = "C:\Path\To\edk2-x86_64-code.fd"
.\tools\run-qemu.ps1
```

For a persistent user-level setting:

```powershell
[Environment]::SetEnvironmentVariable(
  "OVMF_CODE",
  "C:\Path\To\edk2-x86_64-code.fd",
  "User"
)
```

Then open a new PowerShell window.

---

### QEMU shows OVMF debug logs mixed with kernel logs

This is expected. OVMF and NovasphereX both write to the serial channel used by QEMU.

The kernel logs are prefixed with:

```text
[NX]
```

---

### Generated assets are missing

If `cargo build` fails because one of the generated RGBX files is missing, run:

```powershell
.\tools\prepare-assets.ps1
```

Then build again.

The normal QEMU workflow already does this automatically:

```powershell
.\tools\run-qemu.ps1
```

---

### Generated assets are stale or invalid

Force regeneration:

```powershell
.\tools\prepare-assets.ps1 -Force
```

This is useful after replacing source images under:

```text
assets/images/
```

---

### Source image is missing

The asset pipeline expects:

```text
assets/images/bg_1.png
assets/images/bg_2.png
assets/images/bg_3.png
assets/images/bg_4.png
assets/images/bg_5.png
```

If any of these are missing, `prepare-assets.ps1` will fail with a clear error.

---

### Timer ticks are too fast or not exactly 100 Hz

This is currently expected.

The Local APIC timer is working, but it is not calibrated yet. The current initial count is an interrupt-delivery proof value, not a calibrated clock source.

---

### Timer continues after `reached halt loop`

This is expected.

The CPU enters a halt loop, but Local APIC timer interrupts wake it periodically. The timer handler logs throttled tick output.

---

## Scientific Status Summary

NovasphereX v0.0.5 establishes a more capable experimental kernel substrate than a simple boot proof.

The system can boot through a modern UEFI bootloader, receive structured boot protocol data, perform early serial and framebuffer diagnostics, display an embedded Retro16 boot background, install fundamental CPU descriptor tables, handle architecturally significant exceptions, inspect CPU state, read model-specific registers, walk active page tables, map an MMIO page, validate Local APIC MMIO access, and receive periodic hardware timer interrupts through the Local APIC.

The timer research has progressed from a useful negative result in the legacy PIT/PIC path to a working modern Local APIC timer path. The kernel now has real hardware timer interrupt delivery, and the CPU wakes from `hlt` through timer interrupts.

The graphics work has also progressed from a simple framebuffer banner to a deterministic build-time asset pipeline with embedded Retro16 boot backgrounds. This marks the first visible step toward the long-term pixel-art operating system identity.

NovasphereX remains an early-stage kernel, but its current architecture already reflects the intended methodology of the project: incremental construction, observable state transitions, documented unsafe code, hardware-facing diagnostics, and measured progression from simple legacy mechanisms toward modern native operating-system facilities.

---

## Near-Term Direction

The next major technical direction is the memory subsystem.

Recommended next focus:

* parse the Limine memory map,
* classify usable and reserved memory regions,
* introduce a physical frame allocator,
* replace the temporary static page-table pool with real frame allocation,
* formalize page table mapping APIs,
* create a dedicated MMIO mapping region,
* start a kernel heap.

This will turn the current minimal paging experiment into a real memory-management foundation.
