# NovasphereX OS

Rustban írt, x86_64-es, UEFI-n bootolható kísérleti operációs rendszer.

## Host rendszer

A projekt elsődleges fejlesztői környezete:

- Windows 11
- PowerShell
- Rustup
- QEMU for Windows
- Limine UEFI bootloader

## Szükséges programok

### Rust

Telepítsd a Rustupot Windowsra:

```powershell
winget install Rustlang.Rustup
```

Majd:

```powershell
rustup default stable
rustup target add x86_64-unknown-none
```

### QEMU

```powershell
winget install qemu
```

Ellenőrzés:

```powershell
qemu-system-x86_64.exe --version
```

Ha a QEMU nincs a PATH-ban, indíts új terminált, vagy add hozzá kézzel:

```powershell
C:\Program Files\qemu
```

## Indítás

PowerShellből, a projekt gyökerében:

```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass .\tools\run-qemu.ps1
```

Ez:
1. lefordítja a Rust kernelt,
2. letölti a Limine bináris release-t
3. előkészíti a virtuális UEFI ESP könyvtárat
4. elindítja a QEMU-t UEFI módban

## Várt kimenet

A QEMU ablak elindul, a PowerShellben pedig a serial logon meg kell jelennie:

```plaintext
NovasphereX kernel booted.
Rust no_std + Limine + x86_64
```

## Projektfilozófia

Nem Linux-alapú rendszert írunk, hanem saját rendszert.

Első cél:

* bootolás UEFI-n,
* serial log,
* framebuffer teszt,
* saját GDT,
* saját IDT,
* exception handling,
* memória térkép beolvasása,
* fizikai frame allocator,
* paging,
* heap allocator