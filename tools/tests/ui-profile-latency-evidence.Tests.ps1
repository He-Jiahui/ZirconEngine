$script:LatencyEvidenceModule = Join-Path $PSScriptRoot '..\ui-profile-latency-evidence.ps1'
. $script:LatencyEvidenceModule

function New-ValidLatencyArtifact {
    return [ordered]@{
        schema_version = 5
        retention_source_count = 1
        retention_complete = $true
        frame_overwritten = 0
        span_overwritten = 0
        counter_overwritten = 0
        input_outcome_count = 4
        damaged_input_outcome_count = 2
        intentionally_no_damage_input_outcome_count = 1
        rejected_input_outcome_count = 1
        present_batch_count = 1
        present_batch_damaged_count = 2
        typed_input_outcome_complete = $true
        input_to_damage_sample_count = 2
        input_to_damage_p50_us = 200
        input_to_damage_p95_us = 800
        input_to_damage_p99_us = 900
        input_to_damage_max_us = 950
        damage_to_submit_sample_count = 1
        damage_to_submit_p50_us = 2000
        damage_to_submit_p95_us = 7000
        damage_to_submit_p99_us = 7500
        damage_to_submit_max_us = 7900
    }
}

function Write-LatencyArtifact {
    param(
        [string]$ProfileDir,
        [object]$Artifact
    )

    New-Item -ItemType Directory -Force -Path $ProfileDir | Out-Null
    $Artifact | ConvertTo-Json -Depth 6 |
        Set-Content -LiteralPath (Join-Path $ProfileDir 'ui_surface_present_outcomes.json') -Encoding UTF8
}

function Test-ClickLatencyArtifact {
    param([string]$ProfileDir)

    return Test-ZirconUiSurfaceLatencyEvidenceGate `
        -ProfileDir $ProfileDir `
        -ScenarioName 'material_lab_click' `
        -InteractionScenarioName 'click' `
        -AutoClickCount 1000 `
        -AutoPointerMoveCount 0 `
        -AutoWheelCount 0
}

Describe 'ui profile latency evidence' {
    It 'exports schema 5 typed input outcomes, present batches, and recorder retention evidence' {
        $profileDir = Join-Path $TestDrive 'export'
        New-Item -ItemType Directory -Force -Path $profileDir | Out-Null
        [pscustomobject]@{
            recorder_retention = @(
                [pscustomobject]@{
                    frames = [pscustomobject]@{
                        capacity = 8; written = 4; overwritten = 0; retained = 4
                        oldest_sequence = 0; newest_sequence = 3
                    }
                    spans = [pscustomobject]@{
                        capacity = 16; written = 0; overwritten = 0; retained = 0
                        oldest_sequence = $null; newest_sequence = $null
                    }
                    counters = [pscustomobject]@{
                        capacity = 32; written = 11; overwritten = 0; retained = 11
                        oldest_sequence = 0; newest_sequence = 10
                    }
                }
            )
            counters = @(
                [pscustomobject]@{ name = 'ui.surface.submitted_count'; value = 2; timestamp_us = 100 },
                [pscustomobject]@{ name = 'ui.input.outcome.damaged_sequence'; value = 1; timestamp_us = 110 },
                [pscustomobject]@{ name = 'ui.click.input_to_damage_us'; value = 200; timestamp_us = 110 },
                [pscustomobject]@{ name = 'ui.input.outcome.intentionally_no_damage_sequence'; value = 2; timestamp_us = 111 },
                [pscustomobject]@{ name = 'ui.input.outcome.damaged_sequence'; value = 3; timestamp_us = 112 },
                [pscustomobject]@{ name = 'ui.click.input_to_damage_us'; value = 800; timestamp_us = 112 },
                [pscustomobject]@{ name = 'ui.input.outcome.rejected_sequence'; value = 4; timestamp_us = 113 },
                [pscustomobject]@{ name = 'ui.input.present_batch.first_sequence'; value = 1; timestamp_us = 120 },
                [pscustomobject]@{ name = 'ui.input.present_batch.last_sequence'; value = 3; timestamp_us = 120 },
                [pscustomobject]@{ name = 'ui.input.present_batch.damaged_count'; value = 2; timestamp_us = 120 },
                [pscustomobject]@{ name = 'ui.click.damage_to_submit_us'; value = 7000; timestamp_us = 120 }
            )
        } | ConvertTo-Json -Depth 6 |
            Set-Content -LiteralPath (Join-Path $profileDir 'timeline.zrtrace.json') -Encoding UTF8

        Export-ZirconUiSurfacePresentOutcomeEvidence -ProfileDir $profileDir

        $artifact = Get-Content -LiteralPath (Join-Path $profileDir 'ui_surface_present_outcomes.json') -Raw |
            ConvertFrom-Json
        $artifact.schema_version | Should Be 5
        $artifact.retention_source_count | Should Be 1
        $artifact.retention_complete | Should Be $true
        $artifact.frame_written | Should Be 4
        $artifact.counter_written | Should Be 11
        $artifact.counter_overwritten | Should Be 0
        $artifact.input_to_damage_p95_us | Should Be 800
        $artifact.damage_to_submit_p95_us | Should Be 7000
        $artifact.input_outcome_count | Should Be 4
        $artifact.damaged_input_outcome_count | Should Be 2
        $artifact.intentionally_no_damage_input_outcome_count | Should Be 1
        $artifact.rejected_input_outcome_count | Should Be 1
        $artifact.present_batch_count | Should Be 1
        $artifact.present_batch_damaged_count | Should Be 2
        $artifact.typed_input_outcome_complete | Should Be $true
    }

    It 'rejects latency artifacts without recorder retention evidence' {
        $profileDir = Join-Path $TestDrive 'missing-retention'
        $artifact = New-ValidLatencyArtifact
        $artifact.schema_version = 3
        $artifact.Remove('retention_source_count')
        $artifact.Remove('retention_complete')
        Write-LatencyArtifact -ProfileDir $profileDir -Artifact $artifact

        (Test-ClickLatencyArtifact -ProfileDir $profileDir) | Should Be $false
    }

    It 'rejects latency artifacts without complete typed input outcomes' {
        $profileDir = Join-Path $TestDrive 'missing-typed-outcomes'
        $artifact = New-ValidLatencyArtifact
        $artifact.typed_input_outcome_complete = $false
        Write-LatencyArtifact -ProfileDir $profileDir -Artifact $artifact

        (Test-ClickLatencyArtifact -ProfileDir $profileDir) | Should Be $false
    }

    It 'rejects a present batch whose damaged membership count is false' {
        $evidence = Get-ZirconTypedInputOutcomeEvidence -Snapshot ([pscustomobject]@{
                counters = @(
                    [pscustomobject]@{ name = 'ui.input.outcome.damaged_sequence'; value = 7; timestamp_us = 10 },
                    [pscustomobject]@{ name = 'ui.click.input_to_damage_us'; value = 200; timestamp_us = 10 },
                    [pscustomobject]@{ name = 'ui.input.present_batch.first_sequence'; value = 7; timestamp_us = 20 },
                    [pscustomobject]@{ name = 'ui.input.present_batch.last_sequence'; value = 7; timestamp_us = 20 },
                    [pscustomobject]@{ name = 'ui.input.present_batch.damaged_count'; value = 2; timestamp_us = 20 },
                    [pscustomobject]@{ name = 'ui.click.damage_to_submit_us'; value = 500; timestamp_us = 20 }
                )
            })

        $evidence.typed_input_outcome_complete | Should Be $false
    }

    It 'rejects latency artifacts after any recorder overwrite' {
        $profileDir = Join-Path $TestDrive 'overwritten'
        $artifact = New-ValidLatencyArtifact
        $artifact.counter_overwritten = 1
        Write-LatencyArtifact -ProfileDir $profileDir -Artifact $artifact

        (Test-ClickLatencyArtifact -ProfileDir $profileDir) | Should Be $false
    }

    It 'rejects latency artifacts that exceed either p95 budget' {
        $profileDir = Join-Path $TestDrive 'budget'
        $artifact = New-ValidLatencyArtifact
        $artifact.input_to_damage_p95_us = 1001
        $artifact.input_to_damage_p99_us = 1100
        $artifact.input_to_damage_max_us = 1200
        Write-LatencyArtifact -ProfileDir $profileDir -Artifact $artifact
        (Test-ClickLatencyArtifact -ProfileDir $profileDir) | Should Be $false

        $artifact.input_to_damage_p95_us = 800
        $artifact.damage_to_submit_p95_us = 8001
        $artifact.damage_to_submit_p99_us = 8500
        $artifact.damage_to_submit_max_us = 9000
        Write-LatencyArtifact -ProfileDir $profileDir -Artifact $artifact
        (Test-ClickLatencyArtifact -ProfileDir $profileDir) | Should Be $false
    }

    It 'accepts complete zero-overwrite evidence within both p95 budgets' {
        $profileDir = Join-Path $TestDrive 'valid'
        Write-LatencyArtifact -ProfileDir $profileDir -Artifact (New-ValidLatencyArtifact)

        (Test-ClickLatencyArtifact -ProfileDir $profileDir) | Should Be $true
    }
}
