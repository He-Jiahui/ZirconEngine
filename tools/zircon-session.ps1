[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [string]$Command = "status",
    [string]$RepoRoot,
    [ValidateRange(0, 65535)]
    [int]$Port = 6518,
    [switch]$Json,
    [switch]$Automatic,
    [Parameter(ValueFromPipeline = $true)]
    [AllowNull()]
    [AllowEmptyString()]
    [string]$PipelineInput,
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Arguments
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# The desktop command host terminates silent child processes before the
# historical 300-second coordinator deadline. Keep wrapper calls short enough
# to return durable request reconciliation instead of losing their request ID.
if ([string]::IsNullOrWhiteSpace($env:ZIRCON_COORDINATOR_COMMAND_TIMEOUT_SECONDS)) {
    $env:ZIRCON_COORDINATOR_COMMAND_TIMEOUT_SECONDS = '15'
}

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $RepoRoot = Split-Path -Parent $PSScriptRoot
}

$resolvedRepoRoot = (Resolve-Path -LiteralPath $RepoRoot).ProviderPath
$moduleRoot = Split-Path -Parent $PSScriptRoot
$python = (Get-Command python -ErrorAction Stop).Source

function Invoke-CoordinatorModule {
    param(
        [string[]]$ModuleArguments,
        [AllowNull()]
        [string]$StandardInput
    )

    Push-Location -LiteralPath $moduleRoot
    try {
        if ($null -eq $StandardInput) {
            & $python @ModuleArguments
        }
        else {
            $previousOutputEncoding = $OutputEncoding
            $previousPythonIoEncoding = $env:PYTHONIOENCODING
            try {
                $OutputEncoding = [Text.UTF8Encoding]::new($false)
                $env:PYTHONIOENCODING = 'utf-8'
                $StandardInput | & $python @ModuleArguments
            }
            finally {
                $OutputEncoding = $previousOutputEncoding
                $env:PYTHONIOENCODING = $previousPythonIoEncoding
            }
        }
    }
    finally {
        Pop-Location
    }
}

function Get-BaseArguments {
    $base = @("-m", "tools.session_coordinator", "--repo-root", $resolvedRepoRoot, "--port", "$Port")
    if ($Json) {
        $base += "--json"
    }
    return $base
}

function ConvertTo-CoordinatorNativeArguments {
    param([string[]]$SourceArguments)

    $nativeArguments = [System.Collections.Generic.List[string]]::new()
    for ($index = 0; $index -lt $SourceArguments.Count; $index++) {
        $argument = [string]$SourceArguments[$index]
        $nativeArguments.Add($argument) | Out-Null
        if ($argument -ne '--compatibility-json') {
            continue
        }
        if ($index + 1 -ge $SourceArguments.Count -or
            [string]::IsNullOrWhiteSpace([string]$SourceArguments[$index + 1])) {
            throw '--compatibility-json requires a non-empty JSON object value.'
        }
        $index++
        $payload = [Text.Encoding]::UTF8.GetBytes([string]$SourceArguments[$index])
        $nativeArguments.Add('base64:' + [Convert]::ToBase64String($payload)) | Out-Null
    }
    return $nativeArguments.ToArray()
}

function Get-RepositoryKey {
    $identity = $resolvedRepoRoot.Replace('/', '\').TrimEnd('\').ToLowerInvariant()
    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        return -join ($hasher.ComputeHash([Text.Encoding]::UTF8.GetBytes($identity)) | ForEach-Object { $_.ToString('x2') })
    }
    finally {
        $hasher.Dispose()
    }
}

function Get-CoordinatorLogRoot {
    $repositoryKey = Get-RepositoryKey
    return Join-Path $env:LOCALAPPDATA "Zircon Session Coordinator\daemon-log\$repositoryKey"
}

function Test-CoordinatorHealthy {
    $runtimePath = Join-Path $resolvedRepoRoot '.codex\state\session-coordinator\runtime.json'
    if (Test-Path -LiteralPath $runtimePath -PathType Leaf) {
        try {
            $runtime = Get-Content -LiteralPath $runtimePath -Raw -Encoding UTF8 | ConvertFrom-Json -ErrorAction Stop
            $listenerHost = [string]$runtime.host
            $runtimePort = [int]$runtime.port
            if (-not [string]::IsNullOrWhiteSpace($listenerHost) -and $runtimePort -gt 0) {
                $health = Invoke-RestMethod -Uri "http://${listenerHost}:$runtimePort/health" -TimeoutSec 2
                if ($health.status -eq 'ok' -and $health.repo_root -eq $resolvedRepoRoot) {
                    return $true
                }
            }
        }
        catch {
            # A stale or half-written descriptor is not healthy. Fall through
            # to the fixed listener only when one exists.
        }
    }

    # During a controlled default-port restart, the descriptor is briefly
    # absent before the successor writes it. The fixed listener keeps callers
    # from mistaking that publication gap for permission to spawn another daemon.
    if ($Port -eq 0) {
        return $false
    }
    try {
        $health = Invoke-RestMethod -Uri "http://127.0.0.1:$Port/health" -TimeoutSec 2
        return $health.status -eq 'ok' -and $health.repo_root -eq $resolvedRepoRoot
    }
    catch {
        return $false
    }
}

function Test-CoordinatorLaunchPending {
    $latestPath = Join-Path (Get-CoordinatorLogRoot) 'latest.json'
    if (-not (Test-Path -LiteralPath $latestPath -PathType Leaf)) {
        return $false
    }
    try {
        $record = Get-Content -LiteralPath $latestPath -Raw -Encoding UTF8 | ConvertFrom-Json -ErrorAction Stop
        if ([string]$record.repositoryKey -ne (Get-RepositoryKey) -or [int]$record.pid -le 0) {
            return $false
        }
        $process = Get-CimInstance Win32_Process -Filter "ProcessId = $([int]$record.pid)" -ErrorAction Stop
        if ($null -eq $process) {
            return $false
        }
        $commandLine = [string]$process.CommandLine
        return $commandLine.IndexOf('tools.session_coordinator', [StringComparison]::OrdinalIgnoreCase) -ge 0 -and
            $commandLine.IndexOf($resolvedRepoRoot, [StringComparison]::OrdinalIgnoreCase) -ge 0 -and
            $commandLine.IndexOf('serve', [StringComparison]::OrdinalIgnoreCase) -ge 0
    }
    catch {
        return $false
    }
}

function Invoke-CoordinatorStartupGate {
    param([scriptblock]$Action)

    $mutex = [Threading.Mutex]::new(
        $false,
        "Local\ZirconSessionCoordinatorStart-$(Get-RepositoryKey)"
    )
    $acquired = $false
    try {
        try {
            $acquired = $mutex.WaitOne(0)
        }
        catch [Threading.AbandonedMutexException] {
            $acquired = $true
        }
        if (-not $acquired) {
            # Another session is already publishing this repository's daemon.
            # Return immediately; command replay and later probes use the
            # descriptor once that launch finishes.
            return 'starting'
        }
        & $Action
    }
    finally {
        if ($acquired) {
            try {
                $mutex.ReleaseMutex()
            }
            catch [ApplicationException] {
                # The process did not own the mutex; disposal remains safe.
            }
        }
        $mutex.Dispose()
    }
}

function Start-CoordinatorProcess {
    param([string[]]$ServeArguments)

    $repositoryKey = Get-RepositoryKey
    $logRoot = Get-CoordinatorLogRoot
    New-Item -ItemType Directory -Force -Path $logRoot | Out-Null
    $stamp = "{0}-{1}" -f (Get-Date -Format 'yyyyMMdd-HHmmss-fff'), ([guid]::NewGuid().ToString('N').Substring(0, 8))
    $stdoutLog = Join-Path $logRoot "daemon-$stamp.stdout.log"
    $stderrLog = Join-Path $logRoot "daemon-$stamp.stderr.log"
    $process = Start-Process -FilePath $python -ArgumentList $ServeArguments `
        -WorkingDirectory $moduleRoot -WindowStyle Hidden -PassThru `
        -RedirectStandardOutput $stdoutLog -RedirectStandardError $stderrLog

    $record = [ordered]@{
        version = 1
        repositoryKey = $repositoryKey
        pid = $process.Id
        startedAt = [DateTimeOffset]::Now.ToString('o')
        stdoutLog = $stdoutLog
        stderrLog = $stderrLog
    }
    $latestPath = Join-Path $logRoot 'latest.json'
    $temporaryPath = "$latestPath.$([guid]::NewGuid().ToString('N')).tmp"
    [IO.File]::WriteAllText(
        $temporaryPath,
        ($record | ConvertTo-Json -Depth 3),
        [Text.UTF8Encoding]::new($false)
    )
    Move-Item -LiteralPath $temporaryPath -Destination $latestPath -Force

    foreach ($pattern in @('daemon-*.stdout.log', 'daemon-*.stderr.log')) {
        Get-ChildItem -LiteralPath $logRoot -Filter $pattern -File |
            Sort-Object LastWriteTimeUtc -Descending |
            Select-Object -Skip 10 |
            Remove-Item -Force -ErrorAction SilentlyContinue
    }
    return $process
}

function Start-Coordinator {
    return Invoke-CoordinatorStartupGate {
        if (Test-CoordinatorHealthy) {
            return 'ready'
        }
        if (Test-CoordinatorLaunchPending) {
            return 'starting'
        }

        $serveArguments = @("-m", "tools.session_coordinator", "--repo-root", $resolvedRepoRoot, "--port", "$Port", "serve")
        if ($Automatic) {
            $serveArguments += "--automatic-start"
        }
        Start-CoordinatorProcess -ServeArguments $serveArguments | Out-Null
        return 'starting'
    }
}

if ($Command -eq "start") {
    $startupState = Start-Coordinator
    if ($startupState -eq 'ready') {
        if (-not $Json) {
            Write-Output 'Coordinator ready.'
        }
        Invoke-CoordinatorModule -ModuleArguments ((Get-BaseArguments) + @("status"))
        exit $LASTEXITCODE
    }
    $starting = [ordered]@{
        status = 'starting'
        repoRoot = $resolvedRepoRoot
        port = $Port
        message = 'Coordinator launch accepted; queued commands will reconnect automatically.'
    }
    if ($Json) {
        $starting | ConvertTo-Json -Compress
    }
    else {
        Write-Output $starting.message
    }
    exit 0
}

if ($null -ne $Arguments -and $Arguments -contains '--patch-stdin') {
    throw '--patch-stdin is not byte-exact through PowerShell; use --patch-file.'
}

if ($Command -notin @("status", "stop")) {
    $startupState = Start-Coordinator
    if (-not $Json) {
        if ($startupState -eq 'ready') {
            Write-Output 'Coordinator ready.'
        }
        elseif ($startupState -eq 'starting') {
            Write-Output 'Coordinator launch accepted; queued commands will reconnect automatically.'
        }
    }
}

$moduleArguments = (Get-BaseArguments) + @($Command)
if ($null -ne $Arguments) {
    $filteredArguments = @($Arguments | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    $moduleArguments += @(ConvertTo-CoordinatorNativeArguments -SourceArguments $filteredArguments)
}
$pipelineValues = @($input)
$standardInput = $null
if ($moduleArguments -contains '--source-manifest-stdin') {
    # Top-level scripts do not reliably bind ValueFromPipeline without a process block.
    # Preserve the parameter fallback for direct invocation while consuming pipeline input.
    $standardInput = if ($pipelineValues.Count -gt 0) {
        $pipelineValues -join [Environment]::NewLine
    }
    else {
        $PipelineInput
    }
}
Invoke-CoordinatorModule -ModuleArguments $moduleArguments -StandardInput $standardInput
exit $LASTEXITCODE
