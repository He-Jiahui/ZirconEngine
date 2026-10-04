$script:ManagedCargoStorageValidator = Join-Path $PSScriptRoot "validate-matrix.ps1"
$script:ManagedCargoStorageRepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
$script:OriginalManagedCargoStorageTestMode = $env:VALIDATE_MATRIX_TEST_MODE

$env:VALIDATE_MATRIX_TEST_MODE = "1"
. $script:ManagedCargoStorageValidator -DryRun -SkipBuild -SkipTest
$env:VALIDATE_MATRIX_TEST_MODE = $script:OriginalManagedCargoStorageTestMode

Describe "Managed Cargo storage modes" {
    BeforeEach {
        # Environment tests model an already prepared formal binding. They do not
        # own or start a shared daemon; native ownership is covered by its verifier.
        Mock Assert-ManagedPreparedCompilerCacheBinding {
            $paths = Resolve-ManagedCargoStoragePaths -TargetDirectory $TargetDirectory -JobId $JobId
            return [pscustomobject]@{ ServerPort = $paths.SccacheServerPort; ServerProcessId = 1234 }
        }
    }

    It "uses display paths for short Cargo tool paths and keeps verbatim paths for long ones" {
        $short = [pscustomobject]@{
            DisplayPath = "D:\cargo-targets\zircon-engine\pool\short"
            OperationalPath = "\\?\D:\cargo-targets\zircon-engine\pool\short"
        }
        ConvertTo-ManagedCargoToolPath -PathResolution $short | Should Be $short.DisplayPath

        $longDisplay = "D:\" + ("deep\" * 80) + "target"
        $long = [pscustomobject]@{
            DisplayPath = $longDisplay
            OperationalPath = "\\?\" + $longDisplay
        }
        ConvertTo-ManagedCargoToolPath -PathResolution $long | Should Be $long.OperationalPath
    }

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
                -CompilerCacheExecutable $sccache `
                -RepoRoot $script:ManagedCargoStorageRepoRoot -SessionId "model-owner" -PreparedCompilerCacheBinding ([pscustomobject]@{ status = "ready" })

            [Environment]::GetEnvironmentVariable("CARGO_TARGET_DIR", "Process") | Should Be $lease.TargetDisplayPath
            [Environment]::GetEnvironmentVariable("CARGO_BUILD_BUILD_DIR", "Process") | Should BeNullOrEmpty
            [Environment]::GetEnvironmentVariable("CARGO_INCREMENTAL", "Process") | Should BeNullOrEmpty
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
                -CompilerCacheExecutable $sccache `
                -RepoRoot $script:ManagedCargoStorageRepoRoot -SessionId "model-owner" -PreparedCompilerCacheBinding ([pscustomobject]@{ status = "ready" })

            $lease.CargoHomeDisplayPath | Should Be (Join-Path $targetDirectory '.zircon-compile\cargo-home')
            $lease.SccacheDisplayPath | Should Be "E:\cargo-targets\zircon-engine\cache\sccache"
            $lease.SccacheTemporaryDisplayPath | Should Be "E:\cargo-targets\zircon-engine\cache\sccache-temporary"
            $lease.SccacheServerPort | Should Be 42261
            $lease.ScratchDisplayPath | Should Be "E:\cargo-targets\zircon-engine\scratch\$jobId"
            $lease.TemporaryDisplayPath | Should Be "E:\cargo-targets\zircon-engine\scratch\$jobId\temporary"
            $lease.BuildDisplayPath | Should Be "E:\cargo-targets\zircon-engine\scratch\$jobId\build"

            [Environment]::GetEnvironmentVariable("CARGO_TARGET_DIR", "Process") | Should Be $lease.TargetDisplayPath
            [Environment]::GetEnvironmentVariable("CARGO_BUILD_BUILD_DIR", "Process") | Should Be $lease.BuildDisplayPath
            [Environment]::GetEnvironmentVariable("CARGO_HOME", "Process") | Should Be $lease.CargoHomeDisplayPath
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
            $reuse.build_config | Should Match '"CARGO_INCREMENTAL":null'
            $reuse.build_config | Should Match '"CARGO_PROFILE_DEV_DEBUG":"0"'
            $reuse.build_config | Should Match '"CARGO_PROFILE_TEST_DEBUG":"0"'
            $compact.build_config | Should Match '"storage_mode":"compact"'
            $compact.build_config | Should Match '"CARGO_INCREMENTAL":"0"'
            $compact.build_config | Should Match '"CARGO_PROFILE_DEV_DEBUG":"0"'
            $compact.build_config | Should Match '"CARGO_PROFILE_TEST_DEBUG":"0"'
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
        }

        foreach ($entry in $expected.GetEnumerator()) {
            $paths = Resolve-ManagedCargoStoragePaths `
                -TargetDirectory (Join-Path $entry.Key "zircon-engine\pool\test") `
                -JobId "endpoint-contract"

            $paths.SccacheServerPort | Should Be $entry.Value
        }
    }

    It "rejects standalone cache consumers instead of starting a shared daemon" {
        # Shared daemon startup belongs to the formal preparation lifecycle now.
        Mock Assert-ManagedPreparedCompilerCacheBinding { throw 'a formal prepared binding is required' }
        Mock Initialize-ManagedCompilerCacheServer { throw 'consumer must not initialize' }
        { Push-ManagedCargoEnvironment -TargetDirectory 'D:\cargo-targets\zircon-engine\pool\unprepared' -JobId 'unprepared' -StorageMode reuse -CompilerCacheExecutable 'C:\tools\sccache.exe' } | Should Throw
        Assert-MockCalled Initialize-ManagedCompilerCacheServer -Times 0
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

Describe "Prepared compiler cache consumer" {
    BeforeEach {
        Mock Invoke-ManagedCompilerCachePython {
            return [pscustomobject]@{ status = 'ready'; daemonPid = 123; serverPort = 42260; cacheDirectory = 'D:\cargo-targets\zircon-engine\cache\sccache'; temporaryDirectory = 'D:\cargo-targets\zircon-engine\cache\sccache-temporary'; executable = 'C:\tools\sccache.exe'; bindingMarkerPath = 'D:\cargo-targets\zircon-engine\cache\sccache-temporary\server-binding-v1.json' }
        }
        Mock Initialize-ManagedCompilerCacheServer { throw 'consumer must never initialize a daemon' }
    }
    It "derives shared cache from the nearest managed namespace" {
        $paths = Resolve-ManagedCargoStoragePaths -TargetDirectory 'D:\cargo-targets\fixture\zircon-engine\outer\zircon-engine\pool\target' -JobId 'nested'
        $paths.Sccache.DisplayPath | Should Be 'D:\cargo-targets\fixture\zircon-engine\outer\zircon-engine\cache\sccache'
        $paths.SccacheTemporary.DisplayPath | Should Be 'D:\cargo-targets\fixture\zircon-engine\outer\zircon-engine\cache\sccache-temporary'
        $paths.SccacheServerPort | Should Be 42260
    }

    It "requires prepared binding before creating reusable storage" {
        { Push-ManagedCargoEnvironment -TargetDirectory 'D:\cargo-targets\zircon-engine\pool\consumer-test' -JobId 'consumer-test' -StorageMode reuse -CompilerCacheExecutable 'C:\tools\sccache.exe' } | Should Throw
        Assert-MockCalled Initialize-ManagedCompilerCacheServer -Times 0
    }
    It "verifies the binding through the read only native ownership contract" {
        $result = Assert-ManagedPreparedCompilerCacheBinding -RepoRoot $script:ManagedCargoStorageRepoRoot -SessionId 'owner' -JobId 'consumer-test' -TargetDirectory 'D:\cargo-targets\zircon-engine\pool\consumer-test' -CompilerCacheExecutable 'C:\tools\sccache.exe' -Binding ([pscustomobject]@{ status = 'ready' })
        $result.ServerProcessId | Should Be 123
        Assert-MockCalled Invoke-ManagedCompilerCachePython -Times 1 -ParameterFilter { $Operation -eq 'verify' -and $Payload.arguments.session_id -eq 'owner' -and $Payload.arguments.job_id -eq 'consumer-test' }
        Assert-MockCalled Initialize-ManagedCompilerCacheServer -Times 0
    }
    It "rejects native owner or daemon drift without initialization" {
        Mock Invoke-ManagedCompilerCachePython { throw 'native identity changed' }
        { Assert-ManagedPreparedCompilerCacheBinding -RepoRoot $script:ManagedCargoStorageRepoRoot -SessionId 'owner' -JobId 'consumer-test' -TargetDirectory 'D:\cargo-targets\zircon-engine\pool\consumer-test' -CompilerCacheExecutable 'C:\tools\sccache.exe' -Binding ([pscustomobject]@{ status = 'ready' }) } | Should Throw
        Assert-MockCalled Initialize-ManagedCompilerCacheServer -Times 0
    }
    It "rejects a daemon executable outside the consumer contract" {
        { Assert-ManagedPreparedCompilerCacheBinding -RepoRoot $script:ManagedCargoStorageRepoRoot -SessionId 'owner' -JobId 'consumer-test' -TargetDirectory 'D:\cargo-targets\zircon-engine\pool\consumer-test' -CompilerCacheExecutable 'C:\other\sccache.exe' -Binding ([pscustomobject]@{ status = 'ready' }) } | Should Throw
    }
}
