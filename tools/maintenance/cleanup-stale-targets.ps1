[CmdletBinding(SupportsShouldProcess = $true)]
param(
    [ValidateRange(1, 8760)]
    [int]$OlderThanHours = 2,
    [string]$RepoRoot,
    [string[]]$CleanupRoots,
    [switch]$HistoricalRepositoryTarget,
    [switch]$Apply
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
. (Join-Path (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path '.codex/skills/zircon-dev/scripts/coordinator-request-recovery.ps1')

function Get-DefaultCleanupRoots {
    # Legacy roots are scanned only for cleanup. They are never build output allowlist entries.
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

function Test-HistoricalCargoTargetOwnership {
    param([Parameter(Mandatory)][string]$TargetPath)

    $marker = $false
    $rustcMarker = Join-Path $TargetPath ".rustc_info.json"
    $cacheMarker = Join-Path $TargetPath "CACHEDIR.TAG"
    if (Test-Path -LiteralPath $rustcMarker -PathType Leaf) {
        try {
            $document = Get-Content -LiteralPath $rustcMarker -Raw | ConvertFrom-Json
            $marker = ($null -ne $document.PSObject.Properties["rustc_fingerprint"] -and
                $null -ne $document.PSObject.Properties["outputs"])
        } catch { $marker = $false }
    }
    if (-not $marker -and (Test-Path -LiteralPath $cacheMarker -PathType Leaf)) {
        $marker = (Get-Content -LiteralPath $cacheMarker -TotalCount 1) -eq
            "Signature: 8a477f597d28d172789f06886806bc55"
    }
    $cargoLayout = $false
    foreach ($profile in @("debug", "release")) {
        $profileRoot = Join-Path $TargetPath $profile
        if (-not (Test-Path -LiteralPath $profileRoot -PathType Container)) {
            continue
        }
        foreach ($artifactDirectory in @("deps", "build", "incremental")) {
            if (Test-Path -LiteralPath (Join-Path $profileRoot $artifactDirectory) -PathType Container) {
                $cargoLayout = $true
                break
            }
        }
    }
    return ($marker -and $cargoLayout)
}

function Test-HistoricalCargoProcessesIdle {
    param([string]$TargetPath = "E:\Git\ZirconEngine\target")

    try {
        $processes = @(Get-CimInstance Win32_Process -ErrorAction Stop)
    } catch {
        return [pscustomobject]@{ Idle = $false; Reason = "process_probe_failed" }
    }
    foreach ($process in $processes) {
        $name = [string]$process.Name
        $commandLine = [string]$process.CommandLine
        $executablePath = [string]$process.ExecutablePath
        $targetKey = ConvertTo-CleanupPathKey -Path $TargetPath
        if ($executablePath.ToLowerInvariant().StartsWith("$targetKey\") -or
            $commandLine.Replace('/', '\').ToLowerInvariant().Contains("$targetKey\")) {
            return [pscustomobject]@{ Idle = $false; Reason = "target_process_live" }
        }
        if ($name -notmatch '^(cargo|rustc)(\.exe)?$') {
            continue
        }
        if ([string]::IsNullOrWhiteSpace([string]$process.CommandLine)) {
            return [pscustomobject]@{ Idle = $false; Reason = "cargo_process_command_unknown" }
        }
        # Environment-owned target paths are not visible through Win32_Process.
        # Any live compiler therefore makes ownership uncertain and fails closed.
        return [pscustomobject]@{
            Idle = $false
            Reason = "cargo_process_live"
            ProcessId = [int]$process.ProcessId
            ChildProcessIds = @($processes | Where-Object {
                [int]$_.ParentProcessId -eq [int]$process.ProcessId
            } | ForEach-Object { [int]$_.ProcessId })
        }
    }
    return [pscustomobject]@{ Idle = $true; Reason = "idle" }
}

function Test-HistoricalExpectedCargoReparsePoint {
    param(
        [Parameter(Mandatory)][string]$TargetRoot,
        [Parameter(Mandatory)][System.IO.FileSystemInfo]$Item
    )

    $relativePath = [System.IO.Path]::GetRelativePath(
        (ConvertTo-CleanupPathKey -Path $TargetRoot),
        (ConvertTo-CleanupPathKey -Path $Item.FullName)
    ).Replace('/', '\\').ToLowerInvariant()
    $expectedTargets = @{
        'cache\cargo-metadata-home\git\checkouts' = 'cache\cargo-home\git\checkouts'
        'cache\cargo-metadata-home\git\db' = 'cache\cargo-home\git\db'
        'cache\cargo-metadata-home\registry\cache' = 'cache\cargo-home\registry\cache'
        'cache\cargo-metadata-home\registry\index' = 'cache\cargo-home\registry\index'
        'cache\cargo-metadata-home\registry\src' = 'cache\cargo-home\registry\src'
    }
    if (-not $expectedTargets.ContainsKey($relativePath)) {
        return $false
    }
    $expectedTarget = Join-Path $TargetRoot $expectedTargets[$relativePath]
    $actualTargets = @($Item.Target | ForEach-Object { [string]$_ })
    if ($actualTargets.Count -ne 1 -or [string]::IsNullOrWhiteSpace($actualTargets[0])) {
        return $false
    }
    return (ConvertTo-CleanupPathKey -Path $actualTargets[0]) -eq
        (ConvertTo-CleanupPathKey -Path $expectedTarget)
}

function Test-HistoricalRepositoryTarget {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$TargetPath
    )

    if (-not [System.IO.Path]::IsPathRooted($RepositoryRoot)) {
        return [pscustomobject]@{ Valid = $false; Reason = "repository_not_absolute" }
    }
    $requiredRepoKey = ConvertTo-CleanupPathKey -Path "E:\Git\ZirconEngine"
    $repoKey = ConvertTo-CleanupPathKey -Path $RepositoryRoot
    if ($repoKey -ne $requiredRepoKey) {
        return [pscustomobject]@{ Valid = $false; Reason = "historical_repo_not_approved" }
    }
    if (-not [System.IO.Path]::IsPathRooted($TargetPath)) {
        return [pscustomobject]@{ Valid = $false; Reason = "target_not_absolute" }
    }
    if (-not (Test-Path -LiteralPath $TargetPath -PathType Container)) {
        return [pscustomobject]@{ Valid = $false; Reason = "target_missing" }
    }
    $repo = Get-Item -LiteralPath $RepositoryRoot -Force
    $target = Get-Item -LiteralPath $TargetPath -Force
    $ancestor = $repo
    while ($null -ne $ancestor) {
        if (Test-CleanupReparsePoint -Item $ancestor) {
            return [pscustomobject]@{ Valid = $false; Reason = "ancestor_reparse_point" }
        }
        $ancestor = $ancestor.Parent
    }
    if ((Test-CleanupReparsePoint -Item $repo) -or (Test-CleanupReparsePoint -Item $target)) {
        return [pscustomobject]@{ Valid = $false; Reason = "reparse_point" }
    }
    $expectedTargetKey = ConvertTo-CleanupPathKey -Path (Join-Path $repo.FullName "target")
    $targetKey = ConvertTo-CleanupPathKey -Path $target.FullName
    if ($targetKey -ne $expectedTargetKey) {
        return [pscustomobject]@{ Valid = $false; Reason = "not_exact_repository_target" }
    }
    $directories = [System.Collections.Generic.Stack[string]]::new()
    $directories.Push($target.FullName)
    while ($directories.Count -gt 0) {
        foreach ($child in @(Get-ChildItem -LiteralPath $directories.Pop() -Force -ErrorAction Stop)) {
            if (Test-CleanupReparsePoint -Item $child) {
                if (-not (Test-HistoricalExpectedCargoReparsePoint -TargetRoot $target.FullName -Item $child)) {
                    return [pscustomobject]@{ Valid = $false; Reason = "nested_reparse_point" }
                }
                continue
            }
            if ($child.PSIsContainer) { $directories.Push($child.FullName) }
        }
    }
    if (-not (Test-HistoricalCargoTargetOwnership -TargetPath $target.FullName)) {
        return [pscustomobject]@{ Valid = $false; Reason = "cargo_ownership_unproven" }
    }
    $processState = Test-HistoricalCargoProcessesIdle -TargetPath $target.FullName
    if (-not $processState.Idle) {
        return [pscustomobject]@{ Valid = $false; Reason = $processState.Reason }
    }
    return [pscustomobject]@{ Valid = $true; Reason = "reviewed"; Target = $target }
}

function Remove-HistoricalRepositoryTarget {
    [CmdletBinding(SupportsShouldProcess = $true)]
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$TargetPath
    )

    $review = Test-HistoricalRepositoryTarget -RepositoryRoot $RepositoryRoot -TargetPath $TargetPath
    if (-not $review.Valid) {
        return [pscustomobject]@{ Path = $TargetPath; Status = "retained"; Reason = $review.Reason }
    }
    if (-not $PSCmdlet.ShouldProcess($review.Target.FullName, "delete approved historical repository Cargo target")) {
        return [pscustomobject]@{ Path = $TargetPath; Status = "retained"; Reason = "should_process_declined" }
    }
    try {
        Remove-Item -LiteralPath $review.Target.FullName -Recurse -Force -ErrorAction Stop
        return [pscustomobject]@{ Path = $TargetPath; Status = "deleted"; Reason = "deleted" }
    } catch {
        return [pscustomobject]@{ Path = $TargetPath; Status = "failed"; Reason = $_.Exception.Message }
    }
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

function Invoke-CoordinatorCleanupClient {
    param(
        [Parameter(Mandatory)][string]$Client,
        [Parameter(Mandatory)][string]$ResolvedRepoRoot,
        [Parameter(Mandatory)][string]$Command,
        [Parameter(Mandatory)][string[]]$Arguments
    )

    $raw = Invoke-CoordinatorUnacceptedRequest -Invoke {
        & $Client -Command $Command -RepoRoot $ResolvedRepoRoot -Json @Arguments
    } -Parse {
        param($rawOutput)
        ($rawOutput -join [Environment]::NewLine) | ConvertFrom-Json
    }
    $exitCode = $LASTEXITCODE
    $response = ($raw -join [Environment]::NewLine) | ConvertFrom-Json
    if ($exitCode -ne 0) {
        $recovered = Wait-CoordinatorAcceptedRequest -Failure $response -Query {
            param($requestId)
            $statusRaw = & $Client -Command 'request-status' -RepoRoot $ResolvedRepoRoot -Json $requestId
            $statusExitCode = $LASTEXITCODE
            $statusResponse = ($statusRaw -join [Environment]::NewLine) | ConvertFrom-Json
            $queryError = $statusResponse.PSObject.Properties['error']
            if ($statusExitCode -ne 0 -and ($null -eq $queryError -or $queryError.Value.code -ne 'request_overloaded')) {
                throw "Coordinator request $requestId status query failed: $($statusRaw -join [Environment]::NewLine)"
            }
            $statusResponse
        }
        if ($null -ne $recovered) { return $recovered }
        throw "Coordinator $Command failed: $($raw -join [Environment]::NewLine)"
    }
    return $response
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
    Invoke-CoordinatorCleanupClient -Client $Client -ResolvedRepoRoot $ResolvedRepoRoot `
        -Command cleanup -Arguments $arguments
}

function Invoke-CoordinatorArtifactCommand {
    param(
        [Parameter(Mandatory)][string]$Client,
        [Parameter(Mandatory)][string]$ResolvedRepoRoot,
        [Parameter(Mandatory)][ValidateSet("audit", "cleanup")][string]$Action
    )

    Invoke-CoordinatorCleanupClient -Client $Client -ResolvedRepoRoot $ResolvedRepoRoot `
        -Command artifact -Arguments @($Action)
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
        [switch]$IncludeHistoricalRepositoryTarget,
        [switch]$ApplyChanges
    )

    if ([string]::IsNullOrWhiteSpace($RepositoryRoot)) {
        $RepositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
    }
    $resolvedRepoRoot = (Resolve-Path -LiteralPath $RepositoryRoot).Path
    $client = Join-Path $resolvedRepoRoot "tools\dev\zircon-session.ps1"
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
    if ($null -ne $plan.PSObject.Properties["cache_budget"] -and $null -ne $plan.cache_budget) {
        Write-Host ("  Managed cache bytes: " + $plan.cache_budget.before.total_bytes)
        Write-Host ("  Incremental bytes: " + $plan.cache_budget.before.incremental_bytes)
        Write-Host ("  Eviction target bytes: " + $plan.cache_budget.target_bytes)
    }
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

    $historicalTarget = Join-Path $resolvedRepoRoot "target"
    $historicalReview = $null
    if ($IncludeHistoricalRepositoryTarget) {
        $historicalReview = Test-HistoricalRepositoryTarget `
            -RepositoryRoot $resolvedRepoRoot `
            -TargetPath $historicalTarget
        Write-Host "Historical repository target: $($historicalReview.Reason) $historicalTarget"
    }

    if (-not $ApplyChanges -or $WhatIfPreference) {
        Write-Host "Plan only. Pass -Apply to request coordinator-managed cleanup."
        return
    }

    if ($IncludeHistoricalRepositoryTarget -and $historicalReview.Valid) {
        $historicalResult = Remove-HistoricalRepositoryTarget `
            -RepositoryRoot $resolvedRepoRoot `
            -TargetPath $historicalTarget `
            -Confirm:$false
        Write-Host "Historical repository target: $($historicalResult.Status) ($($historicalResult.Reason))"
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
        -IncludeHistoricalRepositoryTarget:$HistoricalRepositoryTarget `
        -ApplyChanges:$Apply `
        -WhatIf:$WhatIfPreference
}
