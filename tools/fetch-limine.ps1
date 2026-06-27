$ErrorActionPreference = "Stop"

$Root = Resolve-Path "$PSScriptRoot\.."
$ThirdParty = Join-Path $Root "third_party"
$LimineDir = Join-Path $ThirdParty "limine-binary"
$Archive = Join-Path $ThirdParty "limine-binary.tar.gz"

if (Test-Path $LimineDir) {
    Write-Host "Limine already exists at $LimineDir"
    exit 0
}

New-Item -ItemType Directory -Force -Path $ThirdParty | Out-Null

$Url = "https://github.com/limine-bootloader/limine/releases/latest/download/limine-binary.tar.gz"

Write-Host "Downloading Limine binary release..."
Invoke-WebRequest -Uri $Url -OutFile $Archive

Write-Host "Extracting Limine..."
tar -xzf $Archive -C $ThirdParty

$Extracted = Get-ChildItem $ThirdParty -Directory |
    Where-Object { $_.Name -like "limine-binary*" } |
    Select-Object -First 1

if ($null -eq $Extracted) {
    throw "Could not find extracted Limine directory."
}

if ($Extracted.FullName -ne $LimineDir) {
    Move-Item $Extracted.FullName $LimineDir
}

$BootEfi = Get-ChildItem $LimineDir -Recurse -Filter "BOOTX64.EFI" |
    Select-Object -First 1

if ($null -eq $BootEfi) {
    throw "BOOTX64.EFI not found in Limine release."
}

Write-Host "Limine ready at $LimineDir"
Write-Host "Found $($BootEfi.FullName)"