# Release Process

This document describes the release process for ScalarScope.

## Version Scheme

We follow [Semantic Versioning](https://semver.org/):

```
MAJOR.MINOR.PATCH[-PRERELEASE]
```

### Pre-release Tags
- `rc.N` - Release Candidate (e.g., `1.0.0-rc.1`)
- `beta.N` - Beta release (e.g., `1.0.0-beta.1`)
- `alpha.N` - Alpha release (e.g., `1.0.0-alpha.1`)

### Current Version
- **Display version 3.1.1** — `version` in `rust/Cargo.toml`. Settings > About shows this string.
- **Package identity version 3.1.1.0** — `packaging/AppxManifest.xml`, which `pack.ps1` reads. The embedded `rust/app.manifest` and the MAUI `Package.appxmanifest` carry the same value, and a test holds them equal.
- **ApplicationVersion 32** — the package integer in `src/ScalarScope/ScalarScope.csproj`, beside csproj `Version` 3.1.1.0 and `ApplicationDisplayVersion` 3.1.1. It is not the About string.
- **The tag** is `v3.1.1.0`. `.github/workflows/release.yml` runs only for it, and `build.yml` names its artifacts with it.
- These are separate fields. Read each one. Do not collapse them into a single label.
- Store `9P3HT1PHBKQK`. Package name `mcp-tool-shop.ScalarScope`. Publisher `CN=5305D976-6952-4F00-9C21-3A5DB090359F`. Publisher display name `mcp-tool-shop`.
- Unsigned MSIX. Partner Center signs it on ingestion.
- **3.1.0** - Previous Microsoft Store release, the first Rust package.
## Release Checklist

### Before Tagging

1. **Code Freeze**
   - [ ] All features for this release are merged
   - [ ] No known critical bugs
   - [ ] CI is green

2. **Documentation**
   - [ ] CHANGELOG.md updated with all changes
   - [ ] README.md reflects current features
   - [ ] QUICK_REFERENCE.md is accurate

3. **Testing**
   - [ ] All unit tests pass
   - [ ] Manual testing on Windows completed
   - [ ] Soak test passed (for RC+)
   - [ ] Light and dark theme verified

4. **Version Bump**
   - [ ] Update every field under Current Version, above
   - [ ] Commit: "chore: bump version to X.Y.Z"

### Tagging

```bash
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin vX.Y.Z
```

### After Tagging

1. **GitHub Release**
   - [ ] Create release from tag
   - [ ] Copy changelog section to release notes
   - [ ] Attach MSIX installer
   - [ ] Attach checksums file

2. **Artifacts**
   - [ ] Unsigned `release/ScalarScope_3.1.1.0_x64.msix` is attached
   - [ ] Unsigned `release/ScalarScope_3.1.1.0_Store.msixupload` is attached
   - [ ] `release/checksums.txt` is attached
   - [ ] Partner Center signs the upload during ingestion. This repo does not produce a signed MSIX.

3. **Announcement**
   - [ ] Update README badges if needed
   - [ ] Post to relevant channels

## Artifact Naming

The pack script writes these names. The package version stays `3.1.1.0`.

```
release/ScalarScope_3.1.1.0_x64.msix
release/ScalarScope_3.1.1.0_Store.msixupload
release/checksums.txt
```

Do not look for `ScalarScope-{version}-{arch}.msix`. Partner Center signs the upload. The repo copy stays unsigned.

## Changelog Rules

Every entry should:
1. Be user-facing (not internal refactoring unless significant)
2. Describe what changed, not how
3. Group by: Added, Changed, Deprecated, Removed, Fixed, Security
4. Include issue/PR references where applicable

## Rollback Procedure

If a release has critical issues:

1. Immediately tag a patch release fixing the issue, OR
2. If unfixable quickly:
   ```bash
   git tag -d vX.Y.Z
   git push origin :refs/tags/vX.Y.Z
   ```
3. Remove GitHub release
4. Communicate to users via GitHub issue

## Support Lifecycle

| Version | Status | Support Until |
|---------|--------|---------------|
| 3.0.x   | Active (display version 3.0.0) | Current line |
| 2.0.x   | Previous Store release | — |
| 1.0.x   | Not the current line | — |
| 0.x     | EOL | — |
