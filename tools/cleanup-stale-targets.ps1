[CmdletBinding(SupportsShouldProcess = $true)]
param(
    [ValidateRange(1, 8760)]
    [int]$OlderThanHours = 2,
    [string]$RepoRoot,
    [switch]$Apply
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $RepoRoot = Split-Path -Parent $PSScriptRoot
}

$resolvedRepoRoot = (Resolve-Path -LiteralPath $RepoRoot).Path
$client = Join-Path $resolvedRepoRoot "tools\zircon-session.ps1"
if (-not (Test-Path -LiteralPath $client)) {
    throw "Session coordinator client is missing: $client"
}

function Invoke-CleanupCommand {
    param([string]$Action, [object]$ReviewedPlan)

    $arguments = @($Action, "--older-than-hours", [string]$OlderThanHours)
    if ($Action -eq "apply") {
        $arguments += @("--plan-id", [string]$ReviewedPlan.plan_id)
    }
    $raw = & $client -Command cleanup -RepoRoot $resolvedRepoRoot -Json @arguments
    if ($LASTEXITCODE -ne 0) {
        throw "Coordinator cleanup $Action failed: $($raw -join [Environment]::NewLine)"
    }
    return (($raw -join [Environment]::NewLine) | ConvertFrom-Json)
}

$response = Invoke-CleanupCommand -Action "plan"
$plan = $response.plan
Write-Host "Managed Cargo cleanup plan"
foreach ($root in @($plan.free_bytes_by_root.PSObject.Properties)) {
    $pressure = if (@($plan.pressure_roots) -contains $root.Name) { " LOW-DISK" } else { "" }
    Write-Host ("  Root {0}: {1:N2} GB free{2}" -f $root.Name, ([int64]$root.Value / 1GB), $pressure)
}
Write-Host "  Candidates: $(@($plan.candidates).Count)"
foreach ($candidate in @($plan.candidates)) {
    Write-Host "  - $candidate"
}
Write-Host "  Denied/retained: $(@($plan.denied).Count)"
foreach ($denial in @($plan.denied)) {
    Write-Host "  - [$($denial.code)] $($denial.path): $($denial.message)"
}

if (-not $Apply) {
    Write-Host "Plan only. Pass -Apply to request deletion after service revalidation."
    exit 0
}

if (@($plan.candidates).Count -eq 0) {
    Write-Host "No reviewed cleanup candidates; nothing to apply."
    exit 0
}

if (-not $PSCmdlet.ShouldProcess(
        "$(@($plan.candidates).Count) managed Cargo lane(s)",
        "service cleanup apply with PID, lease, retention, and realpath revalidation"
    )) {
    exit 0
}

$applied = Invoke-CleanupCommand -Action "apply" -ReviewedPlan $plan
Write-Host "Deleted: $(@($applied.result.deleted).Count)"
foreach ($target in @($applied.result.deleted)) {
    Write-Host "  - $target"
}
foreach ($denial in @($applied.result.denied)) {
    Write-Host "  - retained [$($denial.code)] $($denial.path): $($denial.message)"
}
