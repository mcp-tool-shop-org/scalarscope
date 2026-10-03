# ScalarScope Installation Runbook

This document provides step-by-step instructions for installing, upgrading, and uninstalling ScalarScope on a clean Windows machine.

## Prerequisites

### System Requirements
- **OS**: Windows 10 version 1809 (build 17763) or later
- **Architecture**: x64
- **RAM**: 4 GB minimum, 8 GB recommended
- **Disk**: 200 MB free space
- **GPU**: Any DirectX 11 compatible (for SkiaSharp hardware acceleration)

### Runtime Dependencies
ScalarScope is self-contained and includes all required runtimes:
- .NET 9.0 Runtime (bundled)
- Windows App SDK (bundled)
- No additional VC++ redistributables required

## Installation Methods

### Method 1: Microsoft Store

The app is published. Store id `9P3HT1PHBKQK`.

1. Open https://apps.microsoft.com/detail/9P3HT1PHBKQK
2. Click Get
3. Launch ScalarScope from the Start menu

An update must keep package name `mcp-tool-shop.ScalarScope`, publisher `CN=5305D976-6952-4F00-9C21-3A5DB090359F`, and publisher display name `mcp-tool-shop`. The next package version is `3.0.0.0`.

### Method 2: Partner Center upload

The release workflow builds an unsigned MSIX. Partner Center signs it during ingestion. That file will not install if you double-click it.

### Method 3: Build from source

```powershell
git clone https://github.com/mcp-tool-shop-org/scalarscope.git
cd scalarscope
dotnet workload install maui-windows
dotnet build src/ScalarScope/ScalarScope.csproj -c Release -f net9.0-windows10.0.19041.0
dotnet run --project src/ScalarScope/ScalarScope.csproj -f net9.0-windows10.0.19041.0
```

## First Run Verification

After installation, verify the app works correctly:

### Step 1: Launch Application
- Open Start Menu
- Type "ScalarScope"
- Click to launch
- **Expected**: App opens with welcome screen within 3 seconds

### Step 2: Load Sample Data
- On Home, click "Compare Two Runs"
- Load a baseline trace and an optimized trace
- **Expected**: The Compare tab shows the two runs

### Step 3: Test Playback
- Press Space to start playback
- Press +/- to adjust speed
- **Expected**: Smooth animation, responsive controls

### Step 4: Test Export
- Press S to take screenshot
- **Expected**: Screenshot saved, notification appears

## Upgrade Procedure

### Store upgrade
Install the newer package from Partner Center or the Microsoft Store. The package name and publisher must match the installed app. User data stays. An unsigned MSIX from this repo is the upload file, not the installer.

### Upgrade Verification Checklist
- [ ] Version number updated (Help > About)
- [ ] User settings preserved
- [ ] Recent files list preserved
- [ ] No crashes on first launch post-upgrade

## Uninstallation

### Method 1: Settings App
1. Open Settings > Apps > Installed apps
2. Find "ScalarScope"
3. Click "..." > Uninstall
4. Confirm uninstallation

### Method 2: Right-click
1. Open Start Menu
2. Find "ScalarScope"
3. Right-click > Uninstall

### Post-Uninstall State

**Removed**:
- Application files
- Start Menu shortcut
- App registration

**Preserved** (user choice):
- User settings in `%LOCALAPPDATA%\ScalarScope\`
- Export files in user-chosen locations

**To fully remove user data**:
```powershell
Remove-Item -Recurse "$env:LOCALAPPDATA\ScalarScope"
```

## Troubleshooting

### Installation Fails

**Symptom**: "App Installer cannot install this package"

**Solutions**:
1. Enable Developer Mode: Settings > Update & Security > For Developers
2. Or enable sideloading in Group Policy
3. Check Windows version meets minimum requirements

### App Won't Start

**Symptom**: App crashes immediately or shows blank window

**Solutions**:
1. Check GPU drivers are up to date
2. Try: `ScalarScope.exe --software-rendering`
3. Check Event Viewer for crash details
4. Generate support bundle: Help > Create Support Bundle

### Performance Issues

**Symptom**: Choppy playback, high CPU usage

**Solutions**:
1. Ensure GPU acceleration is enabled
2. Reduce export resolution if memory-constrained
3. Close other GPU-intensive applications

## VM Installation Notes

When testing in a virtual machine:

### Hyper-V
- Enable "Enhanced Session Mode" for GPU acceleration
- Allocate at least 4 GB RAM to VM

### VMware
- Enable 3D acceleration in VM settings
- Install VMware Tools for graphics drivers

### VirtualBox
- Enable 3D acceleration
- Install Guest Additions

## Verification Checklist

Use this checklist to certify a clean install:

```
[ ] Downloaded from official source
[ ] Checksum verified
[ ] Installation completed without errors
[ ] App launches within 3 seconds
[ ] Can load training run file
[ ] Playback works smoothly
[ ] Export creates valid PNG
[ ] Keyboard shortcuts respond
[ ] Help menu opens
[ ] About shows correct version
```

## Support

If you encounter issues not covered here:

1. Check [GitHub Issues](https://github.com/mcp-tool-shop-org/scalarscope/issues)
2. Create a support bundle: Help > Create Support Bundle
3. Open a new issue with the support bundle attached
