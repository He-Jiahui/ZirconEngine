$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$receiptModule = Join-Path $repoRoot 'tools\mvp\MvpStagingTerminalReceipt.psm1'
$qualificationContextModule = Join-Path $repoRoot 'tools\mvp\MvpProcessQualificationContext.psm1'
$scenarioRegistryModule = Join-Path $repoRoot 'tools\mvp\MvpScenarioRegistry.psm1'
$storagePolicyModule = Join-Path $repoRoot 'tools\mvp\MvpArtifactStoragePolicy.psm1'
$fixturePathsModule = Join-Path $repoRoot 'tools\mvp\MvpTestFixturePaths.psm1'
$stageScript = Join-Path $repoRoot 'tools\mvp\Stage-MvpProducts.ps1'

Import-Module $fixturePathsModule -Force -ErrorAction Stop
Import-Module $storagePolicyModule -Force -ErrorAction Stop
Import-Module $scenarioRegistryModule -Force -ErrorAction Stop
Import-Module $qualificationContextModule -Force -ErrorAction Stop
Import-Module $receiptModule -Force -ErrorAction Stop

$terminalScenarioRegistry = Read-MvpScenarioRegistry -Path (Join-Path $repoRoot 'tools\mvp\mvp-scenario-registry.json')
$terminalScenarioRegistryReceipt = Get-MvpScenarioRegistryReceipt -Registry $terminalScenarioRegistry
$terminalRuntimeScenario = Get-MvpScenarioRegistration `
    -Registry $terminalScenarioRegistry `
    -ScenarioId 'mvp.runtime-first-frame.v1'

function New-MvpTestTerminalContextSetReceipt {
    param([Parameter(Mandatory)][string]$RunId)

    $context = New-MvpProcessQualificationContext `
        -RunId $RunId `
        -SourceFingerprint ('A' * 64) `
        -BuildSetId ('B' * 64) `
        -ScenarioRegistryReceipt $terminalScenarioRegistryReceipt `
        -ScenarioRegistration $terminalRuntimeScenario `
        -ScenarioVariant 'host.default'
    return Get-MvpProcessQualificationContextSetReceipt `
        -Contexts @($context) `
        -ExpectedRunId $RunId
}

function New-MvpTestTerminalStorageCapabilityEvidence {
    return Get-MvpArtifactStorageCapabilityEvidence `
        -RootPath $script:terminalReceiptRoot `
        -CapabilityClass 'windows-local-artifact' `
        -RequiredFreeSpaceBytes 1
}

Describe 'MVP staging terminal receipt' {
    BeforeAll {
        $script:terminalReceiptRoot = New-MvpTestFixtureRoot -Prefix 'staging-terminal-receipt'
    }

    AfterAll {
        Remove-MvpTestFixtureRoot -Path $script:terminalReceiptRoot
    }

    It 'atomically publishes a bounded successful receipt' {
        $result = Write-MvpStagingTerminalReceipt `
            -StagingRoot $script:terminalReceiptRoot `
            -RunId 'fixture-success' `
            -Outcome 'succeeded' `
            -Phase 'complete' `
            -StartedAtUtc '2026-08-26T01:00:00.0000000+00:00' `
            -EndedAtUtc '2026-08-26T01:00:01.0000000+00:00' `
            -StagingDirectoryPublished $true `
            -CleanupOutcome 'not_required' `
            -QualificationContextSetReceipt (New-MvpTestTerminalContextSetReceipt -RunId 'fixture-success') `
            -StorageCapabilityEvidence (New-MvpTestTerminalStorageCapabilityEvidence) `
            -RequiredFreeSpaceBytes 1 `
            -StagingManifestSha256 ('a' * 64)

        $receipt = Get-Content -LiteralPath $result.path -Raw -Encoding UTF8 | ConvertFrom-Json
        $receipt.schema_version | Should Be 3
        $receipt.receipt_kind | Should Be 'zircon_mvp_staging_terminal'
        $receipt.run_id | Should Be 'fixture-success'
        $receipt.outcome | Should Be 'succeeded'
        $receipt.staging_directory_published | Should Be $true
        $receipt.staging_manifest_sha256 | Should Be ('a' * 64)
        $receipt.qualification_context_set.context_count | Should Be 1
        $receipt.qualification_context_set.qualification_status | Should Be 'unqualified_missing_product_receipt'
        $receipt.qualification_context_set.sha256 | Should Match '^[0-9a-f]{64}$'
        $receipt.storage_capability.capability_kind | Should Be 'zircon.mvp-artifact-storage-capability'
        $receipt.storage_capability.root_id | Should Match '^windows-local-[def]$'
        $receipt.storage_capability.required_free_space_bytes | Should Be 1
        $receipt.storage_capability.durable_file_flush_supported | Should Be $true
        $receipt.storage_capability.same_volume_atomic_move_supported | Should Be $true
        ([IO.FileInfo]::new($result.path).Length -le 16384) | Should Be $true
        @(Get-ChildItem -LiteralPath ([IO.Path]::GetDirectoryName($result.path)) -Filter '*.pending-*.tmp').Count | Should Be 0
    }

    It 'hashes failure and cleanup messages without retaining their text' {
        $failureText = 'fixture failure with private path E:\private\source'
        $cleanupText = 'fixture cleanup detail with private pid 1234'
        $result = Write-MvpStagingTerminalReceipt `
            -StagingRoot $script:terminalReceiptRoot `
            -RunId 'fixture-failure' `
            -Outcome 'failed' `
            -Phase 'product_startup' `
            -StartedAtUtc '2026-08-26T01:00:00.0000000+00:00' `
            -EndedAtUtc '2026-08-26T01:00:02.0000000+00:00' `
            -StagingDirectoryPublished $true `
            -CleanupOutcome 'failed' `
            -CleanupMessage $cleanupText `
            -QualificationContextSetReceipt (New-MvpTestTerminalContextSetReceipt -RunId 'fixture-failure') `
            -FailureKind 'product_startup_failed' `
            -FailureMessage $failureText `
            -StorageCapabilityEvidence (New-MvpTestTerminalStorageCapabilityEvidence) `
            -RequiredFreeSpaceBytes 1 `
            -StagingManifestSha256 ('c' * 64)

        $receiptText = Get-Content -LiteralPath $result.path -Raw -Encoding UTF8
        $receipt = $receiptText | ConvertFrom-Json
        $receipt.failure.kind | Should Be 'product_startup_failed'
        $receipt.staging_directory_published | Should Be $true
        $receipt.staging_manifest_sha256 | Should Be ('c' * 64)
        $receipt.failure.message_sha256 | Should Match '^[0-9a-f]{64}$'
        $receipt.cleanup.outcome | Should Be 'failed'
        $receipt.cleanup.message_sha256 | Should Match '^[0-9a-f]{64}$'
        $receipt.qualification_context_set.run_id | Should Be 'fixture-failure'
        $receipt.storage_capability.policy.sha256 | Should Match '^[0-9A-F]{64}$'
        $receiptText.Contains($failureText) | Should Be $false
        $receiptText.Contains($cleanupText) | Should Be $false
    }

    It 'publishes an admission failure before the staging root exists' {
        $stagingRoot = Join-Path $script:terminalReceiptRoot 'not-created-before-admission'
        Test-Path -LiteralPath $stagingRoot | Should Be $false

        $result = Write-MvpStagingTerminalReceipt `
            -StagingRoot $stagingRoot `
            -RunId 'fixture-admission-failure' `
            -Outcome 'failed' `
            -Phase 'admission' `
            -StartedAtUtc '2026-08-26T01:00:00.0000000+00:00' `
            -EndedAtUtc '2026-08-26T01:00:01.0000000+00:00' `
            -StagingDirectoryPublished $false `
            -CleanupOutcome 'not_required' `
            -FailureKind 'admission_failed' `
            -FailureMessage 'fixture admission rejection'

        $receipt = Get-Content -LiteralPath $result.path -Raw -Encoding UTF8 | ConvertFrom-Json
        $receipt.phase | Should Be 'admission'
        $receipt.outcome | Should Be 'failed'
        $receipt.staging_directory_published | Should Be $false
        ($null -eq $receipt.qualification_context_set) | Should Be $true
        ($null -eq $receipt.storage_capability) | Should Be $true
        Test-Path -LiteralPath (Join-Path $stagingRoot 'fixture-admission-failure') | Should Be $false
    }

    It 'requires current storage capability evidence for a published terminal state' {
        { Write-MvpStagingTerminalReceipt `
                -StagingRoot $script:terminalReceiptRoot `
                -RunId 'fixture-missing-storage-capability' `
                -Outcome 'succeeded' `
                -Phase 'complete' `
                -StartedAtUtc '2026-08-26T01:00:00.0000000+00:00' `
                -EndedAtUtc '2026-08-26T01:00:01.0000000+00:00' `
                -StagingDirectoryPublished $true `
                -CleanupOutcome 'not_required' `
                -QualificationContextSetReceipt (New-MvpTestTerminalContextSetReceipt -RunId 'fixture-missing-storage-capability') `
                -StagingManifestSha256 ('d' * 64) } | Should Throw 'requires storage capability evidence'
    }

    It 'binds every core Stage terminal path to one context-set receipt' {
        $stageSource = Get-Content -LiteralPath $stageScript -Raw
        $receiptSource = Get-Content -LiteralPath $receiptModule -Raw

        $stageSource | Should Match 'processQualificationContextSetReceipt\s+=\s+Get-MvpProcessQualificationContextSetReceipt'
        [regex]::Matches(
            $stageSource,
            '-QualificationContextSetReceipt\s+\$processQualificationContextSetReceipt').Count | Should Be 3
        [regex]::Matches(
            $stageSource,
            '-StorageCapabilityEvidence\s+\$storageCapabilityEvidence').Count | Should Be 3
        [regex]::Matches(
            $stageSource,
            '-RequiredFreeSpaceBytes\s+\(\[Int64\]\$preflight\.required_free_space_bytes\)').Count | Should Be 4
        [regex]::Matches(
            $stageSource,
            'process_qualification_context_set\s+=\s+\$processQualificationContextSetReceipt').Count | Should Be 3
        $receiptSource | Should Match "StagingTerminalReceiptSchemaVersion = 3"
        $receiptSource | Should Match "PSBoundParameters.ContainsKey\('RequiredFreeSpaceBytes'\)"
        $receiptSource | Should Match 'A published MVP staging terminal receipt requires a process qualification context-set receipt'
        $receiptSource | Should Match 'A published MVP staging terminal receipt requires storage capability evidence'
    }

    It 'does not overwrite the first terminal receipt for a run' {
        $first = Write-MvpStagingTerminalReceipt `
            -StagingRoot $script:terminalReceiptRoot `
            -RunId 'fixture-immutable' `
            -Outcome 'succeeded' `
            -Phase 'complete' `
            -StartedAtUtc '2026-08-26T01:00:00.0000000+00:00' `
            -EndedAtUtc '2026-08-26T01:00:01.0000000+00:00' `
            -StagingDirectoryPublished $true `
            -CleanupOutcome 'not_required' `
            -QualificationContextSetReceipt (New-MvpTestTerminalContextSetReceipt -RunId 'fixture-immutable') `
            -StorageCapabilityEvidence (New-MvpTestTerminalStorageCapabilityEvidence) `
            -RequiredFreeSpaceBytes 1 `
            -StagingManifestSha256 ('b' * 64)
        $firstBytes = [IO.File]::ReadAllBytes($first.path)
        $rejected = $false
        try {
            Write-MvpStagingTerminalReceipt `
                -StagingRoot $script:terminalReceiptRoot `
                -RunId 'fixture-immutable' `
                -Outcome 'failed' `
                -Phase 'product_startup' `
                -StartedAtUtc '2026-08-26T01:00:00.0000000+00:00' `
                -EndedAtUtc '2026-08-26T01:00:02.0000000+00:00' `
                -StagingDirectoryPublished $true `
                -CleanupOutcome 'succeeded' `
                -QualificationContextSetReceipt (New-MvpTestTerminalContextSetReceipt -RunId 'fixture-immutable') `
                -StorageCapabilityEvidence (New-MvpTestTerminalStorageCapabilityEvidence) `
                -RequiredFreeSpaceBytes 1 `
                -FailureKind 'late_failure' `
                -FailureMessage 'must not replace the first receipt' `
                -StagingManifestSha256 ('b' * 64) | Out-Null
        }
        catch {
            $rejected = $_.Exception.Message -match 'Refusing to overwrite existing MVP staging terminal receipt'
        }

        $rejected | Should Be $true
        [Convert]::ToBase64String([IO.File]::ReadAllBytes($first.path)) | Should Be ([Convert]::ToBase64String($firstBytes))
    }
}
