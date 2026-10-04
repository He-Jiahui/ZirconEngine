[CmdletBinding()]
param(
    [string]$RepoRoot,
    [string]$OutputDirectory,
    [string[]]$SourceRoots = @(
        "zircon_runtime/src/ui",
        "zircon_runtime/src/graphics/scene/scene_renderer/ui",
        "zircon_runtime/crates/zr_rhi/src/ui_surface.rs",
        "zircon_runtime/crates/zr_rhi/src/ui_surface",
        "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface.rs",
        "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface",
        "zircon_runtime_interface/src/ui",
        "zircon_editor/src/ui"
    )
)

function Get-ZirconUiAuditSignalCount {
    param(
        [Parameter(Mandatory = $true)]
        [AllowEmptyString()]
        [string]$Source,
        [Parameter(Mandatory = $true)]
        [string]$Pattern
    )

    return [regex]::Matches(
        $Source,
        $Pattern,
        [System.Text.RegularExpressions.RegexOptions]::Multiline
    ).Count
}

function Get-ZirconUiAuditProductionSource {
    param(
        [Parameter(Mandatory = $true)]
        [AllowEmptyString()]
        [string]$Source
    )

    $testConfiguration = [regex]::Match(
        $Source,
        '^\s*#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*$',
        [System.Text.RegularExpressions.RegexOptions]::Multiline
    )
    if (-not $testConfiguration.Success) {
        return $Source
    }
    return $Source.Substring(0, $testConfiguration.Index)
}

function Get-ZirconUiAuditDomain {
    param(
        [Parameter(Mandatory = $true)]
        [string]$RelativePath
    )

    $path = $RelativePath.ToLowerInvariant()
    if ($path -match "/layout/") { return "layout" }
    if ($path -match "/surface/render/|/graphics/|/ui_surface(?:/|\.rs$)") { return "render" }
    if ($path -match "/surface/|pointer|input|focus|hit_test|navigation") { return "input_surface" }
    if ($path -match "retained_host") { return "editor_retained_host" }
    if ($path -match "/text/") { return "text" }
    if ($path -match "/template/|/binding/") { return "template_binding" }
    return "ui_other"
}

function Get-ZirconUiAuditGitBinding {
    param(
        [Parameter(Mandatory = $true)]
        [string]$ResolvedRepoRoot,
        [Parameter(Mandatory = $true)]
        [string[]]$SourceRoots
    )

    if (-not (Test-Path -LiteralPath (Join-Path $ResolvedRepoRoot ".git"))) {
        return [ordered]@{
            head_commit = $null
            dirty_path_count = $null
            dirty_paths = @()
        }
    }

    $headCommit = @(& git -C $ResolvedRepoRoot rev-parse HEAD 2>$null)[0]
    $dirtyPaths = @(& git -C $ResolvedRepoRoot status --porcelain --untracked-files=all -- @SourceRoots 2>$null)
    $normalizedDirtyPaths = @(
        $dirtyPaths |
            ForEach-Object {
                if ($_.Length -gt 3) {
                    $_.Substring(3).Trim().Replace("\", "/")
                }
            } |
            Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
    )
    return [ordered]@{
        head_commit = if ($headCommit) { $headCommit.Trim() } else { $null }
        dirty_path_count = $dirtyPaths.Count
        dirty_paths = $normalizedDirtyPaths
    }
}

function Export-ZirconUiStructuralHotspotAudit {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$RepoRoot,
        [Parameter(Mandatory = $true)]
        [string]$OutputDirectory,
        [string[]]$SourceRoots = @(
            "zircon_runtime/src/ui",
            "zircon_runtime/src/graphics/scene/scene_renderer/ui",
            "zircon_runtime/crates/zr_rhi/src/ui_surface.rs",
            "zircon_runtime/crates/zr_rhi/src/ui_surface",
            "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface.rs",
            "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface",
            "zircon_runtime_interface/src/ui",
            "zircon_editor/src/ui"
        )
    )

    $ErrorActionPreference = "Stop"

    $resolvedRepoRoot = (Resolve-Path -LiteralPath $RepoRoot -ErrorAction Stop).Path
    $resolvedOutputDirectory = [System.IO.Path]::GetFullPath($OutputDirectory)
    $outputDrive = [System.IO.Path]::GetPathRoot($resolvedOutputDirectory).TrimEnd("\")
    if ($outputDrive -notin @("D:", "E:", "F:")) {
        throw "UI structural audit artifacts must be written below D:, E:, or F:."
    }

    $filesByPath = [ordered]@{}
    foreach ($sourceRoot in $SourceRoots) {
        $absoluteSourceRoot = Join-Path $resolvedRepoRoot $sourceRoot
        $sourceFiles = if (Test-Path -LiteralPath $absoluteSourceRoot -PathType Leaf) {
            @(Get-Item -LiteralPath $absoluteSourceRoot)
        }
        elseif (Test-Path -LiteralPath $absoluteSourceRoot -PathType Container) {
            @(Get-ChildItem -LiteralPath $absoluteSourceRoot -Recurse -File -Filter "*.rs")
        }
        else {
            continue
        }
        foreach ($file in $sourceFiles) {
            if ($file.Extension -ne ".rs") {
                continue
            }
            $relativePath = $file.FullName.Substring($resolvedRepoRoot.Length).TrimStart("\", "/")
            $normalizedPath = $relativePath.Replace("\", "/")
            if (
                $normalizedPath -match "(^|/)(?:tests?|[^/]+_tests?)(/|$)" -or
                $file.Name -match "(^|_)tests?\.rs$"
            ) {
                continue
            }
            $filesByPath[$normalizedPath] = $file.FullName
        }
    }

    $gitBinding = Get-ZirconUiAuditGitBinding $resolvedRepoRoot $SourceRoots
    $dirtyPathSet = @{}
    foreach ($dirtyPath in @($gitBinding.dirty_paths)) {
        $dirtyPathSet[$dirtyPath] = $true
    }

    $sourcePaths = [string[]]@($filesByPath.Keys)
    [Array]::Sort($sourcePaths, [StringComparer]::Ordinal)
    $sourceManifestHasher = [Security.Cryptography.IncrementalHash]::CreateHash(
        [Security.Cryptography.HashAlgorithmName]::SHA256
    )
    $productionSourceByteCount = [int64]0
    $productionSourceManifestSha256 = $null
    try {
        $hotspots = foreach ($relativePath in $sourcePaths) {
            $source = Get-ZirconUiAuditProductionSource `
                (Get-Content -LiteralPath $filesByPath[$relativePath] -Raw)
            $sourceBytes = [Text.Encoding]::UTF8.GetBytes($source)
            $sourceHash = [Security.Cryptography.SHA256]::HashData($sourceBytes)
            $sourceManifestHasher.AppendData([Text.Encoding]::UTF8.GetBytes($relativePath))
            $sourceManifestHasher.AppendData([byte[]]@(0))
            $sourceManifestHasher.AppendData($sourceHash)
            $sourceManifestHasher.AppendData([byte[]]@(10))
            $productionSourceByteCount += $sourceBytes.Length

            $cloneCalls = Get-ZirconUiAuditSignalCount $source '\.clone\s*\(\s*\)'
            $vecMaterializations =
                (Get-ZirconUiAuditSignalCount $source '\.to_vec\s*\(\s*\)') +
                (Get-ZirconUiAuditSignalCount $source '\.collect\s*::\s*<\s*Vec\s*<') +
                (Get-ZirconUiAuditSignalCount $source '\bVec\s*::\s*(?:with_capacity|from|from_iter)\s*\(') +
                (Get-ZirconUiAuditSignalCount $source '\bvec!\s*\[(?!\s*\])')
            $sortCalls = Get-ZirconUiAuditSignalCount `
                $source `
                '\.(?:sort|sort_by|sort_by_key|sort_unstable|sort_unstable_by|sort_unstable_by_key)\s*\('
            $stringAllocations =
                (Get-ZirconUiAuditSignalCount $source '\.(?:to_string|to_owned)\s*\(') +
                (Get-ZirconUiAuditSignalCount $source '\bString\s*::\s*from\s*\(') +
                (Get-ZirconUiAuditSignalCount $source '\bformat!\s*\(')
            $traversalSignals = Get-ZirconUiAuditSignalCount `
                $source `
                '\.(?:iter|iter_mut|values|values_mut|keys)\s*\('
            $lineCount = if ($source.Length -eq 0) {
                0
            }
            else {
                [regex]::Matches($source, "\r?\n").Count + 1
            }
            $riskScore =
                ($cloneCalls * 3) +
                ($vecMaterializations * 5) +
                ($sortCalls * 8) +
                ($stringAllocations * 3) +
                $traversalSignals

            [pscustomobject][ordered]@{
                path = $relativePath
                crate = $relativePath.Split("/")[0]
                domain = Get-ZirconUiAuditDomain $relativePath
                dirty = $dirtyPathSet.ContainsKey($relativePath)
                line_count = $lineCount
                clone_calls = $cloneCalls
                vec_materializations = $vecMaterializations
                sort_calls = $sortCalls
                string_allocations = $stringAllocations
                traversal_signals = $traversalSignals
                risk_score = $riskScore
            }
        }
        $productionSourceManifestSha256 = [Convert]::ToHexString(
            $sourceManifestHasher.GetHashAndReset()
        )
    }
    finally {
        $sourceManifestHasher.Dispose()
    }
    $hotspots = @($hotspots | Sort-Object -Property @{ Expression = "risk_score"; Descending = $true }, path)

    $summary = [ordered]@{
        file_count = $hotspots.Count
        line_count = @($hotspots | Measure-Object -Property line_count -Sum).Sum
        clone_calls = @($hotspots | Measure-Object -Property clone_calls -Sum).Sum
        vec_materializations = @($hotspots | Measure-Object -Property vec_materializations -Sum).Sum
        sort_calls = @($hotspots | Measure-Object -Property sort_calls -Sum).Sum
        string_allocations = @($hotspots | Measure-Object -Property string_allocations -Sum).Sum
        traversal_signals = @($hotspots | Measure-Object -Property traversal_signals -Sum).Sum
        dirty_hotspots = @($hotspots | Where-Object { $_.dirty }).Count
    }
    $report = [ordered]@{
        schema_version = 1
        source_binding = [ordered]@{
            repo_root = $resolvedRepoRoot
            head_commit = $gitBinding.head_commit
            dirty_path_count = $gitBinding.dirty_path_count
            production_source_manifest_sha256 = $productionSourceManifestSha256
            production_source_byte_count = $productionSourceByteCount
            source_roots = @($SourceRoots)
            generated_at_utc = [DateTime]::UtcNow.ToString("o")
        }
        interpretation = [ordered]@{
            kind = "heuristic_source_signal_inventory"
            limitation = "Risk scores prioritize manual and runtime profiling; they are not CPU, allocation, or latency measurements."
            production_source_definition = "Source prefix before the first standalone #[cfg(test)] attribute, matching repository performance-contract convention."
            vec_materialization_definition = "Potential capacity allocation or element-copy expressions; allocation-free Vec::new() and empty vec![] defaults are excluded."
            weights = [ordered]@{
                clone_call = 3
                vec_materialization = 5
                sort_call = 8
                string_allocation = 3
                traversal_signal = 1
            }
        }
        summary = $summary
        hotspots = $hotspots
    }

    New-Item -ItemType Directory -Path $resolvedOutputDirectory -Force | Out-Null
    $jsonPath = Join-Path $resolvedOutputDirectory "ui-structural-hotspots.json"
    $csvPath = Join-Path $resolvedOutputDirectory "ui-structural-hotspots.csv"
    $report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $jsonPath -Encoding UTF8
    $hotspots | Export-Csv -LiteralPath $csvPath -NoTypeInformation -Encoding UTF8

    return [pscustomobject][ordered]@{
        schema_version = 1
        file_count = $hotspots.Count
        json_path = $jsonPath
        csv_path = $csvPath
    }
}

if ($MyInvocation.InvocationName -ne ".") {
    if ([string]::IsNullOrWhiteSpace($RepoRoot) -or [string]::IsNullOrWhiteSpace($OutputDirectory)) {
        throw "RepoRoot and OutputDirectory are required when invoking this script directly."
    }
    Export-ZirconUiStructuralHotspotAudit `
        -RepoRoot $RepoRoot `
        -OutputDirectory $OutputDirectory `
        -SourceRoots $SourceRoots
}
