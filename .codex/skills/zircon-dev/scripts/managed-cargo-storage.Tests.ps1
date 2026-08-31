$script:ManagedCargoStorageValidator = Join-Path $PSScriptRoot "validate-matrix.ps1"
$script:ManagedCargoStorageRepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
$script:OriginalManagedCargoStorageTestMode = $env:VALIDATE_MATRIX_TEST_MODE

$env:VALIDATE_MATRIX_TEST_MODE = "1"
. $script:ManagedCargoStorageValidator -DryRun -SkipBuild -SkipTest
$env:VALIDATE_MATRIX_TEST_MODE = $script:OriginalManagedCargoStorageTestMode

Describe "Managed Cargo storage modes" {
    It "defaults to a reusable hot target with compact compiler outputs" {
        $jobId = "reuse-{0}" -f [guid]::NewGuid().ToString("N")
        $targetDirectory = Join-Path "E:\cargo-targets\zircon-engine\pool" ([guid]::NewGuid().ToString("N"))
        $sccache = (Get-Command sccache -ErrorAction Stop).Source
        $names = @(
            "CARGO_TARGET_DIR",
            "CARGO_BUILD_BUILD_DIR",
            "CARGO_HOME",
            "CARGO_INCREMENTAL",
            "CARGO_PROFILE_DEV_DEBUG",
            "CARGO_PROFILE_TEST_DEBUG",
            "RUSTC_WRAPPER",
            "SCCACHE_CACHE_SIZE",
            "SCCACHE_CLIENT_SIDE",
            "SCCACHE_DIR",
            "SCCACHE_IDLE_TIMEOUT",
            "SCCACHE_IGNORE_SERVER_IO_ERROR",
            "SCCACHE_SERVER_PORT",
            "TEMP",
            "TMP",
            "TMPDIR"
        )
        $previousValues = @{}
        foreach ($name in $names) {
            $previousValues[$name] = [Environment]::GetEnvironmentVariable($name, "Process")
            [Environment]::SetEnvironmentVariable($name, "C:\caller-$($name.ToLowerInvariant())", "Process")
        }

        $lease = $null
        try {
            $lease = Push-ManagedCargoEnvironment `
                -TargetDirectory $targetDirectory `
                -JobId $jobId `
                -StorageMode "reuse" `
                -CompilerCacheExecutable $sccache

            [Environment]::GetEnvironmentVariable("CARGO_TARGET_DIR", "Process") | Should Be $lease.TargetOperationalPath
            [Environment]::GetEnvironmentVariable("CARGO_BUILD_BUILD_DIR", "Process") | Should BeNullOrEmpty
            [Environment]::GetEnvironmentVariable("CARGO_INCREMENTAL", "Process") | Should Be "0"
            [Environment]::GetEnvironmentVariable("CARGO_PROFILE_DEV_DEBUG", "Process") | Should Be "0"
            [Environment]::GetEnvironmentVariable("CARGO_PROFILE_TEST_DEBUG", "Process") | Should Be "0"
            [Environment]::GetEnvironmentVariable("RUSTC_WRAPPER", "Process") | Should Be $sccache
            [Environment]::GetEnvironmentVariable("SCCACHE_CLIENT_SIDE", "Process") | Should Be "1"
            [Environment]::GetEnvironmentVariable("SCCACHE_IDLE_TIMEOUT", "Process") | Should Be "0"
            [Environment]::GetEnvironmentVariable("SCCACHE_IGNORE_SERVER_IO_ERROR", "Process") | Should Be "1"
            [Environment]::GetEnvironmentVariable("SCCACHE_SERVER_PORT", "Process") | Should Be "42261"
            foreach ($name in @("TEMP", "TMP", "TMPDIR")) {
                [Environment]::GetEnvironmentVariable($name, "Process") | Should Be $lease.TemporaryOperationalPath
            }
            $lease.TemporaryOperationalPath | Should Not Be $lease.SccacheTemporaryOperationalPath
            $lease.SccacheServerProcessId | Should BeGreaterThan 0
            Test-Path -LiteralPath $lease.TargetOperationalPath -PathType Container | Should Be $true
            Test-Path -LiteralPath $lease.BuildOperationalPath | Should Be $false
            Test-Path -LiteralPath $lease.ScratchOperationalPath -PathType Container | Should Be $true
            Test-Path -LiteralPath $lease.SccacheTemporaryOperationalPath -PathType Container | Should Be $true
        }
        finally {
            if ($null -ne $lease) {
                Pop-ManagedCargoEnvironment -Lease $lease
            }
            if (Test-Path -LiteralPath $targetDirectory) {
                Remove-Item -LiteralPath $targetDirectory -Recurse -Force
            }
            foreach ($name in $names) {
                [Environment]::SetEnvironmentVariable($name, $previousValues[$name], "Process")
            }
        }

        Test-Path -LiteralPath $lease.ScratchOperationalPath | Should Be $false
    }

    It "uses shared bounded caches and an isolated ephemeral build directory" {
        $jobId = "compact-{0}" -f [guid]::NewGuid().ToString("N")
        $targetDirectory = Join-Path "E:\cargo-targets\zircon-engine\pool" ([guid]::NewGuid().ToString("N"))
        $sccache = (Get-Command sccache -ErrorAction Stop).Source
        $names = @(
            "CARGO_TARGET_DIR",
            "CARGO_BUILD_BUILD_DIR",
            "CARGO_HOME",
            "CARGO_INCREMENTAL",
            "CARGO_PROFILE_DEV_DEBUG",
            "CARGO_PROFILE_TEST_DEBUG",
            "RUSTC_WRAPPER",
            "SCCACHE_CACHE_SIZE",
            "SCCACHE_CLIENT_SIDE",
            "SCCACHE_DIR",
            "SCCACHE_IDLE_TIMEOUT",
            "SCCACHE_IGNORE_SERVER_IO_ERROR",
            "SCCACHE_SERVER_PORT",
            "TEMP",
            "TMP",
            "TMPDIR"
        )
        $previousValues = @{}
        foreach ($name in $names) {
            $previousValues[$name] = [Environment]::GetEnvironmentVariable($name, "Process")
            [Environment]::SetEnvironmentVariable($name, "C:\caller-$($name.ToLowerInvariant())", "Process")
        }

        $lease = $null
        try {
            $lease = Push-ManagedCargoEnvironment `
                -TargetDirectory $targetDirectory `
                -JobId $jobId `
                -StorageMode "compact" `
                -CompilerCacheExecutable $sccache

            $lease.CargoHomeDisplayPath | Should Be "E:\cargo-targets\zircon-engine\cache\cargo-home"
            $lease.SccacheDisplayPath | Should Be "E:\cargo-targets\zircon-engine\cache\sccache"
            $lease.SccacheTemporaryDisplayPath | Should Be "E:\cargo-targets\zircon-engine\cache\sccache-temporary"
            $lease.SccacheServerPort | Should Be 42261
            $lease.ScratchDisplayPath | Should Be "E:\cargo-targets\zircon-engine\scratch\$jobId"
            $lease.TemporaryDisplayPath | Should Be "E:\cargo-targets\zircon-engine\scratch\$jobId\temporary"
            $lease.BuildDisplayPath | Should Be "E:\cargo-targets\zircon-engine\scratch\$jobId\build"

            [Environment]::GetEnvironmentVariable("CARGO_TARGET_DIR", "Process") | Should Be $lease.TargetOperationalPath
            [Environment]::GetEnvironmentVariable("CARGO_BUILD_BUILD_DIR", "Process") | Should Be $lease.BuildOperationalPath
            [Environment]::GetEnvironmentVariable("CARGO_HOME", "Process") | Should Be $lease.CargoHomeOperationalPath
            [Environment]::GetEnvironmentVariable("CARGO_INCREMENTAL", "Process") | Should Be "0"
            [Environment]::GetEnvironmentVariable("CARGO_PROFILE_DEV_DEBUG", "Process") | Should Be "0"
            [Environment]::GetEnvironmentVariable("CARGO_PROFILE_TEST_DEBUG", "Process") | Should Be "0"
            [Environment]::GetEnvironmentVariable("RUSTC_WRAPPER", "Process") | Should Be $sccache
            [Environment]::GetEnvironmentVariable("SCCACHE_CACHE_SIZE", "Process") | Should Be "12G"
            [Environment]::GetEnvironmentVariable("SCCACHE_CLIENT_SIDE", "Process") | Should Be "1"
            [Environment]::GetEnvironmentVariable("SCCACHE_DIR", "Process") | Should Be $lease.SccacheOperationalPath
            [Environment]::GetEnvironmentVariable("SCCACHE_IDLE_TIMEOUT", "Process") | Should Be "0"
            [Environment]::GetEnvironmentVariable("SCCACHE_IGNORE_SERVER_IO_ERROR", "Process") | Should Be "1"
            [Environment]::GetEnvironmentVariable("SCCACHE_SERVER_PORT", "Process") | Should Be "42261"
            foreach ($name in @("TEMP", "TMP", "TMPDIR")) {
                [Environment]::GetEnvironmentVariable($name, "Process") | Should Be $lease.TemporaryOperationalPath
            }
            $lease.TemporaryOperationalPath | Should Not Be $lease.SccacheTemporaryOperationalPath

            Test-Path -LiteralPath $lease.BuildOperationalPath -PathType Container | Should Be $true
            Test-Path -LiteralPath $lease.ScratchOperationalPath -PathType Container | Should Be $true
            Test-Path -LiteralPath $lease.SccacheTemporaryOperationalPath -PathType Container | Should Be $true
        }
        finally {
            if ($null -ne $lease) {
                Pop-ManagedCargoEnvironment -Lease $lease
            }
            if (Test-Path -LiteralPath $targetDirectory) {
                Remove-Item -LiteralPath $targetDirectory -Recurse -Force
            }
            foreach ($name in $names) {
                [Environment]::SetEnvironmentVariable($name, $previousValues[$name], "Process")
            }
        }

        Test-Path -LiteralPath $lease.ScratchOperationalPath | Should Be $false
    }

    It "refuses low-space admission instead of rebuilding after cargo clean" {
        $blocked = Get-PrebuildStorageAdmissionDecision -FreeBytes 35GB -MinimumFreeBytes 35GB
        $admitted = Get-PrebuildStorageAdmissionDecision -FreeBytes 36GB -MinimumFreeBytes 35GB
        $validatorSource = Get-Content -Raw -Encoding UTF8 $script:ManagedCargoStorageValidator

        $blocked.IsAdmitted | Should Be $false
        $admitted.IsAdmitted | Should Be $true
        $validatorSource | Should Not Match 'Get-CargoCleanArgs'
        $validatorSource | Should Not Match 'Running cargo clean before build/test'
    }

    It "separates reuse, compact, and diagnostic compatibility identities" {
        $previousToolchain = $env:RUSTUP_TOOLCHAIN
        $previousTarget = $env:CARGO_BUILD_TARGET
        try {
            $env:RUSTUP_TOOLCHAIN = "stable-x86_64-pc-windows-msvc"
            $env:CARGO_BUILD_TARGET = "x86_64-pc-windows-msvc"

            $reuse = New-CargoCompatibilityJson `
                -ResolvedRepoRoot $script:ManagedCargoStorageRepoRoot `
                -DryRunMode | ConvertFrom-Json
            $compact = New-CargoCompatibilityJson `
                -ResolvedRepoRoot $script:ManagedCargoStorageRepoRoot `
                -StorageMode "compact" `
                -DryRunMode | ConvertFrom-Json
            $diagnostic = New-CargoCompatibilityJson `
                -ResolvedRepoRoot $script:ManagedCargoStorageRepoRoot `
                -StorageMode "diagnostic" `
                -DryRunMode | ConvertFrom-Json

            $reuse.build_config | Should Not Be $compact.build_config
            $reuse.build_config | Should Not Be $diagnostic.build_config
            $compact.build_config | Should Not Be $diagnostic.build_config
            $reuse.build_config | Should Match '"storage_mode":"reuse"'
            $reuse.build_config | Should Match '"cargo_incremental":"0"'
            $reuse.build_config | Should Match '"dev_debug":"0"'
            $reuse.build_config | Should Match '"test_debug":"0"'
            $reuse.build_config | Should Match '"build_dir":"persistent-target-v1"'
            $compact.build_config | Should Match '"storage_mode":"compact"'
            $compact.build_config | Should Match '"cargo_incremental":"0"'
            $compact.build_config | Should Match '"dev_debug":"0"'
            $compact.build_config | Should Match '"test_debug":"0"'
            $compact.build_config | Should Match '"build_dir":"ephemeral-v1"'
        }
        finally {
            $env:RUSTUP_TOOLCHAIN = $previousToolchain
            $env:CARGO_BUILD_TARGET = $previousTarget
        }
    }

    It "assigns a distinct sccache endpoint to every approved storage root" {
        $expected = [ordered]@{
            "D:\cargo-targets" = 42260
            "E:\cargo-targets" = 42261
            "F:\cargo-targets" = 42262
            "D:\targets"       = 42263
            "E:\targets"       = 42264
            "F:\targets"       = 42265
            "D:\ZirconBuilds"  = 42266
            "E:\ZirconBuilds"  = 42267
            "F:\ZirconBuilds"  = 42268
        }

        foreach ($entry in $expected.GetEnumerator()) {
            $paths = Resolve-ManagedCargoStoragePaths `
                -TargetDirectory (Join-Path $entry.Key "zircon-engine\pool\test") `
                -JobId "endpoint-contract"

            $paths.SccacheServerPort | Should Be $entry.Value
        }
    }

    It "keeps a shared sccache server independent from retired job scratch" {
        $targetDirectory = Join-Path "E:\cargo-targets\zircon-engine\pool" ([guid]::NewGuid().ToString("N"))
        $sccache = (Get-Command sccache -ErrorAction Stop).Source
        $rustc = (Get-Command rustc -ErrorAction Stop).Source
        $activeLease = $null
        $retiredScratch = @()

        try {
            foreach ($sequence in 1..2) {
                $jobId = "sccache-lifecycle-{0}-{1}" -f $sequence, [guid]::NewGuid().ToString("N")
                $activeLease = Push-ManagedCargoEnvironment `
                    -TargetDirectory $targetDirectory `
                    -JobId $jobId `
                    -StorageMode "reuse" `
                    -CompilerCacheExecutable $sccache
                $retiredScratch += $activeLease.ScratchOperationalPath

                $source = Join-Path $activeLease.TemporaryOperationalPath "probe.rs"
                $output = Join-Path $activeLease.TemporaryOperationalPath "probe.rmeta"
                [System.IO.File]::WriteAllText($source, "pub fn managed_sccache_probe() -> u32 { 42 }`n")

                $compilerOutput = @(
                    & $sccache $rustc `
                        "--crate-name" "managed_sccache_probe" `
                        "--crate-type" "lib" `
                        "--edition=2021" `
                        "--emit=metadata" `
                        "-o" $output `
                        $source 2>&1
                )
                $LASTEXITCODE | Should Be 0
                $compilerOutput -join "`n" | Should Not Match "failed to write dependency file"
                Test-Path -LiteralPath $output -PathType Leaf | Should Be $true

                Pop-ManagedCargoEnvironment -Lease $activeLease
                $activeLease = $null
                Test-Path -LiteralPath $retiredScratch[-1] | Should Be $false
            }
        }
        finally {
            if ($null -ne $activeLease) {
                Pop-ManagedCargoEnvironment -Lease $activeLease
            }
            if (Test-Path -LiteralPath $targetDirectory) {
                Remove-Item -LiteralPath $targetDirectory -Recurse -Force
            }
        }

        foreach ($scratch in $retiredScratch) {
            Test-Path -LiteralPath $scratch | Should Be $false
        }
    }

}

Describe "Managed sccache stale binding" {
    It "rebinds a stale sccache daemon before a dependency and link request" {
        $sccache = (Get-Command sccache -ErrorAction Stop).Source
        $rustc = (Get-Command rustc -ErrorAction Stop).Source
        $testRoot = Join-Path ([System.IO.Path]::GetTempPath()) (
            "zircon-sccache-rebind-{0}" -f [guid]::NewGuid().ToString("N")
        )
        $cache = Join-Path $testRoot "cache"
        $retiredTemporary = Join-Path $testRoot "retired-job-temporary"
        $stableTemporary = Join-Path $testRoot "stable-server-temporary"
        $currentTemporary = Join-Path $testRoot "current-job-temporary"
        $outputDirectory = Join-Path $testRoot "output"
        $listener = [System.Net.Sockets.TcpListener]::new(
            [System.Net.IPAddress]::Loopback,
            0
        )
        $listener.Start()
        $serverPort = ([System.Net.IPEndPoint]$listener.LocalEndpoint).Port
        $listener.Stop()
        $names = @(
            "SCCACHE_CACHE_SIZE",
            "SCCACHE_CLIENT_SIDE",
            "SCCACHE_DIR",
            "SCCACHE_IDLE_TIMEOUT",
            "SCCACHE_SERVER_PORT",
            "TEMP",
            "TMP",
            "TMPDIR"
        )
        $previousValues = @{}
        foreach ($name in $names) {
            $previousValues[$name] = [Environment]::GetEnvironmentVariable($name, "Process")
        }

        try {
            foreach ($path in @($cache, $retiredTemporary, $currentTemporary, $outputDirectory)) {
                [System.IO.Directory]::CreateDirectory($path) | Out-Null
            }
            [Environment]::SetEnvironmentVariable("SCCACHE_CACHE_SIZE", "256M", "Process")
            [Environment]::SetEnvironmentVariable("SCCACHE_CLIENT_SIDE", "1", "Process")
            [Environment]::SetEnvironmentVariable("SCCACHE_DIR", $cache, "Process")
            [Environment]::SetEnvironmentVariable("SCCACHE_IDLE_TIMEOUT", "0", "Process")
            [Environment]::SetEnvironmentVariable("SCCACHE_SERVER_PORT", [string]$serverPort, "Process")
            foreach ($name in @("TEMP", "TMP", "TMPDIR")) {
                [Environment]::SetEnvironmentVariable($name, $retiredTemporary, "Process")
            }

            $serverStart = @(& $sccache --start-server 2>&1)
            $LASTEXITCODE | Should Be 0
            $serverStart -join "`n" | Should Match "Listening on address"
            Remove-Item -LiteralPath $retiredTemporary -Recurse -Force

            $binding = Initialize-ManagedCompilerCacheServer `
                -CompilerCacheExecutable $sccache `
                -SccacheDirectory $cache `
                -StableTemporaryDirectory $stableTemporary `
                -ServerPort $serverPort `
                -CacheSize "256M"

            $binding.ServerProcessId | Should BeGreaterThan 0
            $binding.Restarted | Should Be $true
            $binding.StableTemporaryDirectory | Should Be $stableTemporary
            Test-Path -LiteralPath $stableTemporary -PathType Container | Should Be $true
            Test-Path -LiteralPath $retiredTemporary | Should Be $false
            Test-Path -LiteralPath $binding.BindingMarkerPath -PathType Leaf | Should Be $true
            $marker = Get-Content -Raw -Encoding UTF8 -LiteralPath $binding.BindingMarkerPath |
                ConvertFrom-Json
            [int]$marker.server_process_id | Should Be $binding.ServerProcessId
            [long]$marker.server_started_at_utc_ticks | Should Be (
                Get-Process -Id $binding.ServerProcessId
            ).StartTime.ToUniversalTime().Ticks
            [string]$marker.cache_size | Should Be "256M"
            [string]$marker.cache_directory | Should Be $cache
            [string]$marker.stable_temporary_directory | Should Be $stableTemporary
            Test-ManagedCompilerCacheServerEndpoint -ServerPort $serverPort | Should Be $true

            $reusedBinding = Initialize-ManagedCompilerCacheServer `
                -CompilerCacheExecutable $sccache `
                -SccacheDirectory $cache `
                -StableTemporaryDirectory $stableTemporary `
                -ServerPort $serverPort `
                -CacheSize "256M"
            $reusedBinding.ServerProcessId | Should Be $binding.ServerProcessId
            $reusedBinding.Restarted | Should Be $false

            $marker.cache_directory = "\\?\$cache"
            $marker.stable_temporary_directory = "\\?\$stableTemporary"
            $marker.compiler_cache_executable = "\\?\$sccache"
            [System.IO.File]::WriteAllText(
                $binding.BindingMarkerPath,
                ($marker | ConvertTo-Json -Compress)
            )
            $displayPathBinding = Initialize-ManagedCompilerCacheServer `
                -CompilerCacheExecutable $sccache `
                -SccacheDirectory $cache `
                -StableTemporaryDirectory $stableTemporary `
                -ServerPort $serverPort `
                -CacheSize "256M"
            $displayPathBinding.ServerProcessId | Should Be $binding.ServerProcessId
            $displayPathBinding.Restarted | Should Be $false

            $marker.cache_directory = $cache
            $marker.stable_temporary_directory = $stableTemporary
            $marker.compiler_cache_executable = $sccache
            [System.IO.File]::WriteAllText(
                $binding.BindingMarkerPath,
                ($marker | ConvertTo-Json -Compress)
            )
            $extendedPathBinding = Initialize-ManagedCompilerCacheServer `
                -CompilerCacheExecutable "\\?\$sccache" `
                -SccacheDirectory "\\?\$cache" `
                -StableTemporaryDirectory "\\?\$stableTemporary" `
                -ServerPort $serverPort `
                -CacheSize "256M"
            $extendedPathBinding.ServerProcessId | Should Be $binding.ServerProcessId
            $extendedPathBinding.Restarted | Should Be $false

            foreach ($name in @("TEMP", "TMP", "TMPDIR")) {
                [Environment]::SetEnvironmentVariable($name, $currentTemporary, "Process")
            }
            $source = Join-Path $currentTemporary "probe.rs"
            [System.IO.File]::WriteAllText(
                $source,
                "pub fn managed_sccache_link_probe() -> u32 { 42 }`n"
            )
            $metadata = [guid]::NewGuid().ToString("N")
            $compilerOutput = @(
                & $sccache $rustc `
                    "--crate-name" "managed_sccache_link_probe" `
                    "--crate-type" "lib" `
                    "--edition=2021" `
                    "--emit=dep-info,metadata,link" `
                    "-C" "metadata=$metadata" `
                    "-C" "extra-filename=-$metadata" `
                    "--out-dir" $outputDirectory `
                    $source 2>&1
            )

            $LASTEXITCODE | Should Be 0
            $compilerOutput -join "`n" | Should Not Match "Failed to create temp dir"
            @(Get-ChildItem -LiteralPath $outputDirectory -Filter "*.d").Count | Should BeGreaterThan 0
            @(Get-ChildItem -LiteralPath $outputDirectory -Filter "*.rlib").Count | Should BeGreaterThan 0
        }
        finally {
            [Environment]::SetEnvironmentVariable("SCCACHE_SERVER_PORT", [string]$serverPort, "Process")
            & $sccache --stop-server 2>&1 | Out-Null
            foreach ($name in $names) {
                [Environment]::SetEnvironmentVariable($name, $previousValues[$name], "Process")
            }
            if (Test-Path -LiteralPath $testRoot) {
                Remove-Item -LiteralPath $testRoot -Recurse -Force
            }
        }
    }
}
