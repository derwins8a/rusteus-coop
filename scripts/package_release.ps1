<#
.SYNOPSIS
    Packages Rusteus-Coop (Nucleus Co-op) release bundle into a password-protected ZIP archive.
.DESCRIPTION
    Collects the compiled binaries (nucleus-coop.exe, elevator.exe), the complete utils/ folder,
    handlers, webview runtime assets, default configuration, and necessary runtime directories
    into a distribution layout, then compresses it into an encrypted ZIP with the password "nucleus"
    to prevent false-positive antivirus detections.
.PARAMETER TargetDir
    The directory containing compiled release binaries (default: target_ci_release/release or target/release).
.PARAMETER OutputDir
    The directory where the final zip package will be placed (default: repository root).
.PARAMETER ZipName
    The name of the release zip archive (default: rusteus-coop-windows-x64.zip).
.PARAMETER Password
    The archive encryption password (default: nucleus).
.PARAMETER StagingDir
    Optional staging directory path. If omitted, a temporary directory is created and cleaned up.
.PARAMETER BuildPeerParty
    Switch to optionally compile PeerParty binaries from vendor/PeerParty if present.
#>

[CmdletBinding()]
param(
    [string]$TargetDir = "",
    [string]$OutputDir = "",
    [string]$ZipName = "rusteus-coop-windows-x64.zip",
    [string]$Password = "nucleus",
    [string]$StagingDir = "",
    [switch]$BuildPeerParty
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path $PSScriptRoot -Parent
Write-Host "==> Repository Root: $RepoRoot"

# 1. Locate 7-Zip
$7zExe = $null
$Cmd = Get-Command 7z -ErrorAction SilentlyContinue
if ($Cmd) {
    $7zExe = $Cmd.Source
} elseif (Test-Path "C:\Program Files\7-Zip\7z.exe") {
    $7zExe = "C:\Program Files\7-Zip\7z.exe"
} elseif (Test-Path "C:\Program Files (x86)\7-Zip\7z.exe") {
    $7zExe = "C:\Program Files (x86)\7-Zip\7z.exe"
}

if (-not $7zExe) {
    throw "7-Zip (7z.exe) not found in PATH or standard installation locations. 7-Zip is required for password-protected zip packaging."
}
Write-Host "==> Using 7-Zip: $7zExe"

# 2. Resolve TargetDir (compiled binaries)
if (-not $TargetDir) {
    $Candidates = @(
        (Join-Path $RepoRoot "target_ci_release/release"),
        (Join-Path $RepoRoot "target/release"),
        (Join-Path $RepoRoot "target_ci_release"),
        (Join-Path $RepoRoot "target")
    )
    foreach ($Cand in $Candidates) {
        if ((Test-Path (Join-Path $Cand "nucleus-coop.exe")) -or (Test-Path (Join-Path $Cand "release/nucleus-coop.exe"))) {
            if (Test-Path (Join-Path $Cand "nucleus-coop.exe")) {
                $TargetDir = $Cand
            } else {
                $TargetDir = Join-Path $Cand "release"
            }
            break
        }
    }
}

if (-not $TargetDir) {
    $TargetDir = Join-Path $RepoRoot "target_ci_release/release"
}

$ResolvedTarget = [System.IO.Path]::GetFullPath($TargetDir)
Write-Host "==> Binaries Source Directory: $ResolvedTarget"

$MainExe = Join-Path $ResolvedTarget "nucleus-coop.exe"
$ElevatorExe = Join-Path $ResolvedTarget "elevator.exe"

if (-not (Test-Path $MainExe)) {
    throw "Main binary not found at $MainExe. Ensure release build step succeeded."
}

# 3. Resolve OutputDir
if (-not $OutputDir) {
    $OutputDir = $RepoRoot
}
$ResolvedOutput = [System.IO.Path]::GetFullPath($OutputDir)
if (-not (Test-Path $ResolvedOutput)) {
    New-Item -ItemType Directory -Path $ResolvedOutput -Force | Out-Null
}
$FinalZipPath = Join-Path $ResolvedOutput $ZipName
Write-Host "==> Output ZIP Destination: $FinalZipPath"

# 4. Prepare Staging Directory
$CleanupStaging = $false
if (-not $StagingDir) {
    $StagingDir = Join-Path ([System.IO.Path]::GetTempPath()) ("nucleus_dist_" + [System.Guid]::NewGuid().ToString("N"))
    $CleanupStaging = $true
}
$ResolvedStaging = [System.IO.Path]::GetFullPath($StagingDir)
Write-Host "==> Staging Directory: $ResolvedStaging"

if (Test-Path $ResolvedStaging) {
    Remove-Item -Path $ResolvedStaging -Recurse -Force
}
New-Item -ItemType Directory -Path $ResolvedStaging -Force | Out-Null

try {
    # 5. Copy Core Binaries
    Write-Host "==> Staging core executables..."
    Copy-Item -Path $MainExe -Destination (Join-Path $ResolvedStaging "nucleus-coop.exe") -Force
    Write-Host "    [+] nucleus-coop.exe"

    if (Test-Path $ElevatorExe) {
        Copy-Item -Path $ElevatorExe -Destination (Join-Path $ResolvedStaging "elevator.exe") -Force
        Write-Host "    [+] elevator.exe (root)"
    } else {
        Write-Warning "elevator.exe not found at $ElevatorExe; continuing without elevator binary in root."
    }

    # 6. Stage Utils Directory
    Write-Host "==> Staging utils directory..."
    $AssetsUtils = Join-Path $RepoRoot "assets/utils"
    $DestUtils = Join-Path $ResolvedStaging "utils"
    New-Item -ItemType Directory -Path $DestUtils -Force | Out-Null

    if (Test-Path $AssetsUtils) {
        Copy-Item -Path (Join-Path $AssetsUtils "*") -Destination $DestUtils -Recurse -Force
        Write-Host "    [+] Copied assets/utils to dist/utils"
    } else {
        Write-Warning "assets/utils not found at $AssetsUtils"
    }

    # If repo has a top-level utils folder with extra files (e.g. PeerParty), merge them in
    $RepoUtils = Join-Path $RepoRoot "utils"
    if (Test-Path $RepoUtils) {
        Copy-Item -Path (Join-Path $RepoUtils "*") -Destination $DestUtils -Recurse -Force -ErrorAction SilentlyContinue
        Write-Host "    [+] Merged repo utils/ into dist/utils"
    }

    # Also place elevator.exe inside utils/ for legacy and startup parity
    if (Test-Path $ElevatorExe) {
        Copy-Item -Path $ElevatorExe -Destination (Join-Path $DestUtils "elevator.exe") -Force
        Write-Host "    [+] elevator.exe (utils/)"
    }

    # Optionally build/stage PeerParty if explicitly requested
    if ($BuildPeerParty) {
        $BuildScript = Join-Path $RepoRoot "scripts/build_peerparty.ps1"
        if (Test-Path $BuildScript) {
            Write-Host "==> Building PeerParty binaries into utils/PeerParty..."
            try {
                & $BuildScript -OutputDir (Join-Path $DestUtils "PeerParty") -Skip32Bit
                Write-Host "    [+] PeerParty binaries built successfully."
            } catch {
                Write-Warning "PeerParty build skipped or encountered error: $_"
            }
        }
    }

    # 7. Stage Handlers Directory
    $AssetsHandlers = Join-Path $RepoRoot "assets/handlers"
    $DestHandlers = Join-Path $ResolvedStaging "handlers"
    if (Test-Path $AssetsHandlers) {
        Write-Host "==> Staging handlers directory..."
        New-Item -ItemType Directory -Path $DestHandlers -Force | Out-Null
        Copy-Item -Path (Join-Path $AssetsHandlers "*") -Destination $DestHandlers -Recurse -Force
        Write-Host "    [+] Copied assets/handlers to dist/handlers"
    }

    # 8. Stage WebView Directory
    $AssetsWebview = Join-Path $RepoRoot "assets/webview"
    $DestWebview = Join-Path $ResolvedStaging "webview"
    if (Test-Path $AssetsWebview) {
        Write-Host "==> Staging webview directory..."
        New-Item -ItemType Directory -Path $DestWebview -Force | Out-Null
        Copy-Item -Path (Join-Path $AssetsWebview "*") -Destination $DestWebview -Recurse -Force
        Write-Host "    [+] Copied assets/webview to dist/webview"
    }

    # 9. Stage Root Runtime Files (DLLs, configs, readmes)
    Write-Host "==> Staging root runtime files..."
    $RootAssetFiles = @(
        "SDL2.dll",
        "openxinput1_3.dll",
        "Settings.ini",
        "readme.txt",
        "readme Proto Input.txt"
    )

    foreach ($File in $RootAssetFiles) {
        $SrcPath = Join-Path $RepoRoot (Join-Path "assets" $File)
        if (Test-Path $SrcPath) {
            Copy-Item -Path $SrcPath -Destination (Join-Path $ResolvedStaging $File) -Force
            Write-Host "    [+] $File"
        } else {
            Write-Warning "Asset file not found: $SrcPath"
        }
    }

    # 10. Ensure Required GUI / Content Cache Directories Exist
    Write-Host "==> Creating required runtime directories..."
    $RequiredDirs = @(
        (Join-Path $ResolvedStaging "gui/covers"),
        (Join-Path $ResolvedStaging "gui/screenshots"),
        (Join-Path $ResolvedStaging "content/cache/covers")
    )
    foreach ($Dir in $RequiredDirs) {
        if (-not (Test-Path $Dir)) {
            New-Item -ItemType Directory -Path $Dir -Force | Out-Null
            # Add a small placeholder so zip preserving utilities retain the directory
            Set-Content -Path (Join-Path $Dir ".gitkeep") -Value "" -NoNewline
        }
    }

    # 11. Add Password / Antivirus Advisory Note
    $AdvisoryContent = @"
================================================================================
                    Nucleus Co-op (Rusteus) Release
                        Archive Password: $Password
================================================================================

Why is this archive password-protected?
Antivirus software heuristics frequently flag game hooking libraries, emulators,
and process injection utilities (such as Goldberg Steam Emulator, ForceBindIP,
and EasyHook) as false-positives.

Encrypting this archive with the password "$Password" prevents automated antivirus
scanners and web browsers from blocking or corrupting the files during download.

INSTRUCTIONS:
1. Extract all files using the password: $Password
2. Add your extracted Nucleus Co-op folder to your antivirus exceptions list.
3. Launch nucleus-coop.exe as Administrator when running games.

================================================================================
"@
    Set-Content -Path (Join-Path $ResolvedStaging "PASSWORD_IS_$Password.txt") -Value $AdvisoryContent -Encoding utf8
    Write-Host "    [+] PASSWORD_IS_$Password.txt"

    # 12. Create Password-Protected ZIP Archive with 7-Zip
    Write-Host "==> Compressing bundle into encrypted ZIP (Password: $Password)..."
    if (Test-Path $FinalZipPath) {
        Remove-Item -Path $FinalZipPath -Force
    }

    $StagingWildcard = Join-Path $ResolvedStaging "*"
    
    # -tzip: standard zip format
    # -p: password
    # -mem=ZipCrypto: ensures universal compatibility with Windows Explorer native extraction
    # -y: assume Yes on all queries
    # -bd: disable progress indicator in log output
    $7zArgs = @(
        "a",
        "-tzip",
        "-p$Password",
        "-mem=ZipCrypto",
        "-r",
        "-y",
        "-bd",
        $FinalZipPath,
        $StagingWildcard
    )

    & $7zExe $7zArgs
    if ($LASTEXITCODE -ne 0) {
        throw "7-Zip failed to create archive with exit code $LASTEXITCODE"
    }

    # 13. Verify Archive Integrity & Encryption
    Write-Host "==> Verifying encrypted archive integrity..."
    $VerifyArgs = @(
        "t",
        "-p$Password",
        "-bd",
        $FinalZipPath
    )
    & $7zExe $VerifyArgs
    if ($LASTEXITCODE -ne 0) {
        throw "Archive verification failed with password '$Password'"
    }
    Write-Host "[OK] Archive verified successfully with password '$Password'!"

    # Also confirm that scanning with wrong password fails (confirming encryption is active)
    $FailTestArgs = @(
        "t",
        "-pwrong_test_pw",
        "-bd",
        $FinalZipPath
    )
    $PrevEAP = $ErrorActionPreference
    try {
        $ErrorActionPreference = "SilentlyContinue"
        $null = & $7zExe $FailTestArgs 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Warning "Archive test with invalid password unexpectedly succeeded! Encryption may not be active."
        } else {
            Write-Host "[OK] Verified archive contents are protected and inaccessible without password."
        }
    } finally {
        $ErrorActionPreference = $PrevEAP
    }

    $ZipInfo = Get-Item $FinalZipPath
    $ZipSizeMB = [math]::Round($ZipInfo.Length / 1MB, 2)
    Write-Host ""
    Write-Host "================================================================================"
    Write-Host "  RELEASE PACKAGE CREATED SUCCESSFULLY"
    Write-Host "  File:     $FinalZipPath"
    Write-Host "  Size:     $ZipSizeMB MB"
    Write-Host "  Password: $Password"
    Write-Host "================================================================================"

    if ($env:GITHUB_STEP_SUMMARY) {
        @"
## Rusteus-Coop Release Package
- **Archive:** ``$ZipName``
- **Size:** $ZipSizeMB MB
- **Password:** ``$Password``
- **Antivirus Note:** Archive is encrypted with password ``$Password`` to prevent false-positive detection by antivirus heuristics on game hooking DLLs and emulators.
"@ | Out-File -FilePath $env:GITHUB_STEP_SUMMARY -Append -Encoding utf8
    }

} finally {
    if ($CleanupStaging -and (Test-Path $ResolvedStaging)) {
        Write-Host "==> Cleaning up temporary staging directory..."
        Remove-Item -Path $ResolvedStaging -Recurse -Force -ErrorAction SilentlyContinue
    }
}
