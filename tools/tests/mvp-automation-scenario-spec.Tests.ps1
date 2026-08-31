$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$modulePath = Join-Path $repoRoot 'tools\mvp\MvpAutomationScenarioSpec.psm1'
$authoringPath = Join-Path $repoRoot 'tools\mvp\mvp-authoring-automation.json'
$reopenPath = Join-Path $repoRoot 'tools\mvp\mvp-reopen-automation.json'
$stagePath = Join-Path $repoRoot 'tools\mvp\Stage-MvpProducts.ps1'

if (Test-Path -LiteralPath $modulePath) {
    Import-Module $modulePath -Force -ErrorAction Stop
}

Describe 'MVP automation scenario specification' {
    It 'publishes and validates versioned source-bound authoring and reopen scenarios' {
        Test-Path -LiteralPath $modulePath | Should Be $true

        $authoring = Assert-MvpAutomationScenarioSpec `
            -Path $authoringPath `
            -ExpectedScenarioId 'mvp.editor-authoring.v1'
        $reopen = Assert-MvpAutomationScenarioSpec `
            -Path $reopenPath `
            -ExpectedScenarioId 'mvp.editor-reopen.v1'

        $authoring.schema_version | Should Be 1
        $authoring.scenario_kind | Should Be 'zircon.mvp-editor-automation-scenario'
        $authoring.scenario_id | Should Be 'mvp.editor-authoring.v1'
        $authoring.binding_count | Should Be 6
        $reopen.scenario_id | Should Be 'mvp.editor-reopen.v1'
        $reopen.binding_count | Should Be 1
    }

    It 'rejects legacy and extended root schemas' {
        $legacyPath = Join-Path $TestDrive 'legacy.json'
        $extendedPath = Join-Path $TestDrive 'extended.json'
        [IO.File]::WriteAllText($legacyPath, '{"bindings":[{}]}', [Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText(
            $extendedPath,
            '{"schema_version":1,"scenario_kind":"zircon.mvp-editor-automation-scenario","scenario_id":"mvp.fixture.v1","bindings":[{}],"unreviewed":true}',
            [Text.UTF8Encoding]::new($false))

        { Assert-MvpAutomationScenarioSpec -Path $legacyPath -ExpectedScenarioId 'mvp.fixture.v1' } |
            Should Throw 'property count'
        { Assert-MvpAutomationScenarioSpec -Path $extendedPath -ExpectedScenarioId 'mvp.fixture.v1' } |
            Should Throw 'unknown property'
    }

    It 'rejects an unexpected scenario identity before product launch' {
        { Assert-MvpAutomationScenarioSpec -Path $authoringPath -ExpectedScenarioId 'mvp.editor-reopen.v1' } |
            Should Throw 'differs from expected'
    }

    It 'requires Stage to preflight both scenario identities before copying input files' {
        $source = Get-Content -LiteralPath $stagePath -Raw

        $source | Should Match 'MvpAutomationScenarioSpec\.psm1'
        $source | Should Match "Assert-MvpAutomationScenarioSpec[\s\S]*mvp\.editor-authoring\.v1"
        $source | Should Match "Assert-MvpAutomationScenarioSpec[\s\S]*mvp\.editor-reopen\.v1"
        $firstValidation = $source.IndexOf('Assert-MvpAutomationScenarioSpec')
        $firstCopy = $source.IndexOf("-LogicalId 'authoring-automation-request'")
        ($firstValidation -ge 0 -and $firstValidation -lt $firstCopy) | Should Be $true
    }
}
