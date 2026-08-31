$script:ManagedCargoMinimumFreeSpaceBytes = 35GB
$script:ManagedCargoSccacheSize = "12G"
$script:ManagedCargoSccachePorts = @{
    "D:cargo-targets" = 42260
    "E:cargo-targets" = 42261
    "F:cargo-targets" = 42262
    "D:targets"       = 42263
    "E:targets"       = 42264
    "F:targets"       = 42265
    "D:zirconbuilds"  = 42266
    "E:zirconbuilds"  = 42267
    "F:zirconbuilds"  = 42268
}

function Resolve-ManagedCompilerCacheExecutable {
    param(
        [ValidateSet("reuse", "compact", "diagnostic")]
        [string]$StorageMode = "reuse",
        [switch]$DryRunMode
    )

    if ($DryRunMode -or $StorageMode -eq "diagnostic") {
        return $null
    }

    $command = Get-Command sccache -CommandType Application -ErrorAction SilentlyContinue
    if ($null -eq $command) {
        if ($StorageMode -ne "diagnostic") {
            throw "Reusable Cargo storage requires sccache on PATH. Install sccache or use -StorageMode diagnostic."
        }
        return $null
    }
    return $command.Source
}

function Resolve-ManagedCargoStoragePaths {
    param(
        [Parameter(Mandatory)]
        [string]$TargetDirectory,
        [Parameter(Mandatory)]
        [string]$JobId
    )

    if ($JobId -notmatch '^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$') {
        throw "Managed Cargo job id is not safe for a scratch directory: '$JobId'."
    }

    $target = Resolve-ManagedCargoTargetPath -TargetDirectory $TargetDirectory
    $rootMatch = [regex]::Match(
        $target.DisplayPath,
        '^(?<root>[D-F]:\\(?:cargo-targets|targets|ZirconBuilds))(?:\\|$)',
        [System.Text.RegularExpressions.RegexOptions]::IgnoreCase
    )
    if (-not $rootMatch.Success) {
        throw "Managed Cargo target must resolve below an approved storage root: $($target.DisplayPath)"
    }

    $approvedRoot = Resolve-ZirconWindowsPath -Path $rootMatch.Groups["root"].Value
    $engineRoot = Resolve-ZirconWindowsPath -Path (
        Join-ZirconWindowsPath -Path $approvedRoot.OperationalPath -ChildPath "zircon-engine"
    )
    $cacheRoot = Resolve-ZirconWindowsPath -Path (
        Join-ZirconWindowsPath -Path $engineRoot.OperationalPath -ChildPath "cache"
    )
    $scratchRoot = Resolve-ZirconWindowsPath -Path (
        Join-ZirconWindowsPath -Path $engineRoot.OperationalPath -ChildPath "scratch"
    )
    $scratch = Resolve-ZirconWindowsPath -Path (
        Join-ZirconWindowsPath -Path $scratchRoot.OperationalPath -ChildPath $JobId
    )
    $driveLetter = $approvedRoot.DisplayPath.Substring(0, 1).ToUpperInvariant()
    $approvedRootName = [System.IO.Path]::GetFileName(
        $approvedRoot.DisplayPath.TrimEnd('\', '/')
    ).ToLowerInvariant()
    $serverPortKey = "{0}:{1}" -f $driveLetter, $approvedRootName
    if (-not $script:ManagedCargoSccachePorts.ContainsKey($serverPortKey)) {
        throw "Managed Cargo storage root has no sccache endpoint: $($approvedRoot.DisplayPath)"
    }

    return [pscustomobject]@{
        Target      = $target
        CargoHome   = Resolve-ZirconWindowsPath -Path (
            Join-ZirconWindowsPath -Path $cacheRoot.OperationalPath -ChildPath "cargo-home"
        )
        Sccache     = Resolve-ZirconWindowsPath -Path (
            Join-ZirconWindowsPath -Path $cacheRoot.OperationalPath -ChildPath "sccache"
        )
        SccacheTemporary = Resolve-ZirconWindowsPath -Path (
            Join-ZirconWindowsPath -Path $cacheRoot.OperationalPath -ChildPath "sccache-temporary"
        )
        SccacheServerPort = [int]$script:ManagedCargoSccachePorts[$serverPortKey]
        ScratchRoot = $scratchRoot
        Scratch     = $scratch
        Temporary   = Resolve-ZirconWindowsPath -Path (
            Join-ZirconWindowsPath -Path $scratch.OperationalPath -ChildPath "temporary"
        )
        Build       = Resolve-ZirconWindowsPath -Path (
            Join-ZirconWindowsPath -Path $scratch.OperationalPath -ChildPath "build"
        )
    }
}

function Get-ManagedCompilerCacheServerProcessId {
    param(
        [Parameter(Mandatory)]
        [ValidateRange(1024, 65535)]
        [int]$ServerPort
    )

    $listeners = @(
        Get-NetTCPConnection `
            -State Listen `
            -LocalPort $ServerPort `
            -ErrorAction SilentlyContinue
    )
    if ($listeners.Count -gt 1) {
        throw "Multiple processes are listening on managed sccache port $ServerPort."
    }
    if ($listeners.Count -eq 0) {
        return $null
    }
    return [int]$listeners[0].OwningProcess
}

function Test-ManagedCompilerCacheServerEndpoint {
    param(
        [Parameter(Mandatory)]
        [ValidateRange(1024, 65535)]
        [int]$ServerPort
    )

    $client = [System.Net.Sockets.TcpClient]::new()
    try {
        $client.Connect([System.Net.IPAddress]::Loopback, $ServerPort)
        return $client.Connected
    }
    catch [System.Net.Sockets.SocketException] {
        return $false
    }
    finally {
        $client.Dispose()
    }
}

function Resolve-ManagedCompilerCacheBindingMarkerPath {
    param(
        [Parameter(Mandatory)]
        [string]$StableTemporaryDirectory
    )

    $temporary = [System.IO.Path]::GetFullPath($StableTemporaryDirectory)
    return Join-ZirconWindowsPath `
        -Path $temporary `
        -ChildPath "server-binding-v1.json"
}

function Test-ManagedWindowsPathContractEqual {
    param(
        [Parameter(Mandatory)]
        [string]$LeftPath,
        [Parameter(Mandatory)]
        [string]$RightPath
    )

    if ([string]::IsNullOrWhiteSpace($LeftPath) -or
        [string]::IsNullOrWhiteSpace($RightPath)) {
        return $false
    }
    try {
        $left = Resolve-ZirconWindowsPath -Path $LeftPath
        $right = Resolve-ZirconWindowsPath -Path $RightPath
    }
    catch {
        return $false
    }
    $trimCharacters = [char[]]@('\', '/')
    return [string]::Equals(
        $left.OperationalPath.TrimEnd($trimCharacters),
        $right.OperationalPath.TrimEnd($trimCharacters),
        [System.StringComparison]::OrdinalIgnoreCase
    )
}

function Initialize-ManagedCompilerCacheServer {
    param(
        [Parameter(Mandatory)]
        [string]$CompilerCacheExecutable,
        [Parameter(Mandatory)]
        [string]$SccacheDirectory,
        [Parameter(Mandatory)]
        [string]$StableTemporaryDirectory,
        [Parameter(Mandatory)]
        [ValidateRange(1024, 65535)]
        [int]$ServerPort,
        [string]$CacheSize = $script:ManagedCargoSccacheSize
    )

    $cache = [System.IO.Path]::GetFullPath($SccacheDirectory)
    $temporary = [System.IO.Path]::GetFullPath($StableTemporaryDirectory)
    $compilerCachePath = [System.IO.Path]::GetFullPath($CompilerCacheExecutable)
    [System.IO.Directory]::CreateDirectory($cache) | Out-Null
    [System.IO.Directory]::CreateDirectory($temporary) | Out-Null
    $markerPath = Resolve-ManagedCompilerCacheBindingMarkerPath `
        -StableTemporaryDirectory $temporary
    $environmentNames = @(
        "SCCACHE_CACHE_SIZE",
        "SCCACHE_DIR",
        "SCCACHE_IDLE_TIMEOUT",
        "SCCACHE_SERVER_PORT",
        "TEMP",
        "TMP",
        "TMPDIR"
    )
    $previousValues = @{}
    foreach ($name in $environmentNames) {
        $previousValues[$name] = [Environment]::GetEnvironmentVariable($name, "Process")
    }

    $mutex = [System.Threading.Mutex]::new(
        $false,
        "Local\ZirconEngine.ManagedSccache.$ServerPort"
    )
    $mutexAcquired = $false
    try {
        try {
            $mutexAcquired = $mutex.WaitOne([TimeSpan]::FromSeconds(30))
        }
        catch [System.Threading.AbandonedMutexException] {
            $mutexAcquired = $true
        }
        if (-not $mutexAcquired) {
            throw "Timed out waiting for managed sccache lifecycle port $ServerPort."
        }

        [Environment]::SetEnvironmentVariable("SCCACHE_CACHE_SIZE", $CacheSize, "Process")
        [Environment]::SetEnvironmentVariable("SCCACHE_DIR", $cache, "Process")
        [Environment]::SetEnvironmentVariable("SCCACHE_IDLE_TIMEOUT", "0", "Process")
        [Environment]::SetEnvironmentVariable("SCCACHE_SERVER_PORT", [string]$ServerPort, "Process")
        foreach ($name in @("TEMP", "TMP", "TMPDIR")) {
            [Environment]::SetEnvironmentVariable($name, $temporary, "Process")
        }

        $binding = $null
        if (Test-Path -LiteralPath $markerPath -PathType Leaf) {
            try {
                $binding = Get-Content -Raw -Encoding UTF8 -LiteralPath $markerPath | ConvertFrom-Json
            }
            catch {
                $binding = $null
            }
        }
        $bindingConfigurationMatches = (
            $null -ne $binding -and
            [int]$binding.server_port -eq $ServerPort -and
            [string]$binding.cache_size -eq $CacheSize -and
            (Test-ManagedWindowsPathContractEqual `
                -LeftPath ([string]$binding.cache_directory) `
                -RightPath $cache) -and
            (Test-ManagedWindowsPathContractEqual `
                -LeftPath ([string]$binding.stable_temporary_directory) `
                -RightPath $temporary) -and
            (Test-ManagedWindowsPathContractEqual `
                -LeftPath ([string]$binding.compiler_cache_executable) `
                -RightPath $compilerCachePath)
        )
        $serverProcessId = if ($bindingConfigurationMatches) {
            [int]$binding.server_process_id
        } else {
            $null
        }
        $serverProcess = if ($null -ne $serverProcessId) {
            Get-Process -Id $serverProcessId -ErrorAction SilentlyContinue
        } else {
            $null
        }
        $serverStartedAt = if ($null -ne $serverProcess) {
            $serverProcess.StartTime.ToUniversalTime().ToString("O")
        } else {
            $null
        }
        $serverStartedAtUtcTicks = if ($null -ne $serverProcess) {
            $serverProcess.StartTime.ToUniversalTime().Ticks
        } else {
            $null
        }
        $bindingMatches = (
            $bindingConfigurationMatches -and
            $null -ne $serverProcess -and
            [long]$binding.server_started_at_utc_ticks -eq $serverStartedAtUtcTicks -and
            (Test-ManagedCompilerCacheServerEndpoint -ServerPort $ServerPort)
        )
        if (-not $bindingMatches) {
            $serverProcessId = Get-ManagedCompilerCacheServerProcessId `
                -ServerPort $ServerPort
        }

        $restarted = $false
        if (-not $bindingMatches -and $null -ne $serverProcessId) {
            if ($ServerPort -in @($script:ManagedCargoSccachePorts.Values)) {
                $activeCompilers = @(
                    Get-Process -Name cargo, rustc -ErrorAction SilentlyContinue
                )
                if ($activeCompilers.Count -gt 0) {
                    $activeIds = ($activeCompilers.Id | Sort-Object) -join ", "
                    throw "Managed sccache binding is stale, but Cargo/rustc processes are active ($activeIds); refusing an unsafe daemon restart."
                }
            }
            $stopOutput = @(& $CompilerCacheExecutable --stop-server 2>&1)
            if ($LASTEXITCODE -ne 0) {
                throw "Failed to stop stale managed sccache server on port $ServerPort`: $($stopOutput -join ' ')"
            }
            foreach ($attempt in 1..100) {
                if (-not (Test-ManagedCompilerCacheServerEndpoint -ServerPort $ServerPort)) {
                    break
                }
                Start-Sleep -Milliseconds 50
            }
            if (Test-ManagedCompilerCacheServerEndpoint -ServerPort $ServerPort) {
                throw "Managed sccache server on port $ServerPort did not stop."
            }
            $serverProcessId = $null
            $restarted = $true
        }

        if (-not $bindingMatches) {
            $startOutput = @(& $CompilerCacheExecutable --start-server 2>&1)
            if ($LASTEXITCODE -ne 0) {
                throw "Failed to start managed sccache server on port $ServerPort`: $($startOutput -join ' ')"
            }
            foreach ($attempt in 1..100) {
                if (Test-ManagedCompilerCacheServerEndpoint -ServerPort $ServerPort) {
                    break
                }
                Start-Sleep -Milliseconds 50
            }
            if (-not (Test-ManagedCompilerCacheServerEndpoint -ServerPort $ServerPort)) {
                throw "Managed sccache server on port $ServerPort did not become ready."
            }
            $serverProcessId = Get-ManagedCompilerCacheServerProcessId `
                -ServerPort $ServerPort
            if ($null -eq $serverProcessId) {
                throw "Managed sccache server on port $ServerPort has no process identity."
            }
            $serverProcess = Get-Process -Id $serverProcessId -ErrorAction Stop
            $serverStartedAt = $serverProcess.StartTime.ToUniversalTime().ToString("O")
            $serverStartedAtUtcTicks = $serverProcess.StartTime.ToUniversalTime().Ticks
            $marker = [ordered]@{
                schema_version             = 1
                server_port                = $ServerPort
                server_process_id          = $serverProcessId
                server_started_at          = $serverStartedAt
                server_started_at_utc_ticks = $serverStartedAtUtcTicks
                cache_size                = $CacheSize
                cache_directory            = $cache
                stable_temporary_directory = $temporary
                compiler_cache_executable  = $compilerCachePath
            } | ConvertTo-Json -Compress
            $markerTemporaryPath = "$markerPath.$PID.$([guid]::NewGuid().ToString('N')).tmp"
            [System.IO.File]::WriteAllText($markerTemporaryPath, $marker)
            Move-Item -LiteralPath $markerTemporaryPath -Destination $markerPath -Force
        }

        return [pscustomobject]@{
            ServerPort               = $ServerPort
            ServerProcessId          = $serverProcessId
            ServerStartedAt          = $serverStartedAt
            StableTemporaryDirectory = $temporary
            BindingMarkerPath        = $markerPath
            Restarted                = $restarted
        }
    }
    finally {
        foreach ($name in $environmentNames) {
            [Environment]::SetEnvironmentVariable($name, $previousValues[$name], "Process")
        }
        if ($mutexAcquired) {
            $mutex.ReleaseMutex()
        }
        $mutex.Dispose()
    }
}

function Remove-ManagedCargoScratch {
    param(
        [Parameter(Mandatory)]
        [psobject]$Lease
    )

    if (-not [System.IO.Directory]::Exists($Lease.ScratchOperationalPath)) {
        return
    }

    $scratch = Resolve-ZirconWindowsPath -Path $Lease.ScratchOperationalPath
    $scratchRoot = Resolve-ZirconWindowsPath -Path $Lease.ScratchRootOperationalPath
    $scratchPrefix = $scratchRoot.OperationalPath.TrimEnd('\', '/') + [System.IO.Path]::DirectorySeparatorChar
    if (-not $scratch.OperationalPath.StartsWith(
            $scratchPrefix,
            [System.StringComparison]::OrdinalIgnoreCase
        )) {
        throw "Managed Cargo scratch escaped its approved root: $($scratch.DisplayPath)"
    }

    $directoryLease = Open-ZirconWindowsDirectoryLease `
        -Path $scratch.OperationalPath `
        -ExpectedOperationalPath $scratch.OperationalPath `
        -ForMove `
        -NoFollow
    try {
        Remove-ZirconWindowsLeasedDirectoryTree -Lease $directoryLease
    }
    finally {
        $directoryLease.Dispose()
    }
}

function Push-ManagedCargoEnvironment {
    param(
        [Parameter(Mandatory)]
        [string]$TargetDirectory,
        [Parameter(Mandatory)]
        [string]$JobId,
        [ValidateSet("reuse", "compact", "diagnostic")]
        [string]$StorageMode = "reuse",
        [AllowEmptyString()]
        [string]$CompilerCacheExecutable
    )

    if ($StorageMode -ne "diagnostic" -and [string]::IsNullOrWhiteSpace($CompilerCacheExecutable)) {
        throw "Reusable Cargo storage requires a resolved sccache executable."
    }

    $paths = Resolve-ManagedCargoStoragePaths `
        -TargetDirectory $TargetDirectory `
        -JobId $JobId
    if ([System.IO.Directory]::Exists($paths.Scratch.OperationalPath)) {
        throw "Refusing to reuse an existing managed Cargo scratch directory: $($paths.Scratch.DisplayPath)"
    }

    $scratchCreated = $false
    $serverBinding = $null
    $previousValues = @{}
    try {
        [System.IO.Directory]::CreateDirectory($paths.Target.OperationalPath) | Out-Null
        [System.IO.Directory]::CreateDirectory($paths.CargoHome.OperationalPath) | Out-Null
        [System.IO.Directory]::CreateDirectory($paths.Sccache.OperationalPath) | Out-Null
        [System.IO.Directory]::CreateDirectory($paths.SccacheTemporary.OperationalPath) | Out-Null
        if (-not [string]::IsNullOrWhiteSpace($CompilerCacheExecutable)) {
            $serverBinding = Initialize-ManagedCompilerCacheServer `
                -CompilerCacheExecutable $CompilerCacheExecutable `
                -SccacheDirectory $paths.Sccache.OperationalPath `
                -StableTemporaryDirectory $paths.SccacheTemporary.OperationalPath `
                -ServerPort $paths.SccacheServerPort
        }
        [System.IO.Directory]::CreateDirectory($paths.Temporary.OperationalPath) | Out-Null
        $scratchCreated = $true
        if ($StorageMode -eq "compact") {
            [System.IO.Directory]::CreateDirectory($paths.Build.OperationalPath) | Out-Null
        }

        $environment = [ordered]@{
            CARGO_TARGET_DIR   = $paths.Target.OperationalPath
            CARGO_HOME         = $paths.CargoHome.OperationalPath
            SCCACHE_DIR        = $paths.Sccache.OperationalPath
            SCCACHE_CACHE_SIZE = $script:ManagedCargoSccacheSize
            SCCACHE_SERVER_PORT = if ($null -ne $serverBinding) {
                [string]$serverBinding.ServerPort
            } else {
                $null
            }
            SCCACHE_IDLE_TIMEOUT = if ($null -ne $serverBinding) { "0" } else { $null }
            TEMP               = $paths.Temporary.OperationalPath
            TMP                = $paths.Temporary.OperationalPath
            TMPDIR             = $paths.Temporary.OperationalPath
        }
        if (-not [string]::IsNullOrWhiteSpace($CompilerCacheExecutable)) {
            $environment["RUSTC_WRAPPER"] = $CompilerCacheExecutable
            $environment["SCCACHE_CLIENT_SIDE"] = "1"
            $environment["SCCACHE_IGNORE_SERVER_IO_ERROR"] = "1"
        }
        if ($StorageMode -in @("reuse", "compact")) {
            $environment["CARGO_BUILD_BUILD_DIR"] = if ($StorageMode -eq "compact") {
                $paths.Build.OperationalPath
            } else {
                $null
            }
            $environment["CARGO_INCREMENTAL"] = "0"
            $environment["CARGO_PROFILE_DEV_DEBUG"] = "0"
            $environment["CARGO_PROFILE_TEST_DEBUG"] = "0"
        }

        foreach ($name in $environment.Keys) {
            $previousValues[$name] = [Environment]::GetEnvironmentVariable($name, "Process")
            [Environment]::SetEnvironmentVariable($name, [string]$environment[$name], "Process")
        }
    }
    catch {
        foreach ($name in $previousValues.Keys) {
            [Environment]::SetEnvironmentVariable($name, $previousValues[$name], "Process")
        }
        if ($scratchCreated) {
            $failedLease = [pscustomobject]@{
                ScratchOperationalPath     = $paths.Scratch.OperationalPath
                ScratchRootOperationalPath = $paths.ScratchRoot.OperationalPath
            }
            Remove-ManagedCargoScratch -Lease $failedLease
        }
        throw
    }

    return [pscustomobject]@{
        TargetOperationalPath      = $paths.Target.OperationalPath
        TargetDisplayPath          = $paths.Target.DisplayPath
        TemporaryOperationalPath   = $paths.Temporary.OperationalPath
        TemporaryDisplayPath       = $paths.Temporary.DisplayPath
        BuildOperationalPath       = $paths.Build.OperationalPath
        BuildDisplayPath           = $paths.Build.DisplayPath
        CargoHomeOperationalPath   = $paths.CargoHome.OperationalPath
        CargoHomeDisplayPath       = $paths.CargoHome.DisplayPath
        SccacheOperationalPath     = $paths.Sccache.OperationalPath
        SccacheDisplayPath         = $paths.Sccache.DisplayPath
        SccacheTemporaryOperationalPath = $paths.SccacheTemporary.OperationalPath
        SccacheTemporaryDisplayPath = $paths.SccacheTemporary.DisplayPath
        SccacheServerPort          = $paths.SccacheServerPort
        SccacheServerProcessId     = if ($null -ne $serverBinding) {
            $serverBinding.ServerProcessId
        } else {
            $null
        }
        ScratchOperationalPath     = $paths.Scratch.OperationalPath
        ScratchDisplayPath         = $paths.Scratch.DisplayPath
        ScratchRootOperationalPath = $paths.ScratchRoot.OperationalPath
        PreviousValues             = $previousValues
    }
}

function Pop-ManagedCargoEnvironment {
    param(
        [Parameter(Mandatory)]
        [psobject]$Lease
    )

    $failure = $null
    try {
        foreach ($name in $Lease.PreviousValues.Keys) {
            [Environment]::SetEnvironmentVariable($name, $Lease.PreviousValues[$name], "Process")
        }
    }
    catch {
        $failure = $_
    }

    try {
        Remove-ManagedCargoScratch -Lease $Lease
    }
    catch {
        if ($null -eq $failure) {
            $failure = $_
        }
        else {
            Write-Warning ("Managed Cargo scratch cleanup also failed: {0}" -f $_.Exception.Message)
        }
    }

    if ($null -ne $failure) {
        throw $failure
    }
}

function Get-TargetDriveInfo {
    param([string]$AbsoluteTargetDir)

    $driveRoot = [System.IO.Path]::GetPathRoot($AbsoluteTargetDir)
    if ([string]::IsNullOrWhiteSpace($driveRoot)) {
        throw "Could not determine drive root for target directory $AbsoluteTargetDir"
    }

    $drive = [System.IO.DriveInfo]::new($driveRoot)
    return [pscustomobject]@{
        DriveRoot = $driveRoot
        FreeBytes = [int64]$drive.AvailableFreeSpace
    }
}

function Get-PrebuildStorageAdmissionDecision {
    param(
        [int64]$FreeBytes,
        [int64]$MinimumFreeBytes = $script:ManagedCargoMinimumFreeSpaceBytes
    )

    return [pscustomobject]@{
        FreeBytes        = $FreeBytes
        MinimumFreeBytes = $MinimumFreeBytes
        IsAdmitted       = ($FreeBytes -gt $MinimumFreeBytes)
    }
}

function Get-PrebuildStorageAdmissionStatus {
    param(
        [string]$AbsoluteTargetDir,
        [int64]$MinimumFreeBytes = $script:ManagedCargoMinimumFreeSpaceBytes
    )

    $driveInfo = Get-TargetDriveInfo -AbsoluteTargetDir $AbsoluteTargetDir
    $decision = Get-PrebuildStorageAdmissionDecision `
        -FreeBytes $driveInfo.FreeBytes `
        -MinimumFreeBytes $MinimumFreeBytes
    return [pscustomobject]@{
        DriveRoot        = $driveInfo.DriveRoot
        FreeBytes        = $decision.FreeBytes
        MinimumFreeBytes = $decision.MinimumFreeBytes
        IsAdmitted       = $decision.IsAdmitted
    }
}
