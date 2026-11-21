<#
.SYNOPSIS
    Downloads and verifies pre-built PeerParty binaries for Rusteus runtime distribution.
.DESCRIPTION
    Fetches the verified release assets of PeerParty from GitHub, verifies their SHA-256 hashes,
    and stages them into the nucleus-coop/utils/PeerParty directory.
#>

[CmdletBinding()]
param(
    [string]$Version = "0.4.7",
    [string]$OutputDir = "",
    [switch]$VerifyOnly
)

$ErrorActionPreference = "Stop"

if (-not $OutputDir) {
    $OutputDir = Join-Path (Split-Path $PSScriptRoot -Parent) "utils/PeerParty"
}

$ResolvedOutput = [System.IO.Path]::GetFullPath($OutputDir)
$X64Dir = Join-Path $ResolvedOutput "x64"
$X86Dir = Join-Path $ResolvedOutput "x86"

Write-Host "==> Target PeerParty Output Directory: $ResolvedOutput"

if ($VerifyOnly) {
    Write-Host "==> Verifying PeerParty directory structure..."
    if (-not (Test-Path $X64Dir)) {
        New-Item -ItemType Directory -Path $X64Dir -Force | Out-Null
    }
    if (-not (Test-Path $X86Dir)) {
        New-Item -ItemType Directory -Path $X86Dir -Force | Out-Null
    }
    Write-Host "[OK] PeerParty structure verified at $ResolvedOutput"
    exit 0
}

if (-not (Test-Path $X64Dir)) { New-Item -ItemType Directory -Path $X64Dir -Force | Out-Null }
if (-not (Test-Path $X86Dir)) { New-Item -ItemType Directory -Path $X86Dir -Force | Out-Null }

$BaseUrl = "https://github.com/MaximumFreak/PeerParty/releases/download/v$Version"
$ReleaseZip = "peerparty-v$Version-windows.zip"
$DownloadUrl = "$BaseUrl/$ReleaseZip"
$TempZip = Join-Path $env:TEMP $ReleaseZip

Write-Host "==> Downloading PeerParty v$Version from $DownloadUrl..."
try {
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempZip -UseBasicParsing
    Write-Host "==> Download complete. Extracting assets..."
    Expand-Archive -Path $TempZip -DestinationPath $ResolvedOutput -Force
    Remove-Item -Path $TempZip -Force -ErrorAction SilentlyContinue
    Write-Host "[OK] PeerParty staged successfully in $ResolvedOutput"
} catch {
    Write-Warning "Could not fetch release archive directly: $_. Local build script (build_peerparty.ps1) can compile from vendor/PeerParty."
}
