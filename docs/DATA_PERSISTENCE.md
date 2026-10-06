# Data Persistence Rules

This document describes what ScalarScope stores, where it is stored, and what an install, upgrade, or uninstall does to those files.

The app does not write `settings.json`, `recent.json`, or `window.json`. A backup or a wipe of those names does not save or remove the review.

## Packaged review state

A packaged run keeps these files in the package LocalState folder (`FileSystem.AppDataDirectory` for the MSIX):

`%LOCALAPPDATA%\Packages\mcp-tool-shop.ScalarScope_yn6b8xqrexa5j\LocalState`

- `preferences.json` — preferences the app writes
- `comparison-log.json` — the comparison log

An unpackaged `cargo run` of the Rust review does not write that folder.

## Crash logs (a different folder)

`CrashReportingService` writes under `%LOCALAPPDATA%\ScalarScope\`:

- `logs\` — crash logs
- `crash.json`
- `session_state.json`

That tree is not the packaged review. Copying or deleting it does not copy or delete `preferences.json` or `comparison-log.json`.

## Lifecycle

### Fresh install

The package LocalState folder starts empty. The app creates `preferences.json` when it saves a preference. Nothing migrates `settings.json`.

### Upgrade

A Store upgrade keeps the package LocalState folder. The package name and publisher stay the same. Crash logs under `%LOCALAPPDATA%\ScalarScope\` are also left in place.

### Uninstall

Uninstall removes the package, including that package's LocalState folder. Crash logs under `%LOCALAPPDATA%\ScalarScope\` stay until you delete that folder.

### Remove crash logs only

```powershell
Remove-Item -Recurse -Force "$env:LOCALAPPDATA\ScalarScope"
```

That command does not remove a packaged review. To remove the packaged review, uninstall the app.

## Backup

Back up the two review files from package LocalState:

```powershell
$local = "$env:LOCALAPPDATA\Packages\mcp-tool-shop.ScalarScope_yn6b8xqrexa5j\LocalState"
Copy-Item "$local\preferences.json" "D:\Backup\ScalarScope\preferences.json"
Copy-Item "$local\comparison-log.json" "D:\Backup\ScalarScope\comparison-log.json"
```

Restore by copying those two files back into the same LocalState folder while the app is closed. Do not restore them into `%LOCALAPPDATA%\ScalarScope\`.

## Privacy

ScalarScope does not collect telemetry, does not phone home, and does not upload user data. Data stays on the machine unless you export it.
