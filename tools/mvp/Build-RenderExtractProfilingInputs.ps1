[CmdletBinding()]
param(
    [string]$ArtifactOutputDirectory = (Join-Path 'E:\ZirconBuilds' ("mvp-product-inputs-profile-" + [guid]::NewGuid().ToString('N')))
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Import-Module (Join-Path $PSScriptRoot 'MvpProductInputManifest.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $repoRoot 'tools\WindowsPathResolver.psm1') -Force -ErrorAction Stop

function Get-RenderExtractProfilingBuildRequests {
    $runtimeFeatures = 'target-client,platform-winit,input-gamepad,gamepad-gilrs,profiling'
    $editorFeatures = 'target-editor-host,profiling'
    return @(
        [pscustomobject]@{
            logical_id   = 'runtime-profile-executable'
            Product      = 'runtime'
            Package      = 'zircon_app'
            Bin          = 'zircon_runtime'
            Features     = $runtimeFeatures
            CargoProfile = 'profiling'
            ArtifactName = 'zircon_runtime.exe'
        },
        [pscustomobject]@{
            logical_id   = 'runtime-profile-library'
            Product      = 'runtime'
            Package      = 'zircon_runtime'
            Bin          = $null
            Features     = $runtimeFeatures
            CargoProfile = 'profiling'
            ArtifactName = 'zircon_runtime.dll'
        },
        [pscustomobject]@{
            logical_id   = 'editor-profile-executable'
            Product      = 'editor'
            Package      = 'zircon_app'
            Bin          = 'zircon_editor'
            Features     = $editorFeatures
            CargoProfile = 'profiling'
            ArtifactName = 'zircon_editor.exe'
        },
        [pscustomobject]@{
            logical_id   = 'editor-profile-library'
            Product      = 'editor'
            Package      = 'zircon_runtime'
            Bin          = $null
            Features     = $editorFeatures
            CargoProfile = 'profiling'
            ArtifactName = 'zircon_runtime.dll'
        }
    )
}

function Assert-RenderExtractProfilingInputDirectory {
    param(
        [Parameter(Mandatory)]
        [string]$Path
    )

    $resolution = Resolve-ZirconWindowsPath -Path $Path
    $resolvedPath = $resolution.OperationalPath
    $displayPath = $resolution.DisplayPath
    if ($displayPath -notmatch '^[D-F]:\\ZirconBuilds\\mvp-product-inputs-profile-(?:[A-Za-z0-9][A-Za-z0-9._-]*)(?:\\|$)') {
        throw "-ArtifactOutputDirectory render-extract profiling inputs must resolve under D:\ZirconBuilds\mvp-product-inputs-profile-*: $displayPath"
    }

    $directoryExists = [System.IO.Directory]::Exists($resolvedPath)
    if ($directoryExists -and
        [System.IO.Directory]::EnumerateFileSystemEntries($resolvedPath).GetEnumerator().MoveNext()) {
        throw "-ArtifactOutputDirectory must be empty to preserve profiling evidence: $displayPath"
    }
    return $resolvedPath
}

function Get-RenderExtractProfilingValidatorArguments {
    param(
        [Parameter(Mandatory)]
        [string]$Validator,
        [Parameter(Mandatory)]
        [string]$OutputDirectory,
        [Parameter(Mandatory)]
        [pscustomobject]$Request
    )

    $arguments = [System.Collections.Generic.List[string]]::new()
    $arguments.AddRange([string[]]@(
        '-NoProfile',
        '-ExecutionPolicy', 'Bypass',
        '-File', $Validator,
        '-Package', $Request.Package,
        '-NoDefaultFeatures',
        '-Features', $Request.Features
    ))
    if (-not [string]::IsNullOrWhiteSpace($Request.Bin)) {
        $arguments.Add('-Bin')
        $arguments.Add($Request.Bin)
    }
    $arguments.AddRange([string[]]@(
        '-CargoProfile', $Request.CargoProfile,
        '-SkipTest',
        '-MvpProductInputArtifactOutput',
        '-ArtifactOutputDirectory', (Join-ZirconWindowsPath -Path $OutputDirectory -ChildPath $Request.Product),
        '-PublishArtifact', $Request.ArtifactName
    ))
    return $arguments.ToArray()
}

function Assert-RenderExtractProfilingSourceFingerprint {
    param(
        [Parameter(Mandatory)]
        [string]$ExpectedFingerprint,
        [Parameter(Mandatory)]
        [string]$Phase
    )

    $actualFingerprint = Get-MvpSourceFingerprint -RepositoryRoot $repoRoot
    if (-not $actualFingerprint.Equals($ExpectedFingerprint, [StringComparison]::Ordinal)) {
        throw "Render-extract profiling source fingerprint changed during $Phase. Rebuild the profiling input from one source snapshot."
    }
}

function Write-RenderExtractProfilingInputManifest {
    param(
        [Parameter(Mandatory)]
        [string]$Path,
        [Parameter(Mandatory)]
        [string]$SourceFingerprint,
        [Parameter(Mandatory)]
        [string]$ArtifactOutputDirectory
    )

    if ($SourceFingerprint -notmatch '^[0-9A-F]{64}$') {
        throw 'Render-extract profiling source fingerprint must be an uppercase SHA-256 value.'
    }

    $artifacts = [System.Collections.Generic.List[object]]::new()
    foreach ($request in @(Get-RenderExtractProfilingBuildRequests)) {
        $artifactPath = Join-ZirconWindowsPath `
            -Path (Join-ZirconWindowsPath -Path $ArtifactOutputDirectory -ChildPath $request.Product) `
            -ChildPath $request.ArtifactName
        $artifactResolution = Resolve-ZirconWindowsPath -Path $artifactPath
        if (-not [System.IO.File]::Exists($artifactResolution.OperationalPath)) {
            throw "Render-extract profiling artifact does not exist: $artifactPath"
        }

        $artifact = [System.IO.FileInfo]::new($artifactResolution.OperationalPath)
        $artifacts.Add([ordered]@{
                logical_id = $request.logical_id
                product    = $request.Product
                package    = $request.Package
                bin        = $request.Bin
                features   = $request.Features
                path       = $artifactResolution.DisplayPath
                bytes      = [Int64]$artifact.Length
                sha256     = Get-MvpProductInputFileSha256 -Path $artifactResolution.OperationalPath
            })
    }

    $manifest = [ordered]@{
        schema_version     = 2
        generated_at_utc   = [DateTime]::UtcNow.ToString('o')
        source_fingerprint = $SourceFingerprint
        cargo_profile      = 'profiling'
        artifacts          = @($artifacts)
    }
    $stream = $null
    $writer = $null
    try {
        try {
            $stream = [System.IO.FileStream]::new(
                $Path,
                [System.IO.FileMode]::CreateNew,
                [System.IO.FileAccess]::Write,
                [System.IO.FileShare]::None
            )
        }
        catch [System.IO.IOException] {
            throw "Refusing to overwrite existing render-extract profiling input manifest: $Path"
        }
        $writer = [System.IO.StreamWriter]::new($stream, [System.Text.UTF8Encoding]::new($false))
        $stream = $null
        $writer.Write(($manifest | ConvertTo-Json -Depth 4))
    }
    finally {
        if ($null -ne $writer) {
            $writer.Dispose()
        }
        elseif ($null -ne $stream) {
            $stream.Dispose()
        }
    }
    return $manifest
}

function Invoke-RenderExtractProfilingInputBuild {
    param(
        [Parameter(Mandatory)]
        [string]$OutputDirectory
    )

    $validator = Join-Path $repoRoot '.codex\skills\zircon-dev\scripts\validate-matrix.ps1'
    if (-not [System.IO.File]::Exists($validator)) {
        throw "Missing managed Cargo validator: $validator"
    }

    $resolvedOutputDirectory = Assert-RenderExtractProfilingInputDirectory -Path $OutputDirectory
    $sourceFingerprint = Get-MvpSourceFingerprint -RepositoryRoot $repoRoot
    Assert-RenderExtractProfilingSourceFingerprint `
        -ExpectedFingerprint $sourceFingerprint `
        -Phase 'profiling input preflight'
    foreach ($request in @(Get-RenderExtractProfilingBuildRequests)) {
        Assert-RenderExtractProfilingSourceFingerprint `
            -ExpectedFingerprint $sourceFingerprint `
            -Phase "before $($request.logical_id) build"
        $validatorArguments = Get-RenderExtractProfilingValidatorArguments `
            -Validator $validator `
            -OutputDirectory $resolvedOutputDirectory `
            -Request $request
        & powershell.exe @validatorArguments
        if ($LASTEXITCODE -ne 0) {
            throw "Managed profiling build failed for $($request.logical_id)."
        }
        Assert-RenderExtractProfilingSourceFingerprint `
            -ExpectedFingerprint $sourceFingerprint `
            -Phase "after $($request.logical_id) build"

        $artifactPath = Join-ZirconWindowsPath `
            -Path (Join-ZirconWindowsPath -Path $resolvedOutputDirectory -ChildPath $request.Product) `
            -ChildPath $request.ArtifactName
        if (-not [System.IO.File]::Exists($artifactPath)) {
            throw "Managed profiling build did not publish $($request.logical_id): $artifactPath"
        }
    }

    $manifestPath = Join-ZirconWindowsPath `
        -Path $resolvedOutputDirectory `
        -ChildPath 'render-extract-profiling-inputs.json'
    $manifest = Write-RenderExtractProfilingInputManifest `
        -Path $manifestPath `
        -SourceFingerprint $sourceFingerprint `
        -ArtifactOutputDirectory $resolvedOutputDirectory
    Assert-RenderExtractProfilingSourceFingerprint `
        -ExpectedFingerprint $sourceFingerprint `
        -Phase 'profiling input manifest publication'

    $manifestDisplayPath = (Resolve-ZirconWindowsPath -Path $manifestPath).DisplayPath
    Write-Host "Render-extract profiling input manifest: $manifestDisplayPath"
    return $manifest
}

if ($env:RENDER_EXTRACT_PROFILING_INPUTS_TEST_MODE -ne '1') {
    Invoke-RenderExtractProfilingInputBuild -OutputDirectory $ArtifactOutputDirectory
}
