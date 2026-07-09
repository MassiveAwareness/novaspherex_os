param(
    [switch]$Force
)

$ErrorActionPreference = "Stop"

$Root = Resolve-Path "$PSScriptRoot\.."
$SourceDir = Join-Path $Root "assets\images"
$OutputDir = Join-Path $Root ".\kernel\src\assets\generated"


$TargetWidth = 640
$TargetHeight = 360
$BytesPerPixel = 4
$ExpectedByteLength = $TargetWidth * $TargetHeight * $BytesPerPixel
$AssetCount = 5

function Test-GeneratedAsset {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    if(!(Test-Path $Path)) {
        return $false
    }

    $Item = Get-Item $Path

    if ($Item.Length -ne $ExpectedByteLength) {
        Write-Host "Invalid generated asset size: $Path"
        Write-Host "Expected $ExpectedByteLength bytes, got $($Item.Length) bytes"
        return $false
    }

    return $true
}

function Convert-BackgroundAsset {
    param(
        [Parameter(Mandatory = $true)]
        [string]$InputPath,

        [Parameter(Mandatory = $true)]
        [string]$OutputPath
    )

    Add-Type -AssemblyName System.Drawing

    Write-Host "Converting $InputPath -> $OutputPath"

    $SourceBitmap = [System.Drawing.Bitmap]::new($InputPath)
    $TargetBitmap = [System.Drawing.Bitmap]::new(
        $TargetWidth,
        $TargetHeight,
        [System.Drawing.Imaging.PixelFormat]::Format24bppRgb
    )

    $Graphics = [System.Drawing.Graphics]::FromImage($TargetBitmap)
    $Graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::NearestNeighbor
    $Graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::Half
    $Graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::None
    $Graphics.DrawImage($SourceBitmap, 0, 0, $TargetWidth, $TargetHeight)

    $FileStream = [System.IO.File]::Open($OutputPath, [System.IO.FileMode]::Create)
    $Writer = [System.IO.BinaryWriter]::new($FileStream)

    for($Y = 0; $Y -lt $TargetHeight; $Y++) {
        for($X = 0; $X -lt $TargetWidth; $X++) {
            $Pixel = $TargetBitmap.GetPixel($X, $Y)

            $Writer.Write([byte]$Pixel.R)
            $Writer.Write([byte]$Pixel.G)
            $Writer.Write([byte]$Pixel.B)
            $Writer.Write([byte]0)
        }
    }

    $Writer.Dispose()
    $FileStream.Dispose()
    $Graphics.Dispose()
    $TargetBitmap.Dispose()
    $SourceBitmap.Dispose()
}

if (!(Test-Path $SourceDir)) {
    throw "Missing source asset directory: $SourceDir"
}

New-Item -ItemType Directory -Force $OutputDir | Out-Null

Write-Host "Preparing assets..."

$ConvertedCount = 0
$ValidatedCount = 0

for ($Index = 1; $Index -le $AssetCount; $Index++) {
    $InputPath = Join-Path $SourceDir "bg_$Index.png"
    $OutputPath = Join-Path $OutputDir "bg_$Index.rgbx"

    if (!(Test-Path $InputPath)) {
        throw "Missing background source asset: $InputPath"
    }

    $GeneratedAssetIsValid = Test-GeneratedAsset -Path $OutputPath

    if ($GeneratedAssetIsValid -and !$Force) {
        Write-Host "Validated bg_$Index.rgbx"
        $ValidatedCount++;
        continue
    }

    if($Force) {
        Write-Host "Force enabled; regenerating bg_$Index.rgbx"
    }
    else {
        Write-Host "Generated asset missing or invalid; regenerating bg_$Index.rgbx"
    }

    Convert-BackgroundAsset -InputPath $InputPath -OutputPath $OutputPath

    if(!(Test-GeneratedAsset -Path $OutputPath)) {
        throw "Generated asset validation failed after conversion: $OutputPath"
    }

    $ConvertedCount++;
}

Write-Host "Background assets ready. Validated: $ValidatedCount, converted: $ConvertedCount."