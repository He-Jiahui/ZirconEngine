[CmdletBinding()]
param(
    [string]$ProfilingInputManifestPath,
    [string]$ProjectRoot,
    [string]$OutputDirectory,
    [ValidateRange(3, 20)]
    [int]$RepeatCount = 5,
    [ValidateRange(0, 1000000)]
    [int]$WarmupPresentedFrameCount = 120,
    [ValidateRange(1, 1000000)]
    [int]$MeasuredPresentedFrameCount = 600,
    [ValidateRange(1, 600)]
    [int]$TimeoutSeconds = 90,
    [ValidateRange(1, 1048576)]
    [int]$MaxProfileFrames = 4096,
    [ValidateRange(1, 1048576)]
    [int]$MaxProfileSpans = 65536,
    [ValidateRange(1, 1048576)]
    [int]$MaxProfileCounters = 65536,
    [switch]$UseWpr,
    [switch]$UseWprHeap
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Import-Module (Join-Path $PSScriptRoot 'MvpProductInputManifest.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpArtifactStoragePolicy.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $repoRoot 'tools\maintenance\WindowsPathResolver.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'RenderExtractFrozenInput.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'RenderExtractPerformanceScenario.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'RenderExtractProcessJob.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'RenderExtractWprCapture.psm1') -Force -ErrorAction Stop
. (Join-Path $repoRoot 'tools\analysis\profiling\shared\profile-capture-paths.ps1')
. (Join-Path $repoRoot 'tools\analysis\profiling\shared\performance-machine-manifest.ps1')

if ([string]::IsNullOrWhiteSpace($OutputDirectory)) {
    $OutputDirectory = New-MvpArtifactStoragePath -NamespaceId 'render-extract-baselines'
}

function Assert-RenderExtractBaselineOutputDirectory {
    param(
        [Parameter(Mandatory)][string]$Path
    )

    $storage = Resolve-MvpArtifactStoragePath `
        -Path $Path `
        -NamespaceId 'render-extract-baselines'
    $directoryExists = [IO.Directory]::Exists($storage.operation_path)
    if ($directoryExists -and
        [IO.Directory]::EnumerateFileSystemEntries($storage.operation_path).GetEnumerator().MoveNext()) {
        throw "-OutputDirectory must be empty to preserve render-extract evidence: $($storage.display_path)"
    }
    return $storage.operation_path
}

function Assert-RenderExtractBaselineProjectDirectory {
    param([Parameter(Mandatory)][string]$Path)

    $pathResolution = Resolve-ZirconWindowsPath -Path $Path
    $templateRoot = Resolve-ZirconWindowsPath -Path (Join-Path $repoRoot 'templates\projects')
    $templatePrefix = $templateRoot.OperationalPath.TrimEnd('\', '/') + [IO.Path]::DirectorySeparatorChar
    if ($pathResolution.OperationalPath.Equals($templateRoot.OperationalPath, [StringComparison]::OrdinalIgnoreCase) -or
        $pathResolution.OperationalPath.StartsWith($templatePrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "-ProjectRoot must name a created project or example project, not a repository source template: $($pathResolution.DisplayPath)"
    }

    $storage = Resolve-MvpArtifactStorageRootPath `
        -Path $pathResolution.DisplayPath `
        -CapabilityClass 'windows-local-artifact'
    $projectRoot = [pscustomobject]@{
        OperationalPath = $storage.operation_path
        DisplayPath = $storage.display_path
        StoragePolicy = $storage
    }
    if (-not [IO.Directory]::Exists($projectRoot.OperationalPath)) {
        throw "-ProjectRoot does not exist or is not a directory: $($projectRoot.DisplayPath)"
    }
    if (-not [IO.File]::Exists((Join-ZirconWindowsPath -Path $projectRoot.OperationalPath -ChildPath 'zircon-project.toml'))) {
        throw "-ProjectRoot is missing zircon-project.toml: $($projectRoot.DisplayPath)"
    }
    return $projectRoot
}

function Get-RenderExtractScaleProjectMetadata {
    param(
        [Parameter(Mandatory)]$ProjectRoot,
        [Parameter(Mandatory)][string]$ExpectedBuildSetId
    )

    $metadataPath = Join-ZirconWindowsPath `
        -Path $ProjectRoot.OperationalPath `
        -ChildPath 'render-extract-scale-project.json'
    if (-not [IO.File]::Exists($metadataPath)) {
        return $null
    }
    try {
        $metadata = [IO.File]::ReadAllText($metadataPath) | ConvertFrom-Json -ErrorAction Stop
    }
    catch {
        throw "Generated render-extract scale metadata is not valid JSON: ${metadataPath}: $($_.Exception.Message)"
    }
    if ($null -eq $metadata -or [int]$metadata.schema_version -ne 2) {
        throw "Generated render-extract scale metadata has an unsupported schema_version: $metadataPath"
    }
    $projectFingerprint = [string]$metadata.source_fingerprint
    $projectBuildSetId = [string]$metadata.build_set_id
    if ($projectFingerprint -notmatch '^[0-9A-F]{64}$' -or
        $projectBuildSetId -notmatch '^[0-9A-F]{64}$' -or
        -not $projectFingerprint.Equals($projectBuildSetId, [StringComparison]::Ordinal)) {
        throw "Generated render-extract scale metadata has an invalid BuildSet identity: $metadataPath"
    }
    if (-not $projectBuildSetId.Equals($ExpectedBuildSetId, [StringComparison]::Ordinal)) {
        throw 'Generated render-extract scale project belongs to a different BuildSet. Regenerate it before capture.'
    }
    return $metadata
}

function New-RenderExtractBaselineOutputSessionLease {
    param([Parameter(Mandatory)][string]$Path)

    [IO.Directory]::CreateDirectory($Path) | Out-Null
    $leasePath = Join-ZirconWindowsPath -Path $Path -ChildPath '.zircon-render-extract-baseline.active'
    try {
        $leaseStream = [IO.FileStream]::new(
            $leasePath,
            [IO.FileMode]::CreateNew,
            [IO.FileAccess]::ReadWrite,
            [IO.FileShare]::None,
            1,
            [IO.FileOptions]::DeleteOnClose
        )
    }
    catch [IO.IOException] {
        throw "Render-extract baseline output session is already active or changed after preflight: $Path"
    }

    try {
        $entries = [IO.Directory]::EnumerateFileSystemEntries($Path).GetEnumerator()
        try {
            while ($entries.MoveNext()) {
                if (-not $entries.Current.Equals($leasePath, [StringComparison]::OrdinalIgnoreCase)) {
                    throw "-OutputDirectory must remain empty until render-extract capture reserves it: $Path"
                }
            }
        }
        finally {
            $entries.Dispose()
        }
        return [pscustomobject]@{
            Stream = $leaseStream
            Path = $leasePath
            InvocationId = [guid]::NewGuid().ToString('N')
        }
    }
    catch {
        $leaseStream.Dispose()
        throw
    }
}

function Write-RenderExtractBaselineTextFileNew {
    param(
        [Parameter(Mandatory)][string]$Path,
            [Parameter(Mandatory)][AllowEmptyString()][string]$Content
    )

    $stream = $null
    $writer = $null
    try {
        try {
            $stream = [IO.FileStream]::new(
                $Path,
                [IO.FileMode]::CreateNew,
                [IO.FileAccess]::Write,
                [IO.FileShare]::None
            )
        }
        catch [IO.IOException] {
            throw "Refusing to overwrite existing render-extract evidence: $Path"
        }
        $writer = [IO.StreamWriter]::new($stream, [Text.UTF8Encoding]::new($false))
        $stream = $null
        $writer.Write($Content)
    }
    finally {
        if ($null -ne $writer) {
            $writer.Dispose()
        }
        elseif ($null -ne $stream) {
            $stream.Dispose()
        }
    }
}

function Get-RenderExtractBaselineProductArguments {
    param(
        [Parameter(Mandatory)][ValidateSet('runtime', 'editor')][string]$Product,
        [Parameter(Mandatory)][string]$RuntimeProfile
    )

    if ($Product -eq 'editor') {
        return @('--project', '.')
    }
    return @('--project', '.', '--runtime-session-profile', $RuntimeProfile)
}

function ConvertTo-RenderExtractProcessArgument {
    param([Parameter(Mandatory)][string]$Value)

    if ($Value -notmatch '[\s"]') {
        return $Value
    }
    return '"' + $Value.Replace('"', '\"') + '"'
}

function Stop-RenderExtractBaselineProcessTree {
    param(
        [Parameter(Mandatory)][Diagnostics.Process]$Process,
        [Parameter(Mandatory)][string]$SessionId
    )

    if ($Process.HasExited) {
        return
    }
    & taskkill.exe '/PID' $Process.Id '/T' '/F' 2>$null | Out-Null
    if ($Process.WaitForExit(5000)) {
        return
    }
    try {
        $Process.Kill()
    }
    catch {
        throw "Render-extract run '$SessionId' process tree could not be terminated: $($_.Exception.Message)"
    }
    if (-not $Process.WaitForExit(5000)) {
        throw "Render-extract run '$SessionId' process remained alive after forced termination."
    }
}

function Invoke-RenderExtractBaselineProcess {
    param(
        [Parameter(Mandatory)]$ProfilingInput,
        [Parameter(Mandatory)]$ProjectRoot,
        [Parameter(Mandatory)]$Run,
        [Parameter(Mandatory)][int]$Attempt,
        [Parameter(Mandatory)][string]$InvocationId,
        [Parameter(Mandatory)][string]$OutputDirectory,
        [Parameter(Mandatory)][int]$TimeoutSeconds,
        [Parameter(Mandatory)][int]$MaxProfileFrames,
        [Parameter(Mandatory)][int]$MaxProfileSpans,
        [Parameter(Mandatory)][int]$MaxProfileCounters,
        [AllowNull()]
        [string]$WprProfile
    )

    $sessionId = "$($Run.logical_id)-$Attempt"
    if (-not [string]::IsNullOrEmpty($WprProfile) -and $WprProfile -notin @('CPU', 'Heap')) {
        throw "Unsupported render-extract WPR profile '$WprProfile'."
    }
    $useWprCapture = -not [string]::IsNullOrEmpty($WprProfile)
    $productInput = $ProfilingInput.($Run.product)
    if ($null -eq $productInput) {
        throw "Render-extract run '$sessionId' names unsupported product '$($Run.product)'."
    }
    $profilesRoot = Join-ZirconWindowsPath `
        -Path (Join-ZirconWindowsPath -Path $OutputDirectory -ChildPath 'profiles') `
        -ChildPath $InvocationId
    $logsRoot = Join-ZirconWindowsPath `
        -Path (Join-ZirconWindowsPath -Path $OutputDirectory -ChildPath 'logs') `
        -ChildPath $InvocationId
    $capturesRoot = Join-ZirconWindowsPath `
        -Path (Join-ZirconWindowsPath -Path $OutputDirectory -ChildPath 'captures') `
        -ChildPath $InvocationId
    $tracesRoot = Join-ZirconWindowsPath `
        -Path (Join-ZirconWindowsPath -Path $OutputDirectory -ChildPath 'traces') `
        -ChildPath $InvocationId
    [IO.Directory]::CreateDirectory($profilesRoot) | Out-Null
    [IO.Directory]::CreateDirectory($logsRoot) | Out-Null
    [IO.Directory]::CreateDirectory($capturesRoot) | Out-Null
    if ($useWprCapture) {
        [IO.Directory]::CreateDirectory($tracesRoot) | Out-Null
    }
    $stdoutPath = Join-ZirconWindowsPath -Path $logsRoot -ChildPath "$sessionId.stdout.log"
    $stderrPath = Join-ZirconWindowsPath -Path $logsRoot -ChildPath "$sessionId.stderr.log"
    $capturePath = Join-ZirconWindowsPath -Path $capturesRoot -ChildPath "$sessionId.png"
    $systemTracePath = if ($useWprCapture) {
        $traceName = if ($WprProfile -eq 'Heap') { "$sessionId.heap.etl" } else { "$sessionId.etl" }
        Join-ZirconWindowsPath -Path $tracesRoot -ChildPath $traceName
    }
    else {
        $null
    }
    # WPR file mode must keep recorder buffers inside this E-drive evidence session.
    $wprTemporaryDirectory = if ($useWprCapture) {
        Join-ZirconWindowsPath -Path $tracesRoot -ChildPath "$sessionId.wpr-temp"
    }
    else {
        $null
    }
    $systemTraceAnalysisPath = if ($WprProfile -eq 'Heap') {
        Join-ZirconWindowsPath -Path $tracesRoot -ChildPath "$sessionId.heap-product-allocation-stacks.txt"
    }
    elseif ($WprProfile -eq 'CPU') {
        Join-ZirconWindowsPath -Path $tracesRoot -ChildPath "$sessionId.cpu-product-sampled-stacks.txt"
    }
    else {
        $null
    }
    $systemTraceReceiptPath = if ($useWprCapture) {
        Join-ZirconWindowsPath `
            -Path $tracesRoot `
            -ChildPath "$sessionId.$($WprProfile.ToLowerInvariant())-wpr-capture.json"
    }
    else {
        $null
    }
    $diagnosticRoot = Join-ZirconWindowsPath -Path $logsRoot -ChildPath "$sessionId.diagnostics"

    $startInfo = [Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $productInput.executable_path
    $startInfo.WorkingDirectory = $ProjectRoot.OperationalPath
    $startInfo.UseShellExecute = $false
    $startInfo.CreateNoWindow = $true
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    foreach ($name in @(
            'ZIRCON_RUNTIME_EXIT_AFTER_FIRST_FRAME',
            'ZIRCON_RUNTIME_EXIT_AFTER_PRESENTED_FRAMES',
            'ZIRCON_RUNTIME_CAPTURE_FRAME_PNG',
            'ZIRCON_EDITOR_EXIT_AFTER_FIRST_FRAME',
            'ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG',
            'ZIRCON_PROFILE_CAPTURE',
            'ZIRCON_PROFILE_SESSION',
            'ZIRCON_PROFILE_OUTPUT_ROOT',
            'ZIRCON_RUNTIME_LIBRARY',
            'ZIRCON_ASSET_ROOT'
        )) {
        $startInfo.EnvironmentVariables.Remove($name)
    }
    $environment = @{
        ZIRCON_RUNTIME_LIBRARY = 'zircon_runtime.dll'
        ZIRCON_ASSET_ROOT = 'assets'
        ZIRCON_LOG_ROOT = (Resolve-ZirconWindowsPath -Path $diagnosticRoot).DisplayPath
        ZIRCON_LOG_FILTER = 'log'
        ZIRCON_PROFILE_CAPTURE = '1'
        ZIRCON_PROFILE_SESSION = $sessionId
        ZIRCON_PROFILE_OUTPUT_ROOT = (Resolve-ZirconWindowsPath -Path $profilesRoot).DisplayPath
        ZIRCON_PROFILE_MAX_FRAMES = [string]$MaxProfileFrames
        ZIRCON_PROFILE_MAX_SPANS = [string]$MaxProfileSpans
        ZIRCON_PROFILE_MAX_COUNTERS = [string]$MaxProfileCounters
    }
    if ($Run.product -eq 'editor') {
        $environment.ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG = (Resolve-ZirconWindowsPath -Path $capturePath).DisplayPath
        $environment.ZIRCON_EDITOR_EXIT_AFTER_FIRST_FRAME = '1'
    }
    else {
        $environment.ZIRCON_RUNTIME_CAPTURE_FRAME_PNG = (Resolve-ZirconWindowsPath -Path $capturePath).DisplayPath
        if ($Run.exit_after_first_frame) {
            $environment.ZIRCON_RUNTIME_EXIT_AFTER_FIRST_FRAME = '1'
        }
        else {
            $environment.ZIRCON_RUNTIME_EXIT_AFTER_PRESENTED_FRAMES = [string]$Run.presented_frame_count
        }
    }
    foreach ($name in $environment.Keys) {
        $startInfo.EnvironmentVariables[$name] = [string]$environment[$name]
    }
    $startInfo.Arguments = ((Get-RenderExtractBaselineProductArguments `
            -Product $Run.product `
            -RuntimeProfile $Run.runtime_profile | ForEach-Object {
                ConvertTo-RenderExtractProcessArgument -Value $_
            }) -join ' ')

    $process = $null
    $assignedProcess = $null
    $startedAtUtc = $null
    $endedAtUtc = $null
    $processStopwatch = [Diagnostics.Stopwatch]::new()
    $processElapsedMs = $null
    $wprCapture = $null
    $wprReceipt = $null
    $primaryFailure = $null
    $processCleanupFailure = $null
    $wprStopFailure = $null
    $processStarted = $false
    $processJob = $null
    $actualProductHashes = $null
    $peakWorkingSetBytes = $null
    $totalProcessorTimeMs = $null
    $processId = $null
    $productStartedAtUtc = $null
    $productEndedAtUtc = $null
    try {
        $actualProductHashes = Assert-RenderExtractFrozenProductInput `
            -ProductInput $productInput `
            -Product $Run.product
        $processJob = New-RenderExtractBaselineProcessJob
        if ($useWprCapture) {
            [IO.Directory]::CreateDirectory($wprTemporaryDirectory) | Out-Null
            $wprCapture = Start-RenderExtractWprCapture `
                -TemporaryDirectory $wprTemporaryDirectory `
                -Profile $WprProfile
        }
        $startedAtUtc = [DateTimeOffset]::UtcNow.ToString('o')
        $processStopwatch.Start()
        $assignedProcess = Start-RenderExtractBaselineAssignedProcess -Job $processJob -StartInfo $startInfo
        $process = $assignedProcess.Process
        $processId = [Int64]$process.Id
        if ($useWprCapture) {
            $productStartedAtUtc = ([DateTimeOffset]$process.StartTime.ToUniversalTime()).ToString('o')
        }
        $processStarted = $true
        $stdoutTask = $assignedProcess.StandardOutput.ReadToEndAsync()
        $stderrTask = $assignedProcess.StandardError.ReadToEndAsync()
        if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
            throw "Render-extract run '$sessionId' timed out after $TimeoutSeconds seconds."
        }
        $processStopwatch.Stop()
        $processElapsedMs = $processStopwatch.Elapsed.TotalMilliseconds
        $endedAtUtc = [DateTimeOffset]::UtcNow.ToString('o')
        $process.WaitForExit()
        if ($useWprCapture) {
            $productEndedAtUtc = ([DateTimeOffset]$process.ExitTime.ToUniversalTime()).ToString('o')
        }
        if (-not [Threading.Tasks.Task]::WaitAll(@($stdoutTask, $stderrTask), 5000)) {
            throw "Render-extract run '$sessionId' did not drain process output."
        }
        Write-RenderExtractBaselineTextFileNew `
            -Path $stdoutPath `
            -Content $stdoutTask.GetAwaiter().GetResult()
        Write-RenderExtractBaselineTextFileNew `
            -Path $stderrPath `
            -Content $stderrTask.GetAwaiter().GetResult()
        $exitCode = $assignedProcess.TryGetExitCode()
        if ($null -eq $exitCode) {
            throw "Render-extract run '$sessionId' exited but its native process handle did not report an exit code. See $stdoutPath and $stderrPath."
        }
        if ($exitCode -ne 0) {
            throw "Render-extract run '$sessionId' exited with code $exitCode. See $stdoutPath and $stderrPath."
        }
    }
    catch {
        $primaryFailure = $_
    }
    finally {
        if ($processStopwatch.IsRunning) {
            $processStopwatch.Stop()
            $processElapsedMs = $processStopwatch.Elapsed.TotalMilliseconds
        }
        if ($null -eq $endedAtUtc) {
            $endedAtUtc = [DateTimeOffset]::UtcNow.ToString('o')
        }
        # Snapshot root-only metrics while its process handle is still queryable. They are not
        # published until the job-empty barrier below has completed.
        $processExited = $processStarted -and $null -ne $process -and $process.HasExited
        if (-not $processExited) {
            $exitCode = -1
        }
        if ($processExited) {
            # Windows can retire process accounting before this managed handle is queried. Root
            # process statistics are optional evidence; a missing value must not bypass cleanup.
            try {
                $peakWorkingSetBytes = [Int64]$process.PeakWorkingSet64
            }
            catch {}
            try {
                $totalProcessorTime = $process.TotalProcessorTime
                if ($null -ne $totalProcessorTime) {
                    $totalProcessorTimeMs = ([TimeSpan]$totalProcessorTime).TotalMilliseconds
                }
            }
            catch {}
        }
        if ($processStarted) {
            try {
                if ($null -ne $processJob) {
                    Stop-RenderExtractBaselineProcessJob -Job $processJob -SessionId $sessionId
                }
                elseif (-not $process.HasExited) {
                    Stop-RenderExtractBaselineProcessTree -Process $process -SessionId $sessionId
                }
            }
            catch {
                $processCleanupFailure = $_
            }
        }
        if ($null -ne $processJob) {
            try {
                $processJob.Dispose()
            }
            catch {
                if ($null -eq $processCleanupFailure) {
                    $processCleanupFailure = $_
                }
            }
        }
        if ($useWprCapture -and $processStarted -and $null -ne $process -and $process.HasExited) {
            if ($null -eq $productStartedAtUtc) {
                try {
                    $productStartedAtUtc = ([DateTimeOffset]$process.StartTime.ToUniversalTime()).ToString('o')
                }
                catch {}
            }
            if ($null -eq $productEndedAtUtc) {
                try {
                    $productEndedAtUtc = ([DateTimeOffset]$process.ExitTime.ToUniversalTime()).ToString('o')
                }
                catch {}
            }
        }
        if ($null -ne $assignedProcess) {
            $assignedProcess.Dispose()
        }
        if ($null -ne $wprCapture) {
            try {
                $wprStopParameters = @{
                    Capture = $wprCapture
                    TracePath = $systemTracePath
                    AnalysisPath = $systemTraceAnalysisPath
                    ReceiptPath = $systemTraceReceiptPath
                }
                if ($null -ne $processId -and
                    $null -ne $productStartedAtUtc -and
                    $null -ne $productEndedAtUtc) {
                    $wprStopParameters.ProcessId = $processId
                    $wprStopParameters.ProcessStartedAtUtc = $productStartedAtUtc
                    $wprStopParameters.ProcessEndedAtUtc = $productEndedAtUtc
                }
                $wprReceipt = Stop-RenderExtractWprCapture @wprStopParameters
            }
            catch {
                $wprStopFailure = $_
            }
        }
    }

    # The product failure is the primary diagnostic when trace cleanup also fails.
    if ($null -ne $primaryFailure) {
        $cleanupDetails = ''
        if ($null -ne $processCleanupFailure) {
            $cleanupDetails += " Process cleanup also failed: $($processCleanupFailure.Exception.Message)"
        }
        if ($null -ne $wprStopFailure) {
            $cleanupDetails += " WPR cleanup also failed: $($wprStopFailure.Exception.Message)"
        }
        if (-not [string]::IsNullOrEmpty($cleanupDetails)) {
            throw "Render-extract run '$sessionId' failed: $($primaryFailure.Exception.Message)$cleanupDetails"
        }
        throw $primaryFailure
    }
    if ($null -ne $processCleanupFailure) {
        throw $processCleanupFailure
    }
    if ($null -ne $wprStopFailure) {
        throw $wprStopFailure
    }

    $profileBasename = ConvertTo-ZirconProfileSessionBasename -SessionId $sessionId
    $profileDirectory = Join-ZirconWindowsPath -Path $profilesRoot -ChildPath $profileBasename
    foreach ($name in @('timeline.zrtrace.json', 'hotspots.json', 'counter_hotspots.json', 'summary.md')) {
        $path = Join-ZirconWindowsPath -Path $profileDirectory -ChildPath $name
        if (-not [IO.File]::Exists($path) -or [IO.FileInfo]::new($path).Length -le 0) {
            throw "Render-extract run '$sessionId' did not export nonempty $name."
        }
    }
    if (-not [IO.File]::Exists($capturePath) -or [IO.FileInfo]::new($capturePath).Length -le 0) {
        throw "Render-extract run '$sessionId' did not produce a nonempty $($Run.product) PNG."
    }
    if ($useWprCapture -and
        ($null -eq $wprReceipt -or $null -eq $wprReceipt.analysis)) {
        throw "Render-extract run '$sessionId' did not produce product-attributed $WprProfile WPR analysis."
    }
    return [ordered]@{
        logical_id = $Run.logical_id
        scenario_id = $Run.scenario_id
        scenario_version = $Run.scenario_version
        scenario_binding_id = $Run.scenario_binding_id
        product = $Run.product
        attempt = $Attempt
        invocation_id = $InvocationId
        runtime_profile = $Run.runtime_profile
        measurement_window = $Run.measurement_window
        repeat_count = $Run.repeat_count
        warmup_presented_frame_count = $Run.warmup_presented_frame_count
        measured_presented_frame_count = $Run.measured_presented_frame_count
        target_presented_frame_count = $Run.target_presented_frame_count
        cache_contract = $Run.cache_contract
        required_metrics = @($Run.required_metrics)
        budget_contract = $Run.budget_contract
        exit_code = $exitCode
        peak_working_set_bytes = $peakWorkingSetBytes
        total_processor_time_ms = $totalProcessorTimeMs
        process_id = $processId
        process_elapsed_ms = $processElapsedMs
        started_at_utc = $startedAtUtc
        ended_at_utc = $endedAtUtc
        stdout = (Resolve-ZirconWindowsPath -Path $stdoutPath).DisplayPath
        stderr = (Resolve-ZirconWindowsPath -Path $stderrPath).DisplayPath
        profile_directory = (Resolve-ZirconWindowsPath -Path $profileDirectory).DisplayPath
        frame_capture_png = (Resolve-ZirconWindowsPath -Path $capturePath).DisplayPath
        system_trace_profile = if ($null -ne $wprReceipt) { $WprProfile.ToLowerInvariant() } else { $null }
        system_trace_etl = if ($null -ne $wprReceipt) { (Resolve-ZirconWindowsPath -Path $systemTracePath).DisplayPath } else { $null }
        system_trace_analysis = if ($null -ne $wprReceipt) { (Resolve-ZirconWindowsPath -Path $systemTraceAnalysisPath).DisplayPath } else { $null }
        system_trace_receipt = if ($null -ne $wprReceipt) { (Resolve-ZirconWindowsPath -Path $systemTraceReceiptPath).DisplayPath } else { $null }
        profiling_input = [ordered]@{
            manifest_sha256 = $ProfilingInput.manifest_sha256
            build_set_id = $ProfilingInput.build_set_id
            build_set_manifest_sha256 = $ProfilingInput.build_set_manifest_sha256
            executable_sha256 = $actualProductHashes.executable_sha256
            library_sha256 = $actualProductHashes.library_sha256
            asset_manifest_sha256 = $actualProductHashes.asset_manifest_sha256
            asset_file_count = $actualProductHashes.asset_file_count
            asset_bytes = $actualProductHashes.asset_bytes
        }
    }
}

function Invoke-RenderExtractBaselineCapture {
    param(
        [Parameter(Mandatory)][string]$ManifestPath,
        [Parameter(Mandatory)][string]$ProjectPath,
        [Parameter(Mandatory)][string]$EvidenceOutputDirectory
    )

    $wprProfile = Resolve-RenderExtractWprProfile -UseWpr:$UseWpr -UseWprHeap:$UseWprHeap
    $resolvedOutputDirectory = Assert-RenderExtractBaselineOutputDirectory -Path $EvidenceOutputDirectory
    $projectRoot = Assert-RenderExtractBaselineProjectDirectory -Path $ProjectPath

    $profilingInput = Resolve-RenderExtractProfilingInput `
        -ManifestPath $ManifestPath
    $sourceFingerprint = $profilingInput.source_fingerprint
    $scaleProject = Get-RenderExtractScaleProjectMetadata `
        -ProjectRoot $projectRoot `
        -ExpectedBuildSetId $profilingInput.build_set_id

    $runPlan = @(Get-RenderExtractBaselineRunPlan `
            -RepeatCount $RepeatCount `
            -WarmupPresentedFrameCount $WarmupPresentedFrameCount `
            -MeasuredPresentedFrameCount $MeasuredPresentedFrameCount)
    $sessionLease = New-RenderExtractBaselineOutputSessionLease -Path $resolvedOutputDirectory
    $runs = [System.Collections.Generic.List[object]]::new()
    try {
        $machineManifestPath = Join-ZirconWindowsPath `
            -Path $resolvedOutputDirectory `
            -ChildPath 'machine-manifest.json'
        $machineManifest = New-ZirconPerformanceMachineManifest
        Write-RenderExtractBaselineTextFileNew `
            -Path $machineManifestPath `
            -Content ($machineManifest | ConvertTo-Json -Depth 10)
        $machineEvidence = [ordered]@{
            path = (Resolve-ZirconWindowsPath -Path $machineManifestPath).DisplayPath
            sha256 = Get-MvpProductInputFileSha256 -Path $machineManifestPath
        }
        $frozenProfilingInput = New-RenderExtractFrozenProfilingInput `
            -ProfilingInput $profilingInput `
            -EngineAssetRoots @(
                (Join-ZirconWindowsPath -Path $repoRoot -ChildPath 'zircon_editor\assets'),
                (Join-ZirconWindowsPath -Path $repoRoot -ChildPath 'zircon_runtime\assets')
            ) `
            -OutputDirectory $resolvedOutputDirectory `
            -InvocationId $sessionLease.InvocationId
        foreach ($run in $runPlan) {
            for ($attempt = 1; $attempt -le $run.repeat_count; $attempt++) {
                $currentInput = Resolve-RenderExtractProfilingInput `
                    -ManifestPath $ManifestPath
                Assert-RenderExtractProfilingInputIdentity `
                    -Expected $profilingInput `
                    -Actual $currentInput
                $runs.Add((Invoke-RenderExtractBaselineProcess `
                        -ProfilingInput $frozenProfilingInput `
                        -ProjectRoot $projectRoot `
                        -Run $run `
                        -Attempt $attempt `
                        -InvocationId $sessionLease.InvocationId `
                        -OutputDirectory $resolvedOutputDirectory `
                        -TimeoutSeconds $TimeoutSeconds `
                        -MaxProfileFrames $MaxProfileFrames `
                        -MaxProfileSpans $MaxProfileSpans `
                        -MaxProfileCounters $MaxProfileCounters `
                        -WprProfile $wprProfile)) | Out-Null
            }
        }

        $finalInput = Resolve-RenderExtractProfilingInput -ManifestPath $ManifestPath
        Assert-RenderExtractProfilingInputIdentity `
            -Expected $profilingInput `
            -Actual $finalInput

        $summaryPath = Join-ZirconWindowsPath -Path $resolvedOutputDirectory -ChildPath 'render-extract-baseline.json'
        $summary = [ordered]@{
            schema_version = 5
            generated_at_utc = [DateTimeOffset]::UtcNow.ToString('o')
            source_fingerprint = $sourceFingerprint
            profiling_input_manifest_sha256 = $frozenProfilingInput.manifest_sha256
            build_set_id = $frozenProfilingInput.build_set_id
            build_set_manifest_sha256 = $frozenProfilingInput.build_set_manifest_sha256
            invocation_id = $sessionLease.InvocationId
            machine_manifest = $machineEvidence
            project = [ordered]@{
                runtime_argument = '.'
                physical_identity = $projectRoot.DisplayPath
                scale_project = if ($null -eq $scaleProject) {
                    $null
                }
                else {
                    [ordered]@{
                        primitive_count = [int]$scaleProject.primitive_count
                        scene_virtual_path = [string]$scaleProject.scene_virtual_path
                    }
                }
            }
            runs = @($runs)
        }
        Write-RenderExtractBaselineTextFileNew `
            -Path $summaryPath `
            -Content ($summary | ConvertTo-Json -Depth 10)
        & (Join-Path $PSScriptRoot 'Write-RenderExtractBaselineReport.ps1') -BaselineSummaryPath $summaryPath | Out-Null
        Write-Host "Render-extract baseline summary: $((Resolve-ZirconWindowsPath -Path $summaryPath).DisplayPath)"
        return $summary
    }
    finally {
        $sessionLease.Stream.Dispose()
    }
}

if ($env:RENDER_EXTRACT_BASELINE_TEST_MODE -ne '1') {
    if ([string]::IsNullOrWhiteSpace($ProfilingInputManifestPath)) {
        throw '-ProfilingInputManifestPath is required for render-extract baseline capture.'
    }
    if ([string]::IsNullOrWhiteSpace($ProjectRoot)) {
        throw '-ProjectRoot is required for render-extract baseline capture.'
    }
    Invoke-RenderExtractBaselineCapture `
        -ManifestPath $ProfilingInputManifestPath `
        -ProjectPath $ProjectRoot `
        -EvidenceOutputDirectory $OutputDirectory
}
