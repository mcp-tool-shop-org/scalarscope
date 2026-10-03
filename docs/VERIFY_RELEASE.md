# Release Verification

Use this to check a ScalarScope package before it goes to Partner Center, and to check the build you made yourself.

## Package identity

The Store product is `9P3HT1PHBKQK`. An update must carry exactly:

| Field | Value |
| --- | --- |
| Name | `mcp-tool-shop.ScalarScope` |
| Publisher | `CN=5305D976-6952-4F00-9C21-3A5DB090359F` |
| Publisher display name | `mcp-tool-shop` |
| Package Family Name | `mcp-tool-shop.ScalarScope_yn6b8xqrexa5j` |
| Version | `3.0.0.0` for the next upload |

Read the identity out of the MSIX:

```powershell
Add-Type -AssemblyName System.IO.Compression.FileSystem
$msix = "ScalarScope_3.0.0.0_x64.msix"
$zip = [System.IO.Compression.ZipFile]::OpenRead((Resolve-Path $msix))
$entry = $zip.GetEntry("AppxManifest.xml")
$reader = New-Object System.IO.StreamReader($entry.Open())
$xml = $reader.ReadToEnd()
$reader.Close(); $zip.Dispose()
[regex]::Match($xml, '<Identity[^>]*/>').Value
[regex]::Match($xml, '<PublisherDisplayName>[^<]+</PublisherDisplayName>').Value
```

The name, publisher, and version in that line are the Store update key. A different name or publisher is a new product, not an update.

## Checksum

The release workflow writes `checksums.txt` next to the MSIX. Compare it with the file you have:

```powershell
Get-FileHash .\ScalarScope_3.0.0.0_x64.msix -Algorithm SHA256
Get-Content .\checksums.txt
```

## Signing

The store file is `ScalarScope_3.0.0.0_Store.msixupload`. It is a zip of the unsigned MSIX. Partner Center already has `ScalarScope_1.0.3.0_x64.msix` and `ScalarScope_v2.0.0_Store.msixupload`. Partner Center signs the new upload during ingestion. Windows will not install the unsigned MSIX from a double-click. That is the upload shape, not a broken build.

After Partner Center publishes, the installed app's publisher display name is `mcp-tool-shop`.

## Build from source

```powershell
git clone https://github.com/mcp-tool-shop-org/scalarscope.git
cd scalarscope
dotnet publish src/ScalarScope/ScalarScope.csproj -c Release -f net9.0-windows10.0.19041.0 -p:AppxPackageSigningEnabled=false
```

The MSIX lands under `src/ScalarScope/bin/Release/net9.0-windows10.0.19041.0/win-x64/AppPackages/`.
