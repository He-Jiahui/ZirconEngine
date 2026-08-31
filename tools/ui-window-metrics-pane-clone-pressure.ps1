Set-StrictMode -Version Latest

function Get-ZirconUiPressureSha256 {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Text
    )

    $sha = [System.Security.Cryptography.SHA256]::Create()
    try {
        $bytes = [System.Text.Encoding]::UTF8.GetBytes($Text)
        return [Convert]::ToHexString($sha.ComputeHash($bytes))
    }
    finally {
        $sha.Dispose()
    }
}

function Export-ZirconUiWindowMetricsPaneClonePressure {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$RepoRoot,

        [Parameter(Mandatory = $true)]
        [string]$OutputDirectory,

        [ValidateRange(1, [long]::MaxValue)]
        [long]$FrameCount = 600,

        [ValidateRange(0, [int]::MaxValue)]
        [int]$FloatingWindowCount = 0,

        [ValidateRange(1, [long]::MaxValue)]
        [long]$EstimatedPanePayloadBytes = 1048576,

        [ValidateRange(0, [int]::MaxValue)]
        [int]$ExpectedSceneDockCloneSites = 4,

        [ValidateRange(0, [int]::MaxValue)]
        [int]$ExpectedGeometryApplyCloneSites = 4,

        [ValidateRange(0, [int]::MaxValue)]
        [int]$ExpectedFloatingWindowCloneSites = 2
    )

    $resolvedRepoRoot = [System.IO.Path]::GetFullPath($RepoRoot)
    $resolvedOutputDirectory = [System.IO.Path]::GetFullPath($OutputDirectory)
    $systemDrive = [System.IO.Path]::GetPathRoot(
        [Environment]::GetFolderPath([Environment+SpecialFolder]::Windows)
    )
    if ($resolvedOutputDirectory.StartsWith(
        $systemDrive,
        [System.StringComparison]::OrdinalIgnoreCase
    )) {
        throw "UI pressure evidence must be written outside the system drive ($systemDrive)."
    }

    $relativeSources = @(
        "zircon_editor/src/ui/retained_host/ui/apply_presentation/scene_conversion.rs",
        "zircon_editor/src/ui/retained_host/host_contract/data/host_root.rs"
    )
    $sources = [ordered]@{}
    foreach ($relativePath in $relativeSources) {
        $absolutePath = Join-Path $resolvedRepoRoot $relativePath
        if (-not (Test-Path -LiteralPath $absolutePath -PathType Leaf)) {
            throw "Required resize source is missing: $relativePath"
        }
        $sources[$relativePath] = Get-Content -LiteralPath $absolutePath -Raw
    }

    $scenePath = $relativeSources[0]
    $hostRootPath = $relativeSources[1]
    $dockNames = "left_dock|document_dock|right_dock|bottom_dock"
    $sceneDockCloneSites = [regex]::Matches(
        $sources[$scenePath],
        "current\.($dockNames)\.pane\.clone\(\)"
    ).Count
    $geometryApplyCloneSites = [regex]::Matches(
        $sources[$hostRootPath],
        "current\.host_scene_data\.($dockNames)\.pane\.clone\(\)"
    ).Count
    $floatingWindowCloneSiteCount = [regex]::Matches(
        $sources[$scenePath],
        "candidate\.active_pane\.clone\(\)"
    ).Count
    if (
        $sceneDockCloneSites -ne $ExpectedSceneDockCloneSites -or
        $geometryApplyCloneSites -ne $ExpectedGeometryApplyCloneSites -or
        $floatingWindowCloneSiteCount -ne $ExpectedFloatingWindowCloneSites
    ) {
        throw "WindowMetrics pane clone source contract drift: scene_conversion=$sceneDockCloneSites expected=$ExpectedSceneDockCloneSites; geometry_apply=$geometryApplyCloneSites expected=$ExpectedGeometryApplyCloneSites; floating=$floatingWindowCloneSiteCount expected=$ExpectedFloatingWindowCloneSites."
    }

    $manifestLines = foreach ($relativePath in $relativeSources | Sort-Object) {
        $sourceHash = Get-ZirconUiPressureSha256 -Text $sources[$relativePath]
        "$relativePath $sourceHash"
    }
    $manifestText = ($manifestLines -join "`n") + "`n"
    $manifestHash = Get-ZirconUiPressureSha256 -Text $manifestText

    $fixedDockCloneSites = [long]$sceneDockCloneSites + [long]$geometryApplyCloneSites
    $floatingClonesPerFrame = [long]$floatingWindowCloneSiteCount * $FloatingWindowCount
    $legacyClonesPerFrame = $fixedDockCloneSites + $floatingClonesPerFrame
    $legacyClonesTotal = $legacyClonesPerFrame * $FrameCount
    $legacyBytesTotal = $legacyClonesTotal * $EstimatedPanePayloadBytes

    $head = "unavailable"
    if (Test-Path -LiteralPath (Join-Path $resolvedRepoRoot ".git")) {
        $headOutput = & git -C $resolvedRepoRoot rev-parse HEAD 2>$null
        if ($LASTEXITCODE -eq 0 -and $headOutput) {
            $head = $headOutput.Trim()
        }
    }

    $report = [ordered]@{
        schema_version = 1
        source_binding = [ordered]@{
            repo_root = $resolvedRepoRoot
            head = $head
            manifest_sha256 = $manifestHash
            files = @($manifestLines)
        }
        scenario = [ordered]@{
            frame_count = $FrameCount
            floating_window_count = $FloatingWindowCount
            estimated_pane_payload_bytes = $EstimatedPanePayloadBytes
            estimate_kind = "explicit lower-bound model input; not a measured allocation size"
        }
        source_evidence = [ordered]@{
            fixed_dock_clone_sites = [ordered]@{
                scene_conversion = $sceneDockCloneSites
                geometry_apply = $geometryApplyCloneSites
                total = $fixedDockCloneSites
            }
            floating_window_clone_site_count = $floatingWindowCloneSiteCount
            floating_window_clone_formula = `
                "floating_window_clone_site_count * floating_window_count"
        }
        legacy_model = [ordered]@{
            semantic_pane_clones_per_frame = $legacyClonesPerFrame
            semantic_pane_clones_total = $legacyClonesTotal
            estimated_semantic_clone_bytes_total = $legacyBytesTotal
        }
        target_model = [ordered]@{
            invariant = "geometry publication retains semantic pane authority by shared identity"
            semantic_pane_clones_per_frame = 0
            semantic_pane_clones_total = 0
            estimated_semantic_clone_bytes_total = 0
        }
        modeled_reduction = [ordered]@{
            semantic_pane_clone_count = $legacyClonesTotal
            estimated_semantic_clone_bytes = $legacyBytesTotal
        }
    }

    New-Item -ItemType Directory -Path $resolvedOutputDirectory -Force | Out-Null
    $jsonPath = Join-Path $resolvedOutputDirectory `
        "ui-window-metrics-pane-clone-pressure.json"
    $report | ConvertTo-Json -Depth 8 |
        Set-Content -LiteralPath $jsonPath -Encoding UTF8

    [pscustomobject]@{
        schema_version = 1
        json_path = $jsonPath
        manifest_sha256 = $manifestHash
        semantic_pane_clones_per_frame = $legacyClonesPerFrame
        semantic_pane_clones_total = $legacyClonesTotal
    }
}
