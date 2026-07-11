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

$script:LowDiskCleanupThresholdBytes = 50GB
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

function Resolve-OwnerId {
    param([string]$RepoRoot)

    if (-not [string]::IsNullOrWhiteSpace($env:CODEX_THREAD_ID)) {
        return $env:CODEX_THREAD_ID
    }

    $user = [Environment]::UserName
    $machine = [Environment]::MachineName
    $repoId = [System.IO.Path]::GetFullPath($RepoRoot).TrimEnd('\', '/').ToLowerInvariant()
    return "manual:{0}@{1}:{2}" -f $user, $machine, $repoId
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

function Invoke-SessionCoordinatorJson {
    param(
        [string]$RepoRoot,
        [string[]]$Arguments
    )

    $client = Join-Path $RepoRoot "tools\zircon-session.ps1"
    if (-not (Test-Path -LiteralPath $client)) {
        throw "Session coordinator client is missing: $client"
    }
    $command = $Arguments[0]
    $remaining = if ($Arguments.Count -gt 1) { $Arguments[1..($Arguments.Count - 1)] } else { @() }
    $raw = & $client -Command $command -RepoRoot $RepoRoot -Json @remaining
    if ($LASTEXITCODE -ne 0) {
        throw "Session coordinator command failed: $($raw -join [Environment]::NewLine)"
    }
    return (($raw -join [Environment]::NewLine) | ConvertFrom-Json)
}

function Resolve-CoordinatorCargoTarget {
    param(
        [string]$RepoRoot,
        [string]$ManualTargetDir,
        [string]$LaneKind,
        [switch]$DryRunMode
    )

    $ownerId = Resolve-OwnerId -RepoRoot $RepoRoot
    Invoke-SessionCoordinatorJson -RepoRoot $RepoRoot -Arguments @(
        "session", "register", "--session-id", $ownerId,
        "--display-name", "validate-matrix", "--write-scope", "Cargo validation"
    ) | Out-Null

    $requestedTarget = $ManualTargetDir
    $selectionMode = "managed"
    if ([string]::IsNullOrWhiteSpace($requestedTarget) -and -not [string]::IsNullOrWhiteSpace($env:CARGO_TARGET_DIR)) {
        $requestedTarget = $env:CARGO_TARGET_DIR
        $selectionMode = "environment"
    } elseif (-not [string]::IsNullOrWhiteSpace($requestedTarget)) {
        $selectionMode = "manual"
    }

    $arguments = @(
        "cargo", "acquire", $LaneKind,
        "--session-id", $ownerId,
        "--pid", [string]$PID
    )
    if (-not [string]::IsNullOrWhiteSpace($requestedTarget)) {
        $absoluteRequestedTarget = Resolve-AbsoluteTargetDir -RepoRoot $RepoRoot -CliTargetDir $requestedTarget
        $arguments += @("--target-dir", $absoluteRequestedTarget)
    }
    if ($DryRunMode) {
        $arguments += "--dry-run"
    }
    $response = Invoke-SessionCoordinatorJson -RepoRoot $RepoRoot -Arguments $arguments
    $reason = if ($selectionMode -eq "managed") {
        "coordinator managed $LaneKind lane"
    } else {
        "coordinator validated $selectionMode target"
    }
    return [pscustomobject]@{
        SelectionMode     = $selectionMode
        SlotName          = $null
        JobId             = [string]$response.job.job_id
        TargetDir         = [string]$response.job.target_dir
        AbsoluteTargetDir = [string]$response.job.target_dir
        Reason            = $reason
        OwnerId           = $ownerId
        DryRun            = [bool]$response.job.dry_run
    }
}

function Start-CoordinatorCargoTarget {
    param([string]$RepoRoot, [object]$ResolvedTarget)

    if ($ResolvedTarget.DryRun) {
        return
    }
    Invoke-SessionCoordinatorJson -RepoRoot $RepoRoot -Arguments @(
        "cargo", "start", $ResolvedTarget.JobId,
        "--pid", [string]$PID,
        "--session-id", $ResolvedTarget.OwnerId,
        [Environment]::CommandLine
    ) | Out-Null
}

function Complete-CoordinatorCargoTarget {
    param(
        [string]$RepoRoot,
        [object]$ResolvedTarget,
        [int]$ExitCode,
        [switch]$Started
    )

    if ($Started) {
        Invoke-SessionCoordinatorJson -RepoRoot $RepoRoot -Arguments @(
            "cargo", "finish", $ResolvedTarget.JobId,
            "--exit-code", [string]$ExitCode,
            "--session-id", $ResolvedTarget.OwnerId
        ) | Out-Null
    }
    Invoke-SessionCoordinatorJson -RepoRoot $RepoRoot -Arguments @(
        "cargo", "release", $ResolvedTarget.JobId,
        "--session-id", $ResolvedTarget.OwnerId
    ) | Out-Null
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

    $laneKind = if (-not [string]::IsNullOrWhiteSpace($Package)) {
        if ($SkipTest) { "check" } else { "test" }
    } else {
        "workspace"
    }
    $resolvedTarget = Resolve-CoordinatorCargoTarget `
        -RepoRoot $resolvedRepoRoot `
        -ManualTargetDir $TargetDir `
        -LaneKind $laneKind `
        -DryRunMode:$DryRun

    $coordinatorJobFailed = $false
    $coordinatorJobStarted = $false
    $locationPushed = $false
    try {
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

    Start-CoordinatorCargoTarget -RepoRoot $resolvedRepoRoot -ResolvedTarget $resolvedTarget
    $coordinatorJobStarted = -not $resolvedTarget.DryRun
    Push-Location $resolvedRepoRoot
    $locationPushed = $true
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
    } catch {
        $coordinatorJobFailed = $true
        throw
    } finally {
        if ($locationPushed) {
            Pop-Location
        }
        $jobExitCode = if ($coordinatorJobFailed -or ($Results | Where-Object { $_.ExitCode -ne 0 })) { 1 } else { 0 }
        Complete-CoordinatorCargoTarget `
            -RepoRoot $resolvedRepoRoot `
            -ResolvedTarget $resolvedTarget `
            -ExitCode $jobExitCode `
            -Started:$coordinatorJobStarted
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
