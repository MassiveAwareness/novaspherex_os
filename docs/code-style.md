# NovasphereX Code Style

This document defines the coding and documentation principles for the NovasphereX kernel.

The purpose of documentation is not to comment every line. The purpose is to make low-level, dangerous, hardware-specific, or non-obvious kernel code understandable and auditable later.

NovasphereX is a low-level operating system project. Much of the code interacts directly with CPU state, bootloader data structures, page tables, interrupt controllers, MMIO registers, framebuffers, and raw memory. These areas must be documented with enough context that their assumptions remain clear as the system evolves.

---

## Documentation Philosophy

Documentation in the NovasphereX codebase should help explain:

* system architecture,
* hardware assumptions,
* boot-time state,
* unsafe code requirements,
* inline assembly behavior,
* interrupt and exception stack layout,
* page table manipulation,
* MMIO access,
* framebuffer access,
* asset embedding,
* temporary bootstrap decisions,
* debugging expectations.

Documentation should explain intent, constraints, and consequences.

It should not mechanically repeat what the code already says.

Bad example:

```rust
// Increment x by one.
x += 1;
```

Good example:

```rust
// The PIC expects an End-of-Interrupt command after the timer
// handler finishes. Without this, further IRQ0 interrupts remain pending.
send_eoi(IRQ_TIMER);
```

The guiding question for every comment is:

```text
Will this help explain why this kernel code works this way six months from now?
```

If yes, keep it. If no, remove it.

---

## Language Policy

Project-facing documentation should gradually move toward English.

This includes:

* Rust documentation comments,
* module comments,
* public API comments,
* README content,
* roadmap entries,
* release notes,
* script comments,
* commit messages where practical.

Hungarian may still appear in exploratory notes or development conversation, but committed project documentation should prefer English for long-term maintainability.

---

## Comment Types

### Module Documentation: `//!`

Every significant Rust module should start with module-level documentation.

Example:

```rust
//! CPU helper functions for x86_64.
//!
//! This module contains small wrappers around privileged CPU instructions.
//! Higher-level kernel code should use these helpers instead of inline assembly
//! directly.
```

A module comment should explain:

* what the module is responsible for,
* which layer it belongs to,
* what hardware or boot assumptions it relies on,
* what other modules it coordinates with,
* whether it is temporary, experimental, or foundational.

Important modules that should always have module documentation include:

```text
limine.rs
serial.rs
assets/*
arch/mod.rs
arch/x86_64/mod.rs
arch/x86_64/cpu.rs
arch/x86_64/gdt.rs
arch/x86_64/idt.rs
arch/x86_64/interrupts.rs
arch/x86_64/paging.rs
arch/x86_64/apic.rs
arch/x86_64/pic.rs
arch/x86_64/pit.rs
arch/x86_64/timer.rs
arch/x86_64/port.rs
```

---

### Public API Documentation: `///`

Every public type, constant, and function should have a documentation comment.

Example:

```rust
/// Disables maskable interrupts on the current CPU.
pub fn cli() {
    ...
}
```

Public API documentation should explain:

* what the item does,
* when it may be called,
* whether it has side effects,
* whether it returns,
* whether it mutates global hardware state,
* whether it depends on early boot state,
* whether it is temporary or experimental.

For public constants, document the hardware or protocol meaning.

Example:

```rust
/// Local APIC End-of-Interrupt register offset.
pub const APIC_EOI_OFFSET: usize = 0x0b0;
```

---

### Internal Comments: `//`

Internal comments should be used when the intent behind code is not obvious.

Good places for internal comments:

* descriptor magic values,
* page table entry flags,
* interrupt stack layouts,
* CPU-pushed frames,
* boot protocol request markers,
* linker section usage,
* MMIO register access,
* framebuffer pixel format handling,
* temporary compatibility behavior,
* known limitations.

Internal comments should explain why, not merely what.

---

## Unsafe Documentation

Every non-trivial `unsafe` block must include a `SAFETY:` comment.

Example:

```rust
// SAFETY: `hlt` is a privileged instruction. The kernel is already running in
// ring 0, and the instruction does not access memory or the stack.
unsafe {
    asm!("hlt", options(nomem, nostack, preserves_flags));
}
```

A `SAFETY:` comment should explain:

* why the preconditions are valid,
* what hardware state is assumed,
* why memory safety is preserved,
* why stack safety is preserved,
* why Rust undefined behavior is avoided,
* what would make the block invalid.

Short `unsafe` blocks do not always need long comments, but hardware-facing or memory-facing code must always explain the relevant safety contract.

Important areas requiring careful `SAFETY:` comments:

* inline assembly,
* port I/O,
* MSR reads/writes,
* control register reads,
* `invlpg`,
* pointer arithmetic,
* raw framebuffer writes,
* Limine response pointer access,
* page table mutation,
* MMIO reads/writes,
* static mutable early boot data.

---

## Inline Assembly Style

Inline assembly should be kept small, explicit, and isolated.

General rules:

* use small wrapper functions,
* avoid direct `asm!` in high-level code,
* keep register use explicit,
* use accurate `options(...)`,
* do not use `nostack` if the instruction or sequence touches the stack,
* isolate privileged instructions in `cpu.rs` or similarly focused modules,
* document the safety contract.

Instead of writing this in multiple modules:

```rust
unsafe {
    core::arch::asm!("hlt");
}
```

prefer:

```rust
arch::halt_loop();
```

or at the lower level:

```rust
cpu::hlt();
```

Examples of instructions that should remain centralized:

```text
cli
sti
hlt
rdmsr
wrmsr
rdtsc
mov ..., cr2
mov ..., cr3
invlpg
pushfq / pop
segment selector reads
```

Assembly interrupt stubs should include comments documenting:

* which vector they serve,
* whether the CPU pushes an error code,
* which registers are saved,
* how the stack is aligned before calling Rust,
* where the interrupt frame is located,
* whether the handler returns with `iretq`.

---

## Naming Conventions

General naming rules:

* module names: `snake_case`,
* function names: `snake_case`,
* constants: `SCREAMING_SNAKE_CASE`,
* types: `PascalCase`,
* global assembly symbols: `nx_` prefix,
* public kernel log macros: `kprint!`, `kprintln!`.

Examples:

```rust
pub const KERNEL_CODE_SELECTOR: u16 = 0x08;

pub struct InterruptFrame {
    ...
}

extern "C" {
    fn nx_isr_breakpoint();
}
```

The `nx_` prefix is used for globally visible low-level symbols to reduce the risk of name collisions with future external or linker-visible symbols.

---

## Logging Style

Early kernel logs should be concise, stable, and diagnostically useful.

Current log prefixes:

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

Good example:

```rust
crate::kprintln!("[NX][GDT] loaded");
```

For hardware values, use fixed-width hexadecimal formatting when appropriate:

```rust
crate::kprintln!("[NX][CPU] rflags={:#018x}", rflags);
```

For boolean state, prefer explicit labels:

```rust
crate::kprintln!(
    "[NX][APIC] physical_base={:#018x} enabled={} bsp={}",
    physical_base,
    enabled,
    bootstrap_processor
);
```

Avoid noisy logs inside high-frequency interrupt paths. Timer logs should remain throttled.

---

## Magic Numbers

Hardware-related magic numbers should be named as constants when they are repeated or architecturally meaningful.

Good:

```rust
pub const KERNEL_CODE_SELECTOR: u16 = 0x08;
pub const KERNEL_DATA_SELECTOR: u16 = 0x10;
```

Avoid:

```rust
self.selector = 0x08;
```

Descriptor values should include comments explaining their meaning:

```rust
const KERNEL_CODE_DESCRIPTOR: u64 = 0x00af9a000000ffff;
// 64-bit kernel code segment descriptor.
```

This rule is especially important for:

* GDT descriptors,
* IDT attributes,
* page table flags,
* APIC register offsets,
* PIC/PIT ports,
* Limine request IDs,
* framebuffer pixel offsets,
* asset format dimensions,
* boot protocol magic values.

---

## Boot Compatibility Notes

Early boot code must respect the CPU state handed over by the bootloader.

In the current Limine/QEMU environment, the observed selectors are:

```text
CS = 0x28
SS = 0x30
DS = 0x30
ES = 0x30
```

Until the kernel performs a controlled transition into its own `CS=0x08` and `SS=0x10` execution context, the custom GDT must keep the bootloader-provided selectors valid.

Therefore, the early GDT contains:

```text
0x28 — Limine-compatible code selector
0x30 — Limine-compatible data/stack selector
```

This is a temporary bootstrap compatibility mechanism, not a final kernel ABI.

Any code touching GDT setup should preserve this fact until the transition work is completed.

---

## Interrupt Documentation Rules

Interrupt and exception handlers must document:

* the vector number,
* whether the CPU pushes an error code,
* the expected stack layout,
* which registers are saved,
* whether the handler returns with `iretq`,
* whether the handler sends EOI,
* whether the path is recoverable,
* whether the path halts permanently.

Example:

```rust
// For page fault, the CPU pushes:
//   ERROR CODE
//   RIP
//   CS
//   RFLAGS
//
// After saving 15 general-purpose registers and adding 8 bytes of alignment
// padding, the error code is at `rsp + 128`.
```

Recoverable paths, such as breakpoint exceptions, should explain why returning is safe.

Fatal paths, such as the current page fault handler, should explain why the kernel halts.

Timer interrupt paths should document which interrupt controller receives EOI:

```text
Legacy PIC path  -> PIC EOI
Local APIC path  -> APIC EOI
```

---

## Paging and MMIO Style

Paging code must be especially explicit.

Any code that reads or modifies page tables should document:

* which paging level is being accessed,
* whether the physical address came from `CR3`,
* whether access happens through HHDM,
* whether huge pages are supported,
* whether new tables are allocated,
* whether the active page tables are mutated,
* whether `invlpg` is required afterward.

Page table flags must be named constants.

Good:

```rust
const PTE_PRESENT: u64 = 1 << 0;
const PTE_WRITABLE: u64 = 1 << 1;
const PTE_CACHE_DISABLE: u64 = 1 << 4;
```

MMIO mappings should not be treated as ordinary RAM.

MMIO page mappings should use explicit cache-related flags where appropriate, such as:

```text
present
writable
write-through
cache-disable
```

Any temporary paging mechanism, such as a static early page-table pool, must be clearly marked as temporary and not a general allocator.

---

## Limine and Boot Protocol Style

Boot protocol bindings should remain minimal, documented, and explicit.

Rules:

* represent Limine structures with `#[repr(C)]`,
* place requests in the correct linker sections,
* mark request objects with `#[used]`,
* check response pointers for null before use,
* keep unsafe pointer access small,
* document request IDs and response meanings.

Example areas requiring documentation:

* base revision request,
* bootloader info request,
* framebuffer request,
* HHDM request,
* request start/end markers.

Limine response helpers should return safe Rust abstractions where practical, such as:

```rust
pub fn hhdm_offset() -> Option<u64>
```

Raw pointer exposure should be limited.

---

## Framebuffer and Graphics Style

Framebuffer code writes directly into memory provided by the bootloader. It must be treated as unsafe hardware-facing code.

Framebuffer drawing code should document:

* where the framebuffer pointer comes from,
* whether the framebuffer response was checked,
* how pixel format is handled,
* how pitch is used,
* how bounds are enforced,
* whether scaling is nearest-neighbor,
* what source asset format is expected.

Framebuffer code should not assume a fixed pixel format unless it explicitly checks or converts using framebuffer metadata.

Preferred style:

```text
source RGBX asset
  -> pack using framebuffer color masks
  -> write using framebuffer pitch and bytes-per-pixel
```

Boot graphics are currently early-stage and should remain simple.

---

## Asset Pipeline Style

The asset pipeline currently converts Retro16 PNG boot backgrounds into raw RGBX buffers before kernel build time.

Rules:

* source assets live under `assets/images`,
* generated kernel assets live under `kernel/src/assets/generated`,
* generated assets must be validated before use,
* conversion should be skipped when generated files are already valid,
* forced regeneration should be available through a script flag,
* the QEMU run workflow should prepare assets before kernel build,
* kernel code should embed generated assets with `include_bytes!`.

Generated asset format constants should be documented:

```rust
pub const BOOT_BACKGROUND_WIDTH: usize = 640;
pub const BOOT_BACKGROUND_HEIGHT: usize = 360;
pub const BOOT_BACKGROUND_BYTES_PER_PIXEL: usize = 4;
```

The kernel should not parse PNG files during early boot. Runtime image loading belongs to a later filesystem and asset-loading milestone.

---

## Randomness Style

Current boot-time background selection uses a lightweight TSC-based seed.

This is acceptable for harmless visual variation.

It must not be described or used as cryptographic randomness.

Any code using this mechanism should document that it is:

```text
non-cryptographic
early-boot only
suitable for visual variation
not suitable for security decisions
```

Future security-sensitive randomness must come from a proper entropy subsystem.

---

## Public vs Internal Modules

A module should only expose what other modules actually need.

Current layering direction:

```text
arch/mod.rs
  public architecture-level wrapper

arch/x86_64/mod.rs
  x86_64-specific coordination

arch/x86_64/cpu.rs
  low-level CPU helpers

arch/x86_64/gdt.rs
  GDT initialization

arch/x86_64/idt.rs
  IDT initialization

arch/x86_64/interrupts.rs
  interrupt and exception stubs/handlers

arch/x86_64/paging.rs
  minimal paging and MMIO mapping helpers

arch/x86_64/apic.rs
  Local APIC support

arch/x86_64/timer.rs
  timer backend coordination
```

Higher-level kernel code should prefer `arch::...` wrappers where possible rather than directly reaching into deep x86_64 modules.

Low-level modules may expose public functions to sibling architecture modules, but they should avoid becoming a broad global API.

---

## Temporary Decisions

If a solution is intentionally temporary, document it directly in the code.

Example:

```rust
// Temporary bootstrap compatibility: keep Limine's CS selector valid until
// we implement a stable far jump into our own kernel code selector.
```

Temporary decisions should ideally have corresponding roadmap entries.

Current known temporary decisions include:

* Limine-compatible GDT selectors,
* static early page-table pool,
* uncalibrated Local APIC timer initial count,
* TSC-based visual boot variation,
* build-time embedded boot backgrounds instead of runtime asset loading,
* lack of dedicated MMIO virtual region,
* lack of full physical frame allocator.

Temporary does not mean careless. Temporary code must still be documented and safe within its intended constraints.

---

## PowerShell Script Style

PowerShell scripts are part of the project’s development workflow and should remain readable.

Rules:

* use `$ErrorActionPreference = "Stop"`,
* validate required paths before using them,
* throw clear errors,
* keep major sections separated with comments,
* prefer explicit variable names,
* avoid silently ignoring missing critical files,
* print concise status messages.

Script section style:

```powershell
# --- Prepare embedded assets ------------------------------------------------
```

Important scripts include:

```text
tools/build.ps1
tools/fetch-limine.ps1
tools/prepare-esp.ps1
tools/prepare-assets.ps1
tools/run-qemu.ps1
tools/clean.ps1
```

Asset scripts should validate generated output, not merely generate it.

---

## Rust Formatting and Structure

Use standard Rust formatting conventions.

General rules:

* prefer `cargo fmt` formatting,
* keep functions small when possible,
* keep hardware wrappers focused,
* avoid broad utility modules with unrelated responsibilities,
* avoid premature abstraction,
* prefer explicit names over clever names,
* keep boot-critical paths easy to follow.

For early kernel code, clarity is more important than compactness.

---

## Panic and Failure Style

Kernel failure paths should be explicit.

For unrecoverable early kernel failures:

* log the error,
* log relevant CPU state if useful,
* disable interrupts if continuing would be dangerous,
* halt in a stable loop.

Panic-related logs should use:

```text
[NX][PANIC]
```

Page fault and exception logs should use:

```text
[NX][INT]
```

Boot script failures should throw clear PowerShell errors.

---

## Roadmap Integration

When a new subsystem or documented capability is added, update the relevant documentation.

Likely files:

```text
docs/roadmap.md
docs/release-notes.md
docs/vision.md
docs/code-style.md
README.md
```

Examples:

* adding a new hardware subsystem should update the roadmap,
* completing a release should update release notes,
* changing graphics direction should update vision,
* introducing a new coding pattern should update code style,
* changing build workflow should update README/tooling docs.

Documentation should evolve with the kernel.

---

## Guiding Rule

NovasphereX comments should add context, not noise.

The most important question for every comment is:

```text
Will this help a future maintainer understand why this kernel component behaves this way?
```

If yes, keep it.

If no, remove it.
