$previousTestMode = $env:VALIDATE_MATRIX_TEST_MODE
$env:VALIDATE_MATRIX_TEST_MODE = '1'
. (Join-Path $PSScriptRoot 'validate-matrix.ps1') -DryRun -SkipBuild -SkipTest
$env:VALIDATE_MATRIX_TEST_MODE = $previousTestMode

Describe 'Shared compiler environment policy' {
    It 'clears ambient overrides, applies managed values, and restores the caller' {
        $names = @('CFLAGS', 'CARGO_PROFILE_DEV_OPT_LEVEL', 'RUSTC_WRAPPER')
        $previous = @{}
        foreach ($name in $names) {
            $previous[$name] = [Environment]::GetEnvironmentVariable($name, 'Process')
            [Environment]::SetEnvironmentVariable($name, 'caller-value', 'Process')
        }
        $target = Join-Path 'E:\cargo-targets\zircon-engine\pool' ([guid]::NewGuid().ToString('N'))
        $lease = $null
        try {
            $lease = Push-ManagedCargoEnvironment -TargetDirectory $target -JobId ([guid]::NewGuid().ToString('N')) `
                -StorageMode diagnostic -CompilerCacheExecutable '' -ClearEnvironment $names `
                -BuildEnvironment @{ RUSTC_WRAPPER = 'managed-cache' }
            $env:CFLAGS | Should BeNullOrEmpty
            $env:CARGO_PROFILE_DEV_OPT_LEVEL | Should BeNullOrEmpty
            $env:RUSTC_WRAPPER | Should Be 'managed-cache'
            Pop-ManagedCargoEnvironment -Lease $lease
            $lease = $null
            foreach ($name in $names) {
                [Environment]::GetEnvironmentVariable($name, 'Process') | Should Be 'caller-value'
            }
        } finally {
            if ($lease) { Pop-ManagedCargoEnvironment -Lease $lease }
            if (Test-Path -LiteralPath $target) { Remove-Item -LiteralPath $target -Recurse -Force }
            foreach ($name in $names) {
                [Environment]::SetEnvironmentVariable($name, $previous[$name], 'Process')
            }
        }
    }
}
