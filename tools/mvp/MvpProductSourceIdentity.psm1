Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Import-Module (Join-Path $PSScriptRoot 'MvpBuildSet.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpProductInputManifest.psm1') -Force -ErrorAction Stop

function Resolve-MvpProductSourceIdentity {
    param([Parameter(Mandatory)][string]$ManifestPath)

    $productInputs = Resolve-MvpProductInputManifest -Path $ManifestPath
    if ($null -eq $productInputs.build_set) {
        throw 'ProductInputManifest must bind one verified BuildSet.'
    }
    if ([string]$productInputs.build_set.manifest_relative_path -cne 'build-set/build-set.json') {
        throw "ProductInputManifest BuildSet manifest_relative_path must be 'build-set/build-set.json'."
    }

    $manifestDirectory = [IO.Path]::GetDirectoryName([string]$productInputs.operation_path)
    $buildSetManifestPath = [IO.Path]::Combine($manifestDirectory, 'build-set\build-set.json')
    $buildSet = Assert-MvpProductBuildSet -ManifestPath $buildSetManifestPath
    if (-not ([string]$productInputs.build_set.build_set_id).Equals(
            [string]$buildSet.build_set_id,
            [StringComparison]::Ordinal) -or
        -not ([string]$productInputs.build_set.git_revision).Equals(
            [string]$buildSet.git_revision,
            [StringComparison]::Ordinal) -or
        -not ([string]$productInputs.build_set.dirty_overlay_sha256).Equals(
            [string]$buildSet.dirty_overlay_sha256,
            [StringComparison]::Ordinal)) {
        throw 'ProductInputManifest BuildSet binding does not match its verified manifest.'
    }
    if (-not ([string]$productInputs.source_fingerprint).Equals(
            [string]$buildSet.build_set_id,
            [StringComparison]::Ordinal)) {
        throw 'ProductInputManifest source_fingerprint must equal its verified BuildSetId.'
    }

    return [pscustomobject]@{
        manifest_path = [string]$productInputs.operation_path
        manifest_sha256 = [string]$productInputs.sha256
        source_fingerprint = [string]$buildSet.build_set_id
        build_set_id = [string]$buildSet.build_set_id
        build_set = $buildSet
        product_inputs = $productInputs
    }
}

Export-ModuleMember -Function 'Resolve-MvpProductSourceIdentity'
