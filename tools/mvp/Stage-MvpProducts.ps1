[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string]$RuntimeExecutable,
    [Parameter(Mandatory)]
    [string]$EditorExecutable,
    [Parameter(Mandatory)]
    [string]$RuntimeLibrary,
    [Parameter(Mandatory)]
    [ValidateNotNullOrEmpty()]
    [string]$EditorRuntimeLibrary,
    [Parameter(Mandatory)]
    [string]$TemplateRoot,
    [Parameter(Mandatory)]
    [string]$EngineAssetRoot,
    [string]$ProjectRoot,
    [string]$AuthoringAutomationRequest,
    [string]$ReopenAutomationRequest,
    [switch]$CreateProject,
    [string]$ProjectName = 'ZirconMvpFixture',
    [string]$StagingRoot = 'E:\ZirconBuilds',
    [string]$SourceFingerprint,
    [string]$RunId = ('mvp-f0-' + (Get-Date -Format 'yyyyMMdd-HHmmss') + '-' + [guid]::NewGuid().ToString('N').Substring(0, 8)),
    [ValidateRange(1, 4)]
    [int]$RepeatCount = 2,
    [ValidateRange(1, 4)]
    [int]$ReopenRepeatCount = 2,
    [ValidateRange(1, 600)]
    [int]$TimeoutSeconds = 90,
    [switch]$NoLaunch,
    [switch]$AllowUnsafeStagingRoot,
    [switch]$Json
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Import-Module (Join-Path $PSScriptRoot 'MvpProjectOpenEvidence.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpStagingPreflight.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpStagingRelease.psm1') -Force -ErrorAction Stop
$pathResolverRepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Import-Module (Join-Path $pathResolverRepoRoot 'tools\WindowsPathResolver.psm1') -Force -ErrorAction Stop

function Get-TextSha256 {
    param([Parameter(Mandatory)][string]$Text)

    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        $bytes = [Text.Encoding]::UTF8.GetBytes($Text)
        return -join ($hasher.ComputeHash($bytes) | ForEach-Object { $_.ToString('X2') })
    }
    finally {
        $hasher.Dispose()
    }
}

function Get-FileSha256 {
    param([Parameter(Mandatory)][string]$Path)

    $stream = [IO.File]::OpenRead($Path)
    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        return -join ($hasher.ComputeHash($stream) | ForEach-Object { $_.ToString('X2') })
    }
    finally {
        $hasher.Dispose()
        $stream.Dispose()
    }
}

function Get-MvpSourceFingerprint {
    $repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
    $git = Get-Command git -ErrorAction SilentlyContinue
    if ($null -eq $git) {
        throw 'Could not resolve git for the MVP source fingerprint. Supply -SourceFingerprint explicitly.'
    }

    $commit = (& $git.Source -C $repoRoot rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($commit)) {
        throw 'Could not resolve the current source commit. Supply -SourceFingerprint explicitly.'
    }
    $trackedDiff = (& $git.Source -C $repoRoot diff --no-ext-diff --binary HEAD) -join "`n"
    if ($LASTEXITCODE -ne 0) {
        throw 'Could not resolve tracked working-tree content for the MVP source fingerprint. Supply -SourceFingerprint explicitly.'
    }
    $untrackedPaths = @(& $git.Source -C $repoRoot ls-files --others --exclude-standard)
    if ($LASTEXITCODE -ne 0) {
        throw 'Could not enumerate untracked working-tree inputs for the MVP source fingerprint. Supply -SourceFingerprint explicitly.'
    }
    $untrackedInputs = foreach ($relativePath in ($untrackedPaths | Sort-Object)) {
        $path = Join-Path $repoRoot $relativePath
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
            throw "Untracked source input '$relativePath' does not exist or is not a file."
        }
        "$relativePath`0$(Get-FileSha256 -Path $path)"
    }
    return Get-TextSha256 "commit=$commit`ntracked_diff:`n$trackedDiff`nuntracked_inputs:`n$($untrackedInputs -join "`n")"
}

function Resolve-MvpValidationMetadata {
    $rustc = Get-Command rustc -ErrorAction SilentlyContinue
    if ($null -eq $rustc) {
        throw 'Could not resolve rustc for MVP validation metadata.'
    }
    $metadata = @(& $rustc.Source -Vv)
    if ($LASTEXITCODE -ne 0) {
        throw 'Could not inspect rustc for MVP validation metadata.'
    }
    $toolchainLine = @($metadata | Where-Object { $_ -match '^rustc\s+' } | Select-Object -First 1)
    $targetLine = @($metadata | Where-Object { $_ -match '^host:\s+' } | Select-Object -First 1)
    if ($toolchainLine.Count -ne 1 -or $targetLine.Count -ne 1) {
        throw 'rustc -Vv did not report a usable toolchain and host target.'
    }
    $target = ($targetLine[0] -replace '^host:\s+', '').Trim()
    if ([string]::IsNullOrWhiteSpace($target)) {
        throw 'rustc -Vv reported a blank host target.'
    }
    return [ordered]@{
        toolchain = $toolchainLine[0].Trim()
        target = $target
    }
}

function Resolve-MvpInputFile {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Label
    )

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "$Label '$Path' does not exist or is not a file."
    }
    return (Resolve-ZirconWindowsPath -Path $Path).OperationalPath
}

function Resolve-MvpInputDirectory {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Label
    )

    if (-not (Test-Path -LiteralPath $Path -PathType Container)) {
        throw "$Label '$Path' does not exist or is not a directory."
    }
    return (Resolve-ZirconWindowsPath -Path $Path).OperationalPath
}

function Assert-MvpDistinctProfileRuntimeLibraries {
    param(
        [Parameter(Mandatory)][string]$RuntimeLibraryPath,
        [Parameter(Mandatory)][string]$EditorRuntimeLibraryPath
    )

    $runtimeIdentity = Get-ZirconWindowsFileIdentity -Path $RuntimeLibraryPath
    $editorIdentity = Get-ZirconWindowsFileIdentity -Path $EditorRuntimeLibraryPath
    if ($runtimeIdentity -eq $editorIdentity) {
        throw 'RuntimeLibrary and EditorRuntimeLibrary must resolve to distinct physical profile artifacts.'
    }
}

function Resolve-MvpStagingRoot {
    param([Parameter(Mandatory)][string]$Path)

    # The approved staging root has a bounded normal form. Compare and return that display path
    # so later PowerShell provider calls never receive a verbatim path, while arbitrary product
    # inputs still retain their resolver physical paths.
    $resolution = Resolve-ZirconWindowsPath -Path $Path
    $displayPath = $resolution.DisplayPath.TrimEnd('\')
    if (-not $AllowUnsafeStagingRoot -and $displayPath -notmatch '^[D-F]:\\ZirconBuilds(?:\\|$)') {
        throw "StagingRoot '$displayPath' is not under an approved D:\ZirconBuilds, E:\ZirconBuilds, or F:\ZirconBuilds root."
    }
    return $displayPath
}

function Assert-MvpRunId {
    param([Parameter(Mandatory)][string]$Value)

    if ($Value -notmatch '^[A-Za-z0-9][A-Za-z0-9._-]*$') {
        throw "RunId '$Value' must contain only letters, numbers, dot, underscore, and dash."
    }
}

function Assert-MvpProjectName {
    param([Parameter(Mandatory)][string]$Value)

    if ([string]::IsNullOrWhiteSpace($Value)) {
        throw 'CreateProject requires a non-empty ProjectName.'
    }
    if ($Value -eq '.' -or $Value -eq '..' -or
        [IO.Path]::IsPathRooted($Value) -or
        $Value.IndexOf([IO.Path]::DirectorySeparatorChar) -ge 0 -or
        $Value.IndexOf([IO.Path]::AltDirectorySeparatorChar) -ge 0 -or
        $Value.IndexOfAny([IO.Path]::GetInvalidFileNameChars()) -ge 0 -or
        $Value.Trim() -ne $Value -or
        $Value.EndsWith('.', [StringComparison]::Ordinal)) {
        throw "ProjectName '$Value' must be one safe directory name under the staged project root."
    }
}

function Get-MvpRelativePath {
    param(
        [Parameter(Mandatory)][string]$Root,
        [Parameter(Mandatory)][string]$Path,
        [string]$Label = 'Staged file'
    )

    $resolvedRoot = (Resolve-ZirconWindowsPath -Path $Root).OperationalPath
    $resolvedPath = (Resolve-ZirconWindowsPath -Path $Path).OperationalPath
    $directorySeparator = [string][IO.Path]::DirectorySeparatorChar
    $alternateDirectorySeparator = [string][IO.Path]::AltDirectorySeparatorChar
    $rootPrefix = if ($resolvedRoot.EndsWith($directorySeparator) -or $resolvedRoot.EndsWith($alternateDirectorySeparator)) {
        $resolvedRoot
    } else {
        $resolvedRoot + $directorySeparator
    }
    $comparison = [StringComparison]::OrdinalIgnoreCase
    if ($resolvedPath.Equals($resolvedRoot, $comparison)) {
        return '.'
    }
    if (-not $resolvedPath.StartsWith($rootPrefix, $comparison)) {
        throw "$Label '$Path' is outside staging root '$Root'."
    }
    return $resolvedPath.Substring($rootPrefix.Length).Replace('\', '/')
}

function Test-MvpProjectSourceRelativePath {
    param([Parameter(Mandatory)][string]$RelativePath)

    # Derived project state must be rebuilt by the staged products from source inputs.
    $normalized = $RelativePath.Replace('\', '/').TrimStart('/')
    foreach ($generatedDirectory in @(
        '.zircon/autosave',
        '.zircon/cache',
        '.zircon/play',
        '.zircon/registry',
        '.zircon/thumbnails'
    )) {
        if ($normalized.Equals($generatedDirectory, [StringComparison]::OrdinalIgnoreCase) -or
            $normalized.StartsWith($generatedDirectory + '/', [StringComparison]::OrdinalIgnoreCase)) {
            return $false
        }
    }
    return $true
}

function Copy-MvpStageFile {
    param(
        [Parameter(Mandatory)][string]$LogicalId,
        [Parameter(Mandatory)][string]$SourcePath,
        [Parameter(Mandatory)][string]$StageRoot,
        [Parameter(Mandatory)][string]$TargetRelativePath
    )

    if ([IO.Path]::IsPathRooted($TargetRelativePath) -or $TargetRelativePath -match '(^|[\\/])\.\.([\\/]|$)') {
        throw "Staging target '$TargetRelativePath' for $LogicalId escapes the staging root."
    }
    $targetPath = Join-Path $StageRoot $TargetRelativePath
    $targetDirectory = Split-Path -Parent $targetPath
    New-Item -ItemType Directory -Force -Path $targetDirectory | Out-Null
    [IO.File]::Copy($SourcePath, $targetPath, $false)
    $sourceHash = Get-FileSha256 -Path $SourcePath
    $targetHash = Get-FileSha256 -Path $targetPath
    if ($sourceHash -ne $targetHash) {
        throw "Content hash mismatch while staging $LogicalId from '$SourcePath'."
    }

    return [ordered]@{
        logical_id = $LogicalId
        target_relative_path = $TargetRelativePath.Replace('\', '/')
        sha256 = $targetHash
        size_bytes = (Get-Item -LiteralPath $targetPath).Length
    }
}

function Write-MvpJson {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)]$Value
    )

    [IO.File]::WriteAllText(
        $Path,
        ($Value | ConvertTo-Json -Depth 64),
        [Text.UTF8Encoding]::new($false)
    )
}

function Get-MvpStagedFileEvidence {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$StageRoot,
        [Parameter(Mandatory)][string]$Label
    )

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "$Label '$Path' does not exist."
    }
    $item = Get-Item -LiteralPath $Path -ErrorAction Stop
    return [ordered]@{
        path = Get-MvpRelativePath -Root $StageRoot -Path $Path -Label $Label
        sha256 = Get-FileSha256 -Path $Path
        size_bytes = $item.Length
    }
}

function Get-MvpPngCaptureEvidence {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$StageRoot,
        [Parameter(Mandatory)][string]$Label
    )

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "$Label '$Path' was not written."
    }
    $capture = Get-Item -LiteralPath $Path -ErrorAction Stop
    if ($capture.Length -le 0) {
        throw "$Label '$Path' is empty."
    }
    if ($null -eq ('ZirconMvpPngEvidence' -as [type])) {
        Add-Type -AssemblyName System.Drawing -ErrorAction Stop
        $pngEvidenceReferences = @(
            [Drawing.Bitmap].Assembly.Location
            [Drawing.Rectangle].Assembly.Location
            [Security.Cryptography.SHA256].Assembly.Location
        ) | Select-Object -Unique
        Add-Type -TypeDefinition @'
using System;
using System.Drawing;
using System.Drawing.Imaging;
using System.Runtime.InteropServices;
using System.Security.Cryptography;

public sealed class ZirconMvpPngEvidence
{
    public int Width { get; private set; }
    public int Height { get; private set; }
    public long NonBackgroundPixels { get; private set; }
    public long NonTransparentPixels { get; private set; }
    public string PixelSha256 { get; private set; }

    public static ZirconMvpPngEvidence Inspect(string path)
    {
        using (var source = new Bitmap(path))
        {
            if (source.Width <= 0 || source.Height <= 0)
            {
                throw new InvalidOperationException("PNG dimensions must be positive.");
            }
            using (var normalized = new Bitmap(source.Width, source.Height, PixelFormat.Format32bppArgb))
            {
                using (var graphics = Graphics.FromImage(normalized))
                {
                    graphics.DrawImageUnscaled(source, 0, 0);
                }
                var bounds = new Rectangle(0, 0, normalized.Width, normalized.Height);
                var bitmapData = normalized.LockBits(bounds, ImageLockMode.ReadOnly, PixelFormat.Format32bppArgb);
                try
                {
                    var stride = bitmapData.Stride;
                    var bytes = checked(Math.Abs(stride) * normalized.Height);
                    var pixels = new byte[bytes];
                    Marshal.Copy(bitmapData.Scan0, pixels, 0, bytes);
                    var evidence = new ZirconMvpPngEvidence
                    {
                        Width = normalized.Width,
                        Height = normalized.Height,
                    };
                    var canonicalPixels = new byte[checked(normalized.Width * normalized.Height * 4)];
                    var background = 0;
                    var backgroundSet = false;
                    for (var y = 0; y < normalized.Height; y++)
                    {
                        var row = stride >= 0 ? y * stride : (normalized.Height - 1 - y) * -stride;
                        for (var x = 0; x < normalized.Width; x++)
                        {
                            var offset = row + x * 4;
                            var argb = pixels[offset] | (pixels[offset + 1] << 8) |
                                (pixels[offset + 2] << 16) | (pixels[offset + 3] << 24);
                            if (!backgroundSet)
                            {
                                background = argb;
                                backgroundSet = true;
                            }
                            if (argb != background)
                            {
                                evidence.NonBackgroundPixels++;
                            }
                            if (pixels[offset + 3] != 0)
                            {
                                evidence.NonTransparentPixels++;
                            }
                            Buffer.BlockCopy(pixels, offset, canonicalPixels, (y * normalized.Width + x) * 4, 4);
                        }
                    }
                    using (var hasher = SHA256.Create())
                    {
                        evidence.PixelSha256 = BitConverter.ToString(hasher.ComputeHash(canonicalPixels)).Replace("-", string.Empty);
                    }
                    return evidence;
                }
                finally
                {
                    normalized.UnlockBits(bitmapData);
                }
            }
        }
    }
}
'@ -ReferencedAssemblies $pngEvidenceReferences -ErrorAction Stop
    }

    $summary = [ZirconMvpPngEvidence]::Inspect($Path)
    if ($summary.NonTransparentPixels -le 0) {
        throw "$Label '$Path' has no visible pixels."
    }
    if ($summary.NonBackgroundPixels -lt 100) {
        throw "$Label '$Path' has only $($summary.NonBackgroundPixels) non-background pixels; expected at least 100."
    }
    return [ordered]@{
        path = Get-MvpRelativePath -Root $StageRoot -Path $Path -Label $Label
        sha256 = Get-FileSha256 -Path $Path
        size_bytes = $capture.Length
        pixel_sha256 = $summary.PixelSha256
        width = $summary.Width
        height = $summary.Height
        non_background_pixels = $summary.NonBackgroundPixels
        non_transparent_pixels = $summary.NonTransparentPixels
    }
}

function Get-MvpRuntimeFrameCaptureEvidence {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$StageRoot
    )

    return Get-MvpPngCaptureEvidence -Path $Path -StageRoot $StageRoot -Label 'Runtime frame capture'
}

function Get-MvpEditorWindowCaptureEvidence {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$StageRoot
    )

    return Get-MvpPngCaptureEvidence -Path $Path -StageRoot $StageRoot -Label 'Editor window capture'
}

function ConvertTo-MvpProcessArgument {
    param([Parameter(Mandatory)][string]$Value)

    if ($Value.IndexOf('"') -ge 0) {
        throw "Process argument '$Value' contains an unsupported quote character."
    }
    if ($Value -notmatch '[\s]') {
        return $Value
    }

    $trailingBackslashes = 0
    for ($index = $Value.Length - 1; $index -ge 0 -and $Value[$index] -eq '\'; $index--) {
        $trailingBackslashes++
    }
    return '"' + $Value + ((@('\') * $trailingBackslashes) -join '') + '"'
}

function Write-MvpProcessJournalEntry {
    param(
        [Parameter(Mandatory)][string]$StageRoot,
        [Parameter(Mandatory)][string]$Phase,
        [Parameter(Mandatory)][string]$StartedAtUtc,
        [Parameter(Mandatory)][string]$EndedAtUtc,
        [Parameter(Mandatory)][AllowNull()][Nullable[int]]$ExitCode,
        [Parameter(Mandatory)][ValidateSet('exited', 'timed_out', 'cleanup_failed')][string]$Outcome
    )

    $logRoot = Join-Path $StageRoot 'logs'
    New-Item -ItemType Directory -Force -Path $logRoot | Out-Null
    $journalPath = Join-Path $logRoot 'process-execution-journal.jsonl'
    $entry = [ordered]@{
        phase = $Phase
        started_at_utc = $StartedAtUtc
        ended_at_utc = $EndedAtUtc
        exit_code = $ExitCode
        outcome = $Outcome
    }
    [IO.File]::AppendAllText(
        $journalPath,
        (($entry | ConvertTo-Json -Compress) + [Environment]::NewLine),
        [Text.UTF8Encoding]::new($false)
    )
}

function Start-MvpStagedProcess {
    param(
        [Parameter(Mandatory)][string]$ExecutablePath,
        [Parameter(Mandatory)][string]$WorkingDirectory,
        [Parameter(Mandatory)][hashtable]$Environment,
        [Parameter(Mandatory)][string]$StageRoot,
        [Parameter(Mandatory)][string]$Phase,
        [string]$ProjectRoot,
        [string[]]$Arguments = @()
    )

    # ProcessStartInfo keeps F0 launch behavior available to Windows PowerShell 5.1, whose
    # Start-Process command lacks the Environment parameter used by newer PowerShell hosts.
    $startInfo = [Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $ExecutablePath
    $startInfo.WorkingDirectory = $WorkingDirectory
    $startInfo.UseShellExecute = $false
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    foreach ($name in $Environment.Keys) {
        $startInfo.EnvironmentVariables[[string]$name] = [string]$Environment[$name]
    }
    if (-not $Environment.ContainsKey('ZIRCON_RUNTIME_CAPTURE_FRAME_PNG')) {
        $startInfo.EnvironmentVariables.Remove('ZIRCON_RUNTIME_CAPTURE_FRAME_PNG')
    }
    if (-not $Environment.ContainsKey('ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG')) {
        $startInfo.EnvironmentVariables.Remove('ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG')
    }
    if (-not $Environment.ContainsKey('ZIRCON_RUNTIME_MVP_INPUT_PROBE')) {
        $startInfo.EnvironmentVariables.Remove('ZIRCON_RUNTIME_MVP_INPUT_PROBE')
    }
    if (-not [string]::IsNullOrWhiteSpace($ProjectRoot) -and $Arguments.Count -gt 0) {
        throw 'ProjectRoot cannot be combined with explicit staged process arguments.'
    }
    if (-not [string]::IsNullOrWhiteSpace($ProjectRoot)) {
        # The product CLI accepts a normal Windows path and resolves its physical identity at the
        # project boundary. Keep physical resolver paths for staging filesystem operations only.
        $projectRootArgument = (Resolve-ZirconWindowsPath -Path $ProjectRoot).DisplayPath
        $startInfo.Arguments = '--project ' + (ConvertTo-MvpProcessArgument -Value $projectRootArgument)
    } elseif ($Arguments.Count -gt 0) {
        $startInfo.Arguments = ($Arguments | ForEach-Object {
            ConvertTo-MvpProcessArgument -Value $_
        }) -join ' '
    }

    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $startInfo
    $startedAtUtc = [DateTimeOffset]::UtcNow.ToString('o')
    if (-not $process.Start()) {
        $process.Dispose()
        throw "The operating system did not start '$ExecutablePath'."
    }
    return [pscustomobject]@{
        process = $process
        stdout_task = $process.StandardOutput.ReadToEndAsync()
        stderr_task = $process.StandardError.ReadToEndAsync()
        staged_product_root = [IO.Path]::GetFullPath($StageRoot)
        phase = $Phase
        started_at_utc = $startedAtUtc
        ended_at_utc = $null
    }
}

function Get-MvpStagedProcesses {
    param([Parameter(Mandatory)][string]$StageDirectory)

    $resolvedDirectory = [IO.Path]::GetFullPath($StageDirectory).TrimEnd('\\')
    $directoryPrefix = $resolvedDirectory + [IO.Path]::DirectorySeparatorChar
    return @(
        Get-CimInstance Win32_Process -ErrorAction Stop |
            Where-Object {
                $executablePath = [string]$_.ExecutablePath
                -not [string]::IsNullOrWhiteSpace($executablePath) -and
                    $executablePath.StartsWith($directoryPrefix, [StringComparison]::OrdinalIgnoreCase)
            }
    )
}

function Stop-MvpStagedProcesses {
    param([Parameter(Mandatory)][string]$StageDirectory)

    $taskKill = Get-Command taskkill.exe -ErrorAction Stop
    $deadline = [DateTime]::UtcNow.AddSeconds(5)
    do {
        $lingering = @(Get-MvpStagedProcesses -StageDirectory $StageDirectory)
        if ($lingering.Count -eq 0) {
            return @()
        }
        foreach ($stagedProcess in $lingering) {
            & $taskKill.Source '/PID' $stagedProcess.ProcessId '/T' '/F' 2>$null | Out-Null
        }
        if ([DateTime]::UtcNow -lt $deadline) {
            Start-Sleep -Milliseconds 50
        }
    } while ([DateTime]::UtcNow -lt $deadline)

    return @(Get-MvpStagedProcesses -StageDirectory $StageDirectory)
}

function Stop-MvpTimedOutStagedProcessTree {
    param(
        [Parameter(Mandatory)][int]$RootProcessId,
        [Parameter(Mandatory)][string]$StageDirectory
    )

    # A root process can exit between WaitForExit timing out and taskkill receiving its PID.
    # Sweep the staged executable directory as well so an orphaned helper cannot survive that race.
    $taskKill = Get-Command taskkill.exe -ErrorAction Stop
    & $taskKill.Source '/PID' $RootProcessId '/T' '/F' 2>$null | Out-Null

    $lingering = @(Stop-MvpStagedProcesses -StageDirectory $StageDirectory)
    if ($lingering.Count -eq 0) {
        return
    }
    $details = $lingering | ForEach-Object {
        "pid=$($_.ProcessId) name=$($_.Name) path=$($_.ExecutablePath)"
    }
    throw "Could not terminate timed-out staged process tree rooted at pid ${RootProcessId}: $($details -join '; ')."
}

function Receive-MvpProcessStream {
    param(
        [Parameter(Mandatory)]$Task,
        [Parameter(Mandatory)][string]$Label
    )

    try {
        if (-not $Task.Wait(5000)) {
            return "[MVP staging could not drain $Label within 5 seconds after process cleanup.]"
        }
        return $Task.GetAwaiter().GetResult()
    }
    catch {
        return "[MVP staging could not drain ${Label}: $($_.Exception.Message)]"
    }
}

function Complete-MvpStagedProcess {
    param(
        [Parameter(Mandatory)]$ProcessState,
        [Parameter(Mandatory)][string]$StdoutPath,
        [Parameter(Mandatory)][string]$StderrPath,
        [Parameter(Mandatory)][int]$TimeoutSeconds
    )

    $process = $ProcessState.process
    $timedOut = -not $process.WaitForExit($TimeoutSeconds * 1000)
    $timeoutCleanupErrors = [System.Collections.Generic.List[string]]::new()
    if ($timedOut) {
        try {
            Stop-MvpTimedOutStagedProcessTree `
                -RootProcessId $process.Id `
                -StageDirectory $ProcessState.staged_product_root
        }
        catch {
            $timeoutCleanupErrors.Add($_.Exception.Message)
            try {
                if (-not $process.HasExited) {
                    $process.Kill()
                }
            }
            catch {
                $timeoutCleanupErrors.Add("Fallback root-process termination failed: $($_.Exception.Message)")
            }
        }
    }
    $processExited = if ($timedOut) {
        $process.WaitForExit(5000)
    }
    else {
        $process.WaitForExit()
        $true
    }
    if (-not $processExited) {
        $timeoutCleanupErrors.Add('Root process did not exit within 5 seconds after timeout cleanup.')
    }
    $ProcessState.ended_at_utc = [DateTimeOffset]::UtcNow.ToString('o')
    $exitCode = if ($processExited) { $process.ExitCode } else { -1 }
    $releaseError = $null
    if (-not $timedOut) {
        try {
            Assert-MvpStagingProcessesReleased -StageDirectory $ProcessState.staged_product_root
        }
        catch {
            $releaseError = $_.Exception
        }
    }
    $stdout = Receive-MvpProcessStream -Task $ProcessState.stdout_task -Label 'stdout'
    $stderr = Receive-MvpProcessStream -Task $ProcessState.stderr_task -Label 'stderr'
    [IO.File]::WriteAllText($StdoutPath, $stdout, [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText($StderrPath, $stderr, [Text.UTF8Encoding]::new($false))
    $outcome = if ($timedOut) {
        'timed_out'
    }
    elseif ($null -ne $releaseError) {
        'cleanup_failed'
    }
    else {
        'exited'
    }
    $journalExitCode = if ($timedOut) { $null } else { $exitCode }
    Write-MvpProcessJournalEntry `
        -StageRoot $ProcessState.staged_product_root `
        -Phase $ProcessState.phase `
        -StartedAtUtc $ProcessState.started_at_utc `
        -EndedAtUtc $ProcessState.ended_at_utc `
        -ExitCode $journalExitCode `
        -Outcome $outcome
    if ($timedOut) {
        $cleanupDetail = if ($timeoutCleanupErrors.Count -eq 0) {
            ''
        }
        else {
            " Cleanup: $($timeoutCleanupErrors -join '; ')"
        }
        throw [TimeoutException]::new("Process did not exit within $TimeoutSeconds seconds.$cleanupDetail")
    }
    if ($null -ne $releaseError) {
        throw [InvalidOperationException]::new(
            "Process exited with code $exitCode. Cleanup: $($releaseError.Message)"
        )
    }
    return $exitCode
}

function Get-MvpRuntimeProductDiagnosticsEvidence {
    param([Parameter(Mandatory)][string]$DiagnosticText)

    $diagnostic = @(
        $DiagnosticText -split '\r?\n' |
            Where-Object { $_.IndexOf('runtime_product_frame_diagnostics ', [StringComparison]::Ordinal) -ge 0 }
    ) | Select-Object -Last 1
    if ([string]::IsNullOrWhiteSpace($diagnostic)) {
        throw 'Runtime product did not emit the runtime_product_frame_diagnostics diagnostic.'
    }

    $adapterMatch = [regex]::Match(
        $diagnostic,
        '(?:^|\s)render_adapter=(.*?)\s+render_adapter_type='
    )
    if (-not $adapterMatch.Success) {
        throw "Runtime product diagnostic is missing 'render_adapter': $diagnostic"
    }
    $adapter = $adapterMatch.Groups[1].Value.Trim()
    if ([string]::IsNullOrWhiteSpace($adapter) -or $adapter -eq 'unavailable') {
        throw "Runtime product diagnostic has no usable 'render_adapter' evidence: $diagnostic"
    }

    $fields = [ordered]@{}
    foreach ($name in @(
        'frame_index',
        'viewport',
        'project_identity',
        'scene_uri',
        'selected_model_resource_id',
        'selected_material_resource_id',
        'render_backend',
        'render_adapter_type',
        'device_max_bind_groups',
        'device_max_texture_dimension_2d',
        'device_max_texture_array_layers',
        'device_max_sampled_textures_per_shader_stage',
        'device_max_storage_buffers_per_shader_stage',
        'device_max_storage_buffer_binding_size',
        'graph_executed_pass_count',
        'mesh_draw_count',
        'directional_light_count',
        'material_fallback_count',
        'material_validation_error_count',
        'input_viewport_resize_count',
        'input_pointer_move_count',
        'input_mouse_button_press_count',
        'input_mouse_button_release_count',
        'input_keyboard_press_count',
        'input_keyboard_release_count'
    )) {
        $pattern = '(?:^|\s)' + [regex]::Escape($name) + '=([^\s]+)'
        $match = [regex]::Match($diagnostic, $pattern)
        if (-not $match.Success) {
            throw "Runtime product diagnostic is missing '$name': $diagnostic"
        }
        $value = $match.Groups[1].Value
        if ([string]::IsNullOrWhiteSpace($value) -or $value -eq 'unavailable') {
            throw "Runtime product diagnostic has no usable '$name' evidence: $diagnostic"
        }
        $fields[$name] = $value
    }
    $fields['render_adapter'] = $adapter

    if ($fields.viewport -notmatch '^[1-9][0-9]*x[1-9][0-9]*$') {
        throw "Runtime product diagnostic reports invalid viewport '$($fields.viewport)'."
    }
    [UInt64]$frameIndex = 0
    if (-not [UInt64]::TryParse($fields.frame_index, [ref]$frameIndex)) {
        throw "Runtime product diagnostic reports non-numeric frame_index '$($fields.frame_index)'."
    }
    foreach ($name in @(
        'device_max_bind_groups',
        'device_max_texture_dimension_2d',
        'device_max_texture_array_layers',
        'device_max_sampled_textures_per_shader_stage',
        'device_max_storage_buffers_per_shader_stage',
        'device_max_storage_buffer_binding_size',
        'graph_executed_pass_count',
        'mesh_draw_count',
        'directional_light_count'
    )) {
        [UInt64]$count = 0
        if (-not [UInt64]::TryParse($fields[$name], [ref]$count) -or $count -le 0) {
            throw "Runtime product diagnostic reports non-positive $name '$($fields[$name])'."
        }
    }
    foreach ($name in @('material_fallback_count', 'material_validation_error_count')) {
        [UInt64]$count = 0
        if (-not [UInt64]::TryParse($fields[$name], [ref]$count) -or $count -ne 0) {
            throw "Runtime product diagnostic reports $name '$($fields[$name])' instead of 0."
        }
    }
    foreach ($name in @(
        'input_viewport_resize_count',
        'input_pointer_move_count',
        'input_mouse_button_press_count',
        'input_mouse_button_release_count',
        'input_keyboard_press_count',
        'input_keyboard_release_count'
    )) {
        [UInt64]$count = 0
        if (-not [UInt64]::TryParse($fields[$name], [ref]$count) -or $count -le 0) {
            throw "Runtime product diagnostic reports non-positive $name '$($fields[$name])' after the requested host input probe."
        }
    }

    return $fields
}

function Get-MvpEditorProductDiagnosticsEvidence {
    param(
        [Parameter(Mandatory)][string]$DiagnosticText,
        [Parameter(Mandatory)][string]$StageRoot,
        [Parameter(Mandatory)][string]$ProjectRoot
    )

    $diagnostic = @(
        $DiagnosticText -split '\r?\n' |
            Where-Object { $_.IndexOf('editor_product_frame_diagnostics ', [StringComparison]::Ordinal) -ge 0 }
    ) | Select-Object -Last 1
    if ([string]::IsNullOrWhiteSpace($diagnostic)) {
        throw 'Editor product did not emit the editor_product_frame_diagnostics diagnostic.'
    }

    $fields = [ordered]@{}
    foreach ($name in @(
        'project_path',
        'selected_node_id',
        'selected_node_name',
        'inspector_translation_x',
        'inspector_translation_y',
        'inspector_translation_z',
        'inspector_scale_x',
        'inspector_scale_y',
        'inspector_scale_z'
    )) {
        $match = [regex]::Match($diagnostic, '(?:^|\s)' + [regex]::Escape($name) + '=([^\s]+)')
        if (-not $match.Success) {
            throw "Editor product diagnostic is missing '$name': $diagnostic"
        }
        $encoded = $match.Groups[1].Value
        if ($encoded -match '%(?![0-9A-Fa-f]{2})') {
            throw "Editor product diagnostic has malformed percent encoding for '$name': $encoded"
        }
        $decoded = [Uri]::UnescapeDataString($encoded)
        if ([string]::IsNullOrWhiteSpace($decoded)) {
            throw "Editor product diagnostic has an empty '$name' value."
        }
        $fields[$name] = $decoded
    }

    [UInt64]$selectedNodeId = 0
    if (-not [UInt64]::TryParse([string]$fields.selected_node_id, [ref]$selectedNodeId) -or $selectedNodeId -eq 0) {
        throw "Editor product diagnostic has invalid selected_node_id '$($fields.selected_node_id)'."
    }
    $reportedProjectPath = (Resolve-ZirconWindowsPath -Path ([string]$fields.project_path)).OperationalPath
    $expectedProjectPath = (Resolve-ZirconWindowsPath -Path $ProjectRoot).OperationalPath
    if (-not $reportedProjectPath.Equals($expectedProjectPath, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Editor product diagnostic project_path '$reportedProjectPath' differs from staged project '$expectedProjectPath'."
    }
    $fields.project_path = Get-MvpRelativePath -Root $StageRoot -Path $ProjectRoot -Label 'Editor product diagnostic project'
    $fields.selected_node_id = $selectedNodeId
    return $fields
}

function Invoke-MvpStagedProduct {
    param(
        [Parameter(Mandatory)][string]$Product,
        [Parameter(Mandatory)][string]$ExecutablePath,
        [Parameter(Mandatory)][string]$WorkingDirectory,
        [Parameter(Mandatory)][string]$StageRoot,
        [string]$ProjectRoot,
        [ValidateRange(0, [int]::MaxValue)]
        [int]$AttemptOffset = 0,
        [ValidateRange(1, 4)]
        [int]$RunCount = $RepeatCount,
        [string]$EditorWindowCaptureName
    )

    if (-not [string]::IsNullOrWhiteSpace($EditorWindowCaptureName)) {
        if ($Product -ne 'editor') {
            throw "EditorWindowCaptureName is only valid for the editor product, not '$Product'."
        }
        if ([IO.Path]::GetFileName($EditorWindowCaptureName) -ne $EditorWindowCaptureName -or
            -not $EditorWindowCaptureName.EndsWith('.png', [StringComparison]::OrdinalIgnoreCase)) {
            throw "EditorWindowCaptureName '$EditorWindowCaptureName' must be a PNG file name without a directory."
        }
    }

    $exitFlag = switch ($Product) {
        'runtime' { 'ZIRCON_RUNTIME_EXIT_AFTER_FIRST_FRAME' }
        'editor' { 'ZIRCON_EDITOR_EXIT_AFTER_FIRST_FRAME' }
        default { throw "Unsupported staged MVP product '$Product'." }
    }
    $firstFrameDiagnostic = switch ($Product) {
        'runtime' { 'runtime_first_frame_presented' }
        'editor' { 'editor_first_frame_presented' }
        default { throw "Unsupported staged MVP product '$Product'." }
    }
    $teardownDiagnostic = switch ($Product) {
        'runtime' { 'runtime_process_teardown_complete' }
        'editor' { 'editor_process_teardown_complete' }
        default { throw "Unsupported staged MVP product '$Product'." }
    }
    $results = [System.Collections.Generic.List[object]]::new()
    $logDirectory = Join-Path $StageRoot 'logs'
    $captureDirectory = Join-Path $StageRoot 'captures'
    New-Item -ItemType Directory -Force -Path $logDirectory | Out-Null
    for ($runIndex = 1; $runIndex -le $RunCount; $runIndex++) {
        $attempt = $AttemptOffset + $runIndex
        $stdout = Join-Path $logDirectory "$Product-$attempt.stdout.log"
        $stderr = Join-Path $logDirectory "$Product-$attempt.stderr.log"
        $diagnosticRoot = Join-Path $logDirectory "$Product-$attempt.diagnostics"
        $frameCapturePath = if ($Product -eq 'runtime' -and -not [string]::IsNullOrWhiteSpace($ProjectRoot)) {
            New-Item -ItemType Directory -Force -Path $captureDirectory | Out-Null
            Join-Path $captureDirectory "$Product-$attempt.png"
        } else {
            $null
        }
        $editorWindowCapturePath = if ($Product -eq 'editor' -and -not [string]::IsNullOrWhiteSpace($EditorWindowCaptureName)) {
            New-Item -ItemType Directory -Force -Path $captureDirectory | Out-Null
            Join-Path $captureDirectory $EditorWindowCaptureName
        } else {
            $null
        }
        $environment = @{
            $exitFlag = '1'
            ZIRCON_RUNTIME_LIBRARY = ''
            ZIRCON_ASSET_ROOT = (Join-Path $WorkingDirectory 'assets')
            ZIRCON_LOG_ROOT = $diagnosticRoot
            # This wins over a host RUST_LOG override so first-frame evidence is durable.
            ZIRCON_LOG_FILTER = 'log'
        }
        if ($Product -eq 'runtime') {
            $environment.ZIRCON_RUNTIME_MVP_INPUT_PROBE = '1'
        }
        if ($null -ne $frameCapturePath) {
            $environment.ZIRCON_RUNTIME_CAPTURE_FRAME_PNG = $frameCapturePath
        }
        if ($null -ne $editorWindowCapturePath) {
            $environment.ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG = $editorWindowCapturePath
        }
        $started = [Diagnostics.Stopwatch]::StartNew()
        $processState = $null
        $exitCode = $null
        try {
            $processState = Start-MvpStagedProcess `
                -ExecutablePath $ExecutablePath `
                -WorkingDirectory $WorkingDirectory `
                -Environment $environment `
                -StageRoot $StageRoot `
                -Phase "$Product-$attempt" `
                -ProjectRoot $ProjectRoot
        }
        catch {
            $started.Stop()
            throw "Staged $Product attempt $attempt could not launch from '$ExecutablePath' in '$WorkingDirectory': $($_.Exception.Message)"
        }
        try {
            $exitCode = Complete-MvpStagedProcess `
                -ProcessState $processState `
                -StdoutPath $stdout `
                -StderrPath $stderr `
                -TimeoutSeconds $TimeoutSeconds
        }
        catch [TimeoutException] {
            throw "Staged $Product attempt $attempt did not exit within $TimeoutSeconds seconds."
        }
        catch {
            throw "Staged $Product attempt $attempt could not collect process output: $($_.Exception.Message)"
        }
        finally {
            $started.Stop()
            if ($null -ne $processState) {
                $processState.process.Dispose()
            }
        }
        $failureMessage = if ($exitCode -ne 0) {
            "Staged $Product attempt $attempt exited with code $exitCode. See $stdout and $stderr."
        }
        else {
            $null
        }
        try {
            Assert-MvpStagingProcessesReleased -StageDirectory $StageRoot
            if (-not [string]::IsNullOrWhiteSpace($ProjectRoot)) {
                Test-MvpStagedProjectDirectoryReleased `
                    -StageDirectory $StageRoot `
                    -ProjectDirectory $ProjectRoot
            }
        }
        catch {
            if ($null -ne $failureMessage) {
                throw "$failureMessage Cleanup: $($_.Exception.Message)"
            }
            throw
        }
        if ($null -ne $failureMessage) {
            throw $failureMessage
        }
        $diagnosticFiles = @(Get-ChildItem -LiteralPath $diagnosticRoot -Recurse -File -Filter '*.log' -ErrorAction SilentlyContinue | Sort-Object FullName)
        $diagnosticText = ($diagnosticFiles | ForEach-Object { [IO.File]::ReadAllText($_.FullName) }) -join "`n"
        if ($diagnosticText.IndexOf($firstFrameDiagnostic, [StringComparison]::Ordinal) -lt 0) {
            throw "Staged $Product attempt $attempt exited without the $firstFrameDiagnostic diagnostic under '$diagnosticRoot'. See $stdout and $stderr."
        }
        if ($diagnosticText.IndexOf($teardownDiagnostic, [StringComparison]::Ordinal) -lt 0) {
            throw "Staged $Product attempt $attempt exited without the $teardownDiagnostic diagnostic under '$diagnosticRoot'. See $stdout and $stderr."
        }
        $frameCapture = $null
        $editorWindowCapture = $null
        $editorProductDiagnostics = $null
        $runtimeProductDiagnostics = $null
        if ($null -ne $frameCapturePath) {
            if ($diagnosticText.IndexOf('runtime_product_frame_capture_written', [StringComparison]::Ordinal) -lt 0) {
                throw "Staged runtime attempt $attempt exited without the runtime_product_frame_capture_written diagnostic under '$diagnosticRoot'. See $stdout and $stderr."
            }
            $frameCapture = Get-MvpRuntimeFrameCaptureEvidence -Path $frameCapturePath -StageRoot $StageRoot
            $runtimeProductDiagnostics = Get-MvpRuntimeProductDiagnosticsEvidence -DiagnosticText $diagnosticText
        }
        if ($null -ne $editorWindowCapturePath) {
            if ($diagnosticText.IndexOf('editor_product_frame_capture_written', [StringComparison]::Ordinal) -lt 0) {
                throw "Staged editor attempt $attempt exited without the editor_product_frame_capture_written diagnostic under '$diagnosticRoot'. See $stdout and $stderr."
            }
            $editorWindowCapture = Get-MvpEditorWindowCaptureEvidence -Path $editorWindowCapturePath -StageRoot $StageRoot
            $editorProductDiagnostics = Get-MvpEditorProductDiagnosticsEvidence `
                -DiagnosticText $diagnosticText `
                -StageRoot $StageRoot `
                -ProjectRoot $ProjectRoot
        }
        $results.Add([ordered]@{
            product = $Product
            attempt = $attempt
            exit_code = $exitCode
            started_at_utc = $processState.started_at_utc
            ended_at_utc = $processState.ended_at_utc
            first_frame_exit_requested = $true
            first_frame_presented = $true
            teardown_complete = $true
            elapsed_milliseconds = [int][Math]::Round($started.Elapsed.TotalMilliseconds)
            stdout = Get-MvpStagedFileEvidence -Path $stdout -StageRoot $StageRoot -Label 'Product stdout log'
            stderr = Get-MvpStagedFileEvidence -Path $stderr -StageRoot $StageRoot -Label 'Product stderr log'
            diagnostic_logs = @($diagnosticFiles | ForEach-Object { Get-MvpStagedFileEvidence -Path $_.FullName -StageRoot $StageRoot -Label 'Product diagnostic log' })
            frame_capture = $frameCapture
            editor_window_capture = $editorWindowCapture
            editor_product_diagnostics = $editorProductDiagnostics
            runtime_product_diagnostics = $runtimeProductDiagnostics
            project = if ([string]::IsNullOrWhiteSpace($ProjectRoot)) { $null } else { Get-MvpRelativePath -Root $StageRoot -Path $ProjectRoot -Label 'Staged project' }
        }) | Out-Null
    }
    return $results.ToArray()
}

function Get-MvpAuthoringAutomationEvidence {
    param(
        [Parameter(Mandatory)][string]$StdoutPath,
        [Parameter(Mandatory)][string]$StageRoot,
        [Parameter(Mandatory)][string]$AutomationRequestPath,
        [Parameter(Mandatory)][string]$StderrPath,
        [Parameter(Mandatory)][string]$DiagnosticRoot,
        [Parameter(Mandatory)][string]$ProjectRoot
    )

    $stdout = Get-Content -LiteralPath $StdoutPath -Raw -ErrorAction Stop
    if ([string]::IsNullOrWhiteSpace($stdout)) {
        throw "Staged editor authoring automation did not emit a structured report. See $StdoutPath and $StderrPath."
    }
    try {
        $reports = @($stdout | ConvertFrom-Json -ErrorAction Stop)
    }
    catch {
        throw "Staged editor authoring automation emitted invalid JSON. See $StdoutPath and ${StderrPath}: $($_.Exception.Message)"
    }
    if ($reports.Count -ne 1) {
        throw "Staged editor authoring automation must emit exactly one report; found $($reports.Count). See $StdoutPath and $StderrPath."
    }

    $report = $reports[0]
    foreach ($propertyName in @(
        'project_path',
        'project_identity',
        'manifest_identity',
        'scene_uri',
        'selected_model_resource_id',
        'selected_material_resource_id',
        'opened_project_inspection_generation',
        'records'
    )) {
        $property = $report.PSObject.Properties[$propertyName]
        if ($null -eq $property -or $null -eq $property.Value) {
            throw "Staged editor authoring automation report is missing '$propertyName'. See $StdoutPath and $StderrPath."
        }
    }
    if (@($report.records).Count -eq 0) {
        throw "Staged editor authoring automation report contains no binding records. See $StdoutPath and $StderrPath."
    }
    $reportedProjectPath = [string]$report.project_path
    if ([string]::IsNullOrWhiteSpace($reportedProjectPath)) {
        throw "Staged editor authoring automation report has an empty project_path. See $StdoutPath and $StderrPath."
    }
    $expectedProjectPath = (Resolve-ZirconWindowsPath -Path $ProjectRoot).OperationalPath
    try {
        $resolvedReportedProjectPath = (Resolve-ZirconWindowsPath -Path $reportedProjectPath).OperationalPath
    }
    catch {
        throw "Staged editor authoring automation report has an invalid project_path '$reportedProjectPath'. See $StdoutPath and $StderrPath."
    }
    if (-not $resolvedReportedProjectPath.Equals($expectedProjectPath, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Staged editor authoring automation report project_path '$reportedProjectPath' differs from staged project '$expectedProjectPath'. See $StdoutPath and $StderrPath."
    }
    $report.project_path = Get-MvpRelativePath -Root $StageRoot -Path $ProjectRoot -Label 'Authoring automation project'
    # Retain structured child output without leaking the machine-specific absolute project root
    # into a portable CI artifact. The parsed report still records the normal binding sequence.
    Write-MvpJson -Path $StdoutPath -Value $report

    $diagnosticFiles = @(
        Get-ChildItem -LiteralPath $DiagnosticRoot -Recurse -File -ErrorAction SilentlyContinue |
            Sort-Object FullName
    )
    if ($diagnosticFiles.Count -eq 0) {
        throw "Staged editor authoring automation did not emit diagnostic log evidence under '$DiagnosticRoot'."
    }
    $report | Add-Member -NotePropertyName 'automation_request' -NotePropertyValue (Get-MvpStagedFileEvidence -Path $AutomationRequestPath -StageRoot $StageRoot -Label 'Authoring automation request')
    $report | Add-Member -NotePropertyName 'stdout' -NotePropertyValue (Get-MvpStagedFileEvidence -Path $StdoutPath -StageRoot $StageRoot -Label 'Authoring automation stdout log')
    $report | Add-Member -NotePropertyName 'stderr' -NotePropertyValue (Get-MvpStagedFileEvidence -Path $StderrPath -StageRoot $StageRoot -Label 'Authoring automation stderr log')
    $report | Add-Member -NotePropertyName 'diagnostic_logs' -NotePropertyValue @($diagnosticFiles | ForEach-Object { Get-MvpStagedFileEvidence -Path $_.FullName -StageRoot $StageRoot -Label 'Authoring automation diagnostic log' })
    return $report
}

function Invoke-MvpStagedAuthoringAutomation {
    param(
        [Parameter(Mandatory)][string]$ExecutablePath,
        [Parameter(Mandatory)][string]$WorkingDirectory,
        [Parameter(Mandatory)][string]$StageRoot,
        [Parameter(Mandatory)][string]$ProjectRoot,
        [Parameter(Mandatory)][string]$AutomationRequestPath,
        [Parameter(Mandatory)][string]$EvidenceLabel
    )

    $logDirectory = Join-Path $StageRoot 'logs'
    New-Item -ItemType Directory -Force -Path $logDirectory | Out-Null
    $stdout = Join-Path $logDirectory "$EvidenceLabel.stdout.log"
    $stderr = Join-Path $logDirectory "$EvidenceLabel.stderr.log"
    $environment = @{
        ZIRCON_RUNTIME_LIBRARY = ''
        ZIRCON_ASSET_ROOT = (Join-Path $WorkingDirectory 'assets')
        ZIRCON_LOG_ROOT = (Join-Path $logDirectory "$EvidenceLabel.diagnostics")
        ZIRCON_LOG_FILTER = 'log'
    }
    $started = [Diagnostics.Stopwatch]::StartNew()
    $processState = $null
    $projectRootArgument = (Resolve-ZirconWindowsPath -Path $ProjectRoot).DisplayPath
    try {
        $processState = Start-MvpStagedProcess `
            -ExecutablePath $ExecutablePath `
            -WorkingDirectory $WorkingDirectory `
            -Environment $environment `
            -StageRoot $StageRoot `
            -Phase $EvidenceLabel `
            -Arguments @('--project', $projectRootArgument, '--automation', $AutomationRequestPath, '--headless')
        $exitCode = Complete-MvpStagedProcess `
            -ProcessState $processState `
            -StdoutPath $stdout `
            -StderrPath $stderr `
            -TimeoutSeconds $TimeoutSeconds
    }
    catch [TimeoutException] {
        throw "Staged editor $EvidenceLabel automation did not exit within $TimeoutSeconds seconds."
    }
    catch {
        throw "Staged editor $EvidenceLabel automation could not launch or collect output: $($_.Exception.Message)"
    }
    finally {
        $started.Stop()
        if ($null -ne $processState) {
            $processState.process.Dispose()
        }
    }
    Assert-MvpStagingProcessesReleased -StageDirectory $StageRoot
    Test-MvpStagedProjectDirectoryReleased `
        -StageDirectory $StageRoot `
        -ProjectDirectory $ProjectRoot
    if ($exitCode -ne 0) {
        throw "Staged editor $EvidenceLabel automation exited with code $exitCode. See $stdout and $stderr."
    }
    $report = Get-MvpAuthoringAutomationEvidence `
        -StdoutPath $stdout `
        -StageRoot $StageRoot `
        -AutomationRequestPath $AutomationRequestPath `
        -StderrPath $stderr `
        -DiagnosticRoot $environment.ZIRCON_LOG_ROOT `
        -ProjectRoot $ProjectRoot
    $report | Add-Member -NotePropertyName 'exit_code' -NotePropertyValue $exitCode
    $report | Add-Member -NotePropertyName 'started_at_utc' -NotePropertyValue $processState.started_at_utc
    $report | Add-Member -NotePropertyName 'ended_at_utc' -NotePropertyValue $processState.ended_at_utc
    $report | Add-Member -NotePropertyName 'elapsed_milliseconds' -NotePropertyValue ([int][Math]::Round($started.Elapsed.TotalMilliseconds))
    return $report
}

function Assert-MvpStagingProcessesReleased {
    param([Parameter(Mandatory)][string]$StageDirectory)

    $lingering = @(Get-MvpStagedProcesses -StageDirectory $StageDirectory)
    if ($lingering.Count -eq 0) {
        return
    }

    $details = $lingering | ForEach-Object {
        "pid=$($_.ProcessId) name=$($_.Name) path=$($_.ExecutablePath)"
    }
    $remaining = @(Stop-MvpStagedProcesses -StageDirectory $StageDirectory)
    if ($remaining.Count -eq 0) {
        throw "Staged executable process(es) remain after product exit and were terminated: $($details -join '; ')."
    }

    $remainingDetails = $remaining | ForEach-Object {
        "pid=$($_.ProcessId) name=$($_.Name) path=$($_.ExecutablePath)"
    }
    throw "Staged executable process(es) remain after product exit after cleanup: $($remainingDetails -join '; ')."
}

function Test-MvpStagingDirectoryReleased {
    param([Parameter(Mandatory)][string]$StageDirectory)

    Assert-MvpStagingProcessesReleased -StageDirectory $StageDirectory

    $probe = "$StageDirectory.release-probe"
    if (Test-Path -LiteralPath $probe) {
        throw "Staging release probe '$probe' already exists."
    }
    Move-Item -LiteralPath $StageDirectory -Destination $probe -ErrorAction Stop
    Move-Item -LiteralPath $probe -Destination $StageDirectory -ErrorAction Stop
}

function Invoke-MvpProductStaging {
    $runtimeExecutablePath = Resolve-MvpInputFile -Path $RuntimeExecutable -Label 'RuntimeExecutable'
    $editorExecutablePath = Resolve-MvpInputFile -Path $EditorExecutable -Label 'EditorExecutable'
    $runtimeLibraryPath = Resolve-MvpInputFile -Path $RuntimeLibrary -Label 'RuntimeLibrary'
    $editorRuntimeLibraryPath = Resolve-MvpInputFile -Path $EditorRuntimeLibrary -Label 'EditorRuntimeLibrary'
    Assert-MvpDistinctProfileRuntimeLibraries `
        -RuntimeLibraryPath $runtimeLibraryPath `
        -EditorRuntimeLibraryPath $editorRuntimeLibraryPath
    $templateRootPath = Resolve-MvpInputDirectory -Path $TemplateRoot -Label 'TemplateRoot'
    $engineAssetRootPath = Resolve-MvpInputDirectory -Path $EngineAssetRoot -Label 'EngineAssetRoot'
    $projectRootPath = if ([string]::IsNullOrWhiteSpace($ProjectRoot)) {
        $null
    } else {
        Resolve-MvpInputDirectory -Path $ProjectRoot -Label 'ProjectRoot'
    }
    $authoringAutomationRequestPath = if ([string]::IsNullOrWhiteSpace($AuthoringAutomationRequest)) {
        $null
    }
    else {
        Resolve-MvpInputFile -Path $AuthoringAutomationRequest -Label 'AuthoringAutomationRequest'
    }
    $reopenAutomationRequestPath = if ([string]::IsNullOrWhiteSpace($ReopenAutomationRequest)) {
        $null
    }
    else {
        Resolve-MvpInputFile -Path $ReopenAutomationRequest -Label 'ReopenAutomationRequest'
    }
    if ($CreateProject -and $null -ne $projectRootPath) {
        throw 'CreateProject cannot be combined with ProjectRoot; the staged editor must create the canonical project.'
    }
    if ($CreateProject) {
        Assert-MvpProjectName -Value $ProjectName
    }
    if ($CreateProject -and $NoLaunch) {
        throw 'CreateProject cannot be combined with NoLaunch; a staged editor launch is required to create the canonical project.'
    }
    if ($null -ne $authoringAutomationRequestPath -and $NoLaunch) {
        throw 'AuthoringAutomationRequest cannot be combined with NoLaunch; a staged headless editor launch is required.'
    }
    if ($null -ne $authoringAutomationRequestPath -and $null -eq $projectRootPath -and -not $CreateProject) {
        throw 'AuthoringAutomationRequest requires ProjectRoot or CreateProject so the staged editor can open one canonical project.'
    }
    if ($null -ne $reopenAutomationRequestPath -and $null -eq $authoringAutomationRequestPath) {
        throw 'ReopenAutomationRequest requires AuthoringAutomationRequest so persisted state has a source-bound authoring predecessor.'
    }
    if ($null -ne $reopenAutomationRequestPath -and ($RepeatCount -ne 2 -or $ReopenRepeatCount -ne 2)) {
        throw 'ReopenAutomationRequest requires RepeatCount and ReopenRepeatCount to both equal 2 for the fixed F5 evidence sequence.'
    }
    $stagingRootPath = Resolve-MvpStagingRoot -Path $StagingRoot
    Assert-MvpRunId -Value $RunId
    if ([string]::IsNullOrWhiteSpace($SourceFingerprint)) {
        $SourceFingerprint = Get-MvpSourceFingerprint
    }
    $validationMetadata = Resolve-MvpValidationMetadata
    $engineAssetFiles = @(Get-ChildItem -LiteralPath $engineAssetRootPath -Recurse -File | Sort-Object FullName)
    if ($engineAssetFiles.Count -eq 0) {
        throw "EngineAssetRoot '$engineAssetRootPath' has no files to stage."
    }
    $templateFiles = @(Get-ChildItem -LiteralPath $templateRootPath -Recurse -File | Sort-Object FullName)
    if ($templateFiles.Count -eq 0) {
        throw "TemplateRoot '$templateRootPath' has no files to stage."
    }
    $projectFiles = if ($null -eq $projectRootPath) {
        @()
    }
    else {
        @(Get-ChildItem -LiteralPath $projectRootPath -Recurse -File | Sort-Object FullName | Where-Object {
            $relative = Get-MvpRelativePath -Root $projectRootPath -Path $_.FullName -Label 'Project file'
            Test-MvpProjectSourceRelativePath -RelativePath $relative
        })
    }
    if ($null -ne $projectRootPath -and $projectFiles.Count -eq 0) {
        throw "ProjectRoot '$projectRootPath' has no source files to stage."
    }
    $inputCopies = [System.Collections.Generic.List[object]]::new()
    foreach ($path in @(
        $runtimeExecutablePath,
        $editorExecutablePath,
        $runtimeLibraryPath,
        $editorRuntimeLibraryPath
    )) {
        $inputCopies.Add([ordered]@{ path = $path; copy_count = 1 }) | Out-Null
    }
    foreach ($file in $engineAssetFiles) {
        $inputCopies.Add([ordered]@{ path = $file.FullName; copy_count = 2 }) | Out-Null
    }
    foreach ($file in @($templateFiles) + @($projectFiles)) {
        $inputCopies.Add([ordered]@{ path = $file.FullName; copy_count = 1 }) | Out-Null
    }
    foreach ($path in @($authoringAutomationRequestPath, $reopenAutomationRequestPath)) {
        if ($null -ne $path) {
            $inputCopies.Add([ordered]@{ path = $path; copy_count = 1 }) | Out-Null
        }
    }
    $preflight = Get-MvpStagingPreflight `
        -StagingRootPath $stagingRootPath `
        -InputCopies ($inputCopies.ToArray()) `
        -InteractiveDesktopRequired (-not $NoLaunch)

    $stageDirectory = Join-Path $stagingRootPath $RunId
    $partialDirectory = "$stageDirectory.partial-$([guid]::NewGuid().ToString('N'))"
    if (Test-Path -LiteralPath $stageDirectory) {
        throw "MVP staging run '$RunId' already exists at '$stageDirectory'; choose a new RunId rather than overwriting a validation run."
    }
    if (Test-Path -LiteralPath $partialDirectory) {
        throw "MVP staging temporary directory '$partialDirectory' already exists."
    }
    $stagedProjectRoot = if ($null -eq $projectRootPath) { $null } else { Join-Path $stageDirectory 'project' }
    $stagedAuthoringAutomationPath = if ($null -eq $authoringAutomationRequestPath) { $null } else { Join-Path $stageDirectory 'authoring\automation.json' }
    $stagedReopenAutomationPath = if ($null -eq $reopenAutomationRequestPath) { $null } else { Join-Path $stageDirectory 'reopen\automation.json' }

    try {
        New-Item -ItemType Directory -Force -Path $partialDirectory | Out-Null
        $entries = [System.Collections.Generic.List[object]]::new()
        $entries.Add((Copy-MvpStageFile -LogicalId 'runtime-executable' -SourcePath $runtimeExecutablePath -StageRoot $partialDirectory -TargetRelativePath 'runtime\zircon_runtime.exe')) | Out-Null
        $entries.Add((Copy-MvpStageFile -LogicalId 'runtime-library/runtime' -SourcePath $runtimeLibraryPath -StageRoot $partialDirectory -TargetRelativePath 'runtime\zircon_runtime.dll')) | Out-Null
        $entries.Add((Copy-MvpStageFile -LogicalId 'editor-executable' -SourcePath $editorExecutablePath -StageRoot $partialDirectory -TargetRelativePath 'editor\zircon_editor.exe')) | Out-Null
        $entries.Add((Copy-MvpStageFile -LogicalId 'runtime-library/editor' -SourcePath $editorRuntimeLibraryPath -StageRoot $partialDirectory -TargetRelativePath 'editor\zircon_runtime.dll')) | Out-Null

        foreach ($engineAssetFile in $engineAssetFiles) {
            $relative = Get-MvpRelativePath -Root $engineAssetRootPath -Path $engineAssetFile.FullName -Label 'Engine asset'
            foreach ($product in @('runtime', 'editor')) {
                $entries.Add((Copy-MvpStageFile `
                    -LogicalId ('engine-asset/' + $product + '/' + $relative) `
                    -SourcePath $engineAssetFile.FullName `
                    -StageRoot $partialDirectory `
                    -TargetRelativePath ($product + '\assets\' + $relative.Replace('/', '\')))) | Out-Null
            }
        }

        foreach ($templateFile in $templateFiles) {
            $relative = Get-MvpRelativePath -Root $templateRootPath -Path $templateFile.FullName
            $entries.Add((Copy-MvpStageFile `
                -LogicalId ('template/' + $relative) `
                -SourcePath $templateFile.FullName `
                -StageRoot $partialDirectory `
                -TargetRelativePath ('templates\' + $relative.Replace('/', '\')))) | Out-Null
        }
        if ($null -ne $authoringAutomationRequestPath) {
            $entries.Add((Copy-MvpStageFile `
                -LogicalId 'authoring-automation-request' `
                -SourcePath $authoringAutomationRequestPath `
                -StageRoot $partialDirectory `
                -TargetRelativePath 'authoring\automation.json')) | Out-Null
        }
        if ($null -ne $reopenAutomationRequestPath) {
            $entries.Add((Copy-MvpStageFile `
                -LogicalId 'reopen-automation-request' `
                -SourcePath $reopenAutomationRequestPath `
                -StageRoot $partialDirectory `
                -TargetRelativePath 'reopen\automation.json')) | Out-Null
        }
        if ($null -ne $projectRootPath) {
            foreach ($projectFile in $projectFiles) {
                $relative = Get-MvpRelativePath -Root $projectRootPath -Path $projectFile.FullName -Label 'Project file'
                $entries.Add((Copy-MvpStageFile `
                    -LogicalId ('project/' + $relative) `
                    -SourcePath $projectFile.FullName `
                    -StageRoot $partialDirectory `
                    -TargetRelativePath ('project\' + $relative.Replace('/', '\')))) | Out-Null
            }
        }

        $null = Assert-MvpStagingEntryBudget `
            -Entries ($entries.ToArray()) `
            -ExpectedInputCopyBytes ([Int64]$preflight.input_copy_bytes)

        $manifest = [ordered]@{
            schema_version = 1
            run_id = $RunId
            source_fingerprint = $SourceFingerprint
            toolchain = $validationMetadata.toolchain
            target = $validationMetadata.target
            staged_at_utc = [DateTimeOffset]::UtcNow.ToString('o')
            preflight = $preflight
            entries = $entries.ToArray()
        }
        Write-MvpJson -Path (Join-Path $partialDirectory 'staging-manifest.json') -Value $manifest
        Move-Item -LiteralPath $partialDirectory -Destination $stageDirectory -ErrorAction Stop
    }
    catch {
        if (Test-Path -LiteralPath $partialDirectory) {
            Remove-Item -LiteralPath $partialDirectory -Recurse -Force -ErrorAction SilentlyContinue
        }
        throw
    }

    $productRuns = @()
    $projectCreation = $null
    $baselineAutomation = $null
    $authoringAutomation = $null
    $reopenAutomation = @()
    try {
        if (-not $NoLaunch) {
            $stagedProjectRoot = if ($null -eq $projectRootPath) { $null } else { Join-Path $stageDirectory 'project' }
            if ($CreateProject) {
                $createLogDirectory = Join-Path $stageDirectory 'logs'
                $createDiagnosticRoot = Join-Path $createLogDirectory 'editor-create.diagnostics'
                $createStdout = Join-Path $createLogDirectory 'editor-create.stdout.log'
                $createStderr = Join-Path $createLogDirectory 'editor-create.stderr.log'
                $createEditorWindowCapturePath = Join-Path $stageDirectory 'captures\editor-before-edit.png'
                New-Item -ItemType Directory -Force -Path $createLogDirectory | Out-Null
                $createEnvironment = @{
                    ZIRCON_EDITOR_EXIT_AFTER_FIRST_FRAME = '1'
                    ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG = $createEditorWindowCapturePath
                    ZIRCON_RUNTIME_LIBRARY = ''
                    ZIRCON_ASSET_ROOT = (Join-Path $stageDirectory 'editor\assets')
                    ZIRCON_LOG_ROOT = $createDiagnosticRoot
                    ZIRCON_LOG_FILTER = 'log'
                }
                $createStarted = [Diagnostics.Stopwatch]::StartNew()
                $createProcess = $null
                $createExitCode = $null
                try {
                    $createProcess = Start-MvpStagedProcess `
                        -ExecutablePath (Join-Path $stageDirectory 'editor\zircon_editor.exe') `
                        -WorkingDirectory (Join-Path $stageDirectory 'editor') `
                        -Environment $createEnvironment `
                        -StageRoot $stageDirectory `
                        -Phase 'editor-create' `
                        -Arguments @('--create-project', '--project-name', $ProjectName, '--location', (Join-Path $stageDirectory 'project'), '--template', 'renderable-empty')
                    $createExitCode = Complete-MvpStagedProcess `
                        -ProcessState $createProcess `
                        -StdoutPath $createStdout `
                        -StderrPath $createStderr `
                        -TimeoutSeconds $TimeoutSeconds
                }
                finally {
                    $createStarted.Stop()
                    if ($null -ne $createProcess) {
                        $createProcess.process.Dispose()
                    }
                }
                Assert-MvpStagingProcessesReleased -StageDirectory $stageDirectory
                if ($createExitCode -ne 0) {
                    throw "Staged editor project creation failed with exit code $createExitCode."
                }
                $createDiagnosticFiles = @(
                    Get-ChildItem -LiteralPath $createDiagnosticRoot -Recurse -File -ErrorAction SilentlyContinue |
                        Sort-Object FullName
                )
                if ($createDiagnosticFiles.Count -eq 0) {
                    throw "Staged editor project creation emitted no diagnostic log evidence under '$createDiagnosticRoot'."
                }
                $createDiagnosticText = ($createDiagnosticFiles | ForEach-Object {
                    Get-Content -LiteralPath $_.FullName -Raw -ErrorAction Stop
                }) -join [Environment]::NewLine
                foreach ($diagnostic in @(
                    'editor_first_frame_presented',
                    'editor_process_teardown_complete',
                    'editor_product_frame_capture_written'
                )) {
                    if ($createDiagnosticText.IndexOf($diagnostic, [StringComparison]::Ordinal) -lt 0) {
                        throw "Staged editor project creation exited without the $diagnostic diagnostic under '$createDiagnosticRoot'. See $createStdout and $createStderr."
                    }
                }
                $createdProjectParentResolution = Resolve-ZirconWindowsPath -Path (Join-Path $stageDirectory 'project')
                $createdProjectExpectedResolution = Resolve-ZirconWindowsPath -Path (Join-ZirconWindowsPath `
                    -Path $createdProjectParentResolution.OperationalPath `
                    -ChildPath $ProjectName)
                $createdProjectParent = $createdProjectParentResolution.OperationalPath
                $createdProjectExpectedRoot = $createdProjectExpectedResolution.OperationalPath
                $createdProjectParentPrefix = $createdProjectParent.TrimEnd([char[]]@('\', '/')) + [IO.Path]::DirectorySeparatorChar
                if (-not $createdProjectExpectedRoot.StartsWith($createdProjectParentPrefix, [StringComparison]::OrdinalIgnoreCase)) {
                    throw "Created project target '$($createdProjectExpectedResolution.DisplayPath)' escapes staged project root '$($createdProjectParentResolution.DisplayPath)'."
                }
                $createdProjectRoot = Resolve-MvpInputDirectory -Path $createdProjectExpectedResolution.DisplayPath -Label 'Staged created project'
                if (-not $createdProjectRoot.Equals($createdProjectExpectedRoot, [StringComparison]::OrdinalIgnoreCase)) {
                    throw "Created project root '$createdProjectRoot' differs from expected staged target '$($createdProjectExpectedResolution.DisplayPath)'."
                }
                Test-MvpStagedProjectDirectoryReleased `
                    -StageDirectory $stageDirectory `
                    -ProjectDirectory $createdProjectRoot
                $projectOpenEvidence = Get-MvpEditorProjectOpenEvidence `
                    -DiagnosticText $createDiagnosticText `
                    -StagingRoot $stageDirectory `
                    -ProjectRoot $createdProjectRoot
                $createEditorWindowCapture = Get-MvpEditorWindowCaptureEvidence `
                    -Path $createEditorWindowCapturePath `
                    -StageRoot $stageDirectory
                $createEditorProductDiagnostics = Get-MvpEditorProductDiagnosticsEvidence `
                    -DiagnosticText $createDiagnosticText `
                    -StageRoot $stageDirectory `
                    -ProjectRoot $createdProjectRoot
                $projectCreation = [ordered]@{
                    exit_code = $createExitCode
                    started_at_utc = $createProcess.started_at_utc
                    ended_at_utc = $createProcess.ended_at_utc
                    first_frame_presented = $true
                    teardown_complete = $true
                    elapsed_milliseconds = [int][Math]::Round($createStarted.Elapsed.TotalMilliseconds)
                    stdout = Get-MvpStagedFileEvidence -Path $createStdout -StageRoot $stageDirectory -Label 'Project creation stdout log'
                    stderr = Get-MvpStagedFileEvidence -Path $createStderr -StageRoot $stageDirectory -Label 'Project creation stderr log'
                    diagnostic_logs = @($createDiagnosticFiles | ForEach-Object { Get-MvpStagedFileEvidence -Path $_.FullName -StageRoot $stageDirectory -Label 'Project creation diagnostic log' })
                    editor_window_capture = $createEditorWindowCapture
                    editor_product_diagnostics = $createEditorProductDiagnostics
                    project_open = $projectOpenEvidence
                }
                $stagedProjectRoot = $createdProjectRoot
            }
            if ($null -ne $stagedReopenAutomationPath) {
                $baselineAutomation = Invoke-MvpStagedAuthoringAutomation `
                    -ExecutablePath (Join-Path $stageDirectory 'editor\zircon_editor.exe') `
                    -WorkingDirectory (Join-Path $stageDirectory 'editor') `
                    -StageRoot $stageDirectory `
                    -ProjectRoot $stagedProjectRoot `
                    -AutomationRequestPath $stagedReopenAutomationPath `
                    -EvidenceLabel 'editor-baseline'
                Assert-MvpStagingProcessesReleased -StageDirectory $stageDirectory
            }
            $productRuns += Invoke-MvpStagedProduct `
                -Product 'runtime' `
                -ExecutablePath (Join-Path $stageDirectory 'runtime\zircon_runtime.exe') `
                -WorkingDirectory (Join-Path $stageDirectory 'runtime') `
                -StageRoot $stageDirectory `
                -ProjectRoot $stagedProjectRoot
            Assert-MvpStagingProcessesReleased -StageDirectory $stageDirectory
            if ($null -ne $stagedAuthoringAutomationPath) {
                $authoringAutomation = Invoke-MvpStagedAuthoringAutomation `
                    -ExecutablePath (Join-Path $stageDirectory 'editor\zircon_editor.exe') `
                    -WorkingDirectory (Join-Path $stageDirectory 'editor') `
                    -StageRoot $stageDirectory `
                    -ProjectRoot $stagedProjectRoot `
                    -AutomationRequestPath $stagedAuthoringAutomationPath `
                    -EvidenceLabel 'editor-authoring'
                Assert-MvpStagingProcessesReleased -StageDirectory $stageDirectory
            }
            if ($null -eq $stagedReopenAutomationPath) {
                $productRuns += Invoke-MvpStagedProduct `
                    -Product 'editor' `
                    -ExecutablePath (Join-Path $stageDirectory 'editor\zircon_editor.exe') `
                    -WorkingDirectory (Join-Path $stageDirectory 'editor') `
                    -StageRoot $stageDirectory `
                    -ProjectRoot $stagedProjectRoot
            }
            else {
                for ($reopenAttempt = 1; $reopenAttempt -le $ReopenRepeatCount; $reopenAttempt++) {
                    $reopenAutomation += Invoke-MvpStagedAuthoringAutomation `
                        -ExecutablePath (Join-Path $stageDirectory 'editor\zircon_editor.exe') `
                        -WorkingDirectory (Join-Path $stageDirectory 'editor') `
                        -StageRoot $stageDirectory `
                        -ProjectRoot $stagedProjectRoot `
                        -AutomationRequestPath $stagedReopenAutomationPath `
                        -EvidenceLabel "editor-reopen-$reopenAttempt"
                    Assert-MvpStagingProcessesReleased -StageDirectory $stageDirectory
                    $editorRunParameters = @{
                        Product = 'editor'
                        ExecutablePath = (Join-Path $stageDirectory 'editor\zircon_editor.exe')
                        WorkingDirectory = (Join-Path $stageDirectory 'editor')
                        StageRoot = $stageDirectory
                        ProjectRoot = $stagedProjectRoot
                        AttemptOffset = $reopenAttempt - 1
                        RunCount = 1
                    }
                    if ($reopenAttempt -eq 1) {
                        $editorRunParameters.EditorWindowCaptureName = 'editor-after-reopen.png'
                    }
                    $productRuns += Invoke-MvpStagedProduct @editorRunParameters
                    Assert-MvpStagingProcessesReleased -StageDirectory $stageDirectory
                }
                $productRuns += Invoke-MvpStagedProduct `
                    -Product 'runtime' `
                    -ExecutablePath (Join-Path $stageDirectory 'runtime\zircon_runtime.exe') `
                    -WorkingDirectory (Join-Path $stageDirectory 'runtime') `
                    -StageRoot $stageDirectory `
                    -ProjectRoot $stagedProjectRoot `
                    -AttemptOffset $RepeatCount `
                    -RunCount 1
                Assert-MvpStagingProcessesReleased -StageDirectory $stageDirectory
            }
            Test-MvpStagingDirectoryReleased -StageDirectory $stageDirectory
            Write-MvpJson -Path (Join-Path $stageDirectory 'startup-summary.json') -Value ([ordered]@{
                run_id = $RunId
                source_fingerprint = $SourceFingerprint
                staged_project_root = if ($null -eq $stagedProjectRoot) { $null } else { Get-MvpRelativePath -Root $stageDirectory -Path $stagedProjectRoot -Label 'Staged project' }
                project_creation = $projectCreation
                products = $productRuns
                baseline_automation = $baselineAutomation
                authoring_automation = $authoringAutomation
                reopen_automation = $reopenAutomation
            })
        }
    }
    catch {
        throw "MVP product startup failed for staging run $($RunId): $($_.Exception.Message)"
    }

    $manifestPath = Join-Path $stageDirectory 'staging-manifest.json'
    return [ordered]@{
        staging_root = $stageDirectory
        manifest = $manifestPath
        output_hash = Get-FileSha256 -Path $manifestPath
        launched = -not $NoLaunch
        staged_project_root = if ($null -eq $stagedProjectRoot) {
            $null
        }
        else {
            (Resolve-ZirconWindowsPath -Path $stagedProjectRoot).DisplayPath
        }
        product_runs = $productRuns
        baseline_automation = $baselineAutomation
        authoring_automation = $authoringAutomation
        reopen_automation = $reopenAutomation
    }
}

$result = Invoke-MvpProductStaging
if ($Json) {
    $result | ConvertTo-Json -Depth 64
} else {
    $result
}
