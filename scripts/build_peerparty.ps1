<#
.SYNOPSIS
    Builds PeerParty hook DLLs and launcher from the vendor submodule for Rusteus.
.DESCRIPTION
    Compiles peerparty_hook and peerparty launcher from vendor/PeerParty using cargo
    with sccache enabled, stages binaries to nucleus-coop/utils/PeerParty/, and
    cleans up compilation artifacts to conserve disk space.
#>

[CmdletBinding()]
param(
    [string]$VendorDir = "",
    [string]$OutputDir = "",
    [switch]$Skip32Bit,
    [switch]$KeepTargetDir
)

$ErrorActionPreference = "Stop"

if (-not $VendorDir) {
    $VendorDir = Join-Path (Split-Path $PSScriptRoot -Parent) "vendor/PeerParty"
}
if (-not $OutputDir) {
    $OutputDir = Join-Path (Split-Path $PSScriptRoot -Parent) "utils/PeerParty"
}

$ResolvedVendor = [System.IO.Path]::GetFullPath($VendorDir)
$ResolvedOutput = [System.IO.Path]::GetFullPath($OutputDir)
$X64Dir = Join-Path $ResolvedOutput "x64"
$X86Dir = Join-Path $ResolvedOutput "x86"

if (-not (Test-Path $ResolvedVendor)) {
    throw "Vendor directory not found at $ResolvedVendor. Did you initialize git submodules?"
}

if (-not (Test-Path $X64Dir)) { New-Item -ItemType Directory -Path $X64Dir -Force | Out-Null }
if (-not (Test-Path $X86Dir)) { New-Item -ItemType Directory -Path $X86Dir -Force | Out-Null }

$TargetDir = Join-Path $ResolvedVendor "target_pp_build"
$env:CARGO_TARGET_DIR = $TargetDir
$env:RUSTC_WRAPPER = "sccache"

Write-Host "==> Building PeerParty 64-bit hook DLL (x86_64-pc-windows-msvc)..."
Push-Location $ResolvedVendor
try {
    # Build 64-bit hook DLL
    cargo build --release -p peerparty_hook --target x86_64-pc-windows-msvc
    $BuiltDll64 = Join-Path $TargetDir "x86_64-pc-windows-msvc/release/peerparty_hook.dll"
    if (Test-Path $BuiltDll64) {
        Copy-Item -Path $BuiltDll64 -Destination (Join-Path $X64Dir "peerparty_hook.dll") -Force
        Write-Host "[OK] Staged 64-bit hook DLL to $X64Dir/peerparty_hook.dll"
    } else {
        Write-Warning "Built DLL not found at $BuiltDll64"
    }

    # Build 64-bit launcher exe
    cargo build --release -p peerparty --target x86_64-pc-windows-msvc
    $BuiltExe = Join-Path $TargetDir "x86_64-pc-windows-msvc/release/peerparty.exe"
    if (Test-Path $BuiltExe) {
        Copy-Item -Path $BuiltExe -Destination (Join-Path $ResolvedOutput "peerparty.exe") -Force
        Write-Host "[OK] Staged launcher to $ResolvedOutput/peerparty.exe"
    }

    # Optionally build 32-bit hook DLL if i686 toolchain installed and not skipped
    if (-not $Skip32Bit) {
        Write-Host "==> Checking for i686-pc-windows-msvc target..."
        $HasI686 = (rustup target list --installed | Select-String "i686-pc-windows-msvc")
        if ($HasI686) {
            Write-Host "==> Building PeerParty 32-bit hook DLL (i686-pc-windows-msvc)..."
            cargo build --release -p peerparty_hook --target i686-pc-windows-msvc
            $BuiltDll32 = Join-Path $TargetDir "i686-pc-windows-msvc/release/peerparty_hook.dll"
            if (Test-Path $BuiltDll32) {
                Copy-Item -Path $BuiltDll32 -Destination (Join-Path $X86Dir "peerparty_hook32.dll") -Force
                Write-Host "[OK] Staged 32-bit hook DLL to $X86Dir/peerparty_hook32.dll"
            }
        } else {
            Write-Host "[INFO] i686-pc-windows-msvc target not installed. Skipping 32-bit build."
        }
    }
} finally {
    Pop-Location
    if (-not $KeepTargetDir -and (Test-Path $TargetDir)) {
        Write-Host "==> Cleaning up compilation target directory to preserve disk space..."
        Remove-Item -Path $TargetDir -Recurse -Force -ErrorAction SilentlyContinue
        Write-Host "[OK] Build target cleanup complete."
    }
}

Write-Host "==> PeerParty build pipeline finished successfully."
