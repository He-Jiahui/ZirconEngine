[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [string]$Command = "status",
    [string]$RepoRoot,
    [switch]$Json,
    [switch]$Automatic,
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Arguments
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $RepoRoot = Split-Path -Parent $PSScriptRoot
}

$resolvedRepoRoot = (Resolve-Path -LiteralPath $RepoRoot).ProviderPath
$python = (Get-Command python -ErrorAction Stop).Source

function Get-BaseArguments {
    $base = @("-m", "tools.session_coordinator", "--repo-root", $resolvedRepoRoot)
    if ($Json) {
        $base += "--json"
    }
    return $base
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

function Test-CoordinatorHealthy {
    & $python @((Get-BaseArguments) + @("status")) *> $null
    if ($LASTEXITCODE -eq 0) {
        return $true
    }

    # During a controlled restart, the descriptor is briefly absent before the
    # successor writes it.  The loopback health endpoint keeps launcher callers
    # from mistaking that publication gap for permission to spawn another daemon.
    try {
        $health = Invoke-RestMethod -Uri 'http://127.0.0.1:6518/health' -TimeoutSec 2
        return $health.status -eq 'ok' -and $health.repo_root -eq $resolvedRepoRoot
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
            $acquired = $mutex.WaitOne([TimeSpan]::FromSeconds(35))
        }
        catch [Threading.AbandonedMutexException] {
            $acquired = $true
        }
        if (-not $acquired) {
            throw 'Timed out waiting for the repository coordinator startup gate.'
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
    $logRoot = Join-Path $env:LOCALAPPDATA "Zircon Session Coordinator\daemon-log\$repositoryKey"
    New-Item -ItemType Directory -Force -Path $logRoot | Out-Null
    $stamp = "{0}-{1}" -f (Get-Date -Format 'yyyyMMdd-HHmmss-fff'), ([guid]::NewGuid().ToString('N').Substring(0, 8))
    $stdoutLog = Join-Path $logRoot "daemon-$stamp.stdout.log"
    $stderrLog = Join-Path $logRoot "daemon-$stamp.stderr.log"
    $process = Start-Process -FilePath $python -ArgumentList $ServeArguments `
        -WorkingDirectory $resolvedRepoRoot -WindowStyle Hidden -PassThru `
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
    Invoke-CoordinatorStartupGate {
        if (Test-CoordinatorHealthy) {
            return
        }

        $serveArguments = @("-m", "tools.session_coordinator", "--repo-root", $resolvedRepoRoot, "serve")
        if ($Automatic) {
            $serveArguments += "--automatic-start"
        }
        Start-CoordinatorProcess -ServeArguments $serveArguments | Out-Null

        for ($attempt = 0; $attempt -lt 300; $attempt++) {
            Start-Sleep -Milliseconds 100
            if (Test-CoordinatorHealthy) {
                return
            }
        }
        throw "Zircon Session coordinator did not become healthy within 30 seconds."
    }
}

if ($Command -eq "start") {
    Start-Coordinator
    & $python @((Get-BaseArguments) + @("status"))
    exit $LASTEXITCODE
}

if ($Command -notin @("status", "stop")) {
    Start-Coordinator
}

& $python @((Get-BaseArguments) + @($Command) + $Arguments)
exit $LASTEXITCODE
