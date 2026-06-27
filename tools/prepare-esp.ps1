$ErrorActionPreference = "Stop"

$Root = Resolve-Path "$PSScriptRoot\.."
$Esp = Join-Path $Root "target\esp"
$KernelElf = Join-Path $Root "target\x86_64-unknown-none\release\novaspherex_kernel"
$LimineDir = Join-Path $Root "third_party\limine-binary"

& "$PSScriptRoot\build.ps1"
& "$PSScriptRoot\fetch-limine.ps1"

if (!(Test-Path $KernelElf)) {
    throw "Kernel ELF not found at $KernelElf"
}

$BootEfi = Get-ChildItem $LimineDir -Recurse -Filter "BOOTX64.EFI" |
    Select-Object -First 1

if ($null -eq $BootEfi) {
    throw "BOOTX64.EFI not found. Run tools\fetch-limine.ps1 again."
}

if (Test-Path $Esp) {
    Remove-Item $Esp -Recurse -Force
}

New-Item -ItemType Directory -Force -Path "$Esp\boot" | Out-Null
New-Item -ItemType Directory -Force -Path "$Esp\EFI\BOOT" | Out-Null

Copy-Item $KernelElf "$Esp\boot\novaspherex.elf"
Copy-Item "$Root\boot\limine.conf" "$Esp\limine.conf"
Copy-Item $BootEfi.FullName "$Esp\EFI\BOOT\BOOTX64.EFI"

Write-Host "Prepared UEFI ESP at $Esp"