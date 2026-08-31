param(
    [Parameter(Mandatory = $true)]
    [string]$ViewerExe,
    [Parameter(Mandatory = $true)]
    [string]$HdriPath,
    [Parameter(Mandatory = $true)]
    [string]$BuildProvenance,
    [Parameter(Mandatory = $true)]
    [string]$CaptureToolchainManifest,
    [ValidateSet("cold", "warm")]
    [string[]]$CacheModes = @("cold", "warm"),
    [ValidateRange(1, 20)]
    [int]$Repetitions = 5,
    [AllowNull()]
    [Nullable[int]]$FaceSize,
    [AllowNull()]
    [Nullable[int]]$PmremFaceSize,
    [ValidateSet("metal-mirror", "dielectric-ior")]
    [string]$MaterialFixture = "metal-mirror",
    [ValidateRange(1, 900)]
    [int]$ViewerTimeoutSeconds = 180,
    [ValidateRange(1, 600)]
    [int]$WprTimeoutSeconds = 60,
    [ValidateRange(1, 60)]
    [int]$EnergySampleIntervalSeconds = 1,
    [string]$EvidenceRoot = "",
    [string]$DisplayVisualOracle = "",
    [string]$PythonExecutable = "python",
    [switch]$SkipWpr,
    [switch]$SkipEnergyMeter,
    [switch]$CaptureRenderDoc
)

$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
. (Join-Path $PSScriptRoot "profile-capture-manifest.ps1")
. (Join-Path $PSScriptRoot "shader-pbr-profile-runtime-evidence.ps1")
. (Join-Path $PSScriptRoot "shader-pbr-profile-contract.ps1")
. (Join-Path $PSScriptRoot "shader-pbr-profile-publication.ps1")
. (Join-Path $PSScriptRoot "shader-pbr-profile-toolchain.ps1")
. (Join-Path $PSScriptRoot "shader-pbr-profile-evidence-identity.ps1")
. (Join-Path $PSScriptRoot "performance-machine-manifest.ps1")

function Assert-ZirconShaderPbrOptionalFaceSize {
    param(
        [AllowNull()]
        [Nullable[int]]$Value,
        [Parameter(Mandatory = $true)]
        [string]$Name
    )

    if ($null -eq $Value) {
        return
    }
    if ([int]$Value -notin @(64, 128, 256, 512, 1024)) {
        throw "$Name must be 64, 128, 256, 512, or 1024 when explicitly set."
    }
}

function Test-ZirconShaderPbrPathWithin {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Root,
        [Parameter(Mandatory = $true)]
        [string]$Candidate
    )

    $normalizedRoot = [System.IO.Path]::GetFullPath($Root).TrimEnd("\\")
    $normalizedCandidate = [System.IO.Path]::GetFullPath($Candidate)
    $rootPrefix = $normalizedRoot + [System.IO.Path]::DirectorySeparatorChar
    return $normalizedCandidate.Equals($normalizedRoot, [System.StringComparison]::OrdinalIgnoreCase) -or
        $normalizedCandidate.StartsWith($rootPrefix, [System.StringComparison]::OrdinalIgnoreCase)
}

function Get-ZirconShaderPbrRelativePath {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Root,
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    $normalizedRoot = [System.IO.Path]::GetFullPath($Root).TrimEnd("\\")
    $normalizedPath = [System.IO.Path]::GetFullPath($Path)
    if ($normalizedPath.Equals($normalizedRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
        return "."
    }
    $rootPrefix = $normalizedRoot + [System.IO.Path]::DirectorySeparatorChar
    if (-not $normalizedPath.StartsWith($rootPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Shader PBR profile path escapes its root: $normalizedPath"
    }
    return $normalizedPath.Substring($rootPrefix.Length)
}

function Resolve-ZirconShaderPbrProfileEvidenceRoot {
    param(
        [Parameter(Mandatory = $true)]
        [string]$RepoRoot,
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    if ([string]::IsNullOrWhiteSpace($Path)) {
        $Path = Join-Path $RepoRoot "docs\tests\runtime\shader"
    }
    $candidate = [System.IO.Path]::GetFullPath($Path)
    $evidenceRoot = [System.IO.Path]::GetFullPath((Join-Path $RepoRoot "docs\tests\runtime\shader"))
    if (-not (Test-ZirconShaderPbrPathWithin -Root $evidenceRoot -Candidate $candidate)) {
        throw "Shader PBR profile evidence root must resolve beneath $evidenceRoot."
    }
    if ($candidate.StartsWith("C:\", [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Shader PBR profile evidence must not be written beneath C:."
    }
    # GetFullPath does not resolve Windows junctions. Reject every existing
    # component before a profile writer can traverse out of the E: evidence tree.
    $componentPath = $evidenceRoot
    $relativePath = Get-ZirconShaderPbrRelativePath -Root $evidenceRoot -Path $candidate
    $pathComponents = if ($relativePath -eq ".") { @() } else {
        @($relativePath.Split(@([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar), [System.StringSplitOptions]::RemoveEmptyEntries))
    }
    foreach ($component in @(".") + $pathComponents) {
        if ($component -ne ".") {
            $componentPath = Join-Path $componentPath $component
        }
        if (-not (Test-Path -LiteralPath $componentPath)) {
            break
        }
        $attributes = (Get-Item -LiteralPath $componentPath -Force).Attributes
        if (($attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "Shader PBR profile evidence root contains a reparse point: $componentPath"
        }
    }
    return $candidate
}

function Get-ZirconShaderPbrProfileFileFingerprint {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path,
        [Parameter(Mandatory = $true)]
        [string]$Description
    )

    return Get-ZirconProfileRequiredFileFingerprint -Path $Path -Description $Description
}

function Assert-ZirconShaderPbrBuildProvenance {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)]$ViewerFingerprint,
        [Parameter(Mandatory = $true)]$SourceFiles
    )

    $provenanceFingerprint = Get-ZirconShaderPbrProfileFileFingerprint `
        -Path $Path `
        -Description "Shader PBR viewer capture provenance"
    try {
        $provenance = Get-Content -LiteralPath $provenanceFingerprint.path -Raw | ConvertFrom-Json
    }
    catch {
        throw "Shader PBR profile build provenance is malformed: $($provenanceFingerprint.path)"
    }
    if ($provenance.schema_version -ne 2 -or $provenance.provenance_kind -ne "zircon_managed_viewer_artifact_provenance") {
        throw "Shader PBR profile build provenance has an unexpected schema: $($provenanceFingerprint.path)"
    }
    $recordedBinary = $provenance.binary
    if ($null -eq $recordedBinary -or
        [System.IO.Path]::GetFullPath([string]$recordedBinary.path) -ne [System.IO.Path]::GetFullPath([string]$ViewerFingerprint.path) -or
        [string]$recordedBinary.sha256 -ne [string]$ViewerFingerprint.sha256 -or
        [int64]$recordedBinary.byte_length -ne [int64]$ViewerFingerprint.byte_length) {
        throw "Shader PBR profile build provenance does not bind the requested viewer binary: $($provenanceFingerprint.path)"
    }
    $sourceManifest = $provenance.repository.source_manifest
    if ($null -eq $sourceManifest) {
        throw "Shader PBR profile build provenance is missing its source manifest: $($provenanceFingerprint.path)"
    }
    $recordedSourceProperties = @($sourceManifest.PSObject.Properties)
    if ($recordedSourceProperties.Count -ne @($SourceFiles).Count) {
        throw "Shader PBR profile build provenance source manifest does not match the required critical source set: $($provenanceFingerprint.path)"
    }
    foreach ($source in @($SourceFiles)) {
        $recordedSource = $sourceManifest.PSObject.Properties[$source.relative_path]
        if ($null -eq $recordedSource -or [string]$recordedSource.Value -ne [string]$source.sha256) {
            throw "Shader PBR profile build provenance source manifest does not match $($source.relative_path): $($provenanceFingerprint.path)"
        }
    }
    $sourceValidationTicket = $provenance.source_validation_ticket
    if ($null -eq $sourceValidationTicket -or
        [string]$sourceValidationTicket.validation_ticket_id -notmatch '^[0-9a-f]{32}$' -or
        [string]$sourceValidationTicket.status -ne "passed") {
        throw "Shader PBR profile build provenance is missing a passed source validation ticket: $($provenanceFingerprint.path)"
    }
    $validationTicket = Get-ZirconShaderPbrCoordinatorValidationTicket `
        -RepoRoot $RepoRoot `
        -ValidationTicketId ([string]$sourceValidationTicket.validation_ticket_id)
    $validatedTicket = Assert-ZirconShaderPbrCoordinatorValidationTicket `
        -Ticket $validationTicket `
        -SourceFiles $SourceFiles `
        -Description "Shader PBR profile build provenance coordinator validation ticket"
    if ([string]$sourceValidationTicket.validation_ticket_id -ne [string]$validatedTicket.validation_ticket_id) {
        throw "Shader PBR profile build provenance does not match its coordinator ticket id: $($provenanceFingerprint.path)"
    }
    if ([string]$sourceValidationTicket.source_manifest_hash -ne [string]$validatedTicket.source_manifest_hash) {
        throw "Shader PBR profile build provenance source validation ticket does not match its coordinator source manifest: $($provenanceFingerprint.path)"
    }
    $recordedArtifactReceipt = $provenance.artifact_receipt
    if ($null -eq $recordedArtifactReceipt -or
        [string]$recordedArtifactReceipt.artifact_receipt_id -notmatch '^[0-9a-f]{32}$') {
        throw "Shader PBR profile build provenance is missing its managed artifact receipt: $($provenanceFingerprint.path)"
    }
    $artifactReceipt = Get-ZirconShaderPbrCoordinatorArtifactReceipt `
        -RepoRoot $RepoRoot `
        -ArtifactReceiptId ([string]$recordedArtifactReceipt.artifact_receipt_id)
    $validatedArtifactReceipt = Assert-ZirconShaderPbrCoordinatorArtifactReceipt `
        -Receipt $artifactReceipt `
        -ViewerFingerprint $ViewerFingerprint `
        -ValidationTicketId $validatedTicket.validation_ticket_id `
        -SourceManifestHash $validatedTicket.source_manifest_hash `
        -Description "Shader PBR profile managed artifact receipt"
    foreach ($field in @(
        "artifact_receipt_id",
        "job_id",
        "run_id",
        "validation_ticket_id",
        "input_manifest_hash",
        "source_manifest_hash",
        "target_relative_path",
        "artifact_path",
        "sha256",
        "byte_length",
        "command_sha256"
    )) {
        if ([string]$recordedArtifactReceipt.$field -ne [string]$validatedArtifactReceipt.$field) {
            throw "Shader PBR profile build provenance artifact receipt field '$field' does not match the coordinator: $($provenanceFingerprint.path)"
        }
    }
    return $provenanceFingerprint
}

function Export-ZirconShaderPbrProfileManifest {
    param(
        [Parameter(Mandatory = $true)]
        [string]$ProfileRoot,
        [Parameter(Mandatory = $true)]
        [string]$ViewerExe,
        [Parameter(Mandatory = $true)]
        [string]$HdriPath,
        [Parameter(Mandatory = $true)]
        [string]$BuildProvenance,
        [Parameter(Mandatory = $true)]
        [string]$EvidenceRoot,
        [Parameter(Mandatory = $true)]
        [int]$Repetitions,
        [AllowNull()]
        [Nullable[int]]$FaceSize,
        [AllowNull()]
        [Nullable[int]]$PmremFaceSize,
        [ValidateSet("metal-mirror", "dielectric-ior")]
        [string]$MaterialFixture = "metal-mirror",
        [AllowNull()]
        [object]$DisplayVisualOracleFingerprint = $null,
        [Parameter(Mandatory = $true)]
        [string[]]$CacheModes,
        [Parameter(Mandatory = $true)]$CaptureToolchain,
        [Parameter(Mandatory = $true)]$MachineManifest
    )

    $gitMetadata = Get-ZirconProfileGitMetadata -RepoRoot $RepoRoot
    $sourceFiles = Get-ZirconShaderPbrProfileCriticalSourcePaths -RepoRoot $RepoRoot | ForEach-Object {
        $relativePath = $_
        $fingerprint = Get-ZirconShaderPbrProfileFileFingerprint `
            -Path (Join-Path $RepoRoot $relativePath) `
            -Description "critical Shader06 source file '$relativePath'"
        [pscustomobject]@{
            relative_path = $relativePath
            sha256 = $fingerprint.sha256
            byte_length = $fingerprint.byte_length
        }
    }
    $profileToolFiles = Get-ZirconShaderPbrProfileToolPaths | ForEach-Object {
        $relativePath = $_
        $fingerprint = Get-ZirconShaderPbrProfileFileFingerprint `
            -Path (Join-Path $RepoRoot $relativePath) `
            -Description "Shader PBR profile tool '$relativePath'"
        [pscustomobject]@{
            relative_path = $relativePath
            sha256 = $fingerprint.sha256
            byte_length = $fingerprint.byte_length
        }
    }
    $viewerFingerprint = Get-ZirconShaderPbrProfileFileFingerprint `
        -Path $ViewerExe `
        -Description "shader PBR viewer binary fingerprint"
    $hdriFingerprint = Get-ZirconShaderPbrProfileFileFingerprint `
        -Path $HdriPath `
        -Description "shader PBR HDR input fingerprint"
    $buildProvenanceFingerprint = Assert-ZirconShaderPbrBuildProvenance `
        -Path $BuildProvenance `
        -ViewerFingerprint $viewerFingerprint `
        -SourceFiles $sourceFiles

    $manifest = [pscustomobject]@{
        schema_version = 1
        profile_kind = "zircon_shader_pbr_viewer_startup"
        capture_started_utc = (Get-Date).ToUniversalTime().ToString("o")
        repository = [pscustomobject]@{
            root = $RepoRoot
            git = $gitMetadata
            critical_source_files = @($sourceFiles)
            profile_tool_files = @($profileToolFiles)
        }
        binary = $viewerFingerprint
        build_provenance = $buildProvenanceFingerprint
        input = [pscustomobject]@{
            hdri = $hdriFingerprint
            requested_source_face_size = $FaceSize
            requested_pmrem_face_size = $PmremFaceSize
            material_fixture = $MaterialFixture
        }
        capture = [pscustomobject]@{
            evidence_root = $EvidenceRoot
            display_visual_oracle = $DisplayVisualOracleFingerprint
            repetitions_per_mode = $Repetitions
            cache_modes = @($CacheModes)
            cold_semantics = "new process and new caller-owned IBL cache directory per measured run; driver caches are not cleared"
            warm_semantics = "one unmeasured cache seed, then new processes reusing its caller-owned IBL cache directory"
            cache_layers = [ordered]@{
                engine_cache = [ordered]@{
                    control_state = "controlled"
                    cold = "new process and new caller-owned IBL cache directory per measured run"
                    warm = "new process reusing the caller-owned IBL cache seeded before measurement"
                }
                shader_cache = [ordered]@{
                    control_state = "uncontrolled"
                    cold = "not cleared or measured by this capture"
                    warm = "not cleared or measured by this capture"
                }
                os_file_cache = [ordered]@{
                    control_state = "uncontrolled"
                    cold = "not cleared or measured by this capture"
                    warm = "not cleared or measured by this capture"
                }
                driver_cache = [ordered]@{
                    control_state = "uncontrolled"
                    cold = "not cleared or measured by this capture"
                    warm = "not cleared or measured by this capture"
                }
            }
            strict_cold_eligible = $false
            comparison_scope = "process_and_caller_owned_engine_cache"
            machine_manifest = $MachineManifest
            toolchain = [pscustomobject]@{
                manifest = $CaptureToolchain.manifest
                graphics = $CaptureToolchain.graphics
                renderdoc = $CaptureToolchain.renderdoc
            }
            wpr_cpu_sampling = -not $SkipWpr
            energy_meter = "sampled Energy Meter Power in watts plus raw Energy/Time counters only when named host meters are available"
        }
    }
    $manifestPath = Join-Path $ProfileRoot "profile_manifest.json"
    $manifest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $manifestPath -Encoding UTF8
    return $manifestPath
}

function New-ZirconShaderPbrProfileRunDirectory {
    param(
        [Parameter(Mandatory = $true)]
        [string]$ModeRoot,
        [Parameter(Mandatory = $true)]
        [string]$Role,
        [Parameter(Mandatory = $true)]
        [int]$Ordinal
    )

    $runDirectory = Join-Path $ModeRoot ("{0}-{1:D2}" -f $Role, $Ordinal)
    New-Item -ItemType Directory -Force -Path $runDirectory | Out-Null
    return $runDirectory
}

function ConvertTo-ZirconShaderPbrCommandLine {
    param([string[]]$Arguments)

    return ($Arguments | ForEach-Object {
            '"' + $_.Replace('"', '\"') + '"'
        }) -join ' '
}

function Invoke-ZirconShaderPbrWprCommand {
    param(
        [Parameter(Mandatory = $true)][string]$WprPath,
        [Parameter(Mandatory = $true)][string[]]$Arguments,
        [Parameter(Mandatory = $true)][int]$TimeoutSeconds,
        [Parameter(Mandatory = $true)][string]$Operation
    )

    $process = Start-Process -FilePath $WprPath -ArgumentList (ConvertTo-ZirconShaderPbrCommandLine -Arguments $Arguments) -PassThru -WindowStyle Hidden
    if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
        Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
        $process.WaitForExit()
        throw "Windows Performance Recorder $Operation timed out after $TimeoutSeconds seconds."
    }
    if ($process.ExitCode -ne 0) {
        throw "Windows Performance Recorder $Operation failed; exit=$($process.ExitCode)"
    }
}

function Invoke-ZirconShaderPbrWprCancel {
    param(
        [Parameter(Mandatory = $true)][string]$WprPath,
        [Parameter(Mandatory = $true)][int]$TimeoutSeconds
    )

    try {
        Invoke-ZirconShaderPbrWprCommand -WprPath $WprPath -Arguments @("-cancel") -TimeoutSeconds $TimeoutSeconds -Operation "cancel"
    }
    catch {
        Write-Warning "Windows Performance Recorder cancellation failed after an interrupted profile: $($_.Exception.Message)"
    }
}

function Start-ZirconShaderPbrWprCapture {
    param(
        [Parameter(Mandatory = $true)][string]$RunDirectory,
        [Parameter(Mandatory = $true)][int]$TimeoutSeconds
    )

    $wprPath = Join-Path $env:WINDIR "System32\wpr.exe"
    if (-not (Test-Path -LiteralPath $wprPath -PathType Leaf)) {
        throw "Shader PBR profile requires Windows Performance Recorder: $wprPath"
    }
    try {
        Invoke-ZirconShaderPbrWprCommand -WprPath $wprPath -Arguments @("-start", "CPU", "-filemode") -TimeoutSeconds $TimeoutSeconds -Operation "start"
    }
    catch {
        Invoke-ZirconShaderPbrWprCancel -WprPath $wprPath -TimeoutSeconds $TimeoutSeconds
        throw
    }
    return [pscustomobject]@{
        executable = $wprPath
        etl_path = Join-Path $RunDirectory "cpu_sampling.etl"
        timeout_seconds = $TimeoutSeconds
    }
}

function Stop-ZirconShaderPbrWprCapture {
    param([Parameter(Mandatory = $true)]$Capture)

    try {
        Invoke-ZirconShaderPbrWprCommand -WprPath $Capture.executable -Arguments @("-stop", $Capture.etl_path) -TimeoutSeconds $Capture.timeout_seconds -Operation "stop"
    }
    catch {
        Invoke-ZirconShaderPbrWprCancel -WprPath $Capture.executable -TimeoutSeconds $Capture.timeout_seconds
        throw
    }
    if (-not (Test-Path -LiteralPath $Capture.etl_path -PathType Leaf)) {
        throw "Windows Performance Recorder did not write its expected ETL: $($Capture.etl_path)"
    }
    return Get-ZirconProfileFileFingerprint -Path $Capture.etl_path
}

function Start-ZirconShaderPbrEnergyMeterCapture {
    param(
        [Parameter(Mandatory = $true)]
        [string]$RunDirectory,
        [Parameter(Mandatory = $true)]
        [int]$SampleIntervalSeconds
    )

    $outputPath = Join-Path $RunDirectory "energy_meter.csv"
    $counterSet = Get-Counter -ListSet "Energy Meter" -ErrorAction SilentlyContinue
    $paths = @($counterSet.PathsWithInstances | Where-Object { $_ -match "\\(Power|Energy|Time)$" })
    $typeperf = Get-Command typeperf.exe -ErrorAction SilentlyContinue
    if ($paths.Count -eq 0 -or $null -eq $typeperf) {
        return [pscustomobject]@{
            status = "unavailable"
            output_path = $outputPath
            counter_paths = @($paths)
            counter_units = @()
            sample_interval_seconds = $SampleIntervalSeconds
            process = $null
        }
    }
    $arguments = @($paths | ForEach-Object { '"' + $_.Replace('"', '\"') + '"' })
    $arguments += @("-si", "$SampleIntervalSeconds", "-o", ('"' + $outputPath + '"'), "-f", "CSV")
    $process = Start-Process -FilePath $typeperf.Source -ArgumentList ($arguments -join ' ') -PassThru -WindowStyle Hidden
    return [pscustomobject]@{
        status = "started"
        output_path = $outputPath
        counter_paths = @($paths)
        counter_units = @(
            [pscustomobject]@{
                counter_suffix = "Power"
                unit = "watts"
                interpretation = "platform-defined instantaneous or averaged power"
            },
            [pscustomobject]@{
                counter_suffix = "Energy"
                unit = "raw_energy_meter_counter"
                interpretation = "retained without cross-run unit conversion"
            },
            [pscustomobject]@{
                counter_suffix = "Time"
                unit = "raw_energy_meter_counter"
                interpretation = "retained without cross-run unit conversion"
            }
        )
        sample_interval_seconds = $SampleIntervalSeconds
        process = $process
    }
}

function Get-ZirconShaderPbrEnergyMeterCaptureStatus {
    param(
        [Parameter(Mandatory = $true)][bool]$TerminatedByProfiler,
        [Parameter(Mandatory = $true)][int]$TypeperfExitCode,
        [Parameter(Mandatory = $true)][bool]$HasRequiredRows
    )

    if (-not $TerminatedByProfiler -and $TypeperfExitCode -ne 0) {
        return "failed"
    }
    if ($HasRequiredRows) {
        return "captured"
    }
    return "insufficient_samples"
}

function Stop-ZirconShaderPbrEnergyMeterCapture {
    param([Parameter(Mandatory = $true)]$Capture)

    if ($Capture.status -ne "started") {
        return [pscustomobject]@{
            status = $Capture.status
            output_path = $Capture.output_path
            counter_paths = @($Capture.counter_paths)
            counter_units = @($Capture.counter_units)
            sample_interval_seconds = $Capture.sample_interval_seconds
            csv_fingerprint = $null
        }
    }
    $terminatedByProfiler = $false
    if (-not $Capture.process.HasExited) {
        $terminatedByProfiler = $true
        Stop-Process -Id $Capture.process.Id -ErrorAction Stop
        $Capture.process.WaitForExit()
    }
    $typeperfExitCode = $Capture.process.ExitCode
    $hasRequiredRows = (Test-Path -LiteralPath $Capture.output_path -PathType Leaf) -and
        ((Get-Content -LiteralPath $Capture.output_path | Measure-Object -Line).Lines -ge 3)
    $status = Get-ZirconShaderPbrEnergyMeterCaptureStatus `
        -TerminatedByProfiler $terminatedByProfiler `
        -TypeperfExitCode $typeperfExitCode `
        -HasRequiredRows $hasRequiredRows
    return [pscustomobject]@{
        status = $status
        output_path = $Capture.output_path
        counter_paths = @($Capture.counter_paths)
        counter_units = @($Capture.counter_units)
        sample_interval_seconds = $Capture.sample_interval_seconds
        typeperf_exit_code = $typeperfExitCode
        terminated_by_profiler = $terminatedByProfiler
        csv_fingerprint = if ($status -eq "captured") { Get-ZirconProfileFileFingerprint -Path $Capture.output_path } else { $null }
    }
}

function Read-ZirconShaderPbrKeyValueEvidence {
    param([Parameter(Mandatory = $true)][string]$Path)

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Shader PBR profile is missing expected evidence: $Path"
    }
    $fields = @{}
    foreach ($line in Get-Content -LiteralPath $Path) {
        if ([string]::IsNullOrWhiteSpace($line)) {
            continue
        }
        $separator = $line.IndexOf('=')
        if ($separator -le 0) {
            throw "Shader PBR profile evidence contains an invalid field: $Path"
        }
        $key = $line.Substring(0, $separator)
        $value = $line.Substring($separator + 1)
        if ($fields.ContainsKey($key) -or [string]::IsNullOrWhiteSpace($value)) {
            throw "Shader PBR profile evidence contains an invalid or repeated field: $Path"
        }
        $fields[$key] = $value
    }
    return $fields
}

function Invoke-ZirconShaderPbrEvidenceValidator {
    param(
        [Parameter(Mandatory = $true)][string]$Script,
        [Parameter(Mandatory = $true)][string[]]$Arguments,
        [Parameter(Mandatory = $true)][string]$OutputPath
    )

    $output = & $PythonExecutable $Script @Arguments 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "Shader PBR profile evidence validator failed: $Script output=$($output -join [Environment]::NewLine)"
    }
    $output | Set-Content -LiteralPath $OutputPath -Encoding UTF8
}

function Invoke-ZirconShaderPbrProfileRun {
    param(
        [Parameter(Mandatory = $true)][string]$RunDirectory,
        [Parameter(Mandatory = $true)][string]$ProfileId,
        [Parameter(Mandatory = $true)][ValidateSet("cold", "warm")][string]$CacheMode,
        [Parameter(Mandatory = $true)][ValidateSet("cache_seed", "measured", "renderdoc")][string]$Role,
        [Parameter(Mandatory = $true)][int]$Ordinal,
        [Parameter(Mandatory = $true)][string]$CacheDirectory,
        [Parameter(Mandatory = $true)][ValidateSet("Written", "Reused")][string]$ExpectedStagingStatus,
        [Parameter(Mandatory = $true)]$CaptureToolchain,
        [Parameter(Mandatory = $true)][string]$BuildProvenance,
        [ValidateSet("metal-mirror", "dielectric-ior")]
        [string]$MaterialFixture = "metal-mirror",
        [AllowEmptyString()][string]$DisplayVisualOracle = "",
        [switch]$Measure,
        [switch]$CaptureRenderDoc
    )

    $screenshotPath = Join-Path $RunDirectory "ready.png"
    $sidecarPath = "$screenshotPath.txt"
    $gpuTimingPath = Join-Path $RunDirectory "gpu_timing.txt"
    $workDirectory = Join-Path $RunDirectory "work"
    $stdoutPath = Join-Path $RunDirectory "viewer.stdout.log"
    $stderrPath = Join-Path $RunDirectory "viewer.stderr.log"
    $runtimeProfileRoot = Join-Path $RunDirectory "runtime-profile"
    $runtimeProfileSession = "$ProfileId-$CacheMode-$Role-$Ordinal"
    New-Item -ItemType Directory -Force -Path $CacheDirectory | Out-Null
    $evidenceIdentity = New-ZirconShaderPbrReadyFrameEvidenceIdentity `
        -RunDirectory $RunDirectory `
        -ProfileId $ProfileId `
        -CacheMode $CacheMode `
        -Role $Role `
        -Ordinal $Ordinal `
        -ViewerExe $ViewerExe `
        -HdriPath $HdriPath `
        -BuildProvenance $BuildProvenance

    $arguments = @(
        "--hdri", $HdriPath,
        "--material-fixture", $MaterialFixture
    )
    if ($null -ne $FaceSize) {
        $arguments += @("--face-size", "$FaceSize")
    }
    if ($null -ne $PmremFaceSize) {
        $arguments += @("--pmrem-face-size", "$PmremFaceSize")
    }
    $arguments += @(
        "--work-dir", $workDirectory,
        "--ibl-cache-dir", $CacheDirectory,
        "--host-mode", "offscreen-diagnostic",
        "--screenshot", $screenshotPath,
        "--evidence-identity", $evidenceIdentity.path
    )
    if ($Measure) {
        $arguments += @("--gpu-timing-report", $gpuTimingPath)
    }
    if ($CaptureRenderDoc) {
        if ($null -eq $CaptureToolchain.renderdoc) {
            throw "Shader PBR RenderDoc capture requires a pinned RenderDoc DLL in the capture toolchain manifest."
        }
        $renderDocDll = [string]$CaptureToolchain.renderdoc.dll.path
        $arguments += @(
            "--renderdoc-capture-once",
            "--renderdoc-dll", $renderDocDll,
            "--renderdoc-capture-path", (Join-Path $RunDirectory "renderdoc"),
            "--exit-after-capture"
        )
    }

    $wprCapture = $null
    $energyCapture = $null
    $wprFingerprint = $null
    $energyReport = $null
    $process = $null
    $primaryFailure = $null
    $cleanupFailure = $null
    $previousBackend = $env:WGPU_BACKEND
    $previousProfileCapture = $env:ZIRCON_PROFILE_CAPTURE
    $previousProfileSession = $env:ZIRCON_PROFILE_SESSION
    $previousProfileOutputRoot = $env:ZIRCON_PROFILE_OUTPUT_ROOT
    $previousProfileMaxFrames = $env:ZIRCON_PROFILE_MAX_FRAMES
    $previousProfileMaxSpans = $env:ZIRCON_PROFILE_MAX_SPANS
    $previousProfileMaxCounters = $env:ZIRCON_PROFILE_MAX_COUNTERS
    $previousProfileIncludePerfetto = $env:ZIRCON_PROFILE_INCLUDE_PERFETTO
    $startedAt = (Get-Date).ToUniversalTime()
    try {
        if ($Measure -and -not $SkipWpr) {
            $wprCapture = Start-ZirconShaderPbrWprCapture -RunDirectory $RunDirectory -TimeoutSeconds $WprTimeoutSeconds
        }
        if ($Measure -and -not $SkipEnergyMeter) {
            $energyCapture = Start-ZirconShaderPbrEnergyMeterCapture `
                -RunDirectory $RunDirectory `
                -SampleIntervalSeconds $EnergySampleIntervalSeconds
        }
        $env:WGPU_BACKEND = [string]$CaptureToolchain.graphics.wgpu_backend
        if ($Measure) {
            New-Item -ItemType Directory -Force -Path $runtimeProfileRoot | Out-Null
            $env:ZIRCON_PROFILE_CAPTURE = "1"
            $env:ZIRCON_PROFILE_SESSION = $runtimeProfileSession
            $env:ZIRCON_PROFILE_OUTPUT_ROOT = $runtimeProfileRoot
            $env:ZIRCON_PROFILE_MAX_FRAMES = "4096"
            $env:ZIRCON_PROFILE_MAX_SPANS = "262144"
            $env:ZIRCON_PROFILE_MAX_COUNTERS = "262144"
            $env:ZIRCON_PROFILE_INCLUDE_PERFETTO = "0"
        }
        $process = Start-Process `
            -FilePath $ViewerExe `
            -ArgumentList (ConvertTo-ZirconShaderPbrCommandLine -Arguments $arguments) `
            -PassThru `
            -WindowStyle Hidden `
            -RedirectStandardOutput $stdoutPath `
            -RedirectStandardError $stderrPath
        if (-not $process.WaitForExit($ViewerTimeoutSeconds * 1000)) {
            Stop-Process -Id $process.Id -Force
            $process.WaitForExit()
            throw "Shader PBR viewer exceeded its profile timeout of $ViewerTimeoutSeconds seconds."
        }
        if ($process.ExitCode -ne 0) {
            throw "Shader PBR viewer exited with code $($process.ExitCode); stderr=$stderrPath"
        }
    }
    catch {
        $primaryFailure = $_
    }
    finally {
        $env:WGPU_BACKEND = $previousBackend
        $env:ZIRCON_PROFILE_CAPTURE = $previousProfileCapture
        $env:ZIRCON_PROFILE_SESSION = $previousProfileSession
        $env:ZIRCON_PROFILE_OUTPUT_ROOT = $previousProfileOutputRoot
        $env:ZIRCON_PROFILE_MAX_FRAMES = $previousProfileMaxFrames
        $env:ZIRCON_PROFILE_MAX_SPANS = $previousProfileMaxSpans
        $env:ZIRCON_PROFILE_MAX_COUNTERS = $previousProfileMaxCounters
        $env:ZIRCON_PROFILE_INCLUDE_PERFETTO = $previousProfileIncludePerfetto
        if ($null -ne $energyCapture) {
            try {
                $energyReport = Stop-ZirconShaderPbrEnergyMeterCapture -Capture $energyCapture
            }
            catch {
                if ($null -eq $cleanupFailure) {
                    $cleanupFailure = $_
                }
            }
        }
        if ($null -ne $wprCapture) {
            try {
                $wprFingerprint = Stop-ZirconShaderPbrWprCapture -Capture $wprCapture
            }
            catch {
                if ($null -eq $cleanupFailure) {
                    $cleanupFailure = $_
                }
            }
        }
    }
    if ($null -ne $primaryFailure) {
        throw $primaryFailure
    }
    if ($null -ne $cleanupFailure) {
        throw $cleanupFailure
    }
    $runtimeProfile = if ($Measure) {
        Get-ZirconShaderPbrRuntimeProfileEvidence `
            -ProfileRoot $runtimeProfileRoot `
            -SessionId $runtimeProfileSession
    } else {
        $null
    }

    $readyValidator = Join-Path $RepoRoot "tools\zircon_validate_shader_pbr_viewer_evidence.py"
    $readyValidationPath = Join-Path $RunDirectory "ready_validation.json"
    $readyValidatorArguments = @(
        $screenshotPath,
        "--expected-backend", $CaptureToolchain.graphics.evidence_backend,
        "--expected-host-mode", "offscreen-diagnostic"
    )
    if (-not [string]::IsNullOrWhiteSpace($DisplayVisualOracle)) {
        $readyValidatorArguments += @("--display-visual-oracle", $DisplayVisualOracle)
    }
    Invoke-ZirconShaderPbrEvidenceValidator `
        -Script $readyValidator `
        -Arguments $readyValidatorArguments `
        -OutputPath $readyValidationPath
    if ($Measure) {
        $gpuValidator = Join-Path $RepoRoot "tools\zircon_validate_shader_pbr_gpu_timing_evidence.py"
        Invoke-ZirconShaderPbrEvidenceValidator `
            -Script $gpuValidator `
            -Arguments @($gpuTimingPath, $screenshotPath) `
            -OutputPath (Join-Path $RunDirectory "gpu_timing_validation.json")
    }

    $sidecar = Read-ZirconShaderPbrKeyValueEvidence -Path $sidecarPath
    if ($sidecar["ibl_staging_status"] -ne $ExpectedStagingStatus) {
        throw "Shader PBR $CacheMode $Role run expected ibl_staging_status=$ExpectedStagingStatus, actual=$($sidecar['ibl_staging_status'])"
    }
    if ($sidecar["material_fixture"] -ne $MaterialFixture) {
        throw "Shader PBR $CacheMode $Role run expected material_fixture=$MaterialFixture, actual=$($sidecar['material_fixture'])"
    }
    $renderdocCapture = $null
    $renderdocReplay = $null
    if ($CaptureRenderDoc) {
        $captures = @(Get-ChildItem -LiteralPath $RunDirectory -File -Filter "*.rdc")
        if ($captures.Count -ne 1) {
            throw "Shader PBR RenderDoc run expected exactly one .rdc capture beneath $RunDirectory, found $($captures.Count)."
        }
        $renderdocValidator = Join-Path $RepoRoot "tools\zircon_validate_shader_pbr_renderdoc_replay.py"
        $renderdocReplayPath = Join-Path $RunDirectory "renderdoc_replay.json"
        Invoke-ZirconShaderPbrEvidenceValidator `
            -Script $renderdocValidator `
            -Arguments @(
                $captures[0].FullName,
                "--renderdoccmd", [string]$CaptureToolchain.renderdoc.command.path
            ) `
            -OutputPath $renderdocReplayPath
        try {
            $renderdocReplayEvidence = Get-Content -LiteralPath $renderdocReplayPath -Raw | ConvertFrom-Json
        }
        catch {
            throw "Shader PBR RenderDoc replay evidence is malformed: $renderdocReplayPath"
        }
        $expectedRenderDocCommand = [System.IO.Path]::GetFullPath(
            [string]$CaptureToolchain.renderdoc.command.path
        )
        $recordedRenderDocCommand = [System.IO.Path]::GetFullPath(
            [string]$renderdocReplayEvidence.renderdoccmd
        )
        if (-not $recordedRenderDocCommand.Equals(
                $expectedRenderDocCommand,
                [System.StringComparison]::OrdinalIgnoreCase
            )) {
            throw "Shader PBR RenderDoc replay did not use the pinned command: expected=$expectedRenderDocCommand actual=$recordedRenderDocCommand"
        }
        $renderdocCapture = Get-ZirconProfileFileFingerprint -Path $captures[0].FullName
        $renderdocReplay = Get-ZirconProfileFileFingerprint -Path $renderdocReplayPath
    }

    $report = [pscustomobject]@{
        schema_version = 1
        profile_kind = "zircon_shader_pbr_viewer_startup_run"
        mode = $CacheMode
        role = $Role
        ordinal = $Ordinal
        started_utc = $startedAt.ToString("o")
        completed_utc = (Get-Date).ToUniversalTime().ToString("o")
        backend = [string]$sidecar["backend"]
        capture_toolchain = [pscustomobject]@{
            manifest = $CaptureToolchain.manifest
            graphics = $CaptureToolchain.graphics
            renderdoc = $CaptureToolchain.renderdoc
        }
        viewer_command = @($arguments)
        viewer_exit_code = $process.ExitCode
        cache_directory = $CacheDirectory
        expected_ibl_staging_status = $ExpectedStagingStatus
        display_visual_oracle = if ([string]::IsNullOrWhiteSpace($DisplayVisualOracle)) {
            $null
        } else {
            Get-ZirconShaderPbrProfileFileFingerprint `
                -Path $DisplayVisualOracle `
                -Description "shader PBR display visual oracle"
        }
        ready_sidecar = [pscustomobject]@{
            schema = $sidecar["schema"]
            material_fixture = $sidecar["material_fixture"]
            required_material_base_pipeline_kind = $sidecar["required_material_base_pipeline_kind"]
            required_material_base_pipeline_ready_at_capture = $sidecar["required_material_base_pipeline_ready_at_capture"]
            environment_only_base_prewarm_requested = $sidecar["environment_only_base_prewarm_requested"]
            screenshot_sha256 = $sidecar["screenshot_sha256"]
            screenshot_byte_length = $sidecar["screenshot_byte_length"]
            evidence_identity_schema = $sidecar["evidence_identity_schema"]
            evidence_run_id = $sidecar["evidence_run_id"]
            evidence_validation_policy = $sidecar["evidence_validation_policy"]
            evidence_identity_path = $sidecar["evidence_identity_path"]
            evidence_identity_sha256 = $sidecar["evidence_identity_sha256"]
            evidence_identity_byte_length = $sidecar["evidence_identity_byte_length"]
            viewer_binary_path = $sidecar["viewer_binary_path"]
            viewer_binary_sha256 = $sidecar["viewer_binary_sha256"]
            viewer_binary_byte_length = $sidecar["viewer_binary_byte_length"]
            hdri_sha256 = $sidecar["hdri_sha256"]
            hdri_byte_length = $sidecar["hdri_byte_length"]
            build_provenance_path = $sidecar["build_provenance_path"]
            build_provenance_sha256 = $sidecar["build_provenance_sha256"]
            build_provenance_byte_length = $sidecar["build_provenance_byte_length"]
            source_manifest_sha256 = $sidecar["source_manifest_sha256"]
            ibl_staging_status = $sidecar["ibl_staging_status"]
            requested_source_face_size = $sidecar["requested_source_face_size"]
            requested_pmrem_face_size = $sidecar["requested_pmrem_face_size"]
            active_source_cubemap_face_size = $sidecar["active_source_cubemap_face_size"]
            active_source_cubemap_mip_count = $sidecar["active_source_cubemap_mip_count"]
            active_pmrem_face_size = $sidecar["active_pmrem_face_size"]
            active_pmrem_mip_count = $sidecar["active_pmrem_mip_count"]
            ibl_staging_source_decode_ns = $sidecar["ibl_staging_source_decode_ns"]
            ibl_staging_cubemap_build_ns = $sidecar["ibl_staging_cubemap_build_ns"]
            ibl_staging_equirect_projection_ns = $sidecar["ibl_staging_equirect_projection_ns"]
            ibl_staging_source_mip_build_ns = $sidecar["ibl_staging_source_mip_build_ns"]
            ibl_staging_pmrem_build_ns = $sidecar["ibl_staging_pmrem_build_ns"]
            ibl_staging_sh9_build_ns = $sidecar["ibl_staging_sh9_build_ns"]
            ibl_staging_irradiance_cube_build_ns = $sidecar["ibl_staging_irradiance_cube_build_ns"]
            ibl_staging_bundle_write_ns = $sidecar["ibl_staging_bundle_write_ns"]
            ibl_staging_parallel_executor_work_items = $sidecar["ibl_staging_parallel_executor_work_items"]
            ibl_staging_equirect_projection_parallel_work_items = $sidecar["ibl_staging_equirect_projection_parallel_work_items"]
            ibl_staging_source_mip_build_parallel_work_items = $sidecar["ibl_staging_source_mip_build_parallel_work_items"]
            ibl_staging_pmrem_build_parallel_work_items = $sidecar["ibl_staging_pmrem_build_parallel_work_items"]
            ibl_staging_irradiance_cube_build_parallel_work_items = $sidecar["ibl_staging_irradiance_cube_build_parallel_work_items"]
            scene_startup_renderer_initialization_ns = $sidecar["scene_startup_renderer_initialization_ns"]
            scene_startup_renderer_deferred_standard_pipeline_ns = $sidecar["scene_startup_renderer_deferred_standard_pipeline_ns"]
            scene_startup_ibl_restore_ns = $sidecar["scene_startup_ibl_restore_ns"]
            scene_startup_total_ns = $sidecar["scene_startup_total_ns"]
            viewer_ready_elapsed_ns = $sidecar["viewer_ready_elapsed_ns"]
            render_pipeline_creation_count = $sidecar["render_pipeline_creation_count"]
            render_pipeline_creation_cpu_microseconds = $sidecar["render_pipeline_creation_cpu_microseconds"]
            shader_module_creation_count = $sidecar["shader_module_creation_count"]
            shader_module_creation_cpu_microseconds = $sidecar["shader_module_creation_cpu_microseconds"]
            async_base_pipeline_queue_wait_count = $sidecar["async_base_pipeline_queue_wait_count"]
            async_base_pipeline_queue_wait_microseconds = $sidecar["async_base_pipeline_queue_wait_microseconds"]
        }
        artifacts = [pscustomobject]@{
            ready_png = Get-ZirconProfileFileFingerprint -Path $screenshotPath
            ready_sidecar = Get-ZirconProfileFileFingerprint -Path $sidecarPath
            evidence_identity = $evidenceIdentity
            ready_validation = Get-ZirconProfileFileFingerprint -Path $readyValidationPath
            gpu_timing = if ($Measure) { Get-ZirconProfileFileFingerprint -Path $gpuTimingPath } else { $null }
            runtime_profile = $runtimeProfile
        cpu_sampling = if ($Measure -and -not $SkipWpr) {
                [pscustomobject]@{
                    status = if ($null -ne $wprFingerprint) { "captured" } else { "unavailable" }
                    etl = $wprFingerprint
                }
            } else {
                [pscustomobject]@{
                    status = "not_requested"
                    etl = $null
                }
            }
            energy_meter = if ($Measure) { $energyReport } else { [pscustomobject]@{ status = "not_requested" } }
            renderdoc_capture = $renderdocCapture
            renderdoc_replay = $renderdocReplay
        }
    }
    $reportPath = Join-Path $RunDirectory "run_report.json"
    $report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $reportPath -Encoding UTF8
    return $report
}

function Invoke-ZirconShaderPbrProfileMode {
    param(
        [Parameter(Mandatory = $true)][ValidateSet("cold", "warm")][string]$Mode,
        [Parameter(Mandatory = $true)][string]$ProfileRoot,
        [Parameter(Mandatory = $true)][string]$ProfileId,
        [Parameter(Mandatory = $true)]$CaptureToolchain,
        [Parameter(Mandatory = $true)][string]$BuildProvenance,
        [ValidateSet("metal-mirror", "dielectric-ior")]
        [string]$MaterialFixture = "metal-mirror",
        [AllowEmptyString()][string]$DisplayVisualOracle = ""
    )

    $modeRoot = Join-Path $ProfileRoot $Mode
    New-Item -ItemType Directory -Force -Path $modeRoot | Out-Null
    $reports = @()
    if ($Mode -eq "warm") {
        $warmCache = Join-Path $modeRoot "shared-ibl-cache"
        $seedDirectory = New-ZirconShaderPbrProfileRunDirectory -ModeRoot $modeRoot -Role "cache_seed" -Ordinal 0
        Invoke-ZirconShaderPbrProfileRun `
            -RunDirectory $seedDirectory `
            -ProfileId $ProfileId `
            -CacheMode $Mode `
            -Role "cache_seed" `
            -Ordinal 0 `
               -CacheDirectory $warmCache `
               -ExpectedStagingStatus "Written" `
               -CaptureToolchain $CaptureToolchain `
               -BuildProvenance $BuildProvenance `
               -MaterialFixture $MaterialFixture `
               -DisplayVisualOracle $DisplayVisualOracle `
               -Measure:$false | Out-Null
        for ($ordinal = 1; $ordinal -le $Repetitions; $ordinal++) {
            $runDirectory = New-ZirconShaderPbrProfileRunDirectory -ModeRoot $modeRoot -Role "measured" -Ordinal $ordinal
            $reports += Invoke-ZirconShaderPbrProfileRun `
                -RunDirectory $runDirectory `
                -ProfileId $ProfileId `
                -CacheMode $Mode `
                -Role "measured" `
                -Ordinal $ordinal `
                   -CacheDirectory $warmCache `
                   -ExpectedStagingStatus "Reused" `
                   -CaptureToolchain $CaptureToolchain `
                   -BuildProvenance $BuildProvenance `
                   -MaterialFixture $MaterialFixture `
                   -DisplayVisualOracle $DisplayVisualOracle `
                   -Measure
        }
    }
    else {
        for ($ordinal = 1; $ordinal -le $Repetitions; $ordinal++) {
            $runDirectory = New-ZirconShaderPbrProfileRunDirectory -ModeRoot $modeRoot -Role "measured" -Ordinal $ordinal
            $reports += Invoke-ZirconShaderPbrProfileRun `
                -RunDirectory $runDirectory `
                -ProfileId $ProfileId `
                -CacheMode $Mode `
                -Role "measured" `
                -Ordinal $ordinal `
                   -CacheDirectory (Join-Path $runDirectory "fresh-ibl-cache") `
                   -ExpectedStagingStatus "Written" `
                   -CaptureToolchain $CaptureToolchain `
                   -BuildProvenance $BuildProvenance `
                   -MaterialFixture $MaterialFixture `
                   -DisplayVisualOracle $DisplayVisualOracle `
                   -Measure
        }
    }
    return @($reports)
}

function Invoke-ZirconShaderPbrProfileCapture {
    Assert-ZirconShaderPbrOptionalFaceSize -Value $FaceSize -Name "-FaceSize"
    Assert-ZirconShaderPbrOptionalFaceSize -Value $PmremFaceSize -Name "-PmremFaceSize"
    $captureToolchain = Resolve-ZirconShaderPbrCaptureToolchain -ManifestPath $CaptureToolchainManifest
    $resolvedEvidenceRoot = Resolve-ZirconShaderPbrProfileEvidenceRoot -RepoRoot $RepoRoot -Path $EvidenceRoot
    $displayVisualOracleFingerprint = $null
    if (-not [string]::IsNullOrWhiteSpace($DisplayVisualOracle)) {
        $DisplayVisualOracle = Resolve-ZirconShaderPbrProfileEvidenceRoot `
            -RepoRoot $RepoRoot `
            -Path $DisplayVisualOracle
        if (-not (Test-Path -LiteralPath $DisplayVisualOracle -PathType Leaf)) {
            throw "Shader PBR display visual oracle is unavailable: $DisplayVisualOracle"
        }
        if ([System.IO.Path]::GetExtension($DisplayVisualOracle) -ne ".json") {
            throw "Shader PBR display visual oracle must be a JSON manifest: $DisplayVisualOracle"
        }
        $displayVisualOracleFingerprint = Get-ZirconShaderPbrProfileFileFingerprint `
            -Path $DisplayVisualOracle `
            -Description "shader PBR display visual oracle"
    }
    $uniqueCacheModes = @($CacheModes | Select-Object -Unique)
    if ($uniqueCacheModes.Count -ne 2 -or -not ($uniqueCacheModes -contains "cold") -or -not ($uniqueCacheModes -contains "warm")) {
        throw "Shader PBR profile requires exactly the cold and warm cache modes."
    }
    $viewerFingerprint = Get-ZirconShaderPbrProfileFileFingerprint -Path $ViewerExe -Description "shader PBR viewer binary"
    $hdriFingerprint = Get-ZirconShaderPbrProfileFileFingerprint -Path $HdriPath -Description "shader PBR HDR input"
    if ([System.IO.Path]::GetExtension($hdriFingerprint.path) -notmatch "^\.hdr$") {
        throw "Shader PBR profile requires a Radiance .hdr input: $($hdriFingerprint.path)"
    }
    if ($CaptureRenderDoc -and -not ($CacheModes -contains "warm")) {
        throw "Shader PBR RenderDoc capture requires the warm cache mode so it can preserve the measured cold/warm matrix."
    }
    if ($CaptureRenderDoc -and $null -eq $captureToolchain.renderdoc) {
        throw "Shader PBR RenderDoc capture requires a pinned RenderDoc DLL in the capture toolchain manifest."
    }
    if ($SkipWpr) {
        Write-Warning "Shader PBR profile is diagnostic only because -SkipWpr omits required CPU attribution."
    }
    $profileId = "shader-pbr-{0}-{1}" -f (Get-Date -Format "yyyyMMdd-HHmmss"), ([guid]::NewGuid().ToString("N").Substring(0, 8))
    $profileCapturesRoot = Resolve-ZirconShaderPbrProfileEvidenceRoot `
        -RepoRoot $RepoRoot `
        -Path (Join-Path $resolvedEvidenceRoot "profile-captures")
    New-Item -ItemType Directory -Force -Path $profileCapturesRoot | Out-Null
    $profileCapturesRoot = Resolve-ZirconShaderPbrProfileEvidenceRoot `
        -RepoRoot $RepoRoot `
        -Path $profileCapturesRoot
    Invoke-ZirconShaderPbrProfileStaleRunScavenger `
        -ProfileCapturesRoot $profileCapturesRoot | Out-Null
    $profileRoot = New-ZirconShaderPbrProfileStagingRoot `
        -ProfileCapturesRoot $profileCapturesRoot `
        -ProfileId $profileId
    $runLease = $null
    $completionReceiptPath = $null
    try {
        $runLease = New-ZirconShaderPbrProfileRunLease `
            -ProfileCapturesRoot $profileCapturesRoot `
            -ProfileRoot $profileRoot `
            -ProfileId $profileId
        Update-ZirconShaderPbrProfileRunLeaseHeartbeat -Lease $runLease | Out-Null
           $machineManifest = New-ZirconPerformanceMachineManifest
           $manifestPath = Export-ZirconShaderPbrProfileManifest `
            -ProfileRoot $profileRoot `
            -ViewerExe $viewerFingerprint.path `
            -HdriPath $hdriFingerprint.path `
            -BuildProvenance $BuildProvenance `
            -EvidenceRoot $resolvedEvidenceRoot `
            -Repetitions $Repetitions `
            -FaceSize $FaceSize `
            -PmremFaceSize $PmremFaceSize `
            -MaterialFixture $MaterialFixture `
            -DisplayVisualOracleFingerprint $displayVisualOracleFingerprint `
            -CacheModes $CacheModes `
            -CaptureToolchain $captureToolchain `
            -MachineManifest $machineManifest

        $modeReports = @{}
        foreach ($mode in ($CacheModes | Select-Object -Unique)) {
            Update-ZirconShaderPbrProfileRunLeaseHeartbeat -Lease $runLease | Out-Null
            $modeReports[$mode] = @(Invoke-ZirconShaderPbrProfileMode `
                -Mode $mode `
                -ProfileRoot $profileRoot `
                -ProfileId $profileId `
                -CaptureToolchain $captureToolchain `
                -BuildProvenance $BuildProvenance `
                -MaterialFixture $MaterialFixture `
                -DisplayVisualOracle $DisplayVisualOracle)
            Update-ZirconShaderPbrProfileRunLeaseHeartbeat -Lease $runLease | Out-Null
        }
        $renderdocReport = $null
        if ($CaptureRenderDoc) {
            $captureDirectory = New-ZirconShaderPbrProfileRunDirectory `
                -ModeRoot (Join-Path $profileRoot "warm") `
                -Role "renderdoc" `
                -Ordinal 1
            $renderdocReport = Invoke-ZirconShaderPbrProfileRun `
                -RunDirectory $captureDirectory `
                -ProfileId $profileId `
                -CacheMode "warm" `
                -Role "renderdoc" `
                -Ordinal 1 `
                -CacheDirectory (Join-Path $profileRoot "warm\shared-ibl-cache") `
                -ExpectedStagingStatus "Reused" `
                -CaptureToolchain $captureToolchain `
                -BuildProvenance $BuildProvenance `
                -MaterialFixture $MaterialFixture `
                -DisplayVisualOracle $DisplayVisualOracle `
                -Measure:$false `
                -CaptureRenderDoc
            Update-ZirconShaderPbrProfileRunLeaseHeartbeat -Lease $runLease | Out-Null
        }
        $summary = [pscustomobject]@{
            schema_version = 1
            profile_kind = "zircon_shader_pbr_viewer_startup_matrix"
            profile_id = $profileId
            profile_manifest = Get-ZirconProfileFileFingerprint -Path $manifestPath
            profile_root = $profileRoot
            repetitions_per_mode = $Repetitions
            source_binary = $viewerFingerprint
            source_hdri = $hdriFingerprint
            display_visual_oracle = $displayVisualOracleFingerprint
            modes = $modeReports
            renderdoc = $renderdocReport
            driver_cache_note = "The profile controls process and caller-owned IBL caches only. It does not clear DX12 or driver caches."
        }
        $summaryPath = Join-Path $profileRoot "profile_summary.json"
        $summary | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $summaryPath -Encoding UTF8
        $analysisPath = Join-Path $profileRoot "profile_analysis.json"
        $summarizer = Join-Path $RepoRoot "tools\zircon_summarize_shader_pbr_profile.py"
        Invoke-ZirconShaderPbrEvidenceValidator `
            -Script $summarizer `
            -Arguments @($summaryPath, "--output", $analysisPath) `
            -OutputPath (Join-Path $profileRoot "profile_analysis_validation.log")
        Update-ZirconShaderPbrProfileRunLeaseHeartbeat -Lease $runLease | Out-Null
        $completionReceiptPath = Publish-ZirconShaderPbrProfileCompletion `
            -ProfileCapturesRoot $profileCapturesRoot `
            -ProfileRoot $profileRoot `
            -ProfileId $profileId
        Complete-ZirconShaderPbrProfileRunLease `
            -Lease $runLease `
            -ReceiptPath $completionReceiptPath | Out-Null
    }
    catch {
        $captureFailure = $_
        if ($null -eq $completionReceiptPath) {
            try {
                Write-ZirconShaderPbrProfileIncompleteReceipt `
                    -ProfileRoot $profileRoot `
                    -ProfileId $profileId `
                    -FailureMessage $captureFailure.Exception.Message | Out-Null
            }
            catch {
                Write-Warning "Shader PBR profile could not write incomplete receipt: $($_.Exception.Message)"
            }
            if ($null -ne $runLease) {
                try {
                    Fail-ZirconShaderPbrProfileRunLease `
                        -Lease $runLease `
                        -FailureMessage $captureFailure.Exception.Message | Out-Null
                }
                catch {
                    Write-Warning "Shader PBR profile could not persist failed lease state: $($_.Exception.Message)"
                }
            }
        }
        else {
            Write-Warning "Shader PBR profile completion receipt was already created; preserving its immutable artifact closure."
        }
        throw $captureFailure
    }
    finally {
        if ($null -ne $runLease) {
            Close-ZirconShaderPbrProfileRunLease -Lease $runLease
        }
    }
    Write-Host "Shader PBR profile summary: $summaryPath"
    Write-Host "Shader PBR profile analysis: $analysisPath"
    Write-Host "Shader PBR profile completion receipt: $completionReceiptPath"
}

if ($MyInvocation.InvocationName -ne ".") {
    Invoke-ZirconShaderPbrProfileCapture
}
