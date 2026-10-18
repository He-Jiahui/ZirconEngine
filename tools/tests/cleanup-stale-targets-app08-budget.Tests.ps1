$scriptPath = Join-Path (Split-Path -Parent $PSScriptRoot) "cleanup-stale-targets.ps1"
. $scriptPath

function Invoke-App08CleanupTestClient {
    param($Command, $RepoRoot, [switch]$Json, [Parameter(ValueFromRemainingArguments = $true)]$Arguments)
}

Describe 'App08 cleanup durable request recovery' {
    BeforeEach {
        Mock Invoke-App08CleanupTestClient {
            param($Command, $RepoRoot, [switch]$Json, $Arguments)
            if ($Command -eq 'request-status') {
                $global:LASTEXITCODE = 0
                return '{"request":{"requestId":"0123456789abcdef0123456789abcdef","status":"completed"},"result":{"plan":{"plan_id":"original-plan"},"unmanaged":[]}}'
            }
            $global:LASTEXITCODE = 2
            return '{"status":"error","error":{"code":"command_post_timeout","details":{"requestId":"0123456789abcdef0123456789abcdef","phase":"post_response"}}}'
        }
    }

    It 'recovers the same cleanup plan without submitting a second request' {
        $result = Invoke-CoordinatorCleanupCommand -Client Invoke-App08CleanupTestClient `
            -ResolvedRepoRoot 'E:\Git\ZirconEngine' -Action plan -RetentionHours 2
        $result.plan.plan_id | Should Be 'original-plan'
        Assert-MockCalled Invoke-App08CleanupTestClient -Times 1 -Exactly -Scope It -ParameterFilter { $Command -eq 'cleanup' }
        Assert-MockCalled Invoke-App08CleanupTestClient -Times 1 -Exactly -Scope It -ParameterFilter { $Command -eq 'request-status' }
    }

    It 'recovers artifact auditing through the same accepted request protocol' {
        $result = Invoke-CoordinatorArtifactCommand -Client Invoke-App08CleanupTestClient `
            -ResolvedRepoRoot 'E:\Git\ZirconEngine' -Action audit
        @($result.unmanaged).Count | Should Be 0
        Assert-MockCalled Invoke-App08CleanupTestClient -Times 1 -Exactly -Scope It -ParameterFilter { $Command -eq 'artifact' }
        Assert-MockCalled Invoke-App08CleanupTestClient -Times 1 -Exactly -Scope It -ParameterFilter { $Command -eq 'request-status' }
    }

    It 'recovers a cleanup plan when its status endpoint is temporarily overloaded' {
        $script:statusQueries = 0
        Mock Start-Sleep {}
        Mock Invoke-App08CleanupTestClient {
            param($Command, $RepoRoot, [switch]$Json, $Arguments)
            if ($Command -eq 'request-status') {
                $script:statusQueries++
                if ($script:statusQueries -eq 1) {
                    $global:LASTEXITCODE = 1
                    return '{"error":{"code":"request_overloaded","message":"busy"}}'
                }
                $global:LASTEXITCODE = 0
                return '{"request":{"requestId":"0123456789abcdef0123456789abcdef","status":"completed"},"result":{"plan":{"plan_id":"original-plan"}}}'
            }
            $global:LASTEXITCODE = 2
            return '{"error":{"code":"command_post_timeout","details":{"requestId":"0123456789abcdef0123456789abcdef","phase":"post_response"}}}'
        }
        $result = Invoke-CoordinatorCleanupCommand -Client Invoke-App08CleanupTestClient `
            -ResolvedRepoRoot 'E:\Git\ZirconEngine' -Action plan -RetentionHours 2
        $result.plan.plan_id | Should Be 'original-plan'
        Assert-MockCalled Invoke-App08CleanupTestClient -Times 1 -Exactly -Scope It -ParameterFilter { $Command -eq 'cleanup' }
        Assert-MockCalled Invoke-App08CleanupTestClient -Times 2 -Exactly -Scope It -ParameterFilter { $Command -eq 'request-status' }
    }
}

Describe "App08 historical Cargo target cleanup" {
    It "requires Cargo ownership markers and artifact layout" {
        $root = Join-Path $TestDrive "target"
        New-Item -ItemType Directory -Path (Join-Path $root "debug\deps") -Force | Out-Null
        Set-Content -LiteralPath (Join-Path $root ".rustc_info.json") -Value '{"rustc_fingerprint":1,"outputs":{}}'

        Test-HistoricalCargoTargetOwnership -TargetPath $root | Should Be $true
    }

    It "rejects an unowned directory" {
        $root = Join-Path $TestDrive "unowned-target"
        New-Item -ItemType Directory -Path (Join-Path $root "debug") -Force | Out-Null

        Test-HistoricalCargoTargetOwnership -TargetPath $root | Should Be $false
    }

    It "rejects every repository path except the approved absolute E root" {
        $root = Join-Path $TestDrive "repo"
        $target = Join-Path $root "target"
        New-Item -ItemType Directory -Path $target -Force | Out-Null

        $result = Test-HistoricalRepositoryTarget -RepositoryRoot $root -TargetPath $target

        $result.Valid | Should Be $false
        $result.Reason | Should Be "historical_repo_not_approved"
    }

    It "returns a retained status when the final safety review fails" {
        $result = Remove-HistoricalRepositoryTarget `
            -RepositoryRoot (Join-Path $TestDrive "unapproved-repo") `
            -TargetPath (Join-Path $TestDrive "unapproved-repo\target") `
            -Confirm:$false

        $result.Status | Should Be "retained"
        $result.Reason | Should Be "historical_repo_not_approved"
    }

    It "keeps Apply explicit and defaults to preview" {
        $source = Get-Content -LiteralPath $scriptPath -Raw

        $source | Should Match '\[switch\]\$HistoricalRepositoryTarget'
        $source | Should Match '\[switch\]\$Apply'
        $source | Should Match 'if \(-not \$ApplyChanges -or \$WhatIfPreference\)'
    }

    It "rejects malformed Cargo markers" {
        $root = Join-Path $TestDrive "malformed-target"
        New-Item -ItemType Directory -Path (Join-Path $root "debug\deps") -Force | Out-Null
        Set-Content -LiteralPath (Join-Path $root ".rustc_info.json") -Value '{}'

        Test-HistoricalCargoTargetOwnership -TargetPath $root | Should Be $false
    }

    It "accepts only coordinator metadata-home junctions" {
        $root = Join-Path $TestDrive "junction-target"
        $source = Join-Path $root "cache\cargo-home\git\db"
        $junction = Join-Path $root "cache\cargo-metadata-home\git\db"
        New-Item -ItemType Directory -Path $source -Force | Out-Null
        New-Item -ItemType Directory -Path (Split-Path -Parent $junction) -Force | Out-Null
        New-Item -ItemType Junction -Path $junction -Target $source | Out-Null
        $item = Get-Item -LiteralPath $junction -Force

        Test-HistoricalExpectedCargoReparsePoint -TargetRoot $root -Item $item | Should Be $true
    }

    It "rejects an unexpected nested junction" {
        $root = Join-Path $TestDrive "unexpected-junction-target"
        $source = Join-Path $root "cache\cargo-home\git\db"
        $junction = Join-Path $root "debug\deps-link"
        New-Item -ItemType Directory -Path $source -Force | Out-Null
        New-Item -ItemType Directory -Path (Split-Path -Parent $junction) -Force | Out-Null
        New-Item -ItemType Junction -Path $junction -Target $source | Out-Null
        $item = Get-Item -LiteralPath $junction -Force

        Test-HistoricalExpectedCargoReparsePoint -TargetRoot $root -Item $item | Should Be $false
    }

    It "rejects relative target paths before looking at the repository" {
        $result = Test-HistoricalRepositoryTarget `
            -RepositoryRoot "E:\Git\ZirconEngine" -TargetPath "target"
        $result.Reason | Should Be "target_not_absolute"
    }

    It "fails closed when a Cargo command is unreadable" {
        Mock Get-CimInstance {
            [pscustomobject]@{
                Name = 'cargo.exe'; CommandLine = $null; ExecutablePath = $null
                ProcessId = 100; ParentProcessId = 20
            }
        }

        $result = Test-HistoricalCargoProcessesIdle

        $result.Idle | Should Be $false
        $result.Reason | Should Be "cargo_process_command_unknown"
    }

    It "protects orphaned build executables under the historical target" {
        Mock Get-CimInstance {
            [pscustomobject]@{
                Name = 'build-script-build.exe'; CommandLine = 'build-script-build.exe'
                ExecutablePath = 'E:\Git\ZirconEngine\target\debug\build\job\build-script-build.exe'
                ProcessId = 101; ParentProcessId = 100
            }
        }

        $result = Test-HistoricalCargoProcessesIdle

        $result.Idle | Should Be $false
        $result.Reason | Should Be "target_process_live"
    }

    It "retains process tree evidence for a live Cargo owner" {
        Mock Get-CimInstance {
            [pscustomobject]@{
                Name = 'cargo.exe'; CommandLine = 'cargo test'; ExecutablePath = 'C:\Rust\cargo.exe'
                ProcessId = 100; ParentProcessId = 20
            }
            [pscustomobject]@{
                Name = 'rustc.exe'; CommandLine = 'rustc --test'; ExecutablePath = 'C:\Rust\rustc.exe'
                ProcessId = 101; ParentProcessId = 100
            }
        }

        $result = Test-HistoricalCargoProcessesIdle

        $result.Reason | Should Be "cargo_process_live"
        ($result.ChildProcessIds -contains 101) | Should Be $true
    }
}
