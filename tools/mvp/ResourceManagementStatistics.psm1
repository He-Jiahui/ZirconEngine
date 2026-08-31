Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Get-ResourceManagementStatisticsPercentile {
    param(
        [Parameter(Mandatory)][double[]]$SortedValues,
        [Parameter(Mandatory)][ValidateRange(0.0, 1.0)][double]$Probability
    )

    if ($SortedValues.Count -eq 1) {
        return $SortedValues[0]
    }
    $position = ($SortedValues.Count - 1) * $Probability
    $lowerIndex = [int][Math]::Floor($position)
    $upperIndex = [int][Math]::Ceiling($position)
    if ($lowerIndex -eq $upperIndex) {
        return $SortedValues[$lowerIndex]
    }
    $weight = $position - $lowerIndex
    return $SortedValues[$lowerIndex] + (($SortedValues[$upperIndex] - $SortedValues[$lowerIndex]) * $weight)
}

function Get-ResourceManagementCohortStatistics {
    param(
        [Parameter(Mandatory)][double[]]$Values,
        [Parameter(Mandatory)][ValidateRange(1, [Int32]::MaxValue)][int]$MinimumSampleCount,
        [Parameter(Mandatory)][ValidateRange(0.000001, 1.0)][double]$MaximumCoefficientOfVariation,
        [Parameter(Mandatory)][ValidateRange(0.000001, 1.0)][double]$MaximumRelativeMarginOfError
    )

    if ($Values.Count -eq 0) {
        throw 'Cannot aggregate zero resource-management baseline samples.'
    }
    $rawSamples = [double[]]$Values.Clone()
    $sorted = [double[]]$Values.Clone()
    [Array]::Sort($sorted)

    $mean = 0.0
    $sumOfSquaredDifferences = 0.0
    $sampleIndex = 0
    $total = 0.0
    foreach ($value in $rawSamples) {
        if ([double]::IsNaN($value) -or [double]::IsInfinity($value) -or $value -lt 0) {
            throw 'Resource-management baseline samples must be finite non-negative numbers.'
        }
        $sampleIndex++
        $total += $value
        $delta = $value - $mean
        $mean += $delta / $sampleIndex
        $sumOfSquaredDifferences += $delta * ($value - $mean)
    }

    $sampleStandardDeviation = if ($rawSamples.Count -gt 1) {
        [Math]::Sqrt($sumOfSquaredDifferences / ($rawSamples.Count - 1))
    }
    else {
        0.0
    }
    $median = Get-ResourceManagementStatisticsPercentile -SortedValues $sorted -Probability 0.5
    $absoluteDeviations = [double[]]::new($sorted.Count)
    for ($index = 0; $index -lt $sorted.Count; $index++) {
        $absoluteDeviations[$index] = [Math]::Abs($sorted[$index] - $median)
    }
    [Array]::Sort($absoluteDeviations)
    $medianAbsoluteDeviation = Get-ResourceManagementStatisticsPercentile `
        -SortedValues $absoluteDeviations `
        -Probability 0.5
    $coefficientOfVariation = if ($mean -eq 0.0) { 0.0 } else { $sampleStandardDeviation / $mean }

    # 2.093 is the two-sided 95% Student-t critical value at the minimum admitted df=19.
    # It is conservative for every larger cohort admitted by the current policy.
    $confidenceCriticalValue = 2.093
    $confidenceMargin = if ($rawSamples.Count -gt 1) {
        $confidenceCriticalValue * $sampleStandardDeviation / [Math]::Sqrt($rawSamples.Count)
    }
    else {
        0.0
    }
    $relativeMarginOfError = if ($mean -eq 0.0) { 0.0 } else { $confidenceMargin / $mean }
    $noiseStatus = if ($rawSamples.Count -lt $MinimumSampleCount) {
        'insufficient-samples'
    }
    elseif ($coefficientOfVariation -gt $MaximumCoefficientOfVariation -or
        $relativeMarginOfError -gt $MaximumRelativeMarginOfError) {
        'unstable'
    }
    else {
        'stable'
    }

    return [pscustomobject][ordered]@{
        sample_count = $rawSamples.Count
        raw_samples = $rawSamples
        min = $sorted[0]
        median = $median
        p95 = Get-ResourceManagementStatisticsPercentile -SortedValues $sorted -Probability 0.95
        max = $sorted[$sorted.Count - 1]
        mean = $mean
        sample_standard_deviation = $sampleStandardDeviation
        median_absolute_deviation = $medianAbsoluteDeviation
        coefficient_of_variation = $coefficientOfVariation
        confidence_95_lower = [Math]::Max(0.0, $mean - $confidenceMargin)
        confidence_95_upper = $mean + $confidenceMargin
        relative_margin_of_error = $relativeMarginOfError
        noise_status = $noiseStatus
        total = $total
    }
}

Export-ModuleMember -Function Get-ResourceManagementCohortStatistics
