[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$RecoveryReleaseFile,
    [string]$CodexExecutable,
    [switch]$CheckOnly
)

$ErrorActionPreference = 'Stop'
$taskRepository = [IO.Path]::GetFullPath((Split-Path -Parent (Split-Path -Parent $PSScriptRoot)))
$taskConfig = Join-Path $taskRepository '.codex\config.toml'
$taskPython = Join-Path $taskRepository '.jenkins\runtime\python\python.exe'
$taskEntry = Join-Path $taskRepository 'tools\dev\jenkins_recovery_keeper.py'
$taskRelease = (Resolve-Path -LiteralPath $RecoveryReleaseFile).ProviderPath
foreach ($taskRequired in @($taskConfig, $taskPython, $taskEntry, $taskRelease)) {
    if (-not (Test-Path -LiteralPath $taskRequired -PathType Leaf)) { throw "Required file is missing: $taskRequired" }
}
$taskReleaseDocument = Get-Content -LiteralPath $taskRelease -Raw | ConvertFrom-Json
$taskReviewedHost = $taskReleaseDocument.sandboxHost
if (-not $taskReviewedHost.path -or -not $taskReviewedHost.sha256 -or $taskReviewedHost.profile -ne 'zircon-strict' -or
    $taskReviewedHost.includeManagedConfig -ne $true) { throw 'The release must pin the official strict sandbox host.' }
if (-not $CodexExecutable) { $CodexExecutable = $taskReviewedHost.path }
$taskCodex = (Resolve-Path -LiteralPath $CodexExecutable).ProviderPath
$taskInstalledRoot = [IO.Path]::GetFullPath((Join-Path ([Environment]::GetFolderPath('LocalApplicationData')) 'OpenAI\Codex\bin'))
if ([IO.Path]::GetFileName($taskCodex) -ne 'codex.exe' -or
    -not $taskCodex.StartsWith($taskInstalledRoot + '\', [StringComparison]::OrdinalIgnoreCase) -or
    $taskCodex -ne $taskReviewedHost.path -or
    (Get-FileHash -LiteralPath $taskCodex -Algorithm SHA256).Hash.ToLowerInvariant() -ne $taskReviewedHost.sha256) {
    throw 'The installed official host differs from the reviewed release.'
}
$taskConfigSha = (Get-FileHash -LiteralPath $taskConfig -Algorithm SHA256).Hash.ToLowerInvariant()
$taskReleaseSha = (Get-FileHash -LiteralPath $taskRelease -Algorithm SHA256).Hash.ToLowerInvariant()
$taskArguments = @('sandbox', '-P', 'zircon-strict', '--include-managed-config', '-C', $taskRepository,
    '--', $taskPython, '-B', $taskEntry, '--config-sha256', $taskConfigSha,
    '--release-file', $taskRelease, '--release-sha256', $taskReleaseSha)
if ($CheckOnly) { $taskArguments += '--check-only' }

function ConvertTo-TaskWindowsArgument([string]$Value) {
    if ($Value -notmatch '[\s"]' -and $Value.Length) { return $Value }
    $taskQuoted = [regex]::Replace($Value, '(\\*)"', '$1$1\"')
    $taskQuoted = [regex]::Replace($taskQuoted, '(\\+)$', '$1$1')
    return '"' + $taskQuoted + '"'
}

function Read-TaskDesktopRecord([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) { return $null }
    if ((Get-Item -LiteralPath $Path).Length -gt 1048576) { throw 'Desktop readiness record exceeds its bound.' }
    return Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json
}

function Test-TaskReleaseReference($Reference) {
    return $Reference -and $Reference.path -ceq $taskRelease -and $Reference.sha256 -ceq $taskReleaseSha
}

function Get-TaskCommittedDesktopRuntime {
    $taskDescriptor = Read-TaskDesktopRecord (Join-Path $taskRepository '.jenkins\state\formal-lifecycle\recovery-keeper.json')
    $taskCommand = Read-TaskDesktopRecord (Join-Path $taskRepository '.jenkins\state\recovery-lifecycle\command.json')
    if (-not $taskDescriptor -or -not $taskCommand -or
        $taskDescriptor.kind -cne 'formal-lost-owner-recovery-keeper-descriptor' -or $taskDescriptor.admission -cne 'idle' -or
        $taskDescriptor.runtime.state -cne 'running' -or $taskCommand.kind -cne 'formal-lost-owner-recovery-command' -or
        $taskCommand.status -cne 'committed' -or -not (Test-TaskReleaseReference $taskDescriptor.release) -or
        -not (Test-TaskReleaseReference $taskCommand.release)) { return $null }
    $taskSelected = Read-TaskDesktopRecord (Join-Path $taskRepository '.jenkins\state\deployment\driver.json')
    $taskOperation = Read-TaskDesktopRecord (Join-Path $taskRepository '.jenkins\state\deployment-operation.json')
    $taskHost = Read-TaskDesktopRecord (Join-Path $taskRepository '.jenkins\state\deployment\host.json')
    if (-not $taskSelected -or -not $taskOperation -or -not $taskHost -or
        $taskOperation.state -cne 'running' -or $taskHost.status -cne 'running' -or
        -not (Test-TaskReleaseReference $taskSelected.recoveryRelease)) { return $null }
    $taskRuntimeId = $taskSelected.runtimeOperationId
    if (-not $taskRuntimeId -or $taskOperation.operationId -cne $taskRuntimeId -or $taskOperation.generation -cne $taskRuntimeId -or
        $taskHost.operationId -cne $taskRuntimeId -or $taskHost.generation -cne $taskRuntimeId -or
        $taskDescriptor.runtime.operation.operationId -cne $taskRuntimeId -or
        $taskDescriptor.runtime.operation.generation -cne $taskRuntimeId -or
        $taskDescriptor.runtime.host.operationId -cne $taskRuntimeId -or $taskDescriptor.runtime.host.generation -cne $taskRuntimeId -or
        $taskDescriptor.runtime.selected.runtimeOperationId -cne $taskRuntimeId) { return $null }
    foreach ($taskField in @('driverDigest', 'root', 'launcher', 'manifest', 'steps')) {
        $taskExpected = $taskReleaseDocument.targetDriver.$taskField
        if (-not $taskExpected -or $taskSelected.$taskField -cne $taskExpected -or
            $taskDescriptor.runtime.selected.$taskField -cne $taskExpected) { return $null }
    }
    foreach ($taskField in @('pid', 'creationTime', 'executable', 'commandDigest')) {
        if (-not $taskDescriptor.keeper.$taskField -or $taskCommand.keeper.$taskField -cne $taskDescriptor.keeper.$taskField) { return $null }
    }
    $taskResultRef = $taskCommand.result
    $taskHistory = Join-Path $taskRepository '.jenkins\state\recovery-lifecycle\history'
    if (-not $taskResultRef.path -or $taskResultRef.sha256 -cnotmatch '^[0-9a-f]{64}$' -or
        [IO.Path]::GetFullPath($taskResultRef.path) -cne $taskResultRef.path -or
        [IO.Path]::GetDirectoryName($taskResultRef.path) -cne $taskHistory -or
        (Get-FileHash -LiteralPath $taskResultRef.path -Algorithm SHA256).Hash.ToLowerInvariant() -cne $taskResultRef.sha256) { return $null }
    $taskResult = Read-TaskDesktopRecord $taskResultRef.path
    if (-not $taskResult -or $taskResult.kind -cne 'recovery-keeper-command-result' -or
        $taskResult.status -cne 'running-acceptance-pending' -or $taskResult.operationId -cne $taskRuntimeId -or
        -not (Test-TaskReleaseReference $taskResult.release) -or
        $taskResult.runtimeReceipt.path -cne $taskDescriptor.runtimeReceipt.path -or
        $taskResult.runtimeReceipt.sha256 -cne $taskDescriptor.runtimeReceipt.sha256 -or
        $taskResult.request.path -cne $taskCommand.request.path -or $taskResult.request.sha256 -cne $taskCommand.request.sha256) { return $null }
    return $taskRuntimeId
}

function Assert-TaskDesktopPin([string]$Path, $Pins) {
    $taskPin = @($Pins.PSObject.Properties | Where-Object { $_.Name -ceq $Path })
    if ($taskPin.Count -ne 1 -or $taskPin[0].Value -cnotmatch '^[0-9a-f]{64}$' -or
        -not (Test-Path -LiteralPath $Path -PathType Leaf) -or
        (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant() -cne $taskPin[0].Value) {
        throw 'The release-pinned desktop launcher or interpreter changed.'
    }
}

function Test-TaskDesktopUi([string]$RuntimeId, [string]$Pythonw, [string]$Arguments) {
    $taskUi = Read-TaskDesktopRecord (Join-Path $taskRepository '.jenkins\state\tray\ui.json')
    if (-not $taskUi -or $taskUi.schemaVersion -ne 1 -or $taskUi.iconAdded -ne $true -or
        $taskUi.hwnd -le 0 -or $taskUi.uiPid -le 0 -or -not $taskUi.uiBirth -or
        $taskUi.uiExecutable -cne $Pythonw -or $taskUi.operation.operationId -cne $RuntimeId -or
        $taskUi.operation.generation -cne $RuntimeId) { return $false }
    $taskUiProcess = $null
    try {
        $taskUiProcess = [Diagnostics.Process]::GetProcessById([int]$taskUi.uiPid)
        if ($taskUiProcess.HasExited -or $taskUiProcess.MainModule.FileName -cne $Pythonw -or
            [string]$taskUiProcess.StartTime.ToUniversalTime().ToFileTimeUtc() -cne [string]$taskUi.uiBirth -or
            $taskUiProcess.SessionId -ne [Diagnostics.Process]::GetCurrentProcess().SessionId) { return $false }
        $taskNativeUi = Get-CimInstance -ClassName Win32_Process -Filter ('ProcessId=' + [int]$taskUi.uiPid) -OperationTimeoutSec 5
        $taskNativeOwner = Invoke-CimMethod -InputObject $taskNativeUi -MethodName GetOwnerSid -OperationTimeoutSec 5
        if ($taskNativeOwner.ReturnValue -ne 0 -or
            $taskNativeOwner.Sid -cne [Security.Principal.WindowsIdentity]::GetCurrent().User.Value) { return $false }
        $taskExpected = (ConvertTo-TaskWindowsArgument $Pythonw) + ' ' + $Arguments
        $taskQuotedExpected = '"' + $Pythonw + '" ' + $Arguments
        return $taskNativeUi.CommandLine -ceq $taskExpected -or $taskNativeUi.CommandLine -ceq $taskQuotedExpected
    } catch { return $false } finally { if ($taskUiProcess) { $taskUiProcess.Dispose() } }
}

function Invoke-TaskDesktopCheck([string]$Launcher) {
    $taskProbeStart = [Diagnostics.ProcessStartInfo]::new()
    $taskProbeStart.FileName = $taskPython
    $taskProbeStart.WorkingDirectory = $taskRepository
    $taskProbeStart.UseShellExecute = $false
    $taskProbeStart.CreateNoWindow = $true
    $taskProbeStart.RedirectStandardOutput = $true
    $taskProbeStart.RedirectStandardError = $true
    $taskProbeStart.Arguments = (@('-B', $Launcher, '--release', $taskRelease, '--release-sha256', $taskReleaseSha, '--check-only') |
        ForEach-Object { ConvertTo-TaskWindowsArgument $_ }) -join ' '
    $taskProbeStart.EnvironmentVariables['PYTHONDONTWRITEBYTECODE'] = '1'
    foreach ($taskName in @('TEMP', 'TMP', 'TMPDIR')) { $taskProbeStart.EnvironmentVariables[$taskName] = $taskTemporary }
    $taskProbe = [Diagnostics.Process]::new()
    $taskProbe.StartInfo = $taskProbeStart
    $taskProbeOut = $null; $taskProbeErr = $null
    try {
        if (-not $taskProbe.Start()) { throw 'Desktop launcher check did not start.' }
        $taskProbeOut = $taskProbe.StandardOutput.ReadToEndAsync()
        $taskProbeErr = $taskProbe.StandardError.ReadToEndAsync()
        if (-not $taskProbe.WaitForExit(60000)) {
            # This is only the read-only pin checker, never the strict host.
            $taskProbe.Kill()
            $taskProbe.WaitForExit(5000) | Out-Null
            throw 'Desktop launcher check exceeded its 60-second budget.'
        }
        if ($taskProbe.ExitCode -ne 0) { throw 'Desktop launcher check rejected the current release or runtime.' }
    } finally {
        foreach ($taskStream in @(@{task=$taskProbeOut; suffix='stdout'}, @{task=$taskProbeErr; suffix='stderr'})) {
            if ($taskStream.task -and $taskStream.task.IsCompleted) {
                try {
                    $taskText = $taskStream.task.GetAwaiter().GetResult()
                    $taskText = $taskText.Substring(0, [Math]::Min(8192, $taskText.Length))
                    [IO.File]::WriteAllText((Join-Path $taskLogs "desktop-check-$taskStamp.$($taskStream.suffix).log"), $taskText)
                } catch { Write-Warning 'Desktop checker output could not be recorded.' }
            }
        }
        $taskProbe.Dispose()
    }
}

function Start-TaskUserDesktopTray {
    $taskSealedDesktop = $taskReleaseDocument.kind -ceq 'formal-owned-recovery-upgrade-release' -or
        ($taskReleaseDocument.kind -ceq 'formal-lost-owner-recovery-release' -and $taskReleaseDocument.lostRecoveryRuntime)
    if ($CheckOnly -or -not $taskSealedDesktop) { return }
    try {
        $taskDesktopWait = [Diagnostics.Stopwatch]::StartNew()
        $taskRuntimeId = $null
        while ($taskDesktopWait.Elapsed.TotalSeconds -lt 240 -and -not $taskProcess.HasExited) {
            try { $taskRuntimeId = Get-TaskCommittedDesktopRuntime } catch { $taskRuntimeId = $null }
            if ($taskRuntimeId) { break }
            Start-Sleep -Milliseconds 500
        }
        if (-not $taskRuntimeId -or $taskProcess.HasExited) { throw 'Desktop tray launch is pending: the exact committed running successor was not published within 240 seconds.' }
        $taskLauncher = Join-Path $taskRepository 'tools\dev\jenkins_desktop_tray.py'
        $taskPythonw = Join-Path $taskRepository '.jenkins\runtime\python\pythonw.exe'
        Assert-TaskDesktopPin $taskLauncher $taskReleaseDocument.operatorFiles
        Assert-TaskDesktopPin $taskPython $taskReleaseDocument.runtimeFiles
        Assert-TaskDesktopPin $taskPythonw $taskReleaseDocument.runtimeFiles
        Invoke-TaskDesktopCheck $taskLauncher
        if ($taskProcess.HasExited -or (Get-TaskCommittedDesktopRuntime) -cne $taskRuntimeId) { throw 'The committed runtime changed before desktop tray launch.' }
        $taskUiArguments = (@('-B', $taskLauncher, '--release', $taskRelease, '--release-sha256', $taskReleaseSha, '--run') |
            ForEach-Object { ConvertTo-TaskWindowsArgument $_ }) -join ' '
        if (Test-TaskDesktopUi $taskRuntimeId $taskPythonw $taskUiArguments) { return }
        # Inherit the caller's user/session/desktop. The pinned launcher proves
        # Explorer SID and WinSta0\Default before importing the sealed tray.
        $taskUiStart = [Diagnostics.ProcessStartInfo]::new()
        $taskUiStart.FileName = $taskPythonw
        $taskUiStart.WorkingDirectory = $taskRepository
        $taskUiStart.UseShellExecute = $false
        $taskUiStart.CreateNoWindow = $true
        $taskUiStart.Arguments = $taskUiArguments
        $taskUiStart.EnvironmentVariables['PYTHONDONTWRITEBYTECODE'] = '1'
        foreach ($taskName in @('TEMP', 'TMP', 'TMPDIR')) { $taskUiStart.EnvironmentVariables[$taskName] = $taskTemporary }
        $taskUiProcess = [Diagnostics.Process]::new()
        $taskUiProcess.StartInfo = $taskUiStart
        try {
            if (-not $taskUiProcess.Start()) { throw 'The pinned desktop tray did not start.' }
            $taskUiWait = [Diagnostics.Stopwatch]::StartNew()
            while ($taskUiWait.Elapsed.TotalSeconds -lt 15 -and -not $taskUiProcess.HasExited -and -not $taskProcess.HasExited) {
                if (Test-TaskDesktopUi $taskRuntimeId $taskPythonw $taskUiArguments) { return }
                Start-Sleep -Milliseconds 500
            }
            if (-not (Test-TaskDesktopUi $taskRuntimeId $taskPythonw $taskUiArguments)) { throw 'Desktop tray startup was not confirmed by matching live UI telemetry.' }
        } finally { $taskUiProcess.Dispose() }
    } catch {
        $taskUiMessage = 'Jenkins service host remains retained; desktop tray unavailable: ' + $_.Exception.Message
        $taskUiMessage = $taskUiMessage.Substring(0, [Math]::Min(4096, $taskUiMessage.Length))
        try { [IO.File]::WriteAllText((Join-Path $taskLogs "desktop-tray-$taskStamp.warning.log"), $taskUiMessage) } catch { }
        Write-Warning $taskUiMessage
    }
}

$taskTemporary = Join-Path $taskRepository '.jenkins\tmp'
$taskLogs = Join-Path $taskRepository '.jenkins\logs'
[IO.Directory]::CreateDirectory($taskTemporary) | Out-Null
[IO.Directory]::CreateDirectory($taskLogs) | Out-Null
$taskStart = [Diagnostics.ProcessStartInfo]::new()
$taskStart.FileName = $taskCodex
$taskStart.WorkingDirectory = $taskRepository
$taskStart.UseShellExecute = $false
$taskStart.CreateNoWindow = $true
$taskStart.RedirectStandardOutput = $true
$taskStart.RedirectStandardError = $true
$taskStart.Arguments = ($taskArguments | ForEach-Object { ConvertTo-TaskWindowsArgument $_ }) -join ' '
foreach ($taskName in @('TEMP', 'TMP', 'TMPDIR')) { $taskStart.EnvironmentVariables[$taskName] = $taskTemporary }
$taskStart.EnvironmentVariables['PYTHONDONTWRITEBYTECODE'] = '1'
$taskStamp = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfffffffZ')
$taskHostDirectory = Join-Path $taskRepository '.jenkins\state\recovery-lifecycle\sandbox-hosts'
[IO.Directory]::CreateDirectory($taskHostDirectory) | Out-Null
$taskHostRecord = Join-Path $taskHostDirectory "$taskStamp.json"
$taskStart.EnvironmentVariables['ZIRCON_JENKINS_RECOVERY_HOST_RECORD'] = $taskHostRecord
$taskOutPath = Join-Path $taskLogs "recovery-sandbox-$taskStamp.stdout.log"
$taskErrPath = Join-Path $taskLogs "recovery-sandbox-$taskStamp.stderr.log"
$taskOut = [IO.File]::Open($taskOutPath, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::Read)
$taskErr = [IO.File]::Open($taskErrPath, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::Read)
$taskProcess = [Diagnostics.Process]::new()
$taskProcess.StartInfo = $taskStart
$taskStarted = $false
try {
    if (-not $taskProcess.Start()) { throw 'The official strict host did not start.' }
    $taskStarted = $true
    $taskHostArgs = @($taskCodex) + $taskArguments
    $taskDigestAlgorithm = [Security.Cryptography.SHA256]::Create()
    try {
        $taskArgBytes = [Text.Encoding]::UTF8.GetBytes(($taskHostArgs -join [char]0))
        $taskCommandDigest = ([BitConverter]::ToString($taskDigestAlgorithm.ComputeHash($taskArgBytes))).Replace('-', '').ToLowerInvariant()
    } finally { $taskDigestAlgorithm.Dispose() }
    $taskHostEvidence = [ordered]@{
        schemaVersion = 1; kind = 'official-strict-recovery-sandbox-host'
        host = [ordered]@{pid=$taskProcess.Id; creationTime=[string]$taskProcess.StartTime.ToUniversalTime().ToFileTimeUtc();
            executable=$taskCodex; commandDigest=$taskCommandDigest}
        arguments=$taskHostArgs; release=@{path=$taskRelease; sha256=$taskReleaseSha}
        dispatcher=@{path=$PSCommandPath; sha256=(Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash.ToLowerInvariant()}
        profile='zircon-strict'; includeManagedConfig=$true; automaticRestart=$false; runtimeAcceptance=$false
    }
    $taskHostRaw = [Text.Encoding]::UTF8.GetBytes(($taskHostEvidence | ConvertTo-Json -Depth 8) + [Environment]::NewLine)
    $taskHostStream = [IO.File]::Open($taskHostRecord, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::Read)
    try { $taskHostStream.Write($taskHostRaw, 0, $taskHostRaw.Length); $taskHostStream.Flush($true) } finally { $taskHostStream.Dispose() }
    $taskCopyOut = $taskProcess.StandardOutput.BaseStream.CopyToAsync($taskOut)
    $taskCopyErr = $taskProcess.StandardError.BaseStream.CopyToAsync($taskErr)
    Start-TaskUserDesktopTray
    $taskProcess.WaitForExit()
    $taskCopyOut.GetAwaiter().GetResult()
    $taskCopyErr.GetAwaiter().GetResult()
    $taskExitCode = $taskProcess.ExitCode
} finally {
    while ($taskStarted -and -not $taskProcess.HasExited) {
        try { $taskProcess.WaitForExit(1000) | Out-Null } catch { }
    }
    if ($taskStarted) {
        $taskCopyOut.GetAwaiter().GetResult()
        $taskCopyErr.GetAwaiter().GetResult()
    }
    $taskOut.Dispose()
    $taskErr.Dispose()
    $taskProcess.Dispose()
}
if ($taskExitCode -ne 0) { Write-Error "Jenkins recovery host exited with $taskExitCode. Logs: $taskOutPath and $taskErrPath" }
exit $taskExitCode
