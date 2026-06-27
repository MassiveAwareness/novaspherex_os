# NovasphereX Code Style

Ez a dokumentum a NovasphereX kernel kódolási és dokumentációs alapelveit rögzíti.

A cél nem az, hogy minden sort kommenteljünk, hanem az, hogy a kernel alacsony szintű, veszélyes vagy nem nyilvánvaló részei később is érthetők és auditálhatók legyenek.

## Documentation Philosophy

A NovasphereX kódbázisban a dokumentáció célja:

* a rendszerarchitektúra megértésének segítése,
* az `unsafe` blokkok indoklása,
* az assembly kód céljának magyarázata,
* a hardverközeli döntések rögzítése,
* a korai boot folyamat követhetősége,
* a későbbi hibakeresés támogatása.

A dokumentáció ne ismételje meg mechanikusan azt, amit a kód egyértelműen mutat.

Rossz példa:

```rust
// Increment x by one.
x += 1;
```

Jó példa:

```rust
// The PIC expects an End-of-Interrupt command after the timer
// handler finishes. Without this, further IRQ0 interrupts remain masked.
send_eoi(IRQ_TIMER);
```

## Comment Types

### Module documentation: `//!`

Minden jelentősebb Rust modul tetején legyen modulkomment.

Példa:

```rust
//! CPU helper functions for x86_64.
//!
//! This module contains small wrappers around privileged CPU instructions.
//! Higher-level kernel code should use these helpers instead of inline
//! assembly directly.
```

A modulkomment írja le:

* mire való a modul,
* milyen réteghez tartozik,
* milyen safety vagy architektúra-feltételezései vannak,
* milyen más modulokkal van kapcsolatban.

### Public API documentation: `///`

Minden publikus típushoz és függvényhez legyen dokumentációs komment.

Példa:

```rust
/// Disables maskable interrupts on the current CPU.
pub fn cli() {
    ...
}
```

A publikus API kommentje térjen ki arra, hogy:

* mit csinál,
* mikor szabad hívni,
* van-e mellékhatása,
* visszatér-e,
* kernel-szinten milyen következménye van.

### Internal comments: `//`

Belső kommentet csak akkor írjunk, ha a kód mögötti szándék nem nyilvánvaló.

Jó helyek belső kommentre:

* descriptor magic számok,
* interrupt stack layout,
* CPU által automatikusan pusholt frame-ek,
* boot protocol request markerek,
* linker section használat,
* ideiglenes kompatibilitási megoldások,
* ismert korlátok.

## Unsafe Documentation

Minden nem triviális `unsafe` blokkhoz legyen `SAFETY:` komment.

Példa:

```rust
// SAFETY: `hlt` is a privileged instruction. At this point the kernel is
// already running in ring 0, and the instruction does not access memory.
unsafe {
    asm!("hlt", options(nomem, nostack, preserves_flags));
}
```

A `SAFETY:` komment magyarázza el:

* miért érvényes az adott előfeltétel,
* milyen hardverállapotot feltételezünk,
* miért nem sérül memória- vagy stackbiztonság,
* miért nem vezet undefined behaviorhöz Rust szempontból.

Nem minden egyes apró `unsafe`-hez kell hosszú bekezdés, de a veszélyes vagy architektúrafüggő részeknél kötelező az indoklás.

## Assembly Style

Inline assembly esetén törekedjünk arra, hogy:

* kis, célzott wrapper függvényekben legyen,
* magasabb szintű kód ne használjon közvetlen `asm!` hívást,
* a regiszterhasználat explicit legyen,
* a `options(...)` pontosan tükrözze a műveletet,
* stacket használó assembly ne kapjon `nostack` opciót,
* privilégizált utasítások külön helperben legyenek.

Például ahelyett, hogy több modulban közvetlenül ez szerepelne:

```rust
unsafe {
    core::arch::asm!("hlt");
}
```

használjuk ezt:

```rust
arch::halt_loop();
```

vagy alacsonyabb szinten:

```rust
cpu::hlt();
```

## Naming Conventions

Általános elvek:

* modulnevek: `snake_case`,
* függvénynevek: `snake_case`,
* konstansok: `SCREAMING_SNAKE_CASE`,
* típusok: `PascalCase`,
* assembly szimbólumok: `nx_` prefix.

Példák:

```rust
pub const KERNEL_CODE_SELECTOR: u16 = 0x08;

pub struct InterruptFrame {
    ...
}

extern "C" {
    fn nx_isr_breakpoint();
}
```

Az `nx_` prefix célja, hogy a globális assembly szimbólumok ne ütközzenek külső vagy későbbi szimbólumokkal.

## Logging Style

A korai kernel logok prefixei:

```text
[NX]        általános boot log
[NX][ARCH] architektúra inicializálás
[NX][CPU]  CPU állapot és helper log
[NX][GDT]  Global Descriptor Table
[NX][IDT]  Interrupt Descriptor Table
[NX][INT]  interrupt/exception handler
[NX][PANIC] kernel panic
```

A log legyen rövid, de diagnosztikailag hasznos.

Példa:

```rust
crate::kprintln!("[NX][GDT] loaded");
```

Hardverállapotnál használjunk fix szélességű hex formátumot:

```rust
crate::kprintln!("[NX][CPU] rflags={:#018x}", rflags);
```

## Magic Numbers

A hardverközeli magic számokat nevezzük el konstansként, amikor többször szerepelnek vagy architekturális jelentőségük van.

Jó:

```rust
pub const KERNEL_CODE_SELECTOR: u16 = 0x08;
pub const KERNEL_DATA_SELECTOR: u16 = 0x10;
```

Kerülendő:

```rust
self.selector = 0x08;
```

Descriptor értékeknél kommentben jelezzük a céljukat:

```rust
const KERNEL_CODE_DESCRIPTOR: u64 = 0x00af9a000000ffff;
// 64-bit kernel code segment descriptor.
```

## Boot Compatibility Notes

A korai boot fázisban a bootloader által átadott CPU állapotot tiszteletben kell tartani.

A jelenlegi Limine/QEMU környezetben megfigyelt selectorok:

```text
CS = 0x28
SS = 0x30
DS = 0x30
ES = 0x30
```

Amíg nem váltunk át teljesen saját `CS=0x08` és `SS=0x10` kontextusra, a saját GDT-ben kompatibilisen érvényben kell tartani a bootloader által használt selectorokat is.

Ezért a korai GDT-ben szerepelnek ezek:

```text
0x28 — Limine-kompatibilis code selector
0x30 — Limine-kompatibilis data/stack selector
```

Ez ideiglenes bootstrap-megoldás, nem végleges ABI.

## Interrupt Documentation Rules

Interrupt vagy exception handler dokumentálásakor rögzíteni kell:

* melyik vectorhoz tartozik,
* van-e CPU által pusholt error code,
* milyen stack layoutot várunk,
* milyen regisztereket mentünk,
* visszatér-e `iretq`-val,
* küld-e EOI-t,
* haltol-e.

Példa:

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

## Public vs Internal Modules

Egy modul csak azt tegye `pub`-bá, amit más modulnak ténylegesen használnia kell.

A jelenlegi irány:

```text
arch/mod.rs
  publikus architektúra-független wrapper

arch/x86_64/mod.rs
  x86_64-specifikus koordináció

arch/x86_64/cpu.rs
  alacsony szintű CPU helper

arch/x86_64/gdt.rs
  GDT inicializálás

arch/x86_64/idt.rs
  IDT inicializálás

arch/x86_64/interrupts.rs
  exception/interrupt stubok és handlerek
```

Magasabb szintű kód lehetőleg az `arch::...` wrapperen keresztül használja ezeket, ne közvetlenül a mély x86_64 modulokat hívja.

## Temporary Decisions

Ha egy megoldás szándékosan ideiglenes, írjuk oda.

Példa:

```rust
// Temporary bootstrap compatibility: keep Limine's CS selector valid until
// we implement a stable far jump into our own kernel code selector.
```

Az ideiglenes döntésekhez később roadmap taskot is érdemes felvenni.

## Roadmap Integration

Ha egy modul dokumentációja elkészült, frissítsük a roadmap dokumentációs részét.

Javasolt szekció:

```md
## Documentation Baseline

- [x] Define code documentation style
- [ ] Document CPU helpers
- [ ] Document GDT setup
- [ ] Document IDT setup
- [ ] Document interrupt stubs
- [ ] Document Limine boot protocol wrapper
- [ ] Document serial driver
- [ ] Document boot pipeline
```

## Guiding Rule

A NovasphereX kernel kommentjei ne zajt adjanak a kódhoz, hanem kontextust.

A legfontosabb kérdés minden kommentnél:

```text
Segíteni fog ez a komment hat hónap múlva megérteni, miért így működik ez a kernelrész?
```

Ha igen, maradjon. Ha nem, töröljük.
