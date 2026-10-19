[CmdletBinding()]
param([switch]$UserMode, [switch]$EnableControls, [switch]$RegisterOnly,
      [switch]$RegisterHandoff,
      [ValidateSet('observe','pause','stop-instance','activate')][string]$HandoffStep,
      [string]$HandoffPlan, [string]$HandoffSha,
      [string]$HandoffEvidence, [string]$HandoffEvidenceSha)

$ErrorActionPreference = 'Stop'
if ($EnableControls -and -not $RegisterOnly) { throw 'Controlled resident activation requires the external managed native handoff; register-only first.' }
if ($RegisterOnly -and -not $EnableControls) { throw 'Register-only is reserved for the finite controlled-resident upgrade.' }
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

if ($RegisterHandoff -or $HandoffStep) {
    if ($UserMode -or $EnableControls -or $RegisterOnly -or ($RegisterHandoff -and $HandoffStep)) {
        throw 'Initial handoff is a separate finite unattended task action.'
    }
    function Read-HandoffReference($reference) {
        $path = [IO.Path]::GetFullPath([string]$reference.path)
        $cursor = $path
        while ($null -ne $cursor) {
            $item = Get-Item -LiteralPath $cursor
            if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'Handoff reference contains a link.' }
            $parent = [IO.Directory]::GetParent($cursor)
            $cursor = $(if ($null -ne $parent) { $parent.FullName } else { $null })
        }
        $bytes = [IO.File]::ReadAllBytes($path)
        $sha = [BitConverter]::ToString([Security.Cryptography.SHA256]::Create().ComputeHash($bytes)).Replace('-','').ToLowerInvariant()
        if ($sha -cne [string]$reference.sha256 -or $bytes.Length -ne [long]$reference.bytes) {
            throw 'Handoff reference bytes changed.'
        }
        return [Text.UTF8Encoding]::new($false,$true).GetString($bytes)
    }
    function Get-TextSha([string]$text) {
        return [BitConverter]::ToString([Security.Cryptography.SHA256]::Create().ComputeHash([Text.UTF8Encoding]::new($false).GetBytes($text))).Replace('-','').ToLowerInvariant()
    }
    if ($HandoffSha -cnotmatch '^[0-9a-f]{64}$' -or -not $HandoffPlan) { throw 'An exact handoff plan is required.' }
    $planPath = [IO.Path]::GetFullPath($HandoffPlan)
    $planBytes = [IO.File]::ReadAllBytes($planPath)
    $planText = [Text.UTF8Encoding]::new($false,$true).GetString($planBytes)
    if ((Get-TextSha $planText) -cne $HandoffSha) { throw 'Handoff plan SHA differs.' }
    $plan = $planText | ConvertFrom-Json
    $handoffRoot = Join-Path $evidence ('handoffs\' + $plan.requestId)
    if ($plan.requestId -cnotmatch '^[0-9a-f]{32}$' -or $plan.schemaVersion -ne 1 -or
            $plan.kind -cne 'independent-service-resident-handoff-v1' -or
            $plan.repository -ine $repository -or $plan.taskName -cne $taskName -or
            $plan.handoffTaskName -cne 'ZirconEngine-Jenkins-Service-Handoff' -or
            $planPath -ine (Join-Path $handoffRoot 'plan.json')) { throw 'Handoff plan boundary differs.' }
    if ($plan.sourceRefs.taskInstaller.path -ine $PSCommandPath) { throw 'Handoff executor path differs.' }
    $null = Read-HandoffReference $plan.sourceRefs.taskInstaller
    $null = Read-HandoffReference $plan.artifactsRef
    $previousNames = @('previousDesiredRef','previousInitialHandoffRef','previousPolicyRef')
    $previousCount = @($previousNames | Where-Object { $null -ne $plan.PSObject.Properties[$_] }).Count
    $successor = $previousCount -gt 0
    if ($successor) {
        if ($previousCount -ne 3) { throw 'Successor requires all three explicit predecessor references.' }
        $previousDesired = (Read-HandoffReference $plan.previousDesiredRef) | ConvertFrom-Json
        $previousReceipt = (Read-HandoffReference $plan.previousInitialHandoffRef) | ConvertFrom-Json
        $previousPlan = (Read-HandoffReference $previousReceipt.planRef) | ConvertFrom-Json
        $null = Read-HandoffReference $plan.previousPolicyRef
        if ($previousDesired.phase -cne 'completed' -or $previousDesired.desiredState -cne 'running' -or
                $previousDesired.startedOperationId -cne $plan.runtimeOperationId -or
                $previousDesired.initialHandoffRef.path -cne $plan.previousInitialHandoffRef.path -or
                $previousDesired.initialHandoffRef.sha256 -cne $plan.previousInitialHandoffRef.sha256 -or
                $previousDesired.initialHandoffRef.bytes -ne $plan.previousInitialHandoffRef.bytes -or
                $previousReceipt.kind -cne 'independent-service-resident-handoff' -or
                $previousReceipt.status -cne 'predecessor-retired') { throw 'Successor predecessor differs.' }
        if ($RegisterHandoff) {
            $null = Read-HandoffReference @{path=(Join-Path $evidence 'desired.json');sha256=$plan.previousDesiredRef.sha256;bytes=$plan.previousDesiredRef.bytes}
            $null = Read-HandoffReference @{path=$policy;sha256=$plan.previousPolicyRef.sha256;bytes=$plan.previousPolicyRef.bytes}
        }
    }
    $beforeXml = Read-HandoffReference $plan.taskBeforeRef
    $serviceTask = New-Object -ComObject Schedule.Service
    $serviceTask.Connect()
    $serviceFolder = $serviceTask.GetFolder('\')
    $task = $serviceFolder.GetTask($taskName)
    $taskUser = $task.Definition.Principal.UserId
    $sid = if ($taskUser -match '^S-1-') { $taskUser } else {
        ([Security.Principal.NTAccount]::new($taskUser)).Translate([Security.Principal.SecurityIdentifier]).Value
    }
    if ($sid -cne $identity.User.Value -or $task.Definition.Principal.LogonType -ne 2 -or
            $task.Definition.Principal.RunLevel -ne 0) { throw 'Handoff task owner is not the existing S4U/LUA owner.' }
    $legacyArgs = '-B "{0}" run --spec "{1}" --policy "{2}"' -f $entry,$specification,$policy
    $beforeArgs = if ($successor) { $legacyArgs + ' --service-controls' } else { $legacyArgs }
    $definition = $serviceTask.NewTask(0)
    $definition.XmlText = $beforeXml
    if ($definition.Actions.Count -ne 1 -or $definition.Actions.Item(1).Path -ine $python -or
            $definition.Actions.Item(1).Arguments -cne $beforeArgs -or
            $definition.Actions.Item(1).WorkingDirectory -ine $repository -or
            $definition.Principal.LogonType -ne 2 -or $definition.Principal.RunLevel -ne 0) {
        throw 'Handoff original task action differs.'
    }
    $pausedDefinition = $serviceTask.NewTask(0)
    $pausedDefinition.XmlText = $beforeXml
    foreach ($trigger in $pausedDefinition.Triggers) { $trigger.Enabled = $false }
    $pausedDefinition.Settings.RestartCount = 0
    # Keep task Enabled and its original AllowHardTerminate; instance.Stop is
    # attempted honestly and the separately held native root handle proves exit.
    $pausedXml = $pausedDefinition.XmlText
    function Assert-TaskXml([string]$expected) {
        [xml]$actualDocument = $task.Xml
        [xml]$expectedDocument = $expected
        if ($actualDocument.OuterXml -cne $expectedDocument.OuterXml) { throw 'Task XML preimage changed.' }
    }
    function Get-OwnedInstance($evidenceRow) {
        $instances = @($task.GetInstances(0))
        if ($instances.Count -ne 1 -or $instances[0].InstanceGuid -cne $evidenceRow.instanceGuid -or
                [int]$instances[0].EnginePID -ne [int]$evidenceRow.enginePid) {
            throw 'Original Task Scheduler instance changed.'
        }
        return $instances[0]
    }
    $helperName = 'ZirconEngine-Jenkins-Service-Handoff'
    $helperArgs = '-B "{0}" handoff --spec "{1}" --policy "{2}" --handoff-plan "{3}" --handoff-sha {4}' -f $entry,$specification,$policy,$planPath,$HandoffSha
    if ($RegisterHandoff) {
        Assert-TaskXml $beforeXml
        if (@($task.GetInstances(0)).Count -ne 1) { throw 'Original service does not have one task instance.' }
        $oldHelper = $null
        try { $oldHelper = $serviceFolder.GetTask($helperName) }
        catch { if ($_.Exception.HResult -ne -2147024894) { throw } }
        $helperRegistration = 2
        if ($null -ne $oldHelper) {
            if ($oldHelper.Definition.Actions.Count -ne 1 -or $oldHelper.Definition.Actions.Item(1).Path -ine $python -or
                    $oldHelper.Definition.Principal.UserId -ine $taskUser -or
                    $oldHelper.Definition.Principal.LogonType -ne 2 -or $oldHelper.Definition.Principal.RunLevel -ne 0) {
                throw 'A different handoff task exists; retain it.'
            }
            if ($oldHelper.Definition.Actions.Item(1).Arguments -ceq $helperArgs) {
                [ordered]@{status='existing-handoff-observe-only';task=$helperName;state=$oldHelper.State;newRunIssued=$false} | ConvertTo-Json -Compress
                exit 0
            }
            if (-not $successor -or -not $plan.handoffTaskBeforeRef -or
                    $previousPlan.handoffTaskName -cne $helperName) { throw 'Different handoff task has no explicit completed predecessor.' }
            $helperBeforeXml = Read-HandoffReference $plan.handoffTaskBeforeRef
            [xml]$helperActualDocument = $oldHelper.Xml
            [xml]$helperBeforeDocument = $helperBeforeXml
            $previousHelperArgs = '-B "{0}" handoff --spec "{1}" --policy "{2}" --handoff-plan "{3}" --handoff-sha {4}' -f $entry,$specification,$policy,$previousReceipt.planRef.path,$previousReceipt.planRef.sha256
            if ($helperActualDocument.OuterXml -cne $helperBeforeDocument.OuterXml -or
                    $oldHelper.Definition.Actions.Item(1).Arguments -cne $previousHelperArgs -or
                    $oldHelper.Definition.Actions.Item(1).WorkingDirectory -ine $repository -or
                    $oldHelper.Definition.Triggers.Count -ne 0 -or $oldHelper.Definition.Settings.RestartCount -ne 0 -or
                    @($oldHelper.GetInstances(0)).Count -ne 0 -or $oldHelper.State -ne 3) {
                throw 'Previous finite handoff task is changed or still active.'
            }
            $helper = $serviceTask.NewTask(0)
            $helper.XmlText = $helperBeforeXml
            $helper.Actions.Item(1).Arguments = $helperArgs
            $helperRegistration = 6
        } else {
            if ($plan.handoffTaskBeforeRef) { throw 'Expected previous handoff task is absent.' }
            $helper = $serviceTask.NewTask(0)
            $helper.RegistrationInfo.Description = 'One reviewed independent Jenkins resident handoff. No recurring trigger or client lifetime dependency.'
            $helper.Principal.UserId = $taskUser
            $helper.Principal.LogonType = 2
            $helper.Principal.RunLevel = 0
            $helper.Settings.Enabled = $true
            $helper.Settings.Hidden = $true
            $helper.Settings.ExecutionTimeLimit = 'PT0S'
            $helper.Settings.AllowHardTerminate = $false
            $helper.Settings.MultipleInstances = 2
            $helper.Settings.DisallowStartIfOnBatteries = $false
            $helper.Settings.StopIfGoingOnBatteries = $false
            $helper.Settings.RestartCount = 0
            $action = $helper.Actions.Create(0)
            $action.Path = $python
            $action.Arguments = $helperArgs
            $action.WorkingDirectory = $repository
        }
        if ($helperRegistration -eq 6) {
            [xml]$helperActualDocument = $oldHelper.Xml
            if ($helperActualDocument.OuterXml -cne $helperBeforeDocument.OuterXml -or
                    @($oldHelper.GetInstances(0)).Count -ne 0) { throw 'Previous handoff Task CAS changed.' }
        }
        $helperTask = $serviceFolder.RegisterTaskDefinition($helperName,$helper,$helperRegistration,$taskUser,$null,2,$null)
        [xml]$helperExpectedDocument = $helper.XmlText
        [xml]$helperRegisteredDocument = $helperTask.Xml
        if ($helperRegisteredDocument.OuterXml -cne $helperExpectedDocument.OuterXml -or
                $helperTask.Definition.Actions.Item(1).Arguments -cne $helperArgs -or $helperTask.Definition.Triggers.Count -ne 0) {
            throw 'Registered handoff task differs; do not run.'
        }
        $run = $helperTask.Run($null)
        [ordered]@{status='handoff-task-started';task=$helperName;instanceGuid=$run.InstanceGuid;
            enginePid=$run.EnginePID;serviceTaskChanged=$false;newRunIssued=$true;healthyClaimed=$false} | ConvertTo-Json -Compress
        exit 0
    }
    $evidenceRow = $null
    if ($HandoffStep -ne 'observe') {
        if (-not $HandoffEvidence -or $HandoffEvidenceSha -cnotmatch '^[0-9a-f]{64}$' -or
                [IO.Path]::GetDirectoryName([IO.Path]::GetFullPath($HandoffEvidence)) -ine $handoffRoot) {
            throw 'Handoff step needs its exact same-operation evidence.'
        }
        $evidenceRef = @{path=$HandoffEvidence;sha256=$HandoffEvidenceSha;bytes=(Get-Item -LiteralPath $HandoffEvidence).Length}
        $evidenceRow = (Read-HandoffReference $evidenceRef) | ConvertFrom-Json
    }
    switch ($HandoffStep) {
        'observe' {
            Assert-TaskXml $beforeXml
            $instances = @($task.GetInstances(0))
            if ($instances.Count -ne 1) { throw 'Original service task instance is ambiguous.' }
            $helperTask = $serviceFolder.GetTask($helperName)
            $helpers = @($helperTask.GetInstances(0))
            if ($helpers.Count -ne 1 -or $helperTask.Definition.Actions.Count -ne 1 -or
                    $helperTask.Definition.Actions.Item(1).Path -ine $python -or
                    $helperTask.Definition.Actions.Item(1).Arguments -cne $helperArgs -or
                    $helperTask.Definition.Principal.UserId -ine $taskUser -or
                    $helperTask.Definition.Principal.LogonType -ne 2 -or $helperTask.Definition.Principal.RunLevel -ne 0) {
                throw 'Handoff must be owned by its separate fixed Scheduler task.'
            }
            [ordered]@{schemaVersion=1;kind='independent-service-task-instance';taskPath=$task.Path;
                instanceGuid=$instances[0].InstanceGuid;enginePid=$instances[0].EnginePID;
                handoffTaskPath=$helperTask.Path;handoffInstanceGuid=$helpers[0].InstanceGuid;
                handoffEnginePid=$helpers[0].EnginePID;
                taskXmlSha256=(Get-TextSha $task.Xml);resident=$plan.resident} | ConvertTo-Json -Compress -Depth 6
        }
        'pause' {
            Assert-TaskXml $beforeXml
            $null = Get-OwnedInstance $evidenceRow
            $task = $serviceFolder.RegisterTaskDefinition($taskName,$pausedDefinition,6,$taskUser,$null,2,$null)
            Assert-TaskXml $pausedXml
            $null = Get-OwnedInstance $evidenceRow
            [ordered]@{status='original-triggers-paused';taskPath=$task.Path;
                beforeSha256=(Get-TextSha $beforeXml);afterSha256=(Get-TextSha $task.Xml);
                serviceActionChanged=$false;allowHardTerminate=$task.Definition.Settings.AllowHardTerminate} | ConvertTo-Json -Compress
        }
        'stop-instance' {
            Assert-TaskXml $pausedXml
            $instance = Get-OwnedInstance $evidenceRow
            $stopHresult = 0
            try { $instance.Stop() } catch { $stopHresult = $_.Exception.HResult }
            [ordered]@{status='instance-stop-attempted';taskPath=$task.Path;instanceGuid=$evidenceRow.instanceGuid;
                enginePid=$evidenceRow.enginePid;instanceStopAttempted=$true;hresult=$stopHresult;
                residentExitClaimed=$false;jobProofIssued=$false} | ConvertTo-Json -Compress
        }
        'activate' {
            Assert-TaskXml $pausedXml
            if ($evidenceRow.kind -cne 'windows-native-resident-process-exit' -or
                    $evidenceRow.rootProcessTerminal -ne $true -or $evidenceRow.waitResult -ne 0 -or
                    $evidenceRow.jobProofIssued -ne $false -or $evidenceRow.eofProofIssued -ne $false -or
                    $evidenceRow.planRef.sha256 -cne $HandoffSha -or
                    [int]$evidenceRow.identity.pid -ne [int]$plan.resident.pid -or
                    [string]$evidenceRow.identity.creationTime -cne [string]$plan.resident.creationTime) {
                throw 'Original resident has no bound actual root-process exit proof.'
            }
            $deadline = (Get-Date).AddSeconds(30)
            while (@($task.GetInstances(0)).Count -ne 0 -and (Get-Date) -lt $deadline) { Start-Sleep -Milliseconds 200 }
            if (@($task.GetInstances(0)).Count -ne 0) { throw 'Original Task Scheduler instance has not completed.' }
            if (-not (Test-Path -LiteralPath (Join-Path $evidence 'desired.json'))) { throw 'Actual handoff has not initialized desired state.' }
            $definition.Actions.Item(1).Arguments = $legacyArgs + ' --service-controls'
            $controlledXml = $definition.XmlText
            $task = $serviceFolder.RegisterTaskDefinition($taskName,$definition,6,$taskUser,$null,2,$null)
            Assert-TaskXml $controlledXml
            $run = $task.Run($null)
            [ordered]@{status='controlled-resident-task-started';taskPath=$task.Path;instanceGuid=$run.InstanceGuid;
                enginePid=$run.EnginePID;beforeTaskXmlSha256=(Get-TextSha $beforeXml);
                afterTaskXmlSha256=(Get-TextSha $task.Xml);originalTriggersRestored=$true;
                runtimeBindingsVerified=$false;healthyClaimed=$false} | ConvertTo-Json -Compress
        }
    }
    exit 0
}

if (-not $UserMode -and -not $administrator -and -not ($EnableControls -and $RegisterOnly)) {
    throw 'Run this script in an Administrator PowerShell to enable startup without an interactive login. -UserMode supports login-only hosting.'
}
$verificationArgs = @('-B', $entry, 'verify', '--spec', $specification, '--policy', $policy)
if ($EnableControls) { $verificationArgs += '--service-controls' }
& $python @verificationArgs
if ($LASTEXITCODE -ne 0) { throw 'Standalone service configuration is invalid.' }

$taskService = New-Object -ComObject Schedule.Service
$taskService.Connect()
$folder = $taskService.GetFolder('\')
$existing = $null
try { $existing = $folder.GetTask($taskName) }
catch { if ($_.Exception.HResult -ne -2147024894) { throw } }
$arguments = '-B "{0}" run --spec "{1}" --policy "{2}"' -f $entry, $specification, $policy
$legacyArguments = $arguments
if ($EnableControls) {
    if ($UserMode -or $null -eq $existing -or $existing.Definition.Principal.LogonType -ne 2 -or
            $existing.Definition.Principal.RunLevel -ne 0) {
        throw 'Controls preserve the existing owned S4U/LUA service task only.'
    }
    $ownerSid = if ($existing.Definition.Principal.UserId -match '^S-1-') {
        $existing.Definition.Principal.UserId
    } else {
        ([Security.Principal.NTAccount]::new($existing.Definition.Principal.UserId)).Translate([Security.Principal.SecurityIdentifier]).Value
    }
    if ($ownerSid -cne $identity.User.Value) { throw 'Existing S4U task belongs to another Windows owner.' }
    $arguments += ' --service-controls'
}
$serviceUser = $identity.User.Value
if ($null -ne $existing) {
    $action = $existing.Definition.Actions.Item(1)
    if ($existing.Definition.Actions.Count -ne 1 -or $action.Path -ine $python -or ($action.Arguments -cne $arguments -and (-not $EnableControls -or $action.Arguments -cne $legacyArguments))) {
        throw 'An existing task has a different owner action; preserve it for reconciliation.'
    }
    $serviceUser = $existing.Definition.Principal.UserId
    $taskBeforeXml = $existing.Xml
    $registrationId = [guid]::NewGuid().ToString('N')
    $backup = Join-Path $evidence ('task-before-' + $registrationId + '.xml')
    if ($EnableControls) {
        $controlReceipt = Join-Path $evidence ('installation-' + $registrationId + '.json')
        $artifactPath = Join-Path $evidence ('task-registration-' + $registrationId + '.artifacts.json')
        $artifact = [ordered]@{schemaVersion=1;taskId=$registrationId;owner='independent-service-registration';
            outputs=@($backup,$controlReceipt);dependencies=@($policy,$entry,$specification);
            retention='Retain task preimage and actual registration result; activation requires external native handoff.'}
        $artifactStream = [IO.File]::Open($artifactPath,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
        try { $artifactBytes=[Text.UTF8Encoding]::new($false).GetBytes(($artifact | ConvertTo-Json -Depth 5)); $artifactStream.Write($artifactBytes,0,$artifactBytes.Length); $artifactStream.Flush($true) }
        finally { $artifactStream.Dispose() }
    }
    $stream = [System.IO.File]::Open($backup, [System.IO.FileMode]::CreateNew, [System.IO.FileAccess]::Write, [System.IO.FileShare]::Read)
    try { $bytes = [System.Text.UTF8Encoding]::new($false).GetBytes($existing.Xml); $stream.Write($bytes, 0, $bytes.Length); $stream.Flush($true) }
    finally { $stream.Dispose() }
}
if ($EnableControls) {
    # Keep every original trigger, setting and principal; change only the action.
    $definition = $existing.Definition
    $definition.Actions.Item(1).Arguments = $arguments
} else {
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
}
$registered = $folder.RegisterTaskDefinition($taskName, $definition, 6, $serviceUser, $null, $definition.Principal.LogonType, $null)
if ($EnableControls) {
    [xml]$taskBeforeDocument = $taskBeforeXml
    [xml]$taskAfterDocument = $registered.Xml
    $taskNamespace = [Xml.XmlNamespaceManager]::new($taskAfterDocument.NameTable)
    $taskNamespace.AddNamespace('t','http://schemas.microsoft.com/windows/2004/02/mit/task')
    $taskActualArguments = $taskAfterDocument.SelectNodes('/t:Task/t:Actions/t:Exec/t:Arguments',$taskNamespace)
    if ($taskActualArguments.Count -ne 1 -or $taskActualArguments[0].InnerText -cne $arguments) {
        throw 'Registered controlled task action differs; activation remains pending.'
    }
    # Use the immutable task preimage for the original action value.
    $taskBeforeNamespace = [Xml.XmlNamespaceManager]::new($taskBeforeDocument.NameTable)
    $taskBeforeNamespace.AddNamespace('t','http://schemas.microsoft.com/windows/2004/02/mit/task')
    $taskActualArguments[0].InnerText = $taskBeforeDocument.SelectSingleNode('/t:Task/t:Actions/t:Exec/t:Arguments',$taskBeforeNamespace).InnerText
    if ($taskAfterDocument.OuterXml -cne $taskBeforeDocument.OuterXml) {
        throw 'Registration changed another task field; activation remains pending.'
    }
}
$health = $null
if (-not $RegisterOnly) {
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
}
$receipt = [ordered]@{
    task = $registered.Name
    observedAt = (Get-Date).ToUniversalTime().ToString('o')
    state = $registered.State
    mode = $(if ($UserMode) { 'native-current-login' } else { 'native-unattended-S4U' })
    clientLifetimeDependency = $false
    health = $(if ($RegisterOnly) { $null } else { $health -join "`n" | ConvertFrom-Json })
    controlsRequested = [bool]$EnableControls
    activationPending = [bool]$RegisterOnly
    serviceActionPerformed = (-not $RegisterOnly)
    restartSeconds = 60
    pollSeconds = 15
    historicalEvidencePreserved = $true
}
$output = if ($EnableControls) { $controlReceipt } else { Join-Path $evidence ('installation-' + [guid]::NewGuid().ToString('N') + '.json') }
$stream = [System.IO.File]::Open($output, [System.IO.FileMode]::CreateNew, [System.IO.FileAccess]::Write, [System.IO.FileShare]::Read)
try { $bytes = [System.Text.UTF8Encoding]::new($false).GetBytes(($receipt | ConvertTo-Json -Depth 12)); $stream.Write($bytes, 0, $bytes.Length); $stream.Flush($true) }
finally { $stream.Dispose() }
$receipt | ConvertTo-Json -Depth 12
