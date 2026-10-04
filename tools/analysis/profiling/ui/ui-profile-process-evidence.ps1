$script:ZirconUiMaxAverageCoreUtilizationPercent = 100.0
$script:ZirconUiMaxEndMemoryGrowthBytes = 67108864
$script:ZirconUiMaxPeakMemoryGrowthBytes = 100663296

function Test-ZirconUiInteractionProcessEvidence {
    param(
        [object]$Interaction,
        [int64]$OperationCount = 0,
        [double]$MaxCpuMsPerOperation = 0.0
    )

    if ($null -eq $Interaction) {
        return $false
    }
    $requiredFields = @(
        'process_id',
        'elapsed_ms',
        'processor_time_delta_ms',
        'cpu_core_utilization_percent',
        'cpu_system_utilization_percent',
        'logical_processor_count',
        'start_working_set_bytes',
        'end_working_set_bytes',
        'peak_working_set_bytes',
        'start_private_bytes',
        'end_private_bytes',
        'peak_private_bytes',
        'quiescence_process_id',
        'quiescence_requested_ms',
        'quiescence_elapsed_ms',
        'quiescence_working_set_bytes',
        'quiescence_private_bytes',
        'quiescence_sampled'
    )
    foreach ($field in $requiredFields) {
        $property = $Interaction.PSObject.Properties[$field]
        if ($null -eq $property -or $null -eq $property.Value) {
            return $false
        }
    }

    try {
        $processId = [int64]$Interaction.process_id
        $elapsedMs = [double]$Interaction.elapsed_ms
        $processorTimeDeltaMs = [double]$Interaction.processor_time_delta_ms
        $cpuCorePercent = [double]$Interaction.cpu_core_utilization_percent
        $cpuSystemPercent = [double]$Interaction.cpu_system_utilization_percent
        $logicalProcessorCount = [int64]$Interaction.logical_processor_count
        $startWorkingSetBytes = [int64]$Interaction.start_working_set_bytes
        $endWorkingSetBytes = [int64]$Interaction.end_working_set_bytes
        $peakWorkingSetBytes = [int64]$Interaction.peak_working_set_bytes
        $startPrivateBytes = [int64]$Interaction.start_private_bytes
        $endPrivateBytes = [int64]$Interaction.end_private_bytes
        $peakPrivateBytes = [int64]$Interaction.peak_private_bytes
        $quiescenceProcessId = [int64]$Interaction.quiescence_process_id
        $quiescenceRequestedMs = [double]$Interaction.quiescence_requested_ms
        $quiescenceElapsedMs = [double]$Interaction.quiescence_elapsed_ms
        $quiescenceWorkingSetBytes = [int64]$Interaction.quiescence_working_set_bytes
        $quiescencePrivateBytes = [int64]$Interaction.quiescence_private_bytes
        $quiescenceSampled = [bool]$Interaction.quiescence_sampled
    }
    catch {
        return $false
    }
    foreach ($value in @(
            $elapsedMs,
            $processorTimeDeltaMs,
            $cpuCorePercent,
            $cpuSystemPercent,
            $quiescenceRequestedMs,
            $quiescenceElapsedMs
        )) {
        if ([double]::IsNaN($value) -or [double]::IsInfinity($value) -or $value -lt 0.0) {
            return $false
        }
    }
    if ($processId -le 0 -or $quiescenceProcessId -ne $processId -or
        -not $quiescenceSampled -or
        $quiescenceElapsedMs -lt $quiescenceRequestedMs -or
        $elapsedMs -le 0.0 -or $logicalProcessorCount -le 0 -or
        $startWorkingSetBytes -le 0 -or $endWorkingSetBytes -le 0 -or
        $quiescenceWorkingSetBytes -le 0 -or
        $peakWorkingSetBytes -lt [Math]::Max(
            $quiescenceWorkingSetBytes,
            [Math]::Max($startWorkingSetBytes, $endWorkingSetBytes)) -or
        $startPrivateBytes -le 0 -or $endPrivateBytes -le 0 -or
        $quiescencePrivateBytes -le 0 -or
        $peakPrivateBytes -lt [Math]::Max(
            $quiescencePrivateBytes,
            [Math]::Max($startPrivateBytes, $endPrivateBytes))) {
        return $false
    }

    $expectedCorePercent = ($processorTimeDeltaMs / $elapsedMs) * 100.0
    $expectedSystemPercent = $expectedCorePercent / [double]$logicalProcessorCount
    $coreTolerance = [Math]::Max(0.1, $expectedCorePercent * 0.01)
    $systemTolerance = [Math]::Max(0.1, $expectedSystemPercent * 0.01)
    if ([Math]::Abs($cpuCorePercent - $expectedCorePercent) -gt $coreTolerance -or
        [Math]::Abs($cpuSystemPercent - $expectedSystemPercent) -gt $systemTolerance -or
        $cpuCorePercent -gt $script:ZirconUiMaxAverageCoreUtilizationPercent) {
        return $false
    }

    if (($endWorkingSetBytes - $startWorkingSetBytes) -gt $script:ZirconUiMaxEndMemoryGrowthBytes -or
        ($quiescenceWorkingSetBytes - $startWorkingSetBytes) -gt $script:ZirconUiMaxEndMemoryGrowthBytes -or
        ($peakWorkingSetBytes - $startWorkingSetBytes) -gt $script:ZirconUiMaxPeakMemoryGrowthBytes -or
        ($endPrivateBytes - $startPrivateBytes) -gt $script:ZirconUiMaxEndMemoryGrowthBytes -or
        ($quiescencePrivateBytes - $startPrivateBytes) -gt $script:ZirconUiMaxEndMemoryGrowthBytes -or
        ($peakPrivateBytes - $startPrivateBytes) -gt $script:ZirconUiMaxPeakMemoryGrowthBytes) {
        return $false
    }

    if ($OperationCount -gt 0) {
        if ($MaxCpuMsPerOperation -le 0.0 -or
            ($processorTimeDeltaMs / [double]$OperationCount) -gt $MaxCpuMsPerOperation) {
            return $false
        }
    }
    return $true
}
