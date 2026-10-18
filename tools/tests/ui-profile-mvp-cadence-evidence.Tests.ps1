$script:CadenceEvidenceScript = Join-Path $PSScriptRoot '..\profiling\ui\ui-profile-mvp-cadence-evidence.ps1'
. $script:CadenceEvidenceScript

function Write-MvpCadenceFixture {
    param(
        [string]$Directory,
        [long[]]$Timestamps,
        [int]$CounterOverwritten = 0,
        [int]$Width = 1920,
        [int]$Height = 1080,
        [int[]]$RefreshRates = @(60),
        [int]$SelectableNodeCount = 1000
    )

    New-Item -ItemType Directory -Path $Directory -Force | Out-Null
    $counters = @($Timestamps | ForEach-Object {
            [ordered]@{
                stream = 'editor'
                name = 'ui.surface.submitted_count'
                value = 1
                timestamp_us = $_
            }
        })
    [ordered]@{
        counters = $counters
        spans = @()
        frames = @(
            [ordered]@{ stream = 'editor'; name = 'retained_host_tick'; duration_us = 15000 },
            [ordered]@{ stream = 'editor'; name = 'retained_host_tick'; duration_us = 20000 }
        )
        recorder_retention = @(
            [ordered]@{
                frames = [ordered]@{ capacity = 4096; written = 2; overwritten = 0; retained = 2; oldest_sequence = 0; newest_sequence = 1 }
                spans = [ordered]@{ capacity = 131072; written = 0; overwritten = 0; retained = 0; oldest_sequence = $null; newest_sequence = $null }
                counters = [ordered]@{
                    capacity = 32768
                    written = $counters.Count + $CounterOverwritten
                    overwritten = $CounterOverwritten
                    retained = $counters.Count
                    oldest_sequence = $CounterOverwritten
                    newest_sequence = $counters.Count + $CounterOverwritten - 1
                }
            }
        )
    } | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $Directory 'timeline.zrtrace.json') -Encoding UTF8
    [ordered]@{
        scenario = 'viewport_pointer'
        input_fixture = [ordered]@{ kind = 'viewport_pointer_scene'; selectable_node_count = $SelectableNodeCount }
        capture = [ordered]@{ options = [ordered]@{ run_phase = 'measured'; within_process_warmup = $true } }
    } | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $Directory 'source_manifest.json') -Encoding UTF8
    [ordered]@{
        gpu = [ordered]@{ status = 'captured'; data = @([ordered]@{ name = 'NVIDIA GeForce RTX 3060 Laptop GPU' }, [ordered]@{ name = 'Virtual Display Adapter' }) }
        display_modes = [ordered]@{
            status = 'captured'
            data = @($RefreshRates | ForEach-Object { [ordered]@{ name = 'NVIDIA GeForce RTX 3060 Laptop GPU'; width = 2560; height = 1600; refresh_rate_hz = $_ } })
        }
    } | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $Directory 'machine_manifest.json') -Encoding UTF8

    $png = [byte[]]@(137,80,78,71,13,10,26,10,0,0,0,13,73,72,68,82,
        (($Width -shr 24) -band 255), (($Width -shr 16) -band 255), (($Width -shr 8) -band 255), ($Width -band 255),
        (($Height -shr 24) -band 255), (($Height -shr 16) -band 255), (($Height -shr 8) -band 255), ($Height -band 255))
    [System.IO.File]::WriteAllBytes((Join-Path $Directory 'screenshot_gpu.png'), $png)
}

Describe 'MVP editor cadence evidence' {
    It 'uses successful submission intervals across at least 60 seconds and reports nearest-rank p95/p99' {
        $dir = Join-Path $TestDrive 'valid'
        $timestamps = @(0L)
        for ($i = 1; $i -le 3600; $i++) { $timestamps += [long]($i * 16667) }
        Write-MvpCadenceFixture -Directory $dir -Timestamps $timestamps

        $report = Get-ZirconMvpCadenceEvidence -ProfileDir $dir
        $report.submit_cadence.status | Should Be 'pass'
        $report.submit_cadence.window_seconds | Should BeGreaterThan 60
        $report.submit_cadence.interval_count | Should Be 3600
        $report.submit_cadence.p95_us | Should Be 16667
        $report.submit_cadence.p99_us | Should Be 16667
        $report.editor_tick_cpu.p95_us | Should Be 20000
        $report.editor_tick_cpu.p99_us | Should Be 20000
        $report.editor_tick_cpu.metric | Should Be 'retained_host_tick_cpu_duration'
        $report.window_client.width | Should Be 1920
        $report.display.refresh_rate_hz | Should Be 60
        $report.adapter.actual_wgpu_adapter_status | Should Be 'unavailable'
        $report.product_acceptance | Should Be 'pending'
        $report.submit_cadence.metric | Should Be 'successful_surface_submit_interval'
    }

    It 'rejects a window shorter than 60 seconds even when the requested auto close is longer' {
        $dir = Join-Path $TestDrive 'short'
        Write-MvpCadenceFixture -Directory $dir -Timestamps @(0L, 16667L, 59999999L)
        $report = Get-ZirconMvpCadenceEvidence -ProfileDir $dir
        $report.submit_cadence.status | Should Be 'insufficient'
        $report.product_acceptance | Should Be 'pending'
    }

    It 'rejects overwritten samples instead of treating a truncated trace as a complete window' {
        $dir = Join-Path $TestDrive 'overwrite'
        Write-MvpCadenceFixture -Directory $dir -Timestamps @(0L, 30000000L, 60000000L) -CounterOverwritten 1
        $report = Get-ZirconMvpCadenceEvidence -ProfileDir $dir
        $report.submit_cadence.status | Should Be 'invalid'
    }

    It 'rejects contradictory recorder counts instead of trusting incomplete counters' {
        $dir = Join-Path $TestDrive 'contradictory'
        Write-MvpCadenceFixture -Directory $dir -Timestamps @(0L, 30000000L, 60000000L)
        $timelinePath = Join-Path $dir 'timeline.zrtrace.json'
        $timeline = Get-Content -LiteralPath $timelinePath -Raw | ConvertFrom-Json
        $timeline.recorder_retention[0].counters.retained = 2
        $timeline | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $timelinePath -Encoding UTF8
        $report = Get-ZirconMvpCadenceEvidence -ProfileDir $dir
        $report.submit_cadence.status | Should Be 'invalid'
    }

    It 'reports 1080p and refresh mismatches without upgrading product acceptance' {
        $dir = Join-Path $TestDrive 'mismatch'
        Write-MvpCadenceFixture -Directory $dir -Timestamps @(0L, 30000000L, 60000000L) -Width 1600 -RefreshRates @(60, 144)
        $report = Get-ZirconMvpCadenceEvidence -ProfileDir $dir
        $report.window_client.status | Should Be 'mismatch'
        $report.display.status | Should Be 'ambiguous'
        $report.adapter.actual_wgpu_adapter_status | Should Be 'unavailable'
        $report.product_acceptance | Should Be 'fail'
    }

    It 'calculates nearest-rank tails and rejects a slow p95' {
        $dir = Join-Path $TestDrive 'tail'
        Write-MvpCadenceFixture -Directory $dir -Timestamps @(0L, 10000000L, 20000000L, 30000000L, 60000000L)
        $report = Get-ZirconMvpCadenceEvidence -ProfileDir $dir
        $report.submit_cadence.p95_us | Should Be 30000000
        $report.submit_cadence.p99_us | Should Be 30000000
        $report.submit_cadence.status | Should Be 'fail'
        $report.product_acceptance | Should Be 'fail'
    }

    It 'rejects a low overall submit rate hidden by passing p95 and p99 intervals' {
        $dir = Join-Path $TestDrive 'low-rate'
        $timestamps = @(0L)
        for ($i = 1; $i -le 3000; $i++) { $timestamps += [long]($i * 16000) }
        $timestamps += 60000000L
        Write-MvpCadenceFixture -Directory $dir -Timestamps $timestamps
        $report = Get-ZirconMvpCadenceEvidence -ProfileDir $dir
        $report.submit_cadence.p95_us | Should Be 16000
        $report.submit_cadence.p99_us | Should Be 16000
        $report.submit_cadence.observed_submit_rate_hz | Should BeLessThan 59.9
        $report.submit_cadence.status | Should Be 'fail'
        $report.product_acceptance | Should Be 'fail'
    }

    It 'writes a standalone JSON report without claiming display latency' {
        $dir = Join-Path $TestDrive 'output'
        $output = Join-Path $dir 'evidence.json'
        Write-MvpCadenceFixture -Directory $dir -Timestamps @(0L, 30000000L, 60000000L)
        $null = & powershell.exe -NoProfile -File $script:CadenceEvidenceScript -ProfileDir $dir -OutputPath $output
        $LASTEXITCODE | Should Be 0
        $saved = Get-Content -LiteralPath $output -Raw | ConvertFrom-Json
        $saved.submit_cadence.metric | Should Be 'successful_surface_submit_interval'
        $saved.display_scanout_status | Should Be 'unavailable'
        $saved.product_acceptance | Should Be 'fail'
    }
}
