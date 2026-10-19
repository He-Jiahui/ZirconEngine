[CmdletBinding()]
param(
    [string]$RepoRoot = 'E:/Git/ZirconEngine',
    [Parameter(Mandatory=$true)][ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedDriverDigest,
    [Parameter(Mandatory=$true)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9_.-]{0,127}$')][string]$ExpectedRuntimeOperationId,
    [ValidatePattern('^[A-Za-z0-9][A-Za-z0-9_.-]{0,127}$')][string]$ExpectedPreviousRuntimeOperationId,
    [ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedPreviousDriverDigest,
    [string]$BaseUrl = 'http://127.0.0.1:18080',
    [string]$CodexExe = 'C:/Users/HeJiahui/AppData/Local/OpenAI/Codex/bin/9614f1d9b40d9c4f/codex.exe',
    [string]$ExpectedPreviousCodexExe,
    [ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedPreviousJobConfigSha256,
    [switch]$CheckOnly,
    [switch]$EnableCheckoutProduct,
    [switch]$EnableManagedWorktree,
    [string]$CandidateSourceRoot,
    [string]$CandidateSourceManifestPath,
    [ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedCandidateSourceManifestSha256,
    [string[]]$ExpectedCandidateCapsulePaths = @(),
    [switch]$AnchorManagementClients,
    [switch]$MigrateCanonicalManagementClients,
    [string]$CanonicalClientSourceManifestPath,
    [ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedCanonicalClientSourceManifestSha256,
    [ValidatePattern('^[0-9a-f]{64}$')][string]$ReuseCapsuleSha256,
    [ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedInstalledSha256,
    [switch]$ResumePreparedInstallation,
    [ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedEntrySourceSha256,
    [ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedLocalCargoSourceSha256,
    [ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedFrontendSourceSha256,
    [ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedSubmitSourceSha256,
    [ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedCapsuleSha256,
    [string]$PinnedActiveBuildProofPath,
    [ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedPinnedActiveBuildProofSha256,
    [ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedPinnedActiveGuardSha256,
    [ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedCheckoutCommitAddonSha256,
    [ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedDeliveryAddonSha256,
    [ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedBranchIntegrationAddonSha256,
    [ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedOldBranchCleanupAddonSha256,
    [ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedRetainedPrefixSha256
)
$ErrorActionPreference = 'Stop'
$script:taskInstallerParameters = $PSBoundParameters
function Assert-Physical([string]$Path) {
    $absolute = [IO.Path]::GetFullPath($Path)
    $cursor = $absolute
    while ($cursor) {
        if (Test-Path -LiteralPath $cursor) {
            $item = Get-Item -LiteralPath $cursor -Force
            if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "Path alias: $cursor" }
        }
        $parent = [IO.Directory]::GetParent($cursor)
        $cursor = if ($parent) { $parent.FullName } else { $null }
    }
    return $absolute
}
function Hash-Bytes([byte[]]$Bytes) {
    $digest = [Security.Cryptography.SHA256]::Create()
    try { return ([BitConverter]::ToString($digest.ComputeHash($Bytes))).Replace('-','').ToLowerInvariant() }
    finally { $digest.Dispose() }
}
function Get-ValidatedManagementClientSha256 {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory=$true)][string]$ClientName,
        [Parameter(Mandatory=$true)][AllowEmptyCollection()][byte[]]$ClientBytes,
        [AllowNull()][AllowEmptyString()][string]$OriginalSha256,
        [AllowNull()][AllowEmptyString()][string]$ExpectedEntrySourceSha256,
        [AllowNull()][AllowEmptyString()][string]$ExpectedLocalCargoSourceSha256,
        [AllowNull()][AllowEmptyString()][string]$ExpectedFrontendSourceSha256,
        [AllowNull()][AllowEmptyString()][string]$ExpectedSubmitSourceSha256
    )
    $taskClientHash = Hash-Bytes $ClientBytes
    $taskReview = switch -CaseSensitive -Exact ($ClientName) {
        'tools/jenkins-management/entry.py' {
            @{Parameter='ExpectedEntrySourceSha256';Error='Reviewed public entry pin differs'}
        }
        'tools/dev/local_cargo.py' {
            @{Parameter='ExpectedLocalCargoSourceSha256';Error='Reviewed native helper pin differs'}
        }
        'tools/jenkins/frontend.py' {
            @{Parameter='ExpectedFrontendSourceSha256';Error='Reviewed frontend pin differs'}
        }
        'tools/dev/submit-jenkins-management.ps1' {
            @{Parameter='ExpectedSubmitSourceSha256';Error='ExpectedSubmitSourceSha256 does not match current raw bytes'}
        }
    }
    $taskHasReviewedPin = $null -ne $taskReview -and $PSBoundParameters.ContainsKey($taskReview.Parameter)
    if ($taskHasReviewedPin) {
        $taskExpectedHash = [string]$PSBoundParameters[$taskReview.Parameter]
        if ($taskExpectedHash -notmatch '^[0-9a-f]{64}$' -or $taskExpectedHash -ne $taskClientHash) {
            throw $taskReview.Error
        }
    }
    if ([string]::IsNullOrEmpty($OriginalSha256) -or $taskClientHash -eq $OriginalSha256 -or $taskHasReviewedPin) {
        return $taskClientHash
    }
    if ($ClientName -ceq 'tools/jenkins-management/entry.py') {
        $taskNormalized = [Text.Encoding]::UTF8.GetString($ClientBytes).Replace(([string][char]13 + [char]10), [string][char]10)
        if ((Hash-Bytes ([Text.Encoding]::UTF8.GetBytes($taskNormalized))) -eq $OriginalSha256) {
            return $taskClientHash
        }
    }
    throw 'Unreviewed public entry change'
}
function Get-ManagedWorktreeCapability {
    return [ordered]@{enabled=$true;schemaVersion=1;
        actions=@('apply-patch','commit-worktree','create-branch','create-worktree','generate','inspect-worktree','prepare-flow',
            'prepare-merge','prepare-worktree','recover-worktree','release-worktree','retire-worktree-reader','submit-flow','validate');
        sourcePreparation='managed-worktree-source-preparation-v1';
        compilerInputs='managed-worktree-product-inputs-v1';formalAcceptance=$false}
}
function Assert-ManagedWorktreeDriver([string]$Repo, [string]$Driver) {
    $taskDriverRoot = Assert-Physical "$Repo/.jenkins/runtime/drivers/$Driver"
    $taskDriverManifest = Get-Content -Raw -LiteralPath (Assert-Physical "$taskDriverRoot/driver-manifest.json") | ConvertFrom-Json
    if ($taskDriverManifest.driverDigest -cne $Driver) { throw 'Managed worktree sealed driver identity differs' }
    foreach ($taskModule in @('tools/jenkins/managed_worktree.py','tools/jenkins/managed_worktree_lifetime.py','tools/jenkins/product_inputs.py')) {
        $taskFact = $taskDriverManifest.files.PSObject.Properties[$taskModule]
        if ($null -eq $taskFact -or $taskFact.Value.sha256 -notmatch '^[0-9a-f]{64}$') { throw 'Managed worktree requires the sealed module inventory' }
        $taskData = [IO.File]::ReadAllBytes((Assert-Physical "$taskDriverRoot/$taskModule"))
        if ($taskData.Length -ne $taskFact.Value.bytes -or (Hash-Bytes $taskData) -cne $taskFact.Value.sha256) {
            throw 'Managed worktree sealed module changed'
        }
    }
}
function Resolve-PreviousCodexBinding([string]$Current, [string]$Previous, [string]$ConfigSha256, [bool]$HasPrevious) {
    $taskHasPreviousCodex = -not [string]::IsNullOrEmpty($Previous)
    $taskHasPreviousConfig = -not [string]::IsNullOrEmpty($ConfigSha256)
    if ($taskHasPreviousCodex -ne $taskHasPreviousConfig) { throw 'Previous Codex and raw job config pins must be paired' }
    if (-not $taskHasPreviousCodex) { return $Current }
    if (-not $HasPrevious) { throw 'Previous host pins require an existing installation' }
    if ($Previous -notmatch '^[A-Za-z]:[\\/]' -or $Previous -match '(?:^|[\\/])\.{1,2}(?:[\\/]|$)') {
        throw 'Previous Codex must be an absolute physical path'
    }
    if ($Previous.Contains([char]10) -or $Previous.Contains([char]13) -or $Previous.Contains('"')) {
        throw 'Unsafe previous Codex path'
    }
    # A removed historical executable is evidence, never a launch candidate.
    # Existing ancestors still pass the ordinary physical-path alias checks.
    return (Assert-Physical $Previous)
}
function Assert-PreviousManagementConfig([byte[]]$Bytes, [string]$ExpectedSha256) {
    if (-not [string]::IsNullOrEmpty($ExpectedSha256) -and (Hash-Bytes $Bytes) -cne $ExpectedSha256) {
        throw 'Original management config raw bytes changed'
    }
}
function Assert-ManagementJob([xml]$Job, [string]$Pipeline, [string]$Driver, [string]$Runtime) {
    $taskRoot = $Job.DocumentElement
    if ($taskRoot.LocalName -cne 'flow-definition') { throw 'Management job type differs' }
    if ($taskRoot.HasAttribute('plugin') -and $taskRoot.GetAttribute('plugin') -cnotmatch '^workflow-job(?:@[A-Za-z0-9_.-]+)?$') { throw 'Management job plugin differs' }
    $taskKeep = $taskRoot.SelectSingleNode('keepDependencies')
    if ($null -ne $taskKeep -and ($taskKeep.InnerText -cne 'false' -or $taskKeep.Attributes.Count -ne 0 -or @($taskKeep.ChildNodes | Where-Object { $_.NodeType -eq [Xml.XmlNodeType]::Element }).Count)) { throw 'Management job dependency retention differs' }
    $taskDefinition = $taskRoot.SelectSingleNode('definition')
    if ($null -eq $taskDefinition -or $taskDefinition.GetAttribute('plugin') -cnotmatch '^workflow-cps(?:@[A-Za-z0-9_.-]+)?$' -or
            (@($taskDefinition.ChildNodes | Where-Object { $_.NodeType -eq [Xml.XmlNodeType]::Element } | ForEach-Object { $_.LocalName } | Sort-Object) -join '|') -cne 'sandbox|script') { throw 'Management pipeline definition differs' }
    $taskElements = @($taskRoot.ChildNodes | Where-Object { $_.NodeType -eq [Xml.XmlNodeType]::Element })
    $taskAllowed = @('actions','description','keepDependencies','properties','definition','triggers','disabled')
    if (@($taskElements | Where-Object { $_.LocalName -cnotin $taskAllowed }).Count -or
            @($taskElements | Group-Object LocalName | Where-Object { $_.Count -ne 1 }).Count) { throw 'Management job contains unreviewed configuration' }
    if ($taskRoot.description -cne 'Bounded Jenkins management MVP; no M8 acceptance' -or
            $taskRoot.disabled -cne 'false' -or $taskRoot.definition.class -cne 'org.jenkinsci.plugins.workflow.cps.CpsFlowDefinition' -or
            $taskRoot.definition.script -cne $Pipeline -or $taskRoot.definition.sandbox -cne 'true') { throw 'Management job binding differs' }
    if (@($taskRoot.triggers.ChildNodes | Where-Object { $_.NodeType -eq [Xml.XmlNodeType]::Element }).Count -or
            @($taskRoot.actions.ChildNodes | Where-Object { $_.NodeType -eq [Xml.XmlNodeType]::Element }).Count) { throw 'Management job has unreviewed triggers or actions' }
    if ($taskRoot.SelectSingleNode('properties').Attributes.Count -ne 0) { throw 'Management property container attributes differ' }
    $taskProperties = @($taskRoot.properties.ChildNodes | Where-Object { $_.NodeType -eq [Xml.XmlNodeType]::Element })
    if ($taskProperties.Count -ne 1 -or $taskProperties[0].LocalName -cne 'hudson.model.ParametersDefinitionProperty') { throw 'Management job properties differ' }
    $taskPropertyChildren = @($taskProperties[0].ChildNodes | Where-Object { $_.NodeType -eq [Xml.XmlNodeType]::Element })
    if ($taskPropertyChildren.Count -ne 1 -or $taskPropertyChildren[0].LocalName -cne 'parameterDefinitions' -or $taskProperties[0].Attributes.Count -ne 0) { throw 'Management parameter property contains unreviewed settings' }
    if ($taskPropertyChildren[0].Attributes.Count -ne 0) { throw 'Management parameter container attributes differ' }
    $taskParameters = @($taskProperties[0].parameterDefinitions.ChildNodes | Where-Object { $_.NodeType -eq [Xml.XmlNodeType]::Element })
    $taskNames = @('SESSION_ID','REQUEST_ID','OPERATION_ID','ACTION','INPUT_JSON','EXPECTED_DRIVER_DIGEST','RUNTIME_OPERATION_ID','QUERY_ONLY')
    if ($taskParameters.Count -ne $taskNames.Count -or
            (@($taskParameters | ForEach-Object { [string]$_.name } | Sort-Object) -join '|') -cne
            (@($taskNames | Sort-Object) -join '|')) { throw 'Management job parameter inventory differs' }
    foreach ($taskParameter in $taskParameters) {
        $taskName = [string]$taskParameter.name
        $taskFields = @($taskParameter.ChildNodes | Where-Object { $_.NodeType -eq [Xml.XmlNodeType]::Element })
        $taskExpectedFields = if ($taskName -ceq 'QUERY_ONLY') { @('name','defaultValue') } else { @('name','defaultValue','trim') }
        $taskDescription = $taskParameter.SelectSingleNode('description')
        if ($null -ne $taskDescription) { $taskExpectedFields += 'description'; if ($taskDescription.InnerText -cne '') { throw 'Management parameter description differs' } }
        if ($taskParameter.Attributes.Count -ne 0 -or
                (@($taskFields | ForEach-Object { $_.LocalName } | Sort-Object) -join '|') -cne (@($taskExpectedFields | Sort-Object) -join '|') -or
                @($taskFields | Where-Object { $_.Attributes.Count -ne 0 -or @($_.ChildNodes | Where-Object { $_.NodeType -eq [Xml.XmlNodeType]::Element }).Count }).Count) { throw 'Management parameter contains unreviewed nested settings' }
        if ($taskName -ceq 'QUERY_ONLY') {
            if ($taskParameter.LocalName -cne 'hudson.model.BooleanParameterDefinition' -or $taskParameter.defaultValue -cne 'false') { throw 'Management query parameter differs' }
        } else {
            $taskDefault = if ($taskName -ceq 'EXPECTED_DRIVER_DIGEST') { $Driver } elseif ($taskName -ceq 'RUNTIME_OPERATION_ID') { $Runtime } else { '' }
            if ($taskParameter.LocalName -cne 'hudson.model.StringParameterDefinition' -or
                    [string]$taskParameter.defaultValue -cne $taskDefault -or $taskParameter.trim -cne 'true') { throw 'Management parameter binding differs' }
        }
    }
}
function Get-ManagementJobExtensionPrefix {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory=$true)][AllowEmptyCollection()][byte[]]$AddonBytes,
        [Parameter(Mandatory=$true)][AllowEmptyString()][string]$ExpectedSha256,
        [ValidateSet('checkout-commit','delivery','branch-integration','branch-cleanup')][string]$Kind = 'checkout-commit'
    )
    $taskAddonLabel = if ($Kind -ceq 'branch-integration') { 'Branch integration addon' } elseif ($Kind -ceq 'branch-cleanup') { 'Old branch cleanup addon' } elseif ($Kind -ceq 'delivery') { 'Validated delivery addon' } else { 'Checkout commit addon' }
    if ($ExpectedSha256 -cnotmatch '^[0-9a-f]{64}$' -or (Hash-Bytes $AddonBytes) -cne $ExpectedSha256) {
        throw ($taskAddonLabel + ' source differs')
    }
    $taskAddonText = [Text.UTF8Encoding]::new($false,$true).GetString($AddonBytes)
    if ($taskAddonText.StartsWith([string][char]0xfeff, [StringComparison]::Ordinal)) { $taskAddonText = $taskAddonText.Substring(1) }
    $taskAddonText = $taskAddonText.Replace("`r`n", "`n").TrimEnd([char[]]"`r`n")
    $taskBegin = if ($Kind -ceq 'branch-integration') { '// BEGIN ZIRCON BRANCH QUEUE' } elseif ($Kind -ceq 'branch-cleanup') { '// BEGIN ZIRCON OLD BRANCH CLEANUP' } elseif ($Kind -ceq 'delivery') { '// BEGIN ZIRCON VALIDATED DELIVERY' } else { '// BEGIN ZIRCON SCOPED CHECKOUT COMMIT' }
    $taskEnd = if ($Kind -ceq 'branch-integration') { '// END ZIRCON BRANCH QUEUE' } elseif ($Kind -ceq 'branch-cleanup') { '// END ZIRCON OLD BRANCH CLEANUP' } elseif ($Kind -ceq 'delivery') { '// END ZIRCON VALIDATED DELIVERY' } else { '// END ZIRCON SCOPED CHECKOUT COMMIT' }
    # Use literal ordinal offsets for both unique markers and their exact boundaries.
    $taskBeginOffset = $taskAddonText.IndexOf($taskBegin, [StringComparison]::Ordinal)
    $taskLastBeginOffset = $taskAddonText.LastIndexOf($taskBegin, [StringComparison]::Ordinal)
    $taskEndOffset = $taskAddonText.Length - $taskEnd.Length
    $taskFirstEndOffset = $taskAddonText.IndexOf($taskEnd, [StringComparison]::Ordinal)
    $taskLastEndOffset = $taskAddonText.LastIndexOf($taskEnd, [StringComparison]::Ordinal)
    if ($taskBeginOffset -ne 0 -or $taskLastBeginOffset -ne 0 -or
            $taskEndOffset -lt $taskBegin.Length -or
            $taskFirstEndOffset -ne $taskEndOffset -or $taskLastEndOffset -ne $taskEndOffset) {
        throw (($taskAddonLabel + ' must be a unique complete prefix (length={0};begin={1};lastBegin={2};end={3};lastEnd={4};expectedEnd={5})') -f
            $taskAddonText.Length, $taskBeginOffset, $taskLastBeginOffset, $taskFirstEndOffset, $taskLastEndOffset, $taskEndOffset)
    }
    return ($taskAddonText + "`n`n")
}
function Join-ManagementJobExtension {
    [CmdletBinding()]
    param([Parameter(Mandatory=$true)][AllowEmptyString()][string]$Core, [AllowEmptyString()][string]$Prefix)
    if ($Core.Contains('// BEGIN ZIRCON SCOPED CHECKOUT COMMIT') -or
            $Core.Contains('// END ZIRCON SCOPED CHECKOUT COMMIT')) {
        throw 'Management core already contains a checkout commit addon'
    }
    if ($Core.Contains('// BEGIN ZIRCON VALIDATED DELIVERY') -or
            $Core.Contains('// END ZIRCON VALIDATED DELIVERY')) {
        throw 'Management core already contains a validated delivery addon'
    }
    if ($Core.Contains('// BEGIN ZIRCON BRANCH QUEUE') -or
            $Core.Contains('// END ZIRCON BRANCH QUEUE')) {
        throw 'Management core already contains a branch integration addon'
    }
    if ($Core.Contains('// BEGIN ZIRCON OLD BRANCH CLEANUP') -or
            $Core.Contains('// END ZIRCON OLD BRANCH CLEANUP')) {
        throw 'Management core already contains an old branch cleanup addon'
    }
    return ($Prefix + $Core)
}
function Assert-ManagementJobExtensionSource([object]$Extension, [string]$Prefix, [string]$Kind = 'checkout-commit') {
    if ($null -eq $Extension) { return }
    $taskExtensionNow = [IO.File]::ReadAllBytes((Assert-Physical $Extension.sourcePath))
    if ((Get-ManagementJobExtensionPrefix -AddonBytes $taskExtensionNow -ExpectedSha256 $Extension.sourceSha256 -Kind $Kind) -cne $Prefix) {
        throw ($Kind + ' addon changed during installation')
    }
}
function Get-ManagementJobRetainedPrefix {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory=$true)][AllowEmptyString()][string]$ApiScript,
        [Parameter(Mandatory=$true)][AllowEmptyString()][string]$HomeScript,
        [Parameter(Mandatory=$true)][string]$BareCore,
        [Parameter(Mandatory=$true)][ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedSha256
    )
    if ($ApiScript -cne $HomeScript) { throw 'Management API and Home scripts differ' }
    $taskCoreOffset = $ApiScript.Length - $BareCore.Length
    if ($taskCoreOffset -lt 0 -or -not $ApiScript.EndsWith($BareCore, [StringComparison]::Ordinal) -or
            $ApiScript.IndexOf($BareCore, [StringComparison]::Ordinal) -ne $taskCoreOffset -or
            $ApiScript.LastIndexOf($BareCore, [StringComparison]::Ordinal) -ne $taskCoreOffset) {
        throw 'Management script must end in its unique pinned capsule core'
    }
    $taskPrefix = $ApiScript.Substring(0, $taskCoreOffset)
    if ((Hash-Bytes ([Text.Encoding]::UTF8.GetBytes($taskPrefix))) -cne $ExpectedSha256) {
        throw 'Observed management prefix differs from its reviewed hash'
    }
    return $taskPrefix
}
function Set-ManagementJobPipeline {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory=$true)][string]$XmlText,
        [Parameter(Mandatory=$true)][AllowEmptyString()][string]$Pipeline,
        [string]$PreviousRuntime,
        [string]$Runtime,
        [string]$PreviousDriver,
        [string]$Driver
    )
    $taskOpen = '<script>'
    $taskClose = '</script>'
    $taskStart = $XmlText.IndexOf($taskOpen, [StringComparison]::Ordinal)
    $taskEnd = $XmlText.IndexOf($taskClose, [StringComparison]::Ordinal)
    if ($taskStart -lt 0 -or $taskEnd -lt $taskStart + $taskOpen.Length -or
            $XmlText.LastIndexOf($taskOpen, [StringComparison]::Ordinal) -ne $taskStart -or
            $XmlText.LastIndexOf($taskClose, [StringComparison]::Ordinal) -ne $taskEnd) {
        throw 'Management configuration needs its one literal script element'
    }
    $taskEscaped = [Security.SecurityElement]::Escape($Pipeline).Replace("`r", '&#xd;')
    $taskXml = $XmlText.Substring(0, $taskStart + $taskOpen.Length) + $taskEscaped + $XmlText.Substring($taskEnd)
    if ($PreviousRuntime -cne $Runtime) {
        # Change only this validated parameter's value; retain every other raw byte.
        $taskParameterTag = 'hudson.model.StringParameterDefinition'
        $taskParameterContent = '(?:(?!</' + [regex]::Escape($taskParameterTag) + '>).)*?'
        $taskRuntimePattern = '(?s)<' + [regex]::Escape($taskParameterTag) + '>' +
            $taskParameterContent + '<name>RUNTIME_OPERATION_ID</name>' +
            $taskParameterContent + '<defaultValue>(?<runtime>[^<]*)</defaultValue>' +
            $taskParameterContent + '</' + [regex]::Escape($taskParameterTag) + '>'
        $taskRuntimeMatches = [regex]::Matches($taskXml, $taskRuntimePattern)
        if ($taskRuntimeMatches.Count -ne 1 -or
                $taskRuntimeMatches[0].Groups['runtime'].Value -cne $PreviousRuntime) {
            throw 'Retained management runtime default differs from its exact original'
        }
        $taskRuntimeValue = $taskRuntimeMatches[0].Groups['runtime']
        $taskXml = $taskXml.Substring(0, $taskRuntimeValue.Index) + $Runtime +
            $taskXml.Substring($taskRuntimeValue.Index + $taskRuntimeValue.Length)
    }
    if ($PreviousDriver -cne $Driver) {
        $taskParameterTag = 'hudson.model.StringParameterDefinition'
        $taskParameterContent = '(?:(?!</' + [regex]::Escape($taskParameterTag) + '>).)*?'
        $taskDriverPattern = '(?s)<' + [regex]::Escape($taskParameterTag) + '>' +
            $taskParameterContent + '<name>EXPECTED_DRIVER_DIGEST</name>' +
            $taskParameterContent + '<defaultValue>(?<driver>[^<]*)</defaultValue>' +
            $taskParameterContent + '</' + [regex]::Escape($taskParameterTag) + '>'
        $taskDriverMatches = [regex]::Matches($taskXml, $taskDriverPattern)
        if ($taskDriverMatches.Count -ne 1 -or
                $taskDriverMatches[0].Groups['driver'].Value -cne $PreviousDriver) {
            throw 'Retained management driver default differs from its exact original'
        }
        $taskDriverValue = $taskDriverMatches[0].Groups['driver']
        $taskXml = $taskXml.Substring(0, $taskDriverValue.Index) + $Driver +
            $taskXml.Substring($taskDriverValue.Index + $taskDriverValue.Length)
    }
    [xml]$taskParsed = ($taskXml -replace '^\s*<\?xml[^?]*\?>', '')
    if ([string]$taskParsed.'flow-definition'.definition.script -cne $Pipeline) {
        throw 'Management script did not survive exact XML encoding'
    }
    return $taskXml
}
function Invoke-PinnedActiveManagementGuard([string]$Phase) {
    $taskGuardPath = Assert-Physical "$repo/tools/jenkins-management/install_guard.py"
    if ((Hash-Bytes ([IO.File]::ReadAllBytes($taskGuardPath))) -cne $ExpectedPinnedActiveGuardSha256) {
        throw 'Pinned-active operator source changed'
    }
    $taskGuardRequest = [ordered]@{
        repositoryRoot=$repo;proofPath=(Assert-Physical $PinnedActiveBuildProofPath);
        proofSha256=$ExpectedPinnedActiveBuildProofSha256;
        expectedInstalledSha256=$ExpectedInstalledSha256;
        expectedCandidateCapsuleSha256=$digest;
        expectedPreviousJobConfigSha256=$ExpectedPreviousJobConfigSha256;
        expectedDriverDigest=$ExpectedDriverDigest;expectedRuntimeOperationId=$ExpectedRuntimeOperationId;
        previousCodexExe=$taskPreviousCodex;codexExe=$codex;phase=$Phase
    }
    $taskGuardOutput = @((ConvertTo-Json -InputObject $taskGuardRequest -Depth 8 -Compress) |
        & $python -B -X utf8 $taskGuardPath)
    $taskGuardExit = $LASTEXITCODE
    try { $taskGuardReceipt = ($taskGuardOutput -join [Environment]::NewLine) | ConvertFrom-Json }
    catch { throw 'Pinned-active operator returned no exact JSON observation' }
    if ($taskGuardExit -ne 0 -or $taskGuardReceipt.status -cne 'pinned-active-compatible' -or
            $taskGuardReceipt.proofSha256 -cne $ExpectedPinnedActiveBuildProofSha256 -or
            $taskGuardReceipt.candidateCapsuleSha256 -cne $digest -or $taskGuardReceipt.phase -cne $Phase) {
        throw 'Pinned-active workload proof failed; retain the original installation and reconcile'
    }
    $script:taskPinnedGuardReceipts += $taskGuardReceipt
}

function Get-ManagementJobExtensionState {
    param(
        [Parameter(Mandatory=$true)][System.Collections.IDictionary]$AddonPins,
        [AllowNull()][AllowEmptyString()][string]$ExpectedCombinedSha256
    )
    $taskSpecs = [ordered]@{
        ExpectedBranchIntegrationAddonSha256=@{slot='branchIntegration';file='branch-integration.groovy';kind='branch-integration';contract='branch-integration-v1';label='Branch integration'};
        ExpectedOldBranchCleanupAddonSha256=@{slot='branchCleanup';file='branch_cleanup.groovy';kind='branch-cleanup';contract='old-branch-cleanup-v1';label='Old branch cleanup'};
        ExpectedDeliveryAddonSha256=@{slot='delivery';file='delivery.groovy';kind='delivery';contract='validated-delivery-v1';label='Validated delivery'};
        ExpectedCheckoutCommitAddonSha256=@{slot='checkoutCommit';file='checkout-commit.groovy';kind='checkout-commit';contract='scoped-checkout-commit-v1';label='Checkout commit'}
    }
    foreach ($taskName in $AddonPins.Keys) {
        if (-not $taskSpecs.Contains($taskName)) { throw 'Unknown management addon pin' }
    }
    $taskState = [ordered]@{combinedPrefix='';combinedSha256=''}
    foreach ($taskName in $taskSpecs.Keys) {
        $taskSpec = $taskSpecs[$taskName]
        $taskState[$taskSpec.slot] = $null
        $taskState[$taskSpec.slot + 'Prefix'] = ''
        if ($AddonPins.Contains($taskName)) {
            if (-not $taskPrior) { throw ($taskSpec.label + ' addon preservation requires an existing installation') }
            $taskPath = Assert-Physical "$repo/tools/jenkins-management/$($taskSpec.file)"
            $taskBytes = [IO.File]::ReadAllBytes($taskPath)
            $taskPrefix = Get-ManagementJobExtensionPrefix -AddonBytes $taskBytes -ExpectedSha256 ([string]$AddonPins[$taskName]) -Kind $taskSpec.kind
            $taskState[$taskSpec.slot + 'Prefix'] = $taskPrefix
            $taskState[$taskSpec.slot] = [ordered]@{
                kind=$taskSpec.contract;sourcePath=$taskPath;sourceSha256=[string]$AddonPins[$taskName];
                prefixSha256=(Hash-Bytes ([Text.Encoding]::UTF8.GetBytes($taskPrefix)));
                capsuleMember=$false;liveVerified=$false;preserved=$false
            }
            $taskState.combinedPrefix += $taskPrefix
        }
    }
    $taskState.combinedSha256 = Hash-Bytes ([Text.Encoding]::UTF8.GetBytes($taskState.combinedPrefix))
    if ($ExpectedCombinedSha256 -and ($ExpectedCombinedSha256 -cnotmatch '^[0-9a-f]{64}$' -or
            $taskState.combinedSha256 -cne $ExpectedCombinedSha256)) { throw 'Canonical combined addon prefix differs' }
    return [pscustomobject]$taskState
}

function Get-ReviewedCandidateSources {
    $taskCanonicalNames = @('MigrateCanonicalManagementClients','CanonicalClientSourceManifestPath',
        'ExpectedCanonicalClientSourceManifestSha256')
    $taskCanonicalCount = @($taskCanonicalNames | Where-Object { $script:taskInstallerParameters.ContainsKey($_) }).Count
    if ($taskCanonicalCount -and ($taskCanonicalCount -ne $taskCanonicalNames.Count -or
            -not $MigrateCanonicalManagementClients -or -not $CanonicalClientSourceManifestPath -or
            -not $ExpectedCanonicalClientSourceManifestSha256)) { throw 'Canonical migration parameters must be supplied together' }
    if ($MigrateCanonicalManagementClients) {
        foreach ($taskExclusiveName in @('CandidateSourceRoot','CandidateSourceManifestPath',
                'ExpectedCandidateSourceManifestSha256','ExpectedCandidateCapsulePaths','AnchorManagementClients',
                'ReuseCapsuleSha256','PinnedActiveBuildProofPath','ExpectedPinnedActiveBuildProofSha256',
                'ExpectedPinnedActiveGuardSha256','ExpectedRetainedPrefixSha256')) {
            if ($script:taskInstallerParameters.ContainsKey($taskExclusiveName)) { throw 'Canonical migration cannot mix source or pinned-active modes' }
        }
        if ($taskReuse -or $taskPinnedActive -or -not $taskPrior -or
                $taskOldInstalled.driverDigest -cne $ExpectedDriverDigest -or
                $taskOldInstalled.runtimeOperationId -cne $ExpectedRuntimeOperationId) {
            throw 'Canonical migration requires the exact existing idle runtime'
        }
        $taskCandidateRoot = Assert-Physical $repo
        $taskCandidateManifest = Assert-Physical $CanonicalClientSourceManifestPath
        $taskManifestSha256 = $ExpectedCanonicalClientSourceManifestSha256
        if ((Assert-Physical "$PSScriptRoot/../..") -cne $taskCandidateRoot) { throw 'Canonical installer must execute from its actual repository' }
    } else {
        if ([string]::IsNullOrEmpty($CandidateSourceRoot)) {
            if ($CandidateSourceManifestPath -or $ExpectedCandidateSourceManifestSha256 -or
                    $ExpectedCandidateCapsulePaths.Count -or $AnchorManagementClients) { throw 'Candidate sources require their exact managed root' }
            return $null
        }
        if ($taskReuse -or $taskPinnedActive -or -not $taskPrior -or -not $EnableManagedWorktree -or
                -not $CandidateSourceManifestPath -or -not $ExpectedCandidateSourceManifestSha256 -or
                $taskOldInstalled.driverDigest -cne $ExpectedDriverDigest -or
                $taskOldInstalled.runtimeOperationId -cne $ExpectedRuntimeOperationId) {
            throw 'Candidate overrides require a fresh idle managed installation and exact current original pins'
        }
        $taskCandidateRoot = Assert-Physical $CandidateSourceRoot
        $taskCandidateManifest = Assert-Physical $CandidateSourceManifestPath
        $taskManifestSha256 = $ExpectedCandidateSourceManifestSha256
        if ((Assert-Physical "$PSScriptRoot/../..") -cne $taskCandidateRoot) { throw 'Candidate installer must execute from its original managed WT' }
    }
    $taskCandidateBootstrap = @'
import base64,hashlib,importlib.util,json,os,sys
from pathlib import Path
repo,source,manifest_path,manifest_sha,installed_sha,driver,runtime,capsule_sha,member_paths_json,anchor_clients,canonical,product,addon_json=sys.argv[1:]
member_paths_json=base64.b64decode(member_paths_json,validate=True)
addon_json=base64.b64decode(addon_json,validate=True)
canonical=json.loads(canonical)
product=json.loads(product)
if type(canonical) is not bool or type(product) is not bool: raise ValueError('candidate_mode_invalid')
anchor_clients=json.loads(anchor_clients)
if type(anchor_clients) is not bool: raise ValueError('candidate_client_anchor_invalid')
repo,source,manifest_path=Path(repo),Path(source),Path(manifest_path)
def require(value,message):
    if not value: raise ValueError(message)
sys.path.insert(0,str(repo/'.jenkins/runtime/drivers'/driver))
from tools.jenkins.deployment.driver import verify_driver,_plain
verify_driver(repo/'.jenkins/runtime/drivers'/driver,expected_digest=driver)
require(os.name=='nt' and os.environ.get('JOB_NAME')=='zircon-management-tests'
    and os.environ.get('NODE_NAME')=='zircon-windows-agent','fixed_candidate_jenkins_job_required')
for path in (source,manifest_path,repo/'.jenkins/state/management/installed.json'): _plain(path)
require(manifest_path.is_relative_to(repo/'.jenkins/state/management/reports')
    or manifest_path.is_relative_to(Path('E:/cargo-targets/zircon-local/reports')),'candidate_manifest_root_rejected')
raw=manifest_path.read_bytes()
require(hashlib.sha256(raw).hexdigest()==manifest_sha,'candidate_manifest_changed')
def unique_pairs(pairs):
    result={}
    for key,value in pairs:
        require(key not in result,'candidate_manifest_duplicate_key')
        result[key]=value
    return result
proof=json.loads(raw,object_pairs_hook=unique_pairs)
if canonical:
    require(manifest_path.is_absolute() and manifest_path.resolve()==manifest_path
        and manifest_path.stat().st_nlink==1,'canonical_manifest_physical_identity_invalid')
    fields={'schemaVersion','kind','repositoryRoot','originalInstalledSha256','originalCapsuleSha256',
        'driverDigest','runtimeOperationId','installerSha256','files','clientSources','addonSources',
        'combinedExtensionPrefixSha256'}
    require(type(proof) is dict and set(proof)==fields and type(proof['schemaVersion']) is int
        and proof['schemaVersion']==1 and proof['kind']=='jenkins-management-canonical-client-migration-v1'
        and proof['repositoryRoot']==str(repo) and source==repo
        and proof['originalInstalledSha256']==installed_sha and proof['originalCapsuleSha256']==capsule_sha
        and proof['driverDigest']==driver and proof['runtimeOperationId']==runtime,
        'canonical_manifest_identity_invalid')
    def physical_file(path):
        _plain(path)
        require(path.is_absolute() and path.resolve()==path and path.is_file()
            and path.stat().st_nlink==1,'canonical_source_physical_identity_invalid')
        return path.read_bytes()
    require(repo.is_absolute() and repo.resolve()==repo,'canonical_repository_physical_identity_invalid')
    members={name:'tools/jenkins-management/'+name for name in
        ('runner.py','source_preparation.py','zircon-management.groovy','checkout_guard.py')}
    if product: members['checkout_product.py']='tools/jenkins-management/checkout_product.py'
    client_names={'tools/dev/local_cargo.py','tools/dev/local-cargo.ps1','tools/dev/dev-fast-build.ps1',
        'tools/dev/submit-jenkins-management.ps1','tools/jenkins/frontend.py','tools/jenkins-management/entry.py'}
    addon_paths={'ExpectedCheckoutCommitAddonSha256':'tools/jenkins-management/checkout-commit.groovy',
        'ExpectedDeliveryAddonSha256':'tools/jenkins-management/delivery.groovy',
        'ExpectedBranchIntegrationAddonSha256':'tools/jenkins-management/branch-integration.groovy',
        'ExpectedOldBranchCleanupAddonSha256':'tools/jenkins-management/branch_cleanup.groovy'}
    addon_args=json.loads(addon_json,object_pairs_hook=unique_pairs)
    require(type(proof['files']) is dict and set(proof['files'])==set(members.values())
        and type(proof['clientSources']) is dict and set(proof['clientSources'])==client_names
        and type(proof['addonSources']) is dict and set(proof['addonSources'])==set(addon_paths)
        and type(addon_args) is dict and set(addon_args)<=set(addon_paths)
        and all(type(value) is str and proof['addonSources'][key]==value for key,value in addon_args.items()),
        'canonical_source_inventory_invalid')
    def valid_sha(value):
        return type(value) is str and len(value)==64 and all(c in '0123456789abcdef' for c in value)
    require(valid_sha(proof['installerSha256']) and valid_sha(proof['combinedExtensionPrefixSha256']),'canonical_source_pin_invalid')
    installed_raw=physical_file(repo/'.jenkins/state/management/installed.json')
    require(hashlib.sha256(installed_raw).hexdigest()==installed_sha,'candidate_original_install_changed')
    installed=json.loads(installed_raw,object_pairs_hook=unique_pairs)
    require(installed.get('capsuleSha256')==capsule_sha and installed.get('driverDigest')==driver
        and installed.get('runtimeOperationId')==runtime,'candidate_original_binding_changed')
    capsule=repo/'.jenkins/runtime/management'/capsule_sha
    require(installed.get('capsule')==str(capsule),'canonical_original_capsule_path_invalid')
    metadata_raw=physical_file(capsule/'capsule.json')
    require(hashlib.sha256(metadata_raw).hexdigest()==capsule_sha,'original_capsule_changed')
    metadata=json.loads(metadata_raw,object_pairs_hook=unique_pairs)
    require(metadata.get('clientRoot')=='clients' and type(metadata.get('clientSources')) is dict
        and set(metadata['clientSources'])==client_names,'canonical_original_client_inventory_invalid')
    require(type(metadata.get('files')) is dict and set(metadata['files']) in (
        {'runner.py','source_preparation.py','zircon-management.groovy'},
        {'runner.py','source_preparation.py','zircon-management.groovy','checkout_guard.py'},
        {'runner.py','source_preparation.py','zircon-management.groovy','checkout_guard.py','checkout_product.py'}),
        'canonical_original_member_inventory_invalid')
    for name,checksum in metadata['files'].items():
        require(valid_sha(checksum) and hashlib.sha256(physical_file(capsule/name)).hexdigest()==checksum,
            'canonical_original_member_changed')
    old_clients={}
    for name,checksum in metadata['clientSources'].items():
        require(valid_sha(checksum),'canonical_original_client_pin_invalid')
        path=capsule/'clients'/name
        require(hashlib.sha256(physical_file(path)).hexdigest()==checksum,'canonical_original_client_changed')
        old_clients[name]={'path':str(path),'sha256':checksum}
    pins={**proof['files'],**proof['clientSources'],
        **{addon_paths[key]:value for key,value in proof['addonSources'].items()},
        'tools/dev/install-jenkins-management.ps1':proof['installerSha256']}
    for relative,checksum in pins.items():
        require(valid_sha(checksum),'canonical_source_pin_invalid')
        require(hashlib.sha256(physical_file(repo/relative)).hexdigest()==checksum,'canonical_source_changed')
    print(json.dumps({'kind':proof['kind'],'manifestSha256':manifest_sha,'sourceRoot':str(repo),
        'sourcePins':pins,'driverDigest':driver,'runtimeOperationId':runtime,
        'files':{name:str(repo/relative) for name,relative in members.items()},
        'clientSources':{name:str(repo/name) for name in proof['clientSources']},
        'clientSourcePins':proof['clientSources'],'originalClients':old_clients,
        'addonSourcePins':proof['addonSources'],'combinedExtensionPrefixSha256':proof['combinedExtensionPrefixSha256']}))
    sys.exit(0)
require(source.parent==repo/'.jenkins/worktrees','registered_candidate_source_root_required')
proof_fields={'schemaVersion','kind','repositoryId','worktreeId','sourceRoot',
    'writerSessionId','writerPrincipal','writerAuthorizationSnapshot','ownedPaths','files','installerSha256',
    'driverDigest','runtimeOperationId','originalInstalledSha256'}
if anchor_clients: proof_fields.add('candidateClientSources')
require(isinstance(proof,dict) and set(proof)==proof_fields
    and proof.get('schemaVersion')==1 and proof.get('kind')=='jenkins-management-candidate-sources-v1'
    and proof.get('sourceRoot')==str(source) and proof.get('driverDigest')==driver
    and proof.get('runtimeOperationId')==runtime and proof.get('originalInstalledSha256')==installed_sha,
    'candidate_manifest_identity_invalid')
members={'runner.py':'tools/jenkins-management/runner.py','source_preparation.py':'tools/jenkins-management/source_preparation.py'}
require(isinstance(proof.get('files'),dict) and isinstance(proof.get('ownedPaths'),list),
    'candidate_member_scope_invalid')
member_paths=json.loads(member_paths_json)
if member_paths:
    allowed={'runner.py':'tools/jenkins-management/runner.py',
        'source_preparation.py':'tools/jenkins-management/source_preparation.py',
        'zircon-management.groovy':'tools/jenkins-management/zircon-management.groovy',
        'checkout_guard.py':'tools/jenkins-management/checkout_guard.py',
        'checkout_product.py':'tools/jenkins-management/checkout_product.py'}
    require(isinstance(member_paths,list) and all(isinstance(v,str) for v in member_paths)
        and len(member_paths)==len(set(member_paths))
        and set(member_paths)<=set(allowed.values()),'candidate_declared_members_invalid')
    members={name:path for name,path in allowed.items() if path in member_paths}
    require(set(proof['files'])==set(member_paths)
        and set(member_paths)|{'tools/dev/install-jenkins-management.ps1'}<=set(proof['ownedPaths']),
        'candidate_member_scope_invalid')
elif set(proof['files'])=={'tools/jenkins-management/zircon-management.groovy'}:
    members={'zircon-management.groovy':'tools/jenkins-management/zircon-management.groovy'}
    require(set(proof['ownedPaths'])==set(members.values())|{'tools/dev/install-jenkins-management.ps1'},
        'candidate_member_scope_invalid')
else:
    require(set(proof['files'])==set(members.values())
        and set(members.values())|{'tools/dev/install-jenkins-management.ps1'}<=set(proof['ownedPaths']),
        'candidate_member_scope_invalid')
client_names={'tools/dev/local_cargo.py','tools/dev/local-cargo.ps1','tools/dev/dev-fast-build.ps1',
    'tools/dev/submit-jenkins-management.ps1','tools/jenkins/frontend.py','tools/jenkins-management/entry.py'}
client_pins=proof.get('candidateClientSources',{})
if anchor_clients:
    require(isinstance(client_pins,dict) and set(client_pins)==client_names
        and client_names<=set(proof['ownedPaths']),'candidate_client_scope_invalid')
installed_raw=(repo/'.jenkins/state/management/installed.json').read_bytes()
require(hashlib.sha256(installed_raw).hexdigest()==installed_sha,'candidate_original_install_changed')
installed=json.loads(installed_raw)
require(installed.get('capsuleSha256')==capsule_sha and installed.get('driverDigest')==driver
    and installed.get('runtimeOperationId')==runtime,'candidate_original_binding_changed')
capsule=repo/'.jenkins/runtime/management'/capsule_sha
metadata_raw=(capsule/'capsule.json').read_bytes()
require(hashlib.sha256(metadata_raw).hexdigest()==capsule_sha,'original_capsule_changed')
metadata=json.loads(metadata_raw)
require(set(members)<=set(metadata['files']),'candidate_member_inventory_invalid')
trusted_helper=capsule/'source_preparation.py'
_plain(trusted_helper)
require(hashlib.sha256(trusted_helper.read_bytes()).hexdigest()==metadata['files']['source_preparation.py'],
    'original_authority_helper_changed')
spec=importlib.util.spec_from_file_location('original_candidate_authority_helper',trusted_helper)
helper=importlib.util.module_from_spec(spec)
spec.loader.exec_module(helper)
from tools.jenkins.state import State
state=helper.scoped_authorization_state(State(repo/'.jenkins/state/coordination.sqlite3'))
with state.transaction() as connection:
    row=state.get('management_worktree',proof['worktreeId'],connection=connection)
    record=row['payload'] if row else {}
    writer=state.check_authorization(proof['repositoryId'],proof['writerSessionId'],'implementation',
        proof['ownedPaths'],connection=connection)
    require(record.get('status')=='ready' and record.get('path')==str(source)
        and record.get('sessionId')==proof['writerSessionId']
        and all(record.get(name)==proof[name] for name in ('repositoryId','driverDigest','runtimeOperationId',
            'writerPrincipal','writerAuthorizationSnapshot'))
        and sorted(record.get('ownedPaths',[]))==sorted(proof['ownedPaths'])==sorted(writer['paths'])
        and writer['principal']==proof['writerPrincipal'] and writer['snapshotDigest']==proof['writerAuthorizationSnapshot'],
        'candidate_ready_writer_authority_changed')
    from tools.jenkins.managed_worktree_lifetime import assert_no_writer
    assert_no_writer(state,proof['worktreeId'],connection=connection)
    gitfile=source/'.git'
    _plain(gitfile)
    gittext=gitfile.read_text(encoding='utf-8').strip()
    require(gittext.startswith('gitdir: '),'real_linked_candidate_wt_required')
    gitdir=Path(gittext[8:])
    _plain(gitdir)
    common=gitdir/'commondir'
    _plain(common)
    require(gitdir.is_absolute() and gitdir.parent==repo/'.git/worktrees'
        and (gitdir/common.read_text(encoding='utf-8').strip()).resolve()==repo/'.git','candidate_git_owner_changed')
    pins={**proof['files'],**client_pins,'tools/dev/install-jenkins-management.ps1':proof['installerSha256']}
    for relative,checksum in pins.items():
        require(isinstance(checksum,str) and len(checksum)==64 and all(c in '0123456789abcdef' for c in checksum),
            'candidate_source_pin_invalid')
        path=source/relative
        _plain(path)
        require(path.is_file() and hashlib.sha256(path.read_bytes()).hexdigest()==checksum,'candidate_source_changed')
print(json.dumps({'manifestSha256':manifest_sha,'sourceRoot':str(source),'worktreeId':proof['worktreeId'],
    'writerAuthorizationSnapshot':proof['writerAuthorizationSnapshot'],'sourcePins':pins,
    'driverDigest':driver,'runtimeOperationId':runtime,'files':{name:str(source/relative) for name,relative in members.items()},
    'clientSources':{name:str(source/name) for name in client_pins},'clientSourcePins':client_pins}))
'@
    $taskCandidatePython = Assert-Physical "$repo/.jenkins/runtime/python/python.exe"
    $taskCandidatePathsJson = ConvertTo-Json -InputObject @($ExpectedCandidateCapsulePaths) -Compress
    $taskCandidatePathsB64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($taskCandidatePathsJson))
    $taskAddonPins = [ordered]@{}
    if ($MigrateCanonicalManagementClients) {
        foreach ($taskAddonName in @('ExpectedCheckoutCommitAddonSha256','ExpectedDeliveryAddonSha256',
                'ExpectedBranchIntegrationAddonSha256','ExpectedOldBranchCleanupAddonSha256')) {
            if ($script:taskInstallerParameters.ContainsKey($taskAddonName)) {
                $taskAddonPins[$taskAddonName] = $script:taskInstallerParameters[$taskAddonName]
            }
        }
    }
    $taskAddonPinsJson = ConvertTo-Json -InputObject $taskAddonPins -Compress
    $taskAddonPinsB64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($taskAddonPinsJson))
    # The shared physical helper binds the complete verified manifest prefix below.
    $taskCandidateOutput = & $taskCandidatePython -B -X utf8 -c $taskCandidateBootstrap $repo $taskCandidateRoot $taskCandidateManifest $taskManifestSha256 $ExpectedInstalledSha256 $ExpectedDriverDigest $ExpectedRuntimeOperationId $taskOldCapsuleSha256 $taskCandidatePathsB64 ([string]$AnchorManagementClients.IsPresent).ToLowerInvariant() ([string]$MigrateCanonicalManagementClients.IsPresent).ToLowerInvariant() ([string]$EnableCheckoutProduct.IsPresent).ToLowerInvariant() $taskAddonPinsB64
    if ($LASTEXITCODE -ne 0) { throw 'Jenkins candidate WT authority or source pins could not be verified' }
    $taskCandidateResult = $taskCandidateOutput | ConvertFrom-Json
    if ($MigrateCanonicalManagementClients) {
        $taskVerifiedAddonPins = [ordered]@{}
        foreach ($taskPin in $taskCandidateResult.addonSourcePins.PSObject.Properties) {
            $taskVerifiedAddonPins[$taskPin.Name] = [string]$taskPin.Value
        }
        Get-ManagementJobExtensionState -AddonPins $taskVerifiedAddonPins -ExpectedCombinedSha256 $taskCandidateResult.combinedExtensionPrefixSha256 | Out-Null
    }
    return $taskCandidateResult
}
function Get-ManagementCapsuleSource([string]$Name) {
    if ($null -ne $taskCandidateSources -and
            $null -ne $taskCandidateSources.files.PSObject.Properties[$Name]) {
        return (Assert-Physical ([string]$taskCandidateSources.files.PSObject.Properties[$Name].Value))
    }
    if ($MigrateCanonicalManagementClients) { throw 'Canonical capsule member is missing from the complete source plan' }
    return (Assert-Physical "$taskSourceRoot/$Name")
}
function Get-ManagementClientSource([string]$Name) {
    if ($MigrateCanonicalManagementClients -or $AnchorManagementClients) {
        if ($null -eq $taskCandidateSources -or
                $null -eq $taskCandidateSources.clientSources.PSObject.Properties[$Name]) { throw 'Reviewed candidate client is missing' }
        return (Assert-Physical ([string]$taskCandidateSources.clientSources.PSObject.Properties[$Name].Value))
    }
    if ($taskAnchoredClients) { return (Assert-Physical "$taskOldSourceRoot/clients/$Name") }
    return (Assert-Physical "$repo/$Name")
}
function Assert-ManagementClientBytes([string]$PublishedRoot) {
    foreach ($taskClientName in $clientSources.Keys) {
        if ((Hash-Bytes ([IO.File]::ReadAllBytes((Get-ManagementClientSource $taskClientName)))) -cne
                $clientSources[$taskClientName]) { throw 'Public entry source changed' }
        if ($taskAnchoredClients -and $PublishedRoot) {
            if ((Hash-Bytes ([IO.File]::ReadAllBytes((Assert-Physical "$PublishedRoot/clients/$taskClientName")))) -cne
                    $clientSources[$taskClientName]) { throw 'Pinned capsule client changed' }
        }
    }
}
function Assert-ReviewedCandidateSources {
    if ($null -eq $taskCandidateSources) { return }
    $taskCandidateAgain = Get-ReviewedCandidateSources
    if (($taskCandidateAgain | ConvertTo-Json -Depth 8 -Compress) -cne
            ($taskCandidateSources | ConvertTo-Json -Depth 8 -Compress)) { throw 'Candidate source authority changed before installation' }
}

function Get-CanonicalMigrationPreviousConfig {
    $taskObservedCredentials = Get-Content -Raw -LiteralPath "$repo/.jenkins/state/credentials.json" | ConvertFrom-Json
    $taskObservedToken = if ($taskObservedCredentials.apiToken) { $taskObservedCredentials.apiToken } else { $taskObservedCredentials.password }
    if ($taskObservedCredentials.authenticationMode -cne 'api-token' -or
            -not $taskObservedCredentials.username -or -not $taskObservedToken) { throw 'Managed API token credentials required' }
    $taskObservedAuth = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes("$($taskObservedCredentials.username):$taskObservedToken"))
    $taskObservedResponse = Invoke-WebRequest -Uri ($BaseUrl.TrimEnd('/') + '/job/zircon-management/config.xml') -Headers @{Authorization="Basic $taskObservedAuth"} -UseBasicParsing -TimeoutSec 15
    $taskObservedBytes = [Text.Encoding]::UTF8.GetBytes($taskObservedResponse.Content)
    [xml]$taskObservedJob = ($taskObservedResponse.Content -replace '^\s*<\?xml[^?]*\?>', '')
    [xml]$taskObservedHome = ([Text.Encoding]::UTF8.GetString([IO.File]::ReadAllBytes((Assert-Physical "$repo/.jenkins/jenkins_home/jobs/zircon-management/config.xml"))) -replace '^\s*<\?xml[^?]*\?>', '')
    $taskObservedCore = [IO.File]::ReadAllText((Assert-Physical "$taskOldSourceRoot/zircon-management.groovy")).Replace(([string][char]13 + [char]10), [string][char]10)
    $taskObservedTokens = @{AGENT='zircon-windows-agent';REPOSITORY=$repo;PYTHON=$python;CODEX=$taskPreviousCodex;
        CAPSULE=$taskOldSourceRoot;DRIVER=$taskOldInstalled.driverDigest;RUNTIME=$taskOldInstalled.runtimeOperationId}
    foreach ($taskObservedKey in $taskObservedTokens.Keys) {
        $taskObservedCore = $taskObservedCore.Replace("@@$taskObservedKey@@", $taskObservedTokens[$taskObservedKey].Replace('\','/').Replace("'","\'"))
    }
    $taskObservedPipeline = Join-ManagementJobExtension -Core $taskObservedCore -Prefix $taskCombinedExtensionPrefix
    Assert-ManagementJob $taskObservedJob $taskObservedPipeline $taskOldInstalled.driverDigest $taskOldInstalled.runtimeOperationId
    Assert-ManagementJob $taskObservedHome $taskObservedPipeline $taskOldInstalled.driverDigest $taskOldInstalled.runtimeOperationId
    Assert-PreviousManagementConfig $taskObservedBytes $ExpectedPreviousJobConfigSha256
    return [ordered]@{originalConfigSha256=(Hash-Bytes $taskObservedBytes);apiHomeScriptEqual=$true;
        originalCoreSha256=(Hash-Bytes ([Text.Encoding]::UTF8.GetBytes($taskObservedCore)));
        combinedExtensionPrefixSha256=(Hash-Bytes ([Text.Encoding]::UTF8.GetBytes($taskCombinedExtensionPrefix)));
        liveVerified=$true}
}

$repo = Assert-Physical $RepoRoot
$taskInstallLock = $null
try {
if (-not $CheckOnly) {
    $taskLockFolder = Assert-Physical "$repo/.jenkins/state/management"
    [void][IO.Directory]::CreateDirectory($taskLockFolder)
    $taskInstallLock = [IO.File]::Open((Assert-Physical "$taskLockFolder/install.lock"), [IO.FileMode]::OpenOrCreate, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
}
$selected = Get-Content -Raw -LiteralPath "$repo/.jenkins/state/deployment/driver.json" | ConvertFrom-Json
$operation = Get-Content -Raw -LiteralPath "$repo/.jenkins/state/deployment-operation.json" | ConvertFrom-Json
if ($selected.driverDigest -ne $ExpectedDriverDigest -or $selected.runtimeOperationId -ne $ExpectedRuntimeOperationId) {
    throw 'Selected driver/runtime differs; installation did not change it'
}
if ($operation.operationId -ne $ExpectedRuntimeOperationId -or $operation.state -ne 'running') {
    throw 'The expected formal runtime must already be running'
}
$spec = Get-Content -Raw -LiteralPath "$repo/.jenkins/deployment-spec.json" | ConvertFrom-Json
if ($spec.agent.name -ne 'zircon-windows-agent' -or $BaseUrl.TrimEnd('/') -ne 'http://127.0.0.1:18080') {
    throw 'This MVP uses only the fixed local node and controller'
}
$taskReuse = -not [string]::IsNullOrEmpty($ReuseCapsuleSha256)
$taskPostedAlready = $false
if ($ResumePreparedInstallation -and [string]::IsNullOrEmpty($ExpectedInstalledSha256)) { throw 'Resume requires the exact original installed pin' }
$taskInstalledPath = Assert-Physical "$repo/.jenkins/state/management/installed.json"
$taskPrior = Test-Path -LiteralPath $taskInstalledPath
if ($taskPrior -and [string]::IsNullOrEmpty($ExpectedInstalledSha256)) { throw 'Updating an existing installation requires its exact original installed hash' }
if (-not $taskPrior -and ($taskReuse -or $ExpectedInstalledSha256)) { throw 'Original installation is absent; reconcile instead of rebinding it' }
$taskPinnedNames = @('PinnedActiveBuildProofPath','ExpectedPinnedActiveBuildProofSha256','ExpectedPinnedActiveGuardSha256')
$taskPinnedCount = @($taskPinnedNames | Where-Object { $PSBoundParameters.ContainsKey($_) }).Count
if ($taskPinnedCount -ne 0 -and $taskPinnedCount -ne $taskPinnedNames.Count) { throw 'Pinned-active proof path and both operator/proof hashes must be paired' }
$taskPinnedActive = $taskPinnedCount -eq $taskPinnedNames.Count
$taskRetainedPrefixMode = $PSBoundParameters.ContainsKey('ExpectedRetainedPrefixSha256')
if ($taskRetainedPrefixMode -and (-not $taskPrior -or $taskPinnedActive -or
        [string]::IsNullOrEmpty($ExpectedPreviousJobConfigSha256) -or
        $PSBoundParameters.ContainsKey('ExpectedCheckoutCommitAddonSha256') -or
        $PSBoundParameters.ContainsKey('ExpectedDeliveryAddonSha256') -or
        $PSBoundParameters.ContainsKey('ExpectedBranchIntegrationAddonSha256') -or
        $PSBoundParameters.ContainsKey('ExpectedOldBranchCleanupAddonSha256'))) {
    throw 'Reviewed prefix preservation requires an original config pin and the ordinary idle gate'
}
$taskRetainedRuntimeRebind = $taskRetainedPrefixMode -and $taskReuse -and
    $PSBoundParameters.ContainsKey('ExpectedPreviousRuntimeOperationId')
if ($PSBoundParameters.ContainsKey('ExpectedPreviousRuntimeOperationId') -and
        (-not $taskRetainedRuntimeRebind -or
         $ExpectedPreviousRuntimeOperationId -ceq $ExpectedRuntimeOperationId)) {
    throw 'Cross-runtime prefix reuse requires distinct exact old and new runtime pins'
}
$taskRetainedDriverRebind = $PSBoundParameters.ContainsKey('ExpectedPreviousDriverDigest')
if ($taskRetainedDriverRebind -and (-not $taskRetainedRuntimeRebind -or
        $ExpectedPreviousDriverDigest -ceq $ExpectedDriverDigest -or $CandidateSourceRoot)) {
    throw 'Driver migration requires exact old/new pins, retained prefixes and original capsule reuse'
}
$script:taskPinnedGuardReceipts = @()
if ($taskPinnedActive) {
    if (-not $taskPrior -or $taskReuse -or [string]::IsNullOrEmpty($ExpectedPreviousJobConfigSha256) -or
            [string]::IsNullOrEmpty($PinnedActiveBuildProofPath)) { throw 'Pinned-active update requires an exact existing installation and a fresh candidate' }
    $taskGuardSource = Assert-Physical "$repo/tools/jenkins-management/install_guard.py"
    if ((Hash-Bytes ([IO.File]::ReadAllBytes($taskGuardSource))) -cne $ExpectedPinnedActiveGuardSha256) { throw 'Pinned-active operator source changed' }
    if ((Hash-Bytes ([IO.File]::ReadAllBytes((Assert-Physical $PinnedActiveBuildProofPath)))) -cne $ExpectedPinnedActiveBuildProofSha256) { throw 'Pinned-active proof bytes changed' }
}
$taskProvidedAddonPins = [ordered]@{}
foreach ($taskAddonName in @('ExpectedCheckoutCommitAddonSha256','ExpectedDeliveryAddonSha256',
        'ExpectedBranchIntegrationAddonSha256','ExpectedOldBranchCleanupAddonSha256')) {
    if ($script:taskInstallerParameters.ContainsKey($taskAddonName)) {
        $taskProvidedAddonPins[$taskAddonName] = $script:taskInstallerParameters[$taskAddonName]
    }
}
$taskCombinedExtensionPrefix = ''
$taskSourceRoot = "$repo/tools/jenkins-management"
if ($taskPrior) {
    $taskOldInstalledBytes = [IO.File]::ReadAllBytes($taskInstalledPath)
    if ((Hash-Bytes $taskOldInstalledBytes) -ne $ExpectedInstalledSha256) { throw 'Original installed record changed' }
    $taskOldInstalled = [Text.Encoding]::UTF8.GetString($taskOldInstalledBytes) | ConvertFrom-Json
    $taskOldCapsuleSha256 = [string]$taskOldInstalled.capsuleSha256
    if ($taskOldCapsuleSha256 -notmatch '^[0-9a-f]{64}$') { throw 'Original capsule identity is invalid' }
    $taskOldSourceRoot = Assert-Physical "$repo/.jenkins/runtime/management/$taskOldCapsuleSha256"
    if ($taskOldInstalled.job -ne 'zircon-management' -or $taskOldInstalled.capsule -ne $taskOldSourceRoot) { throw 'Original capsule owner differs' }
    if ($taskReuse -and $ReuseCapsuleSha256 -ne $taskOldCapsuleSha256) { throw 'Requested original capsule differs' }
    $taskOldMetadataBytes = [IO.File]::ReadAllBytes((Assert-Physical "$taskOldSourceRoot/capsule.json"))
    if ((Hash-Bytes $taskOldMetadataBytes) -ne $taskOldCapsuleSha256) { throw 'Original capsule metadata changed' }
    $taskOldMetadata = [Text.Encoding]::UTF8.GetString($taskOldMetadataBytes) | ConvertFrom-Json
    if ($taskOldMetadata.schemaVersion -ne 1 -or $taskOldMetadata.repositoryRoot -ne $repo -or
            $taskOldMetadata.agentLabel -ne 'zircon-windows-agent' -or
            $taskOldMetadata.driverDigest -ne $taskOldInstalled.driverDigest -or
            $taskOldMetadata.runtimeOperationId -ne $taskOldInstalled.runtimeOperationId -or
            $taskOldInstalled.formalAcceptance -ne $false) { throw 'Original capsule contract differs' }
    $taskOldInventory = (@($taskOldMetadata.files.PSObject.Properties.Name | Sort-Object) -join '|')
    if ($taskOldInventory -cnotin @('runner.py|source_preparation.py|zircon-management.groovy','checkout_guard.py|runner.py|source_preparation.py|zircon-management.groovy','checkout_guard.py|checkout_product.py|runner.py|source_preparation.py|zircon-management.groovy')) { throw 'Original capsule inventory differs' }
    foreach ($taskOldName in $taskOldMetadata.files.PSObject.Properties.Name) {
        $taskOldHash = [string]$taskOldMetadata.files.PSObject.Properties[$taskOldName].Value
        if ($taskOldHash -notmatch '^[0-9a-f]{64}$' -or
                (Hash-Bytes ([IO.File]::ReadAllBytes((Assert-Physical "$taskOldSourceRoot/$taskOldName")))) -ne $taskOldHash) { throw 'Original capsule code changed' }
    }
    $taskOldClients = @('tools/dev/local_cargo.py','tools/dev/local-cargo.ps1','tools/dev/dev-fast-build.ps1',
        'tools/dev/submit-jenkins-management.ps1','tools/jenkins/frontend.py','tools/jenkins-management/entry.py')
    if ((@($taskOldMetadata.clientSources.PSObject.Properties.Name | Sort-Object) -join '|') -cne
            (@($taskOldClients | Sort-Object) -join '|')) { throw 'Original public entry inventory differs' }
    foreach ($taskOldClientName in $taskOldClients) {
        if ([string]$taskOldMetadata.clientSources.PSObject.Properties[$taskOldClientName].Value -notmatch '^[0-9a-f]{64}$') { throw 'Original public entry hash is invalid' }
    }
    if ($taskReuse) { $taskSourceRoot = $taskOldSourceRoot }
}
$taskAnchoredClients = $false
if ($taskPrior -and $null -ne $taskOldMetadata.PSObject.Properties['clientRoot']) {
    if ($taskOldMetadata.clientRoot -cne 'clients') { throw 'Original client root differs' }
    $taskAnchoredClients = $true
}
$taskCandidateSources = Get-ReviewedCandidateSources
$taskSelectedAddonPins = $taskProvidedAddonPins
$taskExpectedCombinedPrefix = $null
if ($MigrateCanonicalManagementClients) {
    $taskSelectedAddonPins = [ordered]@{}
    foreach ($taskPin in $taskCandidateSources.addonSourcePins.PSObject.Properties) {
        $taskSelectedAddonPins[$taskPin.Name] = [string]$taskPin.Value
    }
    $taskExpectedCombinedPrefix = [string]$taskCandidateSources.combinedExtensionPrefixSha256
}
$taskExtensionState = Get-ManagementJobExtensionState -AddonPins $taskSelectedAddonPins -ExpectedCombinedSha256 $taskExpectedCombinedPrefix
$taskJobExtension = $taskExtensionState.checkoutCommit
$taskJobExtensionPrefix = $taskExtensionState.checkoutCommitPrefix
$taskDeliveryExtension = $taskExtensionState.delivery
$taskDeliveryExtensionPrefix = $taskExtensionState.deliveryPrefix
$taskBranchCleanupExtension = $taskExtensionState.branchCleanup
$taskBranchCleanupExtensionPrefix = $taskExtensionState.branchCleanupPrefix
$taskBranchIntegrationExtension = $taskExtensionState.branchIntegration
$taskBranchIntegrationExtensionPrefix = $taskExtensionState.branchIntegrationPrefix
$taskCombinedExtensionPrefix = $taskExtensionState.combinedPrefix
if ($MigrateCanonicalManagementClients -and -not $CheckOnly -and -not $ExpectedPreviousJobConfigSha256) {
    throw 'Canonical installation requires the actual previous config hash from CheckOnly'
}
if ($MigrateCanonicalManagementClients -or $AnchorManagementClients) {
    if ($null -eq $taskCandidateSources) { throw 'Client anchoring requires reviewed managed source provenance' }
    $taskAnchoredClients = $true
}
if ($null -ne $taskCandidateSources) { $taskSourceRoot = $taskOldSourceRoot }
$files = [ordered]@{}
$taskCapsuleNames = if ($taskReuse -or ($null -ne $taskCandidateSources -and -not $MigrateCanonicalManagementClients)) { @($taskOldMetadata.files.PSObject.Properties.Name) } else { @('runner.py','source_preparation.py','zircon-management.groovy','checkout_guard.py') }
if (-not $taskReuse -and ($null -eq $taskCandidateSources -or $MigrateCanonicalManagementClients) -and $EnableCheckoutProduct) { $taskCapsuleNames += 'checkout_product.py' }
if ($taskReuse -and $EnableCheckoutProduct -and 'checkout_product.py' -notin $taskCapsuleNames) { throw 'Original capsule has no fixed product capability' }
foreach ($name in $taskCapsuleNames) {
    $source = Get-ManagementCapsuleSource $name
    $files[$name] = Hash-Bytes ([IO.File]::ReadAllBytes($source))
    if ($taskReuse -and $files[$name] -ne $taskOldMetadata.files.PSObject.Properties[$name].Value) { throw 'Original capsule file changed' }
}
$taskClientReviewedPins = @{}
foreach ($taskPinParameterName in @('ExpectedEntrySourceSha256','ExpectedLocalCargoSourceSha256',
        'ExpectedFrontendSourceSha256','ExpectedSubmitSourceSha256')) {
    if ($PSBoundParameters.ContainsKey($taskPinParameterName)) {
        $taskClientReviewedPins[$taskPinParameterName] = $PSBoundParameters[$taskPinParameterName]
    }
}
$clientSources = [ordered]@{}
foreach ($clientName in @('tools/dev/local_cargo.py','tools/dev/local-cargo.ps1','tools/dev/dev-fast-build.ps1',
        'tools/dev/submit-jenkins-management.ps1','tools/jenkins/frontend.py','tools/jenkins-management/entry.py')) {
    $taskClientBytes = [IO.File]::ReadAllBytes((Get-ManagementClientSource $clientName))
    $taskOldClientHash = $null
    if ($taskPrior) {
        $taskOldClient = $taskOldMetadata.clientSources.PSObject.Properties[$clientName]
        if ($null -eq $taskOldClient) { throw 'Original public entry inventory differs' }
        $taskOldClientHash = [string]$taskOldClient.Value
    }
    $taskClientArguments = @{ClientName=$clientName;ClientBytes=$taskClientBytes;OriginalSha256=$taskOldClientHash}
    if ($MigrateCanonicalManagementClients -or $AnchorManagementClients) {
        $taskCandidateClientHash = [string]$taskCandidateSources.clientSourcePins.PSObject.Properties[$clientName].Value
        if ((Hash-Bytes $taskClientBytes) -cne $taskCandidateClientHash) { throw 'Reviewed managed client bytes changed' }
        if ($MigrateCanonicalManagementClients) {
            $taskClientPinName = switch ($clientName) {
                'tools/jenkins-management/entry.py' { 'ExpectedEntrySourceSha256' }
                'tools/dev/local_cargo.py' { 'ExpectedLocalCargoSourceSha256' }
                'tools/jenkins/frontend.py' { 'ExpectedFrontendSourceSha256' }
                'tools/dev/submit-jenkins-management.ps1' { 'ExpectedSubmitSourceSha256' }
                default { $null }
            }
            if ($taskClientPinName -and $taskClientReviewedPins.ContainsKey($taskClientPinName) -and
                    $taskClientReviewedPins[$taskClientPinName] -cne $taskCandidateClientHash) {
                throw 'Existing reviewed client pin differs from the canonical source plan'
            }
        }
        $clientSources[$clientName] = $taskCandidateClientHash
    } else {
        $clientSources[$clientName] = Get-ValidatedManagementClientSha256 @taskClientArguments @taskClientReviewedPins
    }
}
if ($taskPrior -and @($taskOldMetadata.clientSources.PSObject.Properties).Count -ne $clientSources.Count) { throw 'Original public entry inventory differs' }
$ordinaryEntry = [ordered]@{enabled=$true;actions=@('validate');formalAcceptance=$false;m8Accepted=$false}
if ($taskPrior -and (($taskOldMetadata.ordinaryEntry | ConvertTo-Json -Compress) -cne ($ordinaryEntry | ConvertTo-Json -Compress) -or
        ($taskOldInstalled.ordinaryEntry | ConvertTo-Json -Compress) -cne ($ordinaryEntry | ConvertTo-Json -Compress))) { throw 'Original capability differs' }
$metadata = [ordered]@{schemaVersion=1;repositoryRoot=$repo;driverDigest=$ExpectedDriverDigest;
    runtimeOperationId=$ExpectedRuntimeOperationId;agentLabel='zircon-windows-agent';files=$files;clientSources=$clientSources;ordinaryEntry=$ordinaryEntry}
if ($taskAnchoredClients) { $metadata.clientRoot = 'clients' }
if ('checkout_guard.py' -in $files.Keys) {
    if ($taskReuse -and ($taskOldMetadata.checkoutValidation.enabled -ne $true -or
            $taskOldMetadata.checkoutValidation.policy -cne 'actual-checkout' -or
            $taskOldMetadata.checkoutValidation.guard -cne 'checkout_guard.py' -or
            @($taskOldMetadata.checkoutValidation.PSObject.Properties).Count -ne 3)) { throw 'Original capsule has no proven checkout capability; a reviewed fresh candidate is required' }
    $metadata.checkoutValidation = [ordered]@{enabled=$true;policy='actual-checkout';guard='checkout_guard.py'}
}
if ('checkout_product.py' -in $files.Keys) {
    if ($taskReuse) {
        $taskProductCapability = $taskOldMetadata.checkoutProduct
        if ($null -eq $taskProductCapability -or
                @($taskProductCapability.PSObject.Properties).Count -ne 4 -or
                $taskProductCapability.enabled -ne $true -or
                $taskProductCapability.kind -cne 'zircon-development-editor-preview' -or
                $taskProductCapability.helper -cne 'checkout_product.py' -or
                $taskProductCapability.schemaVersion -notin @(1,2)) { throw 'Original product capability changed' }
    } else {
        $taskProductCapability = [ordered]@{enabled=$true;kind='zircon-development-editor-preview';schemaVersion=2;helper='checkout_product.py'}
    }
    $metadata.checkoutProduct = $taskProductCapability
} elseif ($taskReuse -and $null -ne $taskOldMetadata.checkoutProduct) { throw 'Product capability has no pinned helper' }
$taskManagedWorktree = $null
if ($taskReuse) {
    if ($null -ne $taskOldMetadata.managedWorktree) {
        $taskExpectedManaged = Get-ManagedWorktreeCapability
        if (($taskOldMetadata.managedWorktree | ConvertTo-Json -Depth 5 -Compress) -cne
                ($taskExpectedManaged | ConvertTo-Json -Depth 5 -Compress) -or
                ($taskOldInstalled.managedWorktree | ConvertTo-Json -Depth 5 -Compress) -cne
                ($taskExpectedManaged | ConvertTo-Json -Depth 5 -Compress)) { throw 'Original managed worktree capability changed' }
        $taskManagedWorktree = $taskOldMetadata.managedWorktree
    } elseif ($EnableManagedWorktree) { throw 'Original capsule has no managed worktree capability' }
} elseif ($EnableManagedWorktree) {
    $taskRunnerSource = [IO.File]::ReadAllText((Get-ManagementCapsuleSource 'runner.py'))
    if ($taskRunnerSource -notmatch '(?m)^MANAGED_WORKTREE_CAPABILITY = \{') { throw 'Management runner has no managed worktree capability' }
    $taskManagedWorktree = Get-ManagedWorktreeCapability
}
if ($null -ne $taskManagedWorktree) {
    Assert-ManagedWorktreeDriver $repo $ExpectedDriverDigest
    $metadata.managedWorktree = $taskManagedWorktree
}
$bytes = [Text.UTF8Encoding]::new($false).GetBytes(($metadata | ConvertTo-Json -Depth 12 -Compress))
$digest = Hash-Bytes $bytes
if ($ExpectedCapsuleSha256 -and $ExpectedCapsuleSha256 -ne $digest) { throw 'Reviewed candidate capsule differs' }
if (-not $CheckOnly -and [string]::IsNullOrEmpty($ExpectedCapsuleSha256)) { throw 'Installation requires the exact reviewed candidate capsule hash from CheckOnly' }
$capsule = Assert-Physical "$repo/.jenkins/runtime/management/$digest"
$python = Assert-Physical "$repo/.jenkins/runtime/python/python.exe"
if ($CodexExe -notmatch '^[A-Za-z]:[\\/]') { throw 'Current Codex must be an absolute physical path' }
$codex = Assert-Physical $CodexExe
if (!(Test-Path -LiteralPath $python -PathType Leaf) -or !(Test-Path -LiteralPath $codex -PathType Leaf)) { throw 'Managed Python or official Codex sandbox executable missing' }
$taskPreviousCodex = Resolve-PreviousCodexBinding $codex $ExpectedPreviousCodexExe $ExpectedPreviousJobConfigSha256 $taskPrior
$pipeline = [IO.File]::ReadAllText((Get-ManagementCapsuleSource 'zircon-management.groovy')).Replace(([string][char]13 + [char]10), [string][char]10)
$taskPipelineTemplate = $pipeline
$tokens = @{AGENT='zircon-windows-agent';REPOSITORY=$repo;PYTHON=$python;CODEX=$codex;
    CAPSULE=$capsule;DRIVER=$ExpectedDriverDigest;RUNTIME=$ExpectedRuntimeOperationId}
foreach ($key in $tokens.Keys) {
    $value = $tokens[$key].Replace('\','/').Replace("'","\'")
    if ($value.Contains([char]10) -or $value.Contains([char]13) -or $value.Contains('"')) { throw 'Unsafe fixed job path' }
    $pipeline = $pipeline.Replace("@@$key@@", $value)
}
if ($pipeline -match '@@[A-Z]+@@') { throw 'Unresolved capsule marker' }
$taskBarePipeline = $pipeline
$taskRetainedPrefix = $null
if ($taskRetainedPrefixMode) {
    if ($taskOldInstalled.driverDigest -cne $ExpectedDriverDigest) {
        if (-not $taskRetainedDriverRebind -or
                $taskOldInstalled.driverDigest -cne $ExpectedPreviousDriverDigest) {
            throw 'Retained management prefixes require the exact original driver'
        }
    } elseif ($taskRetainedDriverRebind) {
        throw 'Driver migration did not retain distinct exact old/new drivers'
    }
    if ($taskRetainedRuntimeRebind) {
        if ($taskOldInstalled.runtimeOperationId -cne $ExpectedPreviousRuntimeOperationId -or
                $taskOldInstalled.capsuleSha256 -cne $ReuseCapsuleSha256) {
            throw 'Retained runtime reuse differs from its exact old runtime or capsule'
        }
    } elseif ($taskOldInstalled.runtimeOperationId -cne $ExpectedRuntimeOperationId) {
        throw 'Retained management prefixes require the exact current original runtime'
    }
    $taskRetainedOldCore = [IO.File]::ReadAllText((Assert-Physical "$taskOldSourceRoot/zircon-management.groovy")).Replace(([string][char]13 + [char]10), [string][char]10)
    $taskRetainedOldTokens = @{AGENT='zircon-windows-agent';REPOSITORY=$repo;PYTHON=$python;CODEX=$taskPreviousCodex;
        CAPSULE=$taskOldSourceRoot;DRIVER=$taskOldInstalled.driverDigest;RUNTIME=$taskOldInstalled.runtimeOperationId}
    foreach ($taskKey in $taskRetainedOldTokens.Keys) {
        $taskOldValue = $taskRetainedOldTokens[$taskKey].Replace('\','/').Replace("'","\'")
        $taskRetainedOldCore = $taskRetainedOldCore.Replace("@@$taskKey@@", $taskOldValue)
    }
    $taskRetainedCredentials = Get-Content -Raw -LiteralPath "$repo/.jenkins/state/credentials.json" | ConvertFrom-Json
    $taskRetainedToken = if ($taskRetainedCredentials.apiToken) { $taskRetainedCredentials.apiToken } else { $taskRetainedCredentials.password }
    if ($taskRetainedCredentials.authenticationMode -cne 'api-token' -or
            -not $taskRetainedCredentials.username -or -not $taskRetainedToken) { throw 'Managed API token credentials required' }
    $taskRetainedAuth = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes("$($taskRetainedCredentials.username):$taskRetainedToken"))
    $taskRetainedResponse = Invoke-WebRequest -Uri ($BaseUrl.TrimEnd('/') + '/job/zircon-management/config.xml') -Headers @{Authorization="Basic $taskRetainedAuth"} -UseBasicParsing -TimeoutSec 15
    $taskRetainedApiBytes = [Text.Encoding]::UTF8.GetBytes($taskRetainedResponse.Content)
    $taskRetainedHomeBytes = [IO.File]::ReadAllBytes((Assert-Physical "$repo/.jenkins/jenkins_home/jobs/zircon-management/config.xml"))
    [xml]$taskRetainedApiJob = ($taskRetainedResponse.Content -replace '^\s*<\?xml[^?]*\?>', '')
    [xml]$taskRetainedHomeJob = ([Text.Encoding]::UTF8.GetString($taskRetainedHomeBytes) -replace '^\s*<\?xml[^?]*\?>', '')
    $taskRetainedObservedCore = if ($ResumePreparedInstallation) { $taskBarePipeline } else { $taskRetainedOldCore }
    $taskCombinedExtensionPrefix = Get-ManagementJobRetainedPrefix -ApiScript ([string]$taskRetainedApiJob.'flow-definition'.definition.script) -HomeScript ([string]$taskRetainedHomeJob.'flow-definition'.definition.script) -BareCore $taskRetainedObservedCore -ExpectedSha256 $ExpectedRetainedPrefixSha256
    $taskRetainedOriginalBytes = if ($ResumePreparedInstallation) {
        [IO.File]::ReadAllBytes((Assert-Physical "$repo/.jenkins/state/management/install-history/$ExpectedInstalledSha256.config.xml"))
    } else { $taskRetainedApiBytes }
    Assert-PreviousManagementConfig $taskRetainedOriginalBytes $ExpectedPreviousJobConfigSha256
    [xml]$taskRetainedOriginalJob = ([Text.Encoding]::UTF8.GetString($taskRetainedOriginalBytes) -replace '^\s*<\?xml[^?]*\?>', '')
    Assert-ManagementJob $taskRetainedOriginalJob ($taskCombinedExtensionPrefix + $taskRetainedOldCore) $taskOldInstalled.driverDigest $taskOldInstalled.runtimeOperationId
    $taskRetainedObservedRuntime = if ($ResumePreparedInstallation) { $ExpectedRuntimeOperationId } else { $taskOldInstalled.runtimeOperationId }
    $taskRetainedObservedDriver = if ($ResumePreparedInstallation) { $ExpectedDriverDigest } else { $taskOldInstalled.driverDigest }
    Assert-ManagementJob $taskRetainedApiJob ($taskCombinedExtensionPrefix + $taskRetainedObservedCore) $taskRetainedObservedDriver $taskRetainedObservedRuntime
    Assert-ManagementJob $taskRetainedHomeJob ($taskCombinedExtensionPrefix + $taskRetainedObservedCore) $taskRetainedObservedDriver $taskRetainedObservedRuntime
    $taskRetainedPrefix = [ordered]@{mode='observed-prefix-v1';prefixSha256=$ExpectedRetainedPrefixSha256;
        oldRuntimeOperationId=$taskOldInstalled.runtimeOperationId;newRuntimeOperationId=$ExpectedRuntimeOperationId;
        oldDriverDigest=$taskOldInstalled.driverDigest;newDriverDigest=$ExpectedDriverDigest;
        reusedCapsuleSha256=$(if ($taskReuse) { $ReuseCapsuleSha256 } else { $null });
        prefixBytes=[Text.Encoding]::UTF8.GetByteCount($taskCombinedExtensionPrefix);
        originalConfigSha256=$ExpectedPreviousJobConfigSha256;oldCoreSha256=(Hash-Bytes ([Text.Encoding]::UTF8.GetBytes($taskRetainedOldCore)));
        apiSha256=(Hash-Bytes $taskRetainedApiBytes);homeSha256=(Hash-Bytes $taskRetainedHomeBytes);
        apiHomeScriptEqual=$true;liveVerified=$true;preserved=$false}
}
$pipeline = Join-ManagementJobExtension -Core $pipeline -Prefix $taskCombinedExtensionPrefix
$parameters = ''
foreach ($name in @('SESSION_ID','REQUEST_ID','OPERATION_ID','ACTION','INPUT_JSON','EXPECTED_DRIVER_DIGEST','RUNTIME_OPERATION_ID')) {
    $default = if ($name -eq 'EXPECTED_DRIVER_DIGEST') { $ExpectedDriverDigest } elseif ($name -eq 'RUNTIME_OPERATION_ID') { $ExpectedRuntimeOperationId } else { '' }
    $parameters += "<hudson.model.StringParameterDefinition><name>$name</name><defaultValue>$default</defaultValue><trim>true</trim></hudson.model.StringParameterDefinition>"
}
$parameters += '<hudson.model.BooleanParameterDefinition><name>QUERY_ONLY</name><defaultValue>false</defaultValue></hudson.model.BooleanParameterDefinition>'
$script = [Security.SecurityElement]::Escape($pipeline)
$xml = "<?xml version='1.0' encoding='UTF-8'?><flow-definition><description>Bounded Jenkins management MVP; no M8 acceptance</description><properties><hudson.model.ParametersDefinitionProperty><parameterDefinitions>$parameters</parameterDefinitions></hudson.model.ParametersDefinitionProperty></properties><definition class='org.jenkinsci.plugins.workflow.cps.CpsFlowDefinition' plugin='workflow-cps'><script>$script</script><sandbox>true</sandbox></definition><triggers/><disabled>false</disabled></flow-definition>"
if ($taskRetainedPrefixMode) { $xml = Set-ManagementJobPipeline -XmlText $taskRetainedResponse.Content -Pipeline $pipeline -PreviousRuntime $taskRetainedObservedRuntime -Runtime $ExpectedRuntimeOperationId -PreviousDriver $taskRetainedObservedDriver -Driver $ExpectedDriverDigest }
[xml]$parsed = ($xml -replace '^\s*<\?xml[^?]*\?>', '')
if ($CheckOnly) {
    Assert-ReviewedCandidateSources
    Assert-ManagementClientBytes
    $taskCheckResult = [ordered]@{status='check-only';job='zircon-management';capsule=$capsule;capsuleSha256=$digest;
        driverDigest=$ExpectedDriverDigest;runtimeOperationId=$ExpectedRuntimeOperationId;ordinaryEntry=$ordinaryEntry;formalAcceptance=$false}
    if ($null -ne $taskManagedWorktree) { $taskCheckResult.managedWorktree = $taskManagedWorktree }
    if ($null -ne $taskCandidateSources) { $taskCheckResult.candidateSources = $taskCandidateSources }
    if ($MigrateCanonicalManagementClients) {
        $taskCheckResult.previousJobConfig = Get-CanonicalMigrationPreviousConfig
        $taskCheckResult.previousJobConfigVerified = $true
        $taskCheckResult.expectedPreviousJobConfigSha256 = $taskCheckResult.previousJobConfig.originalConfigSha256
        Assert-ReviewedCandidateSources
        Assert-ManagementClientBytes
    }
    if ($ExpectedPreviousCodexExe) {
        $taskCheckResult.codexExe = $codex
        $taskCheckResult.expectedPreviousCodexExe = $taskPreviousCodex
        $taskCheckResult.expectedPreviousJobConfigSha256 = $ExpectedPreviousJobConfigSha256
        if (-not $MigrateCanonicalManagementClients) { $taskCheckResult.previousJobConfigVerified = $false }
    }
    if ($taskPinnedActive) {
        $taskCheckResult.pinnedActiveUpdate = [ordered]@{mode='proof-required';proofPath=(Assert-Physical $PinnedActiveBuildProofPath);
            proofSha256=$ExpectedPinnedActiveBuildProofSha256;operatorSource=$taskGuardSource;
            operatorSha256=$ExpectedPinnedActiveGuardSha256;liveVerified=$false}
    }
    if ($taskRetainedPrefixMode) { $taskCheckResult.retainedPrefix = $taskRetainedPrefix; $taskCheckResult.previousJobConfigVerified = $true }
    if ($null -ne $taskJobExtension) { $taskCheckResult.jobExtension = $taskJobExtension }
    if ($null -ne $taskDeliveryExtension) {
        $taskCheckResult.deliveryExtension = $taskDeliveryExtension
        $taskCheckResult.combinedExtensionPrefixSha256 = Hash-Bytes ([Text.Encoding]::UTF8.GetBytes($taskCombinedExtensionPrefix))
    }
    if ($null -ne $taskBranchIntegrationExtension) {
        $taskCheckResult.branchIntegrationExtension = $taskBranchIntegrationExtension
        $taskCheckResult.combinedExtensionPrefixSha256 = Hash-Bytes ([Text.Encoding]::UTF8.GetBytes($taskCombinedExtensionPrefix))
    }
    if ($null -ne $taskBranchCleanupExtension) {
        $taskCheckResult.branchCleanupExtension = $taskBranchCleanupExtension
        $taskCheckResult.combinedExtensionPrefixSha256 = Hash-Bytes ([Text.Encoding]::UTF8.GetBytes($taskCombinedExtensionPrefix))
    }
    [pscustomobject]$taskCheckResult | ConvertTo-Json -Depth 8
    return
}
Assert-ReviewedCandidateSources
if (Test-Path -LiteralPath $capsule) {
    if ((Hash-Bytes ([IO.File]::ReadAllBytes("$capsule/capsule.json"))) -ne $digest) { throw 'Existing capsule differs' }
    foreach ($name in $files.Keys) {
        if ((Hash-Bytes ([IO.File]::ReadAllBytes("$capsule/$name"))) -ne $files[$name]) { throw 'Existing capsule code differs' }
    }
} else {
    [void][IO.Directory]::CreateDirectory($capsule)
    Assert-ReviewedCandidateSources
    foreach ($name in $files.Keys) { [IO.File]::Copy((Get-ManagementCapsuleSource $name),"$capsule/$name",$false) }
    if ($taskAnchoredClients) {
        foreach ($taskClientName in $clientSources.Keys) {
            $taskDestination = Assert-Physical "$capsule/clients/$taskClientName"
            [void][IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($taskDestination))
            $taskClientBytes = [IO.File]::ReadAllBytes((Get-ManagementClientSource $taskClientName))
            if ((Hash-Bytes $taskClientBytes) -cne $clientSources[$taskClientName]) { throw 'Pinned candidate client changed before publication' }
            [IO.File]::WriteAllBytes($taskDestination,$taskClientBytes)
        }
    }
    [IO.File]::WriteAllBytes("$capsule/capsule.json",$bytes)
}
Assert-ManagementClientBytes $capsule
$credentials = Get-Content -Raw -LiteralPath "$repo/.jenkins/state/credentials.json" | ConvertFrom-Json
$taskApiToken = if ($credentials.apiToken) { $credentials.apiToken } else { $credentials.password }
if ($credentials.authenticationMode -ne 'api-token' -or !$credentials.username -or !$taskApiToken) { throw 'Managed API token credentials required' }
$auth = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes("$($credentials.username):$taskApiToken"))
$headers = @{Authorization="Basic $auth"}
$base = $BaseUrl.TrimEnd('/')
$crumb = Invoke-RestMethod -Method Get -Uri "$base/crumbIssuer/api/json" -Headers $headers
$headers[$crumb.crumbRequestField] = $crumb.crumb
$uri = "$base/job/zircon-management/config.xml"
$taskControllerReady = Invoke-RestMethod -Uri ($base + '/api/json?tree=numExecutors,quietingDown') -Headers $headers -TimeoutSec 15
$taskNodesReady = Invoke-RestMethod -Uri ($base + '/computer/api/json?tree=computer[displayName,offline,temporarilyOffline]') -Headers $headers -TimeoutSec 15
$taskOnlyAgent = @($taskNodesReady.computer | Where-Object { $_.displayName -ne 'Built-In Node' })
if ($taskControllerReady.numExecutors -ne 0 -or $taskControllerReady.quietingDown -ne $false -or
        $taskOnlyAgent.Count -ne 1 -or $taskOnlyAgent[0].displayName -ne 'zircon-windows-agent' -or
        $taskOnlyAgent[0].offline -ne $false -or $taskOnlyAgent[0].temporarilyOffline -ne $false) { throw 'Selected runtime is not ready for installation' }
Assert-ManagementClientBytes $capsule
if ($taskPrior) {
    if ((Hash-Bytes ([IO.File]::ReadAllBytes($taskInstalledPath))) -ne $ExpectedInstalledSha256) { throw 'Original install changed before POST' }
    $taskSelectedNow = Get-Content -Raw -LiteralPath "$repo/.jenkins/state/deployment/driver.json" | ConvertFrom-Json
    $taskOperationNow = Get-Content -Raw -LiteralPath "$repo/.jenkins/state/deployment-operation.json" | ConvertFrom-Json
    if ($taskSelectedNow.driverDigest -ne $ExpectedDriverDigest -or $taskSelectedNow.runtimeOperationId -ne $ExpectedRuntimeOperationId -or
            $taskOperationNow.operationId -ne $ExpectedRuntimeOperationId -or $taskOperationNow.state -ne 'running') { throw 'Selected runtime changed before POST' }
    $taskController = Invoke-RestMethod -Uri ($base + '/api/json?tree=numExecutors,quietingDown') -Headers $headers -TimeoutSec 15
    $taskNodes = Invoke-RestMethod -Uri ($base + '/computer/api/json?tree=computer[displayName,offline,temporarilyOffline]') -Headers $headers -TimeoutSec 15
    $taskFixed = @($taskNodes.computer | Where-Object { $_.displayName -ne 'Built-In Node' })
    if ($taskController.numExecutors -ne 0 -or $taskController.quietingDown -ne $false -or $taskFixed.Count -ne 1 -or
            $taskFixed[0].displayName -ne 'zircon-windows-agent' -or $taskFixed[0].offline -ne $false -or $taskFixed[0].temporarilyOffline -ne $false) { throw 'Selected runtime is not ready' }
    if ($taskPinnedActive) {
        Invoke-PinnedActiveManagementGuard 'before-post'
    } else {
        $taskRuns = Invoke-RestMethod -Uri ($base + '/job/zircon-management/api/json?tree=builds[building]') -Headers $headers -TimeoutSec 15
        if (@($taskRuns.builds | Where-Object { $_.building }).Count) { throw 'Management builds must finish before rebinding' }
        $taskQueue = Invoke-RestMethod -Uri ($base + '/queue/api/json?tree=items[task[name]]') -Headers $headers -TimeoutSec 15
        if (@($taskQueue.items | Where-Object { $_.task.name -eq 'zircon-management' }).Count) { throw 'Queued management work requires reconciliation' }
    }
    Assert-ManagementJobExtensionSource $taskJobExtension $taskJobExtensionPrefix
Assert-ManagementJobExtensionSource $taskDeliveryExtension $taskDeliveryExtensionPrefix -Kind 'delivery'
Assert-ManagementJobExtensionSource $taskBranchIntegrationExtension $taskBranchIntegrationExtensionPrefix -Kind 'branch-integration'
Assert-ManagementJobExtensionSource $taskBranchCleanupExtension $taskBranchCleanupExtensionPrefix -Kind 'branch-cleanup'
    $taskOldResponse = Invoke-WebRequest -Uri $uri -Headers $headers -UseBasicParsing -TimeoutSec 15
    [xml]$taskOldJob = ($taskOldResponse.Content -replace '^\s*<\?xml[^?]*\?>', '')
    $taskOldConfigBytes = [Text.Encoding]::UTF8.GetBytes($taskOldResponse.Content)
    if ($ResumePreparedInstallation) {
        if ($taskOldJob.'flow-definition'.definition.script -cne $pipeline -or $taskOldJob.'flow-definition'.definition.sandbox -ne 'true') { throw 'Prepared POST outcome is not the exact new job' }
        $taskPreparedRecord = [ordered]@{schemaVersion=1;job='zircon-management';capsule=$capsule;capsuleSha256=$digest;
            driverDigest=$ExpectedDriverDigest;runtimeOperationId=$ExpectedRuntimeOperationId;ordinaryEntry=$ordinaryEntry;formalAcceptance=$false}
        if ($null -ne $taskManagedWorktree) { $taskPreparedRecord.managedWorktree = $taskManagedWorktree }
        $taskPreparedBytes = [Text.UTF8Encoding]::new($false).GetBytes(($taskPreparedRecord | ConvertTo-Json -Depth 5 -Compress))
        $taskPreparedPath = Assert-Physical "$repo/.jenkins/state/management/installed.json.startup.tmp"
        if ((Hash-Bytes ([IO.File]::ReadAllBytes($taskPreparedPath))) -ne (Hash-Bytes $taskPreparedBytes)) { throw 'Prepared installation bytes differ' }
        $taskHistoryOriginal = Assert-Physical "$repo/.jenkins/state/management/install-history/$ExpectedInstalledSha256.json"
        $taskHistoryConfig = Assert-Physical "$repo/.jenkins/state/management/install-history/$ExpectedInstalledSha256.config.xml"
        if ((Hash-Bytes ([IO.File]::ReadAllBytes($taskHistoryOriginal))) -ne $ExpectedInstalledSha256) { throw 'Original resume history differs' }
        $taskOldConfigBytes = [IO.File]::ReadAllBytes($taskHistoryConfig)
        [xml]$taskOldJob = ([Text.Encoding]::UTF8.GetString($taskOldConfigBytes) -replace '^\s*<\?xml[^?]*\?>', '')
        $taskPostedAlready = $true
    }
    Assert-PreviousManagementConfig $taskOldConfigBytes $ExpectedPreviousJobConfigSha256
    $taskOldPipeline = [IO.File]::ReadAllText((Assert-Physical "$taskOldSourceRoot/zircon-management.groovy")).Replace(([string][char]13 + [char]10), [string][char]10)
    $taskOldTokens = @{AGENT='zircon-windows-agent';REPOSITORY=$repo;PYTHON=$python;CODEX=$taskPreviousCodex;
        CAPSULE=$taskOldSourceRoot;DRIVER=$taskOldInstalled.driverDigest;RUNTIME=$taskOldInstalled.runtimeOperationId}
    foreach ($taskKey in $taskOldTokens.Keys) {
        $taskOldValue = $taskOldTokens[$taskKey].Replace('\','/').Replace("'","\'")
        $taskOldPipeline = $taskOldPipeline.Replace("@@$taskKey@@", $taskOldValue)
    }
    $taskOldPipeline = Join-ManagementJobExtension -Core $taskOldPipeline -Prefix $taskCombinedExtensionPrefix
    Assert-ManagementJob $taskOldJob $taskOldPipeline $taskOldInstalled.driverDigest $taskOldInstalled.runtimeOperationId
    $taskHistory = Assert-Physical "$repo/.jenkins/state/management/install-history"
    [void][IO.Directory]::CreateDirectory($taskHistory)
    foreach ($taskHistoryItem in @(@{path="$taskHistory/$ExpectedInstalledSha256.json";bytes=$taskOldInstalledBytes},
            @{path="$taskHistory/$ExpectedInstalledSha256.config.xml";bytes=$taskOldConfigBytes})) {
        if (Test-Path -LiteralPath $taskHistoryItem.path) {
            if ((Hash-Bytes ([IO.File]::ReadAllBytes($taskHistoryItem.path))) -ne (Hash-Bytes $taskHistoryItem.bytes)) { throw 'Original history differs' }
        } else { [IO.File]::WriteAllBytes($taskHistoryItem.path, $taskHistoryItem.bytes) }
    }
} else {
    try {
        $null = Invoke-WebRequest -Uri $uri -Headers $headers -UseBasicParsing -TimeoutSec 15
        throw 'Existing management job has no exactly pinned installation'
    } catch {
        if ($null -eq $_.Exception.Response -or [int]$_.Exception.Response.StatusCode -ne 404) { throw }
    }
}
Assert-ReviewedCandidateSources
$taskSelectedFinal = Get-Content -Raw -LiteralPath "$repo/.jenkins/state/deployment/driver.json" | ConvertFrom-Json
$taskOperationFinal = Get-Content -Raw -LiteralPath "$repo/.jenkins/state/deployment-operation.json" | ConvertFrom-Json
if ($taskSelectedFinal.driverDigest -ne $ExpectedDriverDigest -or $taskSelectedFinal.runtimeOperationId -ne $ExpectedRuntimeOperationId -or
        $taskOperationFinal.operationId -ne $ExpectedRuntimeOperationId -or $taskOperationFinal.state -ne 'running') { throw 'Selected runtime changed before POST' }
foreach ($taskName in $files.Keys) {
    if ((Hash-Bytes ([IO.File]::ReadAllBytes((Assert-Physical "$capsule/$taskName")))) -ne $files[$taskName]) { throw 'Candidate capsule changed before POST' }
}
if ((Hash-Bytes ([IO.File]::ReadAllBytes((Assert-Physical "$capsule/capsule.json")))) -ne $digest) { throw 'Candidate metadata changed before POST' }
Assert-ManagementClientBytes $capsule
if ($taskPrior) {
    if ((Hash-Bytes ([IO.File]::ReadAllBytes($taskInstalledPath))) -ne $ExpectedInstalledSha256) { throw 'Original install changed before POST' }
    if (-not $taskPinnedActive) {
        $taskFinalRuns = Invoke-RestMethod -Uri ($base + '/job/zircon-management/api/json?tree=builds[building]') -Headers $headers -TimeoutSec 15
        if (@($taskFinalRuns.builds | Where-Object { $_.building }).Count) { throw 'Management build started during installation' }
    }
    $taskFinalResponse = Invoke-WebRequest -Uri $uri -Headers $headers -UseBasicParsing -TimeoutSec 15
    [xml]$taskFinalJob = ($taskFinalResponse.Content -replace '^\s*<\?xml[^?]*\?>', '')
    $taskExpectedPriorPipeline = if ($taskPostedAlready) { $pipeline } else { $taskOldPipeline }
    $taskPriorDriver = if ($taskPostedAlready) { $ExpectedDriverDigest } else { $taskOldInstalled.driverDigest }
    $taskPriorRuntime = if ($taskPostedAlready) { $ExpectedRuntimeOperationId } else { $taskOldInstalled.runtimeOperationId }
    Assert-ManagementJob $taskFinalJob $taskExpectedPriorPipeline $taskPriorDriver $taskPriorRuntime
    if ($taskRetainedPrefixMode) {
        [xml]$taskFinalHomeJob = ([Text.Encoding]::UTF8.GetString([IO.File]::ReadAllBytes((Assert-Physical "$repo/.jenkins/jenkins_home/jobs/zircon-management/config.xml"))) -replace '^\s*<\?xml[^?]*\?>', '')
        $taskFinalBareCore = if ($taskPostedAlready) { $taskBarePipeline } else { $taskRetainedOldCore }
        $taskFinalPrefix = Get-ManagementJobRetainedPrefix -ApiScript ([string]$taskFinalJob.'flow-definition'.definition.script) -HomeScript ([string]$taskFinalHomeJob.'flow-definition'.definition.script) -BareCore $taskFinalBareCore -ExpectedSha256 $ExpectedRetainedPrefixSha256
        if ($taskFinalPrefix -cne $taskCombinedExtensionPrefix) { throw 'Retained management prefix changed before POST' }
        Assert-ManagementJob $taskFinalHomeJob $taskExpectedPriorPipeline $taskPriorDriver $taskPriorRuntime
    }
    # Resumed POSTs expose the new job; only the archive proves their original.
    # Fresh updates must still observe the exact reviewed old response bytes.
    $taskFinalPreviousConfigBytes = if ($taskPostedAlready) {
        [IO.File]::ReadAllBytes($taskHistoryConfig)
    } else {
        [Text.Encoding]::UTF8.GetBytes($taskFinalResponse.Content)
    }
    Assert-PreviousManagementConfig $taskFinalPreviousConfigBytes $ExpectedPreviousJobConfigSha256
} elseif (Test-Path -LiteralPath $taskInstalledPath) { throw 'Installation appeared during creation' }
if ($taskPinnedActive) {
    Invoke-PinnedActiveManagementGuard 'before-post'
} else {
    $taskFinalQueue = Invoke-RestMethod -Uri ($base + '/queue/api/json?tree=items[task[name]]') -Headers $headers -TimeoutSec 15
    if (@($taskFinalQueue.items | Where-Object { $_.task.name -eq 'zircon-management' }).Count) { throw 'Queued management work requires reconciliation' }
}
Assert-ManagementJobExtensionSource $taskJobExtension $taskJobExtensionPrefix
Assert-ManagementJobExtensionSource $taskDeliveryExtension $taskDeliveryExtensionPrefix -Kind 'delivery'
Assert-ManagementJobExtensionSource $taskBranchIntegrationExtension $taskBranchIntegrationExtensionPrefix -Kind 'branch-integration'
Assert-ManagementJobExtensionSource $taskBranchCleanupExtension $taskBranchCleanupExtensionPrefix -Kind 'branch-cleanup'
$taskActivationRecord = [ordered]@{schemaVersion=1;job='zircon-management';capsule=$capsule;capsuleSha256=$digest;
    driverDigest=$ExpectedDriverDigest;runtimeOperationId=$ExpectedRuntimeOperationId;ordinaryEntry=$ordinaryEntry;formalAcceptance=$false}
if ($null -ne $taskManagedWorktree) { $taskActivationRecord.managedWorktree = $taskManagedWorktree }
$taskActivationBytes = [Text.UTF8Encoding]::new($false).GetBytes(($taskActivationRecord | ConvertTo-Json -Depth 5 -Compress))
$taskPreparedPath = Assert-Physical "$repo/.jenkins/state/management/installed.json.startup.tmp"
if (Test-Path -LiteralPath $taskPreparedPath) {
    if (-not $ResumePreparedInstallation -or (Hash-Bytes ([IO.File]::ReadAllBytes($taskPreparedPath))) -ne (Hash-Bytes $taskActivationBytes)) { throw 'Prepared install exists; reconcile its exact original POST' }
} else {
    if ($ResumePreparedInstallation) { throw 'Original prepared installation is absent' }
    [IO.File]::WriteAllBytes($taskPreparedPath, $taskActivationBytes)
}
if (-not $taskPostedAlready) {
try {
    Invoke-WebRequest -UseBasicParsing -Method Post -Uri $uri -Headers $headers -ContentType 'application/xml; charset=utf-8' -Body ([Text.Encoding]::UTF8.GetBytes($xml)) | Out-Null
} catch {
    if ($taskPrior -or $null -eq $_.Exception.Response -or [int]$_.Exception.Response.StatusCode -ne 404) { throw }
    Invoke-WebRequest -UseBasicParsing -Method Post -Uri "$base/createItem?name=zircon-management" -Headers $headers -ContentType 'application/xml; charset=utf-8' -Body ([Text.Encoding]::UTF8.GetBytes($xml)) | Out-Null
}
}
$taskUpdatedResponse = Invoke-WebRequest -Uri $uri -Headers $headers -UseBasicParsing -TimeoutSec 15
[xml]$taskUpdatedJob = ($taskUpdatedResponse.Content -replace '^\s*<\?xml[^?]*\?>', '')
Assert-ManagementJob $taskUpdatedJob $pipeline $ExpectedDriverDigest $ExpectedRuntimeOperationId
Assert-ReviewedCandidateSources
if ($taskRetainedPrefixMode) {
    [xml]$taskUpdatedHomeJob = ([Text.Encoding]::UTF8.GetString([IO.File]::ReadAllBytes((Assert-Physical "$repo/.jenkins/jenkins_home/jobs/zircon-management/config.xml"))) -replace '^\s*<\?xml[^?]*\?>', '')
    $taskUpdatedPrefix = Get-ManagementJobRetainedPrefix -ApiScript ([string]$taskUpdatedJob.'flow-definition'.definition.script) -HomeScript ([string]$taskUpdatedHomeJob.'flow-definition'.definition.script) -BareCore $taskBarePipeline -ExpectedSha256 $ExpectedRetainedPrefixSha256
    if ($taskUpdatedPrefix -cne $taskCombinedExtensionPrefix) { throw 'Retained management prefix changed after POST' }
    Assert-ManagementJob $taskUpdatedHomeJob $pipeline $ExpectedDriverDigest $ExpectedRuntimeOperationId
    $taskRetainedPrefix.preserved = $true
    $taskRetainedPrefix["apiAfterSha256"] = Hash-Bytes ([Text.Encoding]::UTF8.GetBytes($taskUpdatedResponse.Content))
    $taskRetainedPrefix["homeAfterSha256"] = Hash-Bytes ([IO.File]::ReadAllBytes((Assert-Physical "$repo/.jenkins/jenkins_home/jobs/zircon-management/config.xml")))
}
Assert-ManagementJobExtensionSource $taskJobExtension $taskJobExtensionPrefix
Assert-ManagementJobExtensionSource $taskDeliveryExtension $taskDeliveryExtensionPrefix -Kind 'delivery'
Assert-ManagementJobExtensionSource $taskBranchIntegrationExtension $taskBranchIntegrationExtensionPrefix -Kind 'branch-integration'
Assert-ManagementJobExtensionSource $taskBranchCleanupExtension $taskBranchCleanupExtensionPrefix -Kind 'branch-cleanup'
if ($taskPinnedActive) { Invoke-PinnedActiveManagementGuard 'after-post' }
if ($taskPrior -and (Hash-Bytes ([IO.File]::ReadAllBytes($taskInstalledPath))) -ne $ExpectedInstalledSha256) { throw 'Install changed while updating job' }
if (-not $taskPrior -and (Test-Path -LiteralPath $taskInstalledPath)) { throw 'Installation appeared while updating job' }
Assert-ManagementClientBytes $capsule
$taskSelectedAfter = Get-Content -Raw -LiteralPath "$repo/.jenkins/state/deployment/driver.json" | ConvertFrom-Json
$taskOperationAfter = Get-Content -Raw -LiteralPath "$repo/.jenkins/state/deployment-operation.json" | ConvertFrom-Json
if ($taskSelectedAfter.driverDigest -ne $ExpectedDriverDigest -or $taskSelectedAfter.runtimeOperationId -ne $ExpectedRuntimeOperationId -or
        $taskOperationAfter.operationId -ne $ExpectedRuntimeOperationId -or $taskOperationAfter.state -ne 'running') { throw 'Selected runtime changed after POST' }
$record = $taskActivationRecord
$folder = Assert-Physical "$repo/.jenkins/state/management"
[void][IO.Directory]::CreateDirectory($folder)
$taskRecordTemp = "$folder/installed.json.startup.tmp"
if ((Hash-Bytes ([IO.File]::ReadAllBytes($taskRecordTemp))) -ne (Hash-Bytes $taskActivationBytes)) { throw 'Prepared installation bytes changed before activation' }
if ($taskPrior -and (Hash-Bytes ([IO.File]::ReadAllBytes($taskInstalledPath))) -ne $ExpectedInstalledSha256) { throw 'Original install changed before activation' }
if (Test-Path -LiteralPath "$folder/installed.json") { [IO.File]::Replace($taskRecordTemp, "$folder/installed.json", [Management.Automation.Language.NullString]::Value) }
else { [IO.File]::Move($taskRecordTemp, "$folder/installed.json") }
if ($taskPinnedActive -or $taskRetainedPrefixMode -or $null -ne $taskJobExtension -or $null -ne $taskDeliveryExtension -or $null -ne $taskBranchIntegrationExtension -or $null -ne $taskBranchCleanupExtension -or $null -ne $taskCandidateSources) {
    $taskResult = [ordered]@{}
    foreach ($taskResultKey in $record.Keys) { $taskResult[$taskResultKey] = $record[$taskResultKey] }
    if ($taskPinnedActive) {
        $taskResult.pinnedActiveUpdate = [ordered]@{proofSha256=$ExpectedPinnedActiveBuildProofSha256;
            operatorSha256=$ExpectedPinnedActiveGuardSha256;observations=@($script:taskPinnedGuardReceipts)}
    }
    if ($null -ne $taskJobExtension) {
        $taskResult.jobExtension = [ordered]@{}
        foreach ($taskExtensionKey in $taskJobExtension.Keys) { $taskResult.jobExtension[$taskExtensionKey] = $taskJobExtension[$taskExtensionKey] }
        $taskResult.jobExtension.liveVerified = $true
        $taskResult.jobExtension.preserved = $true
    }
    if ($null -ne $taskDeliveryExtension) {
        $taskResult.deliveryExtension = [ordered]@{}
        foreach ($taskDeliveryKey in $taskDeliveryExtension.Keys) { $taskResult.deliveryExtension[$taskDeliveryKey] = $taskDeliveryExtension[$taskDeliveryKey] }
        $taskResult.deliveryExtension.liveVerified = $true
        $taskResult.deliveryExtension.preserved = $true
        $taskResult.combinedExtensionPrefixSha256 = Hash-Bytes ([Text.Encoding]::UTF8.GetBytes($taskCombinedExtensionPrefix))
    }
    if ($null -ne $taskBranchIntegrationExtension) {
        $taskResult.branchIntegrationExtension = [ordered]@{}
        foreach ($taskBranchIntegrationKey in $taskBranchIntegrationExtension.Keys) { $taskResult.branchIntegrationExtension[$taskBranchIntegrationKey] = $taskBranchIntegrationExtension[$taskBranchIntegrationKey] }
        $taskResult.branchIntegrationExtension.liveVerified = $true
        $taskResult.branchIntegrationExtension.preserved = $true
        $taskResult.combinedExtensionPrefixSha256 = Hash-Bytes ([Text.Encoding]::UTF8.GetBytes($taskCombinedExtensionPrefix))
    }
    if ($null -ne $taskBranchCleanupExtension) {
        $taskResult.branchCleanupExtension = [ordered]@{}
        foreach ($taskBranchCleanupKey in $taskBranchCleanupExtension.Keys) { $taskResult.branchCleanupExtension[$taskBranchCleanupKey] = $taskBranchCleanupExtension[$taskBranchCleanupKey] }
        $taskResult.branchCleanupExtension.liveVerified = $true
        $taskResult.branchCleanupExtension.preserved = $true
        $taskResult.combinedExtensionPrefixSha256 = Hash-Bytes ([Text.Encoding]::UTF8.GetBytes($taskCombinedExtensionPrefix))
    }
    if ($taskRetainedPrefixMode) { $taskResult.retainedPrefix = $taskRetainedPrefix }
    if ($null -ne $taskCandidateSources) { $taskResult.candidateSources = $taskCandidateSources }
    $taskResult | ConvertTo-Json -Depth 10
} else { $record | ConvertTo-Json }

} finally { if ($null -ne $taskInstallLock) { $taskInstallLock.Dispose() } }
