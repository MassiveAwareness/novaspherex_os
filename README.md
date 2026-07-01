# NovasphereX OS

**NovasphereX** is an experimental operating system kernel written in Rust.
The project targets native execution on real x86_64 hardware in the long term, while the current development and validation environment is based on Windows 11, QEMU, OVMF/EDK2 UEFI firmware, and the Limine boot protocol.

The project is not a Linux distribution, not a Unix clone, and not a userspace application. It is a from-scratch kernel experiment whose purpose is to construct, examine, and gradually evolve the fundamental components of an operating system: bootstrapping, CPU initialization, interrupt handling, memory discovery, timer infrastructure, low-level diagnostics, and eventually graphical and userspace facilities.

At the current stage, NovasphereX is best understood as a research-oriented kernel prototype.

---

## Current Version

```text
v0.0.3
```

Recommended release title:

```text
v0.0.3 - HHDM and Local APIC groundwork
```

This version establishes the foundations for higher-half physical memory access and modern interrupt-controller development.

The current implementation includes:

* Rust `no_std` kernel entry point,
* Limine boot protocol integration,
* serial logging through COM1,
* framebuffer boot banner,
* custom GDT,
* custom IDT,
* breakpoint exception handling,
* page fault handling,
* CPU state diagnostics,
* halt-safe panic path,
* documented x86_64 architecture layer,
* PIT/PIC legacy timer experiment,
* timer interrupt software-vector verification,
* Limine HHDM request support,
* physical-to-virtual address helper based on HHDM,
* Local APIC MSR probe,
* Local APIC physical and virtual base candidate logging,
* timer EOI backend abstraction.

The system currently boots successfully in QEMU and reaches a stable halt loop after validating the core exception path.

---

## Project Philosophy

The central goal of NovasphereX is to build an operating system as an explicit, inspectable, low-level system rather than as an opaque stack of abstractions.

The project follows several guiding principles.

First, the kernel should expose hardware reality rather than hide it prematurely. Descriptor tables, interrupt frames, model-specific registers, bootloader requests, and I/O ports are represented explicitly so that the system can be reasoned about from first principles.

Second, the implementation should favor correctness, observability, and documented assumptions over early generalization. A small, well-instrumented kernel is preferred over a larger but poorly understood kernel.

Third, unsafe operations are not avoided, because operating-system kernels necessarily require unsafe hardware access. Instead, unsafe operations are centralized, documented, and isolated behind minimal APIs.

Fourth, the project is designed to evolve incrementally. Each milestone should produce a system that still boots, logs useful diagnostics, and preserves previously validated behavior.

---

## Long-Term Visual Direction

NovasphereX is intended to develop a distinctive graphical identity.

The long-term graphical interface is planned as a pixel-art-oriented, retro-futuristic environment inspired by the 16-bit console era, especially the Sega Genesis / Mega Drive aesthetic.

This is not intended to be a modern desktop with a retro skin. The graphical stack should eventually be designed around:

* pixel-perfect rendering,
* low-resolution-first UI design,
* integer scaling,
* bitmap fonts,
* sprite-like UI elements,
* tile-based visual composition,
* limited and deliberate color palettes,
* a custom 2D framebuffer renderer,
* Retro16-style themes and system visuals.

The current kernel does not yet implement this GUI. However, the visual direction has already been recorded as part of the system vision so that future rendering and UI design decisions remain coherent.

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

The project is deliberately kept compatible with a Windows-first workflow. Build and execution scripts are PowerShell-based rather than Bash-based.

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

This target does not provide a standard library, operating-system runtime, process model, allocator, or filesystem access. The kernel therefore uses:

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

1. ensures the Rust target is installed,
2. builds the kernel in release mode,
3. downloads or reuses the Limine binary release,
4. prepares a virtual UEFI ESP directory,
5. locates QEMU,
6. locates OVMF/EDK2 firmware,
7. launches QEMU with the ESP directory exposed as a FAT drive,
8. redirects serial output to the PowerShell terminal.

Expected high-level output includes:

```text
[NX] NovasphereX kernel booted
[NX] target: x86_64-unknown-none
[NX] runtime: no_std
[NX] boot protocol: Limine
[NX] Limine base revision supported
[NX] bootloader: Limine 12.3.3
[NX] initializing CPU baseline
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
  boot/
    limine.conf
  docs/
    code-style.md
    roadmap.md
    vision.md
  kernel/
    Cargo.toml
    linker.ld
    src/
      main.rs
      limine.rs
      serial.rs
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
          pic.rs
          pit.rs
          port.rs
          timer.rs
  tools/
    build.ps1
    clean.ps1
    fetch-limine.ps1
    prepare-esp.ps1
    run-qemu.ps1
  Cargo.toml
  rust-toolchain.toml
  README.md
```

The structure separates boot protocol support, serial diagnostics, architecture-independent wrappers, and x86_64-specific hardware mechanisms.

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
        -> framebuffer banner
        -> x86_64 CPU baseline initialization
        -> exception smoke tests
        -> halt loop
```

Limine is responsible for loading the kernel ELF and filling in static boot protocol request responses. The kernel requests, among other things:

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

The HHDM request is important for v0.0.3. It allows the kernel to obtain the higher-half direct-map offset supplied by Limine.

Example diagnostic output:

```text
[NX][HHDM] offset=0xffff800000000000
```

A physical address can then be converted into a candidate direct-map virtual address:

```text
virtual = hhdm_offset + physical
```

However, this is an address calculation, not a proof that every physical range is safely mapped or suitable for ordinary memory access.

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
* PIC/PIT legacy timer experiment,
* Local APIC probe,
* HHDM-based address calculation.

The architecture-independent entry points are exposed through:

```text
kernel/src/arch/mod.rs
```

Higher-level code should call:

```rust
arch::init();
arch::halt_loop();
arch::panic_halt_loop();
arch::log_cpu_state("label");
```

rather than directly depending on deep x86_64 modules.

---

## CPU Helper Layer

The file:

```text
kernel/src/arch/x86_64/cpu.rs
```

centralizes low-level CPU instructions.

It currently provides:

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
read_state()
log_state()
read_msr()
write_msr()
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

The timer vector is currently verified by a software interrupt test. Hardware timer delivery is still under development.

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

* clearing direction flag,
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
[NX][INT] breakpoint exception at rip=0xffffffff80000eda, cs=0x0028, flags=0x0000000000000282
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

The PIT is configured to approximately:

```text
100 Hz
```

The timer interrupt vector path itself has been verified using a software interrupt:

```text
int 32 -> nx_isr_timer -> nx_timer_handler -> iretq
```

This proves:

* IDT vector 32 is valid,
* the timer assembly stub works,
* the Rust timer handler works,
* the timer tick counter works,
* the interrupt return path works.

However, hardware PIT/PIC delivery is not yet functional in the current UEFI/QEMU environment.

Diagnostics show:

```text
master_irr=0b00000001
master_isr=0b00000000
```

This means:

```text
PIT -> PIC works
PIC sees IRQ0 pending
IRQ0 is not masked
CPU interrupt flag is enabled
PIC -> CPU delivery does not occur
```

This strongly suggests that the modern UEFI/QEMU configuration routes interrupts through APIC-oriented mechanisms rather than delivering legacy PIC interrupts directly to the CPU.

For this reason, the project is moving toward Local APIC timer support rather than continuing to rely on the legacy PIT/PIC path.

---

## Local APIC Groundwork

The Local APIC groundwork is implemented in:

```text
kernel/src/arch/x86_64/apic.rs
```

The current APIC implementation performs a conservative probe.

It reads:

```text
IA32_APIC_BASE MSR
```

and decodes:

* raw MSR value,
* Local APIC physical base address,
* APIC enabled bit,
* bootstrap processor bit.

Observed QEMU output:

```text
[NX][APIC] IA32_APIC_BASE=0x00000000fee00900
[NX][APIC] physical_base=0x00000000fee00000 enabled=true bsp=true
```

The physical APIC MMIO base is therefore:

```text
0xfee00000
```

Using the Limine HHDM offset, the kernel computes a virtual base candidate such as:

```text
0xffff8000fee00000
```

At v0.0.3, APIC MMIO writes are intentionally guarded by a safety gate:

```text
ENABLE_HHDM_APIC_MMIO_EXPERIMENT = false
```

This is because HHDM address arithmetic alone does not prove that the Local APIC MMIO page is safely mapped and usable.

---

## Timer EOI Backend Abstraction

The timer module no longer hardcodes only:

```rust
pic::send_eoi(pic::IRQ_TIMER)
```

Instead, it supports an EOI backend abstraction:

```rust
EoiBackend::LegacyPic
EoiBackend::LocalApic
```

At v0.0.3, the selected backend remains:

```text
LegacyPic
```

because APIC MMIO access is not yet enabled by default.

The purpose of this abstraction is to allow the timer path to evolve from:

```text
PIT/PIC timer interrupt
```

toward:

```text
Local APIC timer interrupt
```

without rewriting the timer handler logic.

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

For ordinary RAM regions described by the bootloader memory map, HHDM will be central to the upcoming memory subsystem. For device MMIO regions such as Local APIC, additional care is required.

---

## Current Boot Output

A successful v0.0.3 boot produces output broadly similar to:

```text
[NX] NovasphereX kernel booted
[NX] target: x86_64-unknown-none
[NX] runtime: no_std
[NX] boot protocol: Limine
[NX] Limine base revision supported
[NX] bootloader: Limine 12.3.3
[NX] initializing CPU baseline
[NX][ARCH] x86_64 init begin
[NX][CPU] state: before GDT
[NX][GDT] loading
[NX][GDT] loaded
[NX][IDT] loaded
[NX][HHDM] offset=0xffff800000000000
[NX][APIC] probing Local APIC
[NX][APIC] physical_base=0x00000000fee00000 enabled=true bsp=true
[NX][APIC] virtual_base_candidate=0xffff8000fee00000
[NX][APIC] MMIO writes disabled by safety gate
[NX][TIMER] EOI backend set to LegacyPic
[NX][PIC] remapping IRQs to vectors 32..47
[NX][PIT] configured
[NX][CPU] enabling interrupts
[NX][ARCH] x86_64 init complete
[NX] triggering breakpoint exception test
[NX][INT] breakpoint exception at rip=..., cs=0x0028, flags=...
[NX] breakpoint exception returned successfully
[NX] reached halt loop
```

Exact addresses may differ between builds.

---

## Documentation Policy

From v0.0.3 onward, new kernel code is expected to be documented at the time it is written.

The project follows these conventions:

* module-level documentation uses `//!`,
* public items use `///`,
* unsafe blocks include `SAFETY:` comments when non-trivial,
* assembly code includes stack-layout comments,
* hardware constants are named and documented,
* temporary bootstrap decisions are explicitly marked,
* debug helpers are labeled as such.

The documentation goal is not to comment every line. The goal is to record why hardware-facing code is correct, necessary, or intentionally temporary.

---

## Known Limitations

The current kernel has several important limitations.

There is no allocator.

There is no scheduler.

There is no userspace.

There is no filesystem.

There is no keyboard or mouse driver.

There is no page table abstraction.

There is no physical frame allocator.

There is no memory map parser yet.

There is no complete APIC timer implementation.

The Local APIC MMIO path is not enabled by default.

The PIT/PIC hardware timer path is implemented as a diagnostic experiment but does not currently deliver IRQ0 to the CPU in the tested UEFI/QEMU environment.

The GDT still retains Limine-compatible selectors because the kernel has not yet performed a full controlled transition into its own code and stack segment selectors.

---

## Roadmap

### Milestone 0 — Boot Proof

* [x] Rust `no_std` kernel entry point
* [x] Limine boot protocol request markers
* [x] Serial COM1 logging
* [x] Framebuffer presence check and simple banner drawing
* [x] Verified build on local Rust toolchain
* [x] Verified boot in QEMU

### Milestone 1 — CPU Baseline

* [x] Own GDT
* [x] Own IDT
* [x] Breakpoint exception handler
* [x] Page fault handler
* [x] Basic exception logging
* [x] Halt-safe panic path
* [ ] PIT/APIC timer interrupt draft

  * [x] Timer ISR vector path verified with software interrupt
  * [x] PIT/PIC initialization draft implemented
  * [x] PIT reaches PIC IRR bit0
  * [x] Local APIC base MSR probe
  * [x] HHDM offset support
  * [x] Timer EOI backend abstraction
  * [ ] HHDM-based APIC MMIO validation
  * [ ] Local APIC EOI backend
  * [ ] Local APIC timer delivery

### Milestone 2 — Memory

* [ ] Read Limine memory map
* [ ] Classify usable and reserved memory regions
* [ ] Physical frame allocator
* [ ] Page table abstraction
* [ ] Higher-half direct map wrapper
* [ ] Kernel heap allocator
* [ ] Heap smoke test

### Milestone 3 — Execution

* [ ] Cooperative task abstraction
* [ ] Basic executor
* [ ] Preemptive scheduler draft
* [ ] Syscall ABI draft
* [ ] First userspace ELF loader
* [ ] Minimal init process

### Milestone 4 — Retro16 Graphics

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

---

## Development Commands

Build the kernel:

```powershell
cargo build --release -p novaspherex_kernel
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
git tag -a v0.0.3 -m "HHDM support, Local APIC probe, address helpers, and timer EOI backend abstraction"
```

Suggested commit message:

```powershell
git commit -m "v0.0.3: add HHDM, APIC probe, and timer EOI backend"
```

Future target:

```text
v0.0.4 - Local APIC MMIO and timer delivery
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

---

### QEMU shows OVMF debug logs mixed with kernel logs

This is expected. OVMF and NovasphereX both write to the serial channel used by QEMU.

The kernel logs are prefixed with:

```text
[NX]
```

---

### Timer does not tick after `reached halt loop`

This is currently expected for the legacy PIT/PIC hardware path in the tested UEFI/QEMU environment.

The timer vector itself has been verified with a software interrupt. The remaining work is Local APIC timer delivery.

---

## Scientific Status Summary

NovasphereX v0.0.3 establishes a minimal but coherent experimental kernel substrate.

The system can boot through a modern UEFI bootloader, receive structured boot protocol data, perform early serial and framebuffer diagnostics, install fundamental CPU descriptor tables, handle architecturally significant exceptions, inspect CPU state, read model-specific registers, and reason about interrupt-controller state.

The current timer research has produced a useful negative result: the PIT successfully asserts IRQ0 into the legacy PIC, and the PIC records the request, but the interrupt is not delivered to the CPU in the tested UEFI/QEMU configuration. This observation motivates the transition from legacy PIT/PIC timing toward Local APIC timer infrastructure.

NovasphereX OS project currently features **33 files** and **2112 lines of code** *(as of v0.0.3)*.

Thus, the project has moved from merely booting to experimentally characterizing the hardware abstraction boundary between bootloader-provided CPU state, legacy interrupt controllers, and modern APIC-based interrupt delivery.

NovasphereX remains an early-stage kernel, but its current architecture already reflects the intended methodology of the project: incremental construction, observable state transitions, documented unsafe code, and measured progression from simple legacy mechanisms toward modern native operating-system facilities.
