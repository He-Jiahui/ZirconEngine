$script:ScenarioModule = Join-Path $PSScriptRoot '..\mvp\RenderExtractPerformanceScenario.psm1'

Describe 'render-extract performance scenario contract' {
    BeforeAll {
        Import-Module $script:ScenarioModule -Force -ErrorAction Stop
    }

    AfterAll {
        Remove-Module RenderExtractPerformanceScenario -ErrorAction SilentlyContinue
    }

    It 'publishes four unique versioned scenarios with explicit cache metrics and budgets' {
        $definitions = @(Get-RenderExtractPerformanceScenarioDefinitions)

        $definitions.Count | Should Be 4
        @($definitions.scenario_id | Select-Object -Unique).Count | Should Be 4
        @($definitions.logical_id) | Should Be @(
            'pipelined-first-frame',
            'pipelined-steady',
            'synchronous-steady',
            'editor-first-frame'
        )
        foreach ($definition in $definitions) {
            $definition.schema_version | Should Be 1
            $definition.scenario_kind | Should Be 'zircon_render_extract_performance_scenario'
            $definition.scenario_id | Should Match '^render-extract\.[a-z0-9.-]+$'
            $definition.scenario_version | Should Be 1
            $definition.cache_contract.process | Should Be 'fresh-process-per-attempt'
            $definition.cache_contract.strict_cold_eligible | Should Be $false
            @($definition.required_metrics).Count | Should BeGreaterThan 0
            $definition.budget_contract.status | Should Match '^(declared|unconfigured)$'
        }

        $steady = @($definitions | Where-Object { $_.logical_id -eq 'pipelined-steady' })[0]
        $steady.budget_contract.status | Should Be 'declared'
        $steady.budget_contract.metric_id | Should Be 'app.runtime_redraw.frame_duration_us'
        $steady.budget_contract.aggregation | Should Be 'p95'
        $steady.budget_contract.comparator | Should Be 'less_than_or_equal'
        $steady.budget_contract.threshold | Should Be 16670
        $steady.budget_contract.unit | Should Be 'us'
    }

    It 'binds each run to the exact repeat and measurement parameters' {
        $runs = @(Get-RenderExtractBaselineRunPlan `
                -RepeatCount 3 `
                -WarmupPresentedFrameCount 60 `
                -MeasuredPresentedFrameCount 300)

        $runs.Count | Should Be 4
        foreach ($run in $runs) {
            $run.scenario_id | Should Match '^render-extract\.'
            $run.scenario_version | Should Be 1
            $run.scenario_binding_id | Should Match '^[0-9A-F]{64}$'
            $run.repeat_count | Should Be 3
            $run.cache_contract.process | Should Be 'fresh-process-per-attempt'
            @($run.required_metrics).Count | Should BeGreaterThan 0
        }
        $runs[0].warmup_presented_frame_count | Should Be 0
        $runs[0].measured_presented_frame_count | Should Be 1
        $runs[1].warmup_presented_frame_count | Should Be 60
        $runs[1].measured_presented_frame_count | Should Be 300
        $runs[1].target_presented_frame_count | Should Be 360
    }

    It 'derives deterministic binding identities without changing unrelated first-frame bindings' {
        $first = @(Get-RenderExtractBaselineRunPlan `
                -RepeatCount 3 `
                -WarmupPresentedFrameCount 60 `
                -MeasuredPresentedFrameCount 300)
        $same = @(Get-RenderExtractBaselineRunPlan `
                -RepeatCount 3 `
                -WarmupPresentedFrameCount 60 `
                -MeasuredPresentedFrameCount 300)
        $differentWindow = @(Get-RenderExtractBaselineRunPlan `
                -RepeatCount 3 `
                -WarmupPresentedFrameCount 61 `
                -MeasuredPresentedFrameCount 300)
        $differentRepeats = @(Get-RenderExtractBaselineRunPlan `
                -RepeatCount 4 `
                -WarmupPresentedFrameCount 60 `
                -MeasuredPresentedFrameCount 300)

        @($first.scenario_binding_id) | Should Be @($same.scenario_binding_id)
        $first[0].scenario_binding_id | Should Be $differentWindow[0].scenario_binding_id
        $first[1].scenario_binding_id | Should Not Be $differentWindow[1].scenario_binding_id
        $first[0].scenario_binding_id | Should Not Be $differentRepeats[0].scenario_binding_id
    }

    It 're-derives a serialized run binding and rejects a stale binding after parameter tampering' {
        $run = @(Get-RenderExtractBaselineRunPlan `
                -RepeatCount 3 `
                -WarmupPresentedFrameCount 60 `
                -MeasuredPresentedFrameCount 300)[1]
        $serialized = ($run | ConvertTo-Json -Depth 10) | ConvertFrom-Json

        $resolved = Resolve-RenderExtractPerformanceScenarioRunBinding -Run $serialized
        $resolved.scenario_binding_id | Should Be $run.scenario_binding_id
        $resolved.measurement_window | Should Be 'steady-presented-frames-after-warmup'

        $serialized.warmup_presented_frame_count = 61
        $serialized.target_presented_frame_count = 361
        {
            Resolve-RenderExtractPerformanceScenarioRunBinding -Run $serialized
        } | Should Throw 'scenario_binding_id does not match'
    }

    It 'requires capture and report publication to preserve and verify the scenario binding' {
        $captureSource = Get-Content (Join-Path $PSScriptRoot '..\mvp\Capture-RenderExtractBaseline.ps1') -Raw
        $reportSource = Get-Content (Join-Path $PSScriptRoot '..\mvp\Write-RenderExtractBaselineReport.ps1') -Raw

        $captureSource | Should Match 'scenario_binding_id = \$Run\.scenario_binding_id'
        $captureSource | Should Match 'repeat_count = \$Run\.repeat_count'
        $reportSource | Should Match 'Resolve-RenderExtractPerformanceScenarioRunBinding'
        $reportSource | Should Match 'scenario_binding_id = \$scenarioBinding\.scenario_binding_id'
        $reportSource | Should Match 'budget_contract = \$scenarioBinding\.budget_contract'
    }
}
