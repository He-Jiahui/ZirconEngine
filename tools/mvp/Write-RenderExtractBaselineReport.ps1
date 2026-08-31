[CmdletBinding()]
param(
    [string]$BaselineSummaryPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Import-Module (Join-Path $repoRoot 'tools\mvp\RenderExtractBaselineEvidence.psm1') -Force -DisableNameChecking -ErrorAction Stop
Import-Module (Join-Path $repoRoot 'tools\mvp\RenderExtractBaselineMetrics.psm1') -Force -DisableNameChecking -ErrorAction Stop
Import-Module (Join-Path $repoRoot 'tools\mvp\RenderExtractMachineEvidence.psm1') -Force -DisableNameChecking -ErrorAction Stop
Import-Module (Join-Path $repoRoot 'tools\mvp\RenderExtractPerformanceScenario.psm1') -Force -DisableNameChecking -ErrorAction Stop
Import-Module (Join-Path $repoRoot 'tools\WindowsPathResolver.psm1') -Force -ErrorAction Stop

$script:RenderExtractRequiredScenarios = @(Get-RenderExtractPerformanceScenarioDefinitions)

function Get-RenderExtractScenarioBudgetEvaluation {
    param(
        [Parameter(Mandatory)]$BudgetContract,
        [Parameter(Mandatory)]$FrameStatistics
    )

    if ($BudgetContract.status -eq 'unconfigured') {
        return [pscustomobject][ordered]@{
            status = 'not_evaluated'
            reason = $BudgetContract.reason
        }
    }
    if ($BudgetContract.status -ne 'declared' -or
        $BudgetContract.metric_id -ne 'app.runtime_redraw.frame_duration_us' -or
        $BudgetContract.aggregation -ne 'p95' -or
        $BudgetContract.comparator -ne 'less_than_or_equal' -or
        $BudgetContract.unit -ne 'us') {
        throw 'Render-extract scenario has an unsupported budget contract.'
    }
    $observed = [double]$FrameStatistics.p95
    $threshold = [double]$BudgetContract.threshold
    return [pscustomobject][ordered]@{
        status = if ($observed -le $threshold) { 'within_budget' } else { 'over_budget' }
        metric_id = $BudgetContract.metric_id
        aggregation = $BudgetContract.aggregation
        comparator = $BudgetContract.comparator
        observed = $observed
        threshold = $threshold
        unit = $BudgetContract.unit
    }
}

function ConvertTo-RenderExtractBaselineReportMarkdown {
    param([Parameter(Mandatory)]$Report)

    $lines = [System.Collections.Generic.List[string]]::new()
    $lines.Add('# Render Extract Baseline Report')
    $lines.Add('')
    $lines.Add(('- Source fingerprint: `' + $Report.source_fingerprint + '`'))
    $lines.Add(('- Raw baseline summary SHA-256: `' + $Report.raw_evidence.summary_sha256 + '`'))
    $lines.Add(('- Qualification: `' + $Report.qualification.status + '`'))
    if ($null -ne $Report.project.scale_project) {
        $lines.Add(('- Primitive count: `' + $Report.project.scale_project.primitive_count + '`'))
        $lines.Add(('- Scene virtual path: `' + $Report.project.scale_project.scene_virtual_path + '`'))
    }
    $lines.Add('- Percentiles: upper-nearest index, matching the runtime hotspot analyzer.')
    $lines.Add('')
    $lines.Add('## Scenarios')
    foreach ($scenario in $Report.scenarios) {
        $window = if ($scenario.warmup_presented_frame_count -gt 0) {
            "; warmup=$($scenario.warmup_presented_frame_count) presented frames discarded, measured=$($scenario.measured_presented_frame_count) app/runtime_redraw frames; process/CPU/working-set metrics include warmup"
        }
        else {
            ''
        }
        $lines.Add("- `$($scenario.logical_id)` ($($scenario.product), $($scenario.measurement_window)$window): attempts=$($scenario.attempt_count), process median=$($scenario.process_elapsed_ms.median) ms p95=$($scenario.process_elapsed_ms.p95) ms, CPU median=$($scenario.total_processor_time_ms.median) ms p95=$($scenario.total_processor_time_ms.p95) ms, peak working set median=$($scenario.peak_working_set_bytes.median) bytes p95=$($scenario.peak_working_set_bytes.p95) bytes, frame median=$($scenario.frame_duration_us.median) us p95=$($scenario.frame_duration_us.p95) us p99=$($scenario.frame_duration_us.p99) us, budget=$($scenario.budget_evaluation.status)")
    }
    $lines.Add('')
    $lines.Add('## Instrumentation Coverage')
    $lines.Add("- CPU timeline: $($Report.measurement_coverage.cpu_timeline.status)")
    $lines.Add("- Lock wait: $($Report.measurement_coverage.lock_wait.status)")
    $lines.Add("- Queue and backpressure: $($Report.measurement_coverage.queue_backpressure.status)")
    $lines.Add("- Worker submission occupancy: $($Report.measurement_coverage.worker_utilization.status)")
    $lines.Add("- Scene extract ownership: $($Report.measurement_coverage.scene_extract.status)")
    $lines.Add("- App frame cadence: $($Report.measurement_coverage.app_cadence.status)")
    $lines.Add("- Surface presentation and capture: $($Report.measurement_coverage.surface_presentation.status)")
    $lines.Add("- Asset-management generation queries: $($Report.measurement_coverage.asset_management.status)")
    $lines.Add("- Asset-management generation pages: $($Report.measurement_coverage.asset_management_page.status)")
    $lines.Add("- Product-process CPU time: $($Report.measurement_coverage.process_cpu.status)")
    $lines.Add("- CPU scheduling from WPR: $($Report.measurement_coverage.cpu_scheduling.status)")
    $lines.Add("- GPU timing: $($Report.measurement_coverage.gpu_timing.status)")
    $lines.Add("- System power: $($Report.measurement_coverage.system_power.status)")
    $lines.Add("- Working set: $($Report.measurement_coverage.working_set.status)")
    $lines.Add("- Disk I/O: $($Report.measurement_coverage.disk_io.status)")
    $lines.Add('')
    $lines.Add('GPU timing and system power remain not measured unless a calibrated source is added; generic runtime counters do not establish either measurement.')
    return ($lines -join [Environment]::NewLine) + [Environment]::NewLine
}

function Get-RenderExtractBaselineRunMeasurement {
    param(
        [Parameter(Mandatory)]$Run,
        [Parameter(Mandatory)]$Scenario,
        [Parameter(Mandatory)][object[]]$Frames,
        [Parameter(Mandatory)][object[]]$Spans,
        [Parameter(Mandatory)][object[]]$Counters,
        [Parameter(Mandatory)][string]$Label
    )

    $warmupPresentedFrameCount = Get-RenderExtractTimelineInteger `
            -Value $Run `
            -Name 'warmup_presented_frame_count' `
            -Label $Label
    $measuredPresentedFrameCount = Get-RenderExtractTimelineInteger `
            -Value $Run `
            -Name 'measured_presented_frame_count' `
            -Label $Label
    $targetPresentedFrameCount = Get-RenderExtractTimelineInteger `
            -Value $Run `
            -Name 'target_presented_frame_count' `
            -Label $Label
    if ($warmupPresentedFrameCount -gt 1000000 -or $measuredPresentedFrameCount -gt 1000000 -or
        $targetPresentedFrameCount -gt 2000000 -or $measuredPresentedFrameCount -lt 1 -or
        $targetPresentedFrameCount -lt 1) {
        throw "$Label has invalid presented-frame window counts."
    }

    $isSteady = $Scenario.measurement_window -eq 'steady-presented-frames-after-warmup'
    if (-not $isSteady) {
        if ($warmupPresentedFrameCount -ne 0 -or $measuredPresentedFrameCount -ne 1 -or
            $targetPresentedFrameCount -ne 1) {
            throw "$Label cold capture must declare exactly one measured presented frame and no warmup."
        }
        return [pscustomobject][ordered]@{
            measurement_window = $Scenario.measurement_window
            warmup_presented_frame_count = $warmupPresentedFrameCount
            measured_presented_frame_count = $measuredPresentedFrameCount
            target_presented_frame_count = $targetPresentedFrameCount
            primary_frame_stream = $null
            primary_frame_name = $null
            frames = $Frames
            spans = $Spans
            counters = $Counters
        }
    }

    if ($warmupPresentedFrameCount -lt 60 -or $measuredPresentedFrameCount -lt 300) {
        throw "$Label steady capture must discard at least 60 warmup frames and measure at least 300 presented frames."
    }
    if ($targetPresentedFrameCount -ne ($warmupPresentedFrameCount + $measuredPresentedFrameCount)) {
        throw "$Label target_presented_frame_count must equal warmup plus measured presented frames."
    }
    $window = Get-RenderExtractSteadyMeasurementWindow `
        -Frames $Frames `
        -WarmupPresentedFrameCount $warmupPresentedFrameCount `
        -MeasuredPresentedFrameCount $measuredPresentedFrameCount `
        -Label $Label
    $samples = Select-RenderExtractTimelineWindowSamples `
        -Frames $Frames `
        -Spans $Spans `
        -Counters $Counters `
        -Window $window
    return [pscustomobject][ordered]@{
        measurement_window = $Scenario.measurement_window
        warmup_presented_frame_count = $warmupPresentedFrameCount
        measured_presented_frame_count = $measuredPresentedFrameCount
        target_presented_frame_count = $targetPresentedFrameCount
        primary_frame_stream = $window.primary_frame_stream
        primary_frame_name = $window.primary_frame_name
        frames = $samples.frames
        spans = $samples.spans
        counters = $samples.counters
    }
}

function Write-RenderExtractBaselineReport {
    param([Parameter(Mandatory)][string]$BaselineSummaryPath)

    $summaryResolution = Resolve-ZirconWindowsPath -Path $BaselineSummaryPath
    if (-not [IO.File]::Exists($summaryResolution.OperationalPath)) {
        throw "Render-extract baseline summary does not exist: $($summaryResolution.DisplayPath)"
    }
    if ([IO.Path]::GetFileName($summaryResolution.OperationalPath) -ne 'render-extract-baseline.json') {
        throw "Baseline report requires render-extract-baseline.json, got: $($summaryResolution.DisplayPath)"
    }
    $evidenceDirectory = Assert-RenderExtractBaselineEvidenceDirectory -Path ([IO.Path]::GetDirectoryName($summaryResolution.OperationalPath))
    $summarySnapshot = Read-RenderExtractJsonEvidence `
        -Path $summaryResolution.OperationalPath `
        -Label 'Render-extract baseline summary'
    $summary = $summarySnapshot.json
    if ([int](Get-RenderExtractReportProperty -Value $summary -Name 'schema_version' -Label 'Baseline summary') -ne 5) {
        throw 'Baseline summary schema_version must be 5.'
    }
    $sourceFingerprint = [string](Get-RenderExtractReportProperty -Value $summary -Name 'source_fingerprint' -Label 'Baseline summary')
    if ($sourceFingerprint -notmatch '^[0-9A-Fa-f]{64}$') {
        throw 'Baseline summary source_fingerprint must be a SHA-256 hexadecimal value.'
    }
    $inputManifestSha = [string](Get-RenderExtractReportProperty -Value $summary -Name 'profiling_input_manifest_sha256' -Label 'Baseline summary')
    if ($inputManifestSha -notmatch '^[0-9A-Fa-f]{64}$') {
        throw 'Baseline summary profiling_input_manifest_sha256 must be a SHA-256 hexadecimal value.'
    }
    $buildSetId = [string](Get-RenderExtractReportProperty -Value $summary -Name 'build_set_id' -Label 'Baseline summary')
    $buildSetManifestSha = [string](Get-RenderExtractReportProperty -Value $summary -Name 'build_set_manifest_sha256' -Label 'Baseline summary')
    if ($buildSetId -notmatch '^[0-9A-Fa-f]{64}$' -or $buildSetManifestSha -notmatch '^[0-9A-Fa-f]{64}$') {
        throw 'Baseline summary BuildSet identity must contain SHA-256 hexadecimal values.'
    }
    $project = Get-RenderExtractReportProject -Summary $summary
    $machineReference = Get-RenderExtractReportProperty `
        -Value $summary `
        -Name 'machine_manifest' `
        -Label 'Baseline summary'
    $machineEvidence = Resolve-RenderExtractMachineEvidence `
        -Reference $machineReference `
        -EvidenceDirectory $evidenceDirectory.OperationalPath
    $runs = @(Get-RenderExtractReportArrayProperty -Value $summary -Name 'runs' -Label 'Baseline summary')
    if ($runs.Count -eq 0) {
        throw 'Baseline summary contains no runs.'
    }
    $summaryInvocationId = [string](Get-RenderExtractReportProperty -Value $summary -Name 'invocation_id' -Label 'Baseline summary')
    if ($summaryInvocationId -notmatch '^[0-9A-Fa-f]{32}$') {
        throw 'Baseline summary invocation_id must be a 32-character hexadecimal identifier.'
    }

    $profilesRoot = Join-ZirconWindowsPath -Path $evidenceDirectory.OperationalPath -ChildPath 'profiles'
    $capturesRoot = Join-ZirconWindowsPath -Path $evidenceDirectory.OperationalPath -ChildPath 'captures'
    $tracesRoot = Join-ZirconWindowsPath -Path $evidenceDirectory.OperationalPath -ChildPath 'traces'
    $rawEvidence = [System.Collections.Generic.List[object]]::new()
    $frameCaptureEvidence = [System.Collections.Generic.List[object]]::new()
    $systemTraceEvidence = [System.Collections.Generic.List[object]]::new()
    $scenarioRecords = [System.Collections.Generic.List[object]]::new()
    $expectedProfilingInputs = @{}
    foreach ($run in $runs) {
        $logicalId = [string](Get-RenderExtractReportProperty -Value $run -Name 'logical_id' -Label 'Baseline run')
        if ($logicalId -notmatch '^[A-Za-z0-9][A-Za-z0-9._-]*$') {
            throw "Baseline run has invalid logical_id '$logicalId'."
        }
        $scenario = @($script:RenderExtractRequiredScenarios | Where-Object { $_.logical_id -eq $logicalId })
        if ($scenario.Count -ne 1) {
            throw "Baseline run '$logicalId' is not a required scenario."
        }
        $scenarioBinding = Resolve-RenderExtractPerformanceScenarioRunBinding -Run $run
        $product = [string](Get-RenderExtractReportProperty -Value $run -Name 'product' -Label "Baseline run '$logicalId'")
        if ($product -ne $scenario[0].product) {
            throw "Baseline run '$logicalId' has product '$product'; expected '$($scenario[0].product)'."
        }
        $runtimeProfile = [string](Get-RenderExtractReportProperty -Value $run -Name 'runtime_profile' -Label "Baseline run '$logicalId'")
        if ($runtimeProfile -ne $scenario[0].runtime_profile) {
            throw "Baseline run '$logicalId' has runtime_profile '$runtimeProfile'; expected '$($scenario[0].runtime_profile)'."
        }
        $attempt = [int](Get-RenderExtractReportProperty -Value $run -Name 'attempt' -Label "Baseline run '$logicalId'")
        if ($attempt -lt 1) {
            throw "Baseline run '$logicalId' has invalid attempt '$attempt'."
        }
        $exitCode = [int](Get-RenderExtractReportProperty -Value $run -Name 'exit_code' -Label "Baseline run '$logicalId'")
        if ($exitCode -ne 0) {
            throw "Baseline run '$logicalId' attempt $attempt did not succeed (exit code $exitCode)."
        }
        $peakWorkingSetBytes = [Int64](Get-RenderExtractReportProperty -Value $run -Name 'peak_working_set_bytes' -Label "Baseline run '$logicalId'")
        if ($peakWorkingSetBytes -le 0) {
            throw "Baseline run '$logicalId' attempt $attempt has invalid peak_working_set_bytes '$peakWorkingSetBytes'."
        }
        $totalProcessorTimeMs = [double](Get-RenderExtractReportProperty -Value $run -Name 'total_processor_time_ms' -Label "Baseline run '$logicalId'")
        if ($totalProcessorTimeMs -lt 0) {
            throw "Baseline run '$logicalId' attempt $attempt has invalid total_processor_time_ms '$totalProcessorTimeMs'."
        }
        $processId = Get-RenderExtractProcessId -Run $run
        $invocationId = [string](Get-RenderExtractReportProperty -Value $run -Name 'invocation_id' -Label "Baseline run '$logicalId'")
        if ($invocationId -notmatch '^[0-9A-Fa-f]{32}$') {
            throw "Baseline run '$logicalId' attempt $attempt has invalid invocation_id '$invocationId'."
        }
        if (-not $invocationId.Equals($summaryInvocationId, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Baseline run '$logicalId' attempt $attempt invocation_id does not match summary invocation_id."
        }
        $profilingInput = Get-RenderExtractReportProperty -Value $run -Name 'profiling_input' -Label "Baseline run '$logicalId'"
        $runManifestSha = [string](Get-RenderExtractReportProperty -Value $profilingInput -Name 'manifest_sha256' -Label "Baseline run '$logicalId' profiling input")
        $runBuildSetId = [string](Get-RenderExtractReportProperty -Value $profilingInput -Name 'build_set_id' -Label "Baseline run '$logicalId' profiling input")
        $runBuildSetManifestSha = [string](Get-RenderExtractReportProperty -Value $profilingInput -Name 'build_set_manifest_sha256' -Label "Baseline run '$logicalId' profiling input")
        $runExecutableSha = [string](Get-RenderExtractReportProperty -Value $profilingInput -Name 'executable_sha256' -Label "Baseline run '$logicalId' profiling input")
        $runLibrarySha = [string](Get-RenderExtractReportProperty -Value $profilingInput -Name 'library_sha256' -Label "Baseline run '$logicalId' profiling input")
        $runAssetManifestSha = [string](Get-RenderExtractReportProperty -Value $profilingInput -Name 'asset_manifest_sha256' -Label "Baseline run '$logicalId' profiling input")
        $runAssetFileCount = [int](Get-RenderExtractReportProperty -Value $profilingInput -Name 'asset_file_count' -Label "Baseline run '$logicalId' profiling input")
        $runAssetBytes = [Int64](Get-RenderExtractReportProperty -Value $profilingInput -Name 'asset_bytes' -Label "Baseline run '$logicalId' profiling input")
        foreach ($hash in @($runManifestSha, $runBuildSetId, $runBuildSetManifestSha, $runExecutableSha, $runLibrarySha, $runAssetManifestSha)) {
            if ($hash -notmatch '^[0-9A-Fa-f]{64}$') {
                throw "Baseline run '$logicalId' attempt $attempt has an invalid profiling input SHA-256."
            }
        }
        if ($runAssetFileCount -lt 1 -or $runAssetBytes -lt 1) {
            throw "Baseline run '$logicalId' attempt $attempt has invalid frozen asset evidence."
        }
        if (-not $runManifestSha.Equals($inputManifestSha, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Baseline run '$logicalId' attempt $attempt profiling input manifest SHA-256 does not match the capture summary."
        }
        if (-not $runBuildSetId.Equals($buildSetId, [StringComparison]::OrdinalIgnoreCase) -or
            -not $runBuildSetManifestSha.Equals($buildSetManifestSha, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Baseline run '$logicalId' attempt $attempt BuildSet identity does not match the capture summary."
        }
        $runProfilingInput = [pscustomobject]@{
            manifest_sha256 = $runManifestSha.ToUpperInvariant()
            build_set_id = $runBuildSetId.ToUpperInvariant()
            build_set_manifest_sha256 = $runBuildSetManifestSha.ToUpperInvariant()
            executable_sha256 = $runExecutableSha.ToUpperInvariant()
            library_sha256 = $runLibrarySha.ToUpperInvariant()
            asset_manifest_sha256 = $runAssetManifestSha.ToUpperInvariant()
            asset_file_count = $runAssetFileCount
            asset_bytes = $runAssetBytes
        }
        if (-not $expectedProfilingInputs.ContainsKey($product)) {
            $expectedProfilingInputs[$product] = $runProfilingInput
        }
        elseif (-not ($expectedProfilingInputs[$product].executable_sha256 -eq $runProfilingInput.executable_sha256 -and
                $expectedProfilingInputs[$product].library_sha256 -eq $runProfilingInput.library_sha256 -and
                $expectedProfilingInputs[$product].asset_manifest_sha256 -eq $runProfilingInput.asset_manifest_sha256 -and
                $expectedProfilingInputs[$product].asset_file_count -eq $runProfilingInput.asset_file_count -and
                $expectedProfilingInputs[$product].asset_bytes -eq $runProfilingInput.asset_bytes)) {
            throw "Baseline run '$logicalId' attempt $attempt profiling input differs from the $product capture session."
        }
        $invocationProfilesRoot = Join-ZirconWindowsPath -Path $profilesRoot -ChildPath $invocationId
        $profileDirectory = Resolve-ZirconWindowsPath -Path ([string](Get-RenderExtractReportProperty -Value $run -Name 'profile_directory' -Label "Baseline run '$logicalId'"))
        if (-not [IO.Directory]::Exists($profileDirectory.OperationalPath) -or
            -not (Test-RenderExtractPathWithinDirectory -CandidatePath $profileDirectory.OperationalPath -RootPath $invocationProfilesRoot)) {
            throw "Baseline run '$logicalId' attempt $attempt profile directory is outside this evidence session."
        }
        $expectedProfileDirectory = Join-ZirconWindowsPath -Path $invocationProfilesRoot -ChildPath "$logicalId-$attempt"
        if (-not $profileDirectory.OperationalPath.Equals($expectedProfileDirectory, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Baseline run '$logicalId' attempt $attempt profile directory does not match its deterministic session id."
        }

        $timelinePath = Join-ZirconWindowsPath -Path $profileDirectory.OperationalPath -ChildPath 'timeline.zrtrace.json'
        $timelineSnapshot = Read-RenderExtractJsonEvidence `
            -Path $timelinePath `
            -Label "Timeline '$logicalId-$attempt'"
        $rawEvidence.Add([ordered]@{
                logical_id = $logicalId
                attempt = $attempt
                kind = 'timeline.zrtrace.json'
                path = $timelineSnapshot.path
                bytes = $timelineSnapshot.bytes
                sha256 = $timelineSnapshot.sha256
            }) | Out-Null
        foreach ($artifact in @(
                [pscustomobject]@{ name = 'hotspots.json'; path = (Join-ZirconWindowsPath -Path $profileDirectory.OperationalPath -ChildPath 'hotspots.json') },
                [pscustomobject]@{ name = 'counter_hotspots.json'; path = (Join-ZirconWindowsPath -Path $profileDirectory.OperationalPath -ChildPath 'counter_hotspots.json') },
                [pscustomobject]@{ name = 'summary.md'; path = (Join-ZirconWindowsPath -Path $profileDirectory.OperationalPath -ChildPath 'summary.md') }
            )) {
            $rawEvidence.Add((Get-RenderExtractFileEvidence -Path $artifact.path -Kind $artifact.name -LogicalId $logicalId -Attempt $attempt)) | Out-Null
        }
        $uiHotspotsPath = Join-ZirconWindowsPath -Path $profileDirectory.OperationalPath -ChildPath 'ui_hotspots.json'
        if ([IO.File]::Exists($uiHotspotsPath)) {
            $rawEvidence.Add((Get-RenderExtractFileEvidence -Path $uiHotspotsPath -Kind 'ui_hotspots.json' -LogicalId $logicalId -Attempt $attempt)) | Out-Null
        }
        $invocationCapturesRoot = Join-ZirconWindowsPath -Path $capturesRoot -ChildPath $invocationId
        $frameCapture = Resolve-ZirconWindowsPath -Path ([string](Get-RenderExtractReportProperty -Value $run -Name 'frame_capture_png' -Label "Baseline run '$logicalId'"))
        if (-not (Test-RenderExtractPathWithinDirectory -CandidatePath $frameCapture.OperationalPath -RootPath $invocationCapturesRoot)) {
            throw "Baseline run '$logicalId' attempt $attempt frame_capture_png is outside this evidence session."
        }
        $expectedFrameCapture = Join-ZirconWindowsPath -Path $invocationCapturesRoot -ChildPath "$logicalId-$attempt.png"
        if (-not $frameCapture.OperationalPath.Equals($expectedFrameCapture, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Baseline run '$logicalId' attempt $attempt frame_capture_png does not match its deterministic session id."
        }
        $frameCaptureEvidence.Add((Get-RenderExtractFileEvidence -Path $frameCapture.OperationalPath -Kind 'frame_capture_png' -LogicalId $logicalId -Attempt $attempt)) | Out-Null
        $systemTraceProperty = $run.PSObject.Properties['system_trace_etl']
        if ($null -ne $systemTraceProperty -and -not [string]::IsNullOrWhiteSpace([string]$systemTraceProperty.Value)) {
            $invocationTracesRoot = Join-ZirconWindowsPath -Path $tracesRoot -ChildPath $invocationId
            $systemTrace = Resolve-ZirconWindowsPath -Path ([string]$systemTraceProperty.Value)
            if (-not (Test-RenderExtractPathWithinDirectory -CandidatePath $systemTrace.OperationalPath -RootPath $invocationTracesRoot)) {
                throw "Baseline run '$logicalId' attempt $attempt system trace is outside this evidence session."
            }
            $expectedSystemTrace = Join-ZirconWindowsPath -Path $invocationTracesRoot -ChildPath "$logicalId-$attempt.etl"
            if (-not $systemTrace.OperationalPath.Equals($expectedSystemTrace, [StringComparison]::OrdinalIgnoreCase)) {
                throw "Baseline run '$logicalId' attempt $attempt system trace does not match its deterministic session id."
            }
            $systemTraceArtifact = Get-RenderExtractFileEvidence `
                -Path $systemTrace.OperationalPath `
                -Kind 'system_trace_etl' `
                -LogicalId $logicalId `
                -Attempt $attempt
            $systemTraceArtifact['process_id'] = $processId
            $systemTraceEvidence.Add($systemTraceArtifact) | Out-Null
        }

        $timeline = $timelineSnapshot.json
        $timelineSession = [string](Get-RenderExtractReportProperty -Value $timeline -Name 'session_id' -Label "Timeline '$logicalId-$attempt'")
        if ($timelineSession -ne "$logicalId-$attempt") {
            throw "Timeline session_id '$timelineSession' does not match baseline run '$logicalId-$attempt'."
        }
        $frames = @(Get-RenderExtractReportArrayProperty -Value $timeline -Name 'frames' -Label "Timeline '$logicalId-$attempt'")
        if ($frames.Count -eq 0) {
            throw "Timeline '$logicalId-$attempt' contains no frame samples."
        }
        $spans = @(Get-RenderExtractReportArrayProperty -Value $timeline -Name 'spans' -Label "Timeline '$logicalId-$attempt'")
        $counters = @(Get-RenderExtractReportArrayProperty -Value $timeline -Name 'counters' -Label "Timeline '$logicalId-$attempt'")
        $measurement = Get-RenderExtractBaselineRunMeasurement `
            -Run $run `
            -Scenario $scenario[0] `
            -Frames $frames `
            -Spans $spans `
            -Counters $counters `
            -Label "Baseline run '$logicalId' attempt $attempt"
        $scenarioRecords.Add([pscustomobject]@{
                logical_id = $logicalId
                scenario_id = $scenarioBinding.scenario_id
                scenario_version = $scenarioBinding.scenario_version
                scenario_binding_id = $scenarioBinding.scenario_binding_id
                product = $product
                measurement_window = $measurement.measurement_window
                repeat_count = $scenarioBinding.repeat_count
                warmup_presented_frame_count = $measurement.warmup_presented_frame_count
                measured_presented_frame_count = $measurement.measured_presented_frame_count
                target_presented_frame_count = $measurement.target_presented_frame_count
                primary_frame_stream = $measurement.primary_frame_stream
                primary_frame_name = $measurement.primary_frame_name
                cache_contract = $scenarioBinding.cache_contract
                required_metrics = @($scenarioBinding.required_metrics)
                budget_contract = $scenarioBinding.budget_contract
                attempt = $attempt
                process_id = $processId
                process_elapsed_ms = Get-RenderExtractProcessElapsedMilliseconds -Run $run
                peak_working_set_bytes = [double]$peakWorkingSetBytes
                total_processor_time_ms = $totalProcessorTimeMs
                frames = $measurement.frames
                spans = $measurement.spans
                counters = $measurement.counters
        }) | Out-Null
    }

    foreach ($requiredScenario in $script:RenderExtractRequiredScenarios) {
        if (@($scenarioRecords | Where-Object { $_.logical_id -eq $requiredScenario.logical_id }).Count -eq 0) {
            throw "Baseline summary is missing required scenario '$($requiredScenario.logical_id)'."
        }
    }

    $scenarios = foreach ($scenarioGroup in @($scenarioRecords | Group-Object -Property logical_id)) {
        $attemptRecords = @($scenarioGroup.Group | Sort-Object attempt)
        if ($attemptRecords.Count -lt 3) {
            throw "Baseline scenario '$($scenarioGroup.Name)' requires at least 3 successful attempts before publishing percentiles."
        }
        $attemptIds = @($attemptRecords | ForEach-Object { $_.attempt })
        if (@($attemptIds | Select-Object -Unique).Count -ne $attemptIds.Count) {
            throw "Baseline scenario '$($scenarioGroup.Name)' has duplicate attempt identifiers."
        }
        $expectedAttemptCount = [int]$attemptRecords[0].repeat_count
        if ($attemptRecords.Count -ne $expectedAttemptCount -or
            ($attemptIds -join ',') -ne ((1..$expectedAttemptCount) -join ',')) {
            throw "Baseline scenario '$($scenarioGroup.Name)' does not contain its complete declared attempt set."
        }
        $bindingIds = @($attemptRecords | ForEach-Object { $_.scenario_binding_id } | Select-Object -Unique)
        if ($bindingIds.Count -ne 1) {
            throw "Baseline scenario '$($scenarioGroup.Name)' mixes different scenario bindings."
        }
        $frameRecords = [System.Collections.Generic.List[object]]::new()
        $spanRecords = [System.Collections.Generic.List[object]]::new()
        $counterRecords = [System.Collections.Generic.List[object]]::new()
        $workerUtilizationAttempts = [System.Collections.Generic.List[object]]::new()
        foreach ($record in $attemptRecords) {
            $workerOccupancy = Get-RenderExtractSchedulerWorkerOccupancyAttempt `
                -Counters @($record.counters) `
                -Label "Baseline run '$($record.logical_id)' attempt $($record.attempt)"
            $workerUtilizationAttempts.Add([pscustomobject][ordered]@{
                    attempt = [int]$record.attempt
                    status = $workerOccupancy.status
                    reason = $workerOccupancy.reason
                    sample_count = $workerOccupancy.sample_count
                    observed_window_us = $workerOccupancy.observed_window_us
                    busy_duration_us = $workerOccupancy.busy_duration_us
                    occupancy_ratio = $workerOccupancy.occupancy_ratio
                }) | Out-Null
            foreach ($frame in $record.frames) {
                $frameRecords.Add([pscustomobject]@{
                        duration_us = [double](Get-RenderExtractReportProperty -Value $frame -Name 'duration_us' -Label 'Frame sample')
                        over_budget = [bool](Get-RenderExtractReportProperty -Value $frame -Name 'over_budget' -Label 'Frame sample')
                    }) | Out-Null
            }
            foreach ($span in $record.spans) {
                $stream = [string](Get-RenderExtractReportProperty -Value $span -Name 'stream' -Label 'Span sample')
                $category = [string](Get-RenderExtractReportProperty -Value $span -Name 'category' -Label 'Span sample')
                $name = [string](Get-RenderExtractReportProperty -Value $span -Name 'name' -Label 'Span sample')
                $path = [string](Get-RenderExtractReportProperty -Value $span -Name 'path' -Label 'Span sample')
                $spanRecords.Add([pscustomobject]@{
                        group_key = "$stream$([char]0)$category$([char]0)$name$([char]0)$path"
                        stream = $stream
                        category = $category
                        name = $name
                        path = $path
                        duration_us = [double](Get-RenderExtractReportProperty -Value $span -Name 'duration_us' -Label 'Span sample')
                    }) | Out-Null
            }
            foreach ($counter in $record.counters) {
                $stream = [string](Get-RenderExtractReportProperty -Value $counter -Name 'stream' -Label 'Counter sample')
                $name = [string](Get-RenderExtractReportProperty -Value $counter -Name 'name' -Label 'Counter sample')
                $counterRecords.Add([pscustomobject]@{
                        group_key = "$stream$([char]0)$name"
                        stream = $stream
                        name = $name
                        value = [double](Get-RenderExtractReportProperty -Value $counter -Name 'value' -Label 'Counter sample')
                    }) | Out-Null
            }
        }
        $frameStatistics = Get-RenderExtractStatistics -Values @($frameRecords | ForEach-Object { $_.duration_us })
        $frameStatistics.over_budget_count = @($frameRecords | Where-Object { $_.over_budget }).Count
        $frameStatistics.over_budget_rate = $frameStatistics.over_budget_count / $frameStatistics.sample_count
        $spanAggregates = @(Get-RenderExtractAggregates -Records @($spanRecords) -Kind 'span')
        $counterAggregates = @(Get-RenderExtractAggregates -Records @($counterRecords) -Kind 'counter')
        $measuredWorkerUtilizationAttempts = @($workerUtilizationAttempts | Where-Object { $_.status -eq 'measured' })
        $workerUtilization = [pscustomobject][ordered]@{
            status = if ($measuredWorkerUtilizationAttempts.Count -eq 0) {
                'not_emitted'
            }
            elseif ($measuredWorkerUtilizationAttempts.Count -eq $workerUtilizationAttempts.Count) {
                'measured'
            }
            else {
                'partial'
            }
            attempts = @($workerUtilizationAttempts)
            counters = @($counterAggregates | Where-Object {
                    $_.stream -eq 'runtime' -and $_.name -eq 'render_framework.scheduler.worker_utilization'
                })
            occupancy_ratio = if ($measuredWorkerUtilizationAttempts.Count -gt 0) {
                Get-RenderExtractStatistics -Values @($measuredWorkerUtilizationAttempts | ForEach-Object { [double]$_.occupancy_ratio })
            }
            else {
                $null
            }
            observed_window_us = if ($measuredWorkerUtilizationAttempts.Count -gt 0) {
                Get-RenderExtractStatistics -Values @($measuredWorkerUtilizationAttempts | ForEach-Object { [double]$_.observed_window_us })
            }
            else {
                $null
            }
            busy_duration_us = if ($measuredWorkerUtilizationAttempts.Count -gt 0) {
                Get-RenderExtractStatistics -Values @($measuredWorkerUtilizationAttempts | ForEach-Object { [double]$_.busy_duration_us })
            }
            else {
                $null
            }
        }
        [pscustomobject][ordered]@{
            logical_id = $scenarioGroup.Name
            scenario_id = $attemptRecords[0].scenario_id
            scenario_version = $attemptRecords[0].scenario_version
            scenario_binding_id = $attemptRecords[0].scenario_binding_id
            product = $attemptRecords[0].product
            measurement_window = $attemptRecords[0].measurement_window
            repeat_count = [int]$attemptRecords[0].repeat_count
            warmup_presented_frame_count = [int]$attemptRecords[0].warmup_presented_frame_count
            measured_presented_frame_count = [int]$attemptRecords[0].measured_presented_frame_count
            target_presented_frame_count = [int]$attemptRecords[0].target_presented_frame_count
            primary_frame_stream = $attemptRecords[0].primary_frame_stream
            primary_frame_name = $attemptRecords[0].primary_frame_name
            cache_contract = $attemptRecords[0].cache_contract
            required_metrics = @($attemptRecords[0].required_metrics)
            budget_contract = $attemptRecords[0].budget_contract
            budget_evaluation = Get-RenderExtractScenarioBudgetEvaluation `
                -BudgetContract $attemptRecords[0].budget_contract `
                -FrameStatistics $frameStatistics
            process_measurement_scope = if ($attemptRecords[0].warmup_presented_frame_count -gt 0) {
                'full-process-lifetime-including-warmup'
            }
            else {
                'full-process-lifetime'
            }
            attempt_count = $attemptRecords.Count
            attempt_processes = @($attemptRecords | ForEach-Object {
                    [ordered]@{
                        attempt = [int]$_.attempt
                        process_id = [Int64]$_.process_id
                    }
                })
            process_elapsed_ms = Get-RenderExtractStatistics -Values @($attemptRecords | ForEach-Object { $_.process_elapsed_ms })
            peak_working_set_bytes = Get-RenderExtractStatistics -Values @($attemptRecords | ForEach-Object { $_.peak_working_set_bytes })
            total_processor_time_ms = Get-RenderExtractStatistics -Values @($attemptRecords | ForEach-Object { $_.total_processor_time_ms })
            frame_duration_us = $frameStatistics
            top_cpu_spans = @($spanAggregates | Select-Object -First 20)
            top_counter_metrics = @($counterAggregates | Select-Object -First 20)
            lock_wait = Get-RenderExtractInstrumentationCoverage `
                -Spans $spanAggregates `
                -Counters $counterAggregates `
                -SpanCategory 'render_framework.wait' `
                -SpanNames @('operation_lock', 'state')
            queue_backpressure = Get-RenderExtractInstrumentationCoverage `
                -Spans $spanAggregates `
                -Counters $counterAggregates `
                -SpanCategory 'render_framework.scheduler' `
                -SpanNames @('wait_previous_submission', 'wait_worker_start', 'wait_pending_submission') `
                -CounterNames @('render_framework.scheduler.pending_depth')
            worker_utilization = $workerUtilization
            scene_extract = Get-RenderExtractInstrumentationCoverage `
                -Spans $spanAggregates `
                -Counters $counterAggregates `
                -SpanCategory @('scene', 'viewport') `
                -SpanNames @(
                    'world_clone',
                    'world_projection_rebuild',
                    'viewport_render_packet',
                    'render_frame_extract',
                    'render_mesh_visit',
                    'render_mesh_sort',
                    'interaction_extract_rebuild',
                    'pointer_fallback_packet_build'
                ) `
                -CounterNames @(
                    'viewport_packet_mesh_count',
                    'viewport_packet_mesh_payload_bytes',
                    'render_frame_mesh_count',
                    'render_frame_mesh_payload_bytes',
                    'interaction_mesh_copy_payload_bytes',
                    'interaction_extract_cache_hit',
                    'interaction_extract_cache_miss'
                )
            app_cadence = Get-RenderExtractInstrumentationCoverage `
                -Spans $spanAggregates `
                -Counters $counterAggregates `
                -CounterNames @(
                    'runtime_entry.frame_pump',
                    'runtime_entry.frame_pump_suppressed',
                    'runtime_entry.runtime_tick',
                    'runtime_entry.redraw_request'
                )
            surface_presentation = Get-RenderExtractInstrumentationCoverage `
                -Spans $spanAggregates `
                -Counters $counterAggregates `
                -CounterNames @(
                    'runtime_entry.native_present',
                    'runtime_entry.fallback_capture_request',
                    'runtime_entry.fallback_rgba_bytes',
                    'runtime_entry.fallback_cpu_present',
                    'runtime_entry.presented_frame',
                    'runtime_entry.explicit_frame_capture_request',
                    'runtime_entry.explicit_frame_capture_rgba_bytes'
                )
            asset_management = Get-RenderExtractInstrumentationCoverage `
                -Spans $spanAggregates `
                -Counters $counterAggregates `
                -SpanCategory 'resource_management' `
                -SpanNames @('project_asset_manager.kind_query', 'project_asset_manager.record_sets') `
                -CounterNames @(
                    'resource_management.scan.instances',
                    'resource_management.scan.matching_rows',
                    'resource_management.scan.rows_emitted',
                    'resource_management.scan.shard_candidate_checks',
                    'resource_management.scan.filtered_rows_skipped'
                ) `
                -RequireAllCounterNames
            asset_management_page = Get-RenderExtractInstrumentationCoverage `
                -Spans $spanAggregates `
                -Counters $counterAggregates `
                -CounterNames @(
                    'resource_management.page.instances',
                    'resource_management.page.matching_rows',
                    'resource_management.page.candidate_rows',
                    'resource_management.page.rows_returned',
                    'resource_management.page.shard_candidate_checks',
                    'resource_management.page.filtered_rows_skipped'
                ) `
                -RequireAllCounterNames
        }
    }

    $report = [pscustomobject][ordered]@{
        schema_version = 5
        generated_at_utc = [DateTimeOffset]::UtcNow.ToString('o')
        source_fingerprint = $sourceFingerprint.ToUpperInvariant()
        profiling_input_manifest_sha256 = $inputManifestSha.ToUpperInvariant()
        build_set_id = $buildSetId.ToUpperInvariant()
        build_set_manifest_sha256 = $buildSetManifestSha.ToUpperInvariant()
        profiling_inputs = [ordered]@{
            runtime = $expectedProfilingInputs['runtime']
            editor = $expectedProfilingInputs['editor']
        }
        machine_manifest = $machineEvidence.manifest
        project = $project
        aggregation = [ordered]@{
            percentile_method = 'upper-nearest-index: ceil((n-1) * percentile / 100), zero-based'
            scope = 'timeline samples use each scenario evidence-declared window; process CPU and working-set samples cover the full product lifetime'
        }
        qualification = [ordered]@{
            status = 'unqualified'
            blocking_reasons = @(
                'product_receipt_not_bound',
                'device_profile_not_bound',
                'independent_comparison_receipt_missing'
            ) + @(if (-not $machineEvidence.all_required_observed) { 'machine_manifest_incomplete' })
        }
        raw_evidence = [ordered]@{
            summary_path = $summaryResolution.DisplayPath
            summary_sha256 = $summarySnapshot.sha256
            machine_manifest = [ordered]@{
                path = $machineEvidence.path
                bytes = $machineEvidence.bytes
                sha256 = $machineEvidence.sha256
            }
            profile_artifacts = @($rawEvidence)
            frame_capture_artifacts = @($frameCaptureEvidence)
            system_trace_artifacts = @($systemTraceEvidence)
        }
        measurement_coverage = [ordered]@{
            cpu_timeline = [ordered]@{ status = 'measured'; source = 'timeline.zrtrace.json frame and span samples' }
            gpu_timing = [ordered]@{ status = 'not_measured'; reason = 'The native timeline export has no calibrated GPU timestamp samples.' }
            system_power = [ordered]@{ status = 'not_measured'; reason = 'This baseline capture does not collect a power trace.' }
            working_set = [ordered]@{ status = 'measured'; source = 'Windows product-process PeakWorkingSet64 per attempt; excludes child processes and GPU memory' }
            process_cpu = [ordered]@{ status = 'measured'; source = 'Windows product-process TotalProcessorTime per attempt; excludes child processes' }
            disk_io = [ordered]@{ status = 'not_measured'; reason = 'This JSON report does not parse a disk I/O trace.' }
            cpu_scheduling = [ordered]@{ status = 'not_measured'; reason = 'WPR ETL files are retained as raw evidence but are not parsed by this JSON report.' }
        }
        scenarios = @($scenarios)
    }
    $report.measurement_coverage.lock_wait = [ordered]@{
        status = if (@($report.scenarios | Where-Object { $_.lock_wait.status -eq 'measured' }).Count -gt 0) { 'measured' } else { 'not_emitted' }
        source = 'named runtime timeline spans and counters'
    }
    $report.measurement_coverage.queue_backpressure = [ordered]@{
        status = if (@($report.scenarios | Where-Object { $_.queue_backpressure.status -eq 'measured' }).Count -gt 0) { 'measured' } else { 'not_emitted' }
        source = 'named runtime timeline spans and counters'
    }
    $pipelinedWorkerUtilizationStatuses = @(
        $report.scenarios |
            Where-Object { $_.logical_id -like 'pipelined-*' } |
            ForEach-Object { $_.worker_utilization.status }
    )
    $report.measurement_coverage.worker_utilization = [ordered]@{
        status = if ($pipelinedWorkerUtilizationStatuses -contains 'partial' -or
            ($pipelinedWorkerUtilizationStatuses -contains 'measured' -and $pipelinedWorkerUtilizationStatuses -contains 'not_emitted')) {
            'partial'
        }
        elseif ($pipelinedWorkerUtilizationStatuses.Count -gt 0 -and
            @($pipelinedWorkerUtilizationStatuses | Where-Object { $_ -ne 'measured' }).Count -eq 0) {
            'measured'
        }
        else {
            'not_emitted'
        }
        source = '0/1 scheduler worker occupancy timeline counter; measures renderer submission occupancy, not CPU utilization'
    }
    $report.measurement_coverage.scene_extract = [ordered]@{
        status = if (@($report.scenarios | Where-Object { $_.scene_extract.status -eq 'measured' }).Count -gt 0) { 'measured' } else { 'not_emitted' }
        source = 'generation-bound scene extract scopes and DTO payload proxy counters'
    }
    $report.measurement_coverage.app_cadence = [ordered]@{
        status = if (@($report.scenarios | Where-Object { $_.app_cadence.status -eq 'measured' }).Count -gt 0) { 'measured' } else { 'not_emitted' }
        source = 'app runtime frame-pump, tick, suppression, and redraw timeline counters'
    }
    $report.measurement_coverage.surface_presentation = [ordered]@{
        status = if (@($report.scenarios | Where-Object { $_.surface_presentation.status -eq 'measured' }).Count -gt 0) { 'measured' } else { 'not_emitted' }
        source = 'app native-present, fallback capture/copy, and explicit capture timeline counters'
    }
    $assetManagementStatuses = @($report.scenarios | ForEach-Object { $_.asset_management.status })
    $report.measurement_coverage.asset_management = [ordered]@{
        status = if ($assetManagementStatuses -contains 'partial' -or ($assetManagementStatuses -contains 'measured' -and $assetManagementStatuses -contains 'not_emitted')) {
            'partial'
        }
        elseif ($assetManagementStatuses -contains 'measured') {
            'measured'
        }
        else {
            'not_emitted'
        }
        source = 'resource-management scan counters and project-asset-management query spans'
    }
    $assetManagementPageStatuses = @($report.scenarios | ForEach-Object { $_.asset_management_page.status })
    $report.measurement_coverage.asset_management_page = [ordered]@{
        status = if ($assetManagementPageStatuses -contains 'partial' -or ($assetManagementPageStatuses -contains 'measured' -and $assetManagementPageStatuses -contains 'not_emitted')) {
            'partial'
        }
        elseif ($assetManagementPageStatuses -contains 'measured') {
            'measured'
        }
        else {
            'not_emitted'
        }
        source = 'resource-management page counters'
    }

    $reportJsonPath = Join-ZirconWindowsPath -Path $evidenceDirectory.OperationalPath -ChildPath 'render-extract-baseline-report.json'
    $reportMarkdownPath = Join-ZirconWindowsPath -Path $evidenceDirectory.OperationalPath -ChildPath 'render-extract-baseline-report.md'
    Write-RenderExtractBaselineReportFileNew `
        -Path $reportJsonPath `
        -Content ($report | ConvertTo-Json -Depth 12)
    Write-RenderExtractBaselineReportFileNew `
        -Path $reportMarkdownPath `
        -Content (ConvertTo-RenderExtractBaselineReportMarkdown -Report $report)
    Write-Host "Render-extract baseline report: $((Resolve-ZirconWindowsPath -Path $reportJsonPath).DisplayPath)"
    return $report
}

if ($env:RENDER_EXTRACT_BASELINE_REPORT_TEST_MODE -ne '1') {
    if ([string]::IsNullOrWhiteSpace($BaselineSummaryPath)) {
        throw '-BaselineSummaryPath is required for render-extract baseline reporting.'
    }
    Write-RenderExtractBaselineReport -BaselineSummaryPath $BaselineSummaryPath | Out-Null
}
