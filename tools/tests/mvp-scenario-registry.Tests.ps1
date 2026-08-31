$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$registryModulePath = Join-Path $repoRoot 'tools\mvp\MvpScenarioRegistry.psm1'
$automationModulePath = Join-Path $repoRoot 'tools\mvp\MvpAutomationScenarioSpec.psm1'
$registryPath = Join-Path $repoRoot 'tools\mvp\mvp-scenario-registry.json'
$stagePath = Join-Path $repoRoot 'tools\mvp\Stage-MvpProducts.ps1'

if (Test-Path -LiteralPath $registryModulePath) {
    Import-Module $registryModulePath -Force -ErrorAction Stop
}
Import-Module $automationModulePath -Force -ErrorAction Stop

Describe 'MVP scenario registry' {
    It 'publishes five versioned scenarios with stable capability ownership' {
        Test-Path -LiteralPath $registryModulePath | Should Be $true
        Test-Path -LiteralPath $registryPath | Should Be $true
        $registry = Read-MvpScenarioRegistry -Path $registryPath
        $receipt = Get-MvpScenarioRegistryReceipt -Registry $registry

        $registry.schema_version | Should Be 1
        $registry.registry_kind | Should Be 'zircon.mvp-scenario-registry'
        $registry.registry_id | Should Be 'mvp.core-scenarios.v1'
        @($registry.scenarios).Count | Should Be 5
        $receipt.sha256 | Should Match '^[0-9a-f]{64}$'
        $receipt.scenario_count | Should Be 5
        (@($receipt.PSObject.Properties.Name) -contains 'scenarios') | Should Be $false

        $authoring = Get-MvpScenarioRegistration `
            -Registry $registry `
            -ScenarioId 'mvp.editor-authoring.v1'
        $authoring.capability_id | Should Be 'editor.scene.authoring'
        $authoring.owner | Should Be 'zircon_tooling.mvp'
        @($authoring.roles).Count | Should Be 1
        $authoring.roles[0] | Should Be 'editor'
        $authoring.liveness_scenario | Should Be 'editor_authoring'
        @($authoring.steps).Count | Should Be 6
        $authoring.variants[0] | Should Be 'host.default'
    }

    It 'binds automation registrations to their versioned request specs' {
        $registry = Read-MvpScenarioRegistry -Path $registryPath
        foreach ($scenarioId in @('mvp.editor-authoring.v1', 'mvp.editor-reopen.v1')) {
            $registration = Get-MvpScenarioRegistration -Registry $registry -ScenarioId $scenarioId
            $scenarioPath = Join-Path (Split-Path $registryPath -Parent) $registration.automation_request
            $scenario = Assert-MvpAutomationScenarioSpec `
                -Path $scenarioPath `
                -ExpectedScenarioId $registration.scenario_id

            $scenario.binding_count | Should Be @($registration.steps).Count
        }
    }

    It 'resolves per-scenario attempts and step-derived device timeout budgets' {
        $registry = Read-MvpScenarioRegistry -Path $registryPath
        $runtime = Get-MvpScenarioRegistration `
            -Registry $registry `
            -ScenarioId 'mvp.runtime-first-frame.v1'
        $policy = Resolve-MvpScenarioExecutionPolicy `
            -ScenarioRegistration $runtime `
            -ScenarioVariant 'host.default'

        $policy.scenario_id | Should Be 'mvp.runtime-first-frame.v1'
        $policy.scenario_variant | Should Be 'host.default'
        $policy.attempt_count | Should Be 2
        $policy.process_timeout_seconds | Should Be 90
        $policy.progress_inactivity_timeout_seconds | Should Be 45
        @($policy.step_timeouts).Count | Should Be @($runtime.steps).Count

        $bounded = Resolve-MvpScenarioExecutionPolicy `
            -ScenarioRegistration $runtime `
            -ScenarioVariant 'host.default' `
            -RequestedAttemptCount 1 `
            -RequestedTimeoutSeconds 10 `
            -RequestedProgressInactivityTimeoutSeconds 5
        $bounded.attempt_count | Should Be 1
        $bounded.process_timeout_seconds | Should Be 10
        $bounded.progress_inactivity_timeout_seconds | Should Be 5
        $bounded.policy_process_timeout_seconds | Should Be 90
    }

    It 'rejects execution policy step, device, and numeric ambiguity' {
        $unknownStep = Get-Content -LiteralPath $registryPath -Raw | ConvertFrom-Json
        $unknownStep.scenarios[0].execution_policy.step_timeouts[0].step_id = 'unknown_step'
        $unknownStepPath = Join-Path $TestDrive 'unknown-step.json'
        [IO.File]::WriteAllText(
            $unknownStepPath,
            ($unknownStep | ConvertTo-Json -Depth 16),
            [Text.UTF8Encoding]::new($false)
        )
        { Read-MvpScenarioRegistry -Path $unknownStepPath } | Should Throw 'step_timeouts must match scenario steps'

        $unknownDevice = Get-Content -LiteralPath $registryPath -Raw | ConvertFrom-Json
        $unknownDevice.scenarios[0].execution_policy.device_class = 'host.unregistered'
        $unknownDevicePath = Join-Path $TestDrive 'unknown-device.json'
        [IO.File]::WriteAllText(
            $unknownDevicePath,
            ($unknownDevice | ConvertTo-Json -Depth 16),
            [Text.UTF8Encoding]::new($false)
        )
        { Read-MvpScenarioRegistry -Path $unknownDevicePath } | Should Throw 'device_class'

        $booleanAttempt = Get-Content -LiteralPath $registryPath -Raw | ConvertFrom-Json
        $booleanAttempt.scenarios[0].execution_policy.attempts.default = $true
        $booleanAttemptPath = Join-Path $TestDrive 'boolean-attempt.json'
        [IO.File]::WriteAllText(
            $booleanAttemptPath,
            ($booleanAttempt | ConvertTo-Json -Depth 16),
            [Text.UTF8Encoding]::new($false)
        )
        { Read-MvpScenarioRegistry -Path $booleanAttemptPath } | Should Throw 'attempts.default must be a JSON integer'
    }

    It 'rejects unknown scenario fields and duplicate identities' {
        $source = Get-Content -LiteralPath $registryPath -Raw | ConvertFrom-Json
        $source.scenarios[0] | Add-Member -NotePropertyName 'unreviewed' -NotePropertyValue $true
        $unknownPath = Join-Path $TestDrive 'unknown.json'
        [IO.File]::WriteAllText(
            $unknownPath,
            ($source | ConvertTo-Json -Depth 16),
            [Text.UTF8Encoding]::new($false)
        )
        { Read-MvpScenarioRegistry -Path $unknownPath } | Should Throw 'unknown property'

        $duplicate = Get-Content -LiteralPath $registryPath -Raw | ConvertFrom-Json
        $duplicate.scenarios[1].scenario_id = $duplicate.scenarios[0].scenario_id
        $duplicatePath = Join-Path $TestDrive 'duplicate.json'
        [IO.File]::WriteAllText(
            $duplicatePath,
            ($duplicate | ConvertTo-Json -Depth 16),
            [Text.UTF8Encoding]::new($false)
        )
        { Read-MvpScenarioRegistry -Path $duplicatePath } | Should Throw 'duplicate scenario_id'
    }

    It 'rejects unknown scenario lookups' {
        $registry = Read-MvpScenarioRegistry -Path $registryPath
        { Get-MvpScenarioRegistration -Registry $registry -ScenarioId 'mvp.unknown.v1' } |
            Should Throw 'is not registered'
    }

    It 'requires Stage to validate and receipt the registry before input publication' {
        $source = [IO.File]::ReadAllText($stagePath)
        $registryRead = $source.IndexOf('Read-MvpScenarioRegistry')
        $scenarioValidation = $source.IndexOf('Assert-MvpAutomationScenarioSpec')
        $inputPublication = $source.IndexOf("-LogicalId 'product-input-manifest'")

        $source | Should Match 'MvpScenarioRegistry\.psm1'
        ($registryRead -ge 0 -and $registryRead -lt $scenarioValidation) | Should Be $true
        ($scenarioValidation -ge 0 -and $scenarioValidation -lt $inputPublication) | Should Be $true
        $source | Should Match 'Get-MvpScenarioRegistryReceipt -Registry \$scenarioRegistry'
        $source | Should Match 'scenario_registry = \$scenarioRegistryReceipt'
    }

    It 'requires Stage to consume resolved scenario execution policies' {
        $source = [IO.File]::ReadAllText($stagePath)

        ([regex]::Matches($source, 'Resolve-MvpScenarioExecutionPolicy').Count -ge 5) | Should Be $true
        $source | Should Match '-ExecutionPolicy \$[A-Za-z]+ExecutionPolicy'
        $source | Should Not Match '-TimeoutSeconds \$TimeoutSeconds'
        $source | Should Not Match '-ProgressInactivityTimeoutSeconds \$ProgressInactivityTimeoutSeconds'
        $source | Should Match 'scenario_execution_policies = \$scenarioExecutionPolicyReceipts'
    }
}
