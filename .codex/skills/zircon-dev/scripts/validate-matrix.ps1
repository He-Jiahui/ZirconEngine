[CmdletBinding()]
param(
    [string]$RepoRoot,
    [string]$ManifestPath,
    [string]$Package,
    [string]$TargetDir,
    [string]$Features,
    [switch]$NoDefaultFeatures,
    [switch]$Ephemeral,
    [switch]$SkipBuild,
    [switch]$SkipTest,
    [switch]$LibTests,
    [string]$TestTarget,
    [string]$Bin,
    [string]$ArtifactOutputDirectory,
    [string[]]$PublishArtifact,
    [switch]$MvpProductInputArtifactOutput,
    [string]$TestFilter,
    [switch]$IgnoredTests,
    [switch]$RunExportPlatformContract,
    [string]$ExportContractPlatform,
    [switch]$RunProfileFeatureContract,
    [string]$ProfileFeatureContractLabel,
    [switch]$RunConventionStructure,
    [switch]$RunConventionClippy,
    [ValidateSet("development", "release", "profiling")]
    [string]$CargoProfile = "development",
    [ValidateSet("reuse", "compact", "diagnostic")]
    [string]$StorageMode = "reuse",
    [switch]$NoLocked,
    [switch]$VerboseOutput,
    [switch]$DryRun
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$windowsPathResolverRepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
Import-Module (Join-Path $windowsPathResolverRepoRoot "tools\WindowsPathResolver.psm1") -Force -ErrorAction Stop
. (Join-Path $PSScriptRoot "managed-cargo-storage.ps1")

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
        Bin = $null
    },
    [pscustomobject]@{
        Label = "zircon_app target-client-platform"
        Package = "zircon_app"
        Features = "target-client,platform-winit,input-gamepad,gamepad-gilrs"
        Bin = "zircon_runtime"
    },
    [pscustomobject]@{
        Label = "zircon_app target-editor-host"
        Package = "zircon_app"
        Features = "target-editor-host"
        Bin = "zircon_editor"
    },
    [pscustomobject]@{
        Label = "zircon_app target-client shader-pbr-viewer"
        Package = "zircon_app"
        Features = "target-client,platform-winit,input-gamepad,gamepad-gilrs"
        Bin = "zircon_shader_pbr_viewer"
    },
    [pscustomobject]@{
        Label = "zircon_runtime target-client"
        Package = "zircon_runtime"
        Features = "target-client"
        Bin = $null
    },
    [pscustomobject]@{
        Label = "zircon_runtime target-editor-host"
        Package = "zircon_runtime"
        Features = "target-editor-host"
        Bin = $null
    },
    [pscustomobject]@{
        Label = "zircon_runtime target-server"
        Package = "zircon_runtime"
        Features = "target-server"
        Bin = $null
    }
)

function Find-RepoRoot {
    param([string]$StartPath)

    $current = (Resolve-ZirconWindowsPath -Path $StartPath).DisplayExistingPath
    while ($true) {
        if ((Test-Path (Join-ZirconWindowsPath -Path $current -ChildPath "Cargo.toml")) -and
            (Test-Path (Join-ZirconWindowsPath -Path $current -ChildPath ".codex\skills\zircon-dev"))) {
            return $current
        }

        $parent = Split-Path $current -Parent
        if ([string]::IsNullOrWhiteSpace($parent) -or $parent -eq $current) {
            throw "Could not locate repository root from $StartPath"
        }

        $current = (Resolve-ZirconWindowsPath -Path $parent).DisplayExistingPath
    }
}

function Resolve-OwnerId {
    param([string]$RepoRoot)

    if (-not [string]::IsNullOrWhiteSpace($env:CODEX_THREAD_ID)) {
        return $env:CODEX_THREAD_ID
    }

    $user = [Environment]::UserName
    $machine = [Environment]::MachineName
    $repoId = (Resolve-ZirconWindowsPath -Path $RepoRoot).DisplayExistingPath.TrimEnd('\', '/').ToLowerInvariant()
    return "manual:{0}@{1}:{2}" -f $user, $machine, $repoId
}

function Resolve-ValidationSessionId {
    param([string]$RepoRoot)

    # Cargo lane ownership is operational state, not ownership of the caller's source scope.
    # Keep it stable per primary Session while allowing the primary to retain its immutable paths.
    return "validate-matrix:{0}" -f (Resolve-OwnerId -RepoRoot $RepoRoot)
}

function Register-ValidationSession {
    param(
        [string]$RepoRoot,
        [string]$SessionId
    )

    $registered = Invoke-SessionCoordinatorJson -RepoRoot $RepoRoot -Arguments @(
        "session", "register", "--session-id", $SessionId,
        "--display-name", "validate-matrix", "--write-scope", "Cargo validation"
    )
    $registeredSessionId = [string](Require-CoordinatorResponseField `
        -Response $registered `
        -Command "session register" `
        -FieldPath "session.session_id")
    if ($registeredSessionId -ne $SessionId) {
        throw "Session coordinator command 'session register' returned unexpected session id '$registeredSessionId'; expected '$SessionId'."
    }
    return $registered.session
}

function Resolve-AbsoluteTargetDir {
    param(
        [string]$RepoRoot,
        [string]$CliTargetDir
    )

    return (Resolve-ZirconWindowsPath -Path $CliTargetDir -BasePath $RepoRoot).DisplayPath
}

function Resolve-ManagedCargoTargetPath {
    param([Parameter(Mandatory)][string]$TargetDirectory)

    $targetResolution = Resolve-ZirconWindowsPath -Path $TargetDirectory
    if ($targetResolution.DisplayPath -notmatch '^[D-F]:\\') {
        throw "Managed Cargo target must physically resolve under D:, E:, or F:, not '$($targetResolution.DisplayPath)'."
    }
    if ($targetResolution.DisplayPath -notmatch '^[D-F]:\\(?:cargo-targets|targets|ZirconBuilds)(?:\\|$)') {
        throw "Managed Cargo target must resolve under an approved root such as D:\cargo-targets, D:\targets, or D:\ZirconBuilds, not '$($targetResolution.DisplayPath)'."
    }
    return $targetResolution
}

function Resolve-WorkspaceManifest {
    param(
        [string]$RepoRoot,
        [string]$RequestedManifestPath
    )

    $relativePath = if ([string]::IsNullOrWhiteSpace($RequestedManifestPath)) {
        "Cargo.toml"
    } else {
        $RequestedManifestPath.Trim().Replace('\', '/')
    }
    if ([System.IO.Path]::IsPathRooted($relativePath)) {
        throw "-ManifestPath must be repository-relative."
    }

    $rootResolution = Resolve-ZirconWindowsPath -Path $RepoRoot
    $candidateResolution = Resolve-ZirconWindowsPath -Path $relativePath -BasePath $RepoRoot
    $resolvedRoot = $rootResolution.OperationalExistingPath.TrimEnd('\', '/')
    $candidate = $candidateResolution.OperationalPath
    if (-not $candidate.StartsWith($resolvedRoot + '\', [StringComparison]::OrdinalIgnoreCase)) {
        throw "-ManifestPath must remain inside the repository root."
    }
    if ([System.IO.Path]::GetFileName($candidateResolution.DisplayPath) -ine "Cargo.toml") {
        throw "-ManifestPath must name a Cargo.toml file."
    }
    if (-not (Test-Path -LiteralPath $candidateResolution.DisplayPath -PathType Leaf)) {
        throw "-ManifestPath does not exist: $relativePath"
    }

    return [pscustomobject]@{
        RelativePath           = $candidate.Substring($resolvedRoot.Length).TrimStart('\', '/').Replace('\', '/')
        Directory              = Split-Path -Parent $candidateResolution.DisplayPath
        InvocationManifestPath = [System.IO.Path]::GetFileName($candidateResolution.DisplayPath)
    }
}

function Get-RustCompatibilityIdentity {
    param([switch]$DryRunMode)

    if ($DryRunMode) {
        $toolchain = if ([string]::IsNullOrWhiteSpace($env:RUSTUP_TOOLCHAIN)) {
            "dry-run-unresolved-windows-toolchain"
        } else {
            $env:RUSTUP_TOOLCHAIN
        }
        $target = if ([string]::IsNullOrWhiteSpace($env:CARGO_BUILD_TARGET)) {
            "dry-run-unresolved-windows-target"
        } else {
            $env:CARGO_BUILD_TARGET
        }
        return [pscustomobject]@{ Toolchain = $toolchain; TargetArchitecture = $target }
    }

    Get-Command rustc -ErrorAction Stop | Out-Null
    $versionText = @(& rustc -vV)
    if ($LASTEXITCODE -ne 0) {
        throw "Could not determine the active Rust toolchain."
    }
    $release = [string](($versionText | Where-Object { $_ -match '^release:\s*' }) -replace '^release:\s*', '')
    $rustHost = [string](($versionText | Where-Object { $_ -match '^host:\s*' }) -replace '^host:\s*', '')
    if ([string]::IsNullOrWhiteSpace($release) -or [string]::IsNullOrWhiteSpace($rustHost)) {
        throw "rustc -vV did not report both release and host identities."
    }
    $target = if ([string]::IsNullOrWhiteSpace($env:CARGO_BUILD_TARGET)) {
        $rustHost
    } else {
        $env:CARGO_BUILD_TARGET
    }
    return [pscustomobject]@{
        Toolchain         = "$release@$rustHost"
        TargetArchitecture = $target
    }
}

function New-CargoCompatibilityJson {
    param(
        [string]$ResolvedRepoRoot,
        [string]$WorkspaceManifest = "Cargo.toml",
        [ValidateSet("development", "release", "profiling")]
        [string]$CargoProfile = "development",
        [ValidateSet("reuse", "compact", "diagnostic")]
        [string]$StorageMode = "reuse",
        [switch]$DryRunMode
    )

    $rust = Get-RustCompatibilityIdentity -DryRunMode:$DryRunMode
    $compactOutputs = $StorageMode -in @("reuse", "compact")
    $buildDirectoryIdentity = switch ($StorageMode) {
        "reuse" { "persistent-target-v1" }
        "compact" { "ephemeral-v1" }
        default {
            if ([string]::IsNullOrWhiteSpace($env:CARGO_BUILD_BUILD_DIR)) {
                "cargo-default"
            } else {
                [string]$env:CARGO_BUILD_BUILD_DIR
            }
        }
    }
    $configuration = [ordered]@{
        profile_feature_contract = if ($RunProfileFeatureContract) {
            if ([string]::IsNullOrWhiteSpace($ProfileFeatureContractLabel)) { "all" } else { $ProfileFeatureContractLabel }
        } else { "off" }
        export_platform_contract = if ($RunExportPlatformContract) {
            if ([string]::IsNullOrWhiteSpace($ExportContractPlatform)) { "all" } else { $ExportContractPlatform }
        } else { "off" }
        rustflags = [string]$env:RUSTFLAGS
        storage_mode = $StorageMode
        cargo_incremental = if ($compactOutputs) { "0" } else { [string]$env:CARGO_INCREMENTAL }
        cargo_profile = $CargoProfile
        build_dir = $buildDirectoryIdentity
        compiler_cache = if ($compactOutputs) { "sccache" } else { "optional-sccache" }
        dev_debug = if ($compactOutputs) { "0" } else { [string]$env:CARGO_PROFILE_DEV_DEBUG }
        test_debug = if ($compactOutputs) { "0" } else { [string]$env:CARGO_PROFILE_TEST_DEBUG }
        release_debug = [string]$env:CARGO_PROFILE_RELEASE_DEBUG
        profiling_debug = [string]$env:CARGO_PROFILE_PROFILING_DEBUG
    }
    $compatibility = [ordered]@{
        platform = "windows"
        toolchain = $rust.Toolchain
        target_architecture = $rust.TargetArchitecture
        workspace = $WorkspaceManifest
        build_config = ($configuration | ConvertTo-Json -Compress)
    }
    return ($compatibility | ConvertTo-Json -Compress)
}

function Get-ManagedTextSha256 {
    param([Parameter(Mandatory)][string]$Text)

    $algorithm = [System.Security.Cryptography.SHA256]::Create()
    try {
        $bytes = [System.Text.Encoding]::UTF8.GetBytes($Text)
        return [System.BitConverter]::ToString($algorithm.ComputeHash($bytes)).Replace("-", "").ToLowerInvariant()
    }
    finally {
        $algorithm.Dispose()
    }
}

function Resolve-DryRunCargoTargetPath {
    param(
        [Parameter(Mandatory)][string]$RepoRoot,
        [Parameter(Mandatory)][string]$CompatibilityJson
    )

    $repo = Resolve-ZirconWindowsPath -Path $RepoRoot
    $driveRoot = [System.IO.Path]::GetPathRoot($repo.DisplayPath)
    if ($driveRoot -notmatch '^[D-F]:\\$') {
        $driveRoot = "D:\"
    }
    $identity = "{0}`0compile-pool-v2`0{1}" -f $repo.DisplayPath.ToLowerInvariant(), $CompatibilityJson
    $fingerprint = Get-ManagedTextSha256 -Text $identity
    $target = Join-Path $driveRoot ("cargo-targets\zircon-engine\pool\{0}" -f $fingerprint)
    return [pscustomobject]@{
        Fingerprint = $fingerprint
        TargetDir   = (Resolve-ManagedCargoTargetPath -TargetDirectory $target).DisplayPath
    }
}

function Get-CoordinatorResponseSummary {
    param([string]$RawResponse)

    $singleLine = ($RawResponse -replace "[\r\n]+", " ").Trim()
    if ($singleLine.Length -le 400) {
        return $singleLine
    }
    return $singleLine.Substring(0, 400) + "..."
}

function ConvertFrom-StrictCoordinatorJson {
    param(
        [string]$Command,
        [string[]]$RawOutput
    )

    $rawResponse = ($RawOutput -join [Environment]::NewLine).Trim()
    if ([string]::IsNullOrWhiteSpace($rawResponse)) {
        throw "Session coordinator command '$Command' returned empty stdout; expected exactly one JSON document."
    }

    try {
        # Windows PowerShell 5.1 does not provide System.Text.Json. ConvertFrom-Json still
        # rejects trailing documents, but it enumerates array roots and turns [] into null.
        # Guard the root token before parsing so that behavior cannot hide a non-object root.
        if (-not $rawResponse.StartsWith("{")) {
            throw "the JSON root must be an object"
        }
        $response = $rawResponse | ConvertFrom-Json -ErrorAction Stop
        if ($null -eq $response -or $response -is [array] -or $response -is [string] -or $response -is [ValueType]) {
            throw "the JSON root must be an object"
        }
        return $response
    }
    catch {
        $summary = Get-CoordinatorResponseSummary -RawResponse $rawResponse
        throw "Session coordinator command '$Command' must return exactly one JSON document; response summary: '$summary'; parse error: $($_.Exception.Message)"
    }
}

function Require-CoordinatorResponseField {
    param(
        [object]$Response,
        [string]$Command,
        [string]$FieldPath
    )

    $value = $Response
    foreach ($segment in $FieldPath.Split('.')) {
        if ($null -eq $value) {
            throw "Session coordinator command '$Command' returned a response missing required field '$FieldPath'."
        }
        $property = $value.PSObject.Properties[$segment]
        if ($null -eq $property -or $null -eq $property.Value) {
            throw "Session coordinator command '$Command' returned a response missing required field '$FieldPath'."
        }
        $value = $property.Value
    }
    if ($value -is [string] -and [string]::IsNullOrWhiteSpace($value)) {
        throw "Session coordinator command '$Command' returned an empty required field '$FieldPath'."
    }
    return $value
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
    return ConvertFrom-StrictCoordinatorJson -Command $command -RawOutput $raw
}

function Resolve-CoordinatorCargoTarget {
    param(
        [string]$RepoRoot,
        [string]$ManualTargetDir,
        [string]$LaneKind,
        [string]$WorkspaceManifest,
        [ValidateSet("development", "release", "profiling")]
        [string]$CargoProfile = "development",
        [ValidateSet("reuse", "compact", "diagnostic")]
        [string]$StorageMode = "reuse",
        [AllowEmptyString()]
        [string]$PrecomputedCompatibilityJson,
        [switch]$EphemeralLane,
        [switch]$DryRunMode
    )

    $ownerId = Resolve-ValidationSessionId -RepoRoot $RepoRoot
    $compatibilityJson = if ([string]::IsNullOrWhiteSpace($PrecomputedCompatibilityJson)) {
        New-CargoCompatibilityJson `
            -ResolvedRepoRoot $RepoRoot `
            -WorkspaceManifest $WorkspaceManifest `
            -CargoProfile $CargoProfile `
            -StorageMode $StorageMode `
            -DryRunMode:$DryRunMode
    } else {
        $PrecomputedCompatibilityJson
    }
    $requestedTarget = $ManualTargetDir
    $selectionMode = "managed"
    if ([string]::IsNullOrWhiteSpace($requestedTarget) -and -not [string]::IsNullOrWhiteSpace($env:CARGO_TARGET_DIR)) {
        $requestedTarget = $env:CARGO_TARGET_DIR
        $selectionMode = "environment"
    } elseif (-not [string]::IsNullOrWhiteSpace($requestedTarget)) {
        $selectionMode = "manual"
    }
    if ($EphemeralLane -and -not [string]::IsNullOrWhiteSpace($requestedTarget)) {
        throw "-Ephemeral cannot be combined with -TargetDir or CARGO_TARGET_DIR."
    }

    $resolvedRequestedTarget = $null
    if (-not [string]::IsNullOrWhiteSpace($requestedTarget)) {
        $resolvedRequestedTarget = Resolve-AbsoluteTargetDir `
            -RepoRoot $RepoRoot `
            -CliTargetDir $requestedTarget
        $resolvedRequestedTarget = (Resolve-ManagedCargoTargetPath `
            -TargetDirectory $resolvedRequestedTarget).DisplayPath
    }
    $reason = if ($EphemeralLane) {
        "coordinator managed ephemeral $LaneKind lane"
    } elseif ($selectionMode -eq "managed") {
        "coordinator managed $LaneKind lane"
    } else {
        "coordinator validated $selectionMode target"
    }
    if ($DryRunMode) {
        $projection = Resolve-DryRunCargoTargetPath `
            -RepoRoot $RepoRoot `
            -CompatibilityJson $compatibilityJson
        $targetDir = if ($null -ne $resolvedRequestedTarget) {
            $resolvedRequestedTarget
        } else {
            $projection.TargetDir
        }
        return [pscustomobject]@{
            SelectionMode     = $selectionMode
            SlotName          = $null
            JobId             = "dry-run-$($projection.Fingerprint)"
            TargetDir         = $targetDir
            AbsoluteTargetDir = $targetDir
            Reason            = $reason
            OwnerId           = $ownerId
            DryRun            = $true
        }
    }

    $registeredSession = Register-ValidationSession -RepoRoot $RepoRoot -SessionId $ownerId
    $nonExecutableStatuses = @("stale", "completed", "archived", "cancelled")
    if ([string]$registeredSession.status -in $nonExecutableStatuses) {
        # Terminal primary work remains immutable; a new operational child owns this Cargo run.
        $ownerId = "{0}:successor:{1}" -f $ownerId, [guid]::NewGuid().ToString("N")
        $registeredSession = Register-ValidationSession -RepoRoot $RepoRoot -SessionId $ownerId
    }

    $arguments = @(
        "cargo", "acquire", $LaneKind,
        "--session-id", $ownerId
    )
    if (-not $EphemeralLane) {
        $arguments += @("--compatibility-json", $compatibilityJson)
    }
    if (-not $DryRunMode) {
        $arguments += @("--pid", [string]$PID)
    }
    if ($null -ne $resolvedRequestedTarget) {
        $arguments += @("--target-dir", $resolvedRequestedTarget)
    }
    if ($EphemeralLane) {
        $arguments += "--ephemeral"
    }
    $response = Invoke-SessionCoordinatorJson -RepoRoot $RepoRoot -Arguments $arguments
    $jobId = [string](Require-CoordinatorResponseField `
        -Response $response `
        -Command "cargo acquire" `
        -FieldPath "job.job_id")
    try {
        $targetDir = [string](Require-CoordinatorResponseField `
            -Response $response `
            -Command "cargo acquire" `
            -FieldPath "job.target_dir")
        $dryRun = [bool](Require-CoordinatorResponseField `
            -Response $response `
            -Command "cargo acquire" `
            -FieldPath "job.dry_run")
        $targetDir = (Resolve-ManagedCargoTargetPath -TargetDirectory $targetDir).DisplayPath
    }
    catch {
        $resolutionFailure = $_
        try {
            Invoke-SessionCoordinatorJson -RepoRoot $RepoRoot -Arguments @(
                "cargo", "release", $jobId,
                "--session-id", $ownerId
            ) | Out-Null
        }
        catch {
            Write-Warning `
                ("Coordinator release also failed after target resolution failed: {0}" -f $_.Exception.Message) `
                -WarningAction Continue
        }
        throw $resolutionFailure
    }
    return [pscustomobject]@{
        SelectionMode     = $selectionMode
        SlotName          = $null
        JobId             = $jobId
        TargetDir         = $targetDir
        AbsoluteTargetDir = $targetDir
        Reason            = $reason
        OwnerId           = $ownerId
        DryRun            = $dryRun
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
        "--supervisor",
        "--session-id", $ResolvedTarget.OwnerId,
        [Environment]::CommandLine
    ) | Out-Null
}

function Complete-CoordinatorCargoTarget {
    param(
        [string]$RepoRoot,
        [object]$ResolvedTarget,
        [int]$ExitCode,
        [switch]$StartAttempted
    )

    if ($ResolvedTarget.DryRun) {
        return
    }
    if (-not $StartAttempted) {
        Invoke-SessionCoordinatorJson -RepoRoot $RepoRoot -Arguments @(
            "cargo", "release", $ResolvedTarget.JobId,
            "--session-id", $ResolvedTarget.OwnerId
        ) | Out-Null
        return
    }

    $finishFailure = $null
    try {
        Invoke-SessionCoordinatorJson -RepoRoot $RepoRoot -Arguments @(
            "cargo", "finish", $ResolvedTarget.JobId,
            "--exit-code", [string]$ExitCode,
            "--session-id", $ResolvedTarget.OwnerId
        ) | Out-Null
    }
    catch {
        $finishFailure = $_
    }

    try {
        Invoke-SessionCoordinatorJson -RepoRoot $RepoRoot -Arguments @(
            "cargo", "release", $ResolvedTarget.JobId,
            "--session-id", $ResolvedTarget.OwnerId
        ) | Out-Null
    }
    catch {
        if ($null -eq $finishFailure) {
            throw
        }
    }

    if ($null -ne $finishFailure) {
        throw $finishFailure
    }
}

function Resolve-ValidationCleanupFailure {
    param(
        [AllowNull()][object]$CleanupFailure,
        [AllowNull()][object]$PrimaryFailure,
        [switch]$HasFailedStep
    )

    if ($null -eq $CleanupFailure) {
        return
    }
    if ($null -ne $PrimaryFailure) {
        Write-Warning `
            ("Coordinator cleanup also failed after the primary validation error: {0}" -f $CleanupFailure.Exception.Message) `
            -WarningAction Continue
        return
    }
    if ($HasFailedStep) {
        Write-Warning `
            ("Coordinator cleanup also failed after a validation stage returned nonzero: {0}" -f $CleanupFailure.Exception.Message) `
            -WarningAction Continue
        return
    }

    throw $CleanupFailure
}

function Format-ByteCount {
    param([int64]$Bytes)

    return "{0:N2} GB" -f ($Bytes / 1GB)
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

function Add-CargoProfileArguments {
    param(
        [System.Collections.Generic.List[string]]$Arguments,
        [ValidateSet("development", "release", "profiling")]
        [string]$CargoProfile = "development"
    )

    switch ($CargoProfile) {
        "release" {
            $Arguments.Add("--release") | Out-Null
        }
        "profiling" {
            $Arguments.Add("--profile") | Out-Null
            $Arguments.Add("profiling") | Out-Null
        }
    }
}

function Get-CargoArgs {
    param(
        [string]$Subcommand,
        [string]$ResolvedTargetDir,
        [string]$WorkspaceManifest,
        [ValidateSet("development", "release", "profiling")]
        [string]$CargoProfile = "development"
    )

    $args = [System.Collections.Generic.List[string]]::new()
    $args.Add($Subcommand) | Out-Null

    if ($WorkspaceManifest -ne "Cargo.toml") {
        $args.Add("--manifest-path") | Out-Null
        $args.Add($WorkspaceManifest) | Out-Null
    }

    if ([string]::IsNullOrWhiteSpace($Package)) {
        $args.Add("--workspace") | Out-Null
    } else {
        $args.Add("-p") | Out-Null
        $args.Add($Package) | Out-Null
    }

    if (-not [string]::IsNullOrWhiteSpace($Bin)) {
        $args.Add("--bin") | Out-Null
        $args.Add($Bin) | Out-Null
    }

    if ($NoDefaultFeatures) {
        $args.Add("--no-default-features") | Out-Null
    }

    if (-not [string]::IsNullOrWhiteSpace($Features)) {
        $args.Add("--features") | Out-Null
        $args.Add($Features) | Out-Null
    }

    if (-not $NoLocked) {
        $args.Add("--locked") | Out-Null
    }

    if ($VerboseOutput) {
        $args.Add("--verbose") | Out-Null
    }

    Add-CargoProfileArguments -Arguments $args -CargoProfile $CargoProfile

    if ($Subcommand -eq "test") {
        if ($LibTests) {
            $args.Add("--lib") | Out-Null
        } elseif (-not [string]::IsNullOrWhiteSpace($TestTarget)) {
            $args.Add("--test") | Out-Null
            $args.Add($TestTarget) | Out-Null
        }

        if (-not [string]::IsNullOrWhiteSpace($TestFilter)) {
            $args.Add($TestFilter) | Out-Null
        }
    }

    $args.Add("--target-dir") | Out-Null
    $args.Add($ResolvedTargetDir) | Out-Null

    if ($Subcommand -eq "test" -and $IgnoredTests) {
        $args.Add("--") | Out-Null
        $args.Add("--ignored") | Out-Null
    }

    return $args.ToArray()
}

function Get-ConventionStructureArgs {
    param([string]$ResolvedTargetDir)

    return @(
        "test",
        "-p", "zircon_runtime",
        "--lib",
        "structure_convention",
        "--locked",
        "--jobs", "1",
        "--target-dir", $ResolvedTargetDir
    )
}

function Get-ConventionClippyArgs {
    param([string]$ResolvedTargetDir)

    return @(
        "clippy",
        "-p", "zircon_runtime_interface",
        "-p", "zircon_app",
        "--all-targets",
        "--no-deps",
        "--locked",
        "--jobs", "1",
        "--target-dir", $ResolvedTargetDir,
        "--",
        "-D", "warnings"
    )
}

function Assert-ArtifactOutputDirectory {
    param(
        [Parameter(Mandatory)]
        [string]$Path,
        [switch]$MvpProductInputArtifactOutput
    )

    $resolution = Resolve-ZirconWindowsPath -Path $Path
    $resolvedPath = $resolution.OperationalPath
    $displayPath = $resolution.DisplayPath
    $driveRoot = [System.IO.Path]::GetPathRoot($displayPath)
    if ($driveRoot -notmatch "^[A-Za-z]:\\$") {
        throw "-ArtifactOutputDirectory must resolve to a local drive: $displayPath"
    }
    if ($MvpProductInputArtifactOutput) {
        if ($displayPath -notmatch '^[D-F]:\\ZirconBuilds\\mvp-product-inputs-(?:[A-Za-z0-9][A-Za-z0-9._-]*)(?:\\|$)') {
            throw "-ArtifactOutputDirectory MVP product input artifact output must resolve under D:\ZirconBuilds\mvp-product-inputs-*: $displayPath"
        }
        return $resolvedPath
    }
    if ($driveRoot -in @("D:\", "E:\", "F:\")) {
        throw "-ArtifactOutputDirectory must be outside coordinator-governed D/E/F roots: $displayPath"
    }

    return $resolvedPath
}

function Get-ManagedFileSha256 {
    param(
        [Parameter(Mandatory)]
        [string]$Path
    )

    $stream = [System.IO.File]::OpenRead($Path)
    $algorithm = [System.Security.Cryptography.SHA256]::Create()
    try {
        return [System.BitConverter]::ToString($algorithm.ComputeHash($stream)).Replace("-", "")
    }
    finally {
        $algorithm.Dispose()
        $stream.Dispose()
    }
}

function Publish-BuildArtifacts {
    param(
        [Parameter(Mandatory)]
        [string]$TargetDirectory,
        [Parameter(Mandatory)]
        [string]$ArtifactOutputDirectory,
        [Parameter(Mandatory)]
        [string[]]$ArtifactName,
        [ValidateSet("development", "release", "profiling")]
        [string]$CargoProfile = "development",
        [switch]$MvpProductInputArtifactOutput
    )

    $resolvedTargetDirectory = (Resolve-ZirconWindowsPath -Path $TargetDirectory).OperationalPath
    $resolvedArtifactOutputDirectory = Assert-ArtifactOutputDirectory `
        -Path $ArtifactOutputDirectory `
        -MvpProductInputArtifactOutput:$MvpProductInputArtifactOutput
    $profileDirectoryName = switch ($CargoProfile) {
        "development" { "debug" }
        "release" { "release" }
        "profiling" { "profiling" }
    }
    $profileDirectory = Join-ZirconWindowsPath -Path $resolvedTargetDirectory -ChildPath $profileDirectoryName
    [System.IO.Directory]::CreateDirectory($resolvedArtifactOutputDirectory) | Out-Null

    foreach ($name in $ArtifactName) {
        if ([string]::IsNullOrWhiteSpace($name) -or
            $name -ne [System.IO.Path]::GetFileName($name)) {
            throw "Published artifact names must be plain file names: '$name'."
        }

        $sourcePath = Join-ZirconWindowsPath -Path $profileDirectory -ChildPath $name
        $destinationPath = Join-ZirconWindowsPath -Path $resolvedArtifactOutputDirectory -ChildPath $name
        if (-not [System.IO.File]::Exists($sourcePath)) {
            throw "Declared build artifact was not produced: $sourcePath"
        }
        if ([System.IO.File]::Exists($destinationPath)) {
            throw "Refusing to overwrite published build artifact: $destinationPath"
        }

        [System.IO.File]::Copy($sourcePath, $destinationPath, $false)
        $sourceHash = Get-ManagedFileSha256 -Path $sourcePath
        $destinationHash = Get-ManagedFileSha256 -Path $destinationPath
        if ($sourceHash -ne $destinationHash) {
            throw "Published build artifact hash does not match source: $name"
        }

        [pscustomobject]@{
            Name   = $name
            Path   = $destinationPath
            Sha256 = $destinationHash
            Bytes  = [System.IO.FileInfo]::new($destinationPath).Length
        }
    }
}

function Get-ExportPlatformContractArgs {
    param(
        [string]$ResolvedTargetDir,
        [ValidateSet("development", "release", "profiling")]
        [string]$CargoProfile = "development"
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

    Add-CargoProfileArguments -Arguments $args -CargoProfile $CargoProfile

    $args.Add("--target-dir") | Out-Null
    $args.Add($ResolvedTargetDir) | Out-Null

    return $args.ToArray()
}

function Get-ProfileFeatureContractArgs {
    param(
        [object]$Case,
        [string]$ResolvedTargetDir,
        [ValidateSet("development", "release", "profiling")]
        [string]$CargoProfile = "development"
    )

    $args = [System.Collections.Generic.List[string]]::new()
    $args.Add("check") | Out-Null
    $args.Add("-p") | Out-Null
    $args.Add([string]$Case.Package) | Out-Null
    $binaryProperty = $Case.PSObject.Properties["Bin"]
    if ($null -ne $binaryProperty -and -not [string]::IsNullOrWhiteSpace([string]$binaryProperty.Value)) {
        $args.Add("--bin") | Out-Null
        $args.Add([string]$binaryProperty.Value) | Out-Null
    }
    $args.Add("--no-default-features") | Out-Null
    $args.Add("--features") | Out-Null
    $args.Add([string]$Case.Features) | Out-Null

    if (-not $NoLocked) {
        $args.Add("--locked") | Out-Null
    }

    if ($VerboseOutput) {
        $args.Add("--verbose") | Out-Null
    }

    Add-CargoProfileArguments -Arguments $args -CargoProfile $CargoProfile

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
    if ($RunConventionStructure -and $RunConventionClippy) {
        throw "Run only one convention Cargo gate per managed validation."
    }
    if (($RunConventionStructure -or $RunConventionClippy) -and (-not $SkipBuild -or -not $SkipTest)) {
        throw "Convention Cargo gates require -SkipBuild and -SkipTest."
    }
    if (($RunConventionStructure -or $RunConventionClippy) -and
        (-not [string]::IsNullOrWhiteSpace($Package) -or
         -not [string]::IsNullOrWhiteSpace($Features) -or
         $NoDefaultFeatures -or
         $LibTests -or
         -not [string]::IsNullOrWhiteSpace($TestTarget) -or
         -not [string]::IsNullOrWhiteSpace($Bin) -or
         -not [string]::IsNullOrWhiteSpace($TestFilter) -or
         $IgnoredTests -or
         $RunExportPlatformContract -or
         $RunProfileFeatureContract)) {
        throw "Convention Cargo gates cannot be combined with package, feature, test, binary, or contract selectors."
    }
    if (($RunConventionStructure -or $RunConventionClippy) -and
        ($NoLocked -or
         $Ephemeral -or
         $CargoProfile -ne "development")) {
        throw "Convention Cargo gates require the locked development reuse profile and cannot be ephemeral."
    }
    if ($LibTests -and [string]::IsNullOrWhiteSpace($Package)) {
        throw "-LibTests requires -Package."
    }
    if ($LibTests -and $SkipTest) {
        throw "-LibTests cannot be combined with -SkipTest."
    }
    if (-not [string]::IsNullOrWhiteSpace($Bin) -and [string]::IsNullOrWhiteSpace($Package)) {
        throw "-Bin requires -Package."
    }
    if (-not [string]::IsNullOrWhiteSpace($Bin) -and $LibTests) {
        throw "-Bin cannot be combined with -LibTests."
    }
    $requestedPublishedArtifacts = @($PublishArtifact | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    if ($requestedPublishedArtifacts.Count -eq 0 -and -not [string]::IsNullOrWhiteSpace($ArtifactOutputDirectory)) {
        throw "-ArtifactOutputDirectory requires -PublishArtifact."
    }
    if ($requestedPublishedArtifacts.Count -gt 0 -and [string]::IsNullOrWhiteSpace($ArtifactOutputDirectory)) {
        throw "-PublishArtifact requires -ArtifactOutputDirectory."
    }
    if ($requestedPublishedArtifacts.Count -gt 0 -and $SkipBuild) {
        throw "-PublishArtifact requires Cargo build; remove -SkipBuild."
    }
    foreach ($artifactName in $requestedPublishedArtifacts) {
        if ($artifactName -ne [System.IO.Path]::GetFileName($artifactName)) {
            throw "-PublishArtifact values must be plain file names: '$artifactName'."
        }
    }
    if ($requestedPublishedArtifacts.Count -gt 0) {
        $ArtifactOutputDirectory = Assert-ArtifactOutputDirectory `
            -Path $ArtifactOutputDirectory `
            -MvpProductInputArtifactOutput:$MvpProductInputArtifactOutput
    }
    if (-not [string]::IsNullOrWhiteSpace($TestTarget) -and [string]::IsNullOrWhiteSpace($Package)) {
        throw "-TestTarget requires -Package."
    }
    if (-not [string]::IsNullOrWhiteSpace($TestTarget) -and $LibTests) {
        throw "-TestTarget cannot be combined with -LibTests."
    }
    if (-not [string]::IsNullOrWhiteSpace($TestTarget) -and $SkipTest) {
        throw "-TestTarget cannot be combined with -SkipTest."
    }
    if (-not [string]::IsNullOrWhiteSpace($Bin) -and -not [string]::IsNullOrWhiteSpace($TestTarget)) {
        throw "-Bin cannot be combined with -TestTarget."
    }
    if (-not $LibTests -and [string]::IsNullOrWhiteSpace($TestTarget) -and -not [string]::IsNullOrWhiteSpace($TestFilter)) {
        throw "-TestFilter requires -LibTests or -TestTarget."
    }
    if ($IgnoredTests -and [string]::IsNullOrWhiteSpace($TestFilter)) {
        throw "-IgnoredTests requires -TestFilter to avoid running every ignored test."
    }
    if ($IgnoredTests -and $SkipTest) {
        throw "-IgnoredTests cannot be combined with -SkipTest."
    }

    $resolvedRepoRoot = if ($RepoRoot) {
        (Resolve-ZirconWindowsPath -Path $RepoRoot).DisplayExistingPath
    } else {
        Find-RepoRoot $PSScriptRoot
    }
    $resolvedWorkspace = Resolve-WorkspaceManifest `
        -RepoRoot $resolvedRepoRoot `
        -RequestedManifestPath $ManifestPath
    if (($RunConventionStructure -or $RunConventionClippy) -and
        $resolvedWorkspace.RelativePath -ne "Cargo.toml") {
        throw "Convention Cargo gates require the repository root Cargo.toml."
    }
    if ($resolvedWorkspace.RelativePath -ne "Cargo.toml" -and
        ($RunExportPlatformContract -or $RunProfileFeatureContract)) {
        throw "-ManifestPath cannot be combined with export or profile feature contracts."
    }

    $laneKind = if ($RunConventionStructure) {
        "test"
    } elseif ($RunConventionClippy) {
        "check"
    } elseif (-not [string]::IsNullOrWhiteSpace($Package)) {
        if ($SkipTest) { "check" } else { "test" }
    } else {
        "workspace"
    }
    $compatibilityJson = New-CargoCompatibilityJson `
        -ResolvedRepoRoot $resolvedRepoRoot `
        -WorkspaceManifest $resolvedWorkspace.RelativePath `
        -CargoProfile $CargoProfile `
        -StorageMode $StorageMode `
        -DryRunMode:$DryRun
    $compilerCacheExecutable = Resolve-ManagedCompilerCacheExecutable `
        -StorageMode $StorageMode `
        -DryRunMode:$DryRun
    $resolvedTarget = Resolve-CoordinatorCargoTarget `
        -RepoRoot $resolvedRepoRoot `
        -ManualTargetDir $TargetDir `
        -LaneKind $laneKind `
        -WorkspaceManifest $resolvedWorkspace.RelativePath `
        -CargoProfile $CargoProfile `
        -StorageMode $StorageMode `
        -PrecomputedCompatibilityJson $compatibilityJson `
        -EphemeralLane:$Ephemeral `
        -DryRunMode:$DryRun

    $coordinatorJobFailed = $false
    $coordinatorJobStartAttempted = $false
    $primaryFailure = $null
    $locationPushed = $false
    $cargoEnvironmentLease = $null
    try {
    if (-not $resolvedTarget.DryRun) {
        $cargoEnvironmentLease = Push-ManagedCargoEnvironment `
            -TargetDirectory $resolvedTarget.TargetDir `
            -JobId $resolvedTarget.JobId `
            -StorageMode $StorageMode `
            -CompilerCacheExecutable $compilerCacheExecutable
        Write-Host ("Job scratch temp: {0}" -f $cargoEnvironmentLease.TemporaryDisplayPath)
        if ($StorageMode -eq "compact") {
            Write-Host ("Build scratch: {0}" -f $cargoEnvironmentLease.BuildDisplayPath)
        } else {
            Write-Host "Cargo build dir: target default (persistent)"
        }
        Write-Host ("Cargo home: {0}" -f $cargoEnvironmentLease.CargoHomeDisplayPath)
        Write-Host ("sccache cache: {0}" -f $cargoEnvironmentLease.SccacheDisplayPath)
        Write-Host ("sccache server temp: {0}" -f $cargoEnvironmentLease.SccacheTemporaryDisplayPath)
        Write-Host (
            "sccache endpoint: 127.0.0.1:{0} (PID {1})" -f `
                $cargoEnvironmentLease.SccacheServerPort,
                $cargoEnvironmentLease.SccacheServerProcessId
        )
    }
    Write-Host "Repo root: $resolvedRepoRoot"
    Write-Host "Workspace manifest: $($resolvedWorkspace.RelativePath)"
    Write-Host "Cargo working directory: $($resolvedWorkspace.Directory)"
    Write-Host ("Scope: {0}" -f $(if ([string]::IsNullOrWhiteSpace($Package)) { "workspace" } else { "package $Package" }))
    Write-Host ("Locked mode: {0}" -f $(if ($NoLocked) { "off" } else { "on" }))
    Write-Host "Cargo profile: $CargoProfile"
    Write-Host "Storage mode: $StorageMode"
    Write-Host ("Dry run: {0}" -f $(if ($DryRun) { "on" } else { "off" }))
    Write-Host ("Target dir: {0} ({1})" -f $resolvedTarget.TargetDir, $resolvedTarget.Reason)

    $storageAdmission = $null
    if ($SkipBuild -and $SkipTest -and -not $RunExportPlatformContract -and -not $RunProfileFeatureContract -and -not $RunConventionStructure -and -not $RunConventionClippy) {
        Write-Host "No stages selected. Use this mode to sanity-check script parsing or argument handling." -ForegroundColor Yellow
    } elseif ($DryRun) {
        Write-Host "Dry run selected; skipping cargo discovery and storage admission checks." -ForegroundColor Yellow
    } else {
        Get-Command cargo -ErrorAction Stop | Out-Null
        $storageAdmission = Get-PrebuildStorageAdmissionStatus -AbsoluteTargetDir $resolvedTarget.AbsoluteTargetDir
        Write-Host ("Free space on {0}: {1} (required reserve {2})" -f $storageAdmission.DriveRoot, (Format-ByteCount -Bytes $storageAdmission.FreeBytes), (Format-ByteCount -Bytes $storageAdmission.MinimumFreeBytes))
        if (-not $storageAdmission.IsAdmitted) {
            throw ("Cargo validation refused to preserve the disk reserve. Run .\tools\cleanup-stale-targets.ps1, review the plan, then apply it with -Apply. Free={0}; required reserve={1}." -f (Format-ByteCount -Bytes $storageAdmission.FreeBytes), (Format-ByteCount -Bytes $storageAdmission.MinimumFreeBytes))
        }
    }

    $coordinatorJobStartAttempted = -not $resolvedTarget.DryRun
    Start-CoordinatorCargoTarget -RepoRoot $resolvedRepoRoot -ResolvedTarget $resolvedTarget
    Push-Location $resolvedWorkspace.Directory
    $locationPushed = $true
        if (-not $SkipBuild) {
            Invoke-Step "Cargo build" {
                Invoke-Cargo -Arguments (Get-CargoArgs `
                    -Subcommand "build" `
                    -ResolvedTargetDir $resolvedTarget.TargetDir `
                    -WorkspaceManifest $resolvedWorkspace.InvocationManifestPath `
                    -CargoProfile $CargoProfile)
            }

            if (($Results | Select-Object -Last 1).ExitCode -eq 0 -and $requestedPublishedArtifacts.Count -gt 0) {
                Invoke-Step "Publish build artifacts" {
                    Publish-BuildArtifacts `
                        -TargetDirectory $resolvedTarget.TargetDir `
                        -ArtifactOutputDirectory $ArtifactOutputDirectory `
                        -ArtifactName $requestedPublishedArtifacts `
                        -CargoProfile $CargoProfile `
                        -MvpProductInputArtifactOutput:$MvpProductInputArtifactOutput | ForEach-Object {
                        Write-Host ("Published {0} ({1}, SHA256 {2})" -f $_.Name, (Format-ByteCount -Bytes $_.Bytes), $_.Sha256)
                    }
                }
            }
        }

        if (-not $SkipTest) {
            Invoke-Step "Cargo test" {
                Invoke-Cargo -Arguments (Get-CargoArgs `
                    -Subcommand "test" `
                    -ResolvedTargetDir $resolvedTarget.TargetDir `
                    -WorkspaceManifest $resolvedWorkspace.InvocationManifestPath `
                    -CargoProfile $CargoProfile)
            }
        }

        if ($RunConventionStructure) {
            Invoke-Step "Convention structure" {
                Invoke-Cargo -Arguments (Get-ConventionStructureArgs `
                    -ResolvedTargetDir $resolvedTarget.TargetDir)
            }
        }

        if ($RunConventionClippy) {
            Invoke-Step "Convention clippy" {
                Invoke-Cargo -Arguments (Get-ConventionClippyArgs `
                    -ResolvedTargetDir $resolvedTarget.TargetDir)
            }
        }

        if ($RunExportPlatformContract) {
            foreach ($platform in (Get-SelectedExportContractPlatforms -Platform $ExportContractPlatform)) {
                Invoke-Step "Export platform contract ($platform)" {
                    Invoke-CargoWithEnvironment `
                        -Arguments (Get-ExportPlatformContractArgs `
                            -ResolvedTargetDir $resolvedTarget.TargetDir `
                            -CargoProfile $CargoProfile) `
                        -Environment @{ ZR_EXPORT_CONTRACT_PLATFORM = $platform }
                }
            }
        }

        if ($RunProfileFeatureContract) {
            foreach ($case in (Get-SelectedProfileFeatureContractCases -Label $ProfileFeatureContractLabel)) {
                Invoke-Step "Profile feature contract ($($case.Label))" {
                    Invoke-Cargo -Arguments (Get-ProfileFeatureContractArgs `
                        -Case $case `
                        -ResolvedTargetDir $resolvedTarget.TargetDir `
                        -CargoProfile $CargoProfile)
                }
            }
        }
    } catch {
        $coordinatorJobFailed = $true
        $primaryFailure = $_
        throw
    } finally {
        if ($locationPushed) {
            Pop-Location
        }
        $hasFailedStep = $null -ne ($Results | Where-Object { $_.ExitCode -ne 0 } | Select-Object -First 1)
        $jobExitCode = if ($coordinatorJobFailed -or $hasFailedStep) { 1 } else { 0 }
        $cleanupFailure = $null
        try {
            Complete-CoordinatorCargoTarget `
                -RepoRoot $resolvedRepoRoot `
                -ResolvedTarget $resolvedTarget `
                -ExitCode $jobExitCode `
                -StartAttempted:$coordinatorJobStartAttempted
        } catch {
            $cleanupFailure = $_
        }
        try {
            if ($null -ne $cargoEnvironmentLease) {
                Pop-ManagedCargoEnvironment -Lease $cargoEnvironmentLease
            }
        } catch {
            if ($null -eq $cleanupFailure) {
                $cleanupFailure = $_
            } else {
                Write-Warning `
                    ("Additional managed Cargo environment cleanup failure: {0}" -f $_.Exception.Message) `
                    -WarningAction Continue
            }
        }
        Resolve-ValidationCleanupFailure `
            -CleanupFailure $cleanupFailure `
            -PrimaryFailure $primaryFailure `
            -HasFailedStep:$hasFailedStep
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
