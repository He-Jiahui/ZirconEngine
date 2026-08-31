Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Import-Module (Join-Path $PSScriptRoot 'MvpBuildSet.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpProductInputManifest.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path (Split-Path -Parent $PSScriptRoot) 'WindowsPathResolver.psm1') -Force -ErrorAction Stop

function Get-RenderExtractManifestProperty {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Label
    )

    $property = $Value.PSObject.Properties[$Name]
    if ($null -eq $property -or $null -eq $property.Value) {
        throw "$Label is missing '$Name'."
    }
    return $property.Value
}

function Resolve-RenderExtractProfilingSourceIdentity {
    param([Parameter(Mandatory)][string]$ManifestPath)

    $manifestResolution = Resolve-ZirconWindowsPath -Path $ManifestPath
    if (-not [IO.File]::Exists($manifestResolution.OperationalPath)) {
        throw "Profiling input manifest does not exist: $($manifestResolution.DisplayPath)"
    }
    try {
        $manifest = [IO.File]::ReadAllText($manifestResolution.OperationalPath) |
            ConvertFrom-Json -ErrorAction Stop
    }
    catch {
        throw "Profiling input manifest is not valid JSON: $($manifestResolution.DisplayPath): $($_.Exception.Message)"
    }
    if ([int](Get-RenderExtractManifestProperty `
                -Value $manifest `
                -Name 'schema_version' `
                -Label 'Profiling input manifest') -ne 3) {
        throw 'Profiling input manifest schema_version must be 3.'
    }
    if ([string](Get-RenderExtractManifestProperty `
                -Value $manifest `
                -Name 'cargo_profile' `
                -Label 'Profiling input manifest') -ne 'profiling') {
        throw 'Profiling input manifest cargo_profile must be profiling.'
    }

    $sourceFingerprint = [string](Get-RenderExtractManifestProperty `
        -Value $manifest `
        -Name 'source_fingerprint' `
        -Label 'Profiling input manifest')
    if ($sourceFingerprint -notmatch '^[0-9A-F]{64}$') {
        throw 'Profiling input manifest source_fingerprint must be an uppercase SHA-256 value.'
    }
    $buildSetBinding = Get-RenderExtractManifestProperty `
        -Value $manifest `
        -Name 'build_set' `
        -Label 'Profiling input manifest'
    $buildSetId = [string](Get-RenderExtractManifestProperty `
        -Value $buildSetBinding `
        -Name 'build_set_id' `
        -Label 'Profiling input BuildSet')
    $gitRevision = [string](Get-RenderExtractManifestProperty `
        -Value $buildSetBinding `
        -Name 'git_revision' `
        -Label 'Profiling input BuildSet')
    $dirtyOverlaySha256 = [string](Get-RenderExtractManifestProperty `
        -Value $buildSetBinding `
        -Name 'dirty_overlay_sha256' `
        -Label 'Profiling input BuildSet')
    $buildSetManifestRelativePath = [string](Get-RenderExtractManifestProperty `
        -Value $buildSetBinding `
        -Name 'manifest_relative_path' `
        -Label 'Profiling input BuildSet')
    if ($buildSetManifestRelativePath -cne 'build-set/build-set.json') {
        throw "Profiling input BuildSet manifest_relative_path must be 'build-set/build-set.json'."
    }

    $manifestDirectory = [IO.Path]::GetDirectoryName($manifestResolution.OperationalPath)
    $buildSetManifestPath = Join-ZirconWindowsPath `
        -Path $manifestDirectory `
        -ChildPath 'build-set\build-set.json'
    $buildSet = Assert-MvpProductBuildSet -ManifestPath $buildSetManifestPath
    if (-not $buildSetId.Equals([string]$buildSet.build_set_id, [StringComparison]::Ordinal) -or
        -not $gitRevision.Equals([string]$buildSet.git_revision, [StringComparison]::Ordinal) -or
        -not $dirtyOverlaySha256.Equals([string]$buildSet.dirty_overlay_sha256, [StringComparison]::Ordinal)) {
        throw 'Profiling input BuildSet binding does not match its verified manifest.'
    }
    if (-not $sourceFingerprint.Equals([string]$buildSet.build_set_id, [StringComparison]::Ordinal)) {
        throw 'Profiling input source_fingerprint must equal its verified BuildSetId.'
    }

    return [pscustomobject]@{
        manifest = $manifest
        manifest_path = $manifestResolution.OperationalPath
        manifest_directory = $manifestDirectory
        manifest_sha256 = Get-MvpProductInputFileSha256 -Path $manifestResolution.OperationalPath
        source_fingerprint = $sourceFingerprint
        build_set_id = [string]$buildSet.build_set_id
        build_set_manifest_sha256 = Get-MvpProductInputFileSha256 -Path $buildSet.manifest_path
        build_set = $buildSet
    }
}

Export-ModuleMember -Function @(
    'Get-RenderExtractManifestProperty',
    'Resolve-RenderExtractProfilingSourceIdentity'
)
