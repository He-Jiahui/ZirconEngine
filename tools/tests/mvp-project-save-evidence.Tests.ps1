Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$modulePath = Join-Path $repoRoot 'tools\mvp\MvpProjectSaveEvidence.psm1'
$moduleSource = Get-Content -LiteralPath $modulePath -Raw
Import-Module $modulePath -Force -ErrorAction Stop
$module = Get-Module -Name MvpProjectSaveEvidence -ErrorAction Stop
$encodedProject = & $module {
    param([string]$Value)

    ConvertTo-MvpProjectSaveDiagnosticToken -Value $Value
} $repoRoot

function New-MvpProjectSaveStartedDiagnostic {
    param([string]$ExtraFields = '')

    return "editor_project_save result=started project=$encodedProject pre_save_dirty=true pre_save_dirty_generation=1 save_token_generation=1$ExtraFields"
}

function New-MvpProjectSaveCompletedDiagnostic {
    return "editor_project_save result=completed project=$encodedProject pre_save_dirty_generation=1 save_token_generation=1 persisted_generation=1 save_mark=Marked"
}

Describe 'MVP project-save evidence' {
    It 'encodes canonical UTF-8 bytes through one frozen lookup table' {
        $encoded = & $module {
            ConvertTo-MvpProjectSaveDiagnosticToken -Value "!*'()~ 项目 😀"
        }

        $encoded | Should Be '%21%2A%27%28%29~%20%E9%A1%B9%E7%9B%AE%20%F0%9F%98%80'
        $moduleSource | Should Match '\[string\[\]\]::new\(256\)'
        $moduleSource | Should Match '\$builder\.Append\(\$mvpProjectSaveDiagnosticTokenByByte\[\$byte\]\)'
        $moduleSource | Should Not Match '\$byte\.ToString\(''X2'''
    }

    It 'scans save diagnostics without splitting and filtering the complete log' {
        $moduleSource | Should Match 'function Get-MvpProjectSaveLifecycleDiagnostics'
        $moduleSource | Should Match '\.IndexOf\(\$marker, \$searchIndex, \[StringComparison\]::Ordinal\)'
        $moduleSource | Should Not Match '\$DiagnosticText\s+-split'
        $moduleSource | Should Not Match 'Where-Object\s+\{\s*\$_.text\s+-match\s+''editor_project_save'
    }

    It 'indexes started and completed fields once without per-field regex matching' {
        $moduleSource | Should Match 'function Get-MvpProjectSaveDiagnosticTokens'
        ([regex]::Matches($moduleSource, 'Get-MvpProjectSaveDiagnosticTokens\s+-Line')).Count | Should Be 2
        $moduleSource | Should Match '\$Line\.Split\(\s*\[char\[\]\]\$null,\s*\[StringSplitOptions\]::RemoveEmptyEntries\)'
        $moduleSource | Should Not Match 'while \(\$index -lt \$Line\.Length\)'
        $moduleSource | Should Not Match '\[regex\]::Matches\(\$Line'
    }

    It 'validates the two project paths through parallel token and label arrays' {
        $moduleSource | Should Match '\$pathDiagnosticTokens\s*=\s*@\('
        $moduleSource | Should Match '\$pathDiagnosticLabels\s*=\s*\[string\[\]\]@\('
        $moduleSource | Should Match 'for \(\$diagnosticIndex\s*=\s*0;'
        $moduleSource | Should Not Match '@\{ tokens = \$startedTokens; label ='
    }

    It 'allocates duplicate token counts only after the first collision' {
        $moduleSource | Should Match '\$duplicateCounts\s*=\s*\$null'
        $moduleSource | Should Match 'duplicate_counts\s*=\s*\$duplicateCounts'
        $moduleSource | Should Not Match '\$counts\s*=\s*\[Collections\.Generic\.Dictionary\[string, int\]\]::new'
    }

    It 'accepts one ordered started and completed save pair' {
        $diagnosticText = @(
            'unrelated startup output',
            (New-MvpProjectSaveStartedDiagnostic),
            'unrelated authoring output',
            (New-MvpProjectSaveCompletedDiagnostic),
            'trailing output'
        ) -join "`r`n"

        $evidence = Assert-MvpProjectSaveLifecycleEvidence `
            -DiagnosticText $diagnosticText `
            -SaveOperationId 'file.project.save' `
            -SaveGeneration 1 `
            -ExpectedProjectPath $repoRoot

        $evidence.pre_save_dirty | Should Be $true
        $evidence.persisted_generation | Should Be 1
        $evidence.save_mark | Should Be 'Marked'
    }

    It 'accepts Unicode whitespace-delimited save fields' {
        $separator = [char]0x2003
        $started = 'editor_project_save result=started' + $separator + 'project=' + $encodedProject + $separator + 'pre_save_dirty=true' + $separator + 'pre_save_dirty_generation=1' + $separator + 'save_token_generation=1'
        $completed = 'editor_project_save result=completed' + $separator + 'project=' + $encodedProject + $separator + 'pre_save_dirty_generation=1' + $separator + 'save_token_generation=1' + $separator + 'persisted_generation=1' + $separator + 'save_mark=Marked'
        $diagnosticText = $started + [Environment]::NewLine + $completed

        $evidence = Assert-MvpProjectSaveLifecycleEvidence -DiagnosticText $diagnosticText -SaveOperationId 'file.project.save' -SaveGeneration 1 -ExpectedProjectPath $repoRoot

        $evidence.persisted_generation | Should Be 1
        $evidence.save_mark | Should Be 'Marked'
    }

    It 'rejects any failed save lifecycle' {
        $diagnosticText = @(
            (New-MvpProjectSaveStartedDiagnostic),
            'editor_project_save result=failed',
            (New-MvpProjectSaveCompletedDiagnostic)
        ) -join "`n"

        {
            Assert-MvpProjectSaveLifecycleEvidence `
                -DiagnosticText $diagnosticText `
                -SaveOperationId 'file.project.save' `
                -SaveGeneration 1 `
                -ExpectedProjectPath $repoRoot
        } | Should Throw 'failed save lifecycle'
    }

    It 'rejects a duplicate started field' {
        $diagnosticText = @(
            (New-MvpProjectSaveStartedDiagnostic -ExtraFields ' pre_save_dirty=false'),
            (New-MvpProjectSaveCompletedDiagnostic)
        ) -join "`n"

        {
            Assert-MvpProjectSaveLifecycleEvidence `
                -DiagnosticText $diagnosticText `
                -SaveOperationId 'file.project.save' `
                -SaveGeneration 1 `
                -ExpectedProjectPath $repoRoot
        } | Should Throw "exactly one 'pre_save_dirty' field; found 2"
    }

}
