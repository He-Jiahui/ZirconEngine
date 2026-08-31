$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$modulePath = Join-Path $PSScriptRoot '..\mvp\MvpProjectCopyPolicy.psm1'
$policyPath = Join-Path $PSScriptRoot '..\mvp\mvp-project-copy-policy.json'
Import-Module $modulePath -Force -ErrorAction Stop

function New-MvpTestProjectCopyPolicy {
    return [ordered]@{
        schema_version = 1
        policy_kind = 'zircon.mvp-project-copy-policy'
        policy_id = 'mvp-project-source-v1'
        path_comparison = 'ordinal-ignore-case'
        default = [ordered]@{
            ownership = 'source'
            copy_policy = 'include'
        }
        rules = @(
            [ordered]@{
                relative_directory = '.zircon/cache'
                ownership = 'derived'
                copy_policy = 'exclude-subtree'
            },
            [ordered]@{
                relative_directory = '.zircon/play'
                ownership = 'generated'
                copy_policy = 'exclude-subtree'
            }
        )
    }
}

function Write-MvpTestProjectCopyPolicy {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)]$Value
    )

    $json = $Value | ConvertTo-Json -Depth 8
    [IO.File]::WriteAllText($Path, $json, [Text.UTF8Encoding]::new($false))
}

Describe 'MVP project copy policy' {
    It 'loads the versioned production policy and freezes its byte identity' {
        $snapshot = Get-MvpProjectCopyPolicySnapshot -Path $policyPath

        (@($snapshot.receipt.PSObject.Properties.Name) -join ',') |
            Should Be 'schema_version,policy_kind,policy_id,sha256,size_bytes'
        $snapshot.receipt.schema_version | Should Be 1
        $snapshot.receipt.policy_kind | Should Be 'zircon.mvp-project-copy-policy'
        $snapshot.receipt.policy_id | Should Be 'mvp-project-source-v1'
        $snapshot.receipt.sha256 | Should Match '^[0-9A-F]{64}$'
        $snapshot.receipt.size_bytes | Should Be ([IO.FileInfo]::new($policyPath).Length)
        (@($snapshot.rules | ForEach-Object { $_.relative_directory }) -join ',') |
            Should Be '.zircon/autosave,.zircon/cache,.zircon/play,.zircon/registry,.zircon/thumbnails'
    }

    It 'includes source paths and excludes every declared derived or generated subtree' {
        $snapshot = Get-MvpProjectCopyPolicySnapshot -Path $policyPath

        (Test-MvpProjectCopyPolicyPathIncluded -PolicySnapshot $snapshot -RelativePath 'project.zrproj') |
            Should Be $true
        (Test-MvpProjectCopyPolicyPathIncluded -PolicySnapshot $snapshot -RelativePath 'Content/mesh.zasset') |
            Should Be $true
        (Test-MvpProjectCopyPolicyPathIncluded -PolicySnapshot $snapshot -RelativePath '.ZIRCON\CACHE\nested\stale.zasset') |
            Should Be $false
        (Test-MvpProjectCopyPolicyPathIncluded -PolicySnapshot $snapshot -RelativePath '.zircon/play') |
            Should Be $false
    }

    It 'classifies each staged path without per-call rule component or prefix arrays' {
        $snapshot = Get-MvpProjectCopyPolicySnapshot -Path $policyPath
        $source = Get-Content -LiteralPath $modulePath -Raw
        $classifier = [regex]::Match(
            $source,
            '(?s)function Test-MvpProjectCopyPolicyPathIncluded \{.*?(?=\r?\nExport-ModuleMember)')

        $snapshot.rules[0].match_prefix | Should Be '.zircon/autosave/'
        $classifier.Value | Should Match 'foreach \(\$rule in \$PolicySnapshot\.rules\)'
        $classifier.Value | Should Match '\$normalized\.StartsWith\(\$excludedPrefix'
        $classifier.Value | Should Not Match '@\(\$PolicySnapshot\.rules\)'
        $classifier.Value | Should Not Match '\$normalized\.Split\('
        $classifier.Value | Should Not Match '\$excluded \+ ''/'''
    }

    It 'rejects unknown root properties' {
        $value = New-MvpTestProjectCopyPolicy
        $value.unknown = $true
        $path = Join-Path $TestDrive 'unknown-root.json'
        Write-MvpTestProjectCopyPolicy -Path $path -Value $value

        { Get-MvpProjectCopyPolicySnapshot -Path $path } |
            Should Throw "contains unknown property 'unknown'"
    }

    It 'rejects unknown rule properties' {
        $value = New-MvpTestProjectCopyPolicy
        $value.rules[0].unknown = $true
        $path = Join-Path $TestDrive 'unknown-rule.json'
        Write-MvpTestProjectCopyPolicy -Path $path -Value $value

        { Get-MvpProjectCopyPolicySnapshot -Path $path } |
            Should Throw "contains unknown property 'unknown'"
    }

    It 'rejects incompatible identity and default ownership contracts' {
        $value = New-MvpTestProjectCopyPolicy
        $value.schema_version = 2
        $path = Join-Path $TestDrive 'wrong-version.json'
        Write-MvpTestProjectCopyPolicy -Path $path -Value $value
        { Get-MvpProjectCopyPolicySnapshot -Path $path } | Should Throw 'schema_version'

        $value = New-MvpTestProjectCopyPolicy
        $value.default.ownership = 'derived'
        $path = Join-Path $TestDrive 'wrong-default.json'
        Write-MvpTestProjectCopyPolicy -Path $path -Value $value
        { Get-MvpProjectCopyPolicySnapshot -Path $path } | Should Throw 'default ownership'
    }

    It 'rejects unsafe duplicate overlapping and non-ordinal rule paths' {
        $value = New-MvpTestProjectCopyPolicy
        $value.rules[0].relative_directory = '../cache'
        $path = Join-Path $TestDrive 'unsafe.json'
        Write-MvpTestProjectCopyPolicy -Path $path -Value $value
        { Get-MvpProjectCopyPolicySnapshot -Path $path } | Should Throw 'relative_directory'

        $value = New-MvpTestProjectCopyPolicy
        $value.rules[1].relative_directory = '.zircon/cache'
        $path = Join-Path $TestDrive 'duplicate.json'
        Write-MvpTestProjectCopyPolicy -Path $path -Value $value
        { Get-MvpProjectCopyPolicySnapshot -Path $path } | Should Throw 'duplicate'

        $value = New-MvpTestProjectCopyPolicy
        $value.rules[1].relative_directory = '.zircon/cache/child'
        $path = Join-Path $TestDrive 'overlap.json'
        Write-MvpTestProjectCopyPolicy -Path $path -Value $value
        { Get-MvpProjectCopyPolicySnapshot -Path $path } | Should Throw 'overlapping'

        $value = New-MvpTestProjectCopyPolicy
        [array]::Reverse($value.rules)
        $path = Join-Path $TestDrive 'unsorted.json'
        Write-MvpTestProjectCopyPolicy -Path $path -Value $value
        { Get-MvpProjectCopyPolicySnapshot -Path $path } | Should Throw 'ordinally sorted'
    }

    It 'makes Stage prune project directories and bind the policy receipt without physical literals' {
        $stageSource = Get-Content (Join-Path $PSScriptRoot '..\mvp\Stage-MvpProducts.ps1') -Raw

        $stageSource | Should Match "Import-Module .*MvpProjectCopyPolicy\.psm1"
        $stageSource | Should Match 'Get-MvpProjectCopyPolicySnapshot'
        $stageSource | Should Match '-ProjectCopyPolicySnapshot \$projectCopyPolicy'
        $stageSource | Should Match 'Test-MvpProjectCopyPolicyPathIncluded'
        $stageSource | Should Match 'project_copy_policy = \$projectCopyPolicy\.receipt'
        $stageSource | Should Not Match '\.zircon/(?:autosave|cache|play|registry|thumbnails)'
    }
}
