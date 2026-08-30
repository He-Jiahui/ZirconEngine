[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string]$StagingRoot,
    [Parameter(Mandatory)]
    [string]$EvidenceRoot,
    [string]$ExpectedSourceFingerprint,
    [string]$ProfileContractSummaryPath,
    [string]$WorkspaceSummaryPath,
    [switch]$RequireProjectCreationEvidence,
    [switch]$RequireAuthoringAutomation,
    [switch]$RequireReopenAutomation,
    [switch]$RequireProductEvidence,
    [switch]$RequireF5Evidence,
    [switch]$Json
)

$ErrorActionPreference = 'Stop'
$WarningPreference = 'SilentlyContinue'

$driver = Join-Path $PSScriptRoot '..\mvp\Invoke-MvpAcceptance.ps1'
$treeManifestModule = Join-Path $PSScriptRoot '..\mvp\MvpAcceptanceStagingTreeManifest.psm1'
Import-Module $treeManifestModule -Force -ErrorAction Stop
# Test fixtures deliberately mutate isolated staging evidence between negative cases.
# Republish the producer handoff immediately before invoking the production consumer.
$stagingRootItem = Get-Item -LiteralPath $StagingRoot -Force -ErrorAction Stop
if (-not [bool]($stagingRootItem.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
    Write-MvpAcceptanceStagingTreeManifest -StagingRoot $StagingRoot | Out-Null
}
$childCommand = @'
$ErrorActionPreference = 'Stop'
$acceptancePath = [Environment]::GetEnvironmentVariable('ZIRCON_MVP_ACCEPTANCE_SCRIPT_PATH', 'Process')
$driverParametersJson = [Environment]::GetEnvironmentVariable('ZIRCON_MVP_ACCEPTANCE_DRIVER_PARAMETERS_JSON', 'Process')
$childPayloadPath = [Environment]::GetEnvironmentVariable('ZIRCON_MVP_ACCEPTANCE_CHILD_PAYLOAD_PATH', 'Process')
if ([string]::IsNullOrWhiteSpace($acceptancePath) -or
    [string]::IsNullOrWhiteSpace($driverParametersJson) -or
    [string]::IsNullOrWhiteSpace($childPayloadPath)) {
    throw 'MVP acceptance test driver did not receive a child driver invocation contract.'
}
$driverParameters = @{}
foreach ($property in ($driverParametersJson | ConvertFrom-Json -ErrorAction Stop).PSObject.Properties) {
    $driverParameters[$property.Name] = $property.Value
}
try {
    $childOutput = @(& $acceptancePath @driverParameters)
    [IO.File]::WriteAllText(
        $childPayloadPath,
        [Convert]::ToBase64String(
            [Text.UTF8Encoding]::new($false).GetBytes(
                (@($childOutput | ForEach-Object { $_.ToString() }) -join [Environment]::NewLine))),
        [Text.ASCIIEncoding]::new())
}
catch {
    [IO.File]::WriteAllText(
        $childPayloadPath,
        [Convert]::ToBase64String(
            [Text.UTF8Encoding]::new($false).GetBytes(
                (@{ error_message = $_.Exception.Message } | ConvertTo-Json -Compress))),
        [Text.ASCIIEncoding]::new())
    exit 1
}
'@
$driverParameters = [ordered]@{}
foreach ($entry in $PSBoundParameters.GetEnumerator()) {
    $driverParameters[$entry.Key] = if ($entry.Value -is [System.Management.Automation.SwitchParameter]) {
        [bool]$entry.Value
    }
    else {
        $entry.Value
    }
}
$driverParametersJson = ConvertTo-Json -InputObject $driverParameters -Compress
$previousAcceptancePath = [Environment]::GetEnvironmentVariable('ZIRCON_MVP_ACCEPTANCE_SCRIPT_PATH', 'Process')
$previousDriverParametersJson = [Environment]::GetEnvironmentVariable('ZIRCON_MVP_ACCEPTANCE_DRIVER_PARAMETERS_JSON', 'Process')
$previousChildPayloadPath = [Environment]::GetEnvironmentVariable('ZIRCON_MVP_ACCEPTANCE_CHILD_PAYLOAD_PATH', 'Process')
$childOutputPath = Join-Path ([IO.Path]::GetTempPath()) ('.zircon-mvp-acceptance-driver-' + [guid]::NewGuid().ToString('N') + '.stdout')
$childErrorPath = Join-Path ([IO.Path]::GetTempPath()) ('.zircon-mvp-acceptance-driver-' + [guid]::NewGuid().ToString('N') + '.stderr')
$childPayloadPath = Join-Path ([IO.Path]::GetTempPath()) ('.zircon-mvp-acceptance-driver-' + [guid]::NewGuid().ToString('N') + '.payload')
$childProcess = $null
$output = @()
$errorOutput = @()
$exitCode = 1
try {
    [Environment]::SetEnvironmentVariable('ZIRCON_MVP_ACCEPTANCE_SCRIPT_PATH', $driver, 'Process')
    [Environment]::SetEnvironmentVariable('ZIRCON_MVP_ACCEPTANCE_DRIVER_PARAMETERS_JSON', $driverParametersJson, 'Process')
    [Environment]::SetEnvironmentVariable('ZIRCON_MVP_ACCEPTANCE_CHILD_PAYLOAD_PATH', $childPayloadPath, 'Process')
    $childProcess = Start-Process `
        -FilePath (Get-Command pwsh -ErrorAction Stop).Source `
        -ArgumentList @('-NoProfile', '-NonInteractive', '-Command', $childCommand) `
        -RedirectStandardOutput $childOutputPath `
        -RedirectStandardError $childErrorPath `
        -PassThru `
        -WindowStyle Hidden
    if (-not $childProcess.WaitForExit(120000)) {
        & taskkill.exe /PID $childProcess.Id /T /F | Out-Null
        $childProcess.WaitForExit(5000) | Out-Null
        throw [TimeoutException]::new('MVP acceptance nested driver exceeded its 120-second timeout.')
    }
    # Refresh the Process snapshot before reading ExitCode; Start-Process may retain a
    # pre-exit snapshot even after WaitForExit returns on Windows PowerShell.
    $childProcess.Refresh()
    $exitCode = [int]$childProcess.ExitCode
    if (Test-Path -LiteralPath $childOutputPath) {
        $output += @(Get-Content -LiteralPath $childOutputPath -Encoding UTF8)
    }
    if (Test-Path -LiteralPath $childErrorPath) {
        $errorOutput += @(Get-Content -LiteralPath $childErrorPath -Encoding UTF8)
    }
    if (Test-Path -LiteralPath $childPayloadPath) {
        $payloadText = [IO.File]::ReadAllText($childPayloadPath, [Text.ASCIIEncoding]::new()).Trim()
        if (-not [string]::IsNullOrWhiteSpace($payloadText)) {
            $output = @(
                [Text.UTF8Encoding]::new($false).GetString([Convert]::FromBase64String($payloadText)) -split [Environment]::NewLine
            )
        }
    }
    # Some Windows PowerShell hosts report zero for a -Command child that handled a
    # terminating error through its wrapper. Treat the structured error envelope as
    # authoritative so a failed acceptance run can never be promoted as success.
    $childDetail = @($output + $errorOutput | ForEach-Object { $_.ToString() }) -join [Environment]::NewLine
    if ($exitCode -eq 0 -and $childDetail -match '"error_message"\s*:') {
        $exitCode = 1
    }
}
finally {
    [Environment]::SetEnvironmentVariable('ZIRCON_MVP_ACCEPTANCE_SCRIPT_PATH', $previousAcceptancePath, 'Process')
    [Environment]::SetEnvironmentVariable('ZIRCON_MVP_ACCEPTANCE_DRIVER_PARAMETERS_JSON', $previousDriverParametersJson, 'Process')
    [Environment]::SetEnvironmentVariable('ZIRCON_MVP_ACCEPTANCE_CHILD_PAYLOAD_PATH', $previousChildPayloadPath, 'Process')
    if ($null -ne $childProcess) {
        $childProcess.Dispose()
    }
    Remove-Item -LiteralPath $childOutputPath, $childErrorPath, $childPayloadPath -Force -ErrorAction SilentlyContinue
}
if ($exitCode -ne 0) {
    $detail = @($output + $errorOutput | ForEach-Object { $_.ToString() }) -join [Environment]::NewLine
    $structuredError = $null
    try {
        $structuredError = $detail | ConvertFrom-Json -ErrorAction Stop
    }
    catch {
        # Retain readable legacy process output if the child failed before its wrapper was active.
    }
    $errorProperty = if ($null -ne $structuredError) {
        $structuredError.PSObject.Properties['error_message']
    }
    if ($null -ne $errorProperty -and -not [string]::IsNullOrWhiteSpace([string]$errorProperty.Value)) {
        $detail = [string]$errorProperty.Value
    }
    throw "MVP acceptance driver failed with exit code $exitCode.$([Environment]::NewLine)$detail"
}

$output
