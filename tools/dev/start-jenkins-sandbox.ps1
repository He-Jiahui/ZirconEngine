[CmdletBinding()]
param(
    [string]$Candidate,
    [string]$Regression,
    [string]$ReleaseFile,
    [string]$UpgradeReleaseFile,
    [string]$CodexExecutable,
    [switch]$ProbeOnly,
    [switch]$NoTray
)

$ErrorActionPreference = 'Stop'
$taskRepository = [IO.Path]::GetFullPath((Split-Path -Parent (Split-Path -Parent $PSScriptRoot)))
$taskConfig = Join-Path $taskRepository '.codex\config.toml'
$taskPython = Join-Path $taskRepository '.jenkins\runtime\python\python.exe'
$taskHolder = Join-Path $taskRepository '.jenkins\state\migration-baseline\jenkins_desktop_holder.py'

if ($UpgradeReleaseFile) {
    if ($ProbeOnly -or $Candidate -or $Regression -or $ReleaseFile) {
        throw '-UpgradeReleaseFile is an explicit mode; do not combine it with baseline or probe arguments.'
    }
    $taskHolder = Join-Path $taskRepository '.jenkins\state\migration-baseline\jenkins_upgrade_desktop_holder.py'
}

if (-not $ProbeOnly -and ([bool]$Candidate -ne [bool]$Regression)) {
    throw 'Supply both -Candidate and -Regression together.'
}

# A normal repeated start observes the selected healthy owner. It does not create
# another holder or run offline node migration against an online fixed agent.
if (-not $ProbeOnly -and -not $Candidate -and -not $Regression -and
    -not $ReleaseFile -and -not $UpgradeReleaseFile) {
    $taskOperationFile = Join-Path $taskRepository '.jenkins\state\deployment-operation.json'
    if (Test-Path -LiteralPath $taskOperationFile -PathType Leaf) {
        $taskExistingOperation = Get-Content -Raw -LiteralPath $taskOperationFile | ConvertFrom-Json
        if ($taskExistingOperation.component -eq 'lifecycle-host' -and
            $taskExistingOperation.controlPlaneOnly -eq $false -and
            $taskExistingOperation.state -eq 'running') {
            $taskObserver = Join-Path $taskRepository 'tools\jenkins-management\reuse_runtime.py'
            $taskExistingRaw = @(& $taskPython -B $taskObserver --repository-root $taskRepository)
            if ($LASTEXITCODE -ne 0) {
                throw ('Existing Jenkins ownership or health could not be verified: ' + ($taskExistingRaw -join [Environment]::NewLine))
            }
            $taskExisting = ($taskExistingRaw -join [Environment]::NewLine) | ConvertFrom-Json
            if ($taskExisting.state -ne 'ready' -or !$taskExisting.reusedExistingRuntime) {
                throw 'Existing Jenkins runtime observation is incomplete'
            }
            if (-not $NoTray -and -not $taskExisting.trayReady) {
                $taskTray = $taskExisting.trayLaunch
                $taskTrayArguments = @('-B', $taskTray.launcher, '--release', $taskTray.release,
                    '--release-sha256', $taskTray.releaseSha256, '--run')
                $taskTrayQuoted = $taskTrayArguments | ForEach-Object {
                    $taskValue = [string]$_
                    if ($taskValue -notmatch '[\s"]' -and $taskValue.Length) { $taskValue }
                    else {
                        $taskValue = [regex]::Replace($taskValue, '(\\*)"', '$1$1\"')
                        $taskValue = [regex]::Replace($taskValue, '(\\+)$', '$1$1')
                        '"' + $taskValue + '"'
                    }
                }
                $taskTrayProcess = Start-Process -FilePath $taskTray.pythonw -ArgumentList ($taskTrayQuoted -join ' ') -WorkingDirectory $taskRepository -WindowStyle Hidden -PassThru
                $taskExisting | Add-Member -NotePropertyName trayLaunchRequestedPid -NotePropertyValue $taskTrayProcess.Id
            }
            $taskExisting | ConvertTo-Json -Depth 8
            return
        }
    }
}

foreach ($taskRequired in @($taskConfig, $taskPython, $taskHolder)) {
    if (-not (Test-Path -LiteralPath $taskRequired -PathType Leaf)) { throw "Required file is missing: $taskRequired" }
}
if (-not $CodexExecutable) {
    $taskInstalledRoot = Join-Path ([Environment]::GetFolderPath('LocalApplicationData')) 'OpenAI\Codex\bin'
    $taskInstalled = @(Get-ChildItem -LiteralPath $taskInstalledRoot -Directory |
        ForEach-Object { Get-Item -LiteralPath (Join-Path $_.FullName 'codex.exe') -ErrorAction SilentlyContinue } |
        Sort-Object -Property @{ Expression = 'LastWriteTimeUtc'; Descending = $true }, @{ Expression = 'FullName'; Descending = $false })
    if (-not $taskInstalled.Count) { throw 'No installed official Codex executable was found. Supply -CodexExecutable.' }
    $CodexExecutable = $taskInstalled[0].FullName
}
$taskCodex = (Resolve-Path -LiteralPath $CodexExecutable).ProviderPath
if ((Get-Item -LiteralPath $taskCodex).PSIsContainer -or [IO.Path]::GetFileName($taskCodex) -ne 'codex.exe') {
    throw '-CodexExecutable must select the installed official codex.exe.'
}
$taskDigest = (Get-FileHash -LiteralPath $taskConfig -Algorithm SHA256).Hash.ToLowerInvariant()
$taskArguments = @('sandbox', '-P', 'zircon-strict', '--include-managed-config', '-C', $taskRepository,
    '--', $taskPython, '-B', $taskHolder, '--config-sha256', $taskDigest)
if ($ProbeOnly) { $taskArguments += '--probe-only' }
elseif ($UpgradeReleaseFile) {
    $taskUpgradeRelease = (Resolve-Path -LiteralPath $UpgradeReleaseFile).ProviderPath
    if (-not (Test-Path -LiteralPath $taskUpgradeRelease -PathType Leaf)) { throw 'The reviewed upgrade release must be a file.' }
    $taskUpgradeDigest = (Get-FileHash -LiteralPath $taskUpgradeRelease -Algorithm SHA256).Hash.ToLowerInvariant()
    $taskArguments += @('--release-file', $taskUpgradeRelease, '--release-sha256', $taskUpgradeDigest)
}
elseif ($Candidate) {
    $taskCandidate = (Resolve-Path -LiteralPath $Candidate).ProviderPath
    $taskRegression = (Resolve-Path -LiteralPath $Regression).ProviderPath
    if (-not (Test-Path -LiteralPath $taskCandidate -PathType Container) -or
        -not (Test-Path -LiteralPath $taskRegression -PathType Leaf)) { throw 'Candidate or regression has the wrong file type.' }
    $taskArguments += @('--candidate', $taskCandidate, '--regression', $taskRegression)
}
else {
    if (-not $ReleaseFile) { $ReleaseFile = Join-Path $taskRepository '.jenkins\state\deployment\usable-candidate.json' }
    $taskRelease = (Resolve-Path -LiteralPath $ReleaseFile).ProviderPath
    $taskReleaseDigest = (Get-FileHash -LiteralPath $taskRelease -Algorithm SHA256).Hash.ToLowerInvariant()
    $taskArguments += @('--release-file', $taskRelease, '--release-sha256', $taskReleaseDigest)
}
if ($NoTray) { $taskArguments += '--no-tray' }

# ProcessStartInfo invokes the official host bootstrap from this user's desktop.
# The commanded Python process receives the named profile; no alternate launcher,
# token, service, scheduled task, or sandbox fallback is used.
$taskTemporary = Join-Path $taskRepository '.jenkins\tmp'
$taskLogRoot = Join-Path $taskRepository '.jenkins\logs'
[IO.Directory]::CreateDirectory($taskTemporary) | Out-Null
[IO.Directory]::CreateDirectory($taskLogRoot) | Out-Null
$taskStart = [Diagnostics.ProcessStartInfo]::new()
$taskStart.FileName = $taskCodex
$taskStart.WorkingDirectory = $taskRepository
$taskStart.UseShellExecute = $false
$taskStart.CreateNoWindow = $false
$taskStart.RedirectStandardOutput = $true
$taskStart.RedirectStandardError = $true
# Preserve the caller's Codex home. Never relocate it into writable runtime state.
$taskOriginalCodexHome = [Environment]::GetEnvironmentVariable('CODEX_HOME', 'Process')
if ($taskOriginalCodexHome) { $taskStart.EnvironmentVariables['CODEX_HOME'] = $taskOriginalCodexHome }
foreach ($taskName in @('TEMP', 'TMP', 'TMPDIR')) { $taskStart.EnvironmentVariables[$taskName] = $taskTemporary }
$taskStart.EnvironmentVariables['PYTHONDONTWRITEBYTECODE'] = '1'

# ArgumentList is unavailable in Windows PowerShell 5.1. This is the Windows
# CreateProcess quoting rule, without shell evaluation or command substitution.
function ConvertTo-TaskWindowsArgument([string]$Value) {
    if ($Value -notmatch '[\s"]' -and $Value.Length) { return $Value }
    $taskQuoted = [regex]::Replace($Value, '(\\*)"', '$1$1\"')
    $taskQuoted = [regex]::Replace($taskQuoted, '(\\+)$', '$1$1')
    return '"' + $taskQuoted + '"'
}
$taskStart.Arguments = ($taskArguments | ForEach-Object { ConvertTo-TaskWindowsArgument $_ }) -join ' '
$taskStamp = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfffffffZ')
$taskOutPath = Join-Path $taskLogRoot "desktop-sandbox-$taskStamp.stdout.log"
$taskErrPath = Join-Path $taskLogRoot "desktop-sandbox-$taskStamp.stderr.log"
$taskOut = [IO.File]::Open($taskOutPath, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::Read)
$taskErr = [IO.File]::Open($taskErrPath, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::Read)
$taskProcess = [Diagnostics.Process]::new()
$taskProcessStarted = $false
$taskProcess.StartInfo = $taskStart
try {
    if (-not $taskProcess.Start()) { throw 'The official Codex sandbox host did not start.' }
    $taskProcessStarted = $true
    $taskCopyOut = $taskProcess.StandardOutput.BaseStream.CopyToAsync($taskOut)
    $taskCopyErr = $taskProcess.StandardError.BaseStream.CopyToAsync($taskErr)
    Write-Host "Foreground Jenkins sandbox holder. Keep this desktop console open. Logs: $taskLogRoot"
    Write-Host 'Ctrl+C requests a stop of its exact owned generation. A refused drain keeps the holder alive.'
    $taskProcess.WaitForExit()
    $taskCopyOut.GetAwaiter().GetResult()
    $taskCopyErr.GetAwaiter().GetResult()
    $taskExitCode = $taskProcess.ExitCode
} finally {
    # Disposal is never used to terminate a foreign or still-running service.
    # Keep pipe ownership while the holder finishes an interrupted drain.
    while ($taskProcessStarted -and -not $taskProcess.HasExited) {
        try { $taskProcess.WaitForExit(1000) | Out-Null } catch { }
    }
    if ($taskProcessStarted) {
        $taskCopyOut.GetAwaiter().GetResult()
        $taskCopyErr.GetAwaiter().GetResult()
    }
    $taskOut.Dispose()
    $taskErr.Dispose()
    $taskProcess.Dispose()
}
if ($taskExitCode -ne 0) {
    Write-Error "Sandbox holder failed with exit code $taskExitCode. Review $taskErrPath and $taskOutPath."
}
exit $taskExitCode
