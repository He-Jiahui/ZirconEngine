[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$fixture = Join-Path $env:TEMP ("zircon-launcher-log-test-{0}" -f [guid]::NewGuid().ToString('N'))
$fakeRepo = Join-Path $fixture 'repo'
$fakeBin = Join-Path $fixture 'bin'
$fakeLocal = Join-Path $fixture 'localappdata'
$oldPath = $env:PATH
$oldLocal = $env:LOCALAPPDATA
$oldTestRepo = $env:ZIRCON_LAUNCHER_TEST_REPO

try {
    New-Item -ItemType Directory -Force -Path $fakeRepo, $fakeBin, $fakeLocal | Out-Null
    $fakePython = Join-Path $fakeBin 'python.cmd'
    @'
@echo off
echo %* | findstr /C:"serve" >nul
if errorlevel 1 (
  if exist "%ZIRCON_LAUNCHER_TEST_REPO%\.codex\state\session-coordinator\runtime.json" exit /b 0
  exit /b 1
)
mkdir "%ZIRCON_LAUNCHER_TEST_REPO%\.codex\state\session-coordinator" 2>nul
echo serve>>"%ZIRCON_LAUNCHER_TEST_REPO%\serve-count.txt"
timeout /t 1 /nobreak >nul
echo {}>"%ZIRCON_LAUNCHER_TEST_REPO%\.codex\state\session-coordinator\runtime.json"
echo fixture-daemon-stdout
echo fixture-daemon-stderr 1>&2
exit /b 0
'@ | Set-Content -LiteralPath $fakePython -Encoding Ascii

    $env:PATH = "$fakeBin;$oldPath"
    $env:LOCALAPPDATA = $fakeLocal
    $env:ZIRCON_LAUNCHER_TEST_REPO = $fakeRepo
    $launcher = Join-Path $repoRoot 'tools\zircon-session.ps1'
    $startupTimer = [Diagnostics.Stopwatch]::StartNew()
    $process = Start-Process -FilePath 'powershell.exe' -ArgumentList @(
        '-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass',
        '-File', $launcher, 'start', '-RepoRoot', $fakeRepo
    ) -Wait -PassThru -WindowStyle Hidden
    $startupTimer.Stop()
    if ($process.ExitCode -ne 0) {
        throw "launcher fixture failed with exit code $($process.ExitCode)"
    }
    if ($startupTimer.Elapsed.TotalSeconds -ge 6) {
        throw "launcher start waited $([Math]::Round($startupTimer.Elapsed.TotalSeconds, 2)) seconds instead of returning while the daemon initializes"
    }

    $identity = $fakeRepo.Replace('/', '\').TrimEnd('\').ToLowerInvariant()
    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        $hash = -join ($hasher.ComputeHash([Text.Encoding]::UTF8.GetBytes($identity)) | ForEach-Object { $_.ToString('x2') })
    }
    finally {
        $hasher.Dispose()
    }
    $logRoot = Join-Path $fakeLocal "Zircon Session Coordinator\daemon-log\$hash"
    if (-not (Test-Path -LiteralPath $logRoot -PathType Container)) {
        throw "launcher did not create the repository-scoped daemon log directory: $logRoot"
    }
    $stdout = Get-ChildItem -LiteralPath $logRoot -Filter 'daemon-*.stdout.log' -File
    $stderr = Get-ChildItem -LiteralPath $logRoot -Filter 'daemon-*.stderr.log' -File
    if ($stdout.Count -ne 1 -or $stderr.Count -ne 1) {
        throw 'launcher did not create exactly one stdout/stderr log pair'
    }
    $logged = $false
    for ($attempt = 0; $attempt -lt 20; $attempt++) {
        if ((Get-Content -Raw -LiteralPath $stdout.FullName) -match 'fixture-daemon-stdout' -and
            (Get-Content -Raw -LiteralPath $stderr.FullName) -match 'fixture-daemon-stderr') {
            $logged = $true
            break
        }
        Start-Sleep -Milliseconds 100
    }
    if (-not $logged) {
        throw 'daemon output was not preserved after asynchronous startup'
    }
    if ((Get-Content -Raw -LiteralPath $stdout.FullName) -notmatch 'fixture-daemon-stdout') {
        throw 'daemon stdout was not preserved'
    }
    if ((Get-Content -Raw -LiteralPath $stderr.FullName) -notmatch 'fixture-daemon-stderr') {
        throw 'daemon stderr was not preserved'
    }
    $latest = Get-Content -Raw -LiteralPath (Join-Path $logRoot 'latest.json') | ConvertFrom-Json
    if ($latest.repositoryKey -ne $hash -or $latest.pid -le 0) {
        throw 'latest daemon launch metadata is incomplete'
    }
    if ($latest.stdoutLog -ne $stdout.FullName -or $latest.stderrLog -ne $stderr.FullName) {
        throw 'latest daemon launch metadata does not reference the captured log pair'
    }

    $mutexName = "Local\ZirconSessionCoordinatorStart-$hash"
    $mutexReady = Join-Path $fixture 'startup-gate-ready'
    $mutexScript = @"
`$mutex = [Threading.Mutex]::new(`$false, '$mutexName')
try {
    [void]`$mutex.WaitOne()
    New-Item -ItemType File -Path '$mutexReady' -Force | Out-Null
    Start-Sleep -Seconds 3
}
finally {
    `$mutex.ReleaseMutex()
    `$mutex.Dispose()
}
"@
    $mutexEncoded = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($mutexScript))
    $mutexProcess = Start-Process -FilePath 'powershell.exe' -ArgumentList @(
        '-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-EncodedCommand', $mutexEncoded
    ) -PassThru -WindowStyle Hidden
    try {
        for ($attempt = 0; $attempt -lt 30 -and -not (Test-Path -LiteralPath $mutexReady); $attempt++) {
            Start-Sleep -Milliseconds 100
        }
        if (-not (Test-Path -LiteralPath $mutexReady)) {
            throw 'startup-gate fixture did not acquire the mutex'
        }
        $busyTimer = [Diagnostics.Stopwatch]::StartNew()
        $busyStart = Start-Process -FilePath 'powershell.exe' -ArgumentList @(
            '-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass',
            '-File', $launcher, 'start', '-RepoRoot', $fakeRepo
        ) -Wait -PassThru -WindowStyle Hidden
        $busyTimer.Stop()
        if ($busyStart.ExitCode -ne 0) {
            throw "launcher did not accept a busy startup gate: $($busyStart.ExitCode)"
        }
        if ($busyTimer.Elapsed.TotalSeconds -ge 3) {
            throw "launcher waited $([Math]::Round($busyTimer.Elapsed.TotalSeconds, 2)) seconds for a busy startup gate"
        }
    }
    finally {
        if (-not $mutexProcess.HasExited) {
            $mutexProcess.WaitForExit()
        }
    }

    for ($index = 0; $index -lt 12; $index++) {
        $oldStamp = "20000101-0000{0}-000-old{1}" -f ($index % 10), $index
        Set-Content -LiteralPath (Join-Path $logRoot "daemon-$oldStamp.stdout.log") -Value 'old stdout'
        Set-Content -LiteralPath (Join-Path $logRoot "daemon-$oldStamp.stderr.log") -Value 'old stderr'
    }
    Remove-Item -LiteralPath (Join-Path $fakeRepo '.codex\state\session-coordinator\runtime.json') -Force
    $second = Start-Process -FilePath 'powershell.exe' -ArgumentList @(
        '-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass',
        '-File', $launcher, 'start', '-RepoRoot', $fakeRepo
    ) -Wait -PassThru -WindowStyle Hidden
    if ($second.ExitCode -ne 0) {
        throw "second launcher fixture failed with exit code $($second.ExitCode)"
    }
    if ((Get-ChildItem -LiteralPath $logRoot -Filter 'daemon-*.stdout.log' -File).Count -ne 10 -or
        (Get-ChildItem -LiteralPath $logRoot -Filter 'daemon-*.stderr.log' -File).Count -ne 10) {
        throw 'daemon launch log retention did not keep exactly ten generations per stream'
    }

    $serveCount = Join-Path $fakeRepo 'serve-count.txt'
    Remove-Item -LiteralPath $serveCount -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath (Join-Path $fakeRepo '.codex\state\session-coordinator\runtime.json') -Force
    $parallel = @(
        (Start-Process -FilePath 'powershell.exe' -ArgumentList @(
            '-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass',
            '-File', $launcher, 'start', '-RepoRoot', $fakeRepo
        ) -PassThru -WindowStyle Hidden)
        (Start-Process -FilePath 'powershell.exe' -ArgumentList @(
            '-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass',
            '-File', $launcher, 'start', '-RepoRoot', $fakeRepo
        ) -PassThru -WindowStyle Hidden)
    )
    $parallel | ForEach-Object { $_.WaitForExit() }
    if ($parallel | Where-Object { $_.ExitCode -ne 0 }) {
        throw 'concurrent launcher fixture did not return successfully'
    }
    for ($attempt = 0; $attempt -lt 30 -and -not (Test-Path -LiteralPath $serveCount); $attempt++) {
        Start-Sleep -Milliseconds 100
    }
    if (-not (Test-Path -LiteralPath $serveCount)) {
        throw 'concurrent launcher starts did not launch a daemon process'
    }
    if (@(Get-Content -LiteralPath $serveCount).Count -ne 1) {
        throw 'concurrent launcher starts spawned more than one daemon process'
    }
    Write-Host 'zircon session launcher logging test passed'
}
finally {
    $env:PATH = $oldPath
    $env:LOCALAPPDATA = $oldLocal
    $env:ZIRCON_LAUNCHER_TEST_REPO = $oldTestRepo
    if (Test-Path -LiteralPath $fixture) {
        for ($attempt = 0; $attempt -lt 30; $attempt++) {
            try {
                Remove-Item -LiteralPath $fixture -Recurse -Force -ErrorAction Stop
                break
            }
            catch {
                if ($attempt -eq 29) {
                    throw
                }
                Start-Sleep -Milliseconds 100
            }
        }
    }
}
