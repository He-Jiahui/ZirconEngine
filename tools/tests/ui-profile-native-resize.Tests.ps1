$script:ResizeModule = Join-Path $PSScriptRoot '..\ui-profile-native-resize.ps1'
$script:CaptureScript = Join-Path $PSScriptRoot '..\ui-profile-capture.ps1'

Describe 'UI profile native resize contract' {
    It 'builds a bounded deterministic resize sequence that returns to the original extent' {
        . $script:ResizeModule

        $steps = @(Get-ZirconNativeResizeSequence -Width 1280 -Height 720 -StepCount 8)

        $steps.Count | Should Be 9
        $steps[0].width | Should Be 1256
        $steps[0].height | Should Be 704
        $steps[-1].width | Should Be 1280
        $steps[-1].height | Should Be 720
        (@($steps | Where-Object { $_.width -lt 320 -or $_.height -lt 240 })).Count | Should Be 0
    }

    It 'derives bounded process CPU evidence from processor and wall-clock samples' {
        . $script:ResizeModule

        $metrics = Measure-ZirconProcessCpuEvidence `
            -StartProcessorTimeMs 100.0 `
            -EndProcessorTimeMs 350.0 `
            -ElapsedMs 500.0 `
            -LogicalProcessorCount 8

        $metrics.processor_time_delta_ms | Should Be 250.0
        $metrics.cpu_core_utilization_percent | Should Be 50.0
        $metrics.cpu_system_utilization_percent | Should Be 6.25
        $metrics.logical_processor_count | Should Be 8
    }

    It 'attaches bounded quiescence memory evidence from the same process' {
        . $script:ResizeModule

        $process = [System.Diagnostics.Process]::GetCurrentProcess()
        $process.Refresh()
        $interaction = [pscustomobject]@{
            process_id = $process.Id
            start_working_set_bytes = [int64]$process.WorkingSet64
            peak_working_set_bytes = [int64]$process.WorkingSet64
            start_private_bytes = [int64]$process.PrivateMemorySize64
            peak_private_bytes = [int64]$process.PrivateMemorySize64
        }

        $result = Complete-ZirconProcessQuiescenceEvidence `
            -Process $process `
            -Interaction $interaction `
            -QuiescenceSeconds 0

        $result.quiescence_sampled | Should Be $true
        $result.quiescence_process_id | Should Be $process.Id
        $result.quiescence_working_set_bytes | Should BeGreaterThan 0
        $result.quiescence_private_bytes | Should BeGreaterThan 0
        $result.peak_working_set_bytes |
            Should Not BeLessThan $result.quiescence_working_set_bytes
        $result.peak_private_bytes |
            Should Not BeLessThan $result.quiescence_private_bytes
    }

    It 'registers window resize and requires snapshot reuse evidence' {
        $source = Get-Content -LiteralPath $script:CaptureScript -Raw
        $interactionSource = Get-Content -LiteralPath $script:ResizeModule -Raw

        $source | Should Match '"window_resize"'
        $source | Should Match 'Invoke-ZirconNativeResizeInteraction'
        $source | Should Match 'ui\.window_resize\.command_snapshot_build_count'
        $source | Should Match 'ui\.window_resize\.command_snapshot_reuse_count'
        $source | Should Match 'ui\.window_resize\.surface_reconfigure_count'
        $source | Should Match '-StepCount \$AutoResizeStepCount'
        $source | Should Match '-DelayMs \$AutoResizeDelayMs'
        $interactionSource | Should Match 'Invoke-PointerClickStorm'
        $interactionSource | Should Match 'processor_time_delta_ms'
        $interactionSource | Should Match 'cpu_system_utilization_percent'
        $interactionSource | Should Match 'Complete-ZirconProcessQuiescenceEvidence'
        $interactionSource | Should Match 'quiescence_process_id'
        $interactionSource | Should Match 'Start-Sleep -Milliseconds 100'
    }
}
