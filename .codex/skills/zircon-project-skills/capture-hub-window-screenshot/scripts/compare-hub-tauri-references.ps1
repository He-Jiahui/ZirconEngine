[CmdletBinding()]
param(
    [string]$RepoRoot = "",
    [string]$ProjectCaptureDir = "",
    [string]$StateCaptureDir = "",
    [string]$ReferenceDir = "",
    [string]$OutputDir = "",
    [int]$SampleStep = 4,
    [int]$ChangedTolerance = 32,
    [int]$MinimumActualWidth = 1400,
    [int]$MinimumActualHeight = 900,
    [double]$MaxMeanDelta = 35.0,
    [double]$MaxRmsDelta = 75.0,
    [switch]$FailOnSimilarityWarning
)

$ErrorActionPreference = "Stop"

function Resolve-AbsolutePath {
    param(
        [string]$Base,
        [string]$Path
    )

    if ([System.IO.Path]::IsPathRooted($Path)) {
        return [System.IO.Path]::GetFullPath($Path)
    }

    return [System.IO.Path]::GetFullPath((Join-Path $Base $Path))
}

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..\..")
} else {
    $RepoRoot = Resolve-Path $RepoRoot
}
$RepoRoot = [string]$RepoRoot

if ([string]::IsNullOrWhiteSpace($ProjectCaptureDir)) {
    $ProjectCaptureDir = "target\hub-visual-check\tauri-project-pages-full-matrix"
}
if ([string]::IsNullOrWhiteSpace($StateCaptureDir)) {
    $StateCaptureDir = "target\hub-visual-check\tauri-visual-state-matrix"
}
if ([string]::IsNullOrWhiteSpace($ReferenceDir)) {
    $ReferenceDir = "docs\ui-and-layout"
}
if ([string]::IsNullOrWhiteSpace($OutputDir)) {
    $OutputDir = "target\hub-visual-check\tauri-reference-comparison"
}

$ProjectCaptureDir = Resolve-AbsolutePath -Base $RepoRoot -Path $ProjectCaptureDir
$StateCaptureDir = Resolve-AbsolutePath -Base $RepoRoot -Path $StateCaptureDir
$ReferenceDir = Resolve-AbsolutePath -Base $RepoRoot -Path $ReferenceDir
$OutputDir = Resolve-AbsolutePath -Base $RepoRoot -Path $OutputDir
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

Add-Type -AssemblyName System.Drawing

$comparisonSpecs = @(
    [pscustomobject]@{ Id = "projects-dashboard"; CaptureSet = "project"; Actual = "hub-projects-dashboard.png"; Reference = "hub.png"; AiDraft = "" },
    [pscustomobject]@{ Id = "hub-editor"; CaptureSet = "state"; Actual = "hub-state-editor.png"; Reference = "hub-editor.png"; AiDraft = "hub-editor.png" },
    [pscustomobject]@{ Id = "hub-builds"; CaptureSet = "state"; Actual = "hub-state-builds.png"; Reference = "hub-builds.png"; AiDraft = "hub-builds.png" },
    [pscustomobject]@{ Id = "hub-assets"; CaptureSet = "state"; Actual = "hub-state-assets.png"; Reference = "hub-assets.png"; AiDraft = "hub-assets.png" },
    [pscustomobject]@{ Id = "hub-plugins"; CaptureSet = "state"; Actual = "hub-state-plugins.png"; Reference = "hub-plugins.png"; AiDraft = "hub-plugins.png" },
    [pscustomobject]@{ Id = "hub-cloud"; CaptureSet = "state"; Actual = "hub-state-cloud.png"; Reference = "hub-cloud.png"; AiDraft = "hub-cloud.png" },
    [pscustomobject]@{ Id = "hub-team"; CaptureSet = "state"; Actual = "hub-state-team.png"; Reference = "hub-team.png"; AiDraft = "hub-team.png" },
    [pscustomobject]@{ Id = "hub-learn"; CaptureSet = "state"; Actual = "hub-state-learn.png"; Reference = "hub-learn.png"; AiDraft = "hub-learn.png" },
    [pscustomobject]@{ Id = "hub-projects-new"; CaptureSet = "project"; Actual = "hub-projects-new-project.png"; Reference = "hub-projects-new.png"; AiDraft = "hub-projects-new.png" },
    [pscustomobject]@{ Id = "hub-projects-browser"; CaptureSet = "project"; Actual = "hub-projects-browser.png"; Reference = "hub-projects-browser.png"; AiDraft = "hub-projects-browser.png" },
    [pscustomobject]@{ Id = "hub-projects-detail"; CaptureSet = "project"; Actual = "hub-projects-detail.png"; Reference = "hub-projects-detail.png"; AiDraft = "hub-projects-detail.png" },
    [pscustomobject]@{ Id = "hub-projects-browser-filter-menu"; CaptureSet = "project"; Actual = "hub-projects-browser-filter-menu.png"; Reference = "hub-projects-browser-filter-menu.png"; AiDraft = "hub-projects-browser-filter-menu.png" },
    [pscustomobject]@{ Id = "hub-projects-browser-sort-menu"; CaptureSet = "project"; Actual = "hub-projects-browser-sort-menu.png"; Reference = "hub-projects-browser-sort-menu.png"; AiDraft = "hub-projects-browser-sort-menu.png" },
    [pscustomobject]@{ Id = "hub-projects-detail-delete-confirm"; CaptureSet = "project"; Actual = "hub-projects-detail-delete-confirm.png"; Reference = "hub-projects-detail-delete-confirm.png"; AiDraft = "hub-projects-detail-delete-confirm.png" },
    [pscustomobject]@{ Id = "hub-settings"; CaptureSet = "state"; Actual = "hub-state-settings.png"; Reference = "hub-settings.png"; AiDraft = "hub-settings.png" },
    [pscustomobject]@{ Id = "hub-source-engine-popup"; CaptureSet = "state"; Actual = "hub-state-source-engine-popup.png"; Reference = "hub-source-engine-popup.png"; AiDraft = "hub-source-engine-popup.png" },
    [pscustomobject]@{ Id = "hub-user-menu"; CaptureSet = "state"; Actual = "hub-state-user-menu.png"; Reference = "hub-user-menu.png"; AiDraft = "hub-user-menu.png" },
    [pscustomobject]@{ Id = "hub-state-empty"; CaptureSet = "state"; Actual = "hub-state-project-browser-empty.png"; Reference = "hub-state-empty.png"; AiDraft = "hub-state-empty.png" },
    [pscustomobject]@{ Id = "hub-state-loading"; CaptureSet = "state"; Actual = "hub-state-loading.png"; Reference = "hub-state-loading.png"; AiDraft = "hub-state-loading.png" },
    [pscustomobject]@{ Id = "hub-state-error"; CaptureSet = "state"; Actual = "hub-state-error.png"; Reference = "hub-state-error.png"; AiDraft = "hub-state-error.png" }
)

function Get-CapturePath {
    param([object]$Spec)

    if ($Spec.CaptureSet -eq "project") {
        return Join-Path $ProjectCaptureDir $Spec.Actual
    }

    return Join-Path $StateCaptureDir $Spec.Actual
}

function Get-ScaledPixel {
    param(
        [System.Drawing.Bitmap]$Bitmap,
        [int]$X,
        [int]$Y,
        [int]$TargetWidth,
        [int]$TargetHeight
    )

    $sourceX = [int][Math]::Floor((([double]$X + 0.5) * [double]$Bitmap.Width / [double]$TargetWidth))
    $sourceY = [int][Math]::Floor((([double]$Y + 0.5) * [double]$Bitmap.Height / [double]$TargetHeight))
    $sourceX = [Math]::Max(0, [Math]::Min($Bitmap.Width - 1, $sourceX))
    $sourceY = [Math]::Max(0, [Math]::Min($Bitmap.Height - 1, $sourceY))

    return $Bitmap.GetPixel($sourceX, $sourceY)
}

function Measure-ImageHealth {
    param(
        [System.Drawing.Bitmap]$Bitmap,
        [int]$Stride = 8
    )

    $min = @(255, 255, 255)
    $max = @(0, 0, 0)
    $white = 0
    $accent = 0
    $total = 0

    for ($y = 0; $y -lt $Bitmap.Height; $y += $Stride) {
        for ($x = 0; $x -lt $Bitmap.Width; $x += $Stride) {
            $pixel = $Bitmap.GetPixel($x, $y)
            $channels = @([int]$pixel.R, [int]$pixel.G, [int]$pixel.B)
            for ($channel = 0; $channel -lt 3; $channel += 1) {
                $min[$channel] = [Math]::Min($min[$channel], $channels[$channel])
                $max[$channel] = [Math]::Max($max[$channel], $channels[$channel])
            }
            if ($pixel.R -gt 245 -and $pixel.G -gt 245 -and $pixel.B -gt 245) {
                $white += 1
            }
            $isTeal = $pixel.G -gt 115 -and $pixel.B -gt 100 -and $pixel.R -lt 90
            $isWarning = $pixel.R -gt 150 -and $pixel.G -gt 95 -and $pixel.B -lt 80
            $isError = $pixel.R -gt 140 -and $pixel.G -lt 95 -and $pixel.B -lt 95
            if ($isTeal -or $isWarning -or $isError) {
                $accent += 1
            }
            $total += 1
        }
    }

    $dynamicRange = [Math]::Max($max[0] - $min[0], [Math]::Max($max[1] - $min[1], $max[2] - $min[2]))
    return [pscustomobject]@{
        DynamicRange = $dynamicRange
        WhiteRatio = if ($total -gt 0) { [double]$white / [double]$total } else { 0.0 }
        AccentRatio = if ($total -gt 0) { [double]$accent / [double]$total } else { 0.0 }
    }
}

function Compare-Images {
    param(
        [System.Drawing.Bitmap]$Reference,
        [System.Drawing.Bitmap]$Actual,
        [int]$Stride,
        [int]$Tolerance
    )

    $sum = 0.0
    $squared = 0.0
    $changed = 0
    $total = 0
    $channelCount = 0

    for ($y = 0; $y -lt $Reference.Height; $y += $Stride) {
        for ($x = 0; $x -lt $Reference.Width; $x += $Stride) {
            $referencePixel = $Reference.GetPixel($x, $y)
            $actualPixel = Get-ScaledPixel -Bitmap $Actual -X $x -Y $y -TargetWidth $Reference.Width -TargetHeight $Reference.Height
            $pixelDelta = 0
            foreach ($pair in @(
                @([int]$referencePixel.R, [int]$actualPixel.R),
                @([int]$referencePixel.G, [int]$actualPixel.G),
                @([int]$referencePixel.B, [int]$actualPixel.B)
            )) {
                $delta = [Math]::Abs($pair[0] - $pair[1])
                $sum += $delta
                $squared += $delta * $delta
                $pixelDelta += $delta
                $channelCount += 1
            }
            if ($pixelDelta -gt $Tolerance) {
                $changed += 1
            }
            $total += 1
        }
    }

    return [pscustomobject]@{
        MeanDelta = if ($channelCount -gt 0) { $sum / [double]$channelCount } else { 0.0 }
        RmsDelta = if ($channelCount -gt 0) { [Math]::Sqrt($squared / [double]$channelCount) } else { 0.0 }
        ChangedRatio = if ($total -gt 0) { [double]$changed / [double]$total } else { 0.0 }
        SampledPixels = $total
    }
}

$results = New-Object System.Collections.Generic.List[object]
$fatalIssues = New-Object System.Collections.Generic.List[string]
$similarityWarnings = New-Object System.Collections.Generic.List[string]

foreach ($spec in $comparisonSpecs) {
    $actualPath = Get-CapturePath -Spec $spec
    $referencePath = Join-Path $ReferenceDir $spec.Reference
    $aiDraftPath = if ([string]::IsNullOrWhiteSpace($spec.AiDraft)) { "" } else { Join-Path (Join-Path $ReferenceDir "hub-ai-drafts") $spec.AiDraft }

    if (-not (Test-Path -LiteralPath $actualPath)) {
        $fatalIssues.Add("Missing actual Tauri capture for $($spec.Id): $actualPath")
        continue
    }
    if (-not (Test-Path -LiteralPath $referencePath)) {
        $fatalIssues.Add("Missing final reference PNG for $($spec.Id): $referencePath")
        continue
    }

    $actual = New-Object System.Drawing.Bitmap($actualPath)
    $reference = New-Object System.Drawing.Bitmap($referencePath)
    try {
        $actualHealth = Measure-ImageHealth -Bitmap $actual
        $referenceHealth = Measure-ImageHealth -Bitmap $reference
        $metrics = Compare-Images -Reference $reference -Actual $actual -Stride $SampleStep -Tolerance $ChangedTolerance

        $status = "ok"
        $notes = New-Object System.Collections.Generic.List[string]
        if ($actual.Width -lt $MinimumActualWidth -or $actual.Height -lt $MinimumActualHeight) {
            $status = "fail"
            $notes.Add("actual capture is too small for a Hub window")
            $fatalIssues.Add("$($spec.Id) actual capture size is $($actual.Width)x$($actual.Height), below ${MinimumActualWidth}x${MinimumActualHeight}")
        }
        if ($actualHealth.WhiteRatio -gt 0.92) {
            $status = "fail"
            $notes.Add("actual capture is mostly white")
            $fatalIssues.Add("$($spec.Id) actual capture is mostly white")
        }
        if ($actualHealth.DynamicRange -lt 20) {
            $status = "fail"
            $notes.Add("actual capture has low dynamic range")
            $fatalIssues.Add("$($spec.Id) actual capture has low dynamic range")
        }
        if ($metrics.MeanDelta -gt $MaxMeanDelta -or $metrics.RmsDelta -gt $MaxRmsDelta) {
            if ($status -ne "fail") {
                $status = "warn"
            }
            $warning = "$($spec.Id) similarity warning: mean=$([Math]::Round($metrics.MeanDelta, 2)) rms=$([Math]::Round($metrics.RmsDelta, 2))"
            $notes.Add($warning)
            $similarityWarnings.Add($warning)
        }
        if (-not [string]::IsNullOrWhiteSpace($aiDraftPath) -and -not (Test-Path -LiteralPath $aiDraftPath)) {
            $status = "fail"
            $notes.Add("AI draft reference is missing")
            $fatalIssues.Add("Missing AI draft reference for $($spec.Id): $aiDraftPath")
        }

        $results.Add([pscustomobject]@{
            id = $spec.Id
            status = $status
            actual = $actualPath
            reference = $referencePath
            ai_draft = $aiDraftPath
            actual_width = $actual.Width
            actual_height = $actual.Height
            reference_width = $reference.Width
            reference_height = $reference.Height
            sampled_pixels = $metrics.SampledPixels
            mean_delta = [Math]::Round($metrics.MeanDelta, 4)
            rms_delta = [Math]::Round($metrics.RmsDelta, 4)
            changed_ratio = [Math]::Round($metrics.ChangedRatio, 6)
            actual_dynamic_range = $actualHealth.DynamicRange
            actual_white_ratio = [Math]::Round($actualHealth.WhiteRatio, 6)
            actual_accent_ratio = [Math]::Round($actualHealth.AccentRatio, 6)
            reference_dynamic_range = $referenceHealth.DynamicRange
            notes = @($notes.ToArray())
        }) | Out-Null
    } finally {
        $actual.Dispose()
        $reference.Dispose()
    }
}

$resultArray = @($results.ToArray())
$summary = [pscustomobject]@{
    generated_at = (Get-Date).ToString("s")
    project_capture_dir = $ProjectCaptureDir
    state_capture_dir = $StateCaptureDir
    reference_dir = $ReferenceDir
    sample_step = $SampleStep
    changed_tolerance = $ChangedTolerance
    max_mean_delta = $MaxMeanDelta
    max_rms_delta = $MaxRmsDelta
    comparisons = $resultArray.Count
    failures = @($resultArray | Where-Object { $_.status -eq "fail" }).Count
    warnings = @($resultArray | Where-Object { $_.status -eq "warn" }).Count
    results = $resultArray
}

$jsonPath = Join-Path $OutputDir "hub-tauri-reference-comparison.json"
$markdownPath = Join-Path $OutputDir "hub-tauri-reference-comparison.md"
$summary | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $jsonPath -Encoding UTF8

$markdown = New-Object System.Collections.Generic.List[string]
$markdown.Add("# Hub Tauri Reference Comparison") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add("Generated: $($summary.generated_at)") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add("- Project captures: ``$ProjectCaptureDir``") | Out-Null
$markdown.Add("- State captures: ``$StateCaptureDir``") | Out-Null
$markdown.Add("- Final references: ``$ReferenceDir``") | Out-Null
$markdown.Add("- Sample step: ``$SampleStep``; changed tolerance: ``$ChangedTolerance``") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add("| Id | Status | Actual size | Reference size | Mean | RMS | Changed | Accent | Notes |") | Out-Null
$markdown.Add("| --- | --- | --- | --- | ---: | ---: | ---: | ---: | --- |") | Out-Null
foreach ($result in $resultArray) {
    $notes = if ($result.notes.Count -gt 0) { ($result.notes -join "; ") } else { "" }
    $changedPercent = "{0:P2}" -f $result.changed_ratio
    $accentPercent = "{0:P2}" -f $result.actual_accent_ratio
    $markdown.Add("| ``$($result.id)`` | $($result.status) | $($result.actual_width)x$($result.actual_height) | $($result.reference_width)x$($result.reference_height) | $($result.mean_delta) | $($result.rms_delta) | $changedPercent | $accentPercent | $notes |") | Out-Null
}
$markdown.Add("") | Out-Null
$markdown.Add("AI draft PNGs are checked for inventory presence where the manifest defines them. Final similarity metrics compare against the HTML/CSS-finalized ``docs/ui-and-layout`` PNG references, not the AI drafts.") | Out-Null
$markdown | Set-Content -LiteralPath $markdownPath -Encoding UTF8

Write-Host "Comparison JSON: $jsonPath"
Write-Host "Comparison report: $markdownPath"
Write-Host "Comparisons: $($summary.comparisons), failures: $($summary.failures), warnings: $($summary.warnings)"

if ($fatalIssues.Count -gt 0) {
    throw "Hub Tauri reference comparison found fatal issues: $($fatalIssues -join '; ')"
}
if ($FailOnSimilarityWarning -and $similarityWarnings.Count -gt 0) {
    throw "Hub Tauri reference comparison found similarity warnings: $($similarityWarnings -join '; ')"
}
