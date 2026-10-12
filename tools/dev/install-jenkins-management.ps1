[CmdletBinding()]
param(
    [string]$RepoRoot = 'E:/Git/ZirconEngine',
    [Parameter(Mandatory=$true)][ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedDriverDigest,
    [Parameter(Mandatory=$true)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9_.-]{0,127}$')][string]$ExpectedRuntimeOperationId,
    [string]$BaseUrl = 'http://127.0.0.1:18080',
    [string]$CodexExe = 'C:/Users/HeJiahui/AppData/Local/OpenAI/Codex/bin/2e5e00daee91c61d/codex.exe',
    [switch]$CheckOnly
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
$files = [ordered]@{}
foreach ($name in @('runner.py','source_preparation.py','zircon-management.groovy')) {
    $source = Assert-Physical "$repo/tools/jenkins-management/$name"
    $files[$name] = Hash-Bytes ([IO.File]::ReadAllBytes($source))
}
$clientSources = [ordered]@{}
foreach ($clientName in @('tools/dev/local_cargo.py','tools/dev/local-cargo.ps1','tools/dev/dev-fast-build.ps1',
        'tools/dev/submit-jenkins-management.ps1','tools/jenkins/frontend.py','tools/jenkins-management/entry.py')) {
    $clientSources[$clientName] = Hash-Bytes ([IO.File]::ReadAllBytes((Assert-Physical "$repo/$clientName")))
}
$ordinaryEntry = [ordered]@{enabled=$true;actions=@('validate');formalAcceptance=$false;m8Accepted=$false}
$metadata = [ordered]@{schemaVersion=1;repositoryRoot=$repo;driverDigest=$ExpectedDriverDigest;
    runtimeOperationId=$ExpectedRuntimeOperationId;agentLabel='zircon-windows-agent';files=$files;clientSources=$clientSources;ordinaryEntry=$ordinaryEntry}
$bytes = [Text.UTF8Encoding]::new($false).GetBytes(($metadata | ConvertTo-Json -Depth 12 -Compress))
$digest = Hash-Bytes $bytes
$capsule = Assert-Physical "$repo/.jenkins/runtime/management/$digest"
$python = Assert-Physical "$repo/.jenkins/runtime/python/python.exe"
$codex = Assert-Physical $CodexExe
if (!(Test-Path -LiteralPath $python) -or !(Test-Path -LiteralPath $codex)) { throw 'Managed Python or official Codex sandbox executable missing' }
$pipeline = Get-Content -Raw -LiteralPath "$repo/tools/jenkins-management/zircon-management.groovy"
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
    foreach ($name in $files.Keys) { [IO.File]::Copy("$repo/tools/jenkins-management/$name","$capsule/$name",$false) }
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
try {
    Invoke-WebRequest -UseBasicParsing -Method Post -Uri $uri -Headers $headers -ContentType 'application/xml; charset=utf-8' -Body ([Text.Encoding]::UTF8.GetBytes($xml)) | Out-Null
} catch {
    if ([int]$_.Exception.Response.StatusCode -ne 404) { throw }
    Invoke-WebRequest -UseBasicParsing -Method Post -Uri "$base/createItem?name=zircon-management" -Headers $headers -ContentType 'application/xml; charset=utf-8' -Body ([Text.Encoding]::UTF8.GetBytes($xml)) | Out-Null
}
$record = [ordered]@{schemaVersion=1;job='zircon-management';capsule=$capsule;capsuleSha256=$digest;
    driverDigest=$ExpectedDriverDigest;runtimeOperationId=$ExpectedRuntimeOperationId;ordinaryEntry=$ordinaryEntry;formalAcceptance=$false}
$folder = Assert-Physical "$repo/.jenkins/state/management"
[void][IO.Directory]::CreateDirectory($folder)
[IO.File]::WriteAllText("$folder/installed.json",($record | ConvertTo-Json -Compress),[Text.UTF8Encoding]::new($false))
$record | ConvertTo-Json
