$ErrorActionPreference = "Stop"

$Root = Resolve-Path "$PSScriptRoot\.."
Set-Location $Root

$installedTargets = rustup target list --installed
if ($installedTargets -notcontains "#86_64-unknown-none") {
    Write-Host "Installing Rust target: x86_64-unknown-none"
    rustup target add x86_64-unknown-none
}

Write-Host "Building NovasphereX kernel..."
cargo build --release -p novaspherex_kernel