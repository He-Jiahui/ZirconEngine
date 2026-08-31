$script:RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$script:ChromePaintMetrics =
    Join-Path $script:RepoRoot "tools\ui-profile-chrome-paint-metrics.ps1"

if (Test-Path -LiteralPath $script:ChromePaintMetrics) {
    . $script:ChromePaintMetrics
}

function Write-ChromePaintTimelineFixture {
    param(
        [Parameter(Mandatory = $true)]
        [string]$ProfileDir,
        [Parameter(Mandatory = $true)]
        [object[]]$Spans
    )

    New-Item -ItemType Directory -Force -Path $ProfileDir | Out-Null
    [ordered]@{
        counters = @()
        spans = $Spans
    } | ConvertTo-Json -Depth 6 |
        Set-Content -LiteralPath (Join-Path $ProfileDir "timeline.zrtrace.json") -Encoding UTF8
}

function New-ChromePaintSpan {
    param(
        [string]$Name,
        [double]$DurationUs,
        [string]$Stream = "editor",
        [string]$Category = "host_painter"
    )

    return [pscustomobject]@{
        stream = $Stream
        category = $Category
        name = $Name
        duration_us = $DurationUs
    }
}

Describe "UI Chrome paint metrics contract" {
    BeforeEach {
        $script:ProfileDir = Join-Path "E:\zircon-profiles" `
            ("chrome-paint-metrics-test-{0}" -f [guid]::NewGuid().ToString("N"))
    }

    AfterEach {
        if (Test-Path -LiteralPath $script:ProfileDir) {
            Remove-Item -LiteralPath $script:ProfileDir -Recurse -Force
        }
    }

    It "exports source-bound nearest-rank distributions for both Chrome command stages" {
        Get-Command Export-ZirconUiChromePaintMetrics -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty
        Write-ChromePaintTimelineFixture -ProfileDir $script:ProfileDir -Spans @(
            New-ChromePaintSpan "chrome_record_commands" 10
            New-ChromePaintSpan "chrome_record_commands" 20
            New-ChromePaintSpan "chrome_record_commands" 30
            New-ChromePaintSpan "chrome_record_commands" 40
            New-ChromePaintSpan "chrome_extract_commands" 4
            New-ChromePaintSpan "chrome_extract_commands" 8
            New-ChromePaintSpan "ignored_other_stream" 999 -Stream "runtime"
            New-ChromePaintSpan "chrome_record_commands" 777 -Category "other"
        )

        $path = Export-ZirconUiChromePaintMetrics -ProfileDir $script:ProfileDir
        $metrics = Get-Content -LiteralPath $path -Raw | ConvertFrom-Json
        $timelineHash = (Get-FileHash `
                -LiteralPath (Join-Path $script:ProfileDir "timeline.zrtrace.json") `
                -Algorithm SHA256).Hash.ToLowerInvariant()

        $metrics.schema_version | Should Be 1
        $metrics.source.timeline_file | Should Be "timeline.zrtrace.json"
        $metrics.source.timeline_sha256 | Should Be $timelineHash
        $metrics.percentile_policy | Should Be "nearest_rank"
        $metrics.span_duration_us.record_commands.sample_count | Should Be 4
        $metrics.span_duration_us.record_commands.p50 | Should Be 20
        $metrics.span_duration_us.record_commands.p95 | Should Be 40
        $metrics.span_duration_us.record_commands.p99 | Should Be 40
        $metrics.span_duration_us.record_commands.max | Should Be 40
        $metrics.span_duration_us.extract_commands.sample_count | Should Be 2
        $metrics.span_duration_us.extract_commands.p50 | Should Be 4
        $metrics.span_duration_us.extract_commands.p95 | Should Be 8
        Test-ZirconUiChromePaintMetricsGate -ProfileDir $script:ProfileDir |
            Should Be $true
    }

    It "does not manufacture zero samples and rejects missing extraction evidence" {
        Write-ChromePaintTimelineFixture -ProfileDir $script:ProfileDir -Spans @(
            New-ChromePaintSpan "chrome_record_commands" 25
        )

        $path = Export-ZirconUiChromePaintMetrics -ProfileDir $script:ProfileDir
        $metrics = Get-Content -LiteralPath $path -Raw | ConvertFrom-Json

        $metrics.span_duration_us.extract_commands.sample_count | Should Be 0
        $metrics.span_duration_us.extract_commands.p50 | Should BeNullOrEmpty
        Test-ZirconUiChromePaintMetricsGate -ProfileDir $script:ProfileDir |
            Should Be $false
    }

    It "rejects an artifact after the source timeline changes" {
        Write-ChromePaintTimelineFixture -ProfileDir $script:ProfileDir -Spans @(
            New-ChromePaintSpan "chrome_record_commands" 10
            New-ChromePaintSpan "chrome_extract_commands" 5
        )
        Export-ZirconUiChromePaintMetrics -ProfileDir $script:ProfileDir | Out-Null
        Add-Content -LiteralPath (Join-Path $script:ProfileDir "timeline.zrtrace.json") -Value " "

        Test-ZirconUiChromePaintMetricsGate -ProfileDir $script:ProfileDir |
            Should Be $false
    }

    It "is wired into the capture manifest export and required-evidence gate" {
        $captureSource = Get-Content `
            -LiteralPath (Join-Path $script:RepoRoot "tools\ui-profile-capture.ps1") `
            -Raw
        $manifestSource = Get-Content `
            -LiteralPath (Join-Path $script:RepoRoot "tools\profile-capture-manifest.ps1") `
            -Raw

        $captureSource | Should Match 'ui-profile-chrome-paint-metrics\.ps1'
        $captureSource | Should Match 'Export-ZirconUiChromePaintMetrics\s+-ProfileDir\s+\$ProfileDir'
        $captureSource | Should Match 'Test-ZirconUiChromePaintMetricsGate\s+-ProfileDir\s+\$ProfileDir'
        $captureSource | Should Match '\$chromePaintMetricsOk'
        $manifestSource | Should Match 'tools/ui-profile-chrome-paint-metrics\.ps1'
    }
}
