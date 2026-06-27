$ErrorActionPreference = "Stop"

$Root = Resolve-Path "$PSScriptRoot\.."
Set-Location $Root

cargo clean

$Esp = Join-Path $Root "target\esp"
if (Test-Path $Esp) {
    Remove-Item $Esp -Recurse -Force
}

Write-Host "Cleaned NovasphereX build artifacts."