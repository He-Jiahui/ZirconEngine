[CmdletBinding()]
param(
    [string]$ArtifactOutputDirectory = (Join-Path $env:LOCALAPPDATA ("Temp\zircon-mvp-product-inputs-" + [guid]::NewGuid().ToString("N")))
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$pathResolverRepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
Import-Module (Join-Path $pathResolverRepoRoot "tools\WindowsPathResolver.psm1") -Force -ErrorAction Stop

function Get-MvpProductBuildRequests {
    # The client and editor-host cdylibs share a file name, so feature provenance requires separate groups.
    return @(
        [pscustomobject]@{
            Package     = "zircon_app"
            Bin         = "zircon_runtime"
            Features    = "target-client,platform-winit,input-gamepad,gamepad-gilrs"
            ArtifactName = "zircon_runtime.exe"
            OutputGroup = "runtime"
        },
        [pscustomobject]@{
            Package     = "zircon_runtime"
            Bin         = $null
            Features    = "target-client,platform-winit,input-gamepad,gamepad-gilrs"
            ArtifactName = "zircon_runtime.dll"
            OutputGroup = "runtime"
        },
        [pscustomobject]@{
            Package     = "zircon_app"
            Bin         = "zircon_editor"
            Features    = "target-editor-host"
            ArtifactName = "zircon_editor.exe"
            OutputGroup = "editor"
        },
        [pscustomobject]@{
            Package     = "zircon_runtime"
            Bin         = $null
            Features    = "target-editor-host"
            ArtifactName = "zircon_runtime.dll"
            OutputGroup = "editor"
        }
    )
}

function Assert-MvpProductInputDirectory {
    param(
        [Parameter(Mandatory)]
        [string]$Path
    )

    $resolution = Resolve-ZirconWindowsPath -Path $Path
    $resolvedPath = $resolution.OperationalPath
    $displayPath = $resolution.DisplayPath
    $driveRoot = [System.IO.Path]::GetPathRoot($displayPath)
    if ($driveRoot -notmatch "^[A-Za-z]:\\$") {
        throw "-ArtifactOutputDirectory must resolve to a local drive: $displayPath"
    }
    if ($driveRoot -in @("D:\", "E:\", "F:\")) {
        throw "-ArtifactOutputDirectory must be outside coordinator-governed D/E/F roots: $displayPath"
    }

    if ([System.IO.Directory]::Exists($resolvedPath) -and
        [System.IO.Directory]::EnumerateFileSystemEntries($resolvedPath).GetEnumerator().MoveNext()) {
        throw "-ArtifactOutputDirectory must be empty to preserve product evidence: $resolvedPath"
    }

    return $resolvedPath
}

function Invoke-MvpProductInputBuild {
    param(
        [Parameter(Mandatory)]
        [string]$OutputDirectory
    )

    $repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
    $validator = Join-Path $repoRoot ".codex\skills\zircon-dev\scripts\validate-matrix.ps1"
    if (-not [System.IO.File]::Exists($validator)) {
        throw "Missing managed Cargo validator: $validator"
    }

    $resolvedOutputDirectory = Assert-MvpProductInputDirectory -Path $OutputDirectory
    [System.IO.Directory]::CreateDirectory($resolvedOutputDirectory) | Out-Null
    $published = [System.Collections.Generic.List[object]]::new()

    foreach ($request in (Get-MvpProductBuildRequests)) {
        $groupDirectory = Join-ZirconWindowsPath -Path $resolvedOutputDirectory -ChildPath $request.OutputGroup
        [System.IO.Directory]::CreateDirectory($groupDirectory) | Out-Null
        $validatorArguments = @(
            "-NoProfile",
            "-ExecutionPolicy", "Bypass",
            "-File", $validator,
            "-Package", $request.Package,
            "-NoDefaultFeatures",
            "-Features", $request.Features,
            "-SkipTest",
            "-Ephemeral",
            "-ArtifactOutputDirectory", $groupDirectory,
            "-PublishArtifact", $request.ArtifactName
        )
        if (-not [string]::IsNullOrWhiteSpace($request.Bin)) {
            $validatorArguments += @("-Bin", $request.Bin)
        }

        & powershell.exe @validatorArguments
        if ($LASTEXITCODE -ne 0) {
            throw "Managed product build failed for $($request.Package) $($request.ArtifactName)."
        }

        $artifactPath = Join-ZirconWindowsPath -Path $groupDirectory -ChildPath $request.ArtifactName
        if (-not [System.IO.File]::Exists($artifactPath)) {
            throw "Managed product build did not publish the declared artifact: $artifactPath"
        }
        $published.Add([pscustomobject]@{
                Package      = $request.Package
                Bin          = $request.Bin
                Features     = $request.Features
                OutputGroup  = $request.OutputGroup
                ArtifactName = $request.ArtifactName
                Path         = $artifactPath
                Bytes        = [System.IO.FileInfo]::new($artifactPath).Length
                Sha256       = (Get-FileHash -LiteralPath $artifactPath -Algorithm SHA256).Hash
            }) | Out-Null
    }

    $summary = [pscustomobject]@{
        schema_version           = 1
        generated_at_utc         = [DateTime]::UtcNow.ToString("o")
        artifact_output_directory = $resolvedOutputDirectory
        artifacts                = @($published)
    }
    $summaryPath = Join-ZirconWindowsPath -Path $resolvedOutputDirectory -ChildPath "mvp-product-inputs.json"
    $summary | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $summaryPath -Encoding UTF8
    Write-Host "MVP product input manifest: $summaryPath"
    return $summary
}

if ($env:MVP_PRODUCT_INPUTS_TEST_MODE -ne "1") {
    Invoke-MvpProductInputBuild -OutputDirectory $ArtifactOutputDirectory
}
