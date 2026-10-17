[CmdletBinding()]
param(
    [string]$RepoRoot = 'E:/Git/ZirconEngine',
    [Parameter(Mandatory = $true)][ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedDriverDigest,
    [Parameter(Mandatory = $true)][ValidatePattern('^[A-Za-z0-9_.-]+$')][string]$ExpectedRuntimeOperationId,
    [string]$SessionId = 'jenkins-scheduled-cleanup-20261011',
    [ValidatePattern('^http://127[.]0[.]0[.]1:[0-9]+/?$')][string]$BaseUrl = 'http://127.0.0.1:18080',
    [Parameter(Mandatory = $true)][string]$UserName,
    [Parameter(Mandatory = $true)][string]$Token,
    [bool]$DryRun = $true
)
$ErrorActionPreference = 'Stop'
$taskRepo = [IO.Path]::GetFullPath($RepoRoot).TrimEnd('\', '/')
if ($taskRepo -ne 'E:\Git\ZirconEngine') { throw 'Scheduled cleanup requires the configured repository' }
if ($SessionId -notmatch '^[A-Za-z0-9][A-Za-z0-9_.:-]{0,127}$') { throw 'Invalid maintenance session identity' }
$taskSelected = Get-Content -LiteralPath (Join-Path $taskRepo '.jenkins/state/deployment/driver.json') -Raw | ConvertFrom-Json
if ($taskSelected.driverDigest -ne $ExpectedDriverDigest -or $taskSelected.runtimeOperationId -ne $ExpectedRuntimeOperationId) {
    throw 'Selected driver/runtime differs from installation request'
}
$taskDriverRoot = [IO.Path]::GetFullPath((Join-Path $taskRepo ('.jenkins/runtime/drivers/' + $ExpectedDriverDigest)))
$taskSteps = [IO.Path]::GetFullPath((Join-Path $taskDriverRoot 'pipeline/steps.groovy'))
$taskLauncher = [IO.Path]::GetFullPath((Join-Path $taskDriverRoot 'launcher.py'))
if ($taskSelected.root -ne $taskDriverRoot -or $taskSelected.steps -ne $taskSteps -or $taskSelected.launcher -ne $taskLauncher) {
    throw 'Selected sealed driver paths differ from the configured runtime'
}
$taskSourcePath = Join-Path $taskRepo 'tools/jenkins-management/zircon-scheduled-cleanup.groovy'
$taskCacheSourcePath = Join-Path $taskRepo 'tools/jenkins-management/incremental-cache-cleanup.groovy'
$taskBindingPath = Join-Path $taskRepo 'tools/dev/jenkins_cleanup_runtime.py'
$taskPythonPath = Join-Path $taskRepo '.jenkins/runtime/python/python.exe'
foreach ($taskInput in @($taskSourcePath, $taskCacheSourcePath, $taskBindingPath, $taskPythonPath, $taskSteps, $taskLauncher)) {
    if (-not (Test-Path -LiteralPath $taskInput -PathType Leaf)) { throw 'Scheduled cleanup implementation is missing' }
    $taskCursor = $taskInput
    while ($taskCursor) {
        if ((Get-Item -LiteralPath $taskCursor -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Scheduled cleanup input contains an alias' }
        $taskCursor = [IO.Path]::GetDirectoryName($taskCursor)
    }
}
$taskManifest = Get-Content -LiteralPath (Join-Path $taskDriverRoot 'driver-manifest.json') -Raw | ConvertFrom-Json
foreach ($taskBinding in @(@{name = 'pipeline/steps.groovy'; path = $taskSteps}, @{name = 'launcher.py'; path = $taskLauncher})) {
    $taskEntry = $taskManifest.files.PSObject.Properties[$taskBinding.name].Value
    if ($null -eq $taskEntry -or (Get-FileHash -LiteralPath $taskBinding.path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $taskEntry.sha256) {
        throw 'Selected sealed maintenance implementation hash differs'
    }
}
$taskSource = Get-Content -LiteralPath $taskSourcePath -Raw
$taskSource = (Get-Content -LiteralPath $taskCacheSourcePath -Raw) + [Environment]::NewLine + $taskSource
$taskSourceDigest = (Get-FileHash -LiteralPath $taskSourcePath -Algorithm SHA256).Hash.ToLowerInvariant()
$taskEnvironment = [ordered]@{
    ZIRCON_REPO_ROOT = $taskRepo
    ZIRCON_REPOSITORY_ID = ''
    JENKINS_PYTHON = [IO.Path]::GetFullPath((Join-Path $taskRepo '.jenkins/runtime/python/python.exe'))
    ZIRCON_SEALED_DRIVER = $taskSteps
    ZIRCON_DRIVER_LAUNCHER = $taskLauncher
    ZIRCON_DRIVER_DIGEST = $ExpectedDriverDigest
    ZIRCON_RUNTIME_OPERATION_ID = $ExpectedRuntimeOperationId
    ZIRCON_MAINTENANCE_SESSION = $SessionId
    ZIRCON_AGENT_LABEL = 'zircon-windows-agent'
    ZIRCON_CLEANUP_BINDING_SHA256 = (Get-FileHash -LiteralPath $taskBindingPath -Algorithm SHA256).Hash.ToLowerInvariant()
    ZIRCON_CLEANUP_PYTHON_SHA256 = (Get-FileHash -LiteralPath $taskPythonPath -Algorithm SHA256).Hash.ToLowerInvariant()
    PYTHONDONTWRITEBYTECODE = '1'
}
# Repository identity matches the default CLI: contracts.digest(casefolded absolute root).
$taskSha = [Security.Cryptography.SHA256]::Create()
try {
    $taskEnvironment.ZIRCON_REPOSITORY_ID = ([BitConverter]::ToString($taskSha.ComputeHash([Text.Encoding]::UTF8.GetBytes((ConvertTo-Json -InputObject ($taskRepo.ToLowerInvariant()) -Compress))))).Replace('-', '').ToLowerInvariant()
} finally { $taskSha.Dispose() }
$taskPrelude = ($taskEnvironment.GetEnumerator() | ForEach-Object {
    'env.' + $_.Key + '=' + (ConvertTo-Json -InputObject ([string]$_.Value) -Compress) + ';'
}) -join [Environment]::NewLine
function Escape-Xml([string]$Text) { return [Security.SecurityElement]::Escape($Text) }
function Read-JenkinsJobXml([string]$Text) {
    # Jenkins serializes XML 1.1; .NET accepts the same declaration as 1.0.
    # Strict XML parsing still rejects any unsupported content.
    try { return [xml]($Text.Replace('<?xml version="1.1"', '<?xml version="1.0"')) }
    catch { throw 'Cannot parse Jenkins job configuration XML' }
}
$taskDryRunText = $DryRun.ToString().ToLowerInvariant()
$taskXml = @"
<?xml version='1.0' encoding='UTF-8'?>
<flow-definition plugin='workflow-job'>
  <actions/><description>Bounded scheduled managed CAS cleanup; implementation $taskSourceDigest</description>
  <keepDependencies>false</keepDependencies>
  <properties>
    <org.jenkinsci.plugins.workflow.job.properties.DisableConcurrentBuildsJobProperty/>
    <hudson.model.ParametersDefinitionProperty><parameterDefinitions>
      <hudson.model.BooleanParameterDefinition><name>DRY_RUN</name><description>Inventory and plan without deletion</description><defaultValue>$taskDryRunText</defaultValue></hudson.model.BooleanParameterDefinition>
    </parameterDefinitions></hudson.model.ParametersDefinitionProperty>
    <org.jenkinsci.plugins.workflow.job.properties.PipelineTriggersJobProperty><triggers>
      <hudson.triggers.TimerTrigger><spec>H/30 * * * *</spec></hudson.triggers.TimerTrigger>
    </triggers></org.jenkinsci.plugins.workflow.job.properties.PipelineTriggersJobProperty>
  </properties>
  <definition class='org.jenkinsci.plugins.workflow.cps.CpsFlowDefinition' plugin='workflow-cps'>
    <script>$(Escape-Xml ($taskPrelude + [Environment]::NewLine + $taskSource))</script><sandbox>true</sandbox>
  </definition>
  <disabled>false</disabled>
</flow-definition>
"@
[xml]$taskParsed = $taskXml
$taskHeaders = @{ Authorization = 'Basic ' + [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($UserName + ':' + $Token)) }
$taskBase = $BaseUrl.TrimEnd('/')
try {
    $taskCrumb = Invoke-RestMethod -Uri ($taskBase + '/crumbIssuer/api/json') -Headers $taskHeaders -TimeoutSec 15
    if ($taskCrumb.crumbRequestField -and $taskCrumb.crumb) { $taskHeaders[$taskCrumb.crumbRequestField] = $taskCrumb.crumb }
} catch {
    if ($null -eq $_.Exception.Response -or [int]$_.Exception.Response.StatusCode -ne 404) { throw 'Cannot obtain Jenkins installation crumb' }
}
$taskJob = 'zircon-scheduled-cleanup'
$taskConfigUrl = $taskBase + '/job/' + $taskJob + '/config.xml'
$taskExists = $true
$taskPreserveGovernance = $false
try {
    $taskPrevious = Invoke-WebRequest -Uri $taskConfigUrl -Method Get -Headers $taskHeaders -UseBasicParsing -TimeoutSec 15
} catch {
    if ($null -eq $_.Exception.Response -or [int]$_.Exception.Response.StatusCode -ne 404) { throw 'Cannot inspect the scheduled cleanup job' }
    $taskExists = $false
}
if ($taskExists) {
    $taskPreviousDocument = Read-JenkinsJobXml ([string]$taskPrevious.Content)
    $taskPreserveGovernance = ([string]$taskPreviousDocument.'flow-definition'.definition.script).Contains('// ZIRCON HOST GOVERNANCE AUDIT')
    $taskCurrent = Invoke-WebRequest -Uri $taskConfigUrl -Method Get -Headers $taskHeaders -UseBasicParsing -TimeoutSec 15
    if ([string]$taskCurrent.Content -cne [string]$taskPrevious.Content) { throw 'Scheduled cleanup changed during installation review' }
}
$taskPostUrl = if ($taskExists) { $taskConfigUrl } else { $taskBase + '/createItem?name=' + $taskJob }
try {
    $null = Invoke-WebRequest -Uri $taskPostUrl -Method Post -Headers $taskHeaders -ContentType 'application/xml; charset=utf-8' -Body ([Text.Encoding]::UTF8.GetBytes($taskXml)) -UseBasicParsing -TimeoutSec 15
} catch { throw 'Jenkins scheduled cleanup job installation failed' }
# Rebinding the selected runtime must retain the existing governance stage.
if ($taskPreserveGovernance) {
    Push-Location -LiteralPath $taskRepo
    try {
        & $taskEnvironment.JENKINS_PYTHON -B -X utf8 -m tools.dev.jenkins_governance --repo-root $taskRepo attach-maintenance
        if ($LASTEXITCODE -ne 0) { throw 'Scheduled cleanup was rebound but its governance stage could not be restored' }
    } finally { Pop-Location }
}
$taskReadback = Invoke-WebRequest -Uri $taskConfigUrl -Method Get -Headers $taskHeaders -UseBasicParsing -TimeoutSec 15
$taskInstalledDocument = Read-JenkinsJobXml ([string]$taskReadback.Content)
$taskInstalledScript = [string]$taskInstalledDocument.'flow-definition'.definition.script
# Keep the cache result visible when a retained governance hook writes the final description.
$taskDisplayScript = $taskInstalledScript.Replace("currentBuild.description = report.status +", "currentBuild.description = 'compiler cache ' + (env.ZIRCON_INCREMENTAL_DELETED_BYTES ?: '0') + ' logical bytes; ' + report.status +")
if ($taskDisplayScript -cne $taskInstalledScript) {
    $taskInstalledDocument.'flow-definition'.definition.script = $taskDisplayScript
    $null = Invoke-WebRequest -Uri $taskConfigUrl -Method Post -Headers $taskHeaders -ContentType 'application/xml; charset=utf-8' -Body ([Text.Encoding]::UTF8.GetBytes($taskInstalledDocument.OuterXml)) -UseBasicParsing -TimeoutSec 15
    $taskReadback = Invoke-WebRequest -Uri $taskConfigUrl -Method Get -Headers $taskHeaders -UseBasicParsing -TimeoutSec 15
    $taskInstalledDocument = Read-JenkinsJobXml ([string]$taskReadback.Content)
    $taskInstalledScript = [string]$taskInstalledDocument.'flow-definition'.definition.script
    if ($taskInstalledScript -cne $taskDisplayScript) { throw 'Scheduled cleanup description readback differs' }
}
$taskExpectedScript = ($taskPrelude + [Environment]::NewLine + $taskSource).Replace("`r`n", "`n").TrimEnd()
if (-not $taskInstalledScript.Replace("`r`n", "`n").StartsWith($taskExpectedScript) -or
    ($taskPreserveGovernance -and -not $taskInstalledScript.Contains('// ZIRCON HOST GOVERNANCE AUDIT'))) {
    throw 'Scheduled cleanup installation readback differs from the reviewed source and governance stage'
}
[ordered]@{
    job = $taskJob; node = 'zircon-windows-agent'; schedule = 'H/30 * * * *'; dryRun = $DryRun
    implementationDigest = $taskSourceDigest; driverDigest = $ExpectedDriverDigest
    runtimeOperationId = $ExpectedRuntimeOperationId; maintenanceSession = $SessionId
    maxObjectsPerRun = 64; maxBytesPerRun = 268435456; controllerRestartRequired = $false
    incrementalCacheMaxBytesPerRun = 34359738368L; incrementalCacheTriggerFreeBytes = 85899345920L
    governanceStagePreserved = $taskPreserveGovernance; readbackVerified = $true
    runtimeBinding = 'same reviewed driver; fresh native owners per new run; exact runtime fence thereafter'
    runtimeBindingImplementationSha256 = $taskEnvironment.ZIRCON_CLEANUP_BINDING_SHA256
} | ConvertTo-Json -Depth 4
