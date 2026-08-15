[CmdletBinding()]
param(
    [string]$ProfilingInputManifestPath,
    [string]$ProjectRoot,
    [string]$OutputDirectory = (Join-Path 'E:\ZirconBuilds\mvp-perf' ([guid]::NewGuid().ToString('N'))),
    [ValidateRange(3, 20)]
    [int]$RepeatCount = 3,
    [ValidateRange(0, 1000000)]
    [int]$WarmupPresentedFrameCount = 60,
    [ValidateRange(1, 1000000)]
    [int]$MeasuredPresentedFrameCount = 300,
    [ValidateRange(1, 600)]
    [int]$TimeoutSeconds = 90,
    [ValidateRange(1, 1048576)]
    [int]$MaxProfileFrames = 4096,
    [ValidateRange(1, 1048576)]
    [int]$MaxProfileSpans = 65536,
    [ValidateRange(1, 1048576)]
    [int]$MaxProfileCounters = 65536,
    [switch]$UseWpr
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Import-Module (Join-Path $PSScriptRoot 'MvpProductInputManifest.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $repoRoot 'tools\WindowsPathResolver.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'RenderExtractFrozenInput.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'RenderExtractProcessJob.psm1') -Force -ErrorAction Stop

function Get-RenderExtractBaselineRunPlan {
    param(
        [Parameter(Mandatory)][int]$RepeatCount,
        [Parameter(Mandatory)][int]$WarmupPresentedFrameCount,
        [Parameter(Mandatory)][int]$MeasuredPresentedFrameCount
    )

    $targetPresentedFrameCount = $WarmupPresentedFrameCount + $MeasuredPresentedFrameCount
    return @(
        [pscustomobject]@{
            logical_id = 'pipelined-first-frame'
            product = 'runtime'
            runtime_profile = 'runtime-pipelined'
            exit_after_first_frame = $true
            presented_frame_count = $null
            warmup_presented_frame_count = 0
            measured_presented_frame_count = 1
            target_presented_frame_count = 1
            repeat_count = $RepeatCount
        },
        [pscustomobject]@{
            logical_id = 'pipelined-steady'
            product = 'runtime'
            runtime_profile = 'runtime-pipelined'
            exit_after_first_frame = $false
            presented_frame_count = $targetPresentedFrameCount
            warmup_presented_frame_count = $WarmupPresentedFrameCount
            measured_presented_frame_count = $MeasuredPresentedFrameCount
            target_presented_frame_count = $targetPresentedFrameCount
            repeat_count = $RepeatCount
        },
        [pscustomobject]@{
            logical_id = 'synchronous-steady'
            product = 'runtime'
            runtime_profile = 'runtime'
            exit_after_first_frame = $false
            presented_frame_count = $targetPresentedFrameCount
            warmup_presented_frame_count = $WarmupPresentedFrameCount
            measured_presented_frame_count = $MeasuredPresentedFrameCount
            target_presented_frame_count = $targetPresentedFrameCount
            repeat_count = $RepeatCount
        },
        [pscustomobject]@{
            logical_id = 'editor-first-frame'
            product = 'editor'
            runtime_profile = 'editor'
            exit_after_first_frame = $true
            presented_frame_count = $null
            warmup_presented_frame_count = 0
            measured_presented_frame_count = 1
            target_presented_frame_count = 1
            repeat_count = $RepeatCount
        }
    )
}

function Assert-RenderExtractBaselineOutputDirectory {
    param(
        [Parameter(Mandatory)][string]$Path
    )

    $resolution = Resolve-ZirconWindowsPath -Path $Path
    if ($resolution.DisplayPath -notmatch '^E:\\ZirconBuilds\\mvp-perf\\(?:[A-Za-z0-9][A-Za-z0-9._-]*)(?:\\|$)') {
        throw "-OutputDirectory render-extract baseline evidence must resolve under E:\ZirconBuilds\mvp-perf\<session>: $($resolution.DisplayPath)"
    }
    $directoryExists = [IO.Directory]::Exists($resolution.OperationalPath)
    if ($directoryExists -and
        [IO.Directory]::EnumerateFileSystemEntries($resolution.OperationalPath).GetEnumerator().MoveNext()) {
        throw "-OutputDirectory must be empty to preserve render-extract evidence: $($resolution.DisplayPath)"
    }
    return $resolution.OperationalPath
}

function Assert-RenderExtractBaselineProjectDirectory {
    param([Parameter(Mandatory)][string]$Path)

    $projectRoot = Resolve-ZirconWindowsPath -Path $Path
    # A product run can populate project-local .zircon state, so captures stay off C:.
    if ($projectRoot.DisplayPath -notmatch '^[D-F]:\\') {
        throw "-ProjectRoot must resolve on an approved D:, E:, or F: drive: $($projectRoot.DisplayPath)"
    }
    if (-not [IO.Directory]::Exists($projectRoot.OperationalPath)) {
        throw "-ProjectRoot does not exist or is not a directory: $($projectRoot.DisplayPath)"
    }
    if (-not [IO.File]::Exists((Join-ZirconWindowsPath -Path $projectRoot.OperationalPath -ChildPath 'zircon-project.toml'))) {
        throw "-ProjectRoot is missing zircon-project.toml: $($projectRoot.DisplayPath)"
    }

    # Source templates must stay immutable even when they are on an approved drive.
    $templateRoot = Resolve-ZirconWindowsPath -Path (Join-Path $repoRoot 'templates\projects')
    $templatePrefix = $templateRoot.OperationalPath.TrimEnd('\', '/') + [IO.Path]::DirectorySeparatorChar
    if ($projectRoot.OperationalPath.Equals($templateRoot.OperationalPath, [StringComparison]::OrdinalIgnoreCase) -or
        $projectRoot.OperationalPath.StartsWith($templatePrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "-ProjectRoot must name a created project or example project, not a repository source template: $($projectRoot.DisplayPath)"
    }
    return $projectRoot
}

function Get-RenderExtractScaleProjectMetadata {
    param(
        [Parameter(Mandatory)]$ProjectRoot,
        [Parameter(Mandatory)][string]$ExpectedSourceFingerprint
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
    if ($null -eq $metadata -or [int]$metadata.schema_version -ne 1) {
        throw "Generated render-extract scale metadata has an unsupported schema_version: $metadataPath"
    }
    $projectFingerprint = [string]$metadata.source_fingerprint
    if ($projectFingerprint -notmatch '^[0-9A-F]{64}$') {
        throw "Generated render-extract scale metadata has an invalid source fingerprint: $metadataPath"
    }
    if (-not $projectFingerprint.Equals($ExpectedSourceFingerprint, [StringComparison]::Ordinal)) {
        throw 'Generated render-extract scale project belongs to a different source snapshot. Regenerate it before capture.'
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

function Get-RenderExtractManifestProperty {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Label
    )

    $property = $Value.PSObject.Properties[$Name]
    if ($null -eq $property -or $null -eq $property.Value) {
        throw "$Label is missing '$Name'."
    }
    return $property.Value
}

function Get-RenderExtractProfilingArtifact {
    param(
        [Parameter(Mandatory)]$Artifacts,
        [Parameter(Mandatory)][string]$LogicalId,
        [Parameter(Mandatory)][string]$ExpectedProduct,
        [Parameter(Mandatory)][string]$ExpectedPackage,
        [AllowNull()][string]$ExpectedBin,
        [Parameter(Mandatory)][string]$ExpectedFeatures
    )

    $matches = @($Artifacts | Where-Object {
            [string](Get-RenderExtractManifestProperty -Value $_ -Name 'logical_id' -Label 'Profiling input artifact') -eq $LogicalId
        })
    if ($matches.Count -ne 1) {
        throw "Profiling input must contain exactly one '$LogicalId' artifact; found $($matches.Count)."
    }
    $artifact = $matches[0]
    $product = [string](Get-RenderExtractManifestProperty -Value $artifact -Name 'product' -Label $LogicalId)
    $package = [string](Get-RenderExtractManifestProperty -Value $artifact -Name 'package' -Label $LogicalId)
    $features = [string](Get-RenderExtractManifestProperty -Value $artifact -Name 'features' -Label $LogicalId)
    if ($product -ne $ExpectedProduct -or $package -ne $ExpectedPackage -or $features -ne $ExpectedFeatures) {
        throw "Profiling input artifact '$LogicalId' does not match the declared $ExpectedProduct profiling contract."
    }
    $binProperty = $artifact.PSObject.Properties['bin']
    $actualBin = if ($null -eq $binProperty -or $null -eq $binProperty.Value) { $null } else { [string]$binProperty.Value }
    $binMatches = if ([string]::IsNullOrEmpty($ExpectedBin)) {
        [string]::IsNullOrEmpty($actualBin)
    }
    else {
        $actualBin -eq $ExpectedBin
    }
    if (-not $binMatches) {
        throw "Profiling input artifact '$LogicalId' has unexpected bin '$actualBin'."
    }

    $path = [string](Get-RenderExtractManifestProperty -Value $artifact -Name 'path' -Label $LogicalId)
    $resolution = Resolve-ZirconWindowsPath -Path $path
    if (-not [IO.File]::Exists($resolution.OperationalPath)) {
        throw "Profiling input artifact '$LogicalId' does not exist: $($resolution.DisplayPath)"
    }
    $expectedBytes = [Int64](Get-RenderExtractManifestProperty -Value $artifact -Name 'bytes' -Label $LogicalId)
    $actualBytes = [IO.FileInfo]::new($resolution.OperationalPath).Length
    if ($actualBytes -ne $expectedBytes) {
        throw "Profiling input artifact '$LogicalId' byte length changed from $expectedBytes to $actualBytes."
    }
    $expectedHash = [string](Get-RenderExtractManifestProperty -Value $artifact -Name 'sha256' -Label $LogicalId)
    $actualHash = Get-MvpProductInputFileSha256 -Path $resolution.OperationalPath
    if (-not $actualHash.Equals($expectedHash, [StringComparison]::Ordinal)) {
        throw "Profiling input artifact '$LogicalId' SHA-256 no longer matches its manifest."
    }
    return [pscustomobject]@{
        OperationalPath = $resolution.OperationalPath
        DisplayPath = $resolution.DisplayPath
        Sha256 = $actualHash
    }
}

function Resolve-RenderExtractProfilingInput {
    param(
        [Parameter(Mandatory)][string]$ManifestPath,
        [Parameter(Mandatory)][string]$ExpectedSourceFingerprint
    )

    $manifestResolution = Resolve-ZirconWindowsPath -Path $ManifestPath
    if (-not [IO.File]::Exists($manifestResolution.OperationalPath)) {
        throw "Profiling input manifest does not exist: $($manifestResolution.DisplayPath)"
    }
    try {
        $manifest = [IO.File]::ReadAllText($manifestResolution.OperationalPath) | ConvertFrom-Json -ErrorAction Stop
    }
    catch {
        throw "Profiling input manifest is not valid JSON: $($manifestResolution.DisplayPath): $($_.Exception.Message)"
    }
    if ([int](Get-RenderExtractManifestProperty -Value $manifest -Name 'schema_version' -Label 'Profiling input manifest') -ne 2) {
        throw 'Profiling input manifest schema_version must be 2.'
    }
    $sourceFingerprint = [string](Get-RenderExtractManifestProperty -Value $manifest -Name 'source_fingerprint' -Label 'Profiling input manifest')
    if (-not $sourceFingerprint.Equals($ExpectedSourceFingerprint, [StringComparison]::Ordinal)) {
        throw 'Profiling input manifest belongs to a different source snapshot. Rebuild profiling inputs before capture.'
    }
    if ([string](Get-RenderExtractManifestProperty -Value $manifest -Name 'cargo_profile' -Label 'Profiling input manifest') -ne 'profiling') {
        throw 'Profiling input manifest cargo_profile must be profiling.'
    }
    $artifacts = @(Get-RenderExtractManifestProperty -Value $manifest -Name 'artifacts' -Label 'Profiling input manifest')
    if ($artifacts.Count -ne 4) {
        throw "Profiling input manifest must contain exactly four runtime/editor artifacts; found $($artifacts.Count)."
    }
    $runtimeFeatures = 'target-client,platform-winit,input-gamepad,gamepad-gilrs,profiling'
    $editorFeatures = 'target-editor-host,profiling'
    $runtimeExecutable = Get-RenderExtractProfilingArtifact `
        -Artifacts $artifacts `
        -LogicalId 'runtime-profile-executable' `
        -ExpectedProduct 'runtime' `
        -ExpectedPackage 'zircon_app' `
        -ExpectedBin 'zircon_runtime' `
        -ExpectedFeatures $runtimeFeatures
    $runtimeLibrary = Get-RenderExtractProfilingArtifact `
        -Artifacts $artifacts `
        -LogicalId 'runtime-profile-library' `
        -ExpectedProduct 'runtime' `
        -ExpectedPackage 'zircon_runtime' `
        -ExpectedBin $null `
        -ExpectedFeatures $runtimeFeatures
    $editorExecutable = Get-RenderExtractProfilingArtifact `
        -Artifacts $artifacts `
        -LogicalId 'editor-profile-executable' `
        -ExpectedProduct 'editor' `
        -ExpectedPackage 'zircon_app' `
        -ExpectedBin 'zircon_editor' `
        -ExpectedFeatures $editorFeatures
    $editorLibrary = Get-RenderExtractProfilingArtifact `
        -Artifacts $artifacts `
        -LogicalId 'editor-profile-library' `
        -ExpectedProduct 'editor' `
        -ExpectedPackage 'zircon_runtime' `
        -ExpectedBin $null `
        -ExpectedFeatures $editorFeatures
    $manifestDirectory = [IO.Path]::GetDirectoryName($manifestResolution.OperationalPath)
    foreach ($product in @(
            [pscustomobject]@{ Name = 'runtime'; Executable = $runtimeExecutable; Library = $runtimeLibrary },
            [pscustomobject]@{ Name = 'editor'; Executable = $editorExecutable; Library = $editorLibrary }
        )) {
        $executableDirectory = [IO.Path]::GetDirectoryName($product.Executable.OperationalPath)
        $libraryDirectory = [IO.Path]::GetDirectoryName($product.Library.OperationalPath)
        if (-not $executableDirectory.Equals($libraryDirectory, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Profiling input $($product.Name) executable and runtime library must be in the same directory for relative runtime loading."
        }
        $expectedDirectory = Join-ZirconWindowsPath -Path $manifestDirectory -ChildPath $product.Name
        if (-not $executableDirectory.Equals($expectedDirectory, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Profiling input $($product.Name) pair must live in its managed product directory under the manifest."
        }
    }
    return [pscustomobject]@{
        manifest_path = $manifestResolution.OperationalPath
        manifest_sha256 = Get-MvpProductInputFileSha256 -Path $manifestResolution.OperationalPath
        runtime = [pscustomobject]@{
            executable_path = $runtimeExecutable.OperationalPath
            executable_sha256 = $runtimeExecutable.Sha256
            library_path = $runtimeLibrary.OperationalPath
            library_sha256 = $runtimeLibrary.Sha256
        }
        editor = [pscustomobject]@{
            executable_path = $editorExecutable.OperationalPath
            executable_sha256 = $editorExecutable.Sha256
            library_path = $editorLibrary.OperationalPath
            library_sha256 = $editorLibrary.Sha256
        }
    }
}

function Assert-RenderExtractProfilingInputIdentity {
    param(
        [Parameter(Mandatory)]$Expected,
        [Parameter(Mandatory)]$Actual
    )

    foreach ($name in @('manifest_path')) {
        if (-not ([string]$Expected.$name).Equals([string]$Actual.$name, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Profiling input identity changed during baseline capture ('$name')."
        }
    }
    foreach ($name in @('manifest_sha256')) {
        if (-not ([string]$Expected.$name).Equals([string]$Actual.$name, [StringComparison]::Ordinal)) {
            throw "Profiling input identity changed during baseline capture ('$name')."
        }
    }
    foreach ($product in @('runtime', 'editor')) {
        foreach ($name in @('executable_path', 'library_path')) {
            if (-not ([string]$Expected.$product.$name).Equals([string]$Actual.$product.$name, [StringComparison]::OrdinalIgnoreCase)) {
                throw "Profiling input identity changed during baseline capture ('$product.$name')."
            }
        }
        foreach ($name in @('executable_sha256', 'library_sha256')) {
            if (-not ([string]$Expected.$product.$name).Equals([string]$Actual.$product.$name, [StringComparison]::Ordinal)) {
                throw "Profiling input identity changed during baseline capture ('$product.$name')."
            }
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

function Start-RenderExtractWprCapture {
    param([Parameter(Mandatory)][string]$TemporaryDirectory)

    if (-not [IO.Directory]::Exists($TemporaryDirectory)) {
        throw "WPR recording temporary directory does not exist: $TemporaryDirectory"
    }
    $wpr = Get-Command wpr.exe -ErrorAction SilentlyContinue
    if ($null -eq $wpr) {
        throw '-UseWpr requires wpr.exe on the Windows PATH.'
    }
    & $wpr.Source '-start' 'CPU' '-filemode' '-recordtempto' $TemporaryDirectory | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw "WPR could not start the CPU capture; exit code $LASTEXITCODE."
    }
    return $wpr.Source
}

function Stop-RenderExtractWprCapture {
    param(
        [Parameter(Mandatory)][string]$WprPath,
        [Parameter(Mandatory)][string]$TracePath
    )

    & $WprPath '-stop' $TracePath | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw "WPR could not stop the CPU capture at '$TracePath'; exit code $LASTEXITCODE."
    }
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
        [switch]$UseWpr
    )

    $sessionId = "$($Run.logical_id)-$Attempt"
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
    if ($UseWpr) {
        [IO.Directory]::CreateDirectory($tracesRoot) | Out-Null
    }
    $stdoutPath = Join-ZirconWindowsPath -Path $logsRoot -ChildPath "$sessionId.stdout.log"
    $stderrPath = Join-ZirconWindowsPath -Path $logsRoot -ChildPath "$sessionId.stderr.log"
    $capturePath = Join-ZirconWindowsPath -Path $capturesRoot -ChildPath "$sessionId.png"
    $systemTracePath = if ($UseWpr) {
        Join-ZirconWindowsPath -Path $tracesRoot -ChildPath "$sessionId.etl"
    }
    else {
        $null
    }
    # WPR file mode must keep recorder buffers inside this E-drive evidence session.
    $wprTemporaryDirectory = if ($UseWpr) {
        Join-ZirconWindowsPath -Path $tracesRoot -ChildPath "$sessionId.wpr-temp"
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
    $wprPath = $null
    $primaryFailure = $null
    $processCleanupFailure = $null
    $wprStopFailure = $null
    $processStarted = $false
    $processJob = $null
    $actualProductHashes = $null
    $peakWorkingSetBytes = $null
    $totalProcessorTimeMs = $null
    $processId = $null
    try {
        $actualProductHashes = Assert-RenderExtractFrozenProductInput `
            -ProductInput $productInput `
            -Product $Run.product
        $processJob = New-RenderExtractBaselineProcessJob
        if ($UseWpr) {
            [IO.Directory]::CreateDirectory($wprTemporaryDirectory) | Out-Null
            $wprPath = Start-RenderExtractWprCapture -TemporaryDirectory $wprTemporaryDirectory
        }
        $startedAtUtc = [DateTimeOffset]::UtcNow.ToString('o')
        $processStopwatch.Start()
        $assignedProcess = Start-RenderExtractBaselineAssignedProcess -Job $processJob -StartInfo $startInfo
        $process = $assignedProcess.Process
        $processId = [Int64]$process.Id
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
        if ($null -ne $assignedProcess) {
            $assignedProcess.Dispose()
        }
        if ($null -ne $wprPath) {
            try {
                Stop-RenderExtractWprCapture -WprPath $wprPath -TracePath $systemTracePath
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

    $profileDirectory = Join-ZirconWindowsPath -Path $profilesRoot -ChildPath $sessionId
    foreach ($name in @('timeline.zrtrace.json', 'hotspots.json', 'counter_hotspots.json', 'summary.md')) {
        $path = Join-ZirconWindowsPath -Path $profileDirectory -ChildPath $name
        if (-not [IO.File]::Exists($path) -or [IO.FileInfo]::new($path).Length -le 0) {
            throw "Render-extract run '$sessionId' did not export nonempty $name."
        }
    }
    if (-not [IO.File]::Exists($capturePath) -or [IO.FileInfo]::new($capturePath).Length -le 0) {
        throw "Render-extract run '$sessionId' did not produce a nonempty $($Run.product) PNG."
    }
    if ($UseWpr -and (-not [IO.File]::Exists($systemTracePath) -or [IO.FileInfo]::new($systemTracePath).Length -le 0)) {
        throw "Render-extract run '$sessionId' did not produce a nonempty WPR ETL."
    }
    return [ordered]@{
        logical_id = $Run.logical_id
        product = $Run.product
        attempt = $Attempt
        invocation_id = $InvocationId
        runtime_profile = $Run.runtime_profile
        warmup_presented_frame_count = $Run.warmup_presented_frame_count
        measured_presented_frame_count = $Run.measured_presented_frame_count
        target_presented_frame_count = $Run.target_presented_frame_count
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
        system_trace_etl = if ($UseWpr) { (Resolve-ZirconWindowsPath -Path $systemTracePath).DisplayPath } else { $null }
        profiling_input = [ordered]@{
            manifest_sha256 = $ProfilingInput.manifest_sha256
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

    $resolvedOutputDirectory = Assert-RenderExtractBaselineOutputDirectory -Path $EvidenceOutputDirectory
    $projectRoot = Assert-RenderExtractBaselineProjectDirectory -Path $ProjectPath

    $sourceFingerprint = Get-MvpSourceFingerprint -RepositoryRoot $repoRoot
    $currentFingerprint = Get-MvpSourceFingerprint -RepositoryRoot $repoRoot
    if (-not $currentFingerprint.Equals($sourceFingerprint, [StringComparison]::Ordinal)) {
        throw 'Source fingerprint changed during baseline capture preflight. Rebuild profiling inputs before capture.'
    }
    $scaleProject = Get-RenderExtractScaleProjectMetadata `
        -ProjectRoot $projectRoot `
        -ExpectedSourceFingerprint $sourceFingerprint
    $profilingInput = Resolve-RenderExtractProfilingInput `
        -ManifestPath $ManifestPath `
        -ExpectedSourceFingerprint $sourceFingerprint

    $runPlan = @(Get-RenderExtractBaselineRunPlan `
            -RepeatCount $RepeatCount `
            -WarmupPresentedFrameCount $WarmupPresentedFrameCount `
            -MeasuredPresentedFrameCount $MeasuredPresentedFrameCount)
    $sessionLease = New-RenderExtractBaselineOutputSessionLease -Path $resolvedOutputDirectory
    $runs = [System.Collections.Generic.List[object]]::new()
    try {
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
                $currentFingerprint = Get-MvpSourceFingerprint -RepositoryRoot $repoRoot
                if (-not $currentFingerprint.Equals($sourceFingerprint, [StringComparison]::Ordinal)) {
                    throw 'Source fingerprint changed during baseline capture. Rebuild profiling inputs before another capture.'
                }
                $currentInput = Resolve-RenderExtractProfilingInput `
                    -ManifestPath $ManifestPath `
                    -ExpectedSourceFingerprint $sourceFingerprint
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
                        -UseWpr:$UseWpr)) | Out-Null
            }
        }

        $finalFingerprint = Get-MvpSourceFingerprint -RepositoryRoot $repoRoot
        if (-not $finalFingerprint.Equals($sourceFingerprint, [StringComparison]::Ordinal)) {
            throw 'Source fingerprint changed during baseline capture. The collected evidence is not source-bound.'
        }

        $summaryPath = Join-ZirconWindowsPath -Path $resolvedOutputDirectory -ChildPath 'render-extract-baseline.json'
        $summary = [ordered]@{
            schema_version = 4
            generated_at_utc = [DateTimeOffset]::UtcNow.ToString('o')
            source_fingerprint = $sourceFingerprint
            profiling_input_manifest_sha256 = $frozenProfilingInput.manifest_sha256
            invocation_id = $sessionLease.InvocationId
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
            -Content ($summary | ConvertTo-Json -Depth 6)
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
