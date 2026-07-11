[CmdletBinding(SupportsShouldProcess = $true)]
param(
    [ValidateSet("Install", "Update", "Remove", "Query", "Cutover", "Rollback")]
    [string]$Action = "Query",
    [string]$RepoRoot,
    [ValidateSet("ScheduledTask", "UserStartup")]
    [string]$Backend = "ScheduledTask",
    [switch]$DryRun,
    [string[]]$LegacyTaskName = @()
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $RepoRoot = Split-Path -Parent $PSScriptRoot
}

$resolvedRepoRoot = (Resolve-Path -LiteralPath $RepoRoot).Path
$encoded = [Text.Encoding]::UTF8.GetBytes($resolvedRepoRoot.ToLowerInvariant())
$hasher = [Security.Cryptography.SHA256]::Create()
try {
    $hashBytes = $hasher.ComputeHash($encoded)
} finally {
    $hasher.Dispose()
}
$digest = (($hashBytes | ForEach-Object { $_.ToString("X2") }) -join "").Substring(0, 10)
$daemonTask = "ZirconSessionCoordinator-$digest"
$maintenanceTask = "ZirconSessionMaintenance-$digest"
$startupValue = "ZirconSessionCoordinator-$digest"
$startupRegistryPath = "HKCU\Software\Microsoft\Windows\CurrentVersion\Run"
$client = Join-Path $resolvedRepoRoot "tools\zircon-session.ps1"
$cleanup = Join-Path $resolvedRepoRoot "tools\cleanup-stale-targets.ps1"
$stateRoot = Join-Path $resolvedRepoRoot ".codex\state\session-coordinator"
$cutoverRecord = Join-Path $stateRoot "task-cutover.json"
$powerShell = (Get-Command powershell.exe -ErrorAction Stop).Source

function Quote-TaskArgument {
    param([string]$Value)
    return '"' + $Value.Replace('"', '""') + '"'
}

function Invoke-TaskCommand {
    param(
        [string]$Description,
        [string[]]$Arguments
    )

    $rendered = "schtasks.exe " + (($Arguments | ForEach-Object { Quote-TaskArgument $_ }) -join " ")
    if ($DryRun) {
        Write-Output "[$Description] $rendered"
        return
    }
    if (-not $PSCmdlet.ShouldProcess($Description, $rendered)) {
        return
    }
    & schtasks.exe @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "Scheduled task operation failed: $Description"
    }
}

function Get-ScheduledTaskOrNull {
    param([string]$TaskName)
    if ($null -eq (Get-Command Get-ScheduledTask -ErrorAction SilentlyContinue)) {
        throw "Get-ScheduledTask is required to distinguish a missing task from an access failure."
    }
    try {
        return Get-ScheduledTask -TaskName $TaskName -ErrorAction Stop
    }
    catch {
        if ($_.CategoryInfo.Category -eq [Management.Automation.ErrorCategory]::ObjectNotFound) {
            return $null
        }
        throw
    }
}

function Invoke-TaskCommandIfExists {
    param(
        [string]$TaskName,
        [string]$Description,
        [string[]]$Arguments
    )
    if ($DryRun) {
        Invoke-TaskCommand -Description $Description -Arguments $Arguments
        return $true
    }
    if ($null -eq (Get-ScheduledTaskOrNull -TaskName $TaskName)) {
        return $false
    }
    Invoke-TaskCommand -Description $Description -Arguments $Arguments
    return $true
}

function New-TaskArguments {
    param([string]$ScriptPath, [string[]]$ScriptArguments)

    $parts = @(
        "-NoProfile",
        "-NonInteractive",
        "-WindowStyle Hidden",
        "-ExecutionPolicy Bypass",
        "-File", (Quote-TaskArgument $ScriptPath)
    )
    foreach ($argument in $ScriptArguments) {
        $parts += Quote-TaskArgument $argument
    }
    return $parts -join " "
}

function New-TaskCommandLine {
    param([string]$ScriptPath, [string[]]$ScriptArguments)
    $arguments = New-TaskArguments -ScriptPath $ScriptPath -ScriptArguments $ScriptArguments
    return (Quote-TaskArgument $powerShell) + " " + $arguments
}

function Set-UserStartupCommand {
    param([string]$CommandLine)
    if ($DryRun) {
        Write-Output "[$startupValue] set HKCU Run value to $CommandLine"
        return
    }
    if (-not $PSCmdlet.ShouldProcess($startupValue, "Set current-user startup command")) {
        return
    }
    & reg.exe ADD $startupRegistryPath /V $startupValue /T REG_SZ /D $CommandLine /F | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw "Current-user startup registration failed: $startupValue"
    }
}

function Get-UserStartupCommandOrNull {
    $providerPath = "Registry::HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run"
    try {
        $item = Get-ItemProperty -LiteralPath $providerPath -Name $startupValue -ErrorAction Stop
        return [string]$item.PSObject.Properties[$startupValue].Value
    }
    catch {
        if ($_.CategoryInfo.Category -eq [Management.Automation.ErrorCategory]::ObjectNotFound) {
            return $null
        }
        throw
    }
}

function Remove-UserStartupCommand {
    if ($DryRun) {
        Write-Output "[$startupValue] remove HKCU Run value"
        return
    }
    if (-not $PSCmdlet.ShouldProcess($startupValue, "Remove current-user startup command")) {
        return
    }
    $providerPath = "Registry::HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run"
    if ($null -eq (Get-UserStartupCommandOrNull)) { return }
    Remove-ItemProperty -LiteralPath $providerPath -Name $startupValue -ErrorAction Stop
    if ($null -ne (Get-UserStartupCommandOrNull)) {
        throw "Current-user startup removal verification failed: $startupValue"
    }
}

function Test-LegacyCleanupActionForRepo {
    param([string]$ActionText)
    $absoluteCleanup = [Regex]::Escape([IO.Path]::GetFullPath($cleanup))
    $absolutePattern = '(?i)(?:^|\s)["'']?' + $absoluteCleanup + '["'']?(?=\s|$)'
    if ($ActionText -match $absolutePattern) {
        return $true
    }
    $root = [Regex]::Escape([IO.Path]::GetFullPath($resolvedRepoRoot).TrimEnd('\', '/'))
    $relativeCleanup = 'tools[\\/]cleanup-stale-targets\.ps1'
    $rootPattern = '(?i)\bcd\s+/d\s+(?:"' + $root + '"|' + $root + ')(?=\s*(?:&&|&))'
    return $ActionText -match $rootPattern -and $ActionText -match $relativeCleanup
}

function Get-ExistingCutoverRecord {
    if (-not (Test-Path -LiteralPath $cutoverRecord)) {
        return $null
    }
    try {
        $record = Get-Content -Raw -LiteralPath $cutoverRecord | ConvertFrom-Json -ErrorAction Stop
        if ($null -eq $record.PSObject.Properties["status"] -and
            $null -ne $record.PSObject.Properties["cutover_at"]) {
            $record | Add-Member -NotePropertyName status -NotePropertyValue "active"
        }
        return $record
    }
    catch {
        throw "Cutover record is unreadable and cannot be overwritten safely: $cutoverRecord"
    }
}

function Get-RecordStringArray {
    param([object]$Record, [string]$PropertyName)
    if ($null -eq $Record) { return @() }
    $property = $Record.PSObject.Properties[$PropertyName]
    if ($null -eq $property -or $null -eq $property.Value) { return @() }
    return @($property.Value | ForEach-Object { [string]$_ })
}

function Assert-CompatibleActiveCutover {
    param([object]$Record)
    if ($null -eq $Record) { return }
    if ([string]$Record.status -eq "preparing") {
        throw "An interrupted preparing cutover exists. Run explicit Rollback before starting a new Cutover."
    }
    if ([string]$Record.status -ne "active") { return }
    $expectedBackend = if ($Backend -eq "UserStartup") { "user_startup" } else { "scheduled_task" }
    if (-not [string]::Equals([string]$Record.repo_root, $resolvedRepoRoot, [StringComparison]::OrdinalIgnoreCase) -or
        [string]$Record.backend -ne $expectedBackend) {
        throw "An active cutover record exists for a different repository or backend. Roll it back explicitly before changing backend."
    }
}

function Disable-ExternalMaintenanceTask {
    param([object]$Record)
    $task = if ($DryRun) { $null } else { Get-ScheduledTaskOrNull -TaskName $maintenanceTask }
    if ($DryRun -or ($null -ne $task -and $task.Settings.Enabled)) {
        Invoke-TaskCommand -Description "$maintenanceTask retire external scheduler" -Arguments @(
            "/Change", "/TN", $maintenanceTask, "/DISABLE"
        )
        if ($null -ne $Record) {
            if ($Record -is [hashtable]) {
                $Record.external_maintenance_disabled = $true
            }
            else {
                $Record | Add-Member -NotePropertyName external_maintenance_disabled -NotePropertyValue $true -Force
            }
            Write-CutoverRecord -Record $Record
        }
        return $true
    }
    return $false
}

function Restore-ExternalMaintenanceTaskIfRequired {
    param([object]$Record)
    if ($null -eq $Record) { return }
    $wasEnabled = if ($Record -is [Collections.IDictionary]) {
        $Record.Contains("maintenance_task_was_enabled") -and [bool]$Record["maintenance_task_was_enabled"]
    }
    else {
        $property = $Record.PSObject.Properties["maintenance_task_was_enabled"]
        $null -ne $property -and [bool]$property.Value
    }
    if (-not $wasEnabled) { return }
    $task = Get-ScheduledTaskOrNull -TaskName $maintenanceTask
    if ($null -eq $task) {
        throw "The previously enabled external maintenance task is missing: $maintenanceTask"
    }
    if (-not $task.Settings.Enabled) {
        Invoke-TaskCommand -Description "$maintenanceTask rollback enable" -Arguments @(
            "/Change", "/TN", $maintenanceTask, "/ENABLE"
        )
    }
}

function Get-LegacyCleanupTaskNames {
    if ($LegacyTaskName.Count -gt 0) {
        return @($LegacyTaskName | Sort-Object -Unique)
    }
    if ($null -eq (Get-Command Get-ScheduledTask -ErrorAction SilentlyContinue)) {
        return @()
    }
    return @(
        Get-ScheduledTask -ErrorAction SilentlyContinue |
            Where-Object {
                $task = $_
                $actionText = ($task.Actions | ForEach-Object {
                    $action = $_
                    $executeProperty = $action.PSObject.Properties["Execute"]
                    if ($null -eq $executeProperty) {
                        return ""
                    }
                    $argumentsProperty = $action.PSObject.Properties["Arguments"]
                    $arguments = if ($null -eq $argumentsProperty) { "" } else { [string]$argumentsProperty.Value }
                    "$([string]$executeProperty.Value) $arguments"
                }) -join " "
                $task.TaskName -notin @($daemonTask, $maintenanceTask) -and
                (Test-LegacyCleanupActionForRepo -ActionText $actionText)
            } |
            Select-Object -ExpandProperty TaskName |
            Sort-Object -Unique
    )
}

function Invoke-CoordinatorHealth {
    $output = & $client status -RepoRoot $resolvedRepoRoot -Json
    if ($LASTEXITCODE -ne 0 -or ($output -join "") -notmatch '"status"\s*:\s*"ok"') {
        throw "Coordinator health check failed during task cutover"
    }
}

function Invoke-MaintenanceTick {
    $output = & $client maintenance tick -RepoRoot $resolvedRepoRoot -Json
    if ($LASTEXITCODE -ne 0 -or ($output -join "") -notmatch '"status"\s*:\s*"succeeded"') {
        throw "Coordinator maintenance tick failed during task cutover"
    }
}

function Stop-CoordinatorForRollback {
    & $client status -RepoRoot $resolvedRepoRoot -Json *> $null
    if ($LASTEXITCODE -ne 0) {
        return
    }
    & $client stop -RepoRoot $resolvedRepoRoot -Json *> $null
    if ($LASTEXITCODE -ne 0) {
        throw "Coordinator stop failed during rollback"
    }
    for ($attempt = 0; $attempt -lt 50; $attempt++) {
        Start-Sleep -Milliseconds 100
        & $client status -RepoRoot $resolvedRepoRoot -Json *> $null
        if ($LASTEXITCODE -ne 0) {
            return
        }
    }
    throw "Coordinator remained online during rollback"
}

function Write-CutoverRecord {
    param([object]$Record)
    New-Item -ItemType Directory -Path $stateRoot -Force | Out-Null
    $temporary = "$cutoverRecord.tmp-$PID"
    $json = $Record | ConvertTo-Json -Depth 6
    [IO.File]::WriteAllText($temporary, $json, [Text.UTF8Encoding]::new($false))
    Move-Item -LiteralPath $temporary -Destination $cutoverRecord -Force
}

if ($Action -eq "Query") {
    if ($Backend -eq "UserStartup") {
        $output = & reg.exe QUERY $startupRegistryPath /V $startupValue 2>$null
        if ($LASTEXITCODE -ne 0) { Write-Output "$startupValue is not installed" } else { Write-Output ($output -join "`n") }
    }
    else {
        if ($null -eq (Get-ScheduledTaskOrNull -TaskName $daemonTask)) {
            Write-Output "$daemonTask is not installed"
        }
        else {
            Invoke-TaskCommand -Description $daemonTask -Arguments @("/Query", "/TN", $daemonTask, "/FO", "LIST", "/V")
        }
    }
    exit 0
}

if ($Action -eq "Remove") {
    if ($Backend -eq "UserStartup") {
        Remove-UserStartupCommand
    }
    else {
        $null = Invoke-TaskCommandIfExists -TaskName $daemonTask -Description $daemonTask -Arguments @(
            "/Delete", "/TN", $daemonTask, "/F"
        )
    }
    exit 0
}

if ($Action -eq "Rollback") {
    if (-not (Test-Path -LiteralPath $cutoverRecord)) {
        throw "No task cutover record exists at $cutoverRecord"
    }
    $record = Get-ExistingCutoverRecord
    if ([string]$record.backend -eq "user_startup") {
        Remove-UserStartupCommand
    }
    else {
        $null = Invoke-TaskCommandIfExists -TaskName $daemonTask -Description "$daemonTask rollback disable" -Arguments @(
            "/Change", "/TN", $daemonTask, "/DISABLE"
        )
    }
    Stop-CoordinatorForRollback
    foreach ($legacy in @($record.legacy_tasks)) {
        Invoke-TaskCommand -Description "$legacy rollback enable" -Arguments @(
            "/Change", "/TN", [string]$legacy, "/ENABLE"
        )
    }
    Restore-ExternalMaintenanceTaskIfRequired -Record $record
    $record | Add-Member -NotePropertyName status -NotePropertyValue "rolled_back" -Force
    $record | Add-Member -NotePropertyName rolled_back_at -NotePropertyValue ([DateTimeOffset]::Now.ToString("o")) -Force
    Write-CutoverRecord -Record $record
    Write-Output "Scheduled task rollback restored legacy task enablement."
    exit 0
}

if (-not (Test-Path -LiteralPath $client) -or -not (Test-Path -LiteralPath $cleanup)) {
    throw "Coordinator scripts are incomplete under $resolvedRepoRoot"
}

$daemonScriptArguments = @("start", "-RepoRoot", $resolvedRepoRoot, "-Automatic")
$daemonTaskArguments = New-TaskArguments -ScriptPath $client -ScriptArguments $daemonScriptArguments
$daemonCommand = New-TaskCommandLine -ScriptPath $client -ScriptArguments $daemonScriptArguments
$disabledLegacy = @()
$record = $null
$maintenanceDisabledThisAttempt = $false

if ($Action -eq "Cutover") {
    $existingRecord = if ($DryRun) { $null } else { Get-ExistingCutoverRecord }
    Assert-CompatibleActiveCutover -Record $existingRecord
    $legacyTasks = @(Get-LegacyCleanupTaskNames)
    $historicalLegacy = Get-RecordStringArray -Record $existingRecord -PropertyName "legacy_tasks"
    $legacyEnabledNow = @()
    if ($DryRun) {
        $legacyEnabledNow = $legacyTasks
    }
    else {
        foreach ($legacy in $legacyTasks) {
            $task = Get-ScheduledTask -TaskName $legacy -ErrorAction Stop
            if ($task.State -eq "Running") {
                throw "Legacy cleanup task is still running and cannot be cut over: $legacy"
            }
            if ($task.Settings.Enabled) {
                $legacyEnabledNow += $legacy
            }
        }
    }
    $allLegacyTasks = @($historicalLegacy + $legacyEnabledNow | Sort-Object -Unique)

    if ($null -ne $existingRecord -and [string]$existingRecord.status -eq "active") {
        if ($DryRun) {
            Write-Output "[Cutover journal] preserve active rollback record before checking an idempotent cutover"
            Write-Output "[Cutover health gate] two consecutive plan-only maintenance ticks"
            foreach ($legacy in $legacyTasks) {
                Write-Output "[Legacy idle check then disable if enabled] schtasks.exe /Change /TN `"$legacy`" /DISABLE"
            }
            Disable-ExternalMaintenanceTask -Record $null | Where-Object { $_ -isnot [bool] }
        }
        else {
            if ($Backend -eq "ScheduledTask") {
                $daemonRegistration = Get-ScheduledTaskOrNull -TaskName $daemonTask
                if ($null -eq $daemonRegistration -or -not $daemonRegistration.Settings.Enabled) {
                    throw "The active cutover record has no enabled coordinator startup task. Roll back before cutting over again."
                }
                $actions = @($daemonRegistration.Actions)
                $actionExecute = if ($actions.Count -eq 1) { [string]$actions[0].Execute } else { "" }
                $actionArguments = if ($actions.Count -eq 1) { [string]$actions[0].Arguments } else { "" }
                if ($actions.Count -ne 1 -or
                    -not [string]::Equals([IO.Path]::GetFullPath($actionExecute), [IO.Path]::GetFullPath($powerShell), [StringComparison]::OrdinalIgnoreCase) -or
                    -not [string]::Equals($actionArguments.Trim(), $daemonTaskArguments, [StringComparison]::Ordinal)) {
                    throw "The active cutover task action does not match the exact coordinator command. Roll back before cutting over again."
                }
            }
            elseif (-not [string]::Equals(
                (Get-UserStartupCommandOrNull),
                $daemonCommand,
                [StringComparison]::Ordinal
            )) {
                throw "The active cutover record does not have the exact coordinator user-startup command. Roll back before cutting over again."
            }
            $disabled = Get-RecordStringArray -Record $existingRecord -PropertyName "disabled_legacy_tasks"
            $disabledThisRun = @()
            $maintenanceRegistration = Get-ScheduledTaskOrNull -TaskName $maintenanceTask
            $maintenanceEnabledAtAttemptStart = $null -ne $maintenanceRegistration -and $maintenanceRegistration.Settings.Enabled
            $priorMaintenanceWasEnabled = $existingRecord.PSObject.Properties["maintenance_task_was_enabled"]
            $maintenanceWasEnabled = ($null -ne $priorMaintenanceWasEnabled -and [bool]$priorMaintenanceWasEnabled.Value) -or
                $maintenanceEnabledAtAttemptStart
            $existingRecord | Add-Member -NotePropertyName legacy_tasks -NotePropertyValue $allLegacyTasks -Force
            $existingRecord | Add-Member -NotePropertyName disabled_legacy_tasks -NotePropertyValue @($disabled | Sort-Object -Unique) -Force
            $existingRecord | Add-Member -NotePropertyName maintenance_task -NotePropertyValue $(
                if ($null -ne $maintenanceRegistration) { $maintenanceTask } else { $null }
            ) -Force
            $existingRecord | Add-Member -NotePropertyName maintenance_task_was_enabled -NotePropertyValue $maintenanceWasEnabled -Force
            Write-CutoverRecord -Record $existingRecord
            try {
                Invoke-CoordinatorHealth
                Invoke-MaintenanceTick
                Invoke-CoordinatorHealth
                Invoke-MaintenanceTick
                Invoke-CoordinatorHealth
                foreach ($legacy in $legacyTasks) {
                    $task = Get-ScheduledTask -TaskName $legacy -ErrorAction Stop
                    if ($task.State -eq "Running") {
                        throw "Legacy cleanup task is still running and cannot be cut over: $legacy"
                    }
                    if ($task.Settings.Enabled) {
                        Invoke-TaskCommand -Description "$legacy disable after idempotent cutover" -Arguments @(
                            "/Change", "/TN", $legacy, "/DISABLE"
                        )
                        $disabled += $legacy
                        $disabledThisRun += $legacy
                        $existingRecord | Add-Member -NotePropertyName disabled_legacy_tasks -NotePropertyValue @($disabled | Sort-Object -Unique) -Force
                        Write-CutoverRecord -Record $existingRecord
                    }
                }
                $maintenanceDisabledThisAttempt = Disable-ExternalMaintenanceTask -Record $existingRecord
                $existingRecord | Add-Member -NotePropertyName verified_at -NotePropertyValue ([DateTimeOffset]::Now.ToString("o")) -Force
                Write-CutoverRecord -Record $existingRecord
            }
            catch {
                foreach ($legacy in $disabledThisRun) {
                    Invoke-TaskCommand -Description "$legacy failed-idempotent-cutover re-enable" -Arguments @(
                        "/Change", "/TN", $legacy, "/ENABLE"
                    )
                }
                if ($maintenanceEnabledAtAttemptStart) {
                    Restore-ExternalMaintenanceTaskIfRequired -Record $existingRecord
                }
                $existingRecord | Add-Member -NotePropertyName last_verification_failed_at -NotePropertyValue ([DateTimeOffset]::Now.ToString("o")) -Force
                Write-CutoverRecord -Record $existingRecord
                throw
            }
        }
        Write-Output "Coordinator startup configuration ready for $resolvedRepoRoot ($Backend)"
        exit 0
    }

    $maintenanceRegistration = if ($DryRun) { $null } else { Get-ScheduledTaskOrNull -TaskName $maintenanceTask }
    $maintenanceWasEnabled = $null -ne $maintenanceRegistration -and $maintenanceRegistration.Settings.Enabled
    $record = @{
        status = "preparing"
        repo_root = $resolvedRepoRoot
        backend = if ($Backend -eq "UserStartup") { "user_startup" } else { "scheduled_task" }
        daemon_task = if ($Backend -eq "ScheduledTask") { $daemonTask } else { $null }
        maintenance_task = if ($null -ne $maintenanceRegistration) { $maintenanceTask } else { $null }
        maintenance_task_was_enabled = $maintenanceWasEnabled
        external_maintenance_disabled = $false
        startup_value = if ($Backend -eq "UserStartup") { $startupValue } else { $null }
        startup_configured = $false
        coordinator_started = $false
        legacy_tasks = $allLegacyTasks
        disabled_legacy_tasks = @()
        prepared_at = [DateTimeOffset]::Now.ToString("o")
    }
    if ($DryRun) {
        Write-Output "[Cutover journal] write preparing record before startup mutation"
    }
    else {
        Write-CutoverRecord -Record $record
    }
}

try {
    if ($Backend -eq "UserStartup") {
        Set-UserStartupCommand -CommandLine $daemonCommand
    }
    else {
        Invoke-TaskCommand -Description $daemonTask -Arguments @(
            "/Create", "/TN", $daemonTask, "/SC", "ONLOGON", "/TR", $daemonCommand,
            "/RL", "LIMITED", "/F"
        )
    }
    if ($Action -eq "Cutover" -and -not $DryRun) {
        $record.startup_configured = $true
        Write-CutoverRecord -Record $record
    }

    if (-not $DryRun) {
        & $client start -RepoRoot $resolvedRepoRoot | Out-Null
        if ($LASTEXITCODE -ne 0) {
            throw "Coordinator health check failed after startup configuration $Action"
        }
        if ($Action -eq "Cutover") {
            $record.coordinator_started = $true
            Write-CutoverRecord -Record $record
        }
    }

    if ($Action -eq "Cutover") {
        if ($DryRun) {
            Write-Output "[Cutover health gate] two consecutive plan-only maintenance ticks"
            foreach ($legacy in $legacyTasks) {
                Write-Output "[Legacy idle check then disable] schtasks.exe /Change /TN `"$legacy`" /DISABLE"
            }
            Disable-ExternalMaintenanceTask -Record $null | Where-Object { $_ -isnot [bool] }
        }
        else {
            Invoke-CoordinatorHealth
            Invoke-MaintenanceTick
            Invoke-CoordinatorHealth
            Invoke-MaintenanceTick
            Invoke-CoordinatorHealth
            $legacyToDisable = @()
            foreach ($legacy in $legacyTasks) {
                $task = Get-ScheduledTask -TaskName $legacy -ErrorAction Stop
                if ($task.State -eq "Running") {
                    throw "Legacy cleanup task is still running and cannot be cut over: $legacy"
                }
                if ($task.Settings.Enabled) {
                    $legacyToDisable += $legacy
                }
            }
            foreach ($legacy in $legacyToDisable) {
                Invoke-TaskCommand -Description "$legacy disable after cutover" -Arguments @(
                    "/Change", "/TN", $legacy, "/DISABLE"
                )
                $disabledLegacy += $legacy
                $record.disabled_legacy_tasks = @($disabledLegacy)
                Write-CutoverRecord -Record $record
            }
            $maintenanceDisabledThisAttempt = Disable-ExternalMaintenanceTask -Record $record
            $record.status = "active"
            $record.cutover_at = [DateTimeOffset]::Now.ToString("o")
            Write-CutoverRecord -Record $record
        }
    }
    else {
        if ($DryRun) {
            Disable-ExternalMaintenanceTask -Record $null | Where-Object { $_ -isnot [bool] }
        }
        else {
            $maintenanceDisabledThisAttempt = Disable-ExternalMaintenanceTask -Record $null
        }
    }
}
catch {
    if ($Action -eq "Cutover" -and -not $DryRun) {
        try {
            if ($Backend -eq "UserStartup") {
                Remove-UserStartupCommand
            }
            else {
                $null = Invoke-TaskCommandIfExists -TaskName $daemonTask -Description "$daemonTask failed-cutover disable" -Arguments @(
                    "/Change", "/TN", $daemonTask, "/DISABLE"
                )
            }
            Stop-CoordinatorForRollback
            foreach ($legacy in $disabledLegacy) {
                Invoke-TaskCommand -Description "$legacy failed-cutover re-enable" -Arguments @(
                    "/Change", "/TN", $legacy, "/ENABLE"
                )
            }
            Restore-ExternalMaintenanceTaskIfRequired -Record $record
            if ($null -ne $record) {
                $record.status = "rolled_back"
                $record.rolled_back_at = [DateTimeOffset]::Now.ToString("o")
                Write-CutoverRecord -Record $record
            }
        }
        catch {
            throw "Cutover failed and automatic rollback also failed: $($_.Exception.Message)"
        }
    }
    throw
}

Write-Output "Coordinator startup configuration ready for $resolvedRepoRoot ($Backend)"
