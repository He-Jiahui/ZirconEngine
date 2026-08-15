[CmdletBinding()]
param(
    [string]$ArtifactOutputDirectory = (Join-Path 'E:\ZirconBuilds' ("mvp-product-inputs-" + [guid]::NewGuid().ToString("N")))
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$pathResolverRepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
Import-Module (Join-Path $PSScriptRoot "MvpProductInputManifest.psm1") -Force -ErrorAction Stop
# MvpProductInputManifest imports this module for its own scope. Import it last here so its
# forced module reload also leaves the resolver command visible to this build script's functions.
Import-Module (Join-Path $pathResolverRepoRoot "tools\WindowsPathResolver.psm1") -Force -ErrorAction Stop

function Get-MvpProductBuildRequests {
    # The client and editor-host cdylibs share a file name, so feature provenance requires separate groups.
    return @(Get-MvpProductInputSpecifications | ForEach-Object {
            [pscustomobject]@{
                logical_id = $_.logical_id
                Package = $_.package
                Bin = $_.bin
                Features = $_.features
                OutputGroup = $_.output_group
                ArtifactName = $_.artifact_name
            }
        })
}

function Assert-MvpProductInputSourceFingerprint {
    param(
        [Parameter(Mandatory)]
        [string]$RepositoryRoot,
        [Parameter(Mandatory)]
        [string]$ExpectedFingerprint,
        [Parameter(Mandatory)]
        [string]$Phase
    )

    $actualFingerprint = Get-MvpSourceFingerprint -RepositoryRoot $RepositoryRoot
    if (-not $actualFingerprint.Equals($ExpectedFingerprint, [StringComparison]::Ordinal)) {
        throw "MVP product source fingerprint changed during $Phase. Rebuild all product inputs from one source snapshot."
    }
}

function Assert-MvpProductInputDirectory {
    param(
        [Parameter(Mandatory)]
        [string]$Path
    )

    $resolution = Resolve-ZirconWindowsPath -Path $Path
    $resolvedPath = $resolution.OperationalPath
    $displayPath = $resolution.DisplayPath
    if ($displayPath -notmatch '^[D-F]:\\ZirconBuilds\\mvp-product-inputs-(?:[A-Za-z0-9][A-Za-z0-9._-]*)(?:\\|$)') {
        throw "-ArtifactOutputDirectory MVP product input artifact output must resolve under D:\ZirconBuilds\mvp-product-inputs-*: $displayPath"
    }

    if ([System.IO.Directory]::Exists($resolvedPath) -and
        [System.IO.Directory]::EnumerateFileSystemEntries($resolvedPath).GetEnumerator().MoveNext()) {
        throw "-ArtifactOutputDirectory must be empty to preserve product evidence: $resolvedPath"
    }

    return $resolvedPath
}

function Write-MvpProductInputManifest {
    param(
        [Parameter(Mandatory)]
        [string]$Path,
        [Parameter(Mandatory)]
        $Summary
    )

    $bytes = [Text.UTF8Encoding]::new($false).GetBytes(
        ($Summary | ConvertTo-Json -Depth 4)
    )
    try {
        $stream = [IO.File]::Open(
            $Path,
            [IO.FileMode]::CreateNew,
            [IO.FileAccess]::Write,
            [IO.FileShare]::None
        )
    }
    catch [IO.IOException] {
        if ([IO.File]::Exists($Path)) {
            throw "Refusing to overwrite existing MVP product input manifest: $Path"
        }
        throw
    }
    try {
        $stream.Write($bytes, 0, $bytes.Length)
        $stream.Flush($true)
    }
    finally {
        $stream.Dispose()
    }
}

function Publish-MvpProductInputManifest {
    param(
        [Parameter(Mandatory)]
        [string]$Path,
        [Parameter(Mandatory)]
        $Summary,
        [Parameter(Mandatory)]
        [string]$RepositoryRoot,
        [Parameter(Mandatory)]
        [string]$ExpectedFingerprint
    )

    Write-MvpProductInputManifest -Path $Path -Summary $Summary
    Assert-MvpProductInputSourceFingerprint `
        -RepositoryRoot $RepositoryRoot `
        -ExpectedFingerprint $ExpectedFingerprint `
        -Phase 'after product input manifest publication'
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
    $sourceFingerprint = Get-MvpSourceFingerprint -RepositoryRoot $repoRoot
    [System.IO.Directory]::CreateDirectory($resolvedOutputDirectory) | Out-Null
    $published = [System.Collections.Generic.List[object]]::new()

    foreach ($request in (Get-MvpProductBuildRequests)) {
        Assert-MvpProductInputSourceFingerprint `
            -RepositoryRoot $repoRoot `
            -ExpectedFingerprint $sourceFingerprint `
            -Phase "before $($request.logical_id) build"
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
            "-MvpProductInputArtifactOutput",
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
        Assert-MvpProductInputSourceFingerprint `
            -RepositoryRoot $repoRoot `
            -ExpectedFingerprint $sourceFingerprint `
            -Phase "after $($request.logical_id) build"

        $artifactPath = Join-ZirconWindowsPath -Path $groupDirectory -ChildPath $request.ArtifactName
        if (-not [System.IO.File]::Exists($artifactPath)) {
            throw "Managed product build did not publish the declared artifact: $artifactPath"
        }
        $artifactDisplayPath = (Resolve-ZirconWindowsPath -Path $artifactPath).DisplayPath
        $published.Add([pscustomobject]@{
                LogicalId     = $request.logical_id
                Package      = $request.Package
                Bin          = $request.Bin
                Features     = $request.Features
                OutputGroup  = $request.OutputGroup
                ArtifactName = $request.ArtifactName
                Path         = $artifactDisplayPath
                Bytes        = [System.IO.FileInfo]::new($artifactPath).Length
                Sha256       = Get-MvpProductInputFileSha256 -Path $artifactPath
            }) | Out-Null
    }

    Assert-MvpProductInputSourceFingerprint `
        -RepositoryRoot $repoRoot `
        -ExpectedFingerprint $sourceFingerprint `
        -Phase 'before product input manifest publication'

    $summary = [pscustomobject]@{
        schema_version           = 1
        generated_at_utc         = [DateTime]::UtcNow.ToString("o")
        source_fingerprint       = $sourceFingerprint
        artifact_output_directory = (Resolve-ZirconWindowsPath -Path $resolvedOutputDirectory).DisplayPath
        artifacts                = @($published)
    }
    $summaryPath = Join-ZirconWindowsPath -Path $resolvedOutputDirectory -ChildPath "mvp-product-inputs.json"
    Publish-MvpProductInputManifest `
        -Path $summaryPath `
        -Summary $summary `
        -RepositoryRoot $repoRoot `
        -ExpectedFingerprint $sourceFingerprint
    Write-Host "MVP product input manifest: $((Resolve-ZirconWindowsPath -Path $summaryPath).DisplayPath)"
    return $summary
}

if ($env:MVP_PRODUCT_INPUTS_TEST_MODE -ne "1") {
    Invoke-MvpProductInputBuild -OutputDirectory $ArtifactOutputDirectory
}
