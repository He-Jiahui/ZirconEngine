$validatorPath = Join-Path $PSScriptRoot 'validate-matrix.ps1'
$previousTestMode = $env:VALIDATE_MATRIX_TEST_MODE
try {
    $env:VALIDATE_MATRIX_TEST_MODE = '1'
    . $validatorPath -Package zircon_runtime -CheckOnly -DryRun
} finally {
    $env:VALIDATE_MATRIX_TEST_MODE = $previousTestMode
}

Describe 'Validator exit status with Cargo output' {
    BeforeEach {
        Mock New-CargoCompatibilityJson { return '{}' }
        Mock Resolve-ManagedCompilerCacheExecutable { return 'sccache' }
        Mock Resolve-CoordinatorCargoTarget {
            return [pscustomobject]@{
                JobId = 'status-test'
                TargetDir = 'E:\cargo-targets\zircon-engine\status-test'
                AbsoluteTargetDir = 'E:\cargo-targets\zircon-engine\status-test'
                Reason = 'mocked coordinator target'
                OwnerId = 'validate-matrix:status-test'
                DryRun = $true
            }
        }
        Mock Start-CoordinatorCargoTarget {}
        Mock Complete-CoordinatorCargoTarget {}
    }

    It 'returns one nonzero status after a failed check emits diagnostics and metrics' {
        Mock Invoke-Cargo {
            Write-Output 'compiler diagnostic'
            Write-Output '[managed-build-metrics] {"exitCode":101}'
            $global:LASTEXITCODE = 101
        }

        $result = @(Invoke-ValidateMatrixMain)

        $result.Count | Should Be 1
        $result[0] | Should Be 1
        Assert-MockCalled Complete-CoordinatorCargoTarget -Times 1 -Exactly -Scope It -ParameterFilter {
            $ExitCode -eq 1
        }
    }

    It 'returns one zero status after a successful check emits output' {
        Mock Invoke-Cargo {
            Write-Output '[managed-build-metrics] {"exitCode":0}'
            $global:LASTEXITCODE = 0
        }

        $result = @(Invoke-ValidateMatrixMain)

        $result.Count | Should Be 1
        $result[0] | Should Be 0
        Assert-MockCalled Complete-CoordinatorCargoTarget -Times 1 -Exactly -Scope It -ParameterFilter {
            $ExitCode -eq 0
        }
    }

    It 'records an interrupted native stage as failed when the output consumer stops early' {
        Mock Invoke-Cargo {
            & python -c 'import sys; sys.stderr.write("compiler diagnostic\n"); raise SystemExit(101)'
        }
        Mock Complete-CoordinatorCargoTarget { $global:LASTEXITCODE = 0 }
        $initialLocation = (Get-Location).Path

        @(Invoke-ValidateMatrixMain 2>&1 | Select-Object -First 1) | Out-Null

        Assert-MockCalled Complete-CoordinatorCargoTarget -Times 1 -Exactly -Scope It -ParameterFilter {
            $ExitCode -eq 1
        }
        $LASTEXITCODE | Should Be 1
        $script:Results.Count | Should Be 1
        $script:Results[0].ExitCode | Should Be 1
        (Get-Location).Path | Should Be $initialLocation
    }

    It 'preserves a failed status when the log destination cannot be opened' {
        Mock Invoke-Cargo {
            & python -c 'import sys; sys.stderr.write("compiler diagnostic\n"); raise SystemExit(101)'
        }
        Mock Complete-CoordinatorCargoTarget { $global:LASTEXITCODE = 0 }
        $failed = $false
        try {
            Invoke-ValidateMatrixMain 2>&1 | Tee-Object -FilePath (Join-Path $TestDrive 'missing/output.log')
        } catch {
            $failed = $true
        }

        $failed | Should Be $true
        Assert-MockCalled Complete-CoordinatorCargoTarget -Times 1 -Exactly -Scope It -ParameterFilter {
            $ExitCode -eq 1
        }
        $LASTEXITCODE | Should Be 1
    }

    It 'fails interrupted setup even when no Cargo stage has started' {
        Mock Start-CoordinatorCargoTarget {
            & python -c 'import sys; sys.stderr.write("setup diagnostic\n")'
        }
        Mock Complete-CoordinatorCargoTarget { $global:LASTEXITCODE = 0 }

        @(Invoke-ValidateMatrixMain 2>&1 | Select-Object -First 1) | Out-Null

        Assert-MockCalled Complete-CoordinatorCargoTarget -Times 1 -Exactly -Scope It -ParameterFilter {
            $ExitCode -eq 1
        }
        $LASTEXITCODE | Should Be 1
        $script:Results.Count | Should Be 0
    }
}
