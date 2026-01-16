<#
.SYNOPSIS
    Pre-build script for ShikenMatrix Windows UI

.DESCRIPTION
    This script runs before the C# project builds to ensure the Rust library is compiled.
    Called by MSBuild PreBuildEvent or VS Code tasks.
#>

param(
    [ValidateSet('Debug', 'Release')]
    [string]$Configuration = 'Debug'
)

# Set error handling for VS integration
$ErrorActionPreference = 'Stop'
$OriginalLocation = Get-Location

# Ensure output is visible in Visual Studio
$Host.UI.RawUI.ForegroundColor = 'White'

Set-Location $PSScriptRoot

try {
    # Determine Rust build mode
    $rustArgs = if ($Configuration -eq 'Release') { @('build', '--release') } else { @('build') }
    $targetDir = if ($Configuration -eq 'Release') { 'release' } else { 'debug' }

    Write-Host "[ShikenMatrix] Pre-build: Checking Rust library ($Configuration)..." -ForegroundColor Cyan

    # Check if cargo is available in PATH
    $cargoPath = Get-Command cargo -ErrorAction SilentlyContinue
    
    # If not in PATH, try common Rust installation locations
    if (-not $cargoPath) {
        $possiblePaths = @(
            "$env:USERPROFILE\.cargo\bin\cargo.exe",
            "C:\Users\$env:USERNAME\.cargo\bin\cargo.exe"
        )
        
        foreach ($path in $possiblePaths) {
            if (Test-Path $path) {
                $cargoPath = Get-Command $path -ErrorAction SilentlyContinue
                Write-Host "[ShikenMatrix] Found Cargo at: $path" -ForegroundColor Green
                break
            }
        }
    }
    
    if (-not $cargoPath) {
        Write-Host "[ShikenMatrix] Warning: cargo command not found" -ForegroundColor Yellow
        Write-Host "Please install Rust from: https://rustup.rs/" -ForegroundColor Yellow
        Write-Host "After installation, restart your terminal or IDE" -ForegroundColor Yellow
        Write-Host "Continuing without Rust build..." -ForegroundColor Yellow
        Set-Location $OriginalLocation
        exit 0  # Don't fail the build, just warn
    }

    Write-Host "[ShikenMatrix] Found Cargo: $($cargoPath.Source)" -ForegroundColor Green

    # Check if Rust library needs rebuild
    $rustLib = "..\target\$targetDir\shikenmatrix_native.dll"
    $rustLibDest = "rust-lib\shikenmatrix_native.dll"
    $needsBuild = $false

    if (-not (Test-Path $rustLib)) {
        Write-Host "[ShikenMatrix] Rust library not found at $rustLib, building..." -ForegroundColor Yellow
        $needsBuild = $true
    } else {
        # Check if destination rust-lib DLL exists and is up to date
        if (-not (Test-Path $rustLibDest)) {
            Write-Host "[ShikenMatrix] Destination DLL not found, will copy after checking source..." -ForegroundColor Yellow
        } elseif ((Get-Item $rustLib).LastWriteTime -gt (Get-Item $rustLibDest).LastWriteTime) {
            Write-Host "[ShikenMatrix] Target DLL is newer than rust-lib copy, will update..." -ForegroundColor Yellow
        }

        # Check if source files are newer than the compiled DLL
        $rustLibTime = (Get-Item $rustLib).LastWriteTime
        $cargoToml = "..\Cargo.toml"
        $srcFiles = Get-ChildItem -Path "..\src" -Recurse -Include "*.rs" -ErrorAction SilentlyContinue
        
        $needsRebuild = $false
        
        # Check Cargo.toml
        if ((Test-Path $cargoToml) -and ((Get-Item $cargoToml).LastWriteTime -gt $rustLibTime)) {
            Write-Host "[ShikenMatrix] Cargo.toml modified, rebuilding..." -ForegroundColor Yellow
            $needsRebuild = $true
        }
        
        # Check source files
        if (-not $needsRebuild -and $srcFiles) {
            foreach ($file in $srcFiles) {
                if ($file.LastWriteTime -gt $rustLibTime) {
                    Write-Host "[ShikenMatrix] Source file modified: $($file.Name), rebuilding..." -ForegroundColor Yellow
                    $needsRebuild = $true
                    break
                }
            }
        }
        
        if ($needsRebuild) {
            $needsBuild = $true
        } else {
            Write-Host "[ShikenMatrix] Rust library is up to date (last built: $rustLibTime)" -ForegroundColor Green
        }
    }

    # Build Rust library if needed
    if ($needsBuild) {
        Write-Host "[ShikenMatrix] Building Rust library ($Configuration)..." -ForegroundColor Yellow
        Write-Host "" # Empty line for readability
        Set-Location ..

        $startTime = Get-Date
        $cargoExe = $cargoPath.Source
        
        # Run cargo build
        if ($Configuration -eq 'Release') {
            & $cargoExe build --release
        } else {
            & $cargoExe build
        }
        
        $buildResult = $LASTEXITCODE
        $duration = (Get-Date) - $startTime

        Set-Location $PSScriptRoot

        if ($buildResult -ne 0) {
            Write-Host "" # Empty line
            Write-Host "[ShikenMatrix] Error: Rust library build failed (exit code: $buildResult)" -ForegroundColor Red
            Write-Host "Please check the Rust compilation errors above." -ForegroundColor Yellow
            Set-Location $OriginalLocation
            throw "Rust build failed with exit code $buildResult"
        }

        $durationSec = [math]::Round($duration.TotalSeconds, 1)
        Write-Host "[ShikenMatrix] Rust library built successfully (${durationSec}s)" -ForegroundColor Green
    }

    # Verify Rust library exists
    if (-not (Test-Path $rustLib)) {
        Write-Host "[ShikenMatrix] Error: Rust library not found: $rustLib" -ForegroundColor Red
        Write-Host "Please run manually: cargo build $(if ($Configuration -eq 'Release') { '--release' })" -ForegroundColor Yellow
        Set-Location $OriginalLocation
        exit 1
    }

    # Display library info
    $libInfo = Get-Item $rustLib
    $libSizeMB = [math]::Round($libInfo.Length / 1MB, 2)
    Write-Host "[ShikenMatrix] Rust library info:" -ForegroundColor Cyan
    Write-Host "  Path: $rustLib" -ForegroundColor Gray
    Write-Host "  Size: ${libSizeMB} MB" -ForegroundColor Gray
    Write-Host "  Modified: $($libInfo.LastWriteTime)" -ForegroundColor Gray

    # Copy to rust-lib directory for C# project to reference
    Write-Host "[ShikenMatrix] Copying to rust-lib directory..." -ForegroundColor Cyan
    if (-not (Test-Path "rust-lib")) {
        New-Item -ItemType Directory -Path "rust-lib" | Out-Null
    }
    Copy-Item $rustLib $rustLibDest -Force
    Write-Host "[ShikenMatrix] Copied to: $rustLibDest" -ForegroundColor Green

    Write-Host "" # Empty line
    Write-Host "[ShikenMatrix] Pre-build check complete!" -ForegroundColor Green
    Write-Host "" # Empty line
    Set-Location $OriginalLocation
    exit 0
}
catch {
    Write-Host "" # Empty line
    Write-Host "[ShikenMatrix] Pre-build failed: $_" -ForegroundColor Red
    if ($_.ScriptStackTrace) {
        Write-Host $_.ScriptStackTrace -ForegroundColor Gray
    }
    Set-Location $OriginalLocation
    exit 1
}
