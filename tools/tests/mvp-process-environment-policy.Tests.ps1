$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$policyModule = Join-Path $repoRoot 'tools\mvp\MvpProcessEnvironmentPolicy.psm1'
$stagePolicyModule = Join-Path $repoRoot 'tools\mvp\MvpStageProcessEnvironmentPolicy.psm1'

Import-Module $policyModule -Force -ErrorAction Stop
Import-Module $stagePolicyModule -Force -ErrorAction Stop

Describe 'MVP process environment policy' {
    It 'applies only the scenario-owned inherited and declared environment' {
        $startInfo = [Diagnostics.ProcessStartInfo]::new()
        $startInfo.EnvironmentVariables['ZIRCON_MVP_UNAPPROVED_PARENT_ENV'] = 'parent-secret-value'
        $policy = New-MvpProcessEnvironmentPolicy `
            -PolicyId 'test.minimal-v1' `
            -InheritedNames @('ComSpec', 'SystemRoot') `
            -DeclaredNames @('ZIRCON_LOG_FILTER')

        $applied = Set-MvpProcessEnvironmentPolicy `
            -StartInfo $startInfo `
            -Policy $policy `
            -DeclaredEnvironment @{ ZIRCON_LOG_FILTER = 'allowed-fixture-value' }

        @($policy.PSObject.Properties.Name) | Should Be @(
            'schema_version',
            'policy_kind',
            'policy_id',
            'inherited_names',
            'declared_names')
        $policy.schema_version | Should Be 1
        $policy.policy_kind | Should Be 'zircon.mvp-process-environment-policy'
        $applied.schema_version | Should Be 1
        $applied.policy_kind | Should Be 'zircon.mvp-process-environment-policy'
        $applied.policy_id | Should Be 'test.minimal-v1'
        @($applied.variables | Where-Object { $_.name -eq 'ZIRCON_LOG_FILTER' }).Count | Should Be 1
        @($applied.variables | Where-Object { $_.source -eq 'supervisor_inherited' }).Count | Should Be 2
        $startInfo.EnvironmentVariables.ContainsKey('ZIRCON_MVP_UNAPPROVED_PARENT_ENV') | Should Be $false
        $startInfo.EnvironmentVariables['ZIRCON_LOG_FILTER'] | Should Be 'allowed-fixture-value'
        $applied.variables | ForEach-Object {
            $_.sensitivity | Should Be 'non_sensitive'
            $_.value_sha256 | Should Match '^[0-9a-f]{64}$'
        }
    }

    It 'rejects legacy, extended, and unsupported policy schemas before mutating the child environment' {
        $validPolicy = New-MvpProcessEnvironmentPolicy `
            -PolicyId 'test.schema-v1' `
            -InheritedNames @('SystemRoot') `
            -DeclaredNames @('ZIRCON_LOG_FILTER')
        $legacyPolicy = [pscustomobject]@{
            policy_id = $validPolicy.policy_id
            inherited_names = $validPolicy.inherited_names
            declared_names = $validPolicy.declared_names
        }
        $extendedPolicy = $validPolicy | Select-Object *, @{ Name = 'unreviewed'; Expression = { $true } }
        $futurePolicy = [pscustomobject]@{
            schema_version = 2
            policy_kind = 'zircon.mvp-process-environment-policy'
            policy_id = $validPolicy.policy_id
            inherited_names = $validPolicy.inherited_names
            declared_names = $validPolicy.declared_names
        }

        foreach ($policyCase in @($legacyPolicy, $extendedPolicy, $futurePolicy)) {
            $startInfo = [Diagnostics.ProcessStartInfo]::new()
            $startInfo.EnvironmentVariables['ZIRCON_MVP_UNAPPROVED_PARENT_ENV'] = 'must-remain-on-rejection'
            $failure = $null
            try {
                Set-MvpProcessEnvironmentPolicy `
                    -StartInfo $startInfo `
                    -Policy $policyCase `
                    -DeclaredEnvironment @{ ZIRCON_LOG_FILTER = 'unused' } | Out-Null
            }
            catch {
                $failure = $_.Exception
            }

            ($null -ne $failure) | Should Be $true
            $startInfo.EnvironmentVariables['ZIRCON_MVP_UNAPPROVED_PARENT_ENV'] |
                Should Be 'must-remain-on-rejection'
        }
    }

    It 'rejects a globally known variable outside the scenario declaration set' {
        $startInfo = [Diagnostics.ProcessStartInfo]::new()
        $policy = New-MvpProcessEnvironmentPolicy `
            -PolicyId 'test.filter-only-v1' `
            -InheritedNames @('SystemRoot') `
            -DeclaredNames @('ZIRCON_LOG_FILTER')

        $failure = $null
        try {
            Set-MvpProcessEnvironmentPolicy `
                -StartInfo $startInfo `
                -Policy $policy `
                -DeclaredEnvironment @{ ZIRCON_LOG_ROOT = 'diagnostics' }
        }
        catch {
            $failure = $_.Exception
        }

        ($null -ne $failure) | Should Be $true
        $failure.Message | Should Match "not allowed by environment policy 'test\.filter-only-v1'"
    }

    It 'rejects inherited names outside the supervisor host allowlist' {
        $failure = $null
        try {
            New-MvpProcessEnvironmentPolicy `
                -PolicyId 'test.host-escape-v1' `
                -InheritedNames @('SystemRoot', 'USERPROFILE') `
                -DeclaredNames @('ZIRCON_LOG_FILTER')
        }
        catch {
            $failure = $_.Exception
        }

        ($null -ne $failure) | Should Be $true
        $failure.Message | Should Match 'not in the supervisor host allowlist'
    }

    It 'keeps runtime, editor, creation, and authoring declarations separate' {
        $runtime = Get-MvpStageProcessEnvironmentPolicy -Scenario 'runtime_first_frame'
        $editor = Get-MvpStageProcessEnvironmentPolicy -Scenario 'editor_first_frame'
        $creation = Get-MvpStageProcessEnvironmentPolicy -Scenario 'editor_project_create'
        $authoring = Get-MvpStageProcessEnvironmentPolicy -Scenario 'editor_authoring'

        $runtime.policy_id | Should Be 'mvp.runtime-first-frame.v1'
        ($runtime.declared_names -contains 'ZIRCON_RUNTIME_MVP_INPUT_PROBE') | Should Be $true
        ($runtime.declared_names -contains 'ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG') | Should Be $false
        ($editor.declared_names -contains 'ZIRCON_EDITOR_EXIT_AFTER_FIRST_FRAME') | Should Be $true
        ($editor.declared_names -contains 'ZIRCON_RUNTIME_EXIT_AFTER_FIRST_FRAME') | Should Be $false
        $creation.policy_id | Should Be 'mvp.editor-project-create.v1'
        ($creation.declared_names -contains 'ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG') | Should Be $true
        $authoring.policy_id | Should Be 'mvp.editor-authoring.v1'
        ($authoring.declared_names -contains 'ZIRCON_EDITOR_EXIT_AFTER_FIRST_FRAME') | Should Be $false
    }
}
