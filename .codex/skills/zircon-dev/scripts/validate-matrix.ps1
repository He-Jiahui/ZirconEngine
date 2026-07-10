[CmdletBinding()]
param(
    [string]$RepoRoot,
    [string]$Package,
    [string]$TargetDir,
    [switch]$SkipBuild,
    [switch]$SkipTest,
    [switch]$RunExportPlatformContract,
    [string]$ExportContractPlatform,
    [switch]$RunProfileFeatureContract,
    [string]$ProfileFeatureContractLabel,
    [switch]$NoLocked,
    [switch]$VerboseOutput,
    [switch]$DryRun
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$script:SharedTargetLeaseTtl = [TimeSpan]::FromHours(12)
$script:SharedTargetLockRetryCount = 50
$script:SharedTargetLockRetryDelayMs = 200
$script:LowDiskCleanupThresholdBytes = 50GB
$script:DryRunDefaultTargetDir = Join-Path "target" "manual-check"
$script:ExportContractPlatforms = @(
    "windows",
    "linux",
    "macos",
    "android",
    "ios",
    "web_gpu",
    "wasm",
    "headless"
)
$script:ProfileFeatureContractCases = @(
    [pscustomobject]@{
        Label = "zircon_app target-server"
        Package = "zircon_app"
        Features = "target-server"
    },
    [pscustomobject]@{
        Label = "zircon_app target-client-platform"
        Package = "zircon_app"
        Features = "target-client,platform-winit,input-gamepad,gamepad-gilrs"
    },
    [pscustomobject]@{
        Label = "zircon_runtime target-client"
        Package = "zircon_runtime"
        Features = "target-client"
    },
    [pscustomobject]@{
        Label = "zircon_runtime target-editor-host"
        Package = "zircon_runtime"
        Features = "target-editor-host"
    },
    [pscustomobject]@{
        Label = "zircon_runtime target-server"
        Package = "zircon_runtime"
        Features = "target-server"
    }
)

function Find-RepoRoot {
    param([string]$StartPath)

    $current = Resolve-Path $StartPath
    while ($true) {
        if ((Test-Path (Join-Path $current.Path "Cargo.toml")) -and
            (Test-Path (Join-Path $current.Path ".codex\skills\zircon-dev"))) {
            return $current.Path
        }

        $parent = Split-Path $current.Path -Parent
        if ([string]::IsNullOrWhiteSpace($parent) -or $parent -eq $current.Path) {
            throw "Could not locate repository root from $StartPath"
        }

        $current = Resolve-Path $parent
    }
}

function Normalize-PathString {
    param([string]$Path)

    if ([string]::IsNullOrWhiteSpace($Path)) {
        return $null
    }

    return [System.IO.Path]::GetFullPath($Path).TrimEnd('\', '/').ToLowerInvariant()
}

function Get-LeaseValue {
    param(
        [object]$Lease,
        [string]$Name
    )

    if ($null -eq $Lease) {
        return $null
    }

    if ($Lease -is [hashtable]) {
        if ($Lease.ContainsKey($Name)) {
            return $Lease[$Name]
        }

        return $null
    }

    $property = $Lease.PSObject.Properties[$Name]
    if ($null -ne $property) {
        return $property.Value
    }

    return $null
}

function ConvertTo-UtcDateTimeOrNull {
    param([object]$Value)

    if ($null -eq $Value) {
        return $null
    }

    $text = [string]$Value
    if ([string]::IsNullOrWhiteSpace($text)) {
        return $null
    }

    try {
        return [datetime]::Parse(
            $text,
            [System.Globalization.CultureInfo]::InvariantCulture,
            [System.Globalization.DateTimeStyles]::RoundtripKind
        ).ToUniversalTime()
    } catch {
        return $null
    }
}

function Get-TargetLeaseDirectory {
    param([string]$RepoRoot)

    return Join-Path $RepoRoot ".codex\tmp\cargo-target-slots"
}

function Get-SharedCargoSlotDefinitions {
    param([string]$RepoRoot)

    $leaseRoot = Get-TargetLeaseDirectory -RepoRoot $RepoRoot

    return @(
        [pscustomobject]@{
            SlotName          = "a"
            CliTargetDir      = Join-Path "target" "codex-shared-a"
            AbsoluteTargetDir = [System.IO.Path]::GetFullPath((Join-Path $RepoRoot (Join-Path "target" "codex-shared-a")))
            LeasePath         = Join-Path $leaseRoot "slot-a.json"
        },
        [pscustomobject]@{
            SlotName          = "b"
            CliTargetDir      = Join-Path "target" "codex-shared-b"
            AbsoluteTargetDir = [System.IO.Path]::GetFullPath((Join-Path $RepoRoot (Join-Path "target" "codex-shared-b")))
            LeasePath         = Join-Path $leaseRoot "slot-b.json"
        }
    )
}

function Read-TargetLease {
    param([string]$LeasePath)

    if (-not (Test-Path $LeasePath)) {
        return $null
    }

    try {
        return Get-Content -Raw $LeasePath | ConvertFrom-Json
    } catch {
        return [pscustomobject]@{
            _invalid = $true
        }
    }
}

function Write-TargetLease {
    param(
        [string]$LeasePath,
        [hashtable]$Lease
    )

    New-Item -ItemType Directory -Force -Path (Split-Path $LeasePath -Parent) | Out-Null
    $Lease | ConvertTo-Json | Set-Content -Encoding utf8 $LeasePath
}

function Test-TargetLeaseStale {
    param(
        [object]$Lease,
        [string]$RepoRoot,
        [datetime]$NowUtc
    )

    if ($null -eq $Lease) {
        return $true
    }

    if ([bool](Get-LeaseValue -Lease $Lease -Name "_invalid")) {
        return $true
    }

    $leaseRepoRoot = Normalize-PathString -Path ([string](Get-LeaseValue -Lease $Lease -Name "repo_root"))
    $currentRepoRoot = Normalize-PathString -Path $RepoRoot
    if ($leaseRepoRoot -ne $currentRepoRoot) {
        return $true
    }

    $lastSeenUtc = ConvertTo-UtcDateTimeOrNull -Value (Get-LeaseValue -Lease $Lease -Name "last_seen_utc")
    if ($null -eq $lastSeenUtc) {
        return $true
    }

    return (($NowUtc - $lastSeenUtc) -gt $script:SharedTargetLeaseTtl)
}

function Get-TargetLeaseState {
    param(
        [object]$Lease,
        [string]$RepoRoot,
        [string]$OwnerId,
        [datetime]$NowUtc
    )

    if ($null -eq $Lease) {
        return "free"
    }

    if ([bool](Get-LeaseValue -Lease $Lease -Name "_invalid")) {
        return "stale"
    }

    if (Test-TargetLeaseStale -Lease $Lease -RepoRoot $RepoRoot -NowUtc $NowUtc) {
        return "stale"
    }

    $existingOwnerId = [string](Get-LeaseValue -Lease $Lease -Name "owner_id")
    if ([string]::IsNullOrWhiteSpace($existingOwnerId)) {
        return "free"
    }

    if ($existingOwnerId -eq $OwnerId) {
        return "owner"
    }

    return "occupied"
}

function Resolve-OwnerId {
    param([string]$RepoRoot)

    if (-not [string]::IsNullOrWhiteSpace($env:CODEX_THREAD_ID)) {
        return $env:CODEX_THREAD_ID
    }

    $user = [Environment]::UserName
    $machine = [Environment]::MachineName
    return "manual:{0}@{1}:{2}" -f $user, $machine, (Normalize-PathString -Path $RepoRoot)
}

function Use-TargetLeaseLock {
    param(
        [string]$LeaseRoot,
        [scriptblock]$Action
    )

    New-Item -ItemType Directory -Force -Path $LeaseRoot | Out-Null

    $lockDir = Join-Path $LeaseRoot ".lock"
    $lockTaken = $false

    try {
        for ($attempt = 0; $attempt -lt $script:SharedTargetLockRetryCount; $attempt++) {
            try {
                New-Item -ItemType Directory -Path $lockDir -ErrorAction Stop | Out-Null
                $lockTaken = $true
                break
            } catch {
                if ($attempt -eq ($script:SharedTargetLockRetryCount - 1)) {
                    throw "Could not acquire shared cargo target lock at $lockDir"
                }

                Start-Sleep -Milliseconds $script:SharedTargetLockRetryDelayMs
            }
        }

        return & $Action
    } finally {
        if ($lockTaken -and (Test-Path $lockDir)) {
            Remove-Item -Recurse -Force $lockDir -ErrorAction SilentlyContinue
        }
    }
}

function Resolve-SharedCargoTarget {
    param(
        [string]$RepoRoot,
        [string]$OwnerId,
        [object[]]$Leases,
        [datetime]$NowUtc
    )

    $slotDefinitions = Get-SharedCargoSlotDefinitions -RepoRoot $RepoRoot
    $leaseBySlot = @{}

    foreach ($lease in $Leases) {
        $slotName = [string](Get-LeaseValue -Lease $lease -Name "slot_name")
        if (-not [string]::IsNullOrWhiteSpace($slotName)) {
            $leaseBySlot[$slotName] = $lease
        }
    }

    foreach ($slotDefinition in $slotDefinitions) {
        $lease = $leaseBySlot[$slotDefinition.SlotName]
        $existingOwnerId = [string](Get-LeaseValue -Lease $lease -Name "owner_id")
        if (-not [string]::IsNullOrWhiteSpace($existingOwnerId) -and $existingOwnerId -eq $OwnerId) {
            return [pscustomobject]@{
                SelectionMode    = "shared"
                SlotName         = $slotDefinition.SlotName
                TargetDir        = $slotDefinition.CliTargetDir
                AbsoluteTargetDir = $slotDefinition.AbsoluteTargetDir
                Reason           = "reused current thread slot $($slotDefinition.SlotName)"
                PreviousLease    = $lease
            }
        }
    }

    foreach ($slotDefinition in $slotDefinitions) {
        $lease = $leaseBySlot[$slotDefinition.SlotName]
        $leaseState = Get-TargetLeaseState -Lease $lease -RepoRoot $RepoRoot -OwnerId $OwnerId -NowUtc $NowUtc
        if ($leaseState -eq "free" -or $leaseState -eq "stale") {
            $reason = if ($leaseState -eq "free") {
                "claimed free slot $($slotDefinition.SlotName)"
            } else {
                "claimed stale slot $($slotDefinition.SlotName)"
            }

            return [pscustomobject]@{
                SelectionMode    = "shared"
                SlotName         = $slotDefinition.SlotName
                TargetDir        = $slotDefinition.CliTargetDir
                AbsoluteTargetDir = $slotDefinition.AbsoluteTargetDir
                Reason           = $reason
                PreviousLease    = $lease
            }
        }
    }

    throw "Both shared cargo target slots are occupied by other active sessions. Pass -TargetDir to override."
}

function Resolve-AbsoluteTargetDir {
    param(
        [string]$RepoRoot,
        [string]$CliTargetDir
    )

    if ([System.IO.Path]::IsPathRooted($CliTargetDir)) {
        return [System.IO.Path]::GetFullPath($CliTargetDir)
    }

    return [System.IO.Path]::GetFullPath((Join-Path $RepoRoot $CliTargetDir))
}

function Resolve-EnvironmentTargetDir {
    param([string]$RepoRoot)

    if ([string]::IsNullOrWhiteSpace($env:CARGO_TARGET_DIR)) {
        return $null
    }

    $absoluteTargetDir = Resolve-AbsoluteTargetDir -RepoRoot $RepoRoot -CliTargetDir $env:CARGO_TARGET_DIR
    New-Item -ItemType Directory -Force -Path $absoluteTargetDir | Out-Null

    return [pscustomobject]@{
        SelectionMode     = "environment"
        SlotName          = $null
        TargetDir         = $env:CARGO_TARGET_DIR
        AbsoluteTargetDir = $absoluteTargetDir
        Reason            = "CARGO_TARGET_DIR environment override"
        OwnerId           = $null
    }
}

function Resolve-EffectiveTargetDir {
    param(
        [string]$RepoRoot,
        [string]$ManualTargetDir
    )

    if (-not [string]::IsNullOrWhiteSpace($ManualTargetDir)) {
        return [pscustomobject]@{
            SelectionMode    = "manual"
            SlotName         = $null
            TargetDir        = $ManualTargetDir
            AbsoluteTargetDir = Resolve-AbsoluteTargetDir -RepoRoot $RepoRoot -CliTargetDir $ManualTargetDir
            Reason           = "manual override"
            OwnerId          = $null
        }
    }

    $environmentTarget = Resolve-EnvironmentTargetDir -RepoRoot $RepoRoot
    if ($null -ne $environmentTarget) {
        return $environmentTarget
    }

    $leaseRoot = Get-TargetLeaseDirectory -RepoRoot $RepoRoot
    $ownerId = Resolve-OwnerId -RepoRoot $RepoRoot
    $nowUtc = (Get-Date).ToUniversalTime()

    return Use-TargetLeaseLock -LeaseRoot $leaseRoot -Action {
        $slotDefinitions = Get-SharedCargoSlotDefinitions -RepoRoot $RepoRoot
        $leases = foreach ($slotDefinition in $slotDefinitions) {
            $lease = Read-TargetLease -LeasePath $slotDefinition.LeasePath
            if ($null -eq $lease) {
                [pscustomobject]@{
                    slot_name  = $slotDefinition.SlotName
                    target_dir = $slotDefinition.CliTargetDir
                }
            } else {
                if ($null -eq (Get-LeaseValue -Lease $lease -Name "slot_name")) {
                    $lease | Add-Member -NotePropertyName slot_name -NotePropertyValue $slotDefinition.SlotName -Force
                }

                if ($null -eq (Get-LeaseValue -Lease $lease -Name "target_dir")) {
                    $lease | Add-Member -NotePropertyName target_dir -NotePropertyValue $slotDefinition.CliTargetDir -Force
                }

                $lease
            }
        }

        $selection = Resolve-SharedCargoTarget -RepoRoot $RepoRoot -OwnerId $ownerId -Leases $leases -NowUtc $nowUtc

        New-Item -ItemType Directory -Force -Path $selection.AbsoluteTargetDir | Out-Null

        $claimedAtUtc = ConvertTo-UtcDateTimeOrNull -Value (Get-LeaseValue -Lease $selection.PreviousLease -Name "claimed_at_utc")
        if ($null -eq $claimedAtUtc -or ([string](Get-LeaseValue -Lease $selection.PreviousLease -Name "owner_id")) -ne $ownerId) {
            $claimedAtUtc = $nowUtc
        }

        $slotDefinition = $slotDefinitions | Where-Object { $_.SlotName -eq $selection.SlotName } | Select-Object -First 1
        Write-TargetLease -LeasePath $slotDefinition.LeasePath -Lease @{
            slot_name      = $selection.SlotName
            target_dir     = $selection.TargetDir
            owner_id       = $ownerId
            owner_pid      = $PID
            claimed_at_utc = $claimedAtUtc.ToString("o")
            last_seen_utc  = $nowUtc.ToString("o")
            host_name      = [Environment]::MachineName
            repo_root      = [System.IO.Path]::GetFullPath($RepoRoot)
        }

        return [pscustomobject]@{
            SelectionMode    = $selection.SelectionMode
            SlotName         = $selection.SlotName
            TargetDir        = $selection.TargetDir
            AbsoluteTargetDir = $selection.AbsoluteTargetDir
            Reason           = $selection.Reason
            OwnerId          = $ownerId
        }
    }
}

function Resolve-DryRunTargetDir {
    param(
        [string]$RepoRoot,
        [string]$ManualTargetDir
    )

    if (-not [string]::IsNullOrWhiteSpace($ManualTargetDir)) {
        return [pscustomobject]@{
            SelectionMode     = "manual"
            SlotName          = $null
            TargetDir         = $ManualTargetDir
            AbsoluteTargetDir = Resolve-AbsoluteTargetDir -RepoRoot $RepoRoot -CliTargetDir $ManualTargetDir
            Reason            = "manual override"
            OwnerId           = $null
        }
    }

    return [pscustomobject]@{
        SelectionMode     = "dry-run"
        SlotName          = $null
        TargetDir         = $script:DryRunDefaultTargetDir
        AbsoluteTargetDir = Resolve-AbsoluteTargetDir -RepoRoot $RepoRoot -CliTargetDir $script:DryRunDefaultTargetDir
        Reason            = "dry-run default"
        OwnerId           = $null
    }
}

function Format-ByteCount {
    param([int64]$Bytes)

    return "{0:N2} GB" -f ($Bytes / 1GB)
}

function Get-TargetDriveInfo {
    param([string]$AbsoluteTargetDir)

    $driveRoot = [System.IO.Path]::GetPathRoot($AbsoluteTargetDir)
    if ([string]::IsNullOrWhiteSpace($driveRoot)) {
        throw "Could not determine drive root for target directory $AbsoluteTargetDir"
    }

    $drive = [System.IO.DriveInfo]::new($driveRoot)
    return [pscustomobject]@{
        DriveRoot = $driveRoot
        FreeBytes = [int64]$drive.AvailableFreeSpace
    }
}

function Get-PrebuildCleanupDecision {
    param(
        [int64]$FreeBytes,
        [int64]$ThresholdBytes = $script:LowDiskCleanupThresholdBytes
    )

    return [pscustomobject]@{
        FreeBytes       = $FreeBytes
        ThresholdBytes  = $ThresholdBytes
        RequiresCleanup = ($FreeBytes -le $ThresholdBytes)
    }
}

function Get-PrebuildCleanupStatus {
    param(
        [string]$AbsoluteTargetDir,
        [int64]$ThresholdBytes = $script:LowDiskCleanupThresholdBytes
    )

    $driveInfo = Get-TargetDriveInfo -AbsoluteTargetDir $AbsoluteTargetDir
    $decision = Get-PrebuildCleanupDecision -FreeBytes $driveInfo.FreeBytes -ThresholdBytes $ThresholdBytes

    return [pscustomobject]@{
        DriveRoot       = $driveInfo.DriveRoot
        FreeBytes       = $decision.FreeBytes
        ThresholdBytes  = $decision.ThresholdBytes
        RequiresCleanup = $decision.RequiresCleanup
    }
}

function Invoke-Step {
    param(
        [string]$Name,
        [scriptblock]$Action
    )

    Write-Host ""
    Write-Host "==> $Name" -ForegroundColor Cyan

    $code = 0
    $global:LASTEXITCODE = 0

    try {
        & $Action
        if ($LASTEXITCODE -ne 0) {
            $code = $LASTEXITCODE
        }
    } catch {
        $code = if ($LASTEXITCODE -ne 0) { $LASTEXITCODE } else { 1 }
        Write-Host $_.Exception.Message -ForegroundColor Red
    }

    $script:Results.Add([pscustomobject]@{
            Stage    = $Name
            ExitCode = $code
        }) | Out-Null

    if ($code -eq 0) {
        Write-Host "[OK] $Name" -ForegroundColor Green
    } else {
        Write-Host "[FAIL] $Name (exit $code)" -ForegroundColor Red
    }
}

function Get-CargoCleanArgs {
    param([string]$ResolvedTargetDir)

    return @("clean", "--target-dir", $ResolvedTargetDir)
}

function Get-CargoArgs {
    param(
        [string]$Subcommand,
        [string]$ResolvedTargetDir
    )

    $args = [System.Collections.Generic.List[string]]::new()
    $args.Add($Subcommand) | Out-Null

    if ([string]::IsNullOrWhiteSpace($Package)) {
        $args.Add("--workspace") | Out-Null
    } else {
        $args.Add("-p") | Out-Null
        $args.Add($Package) | Out-Null
    }

    if (-not $NoLocked) {
        $args.Add("--locked") | Out-Null
    }

    if ($VerboseOutput) {
        $args.Add("--verbose") | Out-Null
    }

    $args.Add("--target-dir") | Out-Null
    $args.Add($ResolvedTargetDir) | Out-Null

    return $args.ToArray()
}

function Get-ExportPlatformContractArgs {
    param(
        [string]$ResolvedTargetDir
    )

    $args = [System.Collections.Generic.List[string]]::new()
    $args.Add("test") | Out-Null
    $args.Add("-p") | Out-Null
    $args.Add("zircon_runtime") | Out-Null
    $args.Add("platform_target_policy_matches_host_resource_and_plugin_strategy") | Out-Null

    if (-not $NoLocked) {
        $args.Add("--locked") | Out-Null
    }

    if ($VerboseOutput) {
        $args.Add("--verbose") | Out-Null
    }

    $args.Add("--target-dir") | Out-Null
    $args.Add($ResolvedTargetDir) | Out-Null

    return $args.ToArray()
}

function Get-ProfileFeatureContractArgs {
    param(
        [object]$Case,
        [string]$ResolvedTargetDir
    )

    $args = [System.Collections.Generic.List[string]]::new()
    $args.Add("check") | Out-Null
    $args.Add("-p") | Out-Null
    $args.Add([string]$Case.Package) | Out-Null
    $args.Add("--no-default-features") | Out-Null
    $args.Add("--features") | Out-Null
    $args.Add([string]$Case.Features) | Out-Null

    if (-not $NoLocked) {
        $args.Add("--locked") | Out-Null
    }

    if ($VerboseOutput) {
        $args.Add("--verbose") | Out-Null
    }

    $args.Add("--target-dir") | Out-Null
    $args.Add($ResolvedTargetDir) | Out-Null

    return $args.ToArray()
}

function Get-SelectedProfileFeatureContractCases {
    param([string]$Label)

    if ([string]::IsNullOrWhiteSpace($Label)) {
        return $script:ProfileFeatureContractCases
    }

    $matches = @($script:ProfileFeatureContractCases | Where-Object { $_.Label -eq $Label })
    if ($matches.Count -eq 0) {
        $knownLabels = ($script:ProfileFeatureContractCases | ForEach-Object { $_.Label }) -join ", "
        throw "Unknown profile feature contract label '$Label'. Known labels: $knownLabels"
    }

    return $matches
}

function Get-SelectedExportContractPlatforms {
    param([string]$Platform)

    if ([string]::IsNullOrWhiteSpace($Platform)) {
        return $script:ExportContractPlatforms
    }

    $matches = @($script:ExportContractPlatforms | Where-Object { $_ -eq $Platform })
    if ($matches.Count -eq 0) {
        $knownPlatforms = $script:ExportContractPlatforms -join ", "
        throw "Unknown export contract platform '$Platform'. Known platforms: $knownPlatforms"
    }

    return $matches
}

function Format-Command {
    param([string[]]$Arguments)

    $rendered = foreach ($argument in $Arguments) {
        if ($argument -match '\s') {
            '"{0}"' -f ($argument -replace '"', '""')
        } else {
            $argument
        }
    }

    return "cargo {0}" -f ($rendered -join " ")
}

function Invoke-Cargo {
    param([string[]]$Arguments)

    Write-Host (Format-Command -Arguments $Arguments) -ForegroundColor DarkGray

    if ($DryRun) {
        return
    }

    & cargo @Arguments
}

function Invoke-CargoWithEnvironment {
    param(
        [string[]]$Arguments,
        [hashtable]$Environment
    )

    Write-Host (Format-Command -Arguments $Arguments) -ForegroundColor DarkGray

    if ($DryRun) {
        if ($Environment) {
            foreach ($entry in $Environment.GetEnumerator() | Sort-Object Name) {
                Write-Host ("  env:{0}={1}" -f $entry.Name, $entry.Value) -ForegroundColor DarkGray
            }
        }
        return
    }

    $previousValues = @{}
    try {
        foreach ($entry in $Environment.GetEnumerator()) {
            $previousValues[$entry.Name] = [Environment]::GetEnvironmentVariable($entry.Name, "Process")
            [Environment]::SetEnvironmentVariable($entry.Name, [string]$entry.Value, "Process")
        }

        & cargo @Arguments
    } finally {
        foreach ($entry in $Environment.GetEnumerator()) {
            [Environment]::SetEnvironmentVariable($entry.Name, $previousValues[$entry.Name], "Process")
        }
    }
}

function Invoke-ValidateMatrixMain {
    $script:Results = [System.Collections.Generic.List[object]]::new()

    if (-not $RunExportPlatformContract -and -not [string]::IsNullOrWhiteSpace($ExportContractPlatform)) {
        throw "-ExportContractPlatform requires -RunExportPlatformContract."
    }

    if (-not $RunProfileFeatureContract -and -not [string]::IsNullOrWhiteSpace($ProfileFeatureContractLabel)) {
        throw "-ProfileFeatureContractLabel requires -RunProfileFeatureContract."
    }

    $resolvedRepoRoot = if ($RepoRoot) {
        [System.IO.Path]::GetFullPath((Resolve-Path $RepoRoot).Path)
    } else {
        Find-RepoRoot $PSScriptRoot
    }

    $resolvedTarget = if ($DryRun) {
        Resolve-DryRunTargetDir -RepoRoot $resolvedRepoRoot -ManualTargetDir $TargetDir
    } else {
        Resolve-EffectiveTargetDir -RepoRoot $resolvedRepoRoot -ManualTargetDir $TargetDir
    }

    Write-Host "Repo root: $resolvedRepoRoot"
    Write-Host ("Scope: {0}" -f $(if ([string]::IsNullOrWhiteSpace($Package)) { "workspace" } else { "package $Package" }))
    Write-Host ("Locked mode: {0}" -f $(if ($NoLocked) { "off" } else { "on" }))
    Write-Host ("Dry run: {0}" -f $(if ($DryRun) { "on" } else { "off" }))
    Write-Host ("Target dir: {0} ({1})" -f $resolvedTarget.TargetDir, $resolvedTarget.Reason)

    $cleanupStatus = $null
    if ($SkipBuild -and $SkipTest -and -not $RunExportPlatformContract -and -not $RunProfileFeatureContract) {
        Write-Host "No stages selected. Use this mode to sanity-check script parsing or argument handling." -ForegroundColor Yellow
    } elseif ($DryRun) {
        Write-Host "Dry run selected; skipping cargo discovery and target directory cleanup checks." -ForegroundColor Yellow
    } else {
        Get-Command cargo -ErrorAction Stop | Out-Null
        $cleanupStatus = Get-PrebuildCleanupStatus -AbsoluteTargetDir $resolvedTarget.AbsoluteTargetDir
        Write-Host ("Free space on {0}: {1} (threshold {2})" -f $cleanupStatus.DriveRoot, (Format-ByteCount -Bytes $cleanupStatus.FreeBytes), (Format-ByteCount -Bytes $cleanupStatus.ThresholdBytes))
    }

    Push-Location $resolvedRepoRoot
    try {
        if ($null -ne $cleanupStatus -and $cleanupStatus.RequiresCleanup) {
            Write-Host ("Free space is at or below the cleanup threshold. Running cargo clean before build/test.") -ForegroundColor Yellow
            Invoke-Step "Cargo clean" {
                Invoke-Cargo -Arguments (Get-CargoCleanArgs -ResolvedTargetDir $resolvedTarget.TargetDir)
            }

            if (($Results | Select-Object -Last 1).ExitCode -ne 0) {
                return 1
            }
        }

        if (-not $SkipBuild) {
            Invoke-Step "Cargo build" {
                Invoke-Cargo -Arguments (Get-CargoArgs -Subcommand "build" -ResolvedTargetDir $resolvedTarget.TargetDir)
            }
        }

        if (-not $SkipTest) {
            Invoke-Step "Cargo test" {
                Invoke-Cargo -Arguments (Get-CargoArgs -Subcommand "test" -ResolvedTargetDir $resolvedTarget.TargetDir)
            }
        }

        if ($RunExportPlatformContract) {
            foreach ($platform in (Get-SelectedExportContractPlatforms -Platform $ExportContractPlatform)) {
                Invoke-Step "Export platform contract ($platform)" {
                    Invoke-CargoWithEnvironment `
                        -Arguments (Get-ExportPlatformContractArgs -ResolvedTargetDir $resolvedTarget.TargetDir) `
                        -Environment @{ ZR_EXPORT_CONTRACT_PLATFORM = $platform }
                }
            }
        }

        if ($RunProfileFeatureContract) {
            foreach ($case in (Get-SelectedProfileFeatureContractCases -Label $ProfileFeatureContractLabel)) {
                Invoke-Step "Profile feature contract ($($case.Label))" {
                    Invoke-Cargo -Arguments (Get-ProfileFeatureContractArgs `
                        -Case $case `
                        -ResolvedTargetDir $resolvedTarget.TargetDir)
                }
            }
        }
    } finally {
        Pop-Location
    }

    Write-Host ""
    Write-Host "Summary" -ForegroundColor Cyan
    if ($Results.Count -eq 0) {
        Write-Host "No build or test stages were requested."
    } else {
        foreach ($result in $Results) {
            $status = if ($result.ExitCode -eq 0) { "OK" } else { "FAIL" }
            Write-Host ("{0,-20} {1}" -f $result.Stage, $status)
        }
    }

    $failed = $Results | Where-Object { $_.ExitCode -ne 0 }
    if ($failed) {
        return 1
    }

    return 0
}

function Test-ScriptIsDotSourced {
    return $MyInvocation.InvocationName -eq "."
}

if ($env:VALIDATE_MATRIX_TEST_MODE -ne "1") {
    $exitCode = Invoke-ValidateMatrixMain
    if (-not (Test-ScriptIsDotSourced)) {
        exit $exitCode
    }
}
