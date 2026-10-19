[CmdletBinding()]
param(
    [string]$RepoRoot = 'E:/Git/ZirconEngine',
    [Parameter(Mandatory=$true)][ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedDriverDigest,
    [Parameter(Mandatory=$true)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9_.-]{0,127}$')][string]$ExpectedRuntimeOperationId,
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
    [ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedRetainedPrefixSha256
)
$ErrorActionPreference = 'Stop'
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
        [ValidateSet('checkout-commit','delivery','branch-integration')][string]$Kind = 'checkout-commit'
    )
    $taskAddonLabel = if ($Kind -ceq 'branch-integration') { 'Branch integration addon' } elseif ($Kind -ceq 'delivery') { 'Validated delivery addon' } else { 'Checkout commit addon' }
    if ($ExpectedSha256 -cnotmatch '^[0-9a-f]{64}$' -or (Hash-Bytes $AddonBytes) -cne $ExpectedSha256) {
        throw ($taskAddonLabel + ' source differs')
    }
    $taskAddonText = [Text.UTF8Encoding]::new($false,$true).GetString($AddonBytes)
    if ($taskAddonText.StartsWith([string][char]0xfeff, [StringComparison]::Ordinal)) { $taskAddonText = $taskAddonText.Substring(1) }
    $taskAddonText = $taskAddonText.Replace("`r`n", "`n").TrimEnd([char[]]"`r`n")
    $taskBegin = if ($Kind -ceq 'branch-integration') { '// BEGIN ZIRCON BRANCH QUEUE' } elseif ($Kind -ceq 'delivery') { '// BEGIN ZIRCON VALIDATED DELIVERY' } else { '// BEGIN ZIRCON SCOPED CHECKOUT COMMIT' }
    $taskEnd = if ($Kind -ceq 'branch-integration') { '// END ZIRCON BRANCH QUEUE' } elseif ($Kind -ceq 'delivery') { '// END ZIRCON VALIDATED DELIVERY' } else { '// END ZIRCON SCOPED CHECKOUT COMMIT' }
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
    return ($Prefix + $Core)
}
function Assert-ManagementJobExtensionSource([object]$Extension, [string]$Prefix, [string]$Kind = 'checkout-commit') {
    if ($null -eq $Extension) { return }
    $taskExtensionNow = [IO.File]::ReadAllBytes((Assert-Physical $Extension.sourcePath))
    if ((Get-ManagementJobExtensionPrefix -AddonBytes $taskExtensionNow -ExpectedSha256 $Extension.sourceSha256 -Kind $Kind) -cne $Prefix) {
        throw 'Checkout commit addon changed during installation'
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
        [Parameter(Mandatory=$true)][AllowEmptyString()][string]$Pipeline
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

function Get-ReviewedCandidateSources {
    if ([string]::IsNullOrEmpty($CandidateSourceRoot)) {
        if ($CandidateSourceManifestPath -or $ExpectedCandidateSourceManifestSha256) { throw 'Candidate sources require their exact managed root' }
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
    if ((Assert-Physical "$PSScriptRoot/../..") -cne $taskCandidateRoot) { throw 'Candidate installer must execute from its original managed WT' }
    $taskCandidateBootstrap = @'
import hashlib,importlib.util,json,os,sys
from pathlib import Path
repo,source,manifest_path,manifest_sha,installed_sha,driver,runtime,capsule_sha=sys.argv[1:]
repo,source,manifest_path=Path(repo),Path(source),Path(manifest_path)
def require(value,message):
    if not value: raise ValueError(message)
def candidate_members(proof):
    allowed={'runner.py':'tools/jenkins-management/runner.py','source_preparation.py':'tools/jenkins-management/source_preparation.py'}
    files,scopes=proof.get('files'),proof.get('ownedPaths')
    require(isinstance(files,dict) and isinstance(scopes,list)
        and all(isinstance(scope,str) and scope for scope in scopes),'candidate_member_scope_invalid')
    groovy='tools/jenkins-management/zircon-management.groovy'
    if set(files)=={groovy}:
        require(set(scopes)=={groovy,'tools/dev/install-jenkins-management.ps1'},'candidate_member_scope_invalid')
        return {'zircon-management.groovy':groovy}
    require(1<=len(files)<=2 and set(files)<=set(allowed.values()),'candidate_member_scope_invalid')
    required=set(files)|{'tools/dev/install-jenkins-management.ps1'}
    require(all(any(name.casefold()==scope.casefold() or name.casefold().startswith(scope.rstrip('/').casefold()+'/')
        for scope in scopes)for name in required),'candidate_member_scope_invalid')
    return {name:relative for name,relative in allowed.items()if relative in files}
sys.path.insert(0,str(repo/'.jenkins/runtime/drivers'/driver))
from tools.jenkins.deployment.driver import verify_driver,_plain
verify_driver(repo/'.jenkins/runtime/drivers'/driver,expected_digest=driver)
require(os.name=='nt' and os.environ.get('JOB_NAME')=='zircon-management-tests'
    and os.environ.get('NODE_NAME')=='zircon-windows-agent','fixed_candidate_jenkins_job_required')
for path in (source,manifest_path,repo/'.jenkins/state/management/installed.json'): _plain(path)
require(source.parent==repo/'.jenkins/worktrees','registered_candidate_source_root_required')
require(manifest_path.is_relative_to(repo/'.jenkins/state/management/reports')
    or manifest_path.is_relative_to(Path('E:/cargo-targets/zircon-local/reports')),'candidate_manifest_root_rejected')
raw=manifest_path.read_bytes()
require(hashlib.sha256(raw).hexdigest()==manifest_sha,'candidate_manifest_changed')
proof=json.loads(raw)
require(isinstance(proof,dict) and set(proof)=={'schemaVersion','kind','repositoryId','worktreeId','sourceRoot',
    'writerSessionId','writerPrincipal','writerAuthorizationSnapshot','ownedPaths','files','installerSha256',
    'driverDigest','runtimeOperationId','originalInstalledSha256'}
    and proof.get('schemaVersion')==1 and proof.get('kind')=='jenkins-management-candidate-sources-v1'
    and proof.get('sourceRoot')==str(source) and proof.get('driverDigest')==driver
    and proof.get('runtimeOperationId')==runtime and proof.get('originalInstalledSha256')==installed_sha,
    'candidate_manifest_identity_invalid')
members=candidate_members(proof)
installed_raw=(repo/'.jenkins/state/management/installed.json').read_bytes()
require(hashlib.sha256(installed_raw).hexdigest()==installed_sha,'candidate_original_install_changed')
installed=json.loads(installed_raw)
require(installed.get('capsuleSha256')==capsule_sha and installed.get('driverDigest')==driver
    and installed.get('runtimeOperationId')==runtime,'candidate_original_binding_changed')
capsule=repo/'.jenkins/runtime/management'/capsule_sha
metadata_raw=(capsule/'capsule.json').read_bytes()
require(hashlib.sha256(metadata_raw).hexdigest()==capsule_sha,'original_capsule_changed')
metadata=json.loads(metadata_raw)
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
    pins={**proof['files'],'tools/dev/install-jenkins-management.ps1':proof['installerSha256']}
    for relative,checksum in pins.items():
        require(isinstance(checksum,str) and len(checksum)==64 and all(c in '0123456789abcdef' for c in checksum),
            'candidate_source_pin_invalid')
        path=source/relative
        _plain(path)
        require(path.is_file() and hashlib.sha256(path.read_bytes()).hexdigest()==checksum,'candidate_source_changed')
print(json.dumps({'manifestSha256':manifest_sha,'sourceRoot':str(source),'worktreeId':proof['worktreeId'],
    'writerAuthorizationSnapshot':proof['writerAuthorizationSnapshot'],'sourcePins':pins,
    'driverDigest':driver,'runtimeOperationId':runtime,'files':{name:str(source/relative) for name,relative in members.items()}}))
'@
    $taskCandidatePython = Assert-Physical "$repo/.jenkins/runtime/python/python.exe"
    $taskCandidateOutput = & $taskCandidatePython -B -X utf8 -c $taskCandidateBootstrap $repo $taskCandidateRoot $taskCandidateManifest $ExpectedCandidateSourceManifestSha256 $ExpectedInstalledSha256 $ExpectedDriverDigest $ExpectedRuntimeOperationId $taskOldCapsuleSha256
    if ($LASTEXITCODE -ne 0) { throw 'Jenkins candidate WT authority or source pins could not be verified' }
    return ($taskCandidateOutput | ConvertFrom-Json)
}
function Get-ManagementCapsuleSource([string]$Name) {
    if ($null -ne $taskCandidateSources -and
            $null -ne $taskCandidateSources.files.PSObject.Properties[$Name]) {
        return (Assert-Physical ([string]$taskCandidateSources.files.PSObject.Properties[$Name].Value))
    }
    return (Assert-Physical "$taskSourceRoot/$Name")
}
function Assert-ReviewedCandidateSources {
    if ($null -eq $taskCandidateSources) { return }
    $taskCandidateAgain = Get-ReviewedCandidateSources
    if (($taskCandidateAgain | ConvertTo-Json -Depth 8 -Compress) -cne
            ($taskCandidateSources | ConvertTo-Json -Depth 8 -Compress)) { throw 'Candidate source authority changed before installation' }
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
        $PSBoundParameters.ContainsKey('ExpectedBranchIntegrationAddonSha256'))) {
    throw 'Reviewed prefix preservation requires an original config pin and the ordinary idle gate'
}
$script:taskPinnedGuardReceipts = @()
if ($taskPinnedActive) {
    if (-not $taskPrior -or $taskReuse -or [string]::IsNullOrEmpty($ExpectedPreviousJobConfigSha256) -or
            [string]::IsNullOrEmpty($PinnedActiveBuildProofPath)) { throw 'Pinned-active update requires an exact existing installation and a fresh candidate' }
    $taskGuardSource = Assert-Physical "$repo/tools/jenkins-management/install_guard.py"
    if ((Hash-Bytes ([IO.File]::ReadAllBytes($taskGuardSource))) -cne $ExpectedPinnedActiveGuardSha256) { throw 'Pinned-active operator source changed' }
    if ((Hash-Bytes ([IO.File]::ReadAllBytes((Assert-Physical $PinnedActiveBuildProofPath)))) -cne $ExpectedPinnedActiveBuildProofSha256) { throw 'Pinned-active proof bytes changed' }
}
$taskJobExtension = $null
$taskJobExtensionPrefix = ''
if ($PSBoundParameters.ContainsKey('ExpectedCheckoutCommitAddonSha256')) {
    if (-not $taskPrior) { throw 'Checkout commit addon preservation requires an existing installation' }
    $taskExtensionSource = Assert-Physical "$repo/tools/jenkins-management/checkout-commit.groovy"
    $taskExtensionBytes = [IO.File]::ReadAllBytes($taskExtensionSource)
    $taskJobExtensionPrefix = Get-ManagementJobExtensionPrefix -AddonBytes $taskExtensionBytes -ExpectedSha256 $ExpectedCheckoutCommitAddonSha256
    $taskJobExtension = [ordered]@{kind='scoped-checkout-commit-v1';sourcePath=$taskExtensionSource;
        sourceSha256=$ExpectedCheckoutCommitAddonSha256;prefixSha256=(Hash-Bytes ([Text.Encoding]::UTF8.GetBytes($taskJobExtensionPrefix)));
        capsuleMember=$false;liveVerified=$false;preserved=$false}
}
$taskDeliveryExtension = $null
$taskDeliveryExtensionPrefix = ''
if ($PSBoundParameters.ContainsKey('ExpectedDeliveryAddonSha256')) {
    if (-not $taskPrior) { throw 'Validated delivery addon preservation requires an existing installation' }
    $taskDeliverySource = Assert-Physical "$repo/tools/jenkins-management/delivery.groovy"
    $taskDeliveryBytes = [IO.File]::ReadAllBytes($taskDeliverySource)
    $taskDeliveryExtensionPrefix = Get-ManagementJobExtensionPrefix -AddonBytes $taskDeliveryBytes -ExpectedSha256 $ExpectedDeliveryAddonSha256 -Kind 'delivery'
    $taskDeliveryExtension = [ordered]@{kind='validated-delivery-v1';sourcePath=$taskDeliverySource;
        sourceSha256=$ExpectedDeliveryAddonSha256;prefixSha256=(Hash-Bytes ([Text.Encoding]::UTF8.GetBytes($taskDeliveryExtensionPrefix)));
        capsuleMember=$false;liveVerified=$false;preserved=$false}
}
$taskBranchIntegrationExtension = $null
$taskBranchIntegrationExtensionPrefix = ''
if ($PSBoundParameters.ContainsKey('ExpectedBranchIntegrationAddonSha256')) {
    if (-not $taskPrior) { throw 'Branch integration addon preservation requires an existing installation' }
    $taskBranchIntegrationSource = Assert-Physical "$repo/tools/jenkins-management/branch-integration.groovy"
    $taskBranchIntegrationBytes = [IO.File]::ReadAllBytes($taskBranchIntegrationSource)
    $taskBranchIntegrationExtensionPrefix = Get-ManagementJobExtensionPrefix -AddonBytes $taskBranchIntegrationBytes -ExpectedSha256 $ExpectedBranchIntegrationAddonSha256 -Kind 'branch-integration'
    $taskBranchIntegrationExtension = [ordered]@{kind='branch-integration-v1';sourcePath=$taskBranchIntegrationSource;
        sourceSha256=$ExpectedBranchIntegrationAddonSha256;prefixSha256=(Hash-Bytes ([Text.Encoding]::UTF8.GetBytes($taskBranchIntegrationExtensionPrefix)));
        capsuleMember=$false;liveVerified=$false;preserved=$false}
}
$taskCombinedExtensionPrefix = $taskBranchIntegrationExtensionPrefix + $taskDeliveryExtensionPrefix + $taskJobExtensionPrefix
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
$taskCandidateSources = Get-ReviewedCandidateSources
if ($null -ne $taskCandidateSources) { $taskSourceRoot = $taskOldSourceRoot }
$files = [ordered]@{}
$taskCapsuleNames = if ($taskReuse -or $null -ne $taskCandidateSources) { @($taskOldMetadata.files.PSObject.Properties.Name) } else { @('runner.py','source_preparation.py','zircon-management.groovy','checkout_guard.py') }
if (-not $taskReuse -and $null -eq $taskCandidateSources -and $EnableCheckoutProduct) { $taskCapsuleNames += 'checkout_product.py' }
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
    $taskClientBytes = [IO.File]::ReadAllBytes((Assert-Physical "$repo/$clientName"))
    $taskOldClientHash = $null
    if ($taskPrior) {
        $taskOldClient = $taskOldMetadata.clientSources.PSObject.Properties[$clientName]
        if ($null -eq $taskOldClient) { throw 'Original public entry inventory differs' }
        $taskOldClientHash = [string]$taskOldClient.Value
    }
    $taskClientArguments = @{ClientName=$clientName;ClientBytes=$taskClientBytes;OriginalSha256=$taskOldClientHash}
    $clientSources[$clientName] = Get-ValidatedManagementClientSha256 @taskClientArguments @taskClientReviewedPins
}
if ($taskPrior -and @($taskOldMetadata.clientSources.PSObject.Properties).Count -ne $clientSources.Count) { throw 'Original public entry inventory differs' }
$ordinaryEntry = [ordered]@{enabled=$true;actions=@('validate');formalAcceptance=$false;m8Accepted=$false}
if ($taskPrior -and (($taskOldMetadata.ordinaryEntry | ConvertTo-Json -Compress) -cne ($ordinaryEntry | ConvertTo-Json -Compress) -or
        ($taskOldInstalled.ordinaryEntry | ConvertTo-Json -Compress) -cne ($ordinaryEntry | ConvertTo-Json -Compress))) { throw 'Original capability differs' }
$metadata = [ordered]@{schemaVersion=1;repositoryRoot=$repo;driverDigest=$ExpectedDriverDigest;
    runtimeOperationId=$ExpectedRuntimeOperationId;agentLabel='zircon-windows-agent';files=$files;clientSources=$clientSources;ordinaryEntry=$ordinaryEntry}
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
    if ($taskOldInstalled.driverDigest -cne $ExpectedDriverDigest -or
            $taskOldInstalled.runtimeOperationId -cne $ExpectedRuntimeOperationId) {
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
    Assert-ManagementJob $taskRetainedApiJob ($taskCombinedExtensionPrefix + $taskRetainedObservedCore) $ExpectedDriverDigest $ExpectedRuntimeOperationId
    Assert-ManagementJob $taskRetainedHomeJob ($taskCombinedExtensionPrefix + $taskRetainedObservedCore) $ExpectedDriverDigest $ExpectedRuntimeOperationId
    $taskRetainedPrefix = [ordered]@{mode='observed-prefix-v1';prefixSha256=$ExpectedRetainedPrefixSha256;
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
if ($taskRetainedPrefixMode) { $xml = Set-ManagementJobPipeline -XmlText $taskRetainedResponse.Content -Pipeline $pipeline }
[xml]$parsed = ($xml -replace '^\s*<\?xml[^?]*\?>', '')
if ($CheckOnly) {
    $taskCheckResult = [ordered]@{status='check-only';job='zircon-management';capsule=$capsule;capsuleSha256=$digest;
        driverDigest=$ExpectedDriverDigest;runtimeOperationId=$ExpectedRuntimeOperationId;ordinaryEntry=$ordinaryEntry;formalAcceptance=$false}
    if ($null -ne $taskManagedWorktree) { $taskCheckResult.managedWorktree = $taskManagedWorktree }
    if ($null -ne $taskCandidateSources) { $taskCheckResult.candidateSources = $taskCandidateSources }
    if ($ExpectedPreviousCodexExe) {
        $taskCheckResult.codexExe = $codex
        $taskCheckResult.expectedPreviousCodexExe = $taskPreviousCodex
        $taskCheckResult.expectedPreviousJobConfigSha256 = $ExpectedPreviousJobConfigSha256
        $taskCheckResult.previousJobConfigVerified = $false
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
    [pscustomobject]$taskCheckResult | ConvertTo-Json -Depth 8
    return
}
if (Test-Path -LiteralPath $capsule) {
    if ((Hash-Bytes ([IO.File]::ReadAllBytes("$capsule/capsule.json"))) -ne $digest) { throw 'Existing capsule differs' }
    foreach ($name in $files.Keys) {
        if ((Hash-Bytes ([IO.File]::ReadAllBytes("$capsule/$name"))) -ne $files[$name]) { throw 'Existing capsule code differs' }
    }
} else {
    [void][IO.Directory]::CreateDirectory($capsule)
    Assert-ReviewedCandidateSources
    foreach ($name in $files.Keys) { [IO.File]::Copy((Get-ManagementCapsuleSource $name),"$capsule/$name",$false) }
    [IO.File]::WriteAllBytes("$capsule/capsule.json",$bytes)
}
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
foreach ($taskClientName in $clientSources.Keys) {
    if ((Hash-Bytes ([IO.File]::ReadAllBytes((Assert-Physical "$repo/$taskClientName")))) -ne $clientSources[$taskClientName]) { throw 'Public entry changed before POST' }
}
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
foreach ($taskClientName in $clientSources.Keys) {
    if ((Hash-Bytes ([IO.File]::ReadAllBytes((Assert-Physical "$repo/$taskClientName")))) -ne $clientSources[$taskClientName]) { throw 'Public entry changed before POST' }
}
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
if ($taskPinnedActive) { Invoke-PinnedActiveManagementGuard 'after-post' }
if ($taskPrior -and (Hash-Bytes ([IO.File]::ReadAllBytes($taskInstalledPath))) -ne $ExpectedInstalledSha256) { throw 'Install changed while updating job' }
if (-not $taskPrior -and (Test-Path -LiteralPath $taskInstalledPath)) { throw 'Installation appeared while updating job' }
foreach ($taskClientName in $clientSources.Keys) {
    if ((Hash-Bytes ([IO.File]::ReadAllBytes((Assert-Physical "$repo/$taskClientName")))) -ne $clientSources[$taskClientName]) { throw 'Public entry changed while updating job' }
}
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
if ($taskPinnedActive -or $taskRetainedPrefixMode -or $null -ne $taskJobExtension -or $null -ne $taskDeliveryExtension -or $null -ne $taskBranchIntegrationExtension -or $null -ne $taskCandidateSources) {
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
    if ($taskRetainedPrefixMode) { $taskResult.retainedPrefix = $taskRetainedPrefix }
    if ($null -ne $taskCandidateSources) { $taskResult.candidateSources = $taskCandidateSources }
    $taskResult | ConvertTo-Json -Depth 10
} else { $record | ConvertTo-Json }

} finally { if ($null -ne $taskInstallLock) { $taskInstallLock.Dispose() } }
