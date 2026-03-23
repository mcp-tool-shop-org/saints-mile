<#
.SYNOPSIS
  Build Saint's Mile MSIX package.

.DESCRIPTION
  1. Builds the Rust binary (unless -SkipBuild)
  2. Stages exe + assets + manifest into a layout directory
  3. Runs makeappx.exe to create the .msix

.PARAMETER SkipBuild
  Skip the cargo build step (use existing binary from dist/ or target/release/).

.PARAMETER Sign
  Sign the MSIX with a self-signed certificate for sideloading.
  Not needed for Store submission (Store signs it).
#>
param(
  [switch]$SkipBuild,
  [switch]$Sign
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$MsixDir = $PSScriptRoot
$LayoutDir = Join-Path $MsixDir "layout"
$OutputMsix = Join-Path $MsixDir "SaintsMile.msix"

# Find makeappx.exe from Windows SDK
$SdkBin = Get-ChildItem "C:\Program Files (x86)\Windows Kits\10\bin\*\x64\makeappx.exe" |
  Sort-Object FullName -Descending | Select-Object -First 1
if (-not $SdkBin) {
  Write-Error "makeappx.exe not found. Install the Windows 10/11 SDK."
  exit 1
}
$MakeAppx = $SdkBin.FullName
Write-Host "Using: $MakeAppx" -ForegroundColor Cyan

# Binary locations (prefer dist/, fall back to target/release/)
$DistExe = Join-Path $RepoRoot "dist\saints-mile.exe"
$ReleaseExe = Join-Path $RepoRoot "target\release\saints-mile.exe"

# Step 1: Build
if (-not $SkipBuild) {
  Write-Host "`n[1/3] Building Rust binary..." -ForegroundColor Yellow
  Push-Location $RepoRoot
  cargo build --release
  Pop-Location
}

# Find the exe
$ExePath = if (Test-Path $DistExe) { $DistExe }
           elseif (Test-Path $ReleaseExe) { $ReleaseExe }
           else { $null }

if (-not $ExePath) {
  Write-Error "saints-mile.exe not found in dist/ or target/release/. Run without -SkipBuild first."
  exit 1
}
Write-Host "  Using binary: $ExePath"

# Step 2: Stage layout
Write-Host "`n[2/3] Staging MSIX layout..." -ForegroundColor Yellow

if (Test-Path $LayoutDir) {
  Remove-Item $LayoutDir -Recurse -Force
}
New-Item -ItemType Directory -Path $LayoutDir -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $LayoutDir "Assets") -Force | Out-Null

# Copy exe
Copy-Item $ExePath -Destination $LayoutDir

# Copy manifest
Copy-Item (Join-Path $MsixDir "AppxManifest.xml") -Destination $LayoutDir

# Copy assets
Get-ChildItem (Join-Path $MsixDir "Assets\*.png") | ForEach-Object {
  Copy-Item $_.FullName -Destination (Join-Path $LayoutDir "Assets")
}

Write-Host "  Layout staged at: $LayoutDir"
Write-Host "  Files:"
Get-ChildItem $LayoutDir -Recurse | ForEach-Object {
  Write-Host "    $($_.FullName.Replace($LayoutDir, '.'))"
}

# Step 3: Pack MSIX
Write-Host "`n[3/3] Packing MSIX..." -ForegroundColor Yellow

if (Test-Path $OutputMsix) {
  Remove-Item $OutputMsix -Force
}

& $MakeAppx pack /d $LayoutDir /p $OutputMsix /o

if ($LASTEXITCODE -ne 0) {
  Write-Error "makeappx failed with exit code $LASTEXITCODE"
  exit 1
}

Write-Host "`nMSIX created: $OutputMsix" -ForegroundColor Green
Write-Host "Size: $([math]::Round((Get-Item $OutputMsix).Length / 1MB, 2)) MB"

# Optional: Sign for sideloading
if ($Sign) {
  Write-Host "`n[Optional] Signing for sideloading..." -ForegroundColor Yellow

  $SignTool = $SdkBin.FullName.Replace("makeappx.exe", "signtool.exe")
  $CertFile = Join-Path $MsixDir "SaintsMile-dev.pfx"

  if (-not (Test-Path $CertFile)) {
    Write-Host "  Generating self-signed certificate..."
    $cert = New-SelfSignedCertificate `
      -Type Custom `
      -Subject "CN=5305D976-6952-4F00-9C21-3A5DB090359F" `
      -KeyUsage DigitalSignature `
      -FriendlyName "SaintsMile Dev" `
      -CertStoreLocation "Cert:\CurrentUser\My" `
      -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3", "2.5.29.19={text}")

    $password = ConvertTo-SecureString -String "saints-mile-dev" -Force -AsPlainText
    Export-PfxCertificate -Cert "Cert:\CurrentUser\My\$($cert.Thumbprint)" `
      -FilePath $CertFile -Password $password | Out-Null

    Write-Host "  Certificate saved: $CertFile (password: saints-mile-dev)"
    Write-Host "  Thumbprint: $($cert.Thumbprint)"
  }

  & $SignTool sign /fd SHA256 /a /f $CertFile /p "saints-mile-dev" $OutputMsix

  if ($LASTEXITCODE -ne 0) {
    Write-Warning "Signing failed. For Store submission, signing is not required (Store signs it)."
  } else {
    Write-Host "  MSIX signed successfully." -ForegroundColor Green
  }
}

Write-Host "`nDone. Next steps:" -ForegroundColor Cyan
Write-Host "  Store submission: Upload $OutputMsix to Partner Center"
Write-Host "  Sideload testing:  Add-AppxPackage -Path '$OutputMsix'"
Write-Host "  Sign for sideload: .\build-msix.ps1 -SkipBuild -Sign"
