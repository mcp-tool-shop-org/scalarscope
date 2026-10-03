# Pack the Rust review as the unsigned Store update.
# Partner Center signs the upload. This script does not sign and does not upload.
$ErrorActionPreference = 'Stop'

$root = Resolve-Path (Join-Path $PSScriptRoot '..')
$exe = Join-Path $root 'rust\target\release\scalarscope.exe'
if (-not (Test-Path $exe)) {
    throw 'Build the release binary first: cargo build --release --manifest-path rust/Cargo.toml'
}

$makeappx = Get-ChildItem 'C:\Program Files (x86)\Windows Kits\10\bin' -Recurse -Filter makeappx.exe |
    Where-Object { $_.FullName -match '\\x64\\makeappx.exe$' } |
    Sort-Object FullName -Descending |
    Select-Object -First 1
if (-not $makeappx) {
    throw 'makeappx.exe was not found in the Windows SDK.'
}

$stage = Join-Path ([System.IO.Path]::GetTempPath()) ('scalarscope-msix-' + [guid]::NewGuid().ToString('n'))
New-Item -ItemType Directory -Force -Path $stage | Out-Null
try {
    Copy-Item $exe (Join-Path $stage 'scalarscope.exe')
    Copy-Item (Join-Path $PSScriptRoot 'AppxManifest.xml') (Join-Path $stage 'AppxManifest.xml')
    $assets = @(Get-ChildItem (Join-Path $PSScriptRoot 'assets') -Filter '*.png' -ErrorAction SilentlyContinue)
    if ($assets.Count -lt 7) {
        throw 'packaging/assets needs the seven tile and splash images.'
    }
    Copy-Item $assets.FullName $stage

    $release = Join-Path $root 'release'
    New-Item -ItemType Directory -Force -Path $release | Out-Null
    $msixName = 'ScalarScope_3.0.0.0_x64.msix'
    $msix = Join-Path $release $msixName
    & $makeappx.FullName pack /d $stage /p $msix /o
    if ($LASTEXITCODE -ne 0) {
        throw "makeappx failed with exit $LASTEXITCODE"
    }

    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $zip = [System.IO.Compression.ZipFile]::OpenRead($msix)
    try {
        $manifestEntry = $zip.GetEntry('AppxManifest.xml')
        if (-not $manifestEntry) { throw 'The package has no AppxManifest.xml' }
        $reader = New-Object System.IO.StreamReader($manifestEntry.Open())
        [xml]$manifest = $reader.ReadToEnd()
        $reader.Close()
        $identity = $manifest.Package.Identity
        if ($identity.Name -ne 'mcp-tool-shop.ScalarScope') { throw "Package name is $($identity.Name)" }
        if ($identity.Publisher -ne 'CN=5305D976-6952-4F00-9C21-3A5DB090359F') { throw 'Publisher does not match the Store listing.' }
        if ($identity.Version -ne '3.0.0.0') { throw "Version is $($identity.Version)" }
        if ($identity.ProcessorArchitecture -ne 'x64') { throw "Architecture is $($identity.ProcessorArchitecture)" }
        $executable = $manifest.Package.Applications.Application.Executable
        if ($executable -ne 'scalarscope.exe') { throw "Executable is $executable" }
        $publisher = $manifest.Package.Properties.PublisherDisplayName
        if ($publisher -ne 'mcp-tool-shop') { throw "Publisher display name is $publisher" }
    }
    finally {
        $zip.Dispose()
    }

    $uploadName = 'ScalarScope_3.0.0.0_Store.msixupload'
    $upload = Join-Path $release $uploadName
    $bundleStage = Join-Path ([System.IO.Path]::GetTempPath()) ('scalarscope-upload-' + [guid]::NewGuid().ToString('n'))
    New-Item -ItemType Directory -Force -Path $bundleStage | Out-Null
    try {
        Copy-Item $msix (Join-Path $bundleStage $msixName)
        $types = '<?xml version="1.0" encoding="utf-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="msix" ContentType="application/vnd.ms-appx" /></Types>'
        [System.IO.File]::WriteAllText((Join-Path $bundleStage '[Content_Types].xml'), $types)
        if (Test-Path $upload) { Remove-Item $upload -Force }
        [System.IO.Compression.ZipFile]::CreateFromDirectory($bundleStage, $upload)
    }
    finally {
        Remove-Item $bundleStage -Recurse -Force
    }

    $checks = Join-Path $release 'checksums.txt'
    $uploadHash = (Get-FileHash $upload -Algorithm SHA256).Hash
    $msixHash = (Get-FileHash $msix -Algorithm SHA256).Hash
    "$uploadHash  $uploadName`r`n$msixHash  $msixName" | Set-Content -Encoding ascii $checks
    Write-Output "Packed $msixName"
    Write-Output "Identity mcp-tool-shop.ScalarScope 3.0.0.0 x64"
}
finally {
    if (Test-Path $stage) { Remove-Item $stage -Recurse -Force }
}
