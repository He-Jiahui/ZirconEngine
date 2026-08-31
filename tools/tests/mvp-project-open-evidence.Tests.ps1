Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$modulePath = Join-Path $repoRoot 'tools\mvp\MvpProjectOpenEvidence.psm1'
$moduleSource = Get-Content -LiteralPath $modulePath -Raw
Import-Module $modulePath -Force -ErrorAction Stop

function New-MvpProjectOpenDiagnostic {
    param(
        [string]$Result = 'completed',
        [string]$Separator = ' '
    )

    $fields = @(
        "result=$Result",
        'project_root=.',
        'manifest_identity=renderable-empty@v1',
        'scene_uri=res%3A%2F%2Fscenes%2Fmain.scene.toml',
        'settings_source=persisted-project',
        'registry_asset_count=4',
        'registry_ready_asset_count=4',
        'registry_failed_asset_count=0',
        'registry_diagnostic_count=0',
        'project_generation=1',
        'project_generation_publish_epoch=1',
        'catalog_asset_count=4'
    ) -join $Separator
    return 'editor_project_open ' + $fields
}

Describe 'MVP project-open evidence' {
    It 'reuses the expected project resolution when ProjectRoot owns both roles' {
        $moduleSource | Should Match '\$projectRootIsExpected\s*=\s*\[string\]::IsNullOrWhiteSpace\(\$ExpectedProjectRoot\)'
        $moduleSource | Should Match '\$relativeProjectResolution\s*=\s*if \(\$projectRootIsExpected\)'
        $moduleSource | Should Match '-ProjectResolution \$relativeProjectResolution'
    }

    It 'selects the last project-open diagnostic without splitting the complete log' {
        $stagingRoot = $repoRoot
        $projectRoot = Join-Path $repoRoot 'tools'
        $diagnosticText = @(
            'unrelated startup output',
            (New-MvpProjectOpenDiagnostic -Result 'failed'),
            'more unrelated output',
            (New-MvpProjectOpenDiagnostic),
            'trailing output'
        ) -join "`r`n"

        $evidence = Get-MvpEditorProjectOpenEvidence `
            -DiagnosticText $diagnosticText `
            -StagingRoot $stagingRoot `
            -ProjectRoot $projectRoot `
            -ExpectedProjectRoot $projectRoot

        $evidence.project_root | Should Be 'tools'
        $evidence.manifest_identity | Should Be 'renderable-empty@v1'
        $moduleSource | Should Match '\.LastIndexOf\(\$marker, \[StringComparison\]::Ordinal\)'
        $moduleSource | Should Not Match '\$DiagnosticText\s+-split'
        $moduleSource | Should Not Match 'Where-Object\s+\{\s*\$_\.IndexOf\(''editor_project_open '
    }

    It 'indexes the selected diagnostic tokens once without per-field regex matching' {
        $moduleSource | Should Match 'function Get-MvpProjectOpenDiagnosticTokens'
        ([regex]::Matches($moduleSource, 'Get-MvpProjectOpenDiagnosticTokens\s+-Diagnostic')).Count | Should Be 1
        $moduleSource | Should Match 'Dictionary\[string, string\]\]::new\(12, \[StringComparer\]::Ordinal\)'
        $moduleSource | Should Match '\$Diagnostic\.Split\(\s*\[char\[\]\]\$null,\s*\[StringSplitOptions\]::RemoveEmptyEntries\)'
        $moduleSource | Should Not Match 'while \(\$index -lt \$Diagnostic\.Length\)'
        $moduleSource | Should Not Match '\[regex\]::Match\(\s*\$Diagnostic'
    }

    It 'accepts tab-delimited diagnostic fields' {
        $projectRoot = Join-Path $repoRoot 'tools'
        $diagnostic = New-MvpProjectOpenDiagnostic -Separator "`t"

        $evidence = Get-MvpEditorProjectOpenEvidence `
            -DiagnosticText $diagnostic `
            -StagingRoot $repoRoot `
            -ProjectRoot $projectRoot `
            -ExpectedProjectRoot $projectRoot

        $evidence.scene_uri | Should Be 'res://scenes/main.scene.toml'
        $evidence.catalog_asset_count | Should Be 4
    }

    It 'accepts Unicode whitespace-delimited diagnostic fields' {
        $projectRoot = Join-Path $repoRoot 'tools'
        $diagnostic = New-MvpProjectOpenDiagnostic -Separator ([char]0x2003)
        $evidence = Get-MvpEditorProjectOpenEvidence -DiagnosticText $diagnostic -StagingRoot $repoRoot -ProjectRoot $projectRoot -ExpectedProjectRoot $projectRoot

        $evidence.scene_uri | Should Be 'res://scenes/main.scene.toml'
        $evidence.catalog_asset_count | Should Be 4
    }

    It 'fails closed when a required token is absent' {
        $projectRoot = Join-Path $repoRoot 'tools'
        $diagnostic = (New-MvpProjectOpenDiagnostic).Replace(' catalog_asset_count=4', '')

        {
            Get-MvpEditorProjectOpenEvidence `
                -DiagnosticText $diagnostic `
                -StagingRoot $repoRoot `
                -ProjectRoot $projectRoot `
                -ExpectedProjectRoot $projectRoot
        } | Should Throw "missing 'catalog_asset_count'"
    }

}
