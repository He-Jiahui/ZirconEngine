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
    [ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedEntrySourceSha256
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
$repo = Assert-Physical $RepoRoot
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
if ($ResumePreparedInstallation -and -not $taskReuse) { throw 'Resume requires exact original reuse pins' }
if ($ExpectedEntrySourceSha256 -and -not $taskReuse) { throw 'Reviewed entry pin requires the original capsule and installed hashes' }
if ($taskReuse -ne (-not [string]::IsNullOrEmpty($ExpectedInstalledSha256))) { throw 'Reuse requires both original capsule and installed hashes' }
$taskInstalledPath = Assert-Physical "$repo/.jenkins/state/management/installed.json"
$taskSourceRoot = "$repo/tools/jenkins-management"
if ($taskReuse) {
    $taskOldInstalledBytes = [IO.File]::ReadAllBytes($taskInstalledPath)
    if ((Hash-Bytes $taskOldInstalledBytes) -ne $ExpectedInstalledSha256) { throw 'Original installed record changed' }
    $taskOldInstalled = [Text.Encoding]::UTF8.GetString($taskOldInstalledBytes) | ConvertFrom-Json
    $taskSourceRoot = Assert-Physical "$repo/.jenkins/runtime/management/$ReuseCapsuleSha256"
    if ($taskOldInstalled.job -ne 'zircon-management' -or $taskOldInstalled.capsuleSha256 -ne $ReuseCapsuleSha256 -or $taskOldInstalled.capsule -ne $taskSourceRoot) { throw 'Original capsule owner differs' }
    $taskOldMetadataBytes = [IO.File]::ReadAllBytes((Assert-Physical "$taskSourceRoot/capsule.json"))
    if ((Hash-Bytes $taskOldMetadataBytes) -ne $ReuseCapsuleSha256) { throw 'Original capsule metadata changed' }
    $taskOldMetadata = [Text.Encoding]::UTF8.GetString($taskOldMetadataBytes) | ConvertFrom-Json
    if ($taskOldMetadata.schemaVersion -ne 1 -or $taskOldMetadata.repositoryRoot -ne $repo -or
            $taskOldMetadata.agentLabel -ne 'zircon-windows-agent' -or
            $taskOldMetadata.driverDigest -ne $taskOldInstalled.driverDigest -or
            $taskOldMetadata.runtimeOperationId -ne $taskOldInstalled.runtimeOperationId -or
            $taskOldInstalled.formalAcceptance -ne $false) { throw 'Original capsule contract differs' }
    if ((@($taskOldMetadata.files.PSObject.Properties.Name | Sort-Object) -join '|') -cne 'runner.py|source_preparation.py|zircon-management.groovy') { throw 'Original capsule inventory differs' }
}
$files = [ordered]@{}
foreach ($name in @('runner.py','source_preparation.py','zircon-management.groovy')) {
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
    if ($taskReuse) {
        $taskOldClient = $taskOldMetadata.clientSources.PSObject.Properties[$clientName]
        if ($null -eq $taskOldClient) { throw 'Original public entry inventory differs' }
        if ($clientSources[$clientName] -ne $taskOldClient.Value) {
            # Only this public client can change under an explicit reviewed raw-byte pin.
            $taskNormalized = [Text.Encoding]::UTF8.GetString($taskClientBytes).Replace(([string][char]13 + [char]10), [string][char]10)
            $taskReviewedEntry = $clientName -eq 'tools/jenkins-management/entry.py' -and
                $ExpectedEntrySourceSha256 -and $clientSources[$clientName] -eq $ExpectedEntrySourceSha256
            if ($clientName -ne 'tools/jenkins-management/entry.py' -or (-not $taskReviewedEntry -and
                    (Hash-Bytes ([Text.Encoding]::UTF8.GetBytes($taskNormalized))) -ne $taskOldClient.Value)) { throw 'Unreviewed public entry change' }
        }
    }
}
if ($taskReuse -and @($taskOldMetadata.clientSources.PSObject.Properties).Count -ne $clientSources.Count) { throw 'Original public entry inventory differs' }
$ordinaryEntry = [ordered]@{enabled=$true;actions=@('validate');formalAcceptance=$false;m8Accepted=$false}
if ($taskReuse -and (($taskOldMetadata.ordinaryEntry | ConvertTo-Json -Compress) -cne ($ordinaryEntry | ConvertTo-Json -Compress) -or
        ($taskOldInstalled.ordinaryEntry | ConvertTo-Json -Compress) -cne ($ordinaryEntry | ConvertTo-Json -Compress))) { throw 'Original capability differs' }
$metadata = [ordered]@{schemaVersion=1;repositoryRoot=$repo;driverDigest=$ExpectedDriverDigest;
    runtimeOperationId=$ExpectedRuntimeOperationId;agentLabel='zircon-windows-agent';files=$files;clientSources=$clientSources;ordinaryEntry=$ordinaryEntry}
$bytes = [Text.UTF8Encoding]::new($false).GetBytes(($metadata | ConvertTo-Json -Depth 12 -Compress))
$digest = Hash-Bytes $bytes
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
foreach ($taskClientName in $clientSources.Keys) {
    if ((Hash-Bytes ([IO.File]::ReadAllBytes((Assert-Physical "$repo/$taskClientName")))) -ne $clientSources[$taskClientName]) { throw 'Public entry changed before POST' }
}
if ($taskReuse) {
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
    $taskOldPipeline = $taskPipelineTemplate
    $taskOldTokens = @{AGENT='zircon-windows-agent';REPOSITORY=$repo;PYTHON=$python;CODEX=$codex;
        CAPSULE=$taskSourceRoot;DRIVER=$taskOldInstalled.driverDigest;RUNTIME=$taskOldInstalled.runtimeOperationId}
    foreach ($taskKey in $taskOldTokens.Keys) {
        $taskOldValue = $taskOldTokens[$taskKey].Replace('\','/').Replace("'","\'")
        $taskOldPipeline = $taskOldPipeline.Replace("@@$taskKey@@", $taskOldValue)
    }
    if ($taskOldJob.'flow-definition'.definition.script -cne $taskOldPipeline -or
            $taskOldJob.'flow-definition'.definition.sandbox -ne 'true') { throw 'Original job owner or script changed' }
    $taskHistory = Assert-Physical "$repo/.jenkins/state/management/install-history"
    [void][IO.Directory]::CreateDirectory($taskHistory)
    foreach ($taskHistoryItem in @(@{path="$taskHistory/$ExpectedInstalledSha256.json";bytes=$taskOldInstalledBytes},
            @{path="$taskHistory/$ExpectedInstalledSha256.config.xml";bytes=$taskOldConfigBytes})) {
        if (Test-Path -LiteralPath $taskHistoryItem.path) {
            if ((Hash-Bytes ([IO.File]::ReadAllBytes($taskHistoryItem.path))) -ne (Hash-Bytes $taskHistoryItem.bytes)) { throw 'Original history differs' }
        } else { [IO.File]::WriteAllBytes($taskHistoryItem.path, $taskHistoryItem.bytes) }
    }
}
if (-not $taskPostedAlready) {
try {
    Invoke-WebRequest -UseBasicParsing -Method Post -Uri $uri -Headers $headers -ContentType 'application/xml; charset=utf-8' -Body ([Text.Encoding]::UTF8.GetBytes($xml)) | Out-Null
} catch {
    if ([int]$_.Exception.Response.StatusCode -ne 404) { throw }
    Invoke-WebRequest -UseBasicParsing -Method Post -Uri "$base/createItem?name=zircon-management" -Headers $headers -ContentType 'application/xml; charset=utf-8' -Body ([Text.Encoding]::UTF8.GetBytes($xml)) | Out-Null
}
}
if ($taskReuse) {
    $taskUpdatedResponse = Invoke-WebRequest -Uri $uri -Headers $headers -UseBasicParsing -TimeoutSec 15
    [xml]$taskUpdatedJob = ($taskUpdatedResponse.Content -replace '^\s*<\?xml[^?]*\?>', '')
    if ($taskUpdatedJob.'flow-definition'.definition.script -cne $pipeline -or $taskUpdatedJob.'flow-definition'.definition.sandbox -ne 'true') { throw 'Updated job requires reconciliation' }
    if ((Hash-Bytes ([IO.File]::ReadAllBytes($taskInstalledPath))) -ne $ExpectedInstalledSha256) { throw 'Install changed while updating job' }
}
foreach ($taskClientName in $clientSources.Keys) {
    if ((Hash-Bytes ([IO.File]::ReadAllBytes((Assert-Physical "$repo/$taskClientName")))) -ne $clientSources[$taskClientName]) { throw 'Public entry changed while updating job' }
}
$record = [ordered]@{schemaVersion=1;job='zircon-management';capsule=$capsule;capsuleSha256=$digest;
    driverDigest=$ExpectedDriverDigest;runtimeOperationId=$ExpectedRuntimeOperationId;ordinaryEntry=$ordinaryEntry;formalAcceptance=$false}
$folder = Assert-Physical "$repo/.jenkins/state/management"
[void][IO.Directory]::CreateDirectory($folder)
$taskRecordTemp = "$folder/installed.json.startup.tmp"
[IO.File]::WriteAllText($taskRecordTemp,($record | ConvertTo-Json -Compress),[Text.UTF8Encoding]::new($false))
if (Test-Path -LiteralPath "$folder/installed.json") { [IO.File]::Replace($taskRecordTemp, "$folder/installed.json", [Management.Automation.Language.NullString]::Value) }
else { [IO.File]::Move($taskRecordTemp, "$folder/installed.json") }
$record | ConvertTo-Json
