[CmdletBinding()]
param(
    [string]$RepoRoot = 'E:/Git/ZirconEngine',
    [Parameter(Mandatory=$true)][ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedDriverDigest,
    [Parameter(Mandatory=$true)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9_.-]{0,127}$')][string]$ExpectedRuntimeOperationId,
    [string]$BaseUrl = 'http://127.0.0.1:18080',
    [string]$CodexExe = 'C:/Users/HeJiahui/AppData/Local/OpenAI/Codex/bin/2e5e00daee91c61d/codex.exe',
    [switch]$CheckOnly,
    [ValidatePattern('^[0-9a-f]{64}$')][string]$ReuseCapsuleSha256,
    [ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedInstalledSha256,
    [switch]$ResumePreparedInstallation,
    [ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedEntrySourceSha256,
    [ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedLocalCargoSourceSha256,
    [ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedCapsuleSha256
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
    if ($taskOldInventory -cnotin @('runner.py|source_preparation.py|zircon-management.groovy','checkout_guard.py|runner.py|source_preparation.py|zircon-management.groovy')) { throw 'Original capsule inventory differs' }
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
$files = [ordered]@{}
$taskCapsuleNames = if ($taskReuse) { @($taskOldMetadata.files.PSObject.Properties.Name) } else { @('runner.py','source_preparation.py','zircon-management.groovy','checkout_guard.py') }
foreach ($name in $taskCapsuleNames) {
    $source = Assert-Physical "$taskSourceRoot/$name"
    $files[$name] = Hash-Bytes ([IO.File]::ReadAllBytes($source))
    if ($taskReuse -and $files[$name] -ne $taskOldMetadata.files.PSObject.Properties[$name].Value) { throw 'Original capsule file changed' }
}
$clientSources = [ordered]@{}
foreach ($clientName in @('tools/dev/local_cargo.py','tools/dev/local-cargo.ps1','tools/dev/dev-fast-build.ps1',
        'tools/dev/submit-jenkins-management.ps1','tools/jenkins/frontend.py','tools/jenkins-management/entry.py')) {
    $taskClientBytes = [IO.File]::ReadAllBytes((Assert-Physical "$repo/$clientName"))
    $clientSources[$clientName] = Hash-Bytes $taskClientBytes
    if ($clientName -eq 'tools/jenkins-management/entry.py' -and $ExpectedEntrySourceSha256 -and
            $clientSources[$clientName] -ne $ExpectedEntrySourceSha256) { throw 'Reviewed public entry pin differs' }
    if ($clientName -eq 'tools/dev/local_cargo.py' -and $ExpectedLocalCargoSourceSha256 -and
            $clientSources[$clientName] -ne $ExpectedLocalCargoSourceSha256) { throw 'Reviewed native helper pin differs' }
    if ($taskPrior) {
        $taskOldClient = $taskOldMetadata.clientSources.PSObject.Properties[$clientName]
        if ($null -eq $taskOldClient) { throw 'Original public entry inventory differs' }
        if ($clientSources[$clientName] -ne $taskOldClient.Value) {
            # Changed public clients require an explicit reviewed raw-byte pin.
            $taskNormalized = [Text.Encoding]::UTF8.GetString($taskClientBytes).Replace(([string][char]13 + [char]10), [string][char]10)
            $taskReviewedEntry = $clientName -eq 'tools/jenkins-management/entry.py' -and
                $ExpectedEntrySourceSha256 -and $clientSources[$clientName] -eq $ExpectedEntrySourceSha256
            $taskReviewedNative = $clientName -eq 'tools/dev/local_cargo.py' -and
                $ExpectedLocalCargoSourceSha256 -and $clientSources[$clientName] -eq $ExpectedLocalCargoSourceSha256
            if (-not $taskReviewedNative -and ($clientName -ne 'tools/jenkins-management/entry.py' -or (-not $taskReviewedEntry -and
                    (Hash-Bytes ([Text.Encoding]::UTF8.GetBytes($taskNormalized))) -ne $taskOldClient.Value))) { throw 'Unreviewed public entry change' }
        }
    }
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
$bytes = [Text.UTF8Encoding]::new($false).GetBytes(($metadata | ConvertTo-Json -Depth 12 -Compress))
$digest = Hash-Bytes $bytes
if ($ExpectedCapsuleSha256 -and $ExpectedCapsuleSha256 -ne $digest) { throw 'Reviewed candidate capsule differs' }
if (-not $CheckOnly -and [string]::IsNullOrEmpty($ExpectedCapsuleSha256)) { throw 'Installation requires the exact reviewed candidate capsule hash from CheckOnly' }
$capsule = Assert-Physical "$repo/.jenkins/runtime/management/$digest"
$python = Assert-Physical "$repo/.jenkins/runtime/python/python.exe"
$codex = Assert-Physical $CodexExe
if (!(Test-Path -LiteralPath $python) -or !(Test-Path -LiteralPath $codex)) { throw 'Managed Python or official Codex sandbox executable missing' }
$pipeline = [IO.File]::ReadAllText((Assert-Physical "$taskSourceRoot/zircon-management.groovy")).Replace(([string][char]13 + [char]10), [string][char]10)
$taskPipelineTemplate = $pipeline
$tokens = @{AGENT='zircon-windows-agent';REPOSITORY=$repo;PYTHON=$python;CODEX=$codex;
    CAPSULE=$capsule;DRIVER=$ExpectedDriverDigest;RUNTIME=$ExpectedRuntimeOperationId}
foreach ($key in $tokens.Keys) {
    $value = $tokens[$key].Replace('\','/').Replace("'","\'")
    if ($value.Contains([char]10) -or $value.Contains([char]13) -or $value.Contains('"')) { throw 'Unsafe fixed job path' }
    $pipeline = $pipeline.Replace("@@$key@@", $value)
}
if ($pipeline -match '@@[A-Z]+@@') { throw 'Unresolved capsule marker' }
$parameters = ''
foreach ($name in @('SESSION_ID','REQUEST_ID','OPERATION_ID','ACTION','INPUT_JSON','EXPECTED_DRIVER_DIGEST','RUNTIME_OPERATION_ID')) {
    $default = if ($name -eq 'EXPECTED_DRIVER_DIGEST') { $ExpectedDriverDigest } elseif ($name -eq 'RUNTIME_OPERATION_ID') { $ExpectedRuntimeOperationId } else { '' }
    $parameters += "<hudson.model.StringParameterDefinition><name>$name</name><defaultValue>$default</defaultValue><trim>true</trim></hudson.model.StringParameterDefinition>"
}
$parameters += '<hudson.model.BooleanParameterDefinition><name>QUERY_ONLY</name><defaultValue>false</defaultValue></hudson.model.BooleanParameterDefinition>'
$script = [Security.SecurityElement]::Escape($pipeline)
$xml = "<?xml version='1.0' encoding='UTF-8'?><flow-definition><description>Bounded Jenkins management MVP; no M8 acceptance</description><properties><hudson.model.ParametersDefinitionProperty><parameterDefinitions>$parameters</parameterDefinitions></hudson.model.ParametersDefinitionProperty></properties><definition class='org.jenkinsci.plugins.workflow.cps.CpsFlowDefinition' plugin='workflow-cps'><script>$script</script><sandbox>true</sandbox></definition><triggers/><disabled>false</disabled></flow-definition>"
[xml]$parsed = $xml
if ($CheckOnly) {
    [pscustomobject]@{status='check-only';job='zircon-management';capsule=$capsule;capsuleSha256=$digest;
        driverDigest=$ExpectedDriverDigest;runtimeOperationId=$ExpectedRuntimeOperationId;ordinaryEntry=$ordinaryEntry;formalAcceptance=$false} | ConvertTo-Json
    return
}
if (Test-Path -LiteralPath $capsule) {
    if ((Hash-Bytes ([IO.File]::ReadAllBytes("$capsule/capsule.json"))) -ne $digest) { throw 'Existing capsule differs' }
    foreach ($name in $files.Keys) {
        if ((Hash-Bytes ([IO.File]::ReadAllBytes("$capsule/$name"))) -ne $files[$name]) { throw 'Existing capsule code differs' }
    }
} else {
    [void][IO.Directory]::CreateDirectory($capsule)
    foreach ($name in $files.Keys) { [IO.File]::Copy("$taskSourceRoot/$name","$capsule/$name",$false) }
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
    $taskRuns = Invoke-RestMethod -Uri ($base + '/job/zircon-management/api/json?tree=builds[building]') -Headers $headers -TimeoutSec 15
    if (@($taskRuns.builds | Where-Object { $_.building }).Count) { throw 'Management builds must finish before rebinding' }
    $taskQueue = Invoke-RestMethod -Uri ($base + '/queue/api/json?tree=items[task[name]]') -Headers $headers -TimeoutSec 15
    if (@($taskQueue.items | Where-Object { $_.task.name -eq 'zircon-management' }).Count) { throw 'Queued management work requires reconciliation' }
    $taskOldResponse = Invoke-WebRequest -Uri $uri -Headers $headers -UseBasicParsing -TimeoutSec 15
    [xml]$taskOldJob = ($taskOldResponse.Content -replace '^\s*<\?xml[^?]*\?>', '')
    $taskOldConfigBytes = [Text.Encoding]::UTF8.GetBytes($taskOldResponse.Content)
    if ($ResumePreparedInstallation) {
        if ($taskOldJob.'flow-definition'.definition.script -cne $pipeline -or $taskOldJob.'flow-definition'.definition.sandbox -ne 'true') { throw 'Prepared POST outcome is not the exact new job' }
        $taskPreparedRecord = [ordered]@{schemaVersion=1;job='zircon-management';capsule=$capsule;capsuleSha256=$digest;
            driverDigest=$ExpectedDriverDigest;runtimeOperationId=$ExpectedRuntimeOperationId;ordinaryEntry=$ordinaryEntry;formalAcceptance=$false}
        $taskPreparedBytes = [Text.UTF8Encoding]::new($false).GetBytes(($taskPreparedRecord | ConvertTo-Json -Compress))
        $taskPreparedPath = Assert-Physical "$repo/.jenkins/state/management/installed.json.startup.tmp"
        if ((Hash-Bytes ([IO.File]::ReadAllBytes($taskPreparedPath))) -ne (Hash-Bytes $taskPreparedBytes)) { throw 'Prepared installation bytes differ' }
        $taskHistoryOriginal = Assert-Physical "$repo/.jenkins/state/management/install-history/$ExpectedInstalledSha256.json"
        $taskHistoryConfig = Assert-Physical "$repo/.jenkins/state/management/install-history/$ExpectedInstalledSha256.config.xml"
        if ((Hash-Bytes ([IO.File]::ReadAllBytes($taskHistoryOriginal))) -ne $ExpectedInstalledSha256) { throw 'Original resume history differs' }
        $taskOldConfigBytes = [IO.File]::ReadAllBytes($taskHistoryConfig)
        [xml]$taskOldJob = ([Text.Encoding]::UTF8.GetString($taskOldConfigBytes) -replace '^\s*<\?xml[^?]*\?>', '')
        $taskPostedAlready = $true
    }
    $taskOldPipeline = [IO.File]::ReadAllText((Assert-Physical "$taskOldSourceRoot/zircon-management.groovy")).Replace(([string][char]13 + [char]10), [string][char]10)
    $taskOldTokens = @{AGENT='zircon-windows-agent';REPOSITORY=$repo;PYTHON=$python;CODEX=$codex;
        CAPSULE=$taskOldSourceRoot;DRIVER=$taskOldInstalled.driverDigest;RUNTIME=$taskOldInstalled.runtimeOperationId}
    foreach ($taskKey in $taskOldTokens.Keys) {
        $taskOldValue = $taskOldTokens[$taskKey].Replace('\','/').Replace("'","\'")
        $taskOldPipeline = $taskOldPipeline.Replace("@@$taskKey@@", $taskOldValue)
    }
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
    $taskFinalRuns = Invoke-RestMethod -Uri ($base + '/job/zircon-management/api/json?tree=builds[building]') -Headers $headers -TimeoutSec 15
    if (@($taskFinalRuns.builds | Where-Object { $_.building }).Count) { throw 'Management build started during installation' }
    $taskFinalResponse = Invoke-WebRequest -Uri $uri -Headers $headers -UseBasicParsing -TimeoutSec 15
    [xml]$taskFinalJob = ($taskFinalResponse.Content -replace '^\s*<\?xml[^?]*\?>', '')
    $taskExpectedPriorPipeline = if ($taskPostedAlready) { $pipeline } else { $taskOldPipeline }
    $taskPriorDriver = if ($taskPostedAlready) { $ExpectedDriverDigest } else { $taskOldInstalled.driverDigest }
    $taskPriorRuntime = if ($taskPostedAlready) { $ExpectedRuntimeOperationId } else { $taskOldInstalled.runtimeOperationId }
    Assert-ManagementJob $taskFinalJob $taskExpectedPriorPipeline $taskPriorDriver $taskPriorRuntime
} elseif (Test-Path -LiteralPath $taskInstalledPath) { throw 'Installation appeared during creation' }
$taskFinalQueue = Invoke-RestMethod -Uri ($base + '/queue/api/json?tree=items[task[name]]') -Headers $headers -TimeoutSec 15
if (@($taskFinalQueue.items | Where-Object { $_.task.name -eq 'zircon-management' }).Count) { throw 'Queued management work requires reconciliation' }
$taskActivationRecord = [ordered]@{schemaVersion=1;job='zircon-management';capsule=$capsule;capsuleSha256=$digest;
    driverDigest=$ExpectedDriverDigest;runtimeOperationId=$ExpectedRuntimeOperationId;ordinaryEntry=$ordinaryEntry;formalAcceptance=$false}
$taskActivationBytes = [Text.UTF8Encoding]::new($false).GetBytes(($taskActivationRecord | ConvertTo-Json -Compress))
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
$record | ConvertTo-Json

} finally { if ($null -ne $taskInstallLock) { $taskInstallLock.Dispose() } }
