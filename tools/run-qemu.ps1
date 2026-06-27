$ErrorActionPreference = "Stop"

$Root = Resolve-Path "$PSScriptRoot\.."
$Esp = Join-Path $Root "target\esp"

& "$PSScriptRoot\prepare-esp.ps1"

# --- Find QEMU -------------------------------------------------------------

$Qemu = Get-Command "qemu-system-x86_64.exe" -ErrorAction SilentlyContinue

if ($null -eq $Qemu) {
    $QemuCandidates = @(
        "C:\Program Files\qemu\qemu-system-x86_64.exe",
        "C:\Program Files (x86)\qemu\qemu-system-x86_64.exe",
        "$env:LOCALAPPDATA\Programs\qemu\qemu-system-x86_64.exe",
        "$env:USERPROFILE\qemu\qemu-system-x86_64.exe"
    )

    $QemuPath = $QemuCandidates |
        Where-Object { $_ -and (Test-Path $_) } |
        Select-Object -First 1

    if (-not $QemuPath) {
        throw "qemu-system-x86_64.exe not found. Add QEMU to PATH or install QEMU for Windows."
    }
} else {
    $QemuPath = $Qemu.Source
}

$QemuPath = [string]$QemuPath
$QemuDir = Split-Path $QemuPath -Parent

# --- Find OVMF / EDK2 firmware --------------------------------------------

$OvmfCandidates = @()

if ($env:OVMF_CODE -and (Test-Path $env:OVMF_CODE)) {
    $OvmfCandidates += $env:OVMF_CODE
}

$SearchRoots = @(
    $QemuDir,
    "C:\Program Files\qemu",
    "C:\Program Files (x86)\qemu",
    "$env:LOCALAPPDATA\Programs\qemu",
    "$env:USERPROFILE\qemu"
)

foreach ($RootDir in $SearchRoots) {
    if ($RootDir -and (Test-Path $RootDir)) {
        $FoundFirmware = Get-ChildItem $RootDir -Recurse -File -Include `
            "edk2-x86_64-code.fd", `
            "OVMF_CODE.fd", `
            "OVMF.fd" `
            -ErrorAction SilentlyContinue

        foreach ($Firmware in $FoundFirmware) {
            $OvmfCandidates += $Firmware.FullName
        }
    }
}

$OvmfCandidates = @($OvmfCandidates | Select-Object -Unique)

if ($OvmfCandidates.Count -eq 0) {
    throw "OVMF/EDK2 firmware not found. Set OVMF_CODE to your edk2-x86_64-code.fd path."
}

$OvmfCode = [string]$OvmfCandidates[0]

# --- Validate ESP -----------------------------------------------------------

if (!(Test-Path $Esp)) {
    throw "ESP directory not found at $Esp"
}

# --- Run QEMU ---------------------------------------------------------------

Write-Host "QEMU: $QemuPath"
Write-Host "OVMF: $OvmfCode"
Write-Host "ESP:  $Esp"

& $QemuPath `
    -M q35 `
    -m 512M `
    -drive "if=pflash,format=raw,readonly=on,file=$OvmfCode" `
    -drive "format=raw,file=fat:rw:$Esp" `
    -serial stdio `
    -no-reboot `
    -no-shutdown