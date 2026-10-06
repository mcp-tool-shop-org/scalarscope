# FixtureTests is the suite CI runs. The Rust suite is the scientific lock.
# This does not pack or install the MSIX, and it does not run the two-hour soak.
$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot
dotnet test tests/ScalarScope.FixtureTests/ScalarScope.FixtureTests.csproj --configuration Release --verbosity minimal
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo test --locked --all-targets --manifest-path rust/Cargo.toml
exit $LASTEXITCODE
