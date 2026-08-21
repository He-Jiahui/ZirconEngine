$script:ZirconInputToDamageP95BudgetUs = 1000.0
$script:ZirconDamageToSubmitP95BudgetUs = 8000.0

function Get-ZirconLatencyPercentiles {
    param([object[]]$Samples)

    $values = @($Samples | ForEach-Object { [double]$_.value } | Sort-Object)
    if ($values.Count -eq 0) {
        return [pscustomobject]@{
            sample_count = 0
            p50_us = 0.0
            p95_us = 0.0
            p99_us = 0.0
            max_us = 0.0
        }
    }
    $p50Index = [Math]::Max(0, [int][Math]::Ceiling($values.Count * 0.50) - 1)
    $p95Index = [Math]::Max(0, [int][Math]::Ceiling($values.Count * 0.95) - 1)
    $p99Index = [Math]::Max(0, [int][Math]::Ceiling($values.Count * 0.99) - 1)
    return [pscustomobject]@{
        sample_count = $values.Count
        p50_us = $values[$p50Index]
        p95_us = $values[$p95Index]
        p99_us = $values[$p99Index]
        max_us = $values[-1]
    }
}

function Test-ZirconRetentionStreamEvidence {
    param([object]$Stream)

    if ($null -eq $Stream) {
        return $false
    }
    foreach ($field in @('capacity', 'written', 'overwritten', 'retained', 'oldest_sequence', 'newest_sequence')) {
        if ($null -eq $Stream.PSObject.Properties[$field]) {
            return $false
        }
    }
    $capacity = [int64]$Stream.capacity
    $written = [int64]$Stream.written
    $overwritten = [int64]$Stream.overwritten
    $retained = [int64]$Stream.retained
    if ($capacity -le 0 -or $written -lt 0 -or $overwritten -lt 0 -or $retained -lt 0 -or
        $retained -gt $capacity -or $overwritten -gt $written -or
        $written -ne ($overwritten + $retained)) {
        return $false
    }
    if ($retained -eq 0) {
        return $null -eq $Stream.oldest_sequence -and $null -eq $Stream.newest_sequence
    }
    if ($null -eq $Stream.oldest_sequence -or $null -eq $Stream.newest_sequence) {
        return $false
    }
    return [int64]$Stream.oldest_sequence -eq ($written - $retained) -and
        [int64]$Stream.newest_sequence -eq ($written - 1)
}

function Get-ZirconRecorderRetentionEvidence {
    param([object]$Snapshot)

    $sources = @($Snapshot.recorder_retention)
    $result = [ordered]@{
        retention_source_count = $sources.Count
        retention_complete = $sources.Count -gt 0
    }
    foreach ($streamName in @('frame', 'span', 'counter')) {
        foreach ($field in @('capacity', 'written', 'overwritten', 'retained')) {
            $result["${streamName}_${field}"] = [int64]0
        }
    }
    foreach ($source in $sources) {
        foreach ($mapping in @(
                @{ artifact = 'frame'; snapshot = 'frames' },
                @{ artifact = 'span'; snapshot = 'spans' },
                @{ artifact = 'counter'; snapshot = 'counters' }
            )) {
            $stream = $source.($mapping.snapshot)
            if (-not (Test-ZirconRetentionStreamEvidence -Stream $stream)) {
                $result.retention_complete = $false
                continue
            }
            foreach ($field in @('capacity', 'written', 'overwritten', 'retained')) {
                $key = "$($mapping.artifact)_${field}"
                $result[$key] = [int64]$result[$key] + [int64]$stream.$field
            }
        }
    }
    return [pscustomobject]$result
}

function ConvertTo-ZirconInputSequence {
    param([object]$Value)

    try {
        $number = [double]$Value
        if ([double]::IsNaN($number) -or [double]::IsInfinity($number) -or
            $number -lt 0.0 -or $number -gt 9007199254740991.0 -or
            [Math]::Floor($number) -ne $number) {
            return $null
        }
        return [uint64]$number
    }
    catch {
        return $null
    }
}

function Get-ZirconTypedInputOutcomeEvidence {
    param([object]$Snapshot)

    $counters = @($Snapshot.counters)
    $outcomeKinds = @{
        'ui.input.outcome.damaged_sequence' = 'damaged'
        'ui.input.outcome.intentionally_no_damage_sequence' = 'intentionally_no_damage'
        'ui.input.outcome.rejected_sequence' = 'rejected'
    }
    $outcomes = @()
    $valid = $true
    for ($index = 0; $index -lt $counters.Count; $index++) {
        $sample = $counters[$index]
        $kind = $outcomeKinds[[string]$sample.name]
        if ($null -eq $kind) {
            continue
        }
        $sequence = ConvertTo-ZirconInputSequence -Value $sample.value
        if ($null -eq $sequence -or $null -eq $sample.PSObject.Properties['timestamp_us']) {
            $valid = $false
            continue
        }
        $outcomes += [pscustomobject]@{
            sequence = $sequence
            kind = $kind
            counter_index = $index
            timestamp_us = [uint64]$sample.timestamp_us
        }
    }

    $outcomeGroups = @($outcomes | Group-Object -Property sequence)
    if (@($outcomeGroups | Where-Object Count -ne 1).Count -gt 0) {
        $valid = $false
    }
    $damagedOutcomes = @($outcomes | Where-Object kind -eq 'damaged')
    $quietOutcomes = @($outcomes | Where-Object kind -eq 'intentionally_no_damage')
    $rejectedOutcomes = @($outcomes | Where-Object kind -eq 'rejected')
    $inputLatencySamples = @($counters | Where-Object { $_.name -match '^ui\.[^.]+\.input_to_damage_us$' })
    foreach ($outcome in $damagedOutcomes) {
        $latencyIndex = [int]$outcome.counter_index + 1
        if ($latencyIndex -ge $counters.Count -or
            $counters[$latencyIndex].name -notmatch '^ui\.[^.]+\.input_to_damage_us$' -or
            $null -eq $counters[$latencyIndex].PSObject.Properties['timestamp_us'] -or
            [uint64]$counters[$latencyIndex].timestamp_us -ne [uint64]$outcome.timestamp_us) {
            $valid = $false
        }
    }
    if ($inputLatencySamples.Count -ne $damagedOutcomes.Count) {
        $valid = $false
    }

    $presentBatchCounterCount = @($counters | Where-Object {
            $_.name -match '^ui\.input\.present_batch\.(first_sequence|last_sequence|damaged_count)$'
        }).Count
    $damageLatencySamples = @($counters | Where-Object { $_.name -match '^ui\.[^.]+\.damage_to_submit_us$' })
    $damagedMembership = @{}
    $presentBatchCount = 0
    $presentBatchDamagedCount = [uint64]0
    $previousLastSequence = $null
    for ($index = 0; $index -lt $counters.Count; $index++) {
        if ($counters[$index].name -ne 'ui.input.present_batch.first_sequence') {
            continue
        }
        if (($index + 3) -ge $counters.Count) {
            $valid = $false
            continue
        }
        $firstSample = $counters[$index]
        $lastSample = $counters[$index + 1]
        $countSample = $counters[$index + 2]
        $latencySample = $counters[$index + 3]
        $firstSequence = ConvertTo-ZirconInputSequence -Value $firstSample.value
        $lastSequence = ConvertTo-ZirconInputSequence -Value $lastSample.value
        $damagedCount = ConvertTo-ZirconInputSequence -Value $countSample.value
        $hasBatchShape = $lastSample.name -eq 'ui.input.present_batch.last_sequence' -and
            $countSample.name -eq 'ui.input.present_batch.damaged_count' -and
            $latencySample.name -match '^ui\.[^.]+\.damage_to_submit_us$' -and
            $null -ne $firstSample.PSObject.Properties['timestamp_us'] -and
            $null -ne $lastSample.PSObject.Properties['timestamp_us'] -and
            $null -ne $countSample.PSObject.Properties['timestamp_us'] -and
            $null -ne $latencySample.PSObject.Properties['timestamp_us'] -and
            [uint64]$firstSample.timestamp_us -eq [uint64]$lastSample.timestamp_us -and
            [uint64]$firstSample.timestamp_us -eq [uint64]$countSample.timestamp_us -and
            [uint64]$firstSample.timestamp_us -eq [uint64]$latencySample.timestamp_us
        if (-not $hasBatchShape -or $null -eq $firstSequence -or $null -eq $lastSequence -or
            $null -eq $damagedCount -or $damagedCount -eq 0 -or
            $firstSequence -gt $lastSequence -or
            ($null -ne $previousLastSequence -and $firstSequence -le $previousLastSequence)) {
            $valid = $false
            continue
        }
        $members = @($damagedOutcomes | Where-Object {
                $_.sequence -ge $firstSequence -and $_.sequence -le $lastSequence
            })
        if ([uint64]$members.Count -ne $damagedCount) {
            $valid = $false
        }
        foreach ($member in $members) {
            $key = [string]$member.sequence
            $currentMembership = if ($damagedMembership.ContainsKey($key)) {
                [int]$damagedMembership[$key]
            }
            else {
                0
            }
            $damagedMembership[$key] = 1 + $currentMembership
        }
        $presentBatchCount++
        $presentBatchDamagedCount += $damagedCount
        $previousLastSequence = $lastSequence
    }
    if ($presentBatchCounterCount -ne ($presentBatchCount * 3) -or
        $damageLatencySamples.Count -ne $presentBatchCount) {
        $valid = $false
    }
    foreach ($outcome in $damagedOutcomes) {
        $key = [string]$outcome.sequence
        $membershipCount = if ($damagedMembership.ContainsKey($key)) {
            [int]$damagedMembership[$key]
        }
        else {
            0
        }
        if ($membershipCount -ne 1) {
            $valid = $false
        }
    }
    if ($outcomes.Count -eq 0) {
        $valid = $false
    }

    return [pscustomobject]@{
        input_outcome_count = $outcomes.Count
        damaged_input_outcome_count = $damagedOutcomes.Count
        intentionally_no_damage_input_outcome_count = $quietOutcomes.Count
        rejected_input_outcome_count = $rejectedOutcomes.Count
        present_batch_count = $presentBatchCount
        present_batch_damaged_count = $presentBatchDamagedCount
        typed_input_outcome_complete = $valid
    }
}

function Export-ZirconUiSurfacePresentOutcomeEvidence {
    param([string]$ProfileDir)

    $timelinePath = Join-Path $ProfileDir 'timeline.zrtrace.json'
    if (-not (Test-Path $timelinePath)) {
        Write-Warning 'Surface present outcome evidence could not find timeline.zrtrace.json.'
        return
    }

    $snapshot = Get-Content -Path $timelinePath -Raw | ConvertFrom-Json
    $submittedCount = [int64](@($snapshot.counters) |
            Where-Object { $_.name -eq 'ui.surface.submitted_count' } |
            Measure-Object -Property value -Sum).Sum
    $retryableNoSubmitCount = [int64](@($snapshot.counters) |
            Where-Object { $_.name -eq 'ui.surface.retryable_no_submit_count' } |
            Measure-Object -Property value -Sum).Sum
    $retryBackoffSamples = @($snapshot.counters) |
        Where-Object { $_.name -eq 'ui.surface.retry_backoff_ms' }
    $retryBackoffStats = $retryBackoffSamples |
        Measure-Object -Property value -Minimum -Maximum -Average
    $inputToDamageStats = Get-ZirconLatencyPercentiles -Samples @($snapshot.counters |
            Where-Object { $_.name -match '^ui\.[^.]+\.input_to_damage_us$' })
    $damageToSubmitStats = Get-ZirconLatencyPercentiles -Samples @($snapshot.counters |
            Where-Object { $_.name -match '^ui\.[^.]+\.damage_to_submit_us$' })
    $retention = Get-ZirconRecorderRetentionEvidence -Snapshot $snapshot
    $typedOutcomes = Get-ZirconTypedInputOutcomeEvidence -Snapshot $snapshot

    $artifact = [ordered]@{
        schema_version = 5
        source = 'timeline.zrtrace.json'
        submitted_count = $submittedCount
        retryable_no_submit_count = $retryableNoSubmitCount
        retry_backoff_sample_count = $retryBackoffSamples.Count
        retry_backoff_min_ms = if ($retryBackoffSamples.Count -gt 0) { [double]$retryBackoffStats.Minimum } else { 0.0 }
        retry_backoff_max_ms = if ($retryBackoffSamples.Count -gt 0) { [double]$retryBackoffStats.Maximum } else { 0.0 }
        retry_backoff_average_ms = if ($retryBackoffSamples.Count -gt 0) { [double]$retryBackoffStats.Average } else { 0.0 }
        input_to_damage_sample_count = $inputToDamageStats.sample_count
        input_to_damage_p50_us = $inputToDamageStats.p50_us
        input_to_damage_p95_us = $inputToDamageStats.p95_us
        input_to_damage_p99_us = $inputToDamageStats.p99_us
        input_to_damage_max_us = $inputToDamageStats.max_us
        damage_to_submit_sample_count = $damageToSubmitStats.sample_count
        damage_to_submit_p50_us = $damageToSubmitStats.p50_us
        damage_to_submit_p95_us = $damageToSubmitStats.p95_us
        damage_to_submit_p99_us = $damageToSubmitStats.p99_us
        damage_to_submit_max_us = $damageToSubmitStats.max_us
        retry_observed = $retryableNoSubmitCount -gt 0
    }
    foreach ($property in $retention.PSObject.Properties) {
        $artifact[$property.Name] = $property.Value
    }
    foreach ($property in $typedOutcomes.PSObject.Properties) {
        $artifact[$property.Name] = $property.Value
    }
    [pscustomobject]$artifact | ConvertTo-Json -Depth 6 |
        Set-Content -Path (Join-Path $ProfileDir 'ui_surface_present_outcomes.json') -Encoding UTF8
    Write-Host ("- surface_submitted={0} retryable_no_submit={1} retention_sources={2} retention_complete={3} counter_overwritten={4} input_outcomes={5} damaged_outcomes={6} present_batches={7} typed_complete={8} input_to_damage_samples={9} input_to_damage_p95_us={10} damage_to_submit_samples={11} damage_to_submit_p95_us={12}" -f `
            $submittedCount, $retryableNoSubmitCount, $retention.retention_source_count,
            $retention.retention_complete, $retention.counter_overwritten,
            $typedOutcomes.input_outcome_count, $typedOutcomes.damaged_input_outcome_count,
            $typedOutcomes.present_batch_count, $typedOutcomes.typed_input_outcome_complete,
            $inputToDamageStats.sample_count, $inputToDamageStats.p95_us,
            $damageToSubmitStats.sample_count, $damageToSubmitStats.p95_us)
}

function Test-ZirconUiSurfaceLatencyEvidenceGate {
    param(
        [string]$ProfileDir,
        [string]$ScenarioName,
        [string]$InteractionScenarioName,
        [int]$AutoClickCount,
        [int]$AutoPointerMoveCount,
        [int]$AutoWheelCount
    )

    $normalizedScenario = $ScenarioName.Trim().ToLowerInvariant()
    $requiresClickLatency = $InteractionScenarioName -eq 'click' -and $AutoClickCount -gt 0
    $requiresPointerLatency = $normalizedScenario -in @('idle_hover', 'material_lab_hover') -and
        $AutoPointerMoveCount -gt 0
    $requiresWheelLatency = $normalizedScenario -in @('hierarchy_scroll', 'welcome_recent_scroll') -and
        $AutoWheelCount -gt 0
    if (-not $requiresClickLatency -and -not $requiresPointerLatency -and -not $requiresWheelLatency) {
        return $true
    }

    $artifactPath = Join-Path $ProfileDir 'ui_surface_present_outcomes.json'
    if (-not (Test-Path $artifactPath)) {
        Write-Warning 'Interaction latency gate requires ui_surface_present_outcomes.json.'
        return $false
    }
    $artifact = Get-Content -Path $artifactPath -Raw | ConvertFrom-Json
    if ([int]$artifact.schema_version -lt 5) {
        Write-Warning 'Interaction latency gate requires surface outcome schema 5 with typed input outcomes.'
        return $false
    }
    if ([int64]$artifact.retention_source_count -le 0 -or
        -not [bool]$artifact.retention_complete -or
        [int64]$artifact.frame_overwritten -ne 0 -or
        [int64]$artifact.span_overwritten -ne 0 -or
        [int64]$artifact.counter_overwritten -ne 0) {
        Write-Warning 'Interaction latency gate requires complete recorder retention evidence with zero overwritten samples.'
        return $false
    }
    foreach ($field in @(
            'input_outcome_count',
            'damaged_input_outcome_count',
            'intentionally_no_damage_input_outcome_count',
            'rejected_input_outcome_count',
            'present_batch_count',
            'present_batch_damaged_count',
            'typed_input_outcome_complete'
        )) {
        if ($null -eq $artifact.PSObject.Properties[$field]) {
            Write-Warning "Interaction latency gate is missing typed input field '$field'."
            return $false
        }
    }
    if (-not [bool]$artifact.typed_input_outcome_complete -or
        [int64]$artifact.input_outcome_count -le 0 -or
        [int64]$artifact.damaged_input_outcome_count -le 0 -or
        [int64]$artifact.present_batch_count -le 0 -or
        [int64]$artifact.present_batch_damaged_count -ne [int64]$artifact.damaged_input_outcome_count) {
        Write-Warning 'Interaction latency gate requires complete typed outcomes and exact damaged-to-present membership.'
        return $false
    }

    foreach ($stage in @('input_to_damage', 'damage_to_submit')) {
        $sampleCountProperty = $artifact.PSObject.Properties["${stage}_sample_count"]
        $p50Property = $artifact.PSObject.Properties["${stage}_p50_us"]
        $p95Property = $artifact.PSObject.Properties["${stage}_p95_us"]
        $p99Property = $artifact.PSObject.Properties["${stage}_p99_us"]
        $maxProperty = $artifact.PSObject.Properties["${stage}_max_us"]
        if ($null -eq $sampleCountProperty -or $null -eq $p50Property -or
            $null -eq $p95Property -or $null -eq $p99Property -or $null -eq $maxProperty) {
            Write-Warning "Interaction latency gate is missing the $stage percentile summary."
            return $false
        }
        $sampleCount = [int64]$sampleCountProperty.Value
        $p50 = [double]$p50Property.Value
        $p95 = [double]$p95Property.Value
        $p99 = [double]$p99Property.Value
        $max = [double]$maxProperty.Value
        $invalidNumbers = @(
            @($p50, $p95, $p99, $max) | Where-Object {
                [double]::IsNaN($_) -or [double]::IsInfinity($_) -or $_ -lt 0.0
            }
        )
        if ($sampleCount -le 0 -or $invalidNumbers.Count -gt 0 -or
            $p50 -gt $p95 -or $p95 -gt $p99 -or $p99 -gt $max) {
            Write-Warning "Interaction latency gate rejected the $stage sample count or percentile order."
            return $false
        }
    }
    if ([int64]$artifact.input_to_damage_sample_count -ne
            [int64]$artifact.damaged_input_outcome_count -or
        [int64]$artifact.damage_to_submit_sample_count -ne
            [int64]$artifact.present_batch_count) {
        Write-Warning 'Interaction latency gate requires one input latency per damaged outcome and one submit latency per present batch.'
        return $false
    }
    if ([double]$artifact.input_to_damage_p95_us -gt $script:ZirconInputToDamageP95BudgetUs -or
        [double]$artifact.damage_to_submit_p95_us -gt $script:ZirconDamageToSubmitP95BudgetUs) {
        Write-Warning ("Interaction latency p95 exceeded budgets: input_to_damage={0}/{1}us damage_to_submit={2}/{3}us." -f `
                $artifact.input_to_damage_p95_us, $script:ZirconInputToDamageP95BudgetUs,
                $artifact.damage_to_submit_p95_us, $script:ZirconDamageToSubmitP95BudgetUs)
        return $false
    }

    Write-Host ("- latency_gate input_to_damage_p95_us={0}/{1} damage_to_submit_p95_us={2}/{3}" -f `
            $artifact.input_to_damage_p95_us, $script:ZirconInputToDamageP95BudgetUs,
            $artifact.damage_to_submit_p95_us, $script:ZirconDamageToSubmitP95BudgetUs)
    return $true
}
