$validatorPath = Join-Path $PSScriptRoot 'validate-matrix.ps1'
$previousTestMode = $env:VALIDATE_MATRIX_TEST_MODE
try {
    $env:VALIDATE_MATRIX_TEST_MODE = '1'
    . $validatorPath -Package zircon_runtime -LibTests -Features target-server -TestFilter first -DryRun
} finally {
    $env:VALIDATE_MATRIX_TEST_MODE = $previousTestMode
}

function Get-PipelineCheckCommand([string[]]$TestCommand) {
    # Windows PowerShell 5.1 strips the quotes in JSON passed to native tools.
    $encoded = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes(
        (ConvertTo-Json -InputObject $TestCommand -Compress)))
    & python -c 'import base64, json, sys; from tools.session_coordinator.cargo_pipeline import check_command; print(json.dumps(check_command(json.loads(base64.b64decode(sys.argv[1])))))' $encoded
}

Describe 'Validator and Cargo pipeline check identity' {
    It 'keeps profiling features on the system linker unless LLD is explicit' {
        $policyRepo = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..\..')).Path
        foreach ($feature in @('profiling', 'zircon_runtime/profiling-memory')) {
            $command = @('test', '-p', 'zircon_runtime', '--lib', '--features', $feature)
            $automatic = Get-ManagedBuildPolicy -RepoRoot $policyRepo -Command $command -DryRunMode
            $system = Get-ManagedBuildPolicy -RepoRoot $policyRepo -Command $command -Linker system -DryRunMode
            $lld = Get-ManagedBuildPolicy -RepoRoot $policyRepo -Command $command -Linker lld -DryRunMode

            $automatic.environment.ZIRCON_LINK_MODE | Should Be 'static'
            ($automatic.compatibility.build_config | ConvertFrom-Json).linker | Should Be 'system'
            $automatic.compatibility.build_config | Should Be $system.compatibility.build_config
            $lld.environment.CARGO_ENCODED_RUSTFLAGS | Should Match 'linker-flavor=lld-link'
        }
    }

    It 'keeps explicit Cargo targets separate from the implicit host pool' {
        $policyRepo = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..\..')).Path
        $implicit = Get-ManagedBuildPolicy -RepoRoot $policyRepo `
            -Command @('build', '-p', 'zircon_app') -DryRunMode
        $explicit = Get-ManagedBuildPolicy -RepoRoot $policyRepo `
            -Command @('build', '-p', 'zircon_app', '--target', 'x86_64-pc-windows-msvc') -DryRunMode

        $implicit.cargoTarget | Should BeNullOrEmpty
        $explicit.cargoTarget | Should Be 'x86_64-pc-windows-msvc'
        ($implicit.compatibility | ConvertTo-Json -Compress) |
            Should Not Be ($explicit.compatibility | ConvertTo-Json -Compress)
    }

    It 'adds optional timing reports without changing compiler compatibility' {
        $previousTimings = $script:CargoTimings
        $policyRepo = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..\..')).Path
        try {
            $script:CargoTimings = $false
            $normal = @(Get-CargoArgs -Subcommand test -WorkspaceManifest 'Cargo.toml')
            $script:CargoTimings = $true
            $timed = @(Get-CargoArgs -Subcommand test -WorkspaceManifest 'Cargo.toml')
            ($normal -contains '--timings') | Should Be $false
            ($timed -contains '--timings') | Should Be $true
            $normalPolicy = Get-ManagedBuildPolicy -RepoRoot $policyRepo -Command $normal -DryRunMode
            $timedPolicy = Get-ManagedBuildPolicy -RepoRoot $policyRepo -Command $timed -DryRunMode
            ($normalPolicy.compatibility | ConvertTo-Json -Compress) |
                Should Be ($timedPolicy.compatibility | ConvertTo-Json -Compress)
            foreach ($stage in @('check', 'build', 'rustc')) {
                (@(Get-CargoArgs -Subcommand $stage -WorkspaceManifest 'Cargo.toml') -contains '--timings') |
                    Should Be $true
            }
            (@(Get-CargoArgs -Subcommand run -WorkspaceManifest 'Cargo.toml') -contains '--timings') |
                Should Be $false
        } finally {
            $script:CargoTimings = $previousTimings
        }
    }

    It 'passes the explicit check result to the test pipeline' {
        @(Get-CargoPipelineOptions -MetricsPath 'receipt.json') |
            Should Be @('--receipt', 'receipt.json')
        @(Get-CargoPipelineOptions -MetricsPath 'receipt.json' -SkipCheck) |
            Should Be @('--receipt', 'receipt.json', '--skip-check')
    }

    It 'checks test configuration for release and profiling library tests' {
        foreach ($profile in @('release', 'profiling')) {
            $parameters = @{
                ResolvedTargetDir = 'E:\cargo-targets\zircon-engine\pool\test'
                WorkspaceManifest = 'Cargo.toml'
                CargoProfile = $profile
            }
            $check = @('cargo') + @(Get-CargoArgs -Subcommand check @parameters)
            $test = @('cargo') + @(Get-CargoArgs -Subcommand test @parameters)
            $pipelineCheck = Get-PipelineCheckCommand $test
            $LASTEXITCODE | Should Be 0
            ($check -contains '--tests') | Should Be $true
            ($check -join [char]31) | Should Be (($pipelineCheck | ConvertFrom-Json) -join [char]31)
        }
    }

    It 'uses the same check scope before a development dynamic library test' {
        $script:ResolvedLinkMode = 'dev-dynamic'
        $policyRepo = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..\..')).Path
        $script:ManagedBuildPolicy = Get-ManagedBuildPolicy -RepoRoot $policyRepo `
            -Command @('test', '-p', 'zircon_runtime', '--lib') -LinkMode dev-dynamic -DryRunMode
        try {
            $parameters = @{
                ResolvedTargetDir = 'E:\cargo-targets\zircon-engine\pool\test'
                WorkspaceManifest = 'Cargo.toml'
                CargoProfile = 'development'
            }
            $check = @('cargo') + @(Get-CargoArgs -Subcommand check @parameters)
            $test = @('cargo') + @(Get-CargoArgs -Subcommand test @parameters)
            $pipelineCheck = Get-PipelineCheckCommand $test
            $LASTEXITCODE | Should Be 0

            ($check -join [char]31) | Should Be (($pipelineCheck | ConvertFrom-Json) -join [char]31)
            foreach ($setting in @("profile.dev.package.'*'.opt-level=3",
                'profile.dev.package.zr_dev_deps_dylib.opt-level=1',
                'profile.dev.package.zr_runtime_dev_dylib.opt-level=1',
                "profile.test.package.'*'.opt-level=3",
                'profile.test.package.zr_dev_deps_dylib.opt-level=1',
                'profile.test.package.zr_runtime_dev_dylib.opt-level=1')) {
                ($check -contains $setting) | Should Be $true
                ($test -contains $setting) | Should Be $true
            }
        } finally {
            Remove-Variable ResolvedLinkMode -Scope Script -ErrorAction SilentlyContinue
            Remove-Variable ManagedBuildPolicy -Scope Script -ErrorAction SilentlyContinue
        }
    }

    It 'propagates the Runtime DLL export profile to both host validation stages' {
        $previousPackage = $script:Package
        $script:ResolvedLinkMode = 'dev-dynamic'
        $policyRepo = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..\..')).Path
        try {
            foreach ($hostPackage in @('zircon_app', 'zircon_editor')) {
                $script:Package = $hostPackage
                $script:ManagedBuildPolicy = Get-ManagedBuildPolicy -RepoRoot $policyRepo `
                    -Command @('test', '-p', $hostPackage, '--lib') -LinkMode dev-dynamic -DryRunMode
                foreach ($stage in @('check', 'test')) {
                    $arguments = @(Get-CargoArgs -Subcommand $stage -CargoProfile development `
                        -ResolvedTargetDir 'E:\cargo-targets\zircon-engine\pool\test' `
                        -WorkspaceManifest 'Cargo.toml')
                    foreach ($profile in @('dev', 'test')) {
                        ($arguments -contains "profile.$profile.package.zircon_runtime.opt-level=3") |
                            Should Be $true
                    }
                    ($arguments -contains "$hostPackage/dev-dynamic-linking") | Should Be $true
                }
            }
        } finally {
            $script:Package = $previousPackage
            Remove-Variable ResolvedLinkMode -Scope Script -ErrorAction SilentlyContinue
            Remove-Variable ManagedBuildPolicy -Scope Script -ErrorAction SilentlyContinue
        }
    }
}
