function Get-ZirconNativeResizeSequence {
    param(
        [Parameter(Mandatory = $true)]
        [int]$Width,
        [Parameter(Mandatory = $true)]
        [int]$Height,
        [ValidateRange(2, 240)]
        [int]$StepCount = 24
    )

    $widthOffsets = @(-24, -48, -72, -48, -24, 24, 48, 24)
    $heightOffsets = @(-16, -32, -48, -32, -16, 16, 32, 16)
    $steps = for ($index = 0; $index -lt $StepCount; $index++) {
        $offsetIndex = $index % $widthOffsets.Count
        [pscustomobject]@{
            width = [Math]::Max(320, $Width + $widthOffsets[$offsetIndex])
            height = [Math]::Max(240, $Height + $heightOffsets[$offsetIndex])
        }
    }
    @($steps) + [pscustomobject]@{ width = $Width; height = $Height }
}

function Measure-ZirconProcessCpuEvidence {
    param(
        [Parameter(Mandatory = $true)]
        [double]$StartProcessorTimeMs,
        [Parameter(Mandatory = $true)]
        [double]$EndProcessorTimeMs,
        [Parameter(Mandatory = $true)]
        [double]$ElapsedMs,
        [ValidateRange(1, 65536)]
        [int]$LogicalProcessorCount = [Environment]::ProcessorCount
    )

    $processorTimeDeltaMs = [Math]::Max(0.0, $EndProcessorTimeMs - $StartProcessorTimeMs)
    $cpuCoreUtilizationPercent = if ($ElapsedMs -gt 0.0) {
        [Math]::Min(
            [double]$LogicalProcessorCount * 100.0,
            ($processorTimeDeltaMs / $ElapsedMs) * 100.0)
    }
    else {
        0.0
    }

    [pscustomobject]@{
        start_processor_time_ms = [Math]::Round($StartProcessorTimeMs, 3)
        end_processor_time_ms = [Math]::Round($EndProcessorTimeMs, 3)
        processor_time_delta_ms = [Math]::Round($processorTimeDeltaMs, 3)
        cpu_core_utilization_percent = [Math]::Round($cpuCoreUtilizationPercent, 3)
        cpu_system_utilization_percent = [Math]::Round(
            $cpuCoreUtilizationPercent / [double]$LogicalProcessorCount,
            3)
        logical_processor_count = $LogicalProcessorCount
    }
}

function Complete-ZirconProcessQuiescenceEvidence {
    param(
        [Parameter(Mandatory = $true)]
        [System.Diagnostics.Process]$Process,
        [Parameter(Mandatory = $true)]
        [object]$Interaction,
        [ValidateRange(0, 30)]
        [int]$QuiescenceSeconds = 2
    )

    foreach ($field in @(
            'process_id',
            'start_working_set_bytes',
            'peak_working_set_bytes',
            'start_private_bytes',
            'peak_private_bytes'
        )) {
        if ($null -eq $Interaction.PSObject.Properties[$field]) {
            return $Interaction
        }
    }
    if ($Process.HasExited -or [int]$Interaction.process_id -ne $Process.Id) {
        return $Interaction
    }

    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    $deadline = [DateTime]::UtcNow.AddSeconds($QuiescenceSeconds)
    $peakWorkingSetBytes = [int64]$Interaction.peak_working_set_bytes
    $peakPrivateBytes = [int64]$Interaction.peak_private_bytes
    $quiescenceWorkingSetBytes = 0L
    $quiescencePrivateBytes = 0L
    while ($true) {
        if ($Process.HasExited) {
            return $Interaction
        }
        $Process.Refresh()
        $quiescenceWorkingSetBytes = [int64]$Process.WorkingSet64
        $quiescencePrivateBytes = [int64]$Process.PrivateMemorySize64
        $peakWorkingSetBytes = [Math]::Max(
            $peakWorkingSetBytes,
            $quiescenceWorkingSetBytes)
        $peakPrivateBytes = [Math]::Max(
            $peakPrivateBytes,
            $quiescencePrivateBytes)
        if ([DateTime]::UtcNow -ge $deadline) {
            break
        }
        Start-Sleep -Milliseconds 100
    }
    $stopwatch.Stop()

    $Interaction | Add-Member -NotePropertyName peak_working_set_bytes `
        -NotePropertyValue $peakWorkingSetBytes -Force
    $Interaction | Add-Member -NotePropertyName peak_private_bytes `
        -NotePropertyValue $peakPrivateBytes -Force
    $Interaction | Add-Member -NotePropertyName quiescence_process_id `
        -NotePropertyValue $Process.Id -Force
    $Interaction | Add-Member -NotePropertyName quiescence_requested_ms `
        -NotePropertyValue ([int64]$QuiescenceSeconds * 1000) -Force
    $Interaction | Add-Member -NotePropertyName quiescence_elapsed_ms `
        -NotePropertyValue ([Math]::Round($stopwatch.Elapsed.TotalMilliseconds, 3)) -Force
    $Interaction | Add-Member -NotePropertyName quiescence_working_set_bytes `
        -NotePropertyValue $quiescenceWorkingSetBytes -Force
    $Interaction | Add-Member -NotePropertyName quiescence_private_bytes `
        -NotePropertyValue $quiescencePrivateBytes -Force
    $Interaction | Add-Member -NotePropertyName quiescence_sampled `
        -NotePropertyValue $true -Force
    return $Interaction
}

function Invoke-PointerMoveStorm {
    param(
        [System.Diagnostics.Process]$Process,
        [object[]]$Targets,
        [int]$Count,
        [int]$DelayMs
    )

    if ($Count -le 0 -or $Targets.Count -eq 0) {
        return $null
    }

    $completed = 0
    $Process.Refresh()
    $startProcessorTimeMs = [double]$Process.TotalProcessorTime.TotalMilliseconds
    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    $startWorkingSetBytes = [int64]$Process.WorkingSet64
    $startPrivateBytes = [int64]$Process.PrivateMemorySize64
    $peakWorkingSetBytes = $startWorkingSetBytes
    $peakPrivateBytes = $startPrivateBytes

    for ($index = 0; $index -lt $Count; $index++) {
        if ($Process.HasExited) {
            break
        }
        $target = $Targets[$index % $Targets.Count]
        if ([ZirconProfileCaptureNative]::SetCursorPos([int]$target.X, [int]$target.Y)) {
            $completed++
        }
        if ($DelayMs -gt 0) {
            Start-Sleep -Milliseconds $DelayMs
        }
        if (($index % 32) -eq 31) {
            $Process.Refresh()
            $peakWorkingSetBytes = [Math]::Max($peakWorkingSetBytes, [int64]$Process.WorkingSet64)
            $peakPrivateBytes = [Math]::Max($peakPrivateBytes, [int64]$Process.PrivateMemorySize64)
        }
    }

    $stopwatch.Stop()
    $Process.Refresh()
    $endWorkingSetBytes = [int64]$Process.WorkingSet64
    $endPrivateBytes = [int64]$Process.PrivateMemorySize64
    $peakWorkingSetBytes = [Math]::Max($peakWorkingSetBytes, $endWorkingSetBytes)
    $peakPrivateBytes = [Math]::Max($peakPrivateBytes, $endPrivateBytes)
    $cpuEvidence = Measure-ZirconProcessCpuEvidence `
        -StartProcessorTimeMs $startProcessorTimeMs `
        -EndProcessorTimeMs ([double]$Process.TotalProcessorTime.TotalMilliseconds) `
        -ElapsedMs $stopwatch.Elapsed.TotalMilliseconds
    $targetEvidence = @($Targets | ForEach-Object {
            [pscustomobject]@{
                target_id = [string]$_.target_id
                target_kind = [string]$_.target_kind
                target_surface = [string]$_.target_surface
                source = [string]$_.source
                x = [int]$_.X
                y = [int]$_.Y
            }
        })
    $geometryTargetCount = @($targetEvidence | Where-Object {
            $_.source -eq 'ui_profile_geometry.json'
        }).Count

    Write-Host ("Pointer storm: requested={0} completed={1} elapsed_ms={2:N1} cpu_ms={3:N1}" -f `
            $Count,
            $completed,
            $stopwatch.Elapsed.TotalMilliseconds,
            $cpuEvidence.processor_time_delta_ms)
    [pscustomobject]@{
        scenario = 'pointer_move_storm'
        process_id = $Process.Id
        requested_moves = $Count
        completed_moves = $completed
        target_count = $targetEvidence.Count
        used_geometry = $targetEvidence.Count -gt 0 -and $geometryTargetCount -eq $targetEvidence.Count
        targets = $targetEvidence
        delay_ms = $DelayMs
        elapsed_ms = [Math]::Round($stopwatch.Elapsed.TotalMilliseconds, 3)
        start_working_set_bytes = $startWorkingSetBytes
        end_working_set_bytes = $endWorkingSetBytes
        peak_working_set_bytes = $peakWorkingSetBytes
        start_private_bytes = $startPrivateBytes
        end_private_bytes = $endPrivateBytes
        peak_private_bytes = $peakPrivateBytes
        start_processor_time_ms = $cpuEvidence.start_processor_time_ms
        end_processor_time_ms = $cpuEvidence.end_processor_time_ms
        processor_time_delta_ms = $cpuEvidence.processor_time_delta_ms
        cpu_core_utilization_percent = $cpuEvidence.cpu_core_utilization_percent
        cpu_system_utilization_percent = $cpuEvidence.cpu_system_utilization_percent
        logical_processor_count = $cpuEvidence.logical_processor_count
    }
}

function Invoke-PointerClickStorm {
    param(
        [System.Diagnostics.Process]$Process,
        [object[]]$Targets,
        [int]$Count,
        [int]$DelayMs
    )

    if ($Count -le 0 -or $Targets.Count -eq 0) {
        return $null
    }

    $completed = 0
    $Process.Refresh()
    $startProcessorTimeMs = [double]$Process.TotalProcessorTime.TotalMilliseconds
    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    $startWorkingSetBytes = [int64]$Process.WorkingSet64
    $startPrivateBytes = [int64]$Process.PrivateMemorySize64
    $peakWorkingSetBytes = $startWorkingSetBytes
    $peakPrivateBytes = $startPrivateBytes

    for ($index = 0; $index -lt $Count; $index++) {
        if ($Process.HasExited) {
            break
        }
        $target = $Targets[$index % $Targets.Count]
        [ZirconProfileCaptureNative]::SetCursorPos([int]$target.X, [int]$target.Y) | Out-Null
        [ZirconProfileCaptureNative]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)
        [ZirconProfileCaptureNative]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
        $completed++
        if ($DelayMs -gt 0) {
            Start-Sleep -Milliseconds $DelayMs
        }
        if (($index % 32) -eq 31) {
            $Process.Refresh()
            $peakWorkingSetBytes = [Math]::Max($peakWorkingSetBytes, [int64]$Process.WorkingSet64)
            $peakPrivateBytes = [Math]::Max($peakPrivateBytes, [int64]$Process.PrivateMemorySize64)
        }
    }

    $stopwatch.Stop()
    $Process.Refresh()
    $endWorkingSetBytes = [int64]$Process.WorkingSet64
    $endPrivateBytes = [int64]$Process.PrivateMemorySize64
    $peakWorkingSetBytes = [Math]::Max($peakWorkingSetBytes, $endWorkingSetBytes)
    $peakPrivateBytes = [Math]::Max($peakPrivateBytes, $endPrivateBytes)
    $cpuEvidence = Measure-ZirconProcessCpuEvidence `
        -StartProcessorTimeMs $startProcessorTimeMs `
        -EndProcessorTimeMs ([double]$Process.TotalProcessorTime.TotalMilliseconds) `
        -ElapsedMs $stopwatch.Elapsed.TotalMilliseconds
    $targetEvidence = @($Targets | ForEach-Object {
            [pscustomobject]@{
                target_id = [string]$_.target_id
                target_kind = [string]$_.target_kind
                target_surface = [string]$_.target_surface
                source = [string]$_.source
                x = [int]$_.X
                y = [int]$_.Y
            }
        })
    $geometryTargetCount = @($targetEvidence | Where-Object {
            $_.source -eq 'ui_profile_geometry.json'
        }).Count

    Write-Host ("Click storm: requested={0} completed={1} elapsed_ms={2:N1} cpu_ms={3:N1}" -f `
            $Count,
            $completed,
            $stopwatch.Elapsed.TotalMilliseconds,
            $cpuEvidence.processor_time_delta_ms)
    [pscustomobject]@{
        scenario = 'pointer_click_storm'
        process_id = $Process.Id
        requested_clicks = $Count
        completed_clicks = $completed
        point_count = $Targets.Count
        target_count = $targetEvidence.Count
        used_geometry = $targetEvidence.Count -gt 0 -and $geometryTargetCount -eq $targetEvidence.Count
        targets = $targetEvidence
        delay_ms = $DelayMs
        elapsed_ms = [Math]::Round($stopwatch.Elapsed.TotalMilliseconds, 3)
        start_working_set_bytes = $startWorkingSetBytes
        end_working_set_bytes = $endWorkingSetBytes
        peak_working_set_bytes = $peakWorkingSetBytes
        start_private_bytes = $startPrivateBytes
        end_private_bytes = $endPrivateBytes
        peak_private_bytes = $peakPrivateBytes
        start_processor_time_ms = $cpuEvidence.start_processor_time_ms
        end_processor_time_ms = $cpuEvidence.end_processor_time_ms
        processor_time_delta_ms = $cpuEvidence.processor_time_delta_ms
        cpu_core_utilization_percent = $cpuEvidence.cpu_core_utilization_percent
        cpu_system_utilization_percent = $cpuEvidence.cpu_system_utilization_percent
        logical_processor_count = $cpuEvidence.logical_processor_count
    }
}

function Invoke-PointerWheelStorm {
    param(
        [System.Diagnostics.Process]$Process,
        [object[]]$Targets,
        [int]$Count,
        [int]$DelayMs,
        [ValidateRange(1, 1200)]
        [int]$WheelDelta = 120
    )

    if ($Count -le 0 -or $Targets.Count -eq 0) {
        return $null
    }

    $completed = 0
    $Process.Refresh()
    $startProcessorTimeMs = [double]$Process.TotalProcessorTime.TotalMilliseconds
    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    $startWorkingSetBytes = [int64]$Process.WorkingSet64
    $startPrivateBytes = [int64]$Process.PrivateMemorySize64
    $peakWorkingSetBytes = $startWorkingSetBytes
    $peakPrivateBytes = $startPrivateBytes

    for ($index = 0; $index -lt $Count; $index++) {
        if ($Process.HasExited) {
            break
        }
        $target = $Targets[$index % $Targets.Count]
        [ZirconProfileCaptureNative]::SetCursorPos([int]$target.X, [int]$target.Y) | Out-Null
        $signedDelta = if (([Math]::Floor($index / 32) % 2) -eq 0) {
            -[int32]$WheelDelta
        }
        else {
            [int32]$WheelDelta
        }
        $wheelData = [BitConverter]::ToUInt32([BitConverter]::GetBytes($signedDelta), 0)
        [ZirconProfileCaptureNative]::mouse_event(0x0800, 0, 0, $wheelData, [UIntPtr]::Zero)
        $completed++
        if ($DelayMs -gt 0) {
            Start-Sleep -Milliseconds $DelayMs
        }
        if (($index % 32) -eq 31) {
            $Process.Refresh()
            $peakWorkingSetBytes = [Math]::Max($peakWorkingSetBytes, [int64]$Process.WorkingSet64)
            $peakPrivateBytes = [Math]::Max($peakPrivateBytes, [int64]$Process.PrivateMemorySize64)
        }
    }

    $stopwatch.Stop()
    $Process.Refresh()
    $endWorkingSetBytes = [int64]$Process.WorkingSet64
    $endPrivateBytes = [int64]$Process.PrivateMemorySize64
    $peakWorkingSetBytes = [Math]::Max($peakWorkingSetBytes, $endWorkingSetBytes)
    $peakPrivateBytes = [Math]::Max($peakPrivateBytes, $endPrivateBytes)
    $cpuEvidence = Measure-ZirconProcessCpuEvidence `
        -StartProcessorTimeMs $startProcessorTimeMs `
        -EndProcessorTimeMs ([double]$Process.TotalProcessorTime.TotalMilliseconds) `
        -ElapsedMs $stopwatch.Elapsed.TotalMilliseconds
    $targetEvidence = @($Targets | ForEach-Object {
            [pscustomobject]@{
                target_id = [string]$_.target_id
                target_kind = [string]$_.target_kind
                target_surface = [string]$_.target_surface
                source = [string]$_.source
                x = [int]$_.X
                y = [int]$_.Y
            }
        })
    $geometryTargetCount = @($targetEvidence | Where-Object {
            $_.source -eq 'ui_profile_geometry.json'
        }).Count

    Write-Host ("Wheel storm: requested={0} completed={1} elapsed_ms={2:N1} cpu_ms={3:N1}" -f `
            $Count,
            $completed,
            $stopwatch.Elapsed.TotalMilliseconds,
            $cpuEvidence.processor_time_delta_ms)
    [pscustomobject]@{
        scenario = 'pointer_wheel_storm'
        process_id = $Process.Id
        requested_wheel_events = $Count
        completed_wheel_events = $completed
        target_count = $targetEvidence.Count
        used_geometry = $targetEvidence.Count -gt 0 -and $geometryTargetCount -eq $targetEvidence.Count
        targets = $targetEvidence
        wheel_delta = $WheelDelta
        alternating_direction = $true
        direction_batch_size = 32
        delay_ms = $DelayMs
        elapsed_ms = [Math]::Round($stopwatch.Elapsed.TotalMilliseconds, 3)
        start_working_set_bytes = $startWorkingSetBytes
        end_working_set_bytes = $endWorkingSetBytes
        peak_working_set_bytes = $peakWorkingSetBytes
        start_private_bytes = $startPrivateBytes
        end_private_bytes = $endPrivateBytes
        peak_private_bytes = $peakPrivateBytes
        start_processor_time_ms = $cpuEvidence.start_processor_time_ms
        end_processor_time_ms = $cpuEvidence.end_processor_time_ms
        processor_time_delta_ms = $cpuEvidence.processor_time_delta_ms
        cpu_core_utilization_percent = $cpuEvidence.cpu_core_utilization_percent
        cpu_system_utilization_percent = $cpuEvidence.cpu_system_utilization_percent
        logical_processor_count = $cpuEvidence.logical_processor_count
    }
}

function Invoke-ZirconNativeResizeInteraction {
    param(
        [Parameter(Mandatory = $true)]
        [System.Diagnostics.Process]$Process,
        [ValidateRange(2, 240)]
        [int]$StepCount = 24,
        [ValidateRange(1, 1000)]
        [int]$DelayMs = 40
    )

    if (-not ('ZirconProfileCaptureNative' -as [type])) {
        throw 'ZirconProfileCaptureNative must be initialized before native resize interaction.'
    }
    $Process.Refresh()
    if ($Process.MainWindowHandle -eq [IntPtr]::Zero) {
        throw 'Native resize interaction requires an editor main window.'
    }

    $original = New-Object ZirconProfileCaptureRect
    if (-not [ZirconProfileCaptureNative]::GetWindowRect($Process.MainWindowHandle, [ref]$original)) {
        throw 'Native resize interaction could not read the editor window rectangle.'
    }
    $originalWidth = $original.Right - $original.Left
    $originalHeight = $original.Bottom - $original.Top
    $sequence = @(Get-ZirconNativeResizeSequence `
            -Width $originalWidth `
            -Height $originalHeight `
            -StepCount $StepCount)
    $setWindowPosFlags = 0x0002 -bor 0x0004 -bor 0x0010
    $completed = 0
    $startProcessorTimeMs = [double]$Process.TotalProcessorTime.TotalMilliseconds
    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    $startWorkingSetBytes = [int64]$Process.WorkingSet64
    $startPrivateBytes = [int64]$Process.PrivateMemorySize64
    $peakWorkingSetBytes = $startWorkingSetBytes
    $peakPrivateBytes = $startPrivateBytes

    foreach ($step in $sequence) {
        if ($Process.HasExited) {
            break
        }
        if ([ZirconProfileCaptureNative]::SetWindowPos(
                $Process.MainWindowHandle,
                [IntPtr]::Zero,
                0,
                0,
                [int]$step.width,
                [int]$step.height,
                $setWindowPosFlags)) {
            $completed++
        }
        Start-Sleep -Milliseconds $DelayMs
        if (($completed % 4) -eq 0) {
            $Process.Refresh()
            $peakWorkingSetBytes = [Math]::Max($peakWorkingSetBytes, [int64]$Process.WorkingSet64)
            $peakPrivateBytes = [Math]::Max($peakPrivateBytes, [int64]$Process.PrivateMemorySize64)
        }
    }
    Start-Sleep -Milliseconds 200
    $stopwatch.Stop()
    $Process.Refresh()
    $final = New-Object ZirconProfileCaptureRect
    $finalRectAvailable = [ZirconProfileCaptureNative]::GetWindowRect(
        $Process.MainWindowHandle,
        [ref]$final)
    $restored = $finalRectAvailable -and
        ($final.Right - $final.Left) -eq $originalWidth -and
        ($final.Bottom - $final.Top) -eq $originalHeight
    $endWorkingSetBytes = [int64]$Process.WorkingSet64
    $endPrivateBytes = [int64]$Process.PrivateMemorySize64
    $peakWorkingSetBytes = [Math]::Max($peakWorkingSetBytes, $endWorkingSetBytes)
    $peakPrivateBytes = [Math]::Max($peakPrivateBytes, $endPrivateBytes)
    $cpuEvidence = Measure-ZirconProcessCpuEvidence `
        -StartProcessorTimeMs $startProcessorTimeMs `
        -EndProcessorTimeMs ([double]$Process.TotalProcessorTime.TotalMilliseconds) `
        -ElapsedMs $stopwatch.Elapsed.TotalMilliseconds

    [pscustomobject]@{
        scenario = 'window_resize'
        process_id = $Process.Id
        requested_steps = $sequence.Count
        completed_steps = $completed
        delay_ms = $DelayMs
        elapsed_ms = [Math]::Round($stopwatch.Elapsed.TotalMilliseconds, 3)
        original_width = $originalWidth
        original_height = $originalHeight
        restored_original_extent = $restored
        start_working_set_bytes = $startWorkingSetBytes
        end_working_set_bytes = $endWorkingSetBytes
        peak_working_set_bytes = $peakWorkingSetBytes
        start_private_bytes = $startPrivateBytes
        end_private_bytes = $endPrivateBytes
        peak_private_bytes = $peakPrivateBytes
        start_processor_time_ms = $cpuEvidence.start_processor_time_ms
        end_processor_time_ms = $cpuEvidence.end_processor_time_ms
        processor_time_delta_ms = $cpuEvidence.processor_time_delta_ms
        cpu_core_utilization_percent = $cpuEvidence.cpu_core_utilization_percent
        cpu_system_utilization_percent = $cpuEvidence.cpu_system_utilization_percent
        logical_processor_count = $cpuEvidence.logical_processor_count
    }
}
