[CmdletBinding(DefaultParameterSetName = 'Run')]
param(
    [Parameter(Mandatory = $true, ParameterSetName = 'Run')][string]$StartupFile,
    [Parameter(Mandatory = $true, ParameterSetName = 'Run')][ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedStartupSha256,
    [Parameter(Mandatory = $true, ParameterSetName = 'ExitPolicy')][string]$ExitPolicyCasesFile,
    [Parameter(ParameterSetName = 'Run')][switch]$Resident,
    [Parameter(ParameterSetName = 'Run')][switch]$PrepareOnly,
    [Parameter(ParameterSetName = 'Run')][switch]$CheckOnly
)
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$taskModuleRoot = Join-Path $env:SystemRoot 'System32\WindowsPowerShell\v1.0\Modules'
$env:PSModulePath = $taskModuleRoot
foreach ($taskModule in @('Microsoft.PowerShell.Utility','Microsoft.PowerShell.Management','Microsoft.PowerShell.Security','CimCmdlets')) {
    Import-Module (Join-Path $taskModuleRoot ($taskModule+'\'+$taskModule+'.psd1')) -ErrorAction Stop
}
function Test-TaskPublicExitContinuation($Facts) {
    return [bool](
        $Facts.role -ceq 'retained-public' -and $Facts.resident -eq $true -and
        $Facts.child.retainedExactProcess -eq $true -and $Facts.child.nativeExitObserved -eq $true -and
        $Facts.child.pid -gt 0 -and [string]$Facts.child.creationTime -cmatch '^[0-9]+$' -and
        $Facts.expectedRelease.path -is [string] -and $Facts.expectedRelease.path.Length -gt 0 -and
        [string]$Facts.expectedRelease.sha256 -cmatch '^[0-9a-f]{64}$' -and
        $Facts.confirmed.strictActualConfigRplusBDenied -eq $true -and
        $Facts.confirmed.nativeOwnerObserved -eq $true -and
        $Facts.confirmed.release.path -ceq $Facts.expectedRelease.path -and
        $Facts.confirmed.release.sha256 -ceq $Facts.expectedRelease.sha256 -and
        [string]$Facts.confirmed.runtimeOperationId -cmatch '^start-[0-9]+$' -and
        [string]$Facts.confirmed.runtimeReceipt.sha256 -cmatch '^[0-9a-f]{64}$' -and
        [string]$Facts.confirmed.generationProof.sha256 -cmatch '^[0-9a-f]{64}$' -and
        [string]$Facts.confirmed.descriptor.sha256 -cmatch '^[0-9a-f]{64}$' -and
        $Facts.confirmed.keeper.pid -gt 0 -and
        [string]$Facts.confirmed.keeper.creationTime -cmatch '^[0-9]+$' -and
        $Facts.currentDescriptorExact -eq $true -and
        $Facts.keeperIdentityStatus -ceq 'terminated-exact'
    )
}
if ($PSCmdlet.ParameterSetName -ceq 'ExitPolicy') {
    # A bounded read-only policy evaluator; it cannot reach startup or process control.
    if ((Get-Item -LiteralPath $ExitPolicyCasesFile).Length -gt 1048576) { throw 'Exit policy cases exceed their bound.' }
    $taskPolicyCases=Get-Content -LiteralPath $ExitPolicyCasesFile -Raw | ConvertFrom-Json
    if ($taskPolicyCases.Count -gt 128) { throw 'Too many exit policy cases.' }
    $taskPolicyResults=@()
    foreach ($taskPolicyCase in $taskPolicyCases) {
        $taskPolicyResults+=@{id=[string]$taskPolicyCase.id;allowContinue=(Test-TaskPublicExitContinuation $taskPolicyCase.facts)}
    }
    [pscustomobject]@{languageMode=[string]$ExecutionContext.SessionState.LanguageMode;cases=@($taskPolicyResults)} | ConvertTo-Json -Depth 8 -Compress
    exit 0
}
$taskRepository = [IO.Path]::GetFullPath((Split-Path -Parent (Split-Path -Parent $PSScriptRoot)))
$taskStartupPath = (Resolve-Path -LiteralPath $StartupFile).ProviderPath
try { [Diagnostics.Process]::GetCurrentProcess().PriorityClass=[Diagnostics.ProcessPriorityClass]::AboveNormal } catch { Write-Warning 'The native dispatcher priority could not be raised.' }
function Get-TaskSha([string]$Path) { return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant() }
function ConvertTo-TaskArgument([string]$Value) {
    if ($Value -notmatch '[\s"]' -and $Value.Length) { return $Value }
    $taskQuoted = [regex]::Replace($Value, '(\\*)"', '$1$1\"')
    $taskQuoted = [regex]::Replace($taskQuoted, '(\\+)$', '$1$1')
    return '"' + $taskQuoted + '"'
}
function Invoke-TaskChild([string]$Executable, [string[]]$Arguments, [switch]$WatchGenerations, $RetainedPublicRelease, [switch]$KeepStandardInputOpen) {
    $taskInfo = [Diagnostics.ProcessStartInfo]::new()
    $taskInfo.FileName=$Executable; $taskInfo.WorkingDirectory=$taskRepository
    $taskInfo.UseShellExecute=$false; $taskInfo.CreateNoWindow=$true
    $taskInfo.RedirectStandardInput=$true; $taskInfo.RedirectStandardOutput=$true; $taskInfo.RedirectStandardError=$true
    $taskInfo.Arguments=($Arguments | ForEach-Object { ConvertTo-TaskArgument $_ }) -join ' '
    $taskInfo.EnvironmentVariables['PSModulePath']=$taskModuleRoot
    $taskProcess=[Diagnostics.Process]::new(); $taskProcess.StartInfo=$taskInfo
    try {
        if (-not $taskProcess.Start()) { throw 'The reviewed startup child did not start.' }
        $taskRetainedChildIdentity=$null
        if ($WatchGenerations) {
            try {
                $taskRetainedChildIdentity=@{pid=$taskProcess.Id;creationTime=[string]$taskProcess.StartTime.ToUniversalTime().ToFileTimeUtc()}
            } catch { } # An early exit without its native birth remains unknown.
        }
        if (-not $KeepStandardInputOpen) { $taskProcess.StandardInput.Close() }
        try { $taskProcess.PriorityClass=[Diagnostics.ProcessPriorityClass]::AboveNormal } catch { Write-Warning 'The owned startup child priority could not be raised.' }
        $taskOut=$taskProcess.StandardOutput.ReadToEndAsync(); $taskErr=$taskProcess.StandardError.ReadToEndAsync()
        if ($WatchGenerations) {
            while (-not $taskProcess.WaitForExit(15000)) { Confirm-TaskGeneration }
            Confirm-TaskGeneration
        } else { $taskProcess.WaitForExit() }
        $taskOutput=$taskOut.GetAwaiter().GetResult(); $taskError=$taskErr.GetAwaiter().GetResult()
        if ($WatchGenerations) {
            # The exact Process created above has reached native terminal state.
            # A nonzero exit never grants permission on its own.
            $taskExitFacts=Get-TaskPublicExitFacts $RetainedPublicRelease $taskProcess $taskRetainedChildIdentity
            if (-not (Test-TaskPublicExitContinuation $taskExitFacts)) {
                throw ('Retained public exit requires reconciliation without replay: '+($taskExitFacts|ConvertTo-Json -Depth 10 -Compress)+'; '+$taskError+' '+$taskOutput)
            }
            [Console]::Error.WriteLine(('NATIVE_PROVEN_PUBLIC_CHILD_ENDED '+($taskExitFacts|ConvertTo-Json -Depth 10 -Compress)))
            if ($taskError) { [Console]::Error.WriteLine($taskError) }
            return $taskOutput
        }
        if ($taskProcess.ExitCode -ne 0) { throw ('Startup child failed with '+$taskProcess.ExitCode+': '+$taskError+' '+$taskOutput) }
        return $taskOutput
    } finally {
        if ($KeepStandardInputOpen) {
            try {
                if ($taskProcess.HasExited) { $taskProcess.StandardInput.Close() }
            } catch { }
        }
        $taskProcess.Dispose()
    }
}
if ((Get-TaskSha $taskStartupPath) -cne $ExpectedStartupSha256) { throw 'The pinned current-boot startup descriptor changed.' }
$taskStartup = Get-Content -LiteralPath $taskStartupPath -Raw | ConvertFrom-Json
if ($taskStartup.kind -cne 'formal-current-boot-startup' -or $taskStartup.schemaVersion -ne 1 -or
    $taskStartup.repository -cne $taskRepository -or $taskStartup.profile -cne 'zircon-strict' -or
    $taskStartup.includeManagedConfig -ne $true -or
    $taskStartup.userSid -cne [Security.Principal.WindowsIdentity]::GetCurrent().User.Value) {
    throw 'The startup descriptor requires its exact repository, current user and strict managed configuration.'
}
foreach ($taskPin in $taskStartup.files.PSObject.Properties) {
    if ($taskPin.Name -ceq 'codex' -and $taskStartup.hostResolution) { continue }
    if ((Get-TaskSha $taskPin.Value.path) -cne $taskPin.Value.sha256) { throw 'A pinned startup component changed.' }
}
if ($taskStartup.files.dispatcher.path -cne $PSCommandPath) { throw 'Use the exact pinned login dispatcher.' }

function Get-TaskBoundedSignature([string]$Path) {
    $taskSignatureScript = '$ErrorActionPreference=''Stop''; $ProgressPreference=''SilentlyContinue''; $s=Get-AuthenticodeSignature -LiteralPath ''' +
        $Path.Replace("'","''") + '''; [pscustomobject]@{path=$s.Path;status=[string]$s.Status;statusMessage=$s.StatusMessage;subject=$s.SignerCertificate.Subject;thumbprint=$s.SignerCertificate.Thumbprint} | ConvertTo-Json -Compress'
    $taskSignatureInfo=[Diagnostics.ProcessStartInfo]::new()
    $taskSignatureInfo.FileName=Join-Path $env:SystemRoot 'System32\WindowsPowerShell\v1.0\powershell.exe'
    $taskSignatureInfo.WorkingDirectory=$taskRepository
    $taskSignatureInfo.UseShellExecute=$false; $taskSignatureInfo.CreateNoWindow=$true
    $taskSignatureInfo.RedirectStandardInput=$true; $taskSignatureInfo.RedirectStandardOutput=$true; $taskSignatureInfo.RedirectStandardError=$true
    $taskSignatureInfo.Arguments='-NoLogo -NoProfile -NonInteractive -ExecutionPolicy Bypass -WindowStyle Hidden -EncodedCommand ' +
        [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($taskSignatureScript))
    $taskSignatureInfo.EnvironmentVariables['PSModulePath']=$taskModuleRoot
    foreach ($taskName in @('TEMP','TMP','TMPDIR')) { $taskSignatureInfo.EnvironmentVariables[$taskName]=Join-Path $taskRepository '.jenkins\tmp' }
    $taskChecker=[Diagnostics.Process]::new(); $taskChecker.StartInfo=$taskSignatureInfo
    try {
        if (-not $taskChecker.Start()) { throw 'The native read-only signature checker did not start.' }
        $taskChecker.StandardInput.Close()
        try { $taskChecker.PriorityClass=[Diagnostics.ProcessPriorityClass]::AboveNormal } catch { }
        $taskOut=$taskChecker.StandardOutput.ReadToEndAsync(); $taskErr=$taskChecker.StandardError.ReadToEndAsync()
        if (-not $taskChecker.WaitForExit(60000)) {
            # Only this retained read-only signature-checker process is ended.
            # It never launches Codex, a keeper, Java or a service action.
            $taskChecker.Kill()
            $taskChecker.WaitForExit()
            $null=$taskOut.GetAwaiter().GetResult(); $taskFailure=$taskErr.GetAwaiter().GetResult()
            [Console]::Error.WriteLine(('NATIVE_SIGNATURE_TIMEOUT '+$Path+'; budgetSeconds=60; checker='+$taskChecker.Id))
            throw 'Native signature verification exceeded its 60-second budget.'
        }
        $taskSignatureOutput=$taskOut.GetAwaiter().GetResult(); $taskSignatureError=$taskErr.GetAwaiter().GetResult()
        if ($taskChecker.ExitCode -ne 0) { throw ('Native signature checker failed: '+$taskSignatureError) }
        $taskSignature=$taskSignatureOutput | ConvertFrom-Json
        [Console]::Out.WriteLine(('NATIVE_SIGNATURE_RESULT '+($taskSignature|ConvertTo-Json -Compress)))
        return $taskSignature
    } finally { $taskChecker.Dispose() }
}
$script:taskConfirmedGenerations=@{}
$script:taskConfirmedPublicGenerations=@{}
$script:taskGenerationObservationAttempts=@{}
function Save-TaskConfirmedGeneration($Observed) {
    # Only called with this process's successful strict observation output.
    if ($Observed.status -cne 'already-running-no-action' -or
        $Observed.strictActualConfigRplusBDenied -ne $true -or
        (Get-TaskSha $Observed.generationProof.path) -cne $Observed.generationProof.sha256) {
        throw 'A fresh strict generation observation is required.'
    }
    $taskProof=Get-Content -LiteralPath $Observed.generationProof.path -Raw | ConvertFrom-Json
    if ($taskProof.kind -cne 'strict-committed-live-generation-observation' -or
        $taskProof.nativeOwnerObserved -ne $true -or $taskProof.strictActualConfigRplusBDenied -ne $true -or
        $taskProof.runtimeOperationId -cne $Observed.runtimeOperationId -or
        $taskProof.runtimeReceipt.sha256 -cne $Observed.runtimeReceipt.sha256 -or
        $taskProof.keeper.pid -ne $Observed.keeper.pid -or
        $taskProof.keeper.creationTime -cne $Observed.keeper.creationTime -or
        $taskProof.inputs.descriptor.path -cne $Observed.descriptor.path -or
        $taskProof.inputs.descriptor.sha256 -cne $Observed.descriptor.sha256 -or
        [string]$taskProof.release.sha256 -cnotmatch '^[0-9a-f]{64}$') {
        throw 'The confirmed generation changed its exact release, receipt or owner.'
    }
    $script:taskConfirmedPublicGenerations[[string]$taskProof.release.sha256]=@{
        release=$taskProof.release;runtimeOperationId=$taskProof.runtimeOperationId;
        runtimeReceipt=$taskProof.runtimeReceipt;keeper=$taskProof.keeper;
        generationProof=$Observed.generationProof;descriptor=$Observed.descriptor;
        strictActualConfigRplusBDenied=$true;nativeOwnerObserved=$true
    }
}
function Get-TaskPublicExitFacts($ExpectedRelease, $Child, $ChildIdentity) {
    $taskConfirmed=$null
    if ($ExpectedRelease -and $ExpectedRelease.sha256) {
        $taskConfirmed=$script:taskConfirmedPublicGenerations[[string]$ExpectedRelease.sha256]
    }
    $taskFacts=@{role='retained-public';resident=[bool]$Resident;expectedRelease=$ExpectedRelease;
        confirmed=$taskConfirmed;currentDescriptorExact=$false;keeperIdentityStatus='unproven';
        child=@{pid=$ChildIdentity.pid;creationTime=[string]$ChildIdentity.creationTime;
                retainedExactProcess=($null -ne $ChildIdentity -and $ChildIdentity.pid -eq $Child.Id);
                nativeExitObserved=[bool]$Child.HasExited;exitCode=$Child.ExitCode}}
    if (-not $taskConfirmed) { return $taskFacts }
    try {
        $taskCurrent=Get-Content -LiteralPath $taskConfirmed.descriptor.path -Raw | ConvertFrom-Json
        $taskFacts.currentDescriptorExact=(
            (Get-TaskSha $taskConfirmed.descriptor.path) -ceq $taskConfirmed.descriptor.sha256 -and
            $taskCurrent.release.path -ceq $ExpectedRelease.path -and
            $taskCurrent.release.sha256 -ceq $ExpectedRelease.sha256 -and
            $taskCurrent.runtime.selected.runtimeOperationId -ceq $taskConfirmed.runtimeOperationId -and
            $taskCurrent.runtimeReceipt.sha256 -ceq $taskConfirmed.runtimeReceipt.sha256 -and
            $taskCurrent.keeper.pid -eq $taskConfirmed.keeper.pid -and
            $taskCurrent.keeper.creationTime -ceq $taskConfirmed.keeper.creationTime)
        $taskCurrentKeeper=$null
        try { $taskCurrentKeeper=[Diagnostics.Process]::GetProcessById([int]$taskConfirmed.keeper.pid) }
        catch [ArgumentException] {
            # This exact PID was native-live during the strict proof and is now absent.
            $taskFacts.keeperIdentityStatus='terminated-exact'
            return $taskFacts
        }
        try {
            if ([string]$taskCurrentKeeper.StartTime.ToUniversalTime().ToFileTimeUtc() -cne [string]$taskConfirmed.keeper.creationTime) {
                $taskFacts.keeperIdentityStatus='pid-reused'
            } elseif ($taskCurrentKeeper.HasExited) {
                $taskFacts.keeperIdentityStatus='terminated-exact'
            } else { $taskFacts.keeperIdentityStatus='alive-exact' }
        } finally { $taskCurrentKeeper.Dispose() }
    } catch {
        $taskFacts.keeperIdentityStatus='unproven'
        $taskFacts.observationError=$_.Exception.Message
    }
    return $taskFacts
}
function Confirm-TaskGeneration {
    # Only a new committed generation triggers one strict observation, never public replay.
    try {
        $taskNativeDescriptor=Get-Content -LiteralPath (Join-Path $taskRepository '.jenkins\state\formal-lifecycle\recovery-keeper.json') -Raw | ConvertFrom-Json
        $taskNativeCommand=Get-Content -LiteralPath (Join-Path $taskRepository '.jenkins\state\recovery-lifecycle\command.json') -Raw | ConvertFrom-Json
        if ($taskNativeCommand.status -cne 'committed' -or $taskNativeDescriptor.runtime.state -cne 'running' -or
            $taskNativeCommand.keeper.pid -ne $taskNativeDescriptor.keeper.pid -or
            $taskNativeCommand.keeper.creationTime -cne $taskNativeDescriptor.keeper.creationTime -or
            $taskNativeCommand.release.sha256 -cne $taskNativeDescriptor.release.sha256) { return }
        $taskGenerationKey=[string]$taskNativeDescriptor.runtime.selected.runtimeOperationId+'|'+[string]$taskNativeDescriptor.keeper.creationTime+'|'+[string]$taskNativeDescriptor.runtimeReceipt.sha256
        if ($script:taskConfirmedGenerations.ContainsKey($taskGenerationKey)) { return }
        $taskNativeKeeper=$null
        try {
            $taskNativeKeeper=[Diagnostics.Process]::GetProcessById([int]$taskNativeDescriptor.keeper.pid)
            if ($taskNativeKeeper.HasExited -or [string]$taskNativeKeeper.StartTime.ToUniversalTime().ToFileTimeUtc() -cne [string]$taskNativeDescriptor.keeper.creationTime) { return }
        } finally { if ($taskNativeKeeper) { $taskNativeKeeper.Dispose() } }
        $taskAttempts=[int]$script:taskGenerationObservationAttempts[$taskGenerationKey]
        if ($taskAttempts -ge 3) { return }
        $script:taskGenerationObservationAttempts[$taskGenerationKey]=$taskAttempts+1
        # This mode can only observe an already-live owner; it cannot prepare or start recovery.
        $taskObservedOutput=Invoke-TaskChild $taskCodex @($taskArguments+'--observe-only')
        $taskObservedLines=@($taskObservedOutput -split '\r?\n' | Where-Object { $_.Trim() })
        if (-not $taskObservedLines.Count) { throw 'The exact-generation strict observation returned no evidence.' }
        $taskObserved=$taskObservedLines[-1] | ConvertFrom-Json
        if ($taskObserved.status -cne 'already-running-no-action' -or $taskObserved.strictActualConfigRplusBDenied -ne $true -or
            $taskObserved.generationProof.sha256 -cnotmatch '^[0-9a-f]{64}$' -or
            $taskObserved.runtimeOperationId -cne $taskNativeDescriptor.runtime.selected.runtimeOperationId -or
            $taskObserved.keeper.creationTime -cne $taskNativeDescriptor.keeper.creationTime -or
            $taskObserved.runtimeReceipt.sha256 -cne $taskNativeDescriptor.runtimeReceipt.sha256) { throw 'The current exact generation was not confirmed.' }
        Save-TaskConfirmedGeneration $taskObserved
        $script:taskConfirmedGenerations[$taskGenerationKey]=$true
        [Console]::Out.WriteLine(('NATIVE_GENERATION_CONFIRMED '+$taskObservedLines[-1]))
    } catch {
        [Console]::Error.WriteLine(('NATIVE_GENERATION_PROOF_PENDING '+$_.Exception.Message+'; public start was not replayed'))
    }
}

function Wait-TaskKeeper($Keeper) {
    if (-not $Keeper -or $Keeper.pid -le 0 -or [string]$Keeper.creationTime -cnotmatch '^[0-9]+$' -or
        $Keeper.executable -cne (Join-Path $taskRepository '.jenkins\runtime\python\python.exe')) {
        throw 'The strict current-owner proof has no exact keeper identity.'
    }
    $taskWatched=$null
    try {
        try { $taskWatched=[Diagnostics.Process]::GetProcessById([int]$Keeper.pid) }
        catch [ArgumentException] { return }
        if ([string]$taskWatched.StartTime.ToUniversalTime().ToFileTimeUtc() -cne [string]$Keeper.creationTime) { return }
        # Retain this exact process handle. A reused PID never extends this wait.
        # No historical closure, HTTP probe, process-name gate or lifecycle action
        # runs while this keeper remains alive.
        while (-not $taskWatched.WaitForExit(15000)) { Confirm-TaskGeneration }
    } finally { if ($taskWatched) { $taskWatched.Dispose() } }
}
while ($true) {
$taskCodex=$null; $taskBundle=$null
$taskPolicy=$taskStartup.hostResolution
if ($taskPolicy) {
    $taskInstalledRoot=[IO.Path]::GetFullPath((Join-Path ([Environment]::GetFolderPath('LocalApplicationData')) 'OpenAI\Codex\bin'))
    if ($taskPolicy.installedRoot -cne $taskInstalledRoot -or $taskPolicy.signerSubject -cne 'CN="OpenAI OpCo, LLC", O="OpenAI OpCo, LLC", L=San Francisco, S=California, C=US' -or
        ($taskPolicy.requiredFiles -join '|') -cne 'codex.exe|codex-command-runner.exe|codex-windows-sandbox-setup.exe|codex-code-mode-host.exe') { throw 'The official installed host policy changed.' }
    foreach ($taskCandidate in @(Get-ChildItem -LiteralPath $taskInstalledRoot -Directory | Sort-Object LastWriteTimeUtc,Name -Descending)) {
        if ($taskCandidate.Name -cnotmatch '^[0-9a-f]{16}$' -or ($taskCandidate.Attributes -band [IO.FileAttributes]::ReparsePoint)) { continue }
        $taskFiles=[ordered]@{}; $taskSigner=$null; $taskComplete=$true
        # Reject incomplete old installations before reading any large signature.
        foreach ($taskName in $taskPolicy.requiredFiles) {
            $taskPath=Join-Path $taskCandidate.FullName $taskName
            if (-not (Test-Path -LiteralPath $taskPath -PathType Leaf) -or ((Get-Item -LiteralPath $taskPath).Attributes -band [IO.FileAttributes]::ReparsePoint)) { $taskComplete=$false; break }
        }
        if (-not $taskComplete) { continue }
        foreach ($taskName in @('codex-command-runner.exe','codex-windows-sandbox-setup.exe','codex-code-mode-host.exe','codex.exe')) {
            $taskPath=Join-Path $taskCandidate.FullName $taskName
            try { $taskSignature=Get-TaskBoundedSignature $taskPath }
            catch { [Console]::Error.WriteLine(('NATIVE_SIGNATURE_REJECTED '+$taskPath+': '+$_.Exception.Message)); $taskComplete=$false; break }
            if ($taskSignature.status -cne 'Valid' -or $taskSignature.subject -cne $taskPolicy.signerSubject) { $taskComplete=$false; break }
            if ($taskSigner -and $taskSigner -cne $taskSignature.thumbprint) { $taskComplete=$false; break }
            $taskSigner=$taskSignature.thumbprint
            $taskFiles[$taskName]=@{path=$taskPath;sha256=(Get-TaskSha $taskPath)}
        }
        if ($taskComplete) { $taskCodex=$taskFiles['codex.exe'].path; $taskBundle=$taskFiles; break }
    }
    if (-not $taskCodex) { throw 'No complete OpenAI-signed installed Codex host bundle is available; service was not started.' }
} else { $taskCodex=$taskStartup.files.codex.path }
$taskPython=$taskStartup.files.preparationPython.path
$taskArguments=@('sandbox','-P','zircon-strict','--include-managed-config','-C',$taskRepository,'--',$taskPython,'-B','-X','utf8',$taskStartup.files.preparer.path,'--startup-file',$taskStartupPath,'--startup-sha256',$ExpectedStartupSha256)
if ($taskBundle) {
    $taskArguments+=@('--codex-executable',$taskCodex,'--codex-sha256',$taskBundle['codex.exe'].sha256,'--runner-sha256',$taskBundle['codex-command-runner.exe'].sha256,'--setup-sha256',$taskBundle['codex-windows-sandbox-setup.exe'].sha256,'--code-mode-host-sha256',$taskBundle['codex-code-mode-host.exe'].sha256)
}
if ($Resident) {
    $taskArguments+=@('--observer-pid',[string]$PID,'--observer-birth',[string][Diagnostics.Process]::GetCurrentProcess().StartTime.ToUniversalTime().ToFileTimeUtc())
}
if ($CheckOnly) { $taskArguments+='--check-only' }
$taskOutput=Invoke-TaskChild $taskCodex $taskArguments -KeepStandardInputOpen
$taskLines=@($taskOutput -split '\r?\n' | Where-Object { $_.Trim() })
if (-not $taskLines.Count) { throw 'Current-boot recovery preflight returned no evidence.' }
$taskResult=$taskLines[-1] | ConvertFrom-Json
Write-Output $taskOutput
if ($CheckOnly -or $PrepareOnly) { exit 0 }
if ($taskResult.status -ceq 'already-running-no-action') {
    if (-not $Resident) { exit 0 }
    if ($taskResult.generationProof.sha256 -cnotmatch '^[0-9a-f]{64}$') { throw 'A resident owner needs its strict committed-generation proof.' }
    $taskGenerationKey=[string]$taskResult.runtimeOperationId+'|'+[string]$taskResult.keeper.creationTime+'|'+[string]$taskResult.runtimeReceipt.sha256
    Save-TaskConfirmedGeneration $taskResult
    $script:taskConfirmedGenerations[$taskGenerationKey]=$true
    Wait-TaskKeeper $taskResult.keeper
    continue
}
if ($taskResult.status -cne 'prepared-not-started' -or $taskResult.strictActualConfigRplusBDenied -ne $true -or
    $taskResult.startup.path -cne $taskStartupPath -or $taskResult.startup.sha256 -cne $ExpectedStartupSha256 -or
    (Get-TaskSha $taskResult.prepared.path) -cne $taskResult.prepared.sha256 -or
    (Get-TaskSha $taskResult.release.path) -cne $taskResult.release.sha256) { throw 'The boot preparer did not publish its exact reviewed strict release.' }
foreach ($taskPin in $taskStartup.files.PSObject.Properties) {
    if ($taskPin.Name -ceq 'codex' -and $taskPolicy) { continue }
    if ((Get-TaskSha $taskPin.Value.path) -cne $taskPin.Value.sha256) { throw 'A startup pin changed before public start.' }
}
if ($taskBundle) { foreach ($taskPin in $taskBundle.Values) { if ((Get-TaskSha $taskPin.path) -cne $taskPin.sha256) { throw 'The complete signed official host bundle changed before public start.' } } }
$taskIntentPath=[IO.Path]::GetFullPath([string]$taskResult.launchIntent)
if ([string]$taskResult.requestId -cnotmatch '^[0-9a-f]{32}$') { throw 'A distinct exact-predecessor recovery request is required.' }
$taskExpectedDirectory=Join-Path (Join-Path (Join-Path $taskStartup.reportRoot ('boot-'+$taskResult.bootFileTime)) ('startup-'+$ExpectedStartupSha256.Substring(0,16))) ('request-'+$taskResult.requestId)
if ([IO.Path]::GetDirectoryName($taskIntentPath) -cne $taskExpectedDirectory -or [IO.Path]::GetFileName($taskIntentPath) -cne 'launch-intent.json') { throw 'The public start intent escaped its registered current-boot directory.' }
if (Test-Path -LiteralPath $taskIntentPath) { throw 'A public start was already issued for this boot and startup; reconcile its exact original attempt before another start.' }
$taskIntent=[ordered]@{schemaVersion=1;kind='current-boot-public-start-intent';createdAtUtc=[DateTime]::UtcNow.ToString('o');bootFileTime=$taskResult.bootFileTime;requestId=$taskResult.requestId;predecessorGenerationProof=$taskResult.predecessorGenerationProof;startup=$taskResult.startup;prepared=$taskResult.prepared;release=$taskResult.release;userSid=$taskStartup.userSid;publicStarter=$taskStartup.files.publicStarter;maximumPublicStartCalls=1;automaticRetry=$false;unknownRequiresReconciliation=$true;serviceLifecycleAcceptance=$false}
$taskRaw=[Text.UTF8Encoding]::new($false).GetBytes(($taskIntent | ConvertTo-Json -Depth 12)+[Environment]::NewLine)
$taskIntentStream=[IO.File]::Open($taskIntentPath,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
try { $taskIntentStream.Write($taskRaw,0,$taskRaw.Length); $taskIntentStream.Flush($true) } finally { $taskIntentStream.Dispose() }
# The pinned public entry owns the retained official strict host and desktop tray.
$taskPowerShell=Join-Path $env:SystemRoot 'System32\WindowsPowerShell\v1.0\powershell.exe'
$taskPublicArgs=@('-NoLogo','-NoProfile','-NonInteractive','-ExecutionPolicy','Bypass','-WindowStyle','Hidden','-File',$taskStartup.files.publicStarter.path,'-RecoveryReleaseFile',$taskResult.release.path,'-CodexExecutable',$taskCodex)
Write-Output (Invoke-TaskChild $taskPowerShell $taskPublicArgs -WatchGenerations:$Resident -RetainedPublicRelease $taskResult.release)
if (-not $Resident) { exit 0 }
}
