[CmdletBinding(SupportsShouldProcess = $true)]
param(
    [ValidateRange(1, 8760)]
    [int]$OlderThanHours = 2,
    [string]$RepoRoot,
    [string[]]$CleanupRoots,
    [switch]$Apply
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Get-DefaultCleanupRoots {
    foreach ($drive in @("D", "E", "F")) {
        foreach ($name in @("cargo-targets", "targets", "ZirconBuilds")) {
            "$drive`:\$name"
        }
    }
}

function ConvertTo-CleanupPathKey {
    param([Parameter(Mandatory)][string]$Path)

    return [System.IO.Path]::GetFullPath($Path).TrimEnd('\', '/').ToLowerInvariant()
}

function Test-CleanupReparsePoint {
    param([Parameter(Mandatory)][System.IO.FileSystemInfo]$Item)

    return [bool]($Item.Attributes -band [System.IO.FileAttributes]::ReparsePoint)
}

function Test-CleanupPathOverlapsManagedTarget {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][object]$ManagedPathKeys
    )

    $pathKey = ConvertTo-CleanupPathKey -Path $Path
    $descendantPrefix = "$pathKey\"
    foreach ($managedPathKey in $ManagedPathKeys) {
        $managedKey = [string]$managedPathKey
        if ($managedKey.Equals($pathKey, [System.StringComparison]::OrdinalIgnoreCase) -or
            $managedKey.StartsWith($descendantPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
            return $true
        }
    }
    return $false
}

function Get-UnmanagedCleanupCandidates {
    param(
        [Parameter(Mandatory)][string[]]$Roots,
        [Parameter(Mandatory)][object]$ManagedPathKeys,
        [Parameter(Mandatory)][datetime]$CutoffUtc
    )

    $candidates = [System.Collections.Generic.List[object]]::new()
    foreach ($rootPath in $Roots) {
        if (-not (Test-Path -LiteralPath $rootPath -PathType Container)) {
            continue
        }
        $root = Get-Item -LiteralPath $rootPath -Force
        if (Test-CleanupReparsePoint -Item $root) {
            continue
        }
        $rootKey = ConvertTo-CleanupPathKey -Path $root.FullName
        foreach ($child in @(Get-ChildItem -LiteralPath $root.FullName -Directory -Force)) {
            if (Test-CleanupReparsePoint -Item $child) {
                continue
            }
            $childKey = ConvertTo-CleanupPathKey -Path $child.FullName
            if ($childKey -eq $rootKey -or
                (Test-CleanupPathOverlapsManagedTarget -Path $child.FullName -ManagedPathKeys $ManagedPathKeys)) {
                continue
            }
            if ($child.LastWriteTimeUtc -gt $CutoffUtc) {
                continue
            }
            $candidates.Add([pscustomobject]@{
                Root             = $root.FullName
                Path             = $child.FullName
                LastWriteTimeUtc = $child.LastWriteTimeUtc
            }) | Out-Null
        }
    }
    return @($candidates | Sort-Object Path)
}

function Test-UnmanagedCleanupCandidate {
    param(
        [Parameter(Mandatory)][string]$Root,
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][datetime]$CutoffUtc,
        [Parameter(Mandatory)][object]$ManagedPathKeys
    )

    if (-not (Test-Path -LiteralPath $Root -PathType Container)) {
        return [pscustomobject]@{ Valid = $false; Reason = "root_missing" }
    }
    if (-not (Test-Path -LiteralPath $Path -PathType Container)) {
        return [pscustomobject]@{ Valid = $false; Reason = "target_missing" }
    }
    $rootItem = Get-Item -LiteralPath $Root -Force
    $candidate = Get-Item -LiteralPath $Path -Force
    if ((Test-CleanupReparsePoint -Item $rootItem) -or
        (Test-CleanupReparsePoint -Item $candidate)) {
        return [pscustomobject]@{ Valid = $false; Reason = "reparse_point" }
    }
    $rootKey = ConvertTo-CleanupPathKey -Path $rootItem.FullName
    $candidateKey = ConvertTo-CleanupPathKey -Path $candidate.FullName
    $parentKey = ConvertTo-CleanupPathKey -Path $candidate.Parent.FullName
    if ($candidateKey -eq $rootKey -or $parentKey -ne $rootKey) {
        return [pscustomobject]@{ Valid = $false; Reason = "not_direct_child" }
    }
    if (Test-CleanupPathOverlapsManagedTarget -Path $candidate.FullName -ManagedPathKeys $ManagedPathKeys) {
        return [pscustomobject]@{ Valid = $false; Reason = "managed_path_overlap" }
    }
    if ($candidate.LastWriteTimeUtc -gt $CutoffUtc) {
        return [pscustomobject]@{ Valid = $false; Reason = "target_became_fresh" }
    }
    return [pscustomobject]@{
        Valid     = $true
        Reason    = "reviewed"
        Root      = $rootItem
        Candidate = $candidate
    }
}

function Remove-UnmanagedCleanupCandidate {
    [CmdletBinding(SupportsShouldProcess = $true)]
    param(
        [Parameter(Mandatory)][string]$Root,
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][datetime]$CutoffUtc,
        [Parameter(Mandatory)][object]$ManagedPathKeys
    )

    $review = Test-UnmanagedCleanupCandidate `
        -Root $Root `
        -Path $Path `
        -CutoffUtc $CutoffUtc `
        -ManagedPathKeys $ManagedPathKeys
    if (-not $review.Valid) {
        return [pscustomobject]@{ Path = $Path; Status = "retained"; Reason = $review.Reason }
    }
    if (-not $PSCmdlet.ShouldProcess($review.Candidate.FullName, "delete stale unmanaged build target")) {
        return [pscustomobject]@{ Path = $Path; Status = "retained"; Reason = "should_process_declined" }
    }
    try {
        Remove-Item -LiteralPath $review.Candidate.FullName -Recurse -Force -ErrorAction Stop
        return [pscustomobject]@{ Path = $Path; Status = "deleted"; Reason = "deleted" }
    } catch {
        return [pscustomobject]@{ Path = $Path; Status = "failed"; Reason = $_.Exception.Message }
    }
}

function Invoke-CoordinatorCleanupCommand {
    param(
        [Parameter(Mandatory)][string]$Client,
        [Parameter(Mandatory)][string]$ResolvedRepoRoot,
        [Parameter(Mandatory)][string]$Action,
        [Parameter(Mandatory)][int]$RetentionHours,
        [object]$ReviewedPlan
    )

    $arguments = @($Action, "--older-than-hours", [string]$RetentionHours)
    if ($Action -eq "apply") {
        $arguments += @("--plan-id", [string]$ReviewedPlan.plan_id)
    }
    $raw = & $Client -Command cleanup -RepoRoot $ResolvedRepoRoot -Json @arguments
    if ($LASTEXITCODE -ne 0) {
        throw "Coordinator cleanup $Action failed: $($raw -join [Environment]::NewLine)"
    }
    return (($raw -join [Environment]::NewLine) | ConvertFrom-Json)
}

function Invoke-CoordinatorArtifactCommand {
    param(
        [Parameter(Mandatory)][string]$Client,
        [Parameter(Mandatory)][string]$ResolvedRepoRoot,
        [Parameter(Mandatory)][ValidateSet("audit", "cleanup")][string]$Action
    )

    $raw = & $Client -Command artifact -RepoRoot $ResolvedRepoRoot -Json $Action
    if ($LASTEXITCODE -ne 0) {
        throw "Coordinator artifact $Action failed: $($raw -join [Environment]::NewLine)"
    }
    return (($raw -join [Environment]::NewLine) | ConvertFrom-Json)
}

function Get-CoordinatorManagedPathKeys {
    param(
        [Parameter(Mandatory)][string]$Client,
        [Parameter(Mandatory)][string]$ResolvedRepoRoot,
        [Parameter(Mandatory)][object]$Plan
    )

    $managedPathKeys = [System.Collections.Generic.HashSet[string]]::new(
        [System.StringComparer]::OrdinalIgnoreCase
    )
    foreach ($path in @($Plan.candidates) + @($Plan.denied | ForEach-Object { $_.path })) {
        if (-not [string]::IsNullOrWhiteSpace([string]$path)) {
            $managedPathKeys.Add((ConvertTo-CleanupPathKey -Path ([string]$path))) | Out-Null
        }
    }

    $raw = & $Client -Command cargo -RepoRoot $ResolvedRepoRoot -Json list
    if ($LASTEXITCODE -ne 0) {
        throw "Coordinator Cargo list failed: $($raw -join [Environment]::NewLine)"
    }
    $jobs = (($raw -join [Environment]::NewLine) | ConvertFrom-Json).jobs
    foreach ($job in @($jobs)) {
        if (-not [string]::IsNullOrWhiteSpace([string]$job.target_dir)) {
            $managedPathKeys.Add(
                (ConvertTo-CleanupPathKey -Path ([string]$job.target_dir))
            ) | Out-Null
        }
    }
    return $managedPathKeys
}

function Invoke-StaleTargetCleanup {
    [CmdletBinding(SupportsShouldProcess = $true)]
    param(
        [ValidateRange(1, 8760)][int]$RetentionHours = 2,
        [string]$RepositoryRoot,
        [string[]]$Roots,
        [switch]$ApplyChanges
    )

    if ([string]::IsNullOrWhiteSpace($RepositoryRoot)) {
        $RepositoryRoot = Split-Path -Parent $PSScriptRoot
    }
    $resolvedRepoRoot = (Resolve-Path -LiteralPath $RepositoryRoot).Path
    $client = Join-Path $resolvedRepoRoot "tools\zircon-session.ps1"
    if (-not (Test-Path -LiteralPath $client)) {
        throw "Session coordinator client is missing: $client"
    }
    if ($null -eq $Roots -or $Roots.Count -eq 0) {
        $Roots = @(Get-DefaultCleanupRoots)
    }

    $response = Invoke-CoordinatorCleanupCommand `
        -Client $client `
        -ResolvedRepoRoot $resolvedRepoRoot `
        -Action "plan" `
        -RetentionHours $RetentionHours
    $plan = $response.plan
    $unmanaged = @((Invoke-CoordinatorArtifactCommand `
        -Client $client `
        -ResolvedRepoRoot $resolvedRepoRoot `
        -Action "audit").unmanaged)

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
    Write-Host "Unmanaged stale targets: $($unmanaged.Count)"
    foreach ($candidate in $unmanaged) {
        Write-Host "  - $candidate"
    }

    if (-not $ApplyChanges -or $WhatIfPreference) {
        Write-Host "Plan only. Pass -Apply to request coordinator-managed cleanup."
        return
    }

    if (@($plan.candidates).Count -gt 0 -and $PSCmdlet.ShouldProcess(
        "$(@($plan.candidates).Count) managed Cargo lane(s)",
        "service cleanup apply with PID, lease, retention, and realpath revalidation"
    )) {
        $applied = Invoke-CoordinatorCleanupCommand `
            -Client $client `
            -ResolvedRepoRoot $resolvedRepoRoot `
            -Action "apply" `
            -RetentionHours $RetentionHours `
            -ReviewedPlan $plan
        Write-Host "Managed deleted: $(@($applied.result.deleted).Count)"
        foreach ($target in @($applied.result.deleted)) {
            Write-Host "  - $target"
        }
        foreach ($denial in @($applied.result.denied)) {
            Write-Host "  - retained [$($denial.code)] $($denial.path): $($denial.message)"
        }
    }

    $artifactResult = Invoke-CoordinatorArtifactCommand `
        -Client $client `
        -ResolvedRepoRoot $resolvedRepoRoot `
        -Action "cleanup"
    Write-Host "Unmanaged deleted: $(@($artifactResult.deleted).Count)"
    foreach ($path in @($artifactResult.failed)) {
        Write-Host "  - failed $path"
    }
    foreach ($path in @($artifactResult.remaining)) {
        Write-Host "  - retained $path"
    }
}

if ($MyInvocation.InvocationName -ne ".") {
    Invoke-StaleTargetCleanup `
        -RetentionHours $OlderThanHours `
        -RepositoryRoot $RepoRoot `
        -Roots $CleanupRoots `
        -ApplyChanges:$Apply `
        -WhatIf:$WhatIfPreference
}
