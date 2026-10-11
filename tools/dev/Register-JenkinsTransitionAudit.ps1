[CmdletBinding()]
param(
    [string]$Repository = 'E:\Git\ZirconEngine',
    [Parameter(Mandatory)][string]$Python,
    [ValidatePattern('^([01][0-9]|2[0-3]):[0-5][0-9]$')][string]$DailyAt = '09:00',
    [switch]$RunNow
)
$ErrorActionPreference = 'Stop'
$taskName = 'ZirconEngine-Daily-Transition-Audit'
if ((Get-TimeZone).Id -ne 'China Standard Time') {
    throw 'Daily schedule requires the verified China Standard Time host timezone.'
}
$Python = (Get-Item -LiteralPath $Python).FullName
$wrapper = Join-Path $Repository 'tools\dev\Invoke-JenkinsTransitionAudit.ps1'
if (-not (Test-Path -LiteralPath $wrapper -PathType Leaf)) { throw 'Audit wrapper is missing.' }
& $Python -B (Join-Path $Repository 'tools\dev\jenkins_transition_monitor.py') --repo $Repository --preflight
if ($LASTEXITCODE -ne 0) { throw 'Audit permission/path preflight failed.' }
$existing = Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
if ($existing) { throw "Task $taskName already exists; inspect its ownership before changing it." }
$identity = [System.Security.Principal.WindowsIdentity]::GetCurrent().Name
$powershell = (Get-Command pwsh.exe -ErrorAction Stop).Source
$arguments = '-NoProfile -NonInteractive -WindowStyle Hidden -File "{0}" -Repository "{1}" -Python "{2}" -Deep -Notify' -f $wrapper, $Repository, $Python
$action = New-ScheduledTaskAction -Execute $powershell -Argument $arguments -WorkingDirectory $Repository
$trigger = New-ScheduledTaskTrigger -Daily -At ([datetime]::ParseExact($DailyAt, 'HH:mm', [Globalization.CultureInfo]::InvariantCulture))
$principal = New-ScheduledTaskPrincipal -UserId $identity -LogonType Interactive -RunLevel Limited
$settings = New-ScheduledTaskSettingsSet -StartWhenAvailable -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -MultipleInstances IgnoreNew -ExecutionTimeLimit (New-TimeSpan -Minutes 45)
$description = 'User-authorized temporary daily audit of Codex write directories, Jenkins native health and merge blockers. Metadata reports only; no Git writes, build, restart or deletion. Owner chat 01a12af6-7511-70d1-a452-61b80678eb56. Review retention and Jenkins handoff after 30 days.'
Register-ScheduledTask -TaskName $taskName -Action $action -Trigger $trigger -Principal $principal -Settings $settings -Description $description | Out-Null
if ($RunNow) { Start-ScheduledTask -TaskName $taskName }
$task = Get-ScheduledTask -TaskName $taskName
$info = Get-ScheduledTaskInfo -TaskName $taskName
[pscustomobject]@{
    taskName = $task.TaskName
    state = [string]$task.State
    user = $identity
    dailyAt = $DailyAt
    timezone = 'Asia/Shanghai'
    nextRunTime = $info.NextRunTime.ToString('o')
    execute = $task.Actions[0].Execute
    arguments = $task.Actions[0].Arguments
    logonType = [string]$task.Principal.LogonType
    runLevel = [string]$task.Principal.RunLevel
    reportRoot = 'E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor'
} | ConvertTo-Json -Compress
