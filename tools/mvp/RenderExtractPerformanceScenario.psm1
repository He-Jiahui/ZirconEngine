Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:RenderExtractScenarioSchemaVersion = 1
$script:RenderExtractScenarioKind = 'zircon_render_extract_performance_scenario'
$script:RenderExtractScenarioHexDigits = [char[]]'0123456789ABCDEF'

function New-RenderExtractScenarioBudgetContract {
    param([switch]$SteadyFrameBudget)

    if (-not $SteadyFrameBudget) {
        return [pscustomobject][ordered]@{
            status = 'unconfigured'
            reason = 'No accepted first-frame product budget is registered.'
        }
    }
    return [pscustomobject][ordered]@{
        status = 'declared'
        metric_id = 'app.runtime_redraw.frame_duration_us'
        aggregation = 'p95'
        comparator = 'less_than_or_equal'
        threshold = 16670
        unit = 'us'
    }
}

function New-RenderExtractScenarioCacheContract {
    return [pscustomobject][ordered]@{
        process = 'fresh-process-per-attempt'
        project_state = 'preserved-across-attempts'
        os_file_cache = 'uncontrolled'
        gpu_driver_cache = 'uncontrolled'
        strict_cold_eligible = $false
        comparison_scope = 'process-and-caller-owned-project-state'
    }
}

function New-RenderExtractPerformanceScenarioDefinition {
    param(
        [Parameter(Mandatory)][string]$LogicalId,
        [Parameter(Mandatory)][string]$ScenarioId,
        [Parameter(Mandatory)][ValidateSet('runtime', 'editor')][string]$Product,
        [Parameter(Mandatory)][string]$RuntimeProfile,
        [Parameter(Mandatory)][string]$MeasurementWindow,
        [Parameter(Mandatory)][bool]$ExitAfterFirstFrame,
        [Parameter(Mandatory)][object[]]$RequiredMetrics,
        [switch]$SteadyFrameBudget
    )

    return [pscustomobject][ordered]@{
        schema_version = $script:RenderExtractScenarioSchemaVersion
        scenario_kind = $script:RenderExtractScenarioKind
        logical_id = $LogicalId
        scenario_id = $ScenarioId
        scenario_version = 1
        product = $Product
        runtime_profile = $RuntimeProfile
        measurement_window = $MeasurementWindow
        exit_after_first_frame = $ExitAfterFirstFrame
        primary_frame_stream = 'app'
        primary_frame_name = 'runtime_redraw'
        cache_contract = New-RenderExtractScenarioCacheContract
        required_metrics = @($RequiredMetrics)
        budget_contract = New-RenderExtractScenarioBudgetContract -SteadyFrameBudget:$SteadyFrameBudget
    }
}

function Get-RenderExtractPerformanceScenarioDefinitions {
    $firstFrameMetrics = @(
        [pscustomobject][ordered]@{
            metric_id = 'product.process_elapsed_ms'
            unit = 'ms'
            direction = 'lower_is_better'
        },
        [pscustomobject][ordered]@{
            metric_id = 'app.runtime_redraw.frame_duration_us'
            unit = 'us'
            direction = 'lower_is_better'
        }
    )
    $steadyMetrics = @(
        [pscustomobject][ordered]@{
            metric_id = 'app.runtime_redraw.frame_duration_us'
            unit = 'us'
            direction = 'lower_is_better'
        },
        [pscustomobject][ordered]@{
            metric_id = 'product.total_processor_time_ms'
            unit = 'ms'
            direction = 'lower_is_better'
        },
        [pscustomobject][ordered]@{
            metric_id = 'product.peak_working_set_bytes'
            unit = 'bytes'
            direction = 'lower_is_better'
        }
    )

    return @(
        New-RenderExtractPerformanceScenarioDefinition `
            -LogicalId 'pipelined-first-frame' `
            -ScenarioId 'render-extract.runtime.pipelined-first-frame' `
            -Product 'runtime' `
            -RuntimeProfile 'runtime-pipelined' `
            -MeasurementWindow 'cold-first-presented-frame' `
            -ExitAfterFirstFrame $true `
            -RequiredMetrics $firstFrameMetrics
        New-RenderExtractPerformanceScenarioDefinition `
            -LogicalId 'pipelined-steady' `
            -ScenarioId 'render-extract.runtime.pipelined-steady' `
            -Product 'runtime' `
            -RuntimeProfile 'runtime-pipelined' `
            -MeasurementWindow 'steady-presented-frames-after-warmup' `
            -ExitAfterFirstFrame $false `
            -RequiredMetrics $steadyMetrics `
            -SteadyFrameBudget
        New-RenderExtractPerformanceScenarioDefinition `
            -LogicalId 'synchronous-steady' `
            -ScenarioId 'render-extract.runtime.synchronous-steady' `
            -Product 'runtime' `
            -RuntimeProfile 'runtime' `
            -MeasurementWindow 'steady-presented-frames-after-warmup' `
            -ExitAfterFirstFrame $false `
            -RequiredMetrics $steadyMetrics `
            -SteadyFrameBudget
        New-RenderExtractPerformanceScenarioDefinition `
            -LogicalId 'editor-first-frame' `
            -ScenarioId 'render-extract.editor.first-frame' `
            -Product 'editor' `
            -RuntimeProfile 'editor' `
            -MeasurementWindow 'cold-first-presented-frame' `
            -ExitAfterFirstFrame $true `
            -RequiredMetrics $firstFrameMetrics
    )
}

function Get-RenderExtractScenarioBindingSha256 {
    param([Parameter(Mandatory)]$Value)

    $json = $Value | ConvertTo-Json -Depth 10 -Compress
    $bytes = ([Text.UTF8Encoding]::new($false)).GetBytes($json)
    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        [byte[]]$hash = $hasher.ComputeHash($bytes)
    }
    finally {
        $hasher.Dispose()
    }
    [char[]]$characters = [char[]]::new($hash.Length * 2)
    for ($index = 0; $index -lt $hash.Length; $index++) {
        $value = [int]$hash[$index]
        $characters[$index * 2] = $script:RenderExtractScenarioHexDigits[$value -shr 4]
        $characters[($index * 2) + 1] = $script:RenderExtractScenarioHexDigits[$value -band 0x0F]
    }
    return [string]::new($characters)
}

function New-RenderExtractPerformanceScenarioRun {
    param(
        [Parameter(Mandatory)]$Definition,
        [Parameter(Mandatory)][ValidateRange(3, 20)][int]$RepeatCount,
        [Parameter(Mandatory)][ValidateRange(0, 1000000)][int]$WarmupPresentedFrameCount,
        [Parameter(Mandatory)][ValidateRange(1, 1000000)][int]$MeasuredPresentedFrameCount
    )

    $targetPresentedFrameCount = $WarmupPresentedFrameCount + $MeasuredPresentedFrameCount
    $binding = [ordered]@{
        schema_version = $script:RenderExtractScenarioSchemaVersion
        scenario_kind = $script:RenderExtractScenarioKind
        scenario_id = $Definition.scenario_id
        scenario_version = $Definition.scenario_version
        product = $Definition.product
        runtime_profile = $Definition.runtime_profile
        measurement_window = $Definition.measurement_window
        exit_after_first_frame = $Definition.exit_after_first_frame
        repeat_count = $RepeatCount
        measurement = [ordered]@{
            primary_frame_stream = $Definition.primary_frame_stream
            primary_frame_name = $Definition.primary_frame_name
            warmup_presented_frame_count = $WarmupPresentedFrameCount
            measured_presented_frame_count = $MeasuredPresentedFrameCount
            target_presented_frame_count = $targetPresentedFrameCount
        }
        cache_contract = $Definition.cache_contract
        required_metrics = @($Definition.required_metrics)
        budget_contract = $Definition.budget_contract
    }

    return [pscustomobject][ordered]@{
        logical_id = $Definition.logical_id
        scenario_id = $Definition.scenario_id
        scenario_version = $Definition.scenario_version
        scenario_binding_id = Get-RenderExtractScenarioBindingSha256 -Value $binding
        product = $Definition.product
        runtime_profile = $Definition.runtime_profile
        measurement_window = $Definition.measurement_window
        exit_after_first_frame = $Definition.exit_after_first_frame
        presented_frame_count = if ($Definition.exit_after_first_frame) { $null } else { $targetPresentedFrameCount }
        warmup_presented_frame_count = $WarmupPresentedFrameCount
        measured_presented_frame_count = $MeasuredPresentedFrameCount
        target_presented_frame_count = $targetPresentedFrameCount
        repeat_count = $RepeatCount
        primary_frame_stream = $Definition.primary_frame_stream
        primary_frame_name = $Definition.primary_frame_name
        cache_contract = $Definition.cache_contract
        required_metrics = @($Definition.required_metrics)
        budget_contract = $Definition.budget_contract
    }
}

function Get-RenderExtractBaselineRunPlan {
    param(
        [Parameter(Mandatory)][ValidateRange(3, 20)][int]$RepeatCount,
        [Parameter(Mandatory)][ValidateRange(0, 1000000)][int]$WarmupPresentedFrameCount,
        [Parameter(Mandatory)][ValidateRange(1, 1000000)][int]$MeasuredPresentedFrameCount
    )

    $runs = foreach ($definition in @(Get-RenderExtractPerformanceScenarioDefinitions)) {
        $warmup = if ($definition.exit_after_first_frame) { 0 } else { $WarmupPresentedFrameCount }
        $measured = if ($definition.exit_after_first_frame) { 1 } else { $MeasuredPresentedFrameCount }
        New-RenderExtractPerformanceScenarioRun `
            -Definition $definition `
            -RepeatCount $RepeatCount `
            -WarmupPresentedFrameCount $warmup `
            -MeasuredPresentedFrameCount $measured
    }
    return @($runs)
}

function Get-RenderExtractScenarioRunProperty {
    param(
        [Parameter(Mandatory)]$Run,
        [Parameter(Mandatory)][string]$Name
    )

    if ($Run -is [Collections.IDictionary]) {
        if ($Run.Contains($Name)) {
            return $Run[$Name]
        }
    }
    else {
        $property = $Run.PSObject.Properties[$Name]
        if ($null -ne $property) {
            return $property.Value
        }
    }
    throw "Render-extract scenario run is missing '$Name'."
}

function Get-RenderExtractScenarioRunInteger {
    param(
        [Parameter(Mandatory)]$Run,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][int]$Minimum,
        [Parameter(Mandatory)][int]$Maximum
    )

    $value = Get-RenderExtractScenarioRunProperty -Run $Run -Name $Name
    if ($value -isnot [ValueType]) {
        throw "Render-extract scenario run '$Name' must be a JSON integer."
    }
    $number = [double]$value
    if ([double]::IsNaN($number) -or [double]::IsInfinity($number) -or
        $number -ne [Math]::Truncate($number) -or $number -lt $Minimum -or $number -gt $Maximum) {
        throw "Render-extract scenario run '$Name' must be an integer in $Minimum..$Maximum."
    }
    return [int]$number
}

function Assert-RenderExtractScenarioRunValue {
    param(
        [Parameter(Mandatory)]$Run,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)]$Expected
    )

    $actual = Get-RenderExtractScenarioRunProperty -Run $Run -Name $Name
    if ([string]$actual -ne [string]$Expected) {
        throw "Render-extract scenario run '$Name' does not match its registered scenario."
    }
}

function Assert-RenderExtractScenarioRunStructuredValue {
    param(
        [Parameter(Mandatory)]$Run,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)]$Expected
    )

    $actual = Get-RenderExtractScenarioRunProperty -Run $Run -Name $Name
    $actualJson = $actual | ConvertTo-Json -Depth 10 -Compress
    $expectedJson = $Expected | ConvertTo-Json -Depth 10 -Compress
    if ($actualJson -cne $expectedJson) {
        throw "Render-extract scenario run '$Name' does not match its registered scenario binding."
    }
}

function Resolve-RenderExtractPerformanceScenarioRunBinding {
    param([Parameter(Mandatory)]$Run)

    $logicalId = [string](Get-RenderExtractScenarioRunProperty -Run $Run -Name 'logical_id')
    $definitions = @(Get-RenderExtractPerformanceScenarioDefinitions | Where-Object {
            $_.logical_id -ceq $logicalId
        })
    if ($definitions.Count -ne 1) {
        throw "Render-extract scenario run '$logicalId' is not registered."
    }
    $repeatCount = Get-RenderExtractScenarioRunInteger -Run $Run -Name 'repeat_count' -Minimum 3 -Maximum 20
    $warmupCount = Get-RenderExtractScenarioRunInteger `
        -Run $Run `
        -Name 'warmup_presented_frame_count' `
        -Minimum 0 `
        -Maximum 1000000
    $measuredCount = Get-RenderExtractScenarioRunInteger `
        -Run $Run `
        -Name 'measured_presented_frame_count' `
        -Minimum 1 `
        -Maximum 1000000
    $expected = New-RenderExtractPerformanceScenarioRun `
        -Definition $definitions[0] `
        -RepeatCount $repeatCount `
        -WarmupPresentedFrameCount $warmupCount `
        -MeasuredPresentedFrameCount $measuredCount

    foreach ($name in @(
            'scenario_id',
            'scenario_version',
            'product',
            'runtime_profile',
            'measurement_window',
            'target_presented_frame_count'
        )) {
        Assert-RenderExtractScenarioRunValue -Run $Run -Name $name -Expected $expected.$name
    }
    $bindingId = [string](Get-RenderExtractScenarioRunProperty -Run $Run -Name 'scenario_binding_id')
    if ($bindingId -cne $expected.scenario_binding_id) {
        throw "Render-extract scenario run '$logicalId' scenario_binding_id does not match its registered parameters."
    }
    foreach ($name in @('cache_contract', 'required_metrics', 'budget_contract')) {
        Assert-RenderExtractScenarioRunStructuredValue -Run $Run -Name $name -Expected $expected.$name
    }
    return $expected
}

Export-ModuleMember -Function @(
    'Get-RenderExtractPerformanceScenarioDefinitions',
    'Get-RenderExtractBaselineRunPlan',
    'Resolve-RenderExtractPerformanceScenarioRunBinding'
)
