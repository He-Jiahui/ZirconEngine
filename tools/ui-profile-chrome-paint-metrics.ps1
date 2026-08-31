function Get-ZirconUiChromePaintMetricSummary {
    param([object[]]$Values)

    $orderedValues = @($Values |
            Where-Object { $null -ne $_ } |
            ForEach-Object { [double]$_ } |
            Where-Object {
                $_ -ge 0 -and
                -not [double]::IsNaN($_) -and
                -not [double]::IsInfinity($_)
            } |
            Sort-Object)
    if ($orderedValues.Count -eq 0) {
        return [pscustomobject][ordered]@{
            sample_count = 0
            p50 = $null
            p95 = $null
            p99 = $null
            max = $null
            sum = 0
            mean = $null
        }
    }

    $nearestRank = {
        param([double]$Percentile)

        $index = [Math]::Max(0, [Math]::Ceiling($orderedValues.Count * $Percentile) - 1)
        return $orderedValues[$index]
    }
    $measure = $orderedValues | Measure-Object -Sum -Average
    return [pscustomobject][ordered]@{
        sample_count = $orderedValues.Count
        p50 = & $nearestRank 0.50
        p95 = & $nearestRank 0.95
        p99 = & $nearestRank 0.99
        max = $orderedValues[$orderedValues.Count - 1]
        sum = $measure.Sum
        mean = $measure.Average
    }
}

function Get-ZirconUiChromePaintSpanSummary {
    param(
        [object[]]$Spans,
        [string]$Name
    )

    return Get-ZirconUiChromePaintMetricSummary -Values @($Spans |
            Where-Object {
                $_.stream -eq "editor" -and
                $_.category -eq "host_painter" -and
                $_.name -eq $Name
            } |
            ForEach-Object { $_.duration_us })
}

function Export-ZirconUiChromePaintMetrics {
    param([string]$ProfileDir)

    $timelinePath = Join-Path $ProfileDir "timeline.zrtrace.json"
    if (-not (Test-Path -LiteralPath $timelinePath -PathType Leaf)) {
        return $null
    }

    $timeline = Get-Content -LiteralPath $timelinePath -Raw | ConvertFrom-Json
    $timelineHash = (Get-FileHash -LiteralPath $timelinePath -Algorithm SHA256).Hash.ToLowerInvariant()
    $spans = @($timeline.spans)
    $metrics = [pscustomobject][ordered]@{
        schema_version = 1
        source = [pscustomobject][ordered]@{
            timeline_file = "timeline.zrtrace.json"
            timeline_sha256 = $timelineHash
        }
        percentile_policy = "nearest_rank"
        span_duration_us = [pscustomobject][ordered]@{
            record_commands = Get-ZirconUiChromePaintSpanSummary `
                -Spans $spans `
                -Name "chrome_record_commands"
            extract_commands = Get-ZirconUiChromePaintSpanSummary `
                -Spans $spans `
                -Name "chrome_extract_commands"
        }
    }
    $outputPath = Join-Path $ProfileDir "ui_chrome_paint_metrics.json"
    $metrics | ConvertTo-Json -Depth 6 |
        Set-Content -LiteralPath $outputPath -Encoding UTF8
    return $outputPath
}

function Test-ZirconUiChromePaintMetricSummary {
    param([object]$Summary)

    if ($null -eq $Summary -or [int64]$Summary.sample_count -le 0) {
        return $false
    }
    $p50 = [double]$Summary.p50
    $p95 = [double]$Summary.p95
    $p99 = [double]$Summary.p99
    $max = [double]$Summary.max
    if (@($p50, $p95, $p99, $max) | Where-Object {
            $_ -lt 0 -or [double]::IsNaN($_) -or [double]::IsInfinity($_)
        }) {
        return $false
    }
    return $p50 -le $p95 -and $p95 -le $p99 -and $p99 -le $max
}

function Test-ZirconUiChromePaintMetricsGate {
    param([string]$ProfileDir)

    $timelinePath = Join-Path $ProfileDir "timeline.zrtrace.json"
    $metricsPath = Join-Path $ProfileDir "ui_chrome_paint_metrics.json"
    if (-not (Test-Path -LiteralPath $timelinePath -PathType Leaf) -or
        -not (Test-Path -LiteralPath $metricsPath -PathType Leaf)) {
        Write-Warning "Chrome paint metrics require both timeline and derived metrics artifacts."
        return $false
    }

    try {
        $metrics = Get-Content -LiteralPath $metricsPath -Raw | ConvertFrom-Json
        $timelineHash = (Get-FileHash -LiteralPath $timelinePath -Algorithm SHA256).Hash.ToLowerInvariant()
    }
    catch {
        Write-Warning "Chrome paint metrics could not be parsed or source-bound."
        return $false
    }

    if ([int64]$metrics.schema_version -ne 1 -or
        $metrics.percentile_policy -ne "nearest_rank" -or
        $metrics.source.timeline_file -ne "timeline.zrtrace.json" -or
        $metrics.source.timeline_sha256 -ne $timelineHash) {
        Write-Warning "Chrome paint metrics do not match the current timeline source."
        return $false
    }
    if (-not (Test-ZirconUiChromePaintMetricSummary `
                -Summary $metrics.span_duration_us.record_commands) -or
        -not (Test-ZirconUiChromePaintMetricSummary `
                -Summary $metrics.span_duration_us.extract_commands)) {
        Write-Warning "Chrome paint metrics require ordered distributions for recording and extraction."
        return $false
    }
    return $true
}
