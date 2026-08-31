$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$contextModule = Join-Path $repoRoot 'tools\mvp\MvpProcessQualificationContext.psm1'
$registryModule = Join-Path $repoRoot 'tools\mvp\MvpScenarioRegistry.psm1'
$environmentPolicyModule = Join-Path $repoRoot 'tools\mvp\MvpProcessEnvironmentPolicy.psm1'
$artifactBudgetModule = Join-Path $repoRoot 'tools\mvp\MvpRunArtifactBudget.psm1'
$supervisorModule = Join-Path $repoRoot 'tools\mvp\StagedProcessSupervisor.psm1'
$journalModule = Join-Path $repoRoot 'tools\mvp\MvpProcessLifecycleJournal.psm1'
$stageScript = Join-Path $repoRoot 'tools\mvp\Stage-MvpProducts.ps1'
$registryPath = Join-Path $repoRoot 'tools\mvp\mvp-scenario-registry.json'

Import-Module $registryModule -Force -ErrorAction Stop
Import-Module $contextModule -Force -ErrorAction Stop
Import-Module $environmentPolicyModule -Force -ErrorAction Stop
Import-Module $artifactBudgetModule -Force -ErrorAction Stop
Import-Module $supervisorModule -Force -ErrorAction Stop

$registry = Read-MvpScenarioRegistry -Path $registryPath
$registryReceipt = Get-MvpScenarioRegistryReceipt -Registry $registry
$runtimeScenario = Get-MvpScenarioRegistration -Registry $registry -ScenarioId 'mvp.runtime-first-frame.v1'
$editorScenario = Get-MvpScenarioRegistration -Registry $registry -ScenarioId 'mvp.editor-first-frame.v1'

function New-MvpTestQualificationContext {
    param(
        [string]$RunId = 'qualification-fixture',
        [string[]]$ProductReceiptIds = @(),
        $ScenarioRegistration = $runtimeScenario
    )

    return New-MvpProcessQualificationContext `
        -RunId $RunId `
        -SourceFingerprint ('A' * 64) `
        -BuildSetId ('B' * 64) `
        -ScenarioRegistryReceipt $registryReceipt `
        -ScenarioRegistration $ScenarioRegistration `
        -ScenarioVariant 'host.default' `
        -ProductReceiptIds $ProductReceiptIds
}

Describe 'MVP process qualification context' {
    It 'creates one exact unqualified context bound to build and scenario evidence' {
        $context = New-MvpTestQualificationContext

        (@($context.PSObject.Properties.Name) -join ',') | Should Be 'schema_version,context_kind,context_id,qualification_status,run_id,source_fingerprint,build_set_id,scenario_registry_id,scenario_registry_sha256,scenario_id,scenario_variant,product_receipt_ids'
        $context.schema_version | Should Be 1
        $context.context_kind | Should Be 'zircon.mvp-process-qualification-context'
        $context.context_id | Should Match '^[0-9a-f]{64}$'
        $context.qualification_status | Should Be 'unqualified_missing_product_receipt'
        $context.run_id | Should Be 'qualification-fixture'
        $context.source_fingerprint | Should Be ('A' * 64)
        $context.build_set_id | Should Be ('B' * 64)
        $context.scenario_registry_id | Should Be 'mvp.core-scenarios.v1'
        $context.scenario_registry_sha256 | Should Be $registryReceipt.sha256
        $context.scenario_id | Should Be 'mvp.runtime-first-frame.v1'
        $context.scenario_variant | Should Be 'host.default'
        @($context.product_receipt_ids).Count | Should Be 0
    }

    It 'keeps receipt-bound contexts pending until an observation authority qualifies them' {
        $context = New-MvpTestQualificationContext -ProductReceiptIds @(('C' * 64), ('D' * 64))

        $context.qualification_status | Should Be 'pending_observation'
        (@($context.product_receipt_ids) -join ',') | Should Be ((('C' * 64) + ',' + ('D' * 64)))
        ($null -eq $context.PSObject.Properties['qualified']) | Should Be $true
        ($null -eq $context.PSObject.Properties['qualification_passed']) | Should Be $true
    }

    It 'rejects invalid variants and duplicate product receipt identities' {
        { New-MvpProcessQualificationContext `
                -RunId 'qualification-fixture' `
                -SourceFingerprint ('A' * 64) `
                -BuildSetId ('B' * 64) `
                -ScenarioRegistryReceipt $registryReceipt `
                -ScenarioRegistration $runtimeScenario `
                -ScenarioVariant 'device.unregistered' } | Should Throw 'not registered'

        { New-MvpTestQualificationContext -ProductReceiptIds @(('C' * 64), ('c' * 64)) } |
            Should Throw 'duplicate ProductReceipt ID'
    }

    It 'creates one canonical context-set receipt without inventing qualification' {
        $runtimeContext = New-MvpTestQualificationContext
        $editorContext = New-MvpTestQualificationContext -ScenarioRegistration $editorScenario

        $receipt = Get-MvpProcessQualificationContextSetReceipt `
            -Contexts @($runtimeContext, $editorContext) `
            -ExpectedRunId 'qualification-fixture'

        (@($receipt.PSObject.Properties.Name) -join ',') | Should Be 'schema_version,receipt_kind,run_id,context_count,qualification_status,entries,sha256'
        $receipt.schema_version | Should Be 1
        $receipt.receipt_kind | Should Be 'zircon.mvp-process-qualification-context-set'
        $receipt.run_id | Should Be 'qualification-fixture'
        $receipt.context_count | Should Be 2
        $receipt.qualification_status | Should Be 'unqualified_missing_product_receipt'
        @($receipt.entries).Count | Should Be 2
        $receipt.sha256 | Should Match '^[0-9a-f]{64}$'
        ($null -eq $receipt.PSObject.Properties['qualified']) | Should Be $true
    }

    It 'rejects duplicate or mixed-run context sets' {
        $context = New-MvpTestQualificationContext
        { Get-MvpProcessQualificationContextSetReceipt `
                -Contexts @($context, $context) `
                -ExpectedRunId 'qualification-fixture' } | Should Throw 'duplicate context_id'

        $otherRunContext = New-MvpTestQualificationContext -RunId 'qualification-other-run'
        { Get-MvpProcessQualificationContextSetReceipt `
                -Contexts @($context, $otherRunContext) `
                -ExpectedRunId 'qualification-fixture' } | Should Throw 'differs from expected'
    }

    It 'requires Stage to bind all five registered scenarios into supervised launches' {
        $stageSource = Get-Content -LiteralPath $stageScript -Raw

        foreach ($name in @('runtime', 'editor', 'create', 'authoring', 'reopen')) {
            $stageSource | Should Match (('{0}QualificationContext\s+=\s+New-MvpProcessQualificationContext' -f $name))
        }
        $stageSource | Should Match '\[Parameter\(Mandatory\)\]\$QualificationContext'
        ([regex]::Matches($stageSource, '-QualificationContext\s+\$').Count -ge 3) | Should Be $true
    }

    It 'journals bounded raw arguments once and binds every event to the context id' {
        $state = $null
        try {
            $environmentPolicy = New-MvpProcessEnvironmentPolicy `
                -PolicyId 'test.qualification-context.v1' `
                -InheritedNames @('ComSpec', 'PATH', 'PATHEXT', 'SystemRoot', 'TEMP', 'TMP', 'WINDIR') `
                -DeclaredNames @()
            $context = New-MvpTestQualificationContext
            $startInfo = [Diagnostics.ProcessStartInfo]::new()
            $startInfo.FileName = $env:ComSpec
            $startInfo.Arguments = '/d /s /c "echo qualification-context-fixture"'
            $startInfo.WorkingDirectory = $TestDrive
            $startInfo.UseShellExecute = $false
            $startInfo.RedirectStandardOutput = $true
            $startInfo.RedirectStandardError = $true

            $state = Start-MvpSupervisedProcess `
                -StartInfo $startInfo `
                -StageRoot $TestDrive `
                -RunId 'qualification-fixture' `
                -Phase 'qualification-context' `
                -StdoutPath (Join-Path $TestDrive 'qualification.stdout.log') `
                -StderrPath (Join-Path $TestDrive 'qualification.stderr.log') `
                -MaximumRetainedLogBytes 1024 `
                -EnvironmentPolicy $environmentPolicy `
                -QualificationContext $context `
                -HeartbeatIntervalMilliseconds 50
            @(Complete-MvpSupervisedProcess -ProcessState $state -TimeoutSeconds 10) | Should Be 0

            $events = @(Get-Content -LiteralPath (Join-Path $TestDrive 'logs\process-execution-journal.jsonl') |
                    ForEach-Object { $_ | ConvertFrom-Json } |
                    Where-Object { $_.phase -eq 'qualification-context' } |
                    Sort-Object { [int]$_.sequence })
            $events.Count | Should BeGreaterThan 2
            $events | ForEach-Object { $_.qualification_context_id | Should Be $context.context_id }
            $started = @($events | Where-Object { $_.event_kind -eq 'started' })
            $started.Count | Should Be 1
            $started[0].arguments | Should Be $startInfo.Arguments
            $started[0].qualification_context.scenario_id | Should Be 'mvp.runtime-first-frame.v1'
            @($events | Where-Object { $_.event_kind -ne 'started' -and $null -ne $_.PSObject.Properties['arguments'] }).Count | Should Be 0

            $supervisorSource = Get-Content -LiteralPath $supervisorModule -Raw
            $journalSource = Get-Content -LiteralPath $journalModule -Raw
            $supervisorSource | Should Match 'MvpProcessQualificationContext\.psm1'
            $journalSource | Should Match 'if \(\$EventKind -eq ''started''\)'
        }
        finally {
            if ($null -ne $state) {
                Close-MvpSupervisedProcessState -ProcessState $state
            }
        }
    }
}
