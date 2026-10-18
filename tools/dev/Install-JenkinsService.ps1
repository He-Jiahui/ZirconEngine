[CmdletBinding()]
param([switch]$UserMode)

$ErrorActionPreference = 'Stop'
$repository = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..'))
$python = Join-Path $repository '.jenkins\runtime\python\python.exe'
$entry = Join-Path $repository 'tools\dev\jenkins_service.py'
$specification = Join-Path $repository '.jenkins\deployment-spec.json'
$policy = Join-Path $repository '.jenkins\state\independent-service\policy.json'
$evidence = Join-Path $repository '.jenkins\state\independent-service'
$taskName = 'ZirconEngine-Jenkins-Service'

foreach ($path in @($repository, $python, $entry, $specification, $policy, $evidence)) {
    $cursor = $path
    while ($null -ne $cursor) {
        $candidate = Get-Item -LiteralPath $cursor
        if (($candidate.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "A service path contains a link: $($candidate.FullName)"
        }
        $parent = [System.IO.Directory]::GetParent($cursor)
        $cursor = $(if ($null -ne $parent) { $parent.FullName } else { $null })
    }
}
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$administrator = ([Security.Principal.WindowsPrincipal]::new($identity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $UserMode -and -not $administrator) {
    throw 'Run this script in an Administrator PowerShell to enable startup without an interactive login. -UserMode supports login-only hosting.'
}
& $python -B $entry verify --spec $specification --policy $policy
if ($LASTEXITCODE -ne 0) { throw 'Standalone service configuration is invalid.' }

$taskService = New-Object -ComObject Schedule.Service
$taskService.Connect()
$folder = $taskService.GetFolder('\')
$existing = $null
try { $existing = $folder.GetTask($taskName) }
catch { if ($_.Exception.HResult -ne -2147024894) { throw } }
$arguments = '-B "{0}" run --spec "{1}" --policy "{2}"' -f $entry, $specification, $policy
$serviceUser = $identity.User.Value
if ($null -ne $existing) {
    $action = $existing.Definition.Actions.Item(1)
    if ($existing.Definition.Actions.Count -ne 1 -or $action.Path -ine $python -or $action.Arguments -cne $arguments) {
        throw 'An existing task has a different owner action; preserve it for reconciliation.'
    }
    $serviceUser = $existing.Definition.Principal.UserId
    $backup = Join-Path $evidence ('task-before-' + [guid]::NewGuid().ToString('N') + '.xml')
    $stream = [System.IO.File]::Open($backup, [System.IO.FileMode]::CreateNew, [System.IO.FileAccess]::Write, [System.IO.FileShare]::Read)
    try { $bytes = [System.Text.UTF8Encoding]::new($false).GetBytes($existing.Xml); $stream.Write($bytes, 0, $bytes.Length); $stream.Flush($true) }
    finally { $stream.Dispose() }
}
$definition = $taskService.NewTask(0)
$definition.RegistrationInfo.Description = 'Independent Jenkins native lifecycle, health supervision and restart. No Codex process, sandbox, installation or session dependency.'
$definition.Principal.UserId = $serviceUser
$definition.Principal.LogonType = $(if ($UserMode) { 3 } else { 2 })
$definition.Principal.RunLevel = 0
$settings = $definition.Settings
$settings.Enabled = $true
$settings.Hidden = $true
$settings.StartWhenAvailable = $true
$settings.DisallowStartIfOnBatteries = $false
$settings.StopIfGoingOnBatteries = $false
$settings.ExecutionTimeLimit = 'PT0S'
$settings.Priority = 4
$settings.RestartInterval = 'PT1M'
$settings.RestartCount = 255
$settings.MultipleInstances = 2
$settings.AllowHardTerminate = $false
if (-not $UserMode) {
    $boot = $definition.Triggers.Create(8)
    $boot.Enabled = $true
    $boot.Delay = 'PT15S'
}
$login = $definition.Triggers.Create(9)
$login.UserId = $serviceUser
$login.Delay = 'PT15S'
$repeat = $definition.Triggers.Create(1)
$repeat.StartBoundary = (Get-Date).AddMinutes(1).ToString('yyyy-MM-ddTHH:mm:ss')
$repeat.Repetition.Interval = 'PT1M'
$repeat.Repetition.StopAtDurationEnd = $false
$action = $definition.Actions.Create(0)
$action.Path = $python
$action.Arguments = $arguments
$action.WorkingDirectory = $repository
$registered = $folder.RegisterTaskDefinition($taskName, $definition, 6, $serviceUser, $null, $definition.Principal.LogonType, $null)
$null = $registered.Run($null)
Write-Host 'Windows task registered. Waiting for Jenkins controller and agent (up to 10 minutes)...'

$healthy = $false
for ($tick = 0; $tick -lt 120; $tick++) {
    Start-Sleep -Seconds 5
    try {
        $health = & $python -B $entry health --spec $specification --policy $policy 2>$null
        if ($LASTEXITCODE -eq 0) { $healthy = $true; break }
    } catch { $healthy = $false }
}
if (-not $healthy) {
    & $python -B $entry status --spec $specification --policy $policy
    throw 'Task registration completed, but live Jenkins acceptance is still pending. Keep original attempts and inspect independent-service/status.json.'
}
$legacy = $null
try { $legacy = $folder.GetTask('ZirconEngine-Jenkins-Recovery-Host') }
catch { if ($_.Exception.HResult -ne -2147024894) { throw } }
if ($null -ne $legacy -and $legacy.Definition.Actions.Item(1).Arguments -match 'native-resident-r8-control\.ps1' -and $legacy.State -ne 4) {
    $legacy.Enabled = $false
}
$receipt = [ordered]@{
    task = $registered.Name
    observedAt = (Get-Date).ToUniversalTime().ToString('o')
    state = $registered.State
    mode = $(if ($UserMode) { 'native-current-login' } else { 'native-unattended-S4U' })
    clientLifetimeDependency = $false
    health = ($health -join "`n" | ConvertFrom-Json)
    restartSeconds = 60
    pollSeconds = 15
    historicalEvidencePreserved = $true
}
$output = Join-Path $evidence ('installation-' + [guid]::NewGuid().ToString('N') + '.json')
$stream = [System.IO.File]::Open($output, [System.IO.FileMode]::CreateNew, [System.IO.FileAccess]::Write, [System.IO.FileShare]::Read)
try { $bytes = [System.Text.UTF8Encoding]::new($false).GetBytes(($receipt | ConvertTo-Json -Depth 12)); $stream.Write($bytes, 0, $bytes.Length); $stream.Flush($true) }
finally { $stream.Dispose() }
$receipt | ConvertTo-Json -Depth 12
