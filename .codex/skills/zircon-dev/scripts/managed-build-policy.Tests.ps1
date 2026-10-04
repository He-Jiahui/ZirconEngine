$script:ManagedBuildPolicyTestScript = Join-Path $PSScriptRoot 'managed-build-policy.ps1'
$script:windowsPathResolverRepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..\..')).ProviderPath
. $script:ManagedBuildPolicyTestScript

Describe 'Managed compile workspace command transport' {
    It 'passes command JSON through a temporary UTF-8 file and removes it after use' {
        $script:CapturedManagedCompileArguments = @()
        $script:CapturedManagedCompileCommandsPath = $null
        $script:CapturedManagedCompileCommandsJson = $null
        Mock Invoke-ManagedCompileWorkspaceCoordinator {
            param($CommandArguments)
            $script:CapturedManagedCompileArguments = @($CommandArguments)
            $pathIndex = [array]::IndexOf($script:CapturedManagedCompileArguments, '--commands-json-file')
            if ($pathIndex -ge 0) {
                $script:CapturedManagedCompileCommandsPath = $script:CapturedManagedCompileArguments[$pathIndex + 1]
                $script:CapturedManagedCompileCommandsJson = [IO.File]::ReadAllText(
                    $script:CapturedManagedCompileCommandsPath,
                    [Text.Encoding]::UTF8
                )
            }
            $global:LASTEXITCODE = 0
            return '{"sourceRoot":"E:\\sealed\\source","cargoHome":"E:\\sealed\\cargo-home","metrics":{}}'
        }

        $commandsJson = '[["cargo","test","-p","zircon_runtime","--features","feature-one,feature-two"]]'
        $result = Initialize-ManagedCompileWorkspace `
            -RepoRoot $script:windowsPathResolverRepoRoot `
            -TargetDirectory 'E:\cargo-targets\zircon-engine\pool\test' `
            -SourceRoot 'E:\snapshot\source' `
            -CommandsJson $commandsJson

        [array]::IndexOf($script:CapturedManagedCompileArguments, '--commands-json-file') | Should BeGreaterThan -1
        [array]::IndexOf($script:CapturedManagedCompileArguments, '--commands-json') | Should Be -1
        $script:CapturedManagedCompileCommandsJson | Should Be $commandsJson
        Test-Path -LiteralPath $script:CapturedManagedCompileCommandsPath | Should Be $false
        $result.sourceRoot | Should Be 'E:\sealed\source'
    }

    It 'removes the temporary command manifest when synchronization fails' {
        $script:FailedManagedCompileCommandsPath = $null
        Mock Invoke-ManagedCompileWorkspaceCoordinator {
            param($CommandArguments)
            $pathIndex = [array]::IndexOf(@($CommandArguments), '--commands-json-file')
            $script:FailedManagedCompileCommandsPath = $CommandArguments[$pathIndex + 1]
            Test-Path -LiteralPath $script:FailedManagedCompileCommandsPath | Should Be $true
            $global:LASTEXITCODE = 1
        }

        $failure = $null
        try {
            Initialize-ManagedCompileWorkspace `
                -RepoRoot $script:windowsPathResolverRepoRoot `
                -TargetDirectory 'E:\cargo-targets\zircon-engine\pool\test' `
                -SourceRoot 'E:\snapshot\source' `
                -CommandsJson '[["cargo","check"]]'
        }
        catch {
            $failure = $_
        }

        $failure.Exception.Message | Should Be 'Could not synchronize the leased compiler workspace.'
        Test-Path -LiteralPath $script:FailedManagedCompileCommandsPath | Should Be $false
    }
}
