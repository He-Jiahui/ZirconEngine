Set-StrictMode -Version Latest

function Get-ZirconEditorVisualSourceBinding {
    param([Parameter(Mandatory = $true)][string]$RepositoryRoot)

    $relativePaths = [System.Collections.Generic.SortedSet[string]]::new(
        [System.StringComparer]::Ordinal)
    foreach ($relativePath in Get-ZirconProfileCriticalSourcePaths) {
        $relativePaths.Add($relativePath.Replace('\', '/')) | Out-Null
    }
    foreach ($relativePath in Get-ZirconProfileCaptureToolPaths) {
        $relativePaths.Add($relativePath.Replace('\', '/')) | Out-Null
    }
    foreach ($relativePath in @(
            'tools/capture-editor-ui-visual.ps1',
            'tools/editor-ui-visual-interactions.ps1',
            'tools/editor-ui-visual-source-binding.ps1',
            'tools/zircon_editor_ui_visual_oracle.py'
        )) {
        $relativePaths.Add($relativePath) | Out-Null
    }

    $repositoryPrefix = $RepositoryRoot.TrimEnd('\') + '\'
    foreach ($assetRelativeRoot in @('zircon_editor\assets', 'zircon_runtime\assets')) {
        $assetRoot = Join-Path $RepositoryRoot $assetRelativeRoot
        foreach ($asset in Get-ChildItem -LiteralPath $assetRoot -Recurse -File) {
            if (-not $asset.FullName.StartsWith(
                    $repositoryPrefix,
                    [System.StringComparison]::OrdinalIgnoreCase)) {
                throw "Visual acceptance asset escaped the repository root: $($asset.FullName)"
            }
            $relativePaths.Add(
                $asset.FullName.Substring($repositoryPrefix.Length).Replace('\', '/')) | Out-Null
        }
    }

    $sourceFiles = foreach ($relativePath in $relativePaths) {
        $fingerprint = Get-ZirconProfileRequiredFileFingerprint `
            -Path (Join-Path $RepositoryRoot $relativePath) `
            -Description "editor UI source file '$relativePath'"
        [pscustomobject]@{
            relative_path = $relativePath
            sha256 = $fingerprint.sha256
            byte_length = $fingerprint.byte_length
            last_write_utc = $fingerprint.last_write_utc
        }
    }
    $gitMetadata = Get-ZirconProfileGitMetadata -RepoRoot $RepositoryRoot
    $canonical = [System.Text.StringBuilder]::new()
    $canonical.Append('revision=').Append($gitMetadata.revision).Append("`n") | Out-Null
    foreach ($sourceFile in $sourceFiles) {
        $canonical.Append($sourceFile.relative_path).Append("`0").
            Append($sourceFile.sha256).Append("`0").
            Append($sourceFile.byte_length).Append("`n") | Out-Null
    }
    $bytes = [System.Text.Encoding]::UTF8.GetBytes($canonical.ToString())
    $hasher = [System.Security.Cryptography.SHA256]::Create()
    try {
        $sourceSha256 = -join ($hasher.ComputeHash($bytes) | ForEach-Object { $_.ToString('x2') })
    }
    finally {
        $hasher.Dispose()
    }

    [pscustomobject]@{
        source_sha256 = $sourceSha256
        git = $gitMetadata
        critical_source_files = @($sourceFiles)
    }
}

function Get-ZirconEditorVisualBundleAssetBinding {
    param(
        [Parameter(Mandatory = $true)][string]$BundleDirectory,
        [Parameter(Mandatory = $true)][object]$SourceBinding
    )

    $assetRoot = Join-Path $BundleDirectory 'assets'
    if (-not (Test-Path -LiteralPath $assetRoot -PathType Container)) {
        throw "Product bundle assets do not exist: $assetRoot"
    }
    $assetRoot = (Resolve-Path -LiteralPath $assetRoot).Path
    $assetRootPrefix = $assetRoot.TrimEnd('\') + '\'
    $expected = @{}
    foreach ($sourceFile in $SourceBinding.critical_source_files) {
        $relativePath = [string]$sourceFile.relative_path
        $bundleRelativePath = $null
        foreach ($prefix in @('zircon_editor/assets/', 'zircon_runtime/assets/')) {
            if ($relativePath.StartsWith($prefix, [System.StringComparison]::Ordinal)) {
                $bundleRelativePath = $relativePath.Substring($prefix.Length)
                break
            }
        }
        if ($null -eq $bundleRelativePath) {
            continue
        }
        if ($expected.ContainsKey($bundleRelativePath)) {
            throw "Runtime and Editor asset paths collide in the product bundle: $bundleRelativePath"
        }
        $expected[$bundleRelativePath] = $sourceFile
    }

    $actual = @{}
    foreach ($asset in Get-ChildItem -LiteralPath $assetRoot -Recurse -File) {
        if (-not $asset.FullName.StartsWith(
                $assetRootPrefix,
                [System.StringComparison]::OrdinalIgnoreCase)) {
            throw "Product bundle asset escaped its root: $($asset.FullName)"
        }
        $relativePath = $asset.FullName.Substring($assetRootPrefix.Length).Replace('\', '/')
        $actual[$relativePath] = $asset.FullName
    }
    if ($actual.Count -ne $expected.Count) {
        throw "Product bundle asset set differs from current source: expected=$($expected.Count) actual=$($actual.Count)"
    }

    $canonical = [System.Text.StringBuilder]::new()
    $sortedRelativePaths = [System.Collections.Generic.SortedSet[string]]::new(
        [System.StringComparer]::Ordinal)
    foreach ($relativePath in $expected.Keys) {
        $sortedRelativePaths.Add($relativePath) | Out-Null
    }
    foreach ($relativePath in $sortedRelativePaths) {
        if (-not $actual.ContainsKey($relativePath)) {
            throw "Product bundle asset is missing: $relativePath"
        }
        $fingerprint = Get-ZirconProfileRequiredFileFingerprint `
            -Path $actual[$relativePath] `
            -Description "product bundle asset '$relativePath'"
        $sourceFile = $expected[$relativePath]
        if ($fingerprint.sha256 -ne [string]$sourceFile.sha256 -or
            $fingerprint.byte_length -ne [int64]$sourceFile.byte_length) {
            throw "Product bundle asset differs from current source: $relativePath"
        }
        $canonical.Append($relativePath).Append("`0").
            Append($fingerprint.sha256).Append("`0").
            Append($fingerprint.byte_length).Append("`n") | Out-Null
    }
    $bytes = [System.Text.Encoding]::UTF8.GetBytes($canonical.ToString())
    $hasher = [System.Security.Cryptography.SHA256]::Create()
    try {
        $bundleAssetSha256 = -join ($hasher.ComputeHash($bytes) | ForEach-Object { $_.ToString('x2') })
    }
    finally {
        $hasher.Dispose()
    }

    [pscustomobject]@{
        root = $assetRoot
        bundle_asset_sha256 = $bundleAssetSha256
        bundle_asset_file_count = $expected.Count
    }
}
