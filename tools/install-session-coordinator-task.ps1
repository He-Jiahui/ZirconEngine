[CmdletBinding(SupportsShouldProcess = $true)]
param(
    [ValidateSet("Install", "Update", "Remove", "Query")]
    [string]$Action = "Query",
    [string]$RepoRoot,
    [ValidateRange(5, 1440)]
    [int]$MaintenanceMinutes = 15,
    [switch]$DryRun
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
$client = Join-Path $resolvedRepoRoot "tools\zircon-session.ps1"
$cleanup = Join-Path $resolvedRepoRoot "tools\cleanup-stale-targets.ps1"
$powerShell = (Get-Command powershell.exe -ErrorAction Stop).Source

function Quote-TaskArgument {
    param([string]$Value)
    return '"' + $Value.Replace('"', '""') + '"'
}

function Invoke-TaskCommand {
    param(
        [string]$Description,
        [string[]]$Arguments,
        [switch]$IgnoreMissing
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
    if ($LASTEXITCODE -ne 0 -and -not $IgnoreMissing) {
        throw "Scheduled task operation failed: $Description"
    }
}

function New-TaskCommandLine {
    param([string]$ScriptPath, [string[]]$ScriptArguments)

    $parts = @(
        (Quote-TaskArgument $powerShell),
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

if ($Action -eq "Query") {
    Invoke-TaskCommand -Description $daemonTask -Arguments @("/Query", "/TN", $daemonTask, "/FO", "LIST", "/V") -IgnoreMissing
    Invoke-TaskCommand -Description $maintenanceTask -Arguments @("/Query", "/TN", $maintenanceTask, "/FO", "LIST", "/V") -IgnoreMissing
    exit 0
}

if ($Action -eq "Remove") {
    Invoke-TaskCommand -Description $daemonTask -Arguments @("/Delete", "/TN", $daemonTask, "/F") -IgnoreMissing
    Invoke-TaskCommand -Description $maintenanceTask -Arguments @("/Delete", "/TN", $maintenanceTask, "/F") -IgnoreMissing
    exit 0
}

if (-not (Test-Path -LiteralPath $client) -or -not (Test-Path -LiteralPath $cleanup)) {
    throw "Coordinator scripts are incomplete under $resolvedRepoRoot"
}

$daemonCommand = New-TaskCommandLine -ScriptPath $client -ScriptArguments @("start", "-RepoRoot", $resolvedRepoRoot)
$maintenanceCommand = New-TaskCommandLine -ScriptPath $cleanup -ScriptArguments @("-RepoRoot", $resolvedRepoRoot, "-Apply")

Invoke-TaskCommand -Description $daemonTask -Arguments @(
    "/Create", "/TN", $daemonTask, "/SC", "ONLOGON", "/TR", $daemonCommand,
    "/RL", "LIMITED", "/F"
)
Invoke-TaskCommand -Description $maintenanceTask -Arguments @(
    "/Create", "/TN", $maintenanceTask, "/SC", "MINUTE", "/MO", "$MaintenanceMinutes",
    "/TR", $maintenanceCommand, "/RL", "LIMITED", "/F"
)

if (-not $DryRun) {
    & $client start -RepoRoot $resolvedRepoRoot | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw "Coordinator health check failed after scheduled task $Action"
    }
}

Write-Output "Scheduled task configuration ready for $resolvedRepoRoot"
