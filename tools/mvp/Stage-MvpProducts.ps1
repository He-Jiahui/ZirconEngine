[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string]$ProductInputManifest,
    [Parameter(Mandatory)]
    [string]$TemplateRoot,
    [Parameter(Mandatory)]
    [string]$EngineAssetRoot,
    [string]$ProjectRoot,
    [string]$AuthoringAutomationRequest,
    [string]$ReopenAutomationRequest,
    [switch]$CreateProject,
    [string]$ProjectName = 'ZirconMvpFixture',
    [string]$StagingRoot,
    [string]$RunId = ('mvp-f0-' + (Get-Date -Format 'yyyyMMdd-HHmmss') + '-' + [guid]::NewGuid().ToString('N').Substring(0, 8)),
    [Nullable[int]]$RepeatCount,
    [Nullable[int]]$ReopenRepeatCount,
    [Nullable[int]]$TimeoutSeconds,
    [Nullable[int]]$ProgressInactivityTimeoutSeconds,
    [ValidateRange(1024, 67108864)]
    [int]$MaxProcessLogBytes = 4194304,
    [switch]$NoLaunch,
    [switch]$AllowUnsafeStagingRoot,
    [switch]$Json
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Import-Module (Join-Path $PSScriptRoot 'MvpProjectOpenEvidence.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpStagingPreflight.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpStagingRelease.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpProductInputManifest.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpBuildSet.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpAcceptanceStagingTreeManifest.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpStageProcessEnvironmentPolicy.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpStagingTerminalReceipt.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpStagingCancellationRequest.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpProcessLivenessProbe.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpAutomationScenarioSpec.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpScenarioRegistry.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpProcessQualificationContext.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpRunArtifactBudget.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'StagedProcessSupervisor.psm1') -Force -ErrorAction Stop
$pathResolverRepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Import-Module (Join-Path $pathResolverRepoRoot 'tools\WindowsPathResolver.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpArtifactStoragePolicy.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpProjectCopyPolicy.psm1') -Force -ErrorAction Stop

if ([string]::IsNullOrWhiteSpace($StagingRoot)) {
    $StagingRoot = Get-MvpArtifactStorageDefaultRootPath -CapabilityClass 'windows-local-artifact'
}

$script:MvpMaximumDiagnosticFileCount = 64
$script:MvpMaximumDiagnosticDirectoryDepth = 8
$script:MvpMaximumAdditionalArtifactFileCount = 4096
$script:MvpRunArtifactBudgetPolicyId = 'mvp.staging-run-artifacts.v1'
$script:MvpScenarioRegistryPath = Join-Path $PSScriptRoot 'mvp-scenario-registry.json'
$script:MvpProjectCopyPolicyPath = Join-Path $PSScriptRoot 'mvp-project-copy-policy.json'
$script:MvpUpperHexDigits = [char[]]'0123456789ABCDEF'

function Assert-MvpStagingCancellationNotRequested {
    param([Parameter(Mandatory)][scriptblock]$CancellationProbe)

    if ([bool](& $CancellationProbe)) {
        throw [OperationCanceledException]::new('MVP staging was cancelled by its run-bound external request.')
    }
}

function Get-FileSha256 {
    param([Parameter(Mandatory)][string]$Path)

    $stream = [IO.File]::OpenRead($Path)
    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        $hashBytes = $hasher.ComputeHash($stream)
        $characters = [char[]]::new($hashBytes.Length * 2)
        $index = 0
        foreach ($hashByte in $hashBytes) {
            $characters[$index] = $script:MvpUpperHexDigits[$hashByte -shr 4]
            $characters[$index + 1] = $script:MvpUpperHexDigits[$hashByte -band 0x0F]
            $index += 2
        }
        return [string]::new($characters)
    }
    finally {
        $hasher.Dispose()
        $stream.Dispose()
    }
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

    $resolvedPath = (Resolve-ZirconWindowsPath -Path $Path).OperationalPath
    if (-not [IO.File]::Exists($resolvedPath)) {
        throw "$Label '$Path' does not exist or is not a file."
    }
    return $resolvedPath
}

function Resolve-MvpInputDirectory {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Label
    )

    $resolvedPath = (Resolve-ZirconWindowsPath -Path $Path).OperationalPath
    if (-not [IO.Directory]::Exists($resolvedPath)) {
        throw "$Label '$Path' does not exist or is not a directory."
    }
    return $resolvedPath
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

function Assert-MvpProductInputBuildIdentity {
    param([Parameter(Mandatory)]$ProductInputs)

    if ($null -eq $ProductInputs.build_set) {
        throw 'ProductInputManifest requires a BuildSet receipt before staging.'
    }

    $manifestDirectory = [IO.Path]::GetDirectoryName($ProductInputs.operation_path)
    if ([string]::IsNullOrWhiteSpace($manifestDirectory)) {
        throw "ProductInputManifest '$($ProductInputs.operation_path)' does not have a containing directory for its BuildSet receipt."
    }
    $manifestDirectory = [IO.Path]::GetFullPath($manifestDirectory).TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar)
    # The product-input resolver returns a device path. PowerShell's Join-Path does not
    # preserve that form, so compose the validated receipt path through System.IO instead.
    $buildSetRelativePath = $ProductInputs.build_set.manifest_relative_path.Replace('/', [IO.Path]::DirectorySeparatorChar)
    $buildSetManifestPath = [IO.Path]::GetFullPath([IO.Path]::Combine($manifestDirectory, $buildSetRelativePath))
    $manifestDirectoryPrefix = $manifestDirectory + [IO.Path]::DirectorySeparatorChar
    if (-not $buildSetManifestPath.StartsWith($manifestDirectoryPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "ProductInputManifest BuildSet receipt path escapes its product-input directory: $($ProductInputs.build_set.manifest_relative_path)"
    }
    $buildSet = Assert-MvpProductBuildSet -ManifestPath $buildSetManifestPath
    foreach ($propertyName in @('build_set_id', 'git_revision', 'dirty_overlay_sha256')) {
        if ([string]$buildSet.$propertyName -ne [string]$ProductInputs.build_set.$propertyName) {
            throw "ProductInputManifest BuildSet identity '$propertyName' differs from the validated BuildSet receipt."
        }
    }
}

function Resolve-MvpStagingRoot {
    param([Parameter(Mandatory)][string]$Path)

    if ($AllowUnsafeStagingRoot) {
        return (Resolve-ZirconWindowsPath -Path $Path).OperationalPath
    }
    try {
        $storage = Resolve-MvpArtifactStorageRootPath `
            -Path $Path `
            -CapabilityClass 'windows-local-artifact'
        return $storage.operation_path
    }
    catch {
        throw "StagingRoot '$Path' is not under an approved artifact storage root: $($_.Exception.Message)"
    }
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

function Get-MvpOperationalFileList {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Label,
        [string]$Extension,
        [ValidateRange(1, [Int32]::MaxValue)][int]$MaximumFileCount = [Int32]::MaxValue,
        [ValidateRange(0, [Int32]::MaxValue)][int]$MaximumDirectoryDepth = [Int32]::MaxValue,
        [AllowNull()]$ProjectCopyPolicySnapshot
    )

    try {
        # Windows PowerShell 5.1 lacks the EnumerationOptions overload. Traverse the
        # operational tree explicitly so reparse points cannot redirect input staging.
        $pendingDirectories = [System.Collections.Generic.Stack[string]]::new()
        $pendingDirectoryDepths = [System.Collections.Generic.Stack[int]]::new()
        $files = [System.Collections.Generic.List[string]]::new()
        [Int64]$encounteredFileCount = 0
        $pendingDirectories.Push($Path)
        $pendingDirectoryDepths.Push(0)
        while ($pendingDirectories.Count -gt 0) {
            $directory = $pendingDirectories.Pop()
            $directoryDepth = $pendingDirectoryDepths.Pop()
            foreach ($file in [IO.Directory]::GetFiles($directory)) {
                $attributes = [IO.File]::GetAttributes($file)
                if ([bool]($attributes -band [IO.FileAttributes]::ReparsePoint)) {
                    throw "$Label source file '$file' cannot be staged because it is a reparse point."
                }
                if ($null -ne $ProjectCopyPolicySnapshot) {
                    $relativeFile = Get-MvpRelativePath -Root $Path -Path $file -Label "$Label file"
                    if (-not (Test-MvpProjectCopyPolicyPathIncluded `
                            -PolicySnapshot $ProjectCopyPolicySnapshot `
                            -RelativePath $relativeFile)) {
                        continue
                    }
                }
                $encounteredFileCount++
                if ($encounteredFileCount -gt $MaximumFileCount) {
                    throw "$Label file count exceeds its budget of $MaximumFileCount files."
                }
                if (-not [string]::IsNullOrWhiteSpace($Extension) -and
                    -not [IO.Path]::GetExtension($file).Equals($Extension, [StringComparison]::OrdinalIgnoreCase)) {
                    continue
                }
                $files.Add($file) | Out-Null
            }
            foreach ($childDirectory in [IO.Directory]::GetDirectories($directory)) {
                $attributes = [IO.File]::GetAttributes($childDirectory)
                if ([bool]($attributes -band [IO.FileAttributes]::ReparsePoint)) {
                    throw "$Label source directory '$childDirectory' cannot be staged because it is a reparse point."
                }
                if ($null -ne $ProjectCopyPolicySnapshot) {
                    $relativeDirectory = Get-MvpRelativePath -Root $Path -Path $childDirectory -Label "$Label directory"
                    if (-not (Test-MvpProjectCopyPolicyPathIncluded `
                            -PolicySnapshot $ProjectCopyPolicySnapshot `
                            -RelativePath $relativeDirectory)) {
                        continue
                    }
                }
                $childDirectoryDepth = $directoryDepth + 1
                if ($childDirectoryDepth -gt $MaximumDirectoryDepth) {
                    throw "$Label directory depth exceeds its budget of $MaximumDirectoryDepth levels."
                }
                $pendingDirectories.Push($childDirectory)
                $pendingDirectoryDepths.Push($childDirectoryDepth)
            }
        }
        return @($files.ToArray() | Sort-Object)
    }
    catch {
        throw "$Label '$Path' could not be enumerated through its resolver operational path: $($_.Exception.Message)"
    }
}

function Copy-MvpStageFile {
    param(
        [Parameter(Mandatory)][string]$LogicalId,
        [Parameter(Mandatory)][string]$SourcePath,
        [Parameter(Mandatory)][string]$StageRoot,
        [Parameter(Mandatory)][string]$TargetRelativePath,
        [Int64]$ExpectedBytes = -1,
        [string]$ExpectedSha256
    )

    if ([IO.Path]::IsPathRooted($TargetRelativePath) -or $TargetRelativePath -match '(^|[\\/])\.\.([\\/]|$)') {
        throw "Staging target '$TargetRelativePath' for $LogicalId escapes the staging root."
    }
    $targetPath = Join-ZirconWindowsPath -Path $StageRoot -ChildPath $TargetRelativePath
    $targetDirectory = [IO.Path]::GetDirectoryName($targetPath)
    [IO.Directory]::CreateDirectory($targetDirectory) | Out-Null
    [IO.File]::Copy($SourcePath, $targetPath, $false)
    $targetBytes = [IO.FileInfo]::new($targetPath).Length
    $targetHash = Get-FileSha256 -Path $targetPath
    if ($ExpectedBytes -ge 0 -and $targetBytes -ne $ExpectedBytes) {
        throw "Target '$TargetRelativePath' byte length differs from the expected product input '$LogicalId'."
    }
    if (-not [string]::IsNullOrWhiteSpace($ExpectedSha256)) {
        if (-not $targetHash.Equals($ExpectedSha256, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Target '$TargetRelativePath' SHA-256 differs from the expected product input '$LogicalId'."
        }
    }
    else {
        $sourceHash = Get-FileSha256 -Path $SourcePath
        if ($sourceHash -ne $targetHash) {
            throw "Content hash mismatch while staging $LogicalId from '$SourcePath'."
        }
    }

    return [ordered]@{
        logical_id = $LogicalId
        target_relative_path = $TargetRelativePath.Replace('\', '/')
        sha256 = $targetHash
        size_bytes = $targetBytes
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

    $resolvedPath = (Resolve-ZirconWindowsPath -Path $Path).OperationalPath
    if (-not [IO.File]::Exists($resolvedPath)) {
        throw "$Label '$Path' does not exist."
    }
    return [ordered]@{
        path = Get-MvpRelativePath -Root $StageRoot -Path $resolvedPath -Label $Label
        sha256 = Get-FileSha256 -Path $resolvedPath
        size_bytes = [IO.FileInfo]::new($resolvedPath).Length
    }
}

function Get-MvpPngCaptureEvidence {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$StageRoot,
        [Parameter(Mandatory)][string]$Label
    )

    $resolvedPath = (Resolve-ZirconWindowsPath -Path $Path).OperationalPath
    if (-not [IO.File]::Exists($resolvedPath)) {
        throw "$Label '$Path' was not written."
    }
    $capture = [IO.FileInfo]::new($resolvedPath)
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
        using (var stream = new System.IO.FileStream(
            path,
            System.IO.FileMode.Open,
            System.IO.FileAccess.Read,
            System.IO.FileShare.Read))
        using (var source = new Bitmap(stream))
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

    $summary = [ZirconMvpPngEvidence]::Inspect($resolvedPath)
    if ($summary.NonTransparentPixels -le 0) {
        throw "$Label '$Path' has no visible pixels."
    }
    if ($summary.NonBackgroundPixels -lt 100) {
        throw "$Label '$Path' has only $($summary.NonBackgroundPixels) non-background pixels; expected at least 100."
    }
    return [ordered]@{
        path = Get-MvpRelativePath -Root $StageRoot -Path $resolvedPath -Label $Label
        sha256 = Get-FileSha256 -Path $resolvedPath
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

function New-MvpStagedProcessLaunch {
    param(
        [Parameter(Mandatory)][string]$ExecutablePath,
        [Parameter(Mandatory)][string]$WorkingDirectory,
        [Parameter(Mandatory)][hashtable]$Environment,
        [Parameter(Mandatory)][string]$StageRoot,
        [string]$ProjectRoot,
        [string[]]$Arguments = @()
    )

    # Resolve all product-bound paths before the supervisor creates its suspended child.
    $stagedProductRoot = (Resolve-ZirconWindowsPath -Path $StageRoot).OperationalPath
    $executableResolution = Resolve-ZirconWindowsPath -Path $ExecutablePath
    $workingDirectoryResolution = Resolve-ZirconWindowsPath -Path $WorkingDirectory
    $projectRootResolution = if ([string]::IsNullOrWhiteSpace($ProjectRoot)) {
        $null
    }
    else {
        $resolution = Resolve-ZirconWindowsPath -Path $ProjectRoot
        if (-not [IO.Directory]::Exists($resolution.OperationalPath)) {
            throw "ProjectRoot '$ProjectRoot' does not exist or is not a directory."
        }
        $resolution
    }
    $startInfo = [Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $executableResolution.OperationalPath
    $startInfo.WorkingDirectory = if ($null -eq $projectRootResolution) {
        $workingDirectoryResolution.DisplayPath
    }
    else {
        $projectRootResolution.DisplayPath
    }
    $startInfo.UseShellExecute = $false
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    $declaredEnvironment = @{}
    $productPathEnvironmentVariables = @(
        'ZIRCON_LOG_ROOT',
        'ZIRCON_RUNTIME_CAPTURE_FRAME_PNG',
        'ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG'
    )
    foreach ($name in $Environment.Keys) {
        $environmentValue = [string]$Environment[$name]
        if ($productPathEnvironmentVariables -contains [string]$name -and
            -not [string]::IsNullOrWhiteSpace($environmentValue)) {
            $environmentValue = (Resolve-ZirconWindowsPath -Path $environmentValue).DisplayPath
        }
        $startInfo.EnvironmentVariables[[string]$name] = $environmentValue
        $declaredEnvironment[[string]$name] = $environmentValue
    }
    foreach ($name in @(
        'ZIRCON_RUNTIME_CAPTURE_FRAME_PNG',
        'ZIRCON_RUNTIME_EXIT_AFTER_PRESENTED_FRAMES',
        'ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG',
        'ZIRCON_RUNTIME_MVP_INPUT_PROBE'
    )) {
        if (-not $Environment.ContainsKey($name)) {
            $startInfo.EnvironmentVariables.Remove($name)
        }
    }
    $childArguments = if ($null -eq $projectRootResolution) {
        @($Arguments)
    }
    else {
        @('--project', '.') + @($Arguments)
    }
    if ($childArguments.Count -gt 0) {
        $startInfo.Arguments = ($childArguments | ForEach-Object {
            ConvertTo-MvpProcessArgument -Value $_
        }) -join ' '
    }
    return [pscustomobject]@{
        start_info = $startInfo
        declared_environment = $declaredEnvironment
        staged_product_root = $stagedProductRoot
    }
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
    $reportedEditorProjectPath = [string]$fields.project_path
    $expectedEditorProjectResolution = Resolve-ZirconWindowsPath -Path $ProjectRoot
    $expectedProjectPath = $expectedEditorProjectResolution.OperationalPath
    if ($reportedEditorProjectPath -eq '.') {
        $resolvedReportedEditorProjectPath = $expectedProjectPath
    }
    elseif (Test-MvpFullyQualifiedWindowsPath -Path $reportedEditorProjectPath) {
        $resolvedReportedEditorProjectPath = (Resolve-ZirconWindowsPath -Path $reportedEditorProjectPath).OperationalPath
    }
    else {
        if ([IO.Path]::IsPathRooted($reportedEditorProjectPath) -or $reportedEditorProjectPath.Contains(':')) {
            throw "Editor product diagnostic has an invalid project_path '$reportedEditorProjectPath'. Expected '.', a staged-project-parent relative path, or an absolute path."
        }
        $expectedEditorProjectParent = [IO.Directory]::GetParent($expectedEditorProjectResolution.DisplayPath)
        if ($null -eq $expectedEditorProjectParent) {
            throw "Editor product diagnostic cannot derive the staged parent of '$ProjectRoot'."
        }
        $reportedEditorProjectCandidate = [IO.Path]::Combine(
            $expectedEditorProjectParent.FullName,
            $reportedEditorProjectPath)
        $resolvedReportedEditorProjectPath = (Resolve-ZirconWindowsPath -Path $reportedEditorProjectCandidate).OperationalPath
    }
    if (-not $resolvedReportedEditorProjectPath.Equals($expectedProjectPath, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Editor product diagnostic project_path '$reportedEditorProjectPath' differs from staged project '$expectedProjectPath'."
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
        [Parameter(Mandatory)][string]$RunId,
        [string]$ProjectRoot,
        [ValidateRange(0, [int]::MaxValue)]
        [int]$AttemptOffset = 0,
        [Nullable[int]]$RunCount,
        [string]$EditorWindowCaptureName,
        [Parameter(Mandatory)]$ScenarioRegistration,
        [Parameter(Mandatory)]$ExecutionPolicy,
        [Parameter(Mandatory)]$QualificationContext,
        [Parameter(Mandatory)]$ArtifactBudget,
        [Parameter(Mandatory)][scriptblock]$CancellationProbe
    )

    $effectiveRunCount = if ($null -eq $RunCount) { [int]$ExecutionPolicy.attempt_count } else { [int]$RunCount }
    if ($effectiveRunCount -lt [int]$ExecutionPolicy.attempt_minimum -or
        $effectiveRunCount -gt [int]$ExecutionPolicy.attempt_maximum) {
        throw "Scenario '$($ExecutionPolicy.scenario_id)' run count $effectiveRunCount is outside resolved policy range $($ExecutionPolicy.attempt_minimum)..$($ExecutionPolicy.attempt_maximum)."
    }
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
    $environmentPolicy = Get-MvpStageProcessEnvironmentPolicy -Scenario ($Product + '_first_frame')
    $results = [System.Collections.Generic.List[object]]::new()
    $logDirectory = Join-ZirconWindowsPath -Path $StageRoot -ChildPath 'logs'
    $captureDirectory = Join-ZirconWindowsPath -Path $StageRoot -ChildPath 'captures'
    [IO.Directory]::CreateDirectory($logDirectory) | Out-Null
    for ($runIndex = 1; $runIndex -le $effectiveRunCount; $runIndex++) {
        $attempt = $AttemptOffset + $runIndex
        $stdout = Join-ZirconWindowsPath -Path $logDirectory -ChildPath "$Product-$attempt.stdout.log"
        $stderr = Join-ZirconWindowsPath -Path $logDirectory -ChildPath "$Product-$attempt.stderr.log"
        $diagnosticRoot = Join-ZirconWindowsPath -Path $logDirectory -ChildPath "$Product-$attempt.diagnostics"
        $frameCapturePath = if ($Product -eq 'runtime' -and -not [string]::IsNullOrWhiteSpace($ProjectRoot)) {
            [IO.Directory]::CreateDirectory($captureDirectory) | Out-Null
            Join-ZirconWindowsPath -Path $captureDirectory -ChildPath "$Product-$attempt.png"
        } else {
            $null
        }
        $editorWindowCapturePath = if ($Product -eq 'editor' -and -not [string]::IsNullOrWhiteSpace($EditorWindowCaptureName)) {
            [IO.Directory]::CreateDirectory($captureDirectory) | Out-Null
            Join-ZirconWindowsPath -Path $captureDirectory -ChildPath $EditorWindowCaptureName
        } else {
            $null
        }
        $environment = @{
            $exitFlag = '1'
            ZIRCON_RUNTIME_LIBRARY = ''
            ZIRCON_ASSET_ROOT = 'assets'
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
        $progressState = New-MvpProcessLivenessProbeState `
            -DiagnosticRoot $diagnosticRoot `
            -ScenarioRegistration $ScenarioRegistration
        $progressProbe = { Read-MvpProcessLivenessProgress -State $progressState }.GetNewClosure()
        Assert-MvpStagingCancellationNotRequested -CancellationProbe $CancellationProbe
        try {
            $processLaunch = New-MvpStagedProcessLaunch `
                -ExecutablePath $ExecutablePath `
                -WorkingDirectory $WorkingDirectory `
                -Environment $environment `
                -StageRoot $StageRoot `
                -ProjectRoot $ProjectRoot
            $processState = Start-MvpSupervisedProcess `
                -StartInfo $processLaunch.start_info `
                -StageRoot $processLaunch.staged_product_root `
                -RunId $RunId `
                -Phase "$Product-$attempt" `
                -StdoutPath $stdout `
                -StderrPath $stderr `
                -MaximumRetainedLogBytes $MaxProcessLogBytes `
                -EnvironmentPolicy $environmentPolicy `
                -QualificationContext $QualificationContext `
                -ArtifactBudget $ArtifactBudget `
                -DeclaredEnvironment $processLaunch.declared_environment
        }
        catch {
            $started.Stop()
            throw "Staged $Product attempt $attempt could not launch from '$ExecutablePath' in '$WorkingDirectory': $($_.Exception.Message)"
        }
        try {
            $exitCode = Complete-MvpSupervisedProcess `
                -ProcessState $processState `
                -TimeoutSeconds $ExecutionPolicy.process_timeout_seconds `
                -CancellationProbe $CancellationProbe `
                -CancellationReason 'external_request' `
                -ProgressProbe $progressProbe `
                -ProgressInactivityTimeoutSeconds $ExecutionPolicy.progress_inactivity_timeout_seconds
        }
        catch [OperationCanceledException] {
            throw
        }
        catch [TimeoutException] {
            throw "Staged $Product attempt $attempt did not exit within $($ExecutionPolicy.process_timeout_seconds) seconds."
        }
        catch {
            throw "Staged $Product attempt $attempt could not collect process output: $($_.Exception.Message)"
        }
        finally {
            $started.Stop()
            if ($null -ne $processState) {
                Close-MvpSupervisedProcessState -ProcessState $processState
            }
        }
        $failureMessage = if ($exitCode -ne 0) {
            "Staged $Product attempt $attempt exited with code $exitCode. See $stdout and $stderr."
        }
        else {
            $null
        }
        try {
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
        $diagnosticFiles = @(Get-MvpOperationalFileList `
                -Path $diagnosticRoot `
                -Label "Staged $Product diagnostic root" `
                -Extension '.log' `
                -MaximumFileCount $script:MvpMaximumDiagnosticFileCount `
                -MaximumDirectoryDepth $script:MvpMaximumDiagnosticDirectoryDepth)
        $diagnosticText = Get-MvpSupervisedBoundedDiagnosticText -Paths $diagnosticFiles
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
            diagnostic_logs = @($diagnosticFiles | ForEach-Object { Get-MvpStagedFileEvidence -Path $_ -StageRoot $StageRoot -Label 'Product diagnostic log' })
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

    $stdout = [IO.File]::ReadAllText($StdoutPath)
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

    $commandlet = $reports[0]
    if ([string]$commandlet.command -ne 'authoring-automation' -or
        [string]$commandlet.status -ne 'succeeded' -or
        [int]$commandlet.exit_code -ne 0) {
        throw "Staged editor authoring automation commandlet did not report success. See $StdoutPath and $StderrPath."
    }
    $report = $commandlet.automation
    if ($null -eq $report) {
        throw "Staged editor authoring automation commandlet omitted its typed automation report. See $StdoutPath and $StderrPath."
    }
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
    if ($reportedProjectPath -eq '.') {
        # Project-relative product startup uses the staged project as its child cwd.
        $resolvedReportedProjectPath = $expectedProjectPath
    }
    else {
        if (-not (Test-MvpFullyQualifiedWindowsPath -Path $reportedProjectPath)) {
            throw "Staged editor authoring automation report has an invalid project_path '$reportedProjectPath'. Expected '.' or an absolute path; rooted-but-relative Windows paths are not accepted. See $StdoutPath and $StderrPath."
        }
        try {
            $resolvedReportedProjectPath = (Resolve-ZirconWindowsPath -Path $reportedProjectPath).OperationalPath
        }
        catch {
            throw "Staged editor authoring automation report has an invalid project_path '$reportedProjectPath'. See $StdoutPath and $StderrPath."
        }
    }
    if (-not $resolvedReportedProjectPath.Equals($expectedProjectPath, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Staged editor authoring automation report project_path '$reportedProjectPath' differs from staged project '$expectedProjectPath'. See $StdoutPath and $StderrPath."
    }
    $report.project_path = Get-MvpRelativePath -Root $StageRoot -Path $ProjectRoot -Label 'Authoring automation project'
    # Retain structured child output without leaking the machine-specific absolute project root
    # into a portable CI artifact. The parsed report still records the normal binding sequence.
    Write-MvpJson -Path $StdoutPath -Value $report

    $diagnosticFiles = @(Get-MvpOperationalFileList `
            -Path $DiagnosticRoot `
            -Label 'Staged editor authoring automation diagnostic root' `
            -MaximumFileCount $script:MvpMaximumDiagnosticFileCount `
            -MaximumDirectoryDepth $script:MvpMaximumDiagnosticDirectoryDepth)
    if ($diagnosticFiles.Count -eq 0) {
        throw "Staged editor authoring automation did not emit diagnostic log evidence under '$DiagnosticRoot'."
    }
    $report | Add-Member -NotePropertyName 'automation_request' -NotePropertyValue (Get-MvpStagedFileEvidence -Path $AutomationRequestPath -StageRoot $StageRoot -Label 'Authoring automation request')
    $report | Add-Member -NotePropertyName 'stdout' -NotePropertyValue (Get-MvpStagedFileEvidence -Path $StdoutPath -StageRoot $StageRoot -Label 'Authoring automation stdout log')
    $report | Add-Member -NotePropertyName 'stderr' -NotePropertyValue (Get-MvpStagedFileEvidence -Path $StderrPath -StageRoot $StageRoot -Label 'Authoring automation stderr log')
    $report | Add-Member -NotePropertyName 'diagnostic_logs' -NotePropertyValue @($diagnosticFiles | ForEach-Object { Get-MvpStagedFileEvidence -Path $_ -StageRoot $StageRoot -Label 'Authoring automation diagnostic log' })
    return $report
}

function Test-MvpFullyQualifiedWindowsPath {
    param([Parameter(Mandatory)][string]$Path)

    return $Path -match '^[A-Za-z]:[\\/]' -or
        $Path -match '^\\\\\?\\[A-Za-z]:[\\/]' -or
        $Path -match '^\\\\(?![?.][\\/])[^\\/]+[\\/][^\\/]+(?:[\\/]|$)' -or
        $Path -match '^\\\\\?\\UNC[\\/][^\\/]+[\\/][^\\/]+(?:[\\/]|$)'
}

function Get-MvpStagedProcessStderrSummary {
    param(
        [Parameter(Mandatory)][string]$Path,
        [int]$MaximumCharacters = 2048
    )

    $tailPath = [IO.Path]::ChangeExtension($Path, 'tail.log')
    $summaryPath = if ([IO.File]::Exists($tailPath)) { $tailPath } else { $Path }
    return Get-MvpSupervisedBoundedTailText -Path $summaryPath -MaximumCharacters $MaximumCharacters
}

function Invoke-MvpStagedAuthoringAutomation {
    param(
        [Parameter(Mandatory)][string]$ExecutablePath,
        [Parameter(Mandatory)][string]$WorkingDirectory,
        [Parameter(Mandatory)][string]$StageRoot,
        [Parameter(Mandatory)][string]$RunId,
        [Parameter(Mandatory)][string]$ProjectRoot,
        [Parameter(Mandatory)][string]$AutomationRequestPath,
        [Parameter(Mandatory)][string]$EvidenceLabel,
        [Parameter(Mandatory)]$ScenarioRegistration,
        [Parameter(Mandatory)]$ExecutionPolicy,
        [Parameter(Mandatory)]$QualificationContext,
        [Parameter(Mandatory)]$ArtifactBudget,
        [Parameter(Mandatory)][scriptblock]$CancellationProbe
    )

    $logDirectory = Join-ZirconWindowsPath -Path $StageRoot -ChildPath 'logs'
    [IO.Directory]::CreateDirectory($logDirectory) | Out-Null
    $stdout = Join-ZirconWindowsPath -Path $logDirectory -ChildPath "$EvidenceLabel.stdout.log"
    $stderr = Join-ZirconWindowsPath -Path $logDirectory -ChildPath "$EvidenceLabel.stderr.log"
    $environment = @{
        ZIRCON_RUNTIME_LIBRARY = ''
        ZIRCON_ASSET_ROOT = 'assets'
        ZIRCON_LOG_ROOT = (Join-ZirconWindowsPath -Path $logDirectory -ChildPath "$EvidenceLabel.diagnostics")
        ZIRCON_LOG_FILTER = 'log'
    }
    $environmentPolicy = Get-MvpStageProcessEnvironmentPolicy -Scenario 'editor_authoring'
    $progressState = New-MvpProcessLivenessProbeState `
        -DiagnosticRoot $environment.ZIRCON_LOG_ROOT `
        -ScenarioRegistration $ScenarioRegistration
    $progressProbe = { Read-MvpProcessLivenessProgress -State $progressState }.GetNewClosure()
    Assert-MvpStagingCancellationNotRequested -CancellationProbe $CancellationProbe
    $started = [Diagnostics.Stopwatch]::StartNew()
    $processState = $null
    $automationRequestArgument = (Resolve-ZirconWindowsPath -Path $AutomationRequestPath).DisplayPath
    try {
        $processLaunch = New-MvpStagedProcessLaunch `
            -ExecutablePath $ExecutablePath `
            -WorkingDirectory $WorkingDirectory `
            -Environment $environment `
            -StageRoot $StageRoot `
            -ProjectRoot $ProjectRoot `
            -Arguments @('--run', 'authoring-automation', '--automation', $automationRequestArgument)
        $processState = Start-MvpSupervisedProcess `
            -StartInfo $processLaunch.start_info `
            -StageRoot $processLaunch.staged_product_root `
            -RunId $RunId `
            -Phase $EvidenceLabel `
            -StdoutPath $stdout `
            -StderrPath $stderr `
            -MaximumRetainedLogBytes $MaxProcessLogBytes `
            -EnvironmentPolicy $environmentPolicy `
            -QualificationContext $QualificationContext `
            -ArtifactBudget $ArtifactBudget `
            -DeclaredEnvironment $processLaunch.declared_environment
        $exitCode = Complete-MvpSupervisedProcess `
            -ProcessState $processState `
            -TimeoutSeconds $ExecutionPolicy.process_timeout_seconds `
            -CancellationProbe $CancellationProbe `
            -CancellationReason 'external_request' `
            -ProgressProbe $progressProbe `
            -ProgressInactivityTimeoutSeconds $ExecutionPolicy.progress_inactivity_timeout_seconds
    }
    catch [OperationCanceledException] {
        throw
    }
    catch [TimeoutException] {
        throw "Staged editor $EvidenceLabel automation did not exit within $($ExecutionPolicy.process_timeout_seconds) seconds."
    }
    catch {
        $stderrSummary = Get-MvpStagedProcessStderrSummary -Path $stderr
        throw "Staged editor $EvidenceLabel automation could not launch or collect output: $($_.Exception.Message) stderr: $stderrSummary See $stdout and $stderr."
    }
    finally {
        $started.Stop()
        if ($null -ne $processState) {
            Close-MvpSupervisedProcessState -ProcessState $processState
        }
    }
    Test-MvpStagedProjectDirectoryReleased `
        -StageDirectory $StageRoot `
        -ProjectDirectory $ProjectRoot
    if ($exitCode -ne 0) {
        $stderrSummary = Get-MvpStagedProcessStderrSummary -Path $stderr
        throw "Staged editor $EvidenceLabel automation exited with code $exitCode. stderr: $stderrSummary See $stdout and $stderr."
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

function Test-MvpStagingDirectoryReleased {
    param([Parameter(Mandatory)][string]$StageDirectory)

    $probe = "$StageDirectory.release-probe"
    if ([IO.Directory]::Exists($probe) -or [IO.File]::Exists($probe)) {
        throw "Staging release probe '$probe' already exists."
    }
    Move-ZirconWindowsPath -Source $StageDirectory -Destination $probe
    Move-ZirconWindowsPath -Source $probe -Destination $StageDirectory
}

function Invoke-MvpProductStagingCore {
    param(
        [Parameter(Mandatory)][string]$StagingRootPath,
        [Parameter(Mandatory)][string]$StagingStartedAtUtc,
        [Parameter(Mandatory)]$CancellationState
    )

    $cancellationProbe = { Test-MvpStagingCancellationRequested -State $CancellationState }.GetNewClosure()
    Assert-MvpStagingCancellationNotRequested -CancellationProbe $cancellationProbe
    $projectCopyPolicy = Get-MvpProjectCopyPolicySnapshot -Path $script:MvpProjectCopyPolicyPath
    $scenarioRegistry = Read-MvpScenarioRegistry -Path $script:MvpScenarioRegistryPath
    $scenarioRegistryReceipt = Get-MvpScenarioRegistryReceipt -Registry $scenarioRegistry
    $runtimeScenario = Get-MvpScenarioRegistration -Registry $scenarioRegistry -ScenarioId 'mvp.runtime-first-frame.v1'
    $editorScenario = Get-MvpScenarioRegistration -Registry $scenarioRegistry -ScenarioId 'mvp.editor-first-frame.v1'
    $createScenario = Get-MvpScenarioRegistration -Registry $scenarioRegistry -ScenarioId 'mvp.editor-project-create.v1'
    $authoringScenario = Get-MvpScenarioRegistration -Registry $scenarioRegistry -ScenarioId 'mvp.editor-authoring.v1'
    $reopenScenario = Get-MvpScenarioRegistration -Registry $scenarioRegistry -ScenarioId 'mvp.editor-reopen.v1'
    $runtimeExecutionPolicy = Resolve-MvpScenarioExecutionPolicy `
        -ScenarioRegistration $runtimeScenario `
        -ScenarioVariant 'host.default' `
        -RequestedAttemptCount $RepeatCount `
        -RequestedTimeoutSeconds $TimeoutSeconds `
        -RequestedProgressInactivityTimeoutSeconds $ProgressInactivityTimeoutSeconds
    $editorExecutionPolicy = Resolve-MvpScenarioExecutionPolicy `
        -ScenarioRegistration $editorScenario `
        -ScenarioVariant 'host.default' `
        -RequestedAttemptCount $RepeatCount `
        -RequestedTimeoutSeconds $TimeoutSeconds `
        -RequestedProgressInactivityTimeoutSeconds $ProgressInactivityTimeoutSeconds
    $createExecutionPolicy = Resolve-MvpScenarioExecutionPolicy `
        -ScenarioRegistration $createScenario `
        -ScenarioVariant 'host.default' `
        -RequestedTimeoutSeconds $TimeoutSeconds `
        -RequestedProgressInactivityTimeoutSeconds $ProgressInactivityTimeoutSeconds
    $authoringExecutionPolicy = Resolve-MvpScenarioExecutionPolicy `
        -ScenarioRegistration $authoringScenario `
        -ScenarioVariant 'host.default' `
        -RequestedTimeoutSeconds $TimeoutSeconds `
        -RequestedProgressInactivityTimeoutSeconds $ProgressInactivityTimeoutSeconds
    $reopenExecutionPolicy = Resolve-MvpScenarioExecutionPolicy `
        -ScenarioRegistration $reopenScenario `
        -ScenarioVariant 'host.default' `
        -RequestedAttemptCount $ReopenRepeatCount `
        -RequestedTimeoutSeconds $TimeoutSeconds `
        -RequestedProgressInactivityTimeoutSeconds $ProgressInactivityTimeoutSeconds
    $scenarioExecutionPolicyReceipts = @(
        $runtimeExecutionPolicy,
        $editorExecutionPolicy,
        $createExecutionPolicy,
        $authoringExecutionPolicy,
        $reopenExecutionPolicy
    )
    $productInputs = Resolve-MvpProductInputManifest -Path $ProductInputManifest
    Assert-MvpProductInputBuildIdentity -ProductInputs $productInputs
    $SourceFingerprint = $productInputs.source_fingerprint
    $qualificationContextParameters = @{
        RunId = $RunId
        SourceFingerprint = $SourceFingerprint
        BuildSetId = if ($null -eq $productInputs.build_set) { $null } else { $productInputs.build_set.build_set_id }
        ScenarioRegistryReceipt = $scenarioRegistryReceipt
        ScenarioVariant = 'host.default'
        ProductReceiptIds = @()
    }
    $runtimeQualificationContext = New-MvpProcessQualificationContext `
        @qualificationContextParameters `
        -ScenarioRegistration $runtimeScenario
    $editorQualificationContext = New-MvpProcessQualificationContext `
        @qualificationContextParameters `
        -ScenarioRegistration $editorScenario
    $createQualificationContext = New-MvpProcessQualificationContext `
        @qualificationContextParameters `
        -ScenarioRegistration $createScenario
    $authoringQualificationContext = New-MvpProcessQualificationContext `
        @qualificationContextParameters `
        -ScenarioRegistration $authoringScenario
    $reopenQualificationContext = New-MvpProcessQualificationContext `
        @qualificationContextParameters `
        -ScenarioRegistration $reopenScenario
    $processQualificationContexts = @(
        $runtimeQualificationContext,
        $editorQualificationContext,
        $createQualificationContext,
        $authoringQualificationContext,
        $reopenQualificationContext
    )
    $processQualificationContextSetReceipt = Get-MvpProcessQualificationContextSetReceipt `
        -Contexts $processQualificationContexts `
        -ExpectedRunId $RunId
    $runtimeExecutablePath = $productInputs.artifacts['runtime-executable'].operation_path
    $editorExecutablePath = $productInputs.artifacts['editor-executable'].operation_path
    $runtimeLibraryPath = $productInputs.artifacts['runtime-library/runtime'].operation_path
    $editorRuntimeLibraryPath = $productInputs.artifacts['runtime-library/editor'].operation_path
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
    if ($null -ne $authoringAutomationRequestPath) {
        Assert-MvpAutomationScenarioSpec `
            -Path $authoringAutomationRequestPath `
            -ExpectedScenarioId $authoringScenario.scenario_id | Out-Null
    }
    if ($null -ne $reopenAutomationRequestPath) {
        Assert-MvpAutomationScenarioSpec `
            -Path $reopenAutomationRequestPath `
            -ExpectedScenarioId $reopenScenario.scenario_id | Out-Null
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
    if ($null -ne $reopenAutomationRequestPath -and
        ($runtimeExecutionPolicy.attempt_count -ne 2 -or $reopenExecutionPolicy.attempt_count -ne 2)) {
        throw 'ReopenAutomationRequest requires runtime and reopen execution policies to resolve two attempts for the fixed F5 evidence sequence.'
    }
    $stagingRootDisplayPath = (Resolve-ZirconWindowsPath -Path $stagingRootPath).DisplayPath
    $validationMetadata = Resolve-MvpValidationMetadata
    $engineAssetFiles = @(Get-MvpOperationalFileList -Path $engineAssetRootPath -Label 'EngineAssetRoot')
    if ($engineAssetFiles.Count -eq 0) {
        throw "EngineAssetRoot '$engineAssetRootPath' has no files to stage."
    }
    $templateFiles = @(Get-MvpOperationalFileList -Path $templateRootPath -Label 'TemplateRoot')
    if ($templateFiles.Count -eq 0) {
        throw "TemplateRoot '$templateRootPath' has no files to stage."
    }
    $projectFiles = if ($null -eq $projectRootPath) {
        @()
    }
    else {
        @(Get-MvpOperationalFileList `
            -Path $projectRootPath `
            -Label 'ProjectRoot' `
            -ProjectCopyPolicySnapshot $projectCopyPolicy)
    }
    if ($null -ne $projectRootPath -and $projectFiles.Count -eq 0) {
        throw "ProjectRoot '$projectRootPath' has no source files to stage."
    }
    $inputCopies = [System.Collections.Generic.List[object]]::new()
    $inputCopies.Add([ordered]@{ path = $productInputs.operation_path; copy_count = 1 }) | Out-Null
    foreach ($path in @(
        $runtimeExecutablePath,
        $editorExecutablePath,
        $runtimeLibraryPath,
        $editorRuntimeLibraryPath
    )) {
        $inputCopies.Add([ordered]@{ path = $path; copy_count = 1 }) | Out-Null
    }
    foreach ($file in $engineAssetFiles) {
        $inputCopies.Add([ordered]@{ path = $file; copy_count = 2 }) | Out-Null
    }
    foreach ($file in @($templateFiles) + @($projectFiles)) {
        $inputCopies.Add([ordered]@{ path = $file; copy_count = 1 }) | Out-Null
    }
    foreach ($path in @($authoringAutomationRequestPath, $reopenAutomationRequestPath)) {
        if ($null -ne $path) {
            $inputCopies.Add([ordered]@{ path = $path; copy_count = 1 }) | Out-Null
        }
    }
    $preflight = Get-MvpStagingPreflight `
        -StagingRootPath $stagingRootDisplayPath `
        -InputCopies ($inputCopies.ToArray()) `
        -InteractiveDesktopRequired (-not $NoLaunch)
    # The unsafe switch bypasses only the registered staging namespace for tests;
    # it must not bypass the physical storage capability required by terminal receipts.
    $storageCapabilityEvidence = Get-MvpArtifactStorageCapabilityEvidence `
        -RootPath $stagingRootPath `
        -CapabilityClass 'windows-local-artifact' `
        -RequiredFreeSpaceBytes ([Int64]$preflight.required_free_space_bytes)

    $stageDirectory = Join-ZirconWindowsPath -Path $stagingRootPath -ChildPath $RunId
    $partialDirectory = "$stageDirectory.partial-$([guid]::NewGuid().ToString('N'))"
    if ([IO.Directory]::Exists($stageDirectory) -or [IO.File]::Exists($stageDirectory)) {
        throw "MVP staging run '$RunId' already exists at '$stageDirectory'; choose a new RunId rather than overwriting a validation run."
    }
    if ([IO.Directory]::Exists($partialDirectory) -or [IO.File]::Exists($partialDirectory)) {
        throw "MVP staging temporary directory '$partialDirectory' already exists."
    }
    $stagedProjectRoot = if ($null -eq $projectRootPath) { $null } else { Join-ZirconWindowsPath -Path $stageDirectory -ChildPath 'project' }
    $stagedAuthoringAutomationPath = if ($null -eq $authoringAutomationRequestPath) { $null } else { Join-ZirconWindowsPath -Path $stageDirectory -ChildPath 'authoring\automation.json' }
    $stagedReopenAutomationPath = if ($null -eq $reopenAutomationRequestPath) { $null } else { Join-ZirconWindowsPath -Path $stageDirectory -ChildPath 'reopen\automation.json' }

    try {
        [IO.Directory]::CreateDirectory($partialDirectory) | Out-Null
        $entries = [System.Collections.Generic.List[object]]::new()
        $productInputManifestEntry = Copy-MvpStageFile `
            -LogicalId 'product-input-manifest' `
            -SourcePath $productInputs.operation_path `
            -StageRoot $partialDirectory `
            -TargetRelativePath 'build\mvp-product-inputs.json' `
            -ExpectedBytes ([Int64]$productInputs.bytes) `
            -ExpectedSha256 $productInputs.sha256
        $entries.Add($productInputManifestEntry) | Out-Null
        $entries.Add((Copy-MvpStageFile -LogicalId 'runtime-executable' -SourcePath $runtimeExecutablePath -StageRoot $partialDirectory -TargetRelativePath 'runtime\zircon_runtime.exe' -ExpectedBytes ([Int64]$productInputs.artifacts['runtime-executable'].bytes) -ExpectedSha256 $productInputs.artifacts['runtime-executable'].sha256)) | Out-Null
        $entries.Add((Copy-MvpStageFile -LogicalId 'runtime-library/runtime' -SourcePath $runtimeLibraryPath -StageRoot $partialDirectory -TargetRelativePath 'runtime\zircon_runtime.dll' -ExpectedBytes ([Int64]$productInputs.artifacts['runtime-library/runtime'].bytes) -ExpectedSha256 $productInputs.artifacts['runtime-library/runtime'].sha256)) | Out-Null
        $entries.Add((Copy-MvpStageFile -LogicalId 'editor-executable' -SourcePath $editorExecutablePath -StageRoot $partialDirectory -TargetRelativePath 'editor\zircon_editor.exe' -ExpectedBytes ([Int64]$productInputs.artifacts['editor-executable'].bytes) -ExpectedSha256 $productInputs.artifacts['editor-executable'].sha256)) | Out-Null
        $entries.Add((Copy-MvpStageFile -LogicalId 'runtime-library/editor' -SourcePath $editorRuntimeLibraryPath -StageRoot $partialDirectory -TargetRelativePath 'editor\zircon_runtime.dll' -ExpectedBytes ([Int64]$productInputs.artifacts['runtime-library/editor'].bytes) -ExpectedSha256 $productInputs.artifacts['runtime-library/editor'].sha256)) | Out-Null

        foreach ($engineAssetFile in $engineAssetFiles) {
            $relative = Get-MvpRelativePath -Root $engineAssetRootPath -Path $engineAssetFile -Label 'Engine asset'
            foreach ($product in @('runtime', 'editor')) {
                $entries.Add((Copy-MvpStageFile `
                    -LogicalId ('engine-asset/' + $product + '/' + $relative) `
                    -SourcePath $engineAssetFile `
                    -StageRoot $partialDirectory `
                    -TargetRelativePath ($product + '\assets\' + $relative.Replace('/', '\')))) | Out-Null
            }
        }

        foreach ($templateFile in $templateFiles) {
            $relative = Get-MvpRelativePath -Root $templateRootPath -Path $templateFile
            $entries.Add((Copy-MvpStageFile `
                -LogicalId ('template/' + $relative) `
                -SourcePath $templateFile `
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
                $relative = Get-MvpRelativePath -Root $projectRootPath -Path $projectFile -Label 'Project file'
                $entries.Add((Copy-MvpStageFile `
                    -LogicalId ('project/' + $relative) `
                    -SourcePath $projectFile `
                    -StageRoot $partialDirectory `
                    -TargetRelativePath ('project\' + $relative.Replace('/', '\')))) | Out-Null
            }
        }

        $null = Assert-MvpStagingEntryBudget `
            -Entries ($entries.ToArray()) `
            -ExpectedInputCopyBytes ([Int64]$preflight.input_copy_bytes)

        $productInputManifestEvidence = [ordered]@{
            schema_version = 1
            target_relative_path = $productInputManifestEntry.target_relative_path
            size_bytes = $productInputManifestEntry.size_bytes
            sha256 = $productInputManifestEntry.sha256
            source_fingerprint = $SourceFingerprint
            build_set = $productInputs.build_set
            artifacts = @(
                @(
                    'runtime-executable',
                    'runtime-library/runtime',
                    'editor-executable',
                    'runtime-library/editor'
                ) | ForEach-Object {
                        [ordered]@{
                            logical_id = $_
                            bytes = $productInputs.artifacts[$_].bytes
                            sha256 = $productInputs.artifacts[$_].sha256
                        }
                }
            )
        }

        $manifest = [ordered]@{
            schema_version = 1
            run_id = $RunId
            source_fingerprint = $SourceFingerprint
            product_input_manifest = $productInputManifestEvidence
            project_copy_policy = $projectCopyPolicy.receipt
            scenario_registry = $scenarioRegistryReceipt
            scenario_execution_policies = $scenarioExecutionPolicyReceipts
            process_qualification_contexts = $processQualificationContexts
            process_qualification_context_set = $processQualificationContextSetReceipt
            toolchain = $validationMetadata.toolchain
            target = $validationMetadata.target
            staged_at_utc = [DateTimeOffset]::UtcNow.ToString('o')
            preflight = $preflight
            storage_capability = $storageCapabilityEvidence
            entries = $entries.ToArray()
        }
        Write-MvpJson -Path (Join-ZirconWindowsPath -Path $partialDirectory -ChildPath 'staging-manifest.json') -Value $manifest
        Move-ZirconWindowsPath -Source $partialDirectory -Destination $stageDirectory
    }
    catch {
        $stagingFailure = $_
        $cleanupOutcome = 'not_required'
        $cleanupMessage = $null
        try {
            if ([IO.Directory]::Exists($partialDirectory)) {
                [IO.Directory]::Delete($partialDirectory, $true)
                $cleanupOutcome = 'succeeded'
            }
        }
        catch {
            $cleanupOutcome = 'failed'
            $cleanupMessage = $_.Exception.Message
        }
        try {
            Write-MvpStagingTerminalReceipt `
                -StagingRoot $stagingRootPath `
                -RunId $RunId `
                -Outcome 'failed' `
                -Phase 'input_publication' `
                -StartedAtUtc $stagingStartedAtUtc `
                -StagingDirectoryPublished $false `
                -CleanupOutcome $cleanupOutcome `
                -CleanupMessage $cleanupMessage `
                -FailureKind 'input_publication_failed' `
                -QualificationContextSetReceipt $processQualificationContextSetReceipt `
                -StorageCapabilityEvidence $storageCapabilityEvidence `
                -RequiredFreeSpaceBytes ([Int64]$preflight.required_free_space_bytes) `
                -FailureMessage $stagingFailure.Exception.Message | Out-Null
        }
        catch {
            throw "MVP staging input publication failed for run '$RunId': $($stagingFailure.Exception.Message) Terminal receipt publication also failed: $($_.Exception.Message)"
        }
        throw $stagingFailure
    }

    $productRuns = @()
    $projectCreation = $null
    $baselineAutomation = $null
    $authoringAutomation = $null
    $reopenAutomation = @()
    $runArtifactBudget = if ($NoLaunch) {
        $null
    }
    else {
        New-MvpRunArtifactBudget `
            -Root $stageDirectory `
            -PolicyId $script:MvpRunArtifactBudgetPolicyId `
            -MaximumAdditionalBytes ([Int64]$preflight.evidence_reserve_bytes) `
            -MaximumAdditionalFileCount $script:MvpMaximumAdditionalArtifactFileCount
    }
    $runArtifactBudgetReceipt = if ($null -eq $runArtifactBudget) {
        $null
    }
    else {
        Get-MvpRunArtifactBudgetPolicyReceipt -Budget $runArtifactBudget
    }
    $runArtifactBudgetMeasurement = $null
    $stagingPhase = if ($NoLaunch) { 'evidence_publication' } else { 'product_startup' }
    try {
        if (-not $NoLaunch) {
            $stagedProjectRoot = if ($null -eq $projectRootPath) { $null } else { Join-ZirconWindowsPath -Path $stageDirectory -ChildPath 'project' }
            if ($CreateProject) {
                $createLogDirectory = Join-ZirconWindowsPath -Path $stageDirectory -ChildPath 'logs'
                $createDiagnosticRoot = Join-ZirconWindowsPath -Path $createLogDirectory -ChildPath 'editor-create.diagnostics'
                $createStdout = Join-ZirconWindowsPath -Path $createLogDirectory -ChildPath 'editor-create.stdout.log'
                $createStderr = Join-ZirconWindowsPath -Path $createLogDirectory -ChildPath 'editor-create.stderr.log'
                $createEditorWindowCapturePath = Join-ZirconWindowsPath -Path $stageDirectory -ChildPath 'captures\editor-before-edit.png'
                $createProjectLocation = Join-ZirconWindowsPath -Path $stageDirectory -ChildPath 'project'
                [IO.Directory]::CreateDirectory($createLogDirectory) | Out-Null
                [IO.Directory]::CreateDirectory($createProjectLocation) | Out-Null
                $createEnvironment = @{
                    ZIRCON_EDITOR_EXIT_AFTER_FIRST_FRAME = '1'
                    ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG = $createEditorWindowCapturePath
                    ZIRCON_RUNTIME_LIBRARY = ''
                    ZIRCON_ASSET_ROOT = 'assets'
                    ZIRCON_LOG_ROOT = $createDiagnosticRoot
                    ZIRCON_LOG_FILTER = 'log'
                }
                $createEnvironmentPolicy = Get-MvpStageProcessEnvironmentPolicy -Scenario 'editor_project_create'
                $createProgressState = New-MvpProcessLivenessProbeState `
                    -DiagnosticRoot $createDiagnosticRoot `
                    -ScenarioRegistration $createScenario
                $createProgressProbe = { Read-MvpProcessLivenessProgress -State $createProgressState }.GetNewClosure()
                Assert-MvpStagingCancellationNotRequested -CancellationProbe $cancellationProbe
                $createStarted = [Diagnostics.Stopwatch]::StartNew()
                $createProcess = $null
                $createExitCode = $null
                try {
                    $createProcessLaunch = New-MvpStagedProcessLaunch `
                        -ExecutablePath (Join-ZirconWindowsPath -Path $stageDirectory -ChildPath 'editor\zircon_editor.exe') `
                        -WorkingDirectory $createProjectLocation `
                        -Environment $createEnvironment `
                        -StageRoot $stageDirectory `
                        -Arguments @('--create-project', '--project-name', $ProjectName, '--location', '.', '--template', 'renderable-empty')
                    $createProcess = Start-MvpSupervisedProcess `
                        -StartInfo $createProcessLaunch.start_info `
                        -StageRoot $createProcessLaunch.staged_product_root `
                        -RunId $RunId `
                        -Phase 'editor-create' `
                        -StdoutPath $createStdout `
                        -StderrPath $createStderr `
                        -MaximumRetainedLogBytes $MaxProcessLogBytes `
                        -EnvironmentPolicy $createEnvironmentPolicy `
                        -QualificationContext $createQualificationContext `
                        -ArtifactBudget $runArtifactBudget `
                        -DeclaredEnvironment $createProcessLaunch.declared_environment
                    $createExitCode = Complete-MvpSupervisedProcess `
                        -ProcessState $createProcess `
                        -TimeoutSeconds $createExecutionPolicy.process_timeout_seconds `
                        -CancellationProbe $cancellationProbe `
                        -CancellationReason 'external_request' `
                        -ProgressProbe $createProgressProbe `
                        -ProgressInactivityTimeoutSeconds $createExecutionPolicy.progress_inactivity_timeout_seconds
                }
                finally {
                    $createStarted.Stop()
                    if ($null -ne $createProcess) {
                        Close-MvpSupervisedProcessState -ProcessState $createProcess
                    }
                }
                if ($createExitCode -ne 0) {
                    throw "Staged editor project creation failed with exit code $createExitCode."
                }
                $createDiagnosticFiles = @(Get-MvpOperationalFileList `
                        -Path $createDiagnosticRoot `
                        -Label 'Staged editor project creation diagnostic root' `
                        -MaximumFileCount $script:MvpMaximumDiagnosticFileCount `
                        -MaximumDirectoryDepth $script:MvpMaximumDiagnosticDirectoryDepth)
                if ($createDiagnosticFiles.Count -eq 0) {
                    throw "Staged editor project creation emitted no diagnostic log evidence under '$createDiagnosticRoot'."
                }
                $createDiagnosticText = Get-MvpSupervisedBoundedDiagnosticText -Paths $createDiagnosticFiles
                foreach ($diagnostic in @(
                    'editor_first_frame_presented',
                    'editor_process_teardown_complete',
                    'editor_product_frame_capture_written'
                )) {
                    if ($createDiagnosticText.IndexOf($diagnostic, [StringComparison]::Ordinal) -lt 0) {
                        throw "Staged editor project creation exited without the $diagnostic diagnostic under '$createDiagnosticRoot'. See $createStdout and $createStderr."
                    }
                }
                $createdProjectParentResolution = Resolve-ZirconWindowsPath -Path (Join-ZirconWindowsPath -Path $stageDirectory -ChildPath 'project')
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
                    diagnostic_logs = @($createDiagnosticFiles | ForEach-Object { Get-MvpStagedFileEvidence -Path $_ -StageRoot $stageDirectory -Label 'Project creation diagnostic log' })
                    editor_window_capture = $createEditorWindowCapture
                    editor_product_diagnostics = $createEditorProductDiagnostics
                    project_open = $projectOpenEvidence
                }
                $stagedProjectRoot = $createdProjectRoot
            }
            if ($null -ne $stagedReopenAutomationPath) {
                $baselineAutomation = Invoke-MvpStagedAuthoringAutomation `
                    -ExecutablePath (Join-ZirconWindowsPath -Path $stageDirectory -ChildPath 'editor\zircon_editor.exe') `
                    -WorkingDirectory (Join-ZirconWindowsPath -Path $stageDirectory -ChildPath 'editor') `
                    -StageRoot $stageDirectory `
                    -RunId $RunId `
                    -ProjectRoot $stagedProjectRoot `
                    -AutomationRequestPath $stagedReopenAutomationPath `
                    -EvidenceLabel 'editor-baseline' `
                    -ScenarioRegistration $reopenScenario `
                    -ExecutionPolicy $reopenExecutionPolicy `
                    -QualificationContext $reopenQualificationContext `
                    -ArtifactBudget $runArtifactBudget `
                    -CancellationProbe $cancellationProbe
            }
            $productRuns += Invoke-MvpStagedProduct `
                -Product 'runtime' `
                -ExecutablePath (Join-ZirconWindowsPath -Path $stageDirectory -ChildPath 'runtime\zircon_runtime.exe') `
                -WorkingDirectory (Join-ZirconWindowsPath -Path $stageDirectory -ChildPath 'runtime') `
                -StageRoot $stageDirectory `
                -RunId $RunId `
                -ProjectRoot $stagedProjectRoot `
                -ScenarioRegistration $runtimeScenario `
                -ExecutionPolicy $runtimeExecutionPolicy `
                -QualificationContext $runtimeQualificationContext `
                -ArtifactBudget $runArtifactBudget `
                -CancellationProbe $cancellationProbe
            if ($null -ne $stagedAuthoringAutomationPath) {
                $authoringAutomation = Invoke-MvpStagedAuthoringAutomation `
                    -ExecutablePath (Join-ZirconWindowsPath -Path $stageDirectory -ChildPath 'editor\zircon_editor.exe') `
                    -WorkingDirectory (Join-ZirconWindowsPath -Path $stageDirectory -ChildPath 'editor') `
                    -StageRoot $stageDirectory `
                    -RunId $RunId `
                    -ProjectRoot $stagedProjectRoot `
                    -AutomationRequestPath $stagedAuthoringAutomationPath `
                    -EvidenceLabel 'editor-authoring' `
                    -ScenarioRegistration $authoringScenario `
                    -ExecutionPolicy $authoringExecutionPolicy `
                    -QualificationContext $authoringQualificationContext `
                    -ArtifactBudget $runArtifactBudget `
                    -CancellationProbe $cancellationProbe
            }
            if ($null -eq $stagedReopenAutomationPath) {
                $productRuns += Invoke-MvpStagedProduct `
                    -Product 'editor' `
                    -ExecutablePath (Join-ZirconWindowsPath -Path $stageDirectory -ChildPath 'editor\zircon_editor.exe') `
                    -WorkingDirectory (Join-ZirconWindowsPath -Path $stageDirectory -ChildPath 'editor') `
                    -StageRoot $stageDirectory `
                    -RunId $RunId `
                    -ProjectRoot $stagedProjectRoot `
                    -ScenarioRegistration $editorScenario `
                    -ExecutionPolicy $editorExecutionPolicy `
                    -QualificationContext $editorQualificationContext `
                    -ArtifactBudget $runArtifactBudget `
                    -CancellationProbe $cancellationProbe
            }
            else {
                for ($reopenAttempt = 1; $reopenAttempt -le $reopenExecutionPolicy.attempt_count; $reopenAttempt++) {
                    $reopenAutomation += Invoke-MvpStagedAuthoringAutomation `
                        -ExecutablePath (Join-ZirconWindowsPath -Path $stageDirectory -ChildPath 'editor\zircon_editor.exe') `
                        -WorkingDirectory (Join-ZirconWindowsPath -Path $stageDirectory -ChildPath 'editor') `
                        -StageRoot $stageDirectory `
                        -RunId $RunId `
                        -ProjectRoot $stagedProjectRoot `
                        -AutomationRequestPath $stagedReopenAutomationPath `
                        -EvidenceLabel "editor-reopen-$reopenAttempt" `
                        -ScenarioRegistration $reopenScenario `
                        -ExecutionPolicy $reopenExecutionPolicy `
                        -QualificationContext $reopenQualificationContext `
                        -ArtifactBudget $runArtifactBudget `
                        -CancellationProbe $cancellationProbe
                    $editorRunParameters = @{
                        Product = 'editor'
                        ExecutablePath = (Join-ZirconWindowsPath -Path $stageDirectory -ChildPath 'editor\zircon_editor.exe')
                        WorkingDirectory = (Join-ZirconWindowsPath -Path $stageDirectory -ChildPath 'editor')
                        StageRoot = $stageDirectory
                        RunId = $RunId
                        ProjectRoot = $stagedProjectRoot
                        AttemptOffset = $reopenAttempt - 1
                        RunCount = 1
                        ScenarioRegistration = $editorScenario
                        ExecutionPolicy = $editorExecutionPolicy
                        QualificationContext = $editorQualificationContext
                        ArtifactBudget = $runArtifactBudget
                        CancellationProbe = $cancellationProbe
                    }
                    if ($reopenAttempt -eq 1) {
                        $editorRunParameters.EditorWindowCaptureName = 'editor-after-reopen.png'
                    }
                    $productRuns += Invoke-MvpStagedProduct @editorRunParameters
                }
                $productRuns += Invoke-MvpStagedProduct `
                    -Product 'runtime' `
                    -ExecutablePath (Join-ZirconWindowsPath -Path $stageDirectory -ChildPath 'runtime\zircon_runtime.exe') `
                    -WorkingDirectory (Join-ZirconWindowsPath -Path $stageDirectory -ChildPath 'runtime') `
                    -StageRoot $stageDirectory `
                    -RunId $RunId `
                    -ProjectRoot $stagedProjectRoot `
                    -AttemptOffset $runtimeExecutionPolicy.attempt_count `
                    -RunCount 1 `
                    -ScenarioRegistration $runtimeScenario `
                    -ExecutionPolicy $runtimeExecutionPolicy `
                    -QualificationContext $runtimeQualificationContext `
                    -ArtifactBudget $runArtifactBudget `
                    -CancellationProbe $cancellationProbe
            }
            Test-MvpStagingDirectoryReleased -StageDirectory $stageDirectory
            Write-MvpJson -Path (Join-ZirconWindowsPath -Path $stageDirectory -ChildPath 'startup-summary.json') -Value ([ordered]@{
                run_id = $RunId
                source_fingerprint = $SourceFingerprint
                staged_project_root = if ($null -eq $stagedProjectRoot) { $null } else { Get-MvpRelativePath -Root $stageDirectory -Path $stagedProjectRoot -Label 'Staged project' }
                project_creation = $projectCreation
                products = $productRuns
                baseline_automation = $baselineAutomation
                authoring_automation = $authoringAutomation
                reopen_automation = $reopenAutomation
                artifact_budget = $runArtifactBudgetReceipt
                project_copy_policy = $projectCopyPolicy.receipt
                scenario_registry = $scenarioRegistryReceipt
                scenario_execution_policies = $scenarioExecutionPolicyReceipts
                process_qualification_contexts = $processQualificationContexts
                process_qualification_context_set = $processQualificationContextSetReceipt
            })
        }
        $stagingPhase = 'evidence_publication'
        Assert-MvpStagingCancellationNotRequested -CancellationProbe $cancellationProbe
        $treeManifestPath = Write-MvpAcceptanceStagingTreeManifest -StagingRoot $stageDirectory
        if ($null -ne $runArtifactBudget) {
            $runArtifactBudgetMeasurement = Assert-MvpRunArtifactBudget -Budget $runArtifactBudget
        }
    }
    catch {
        $startupFailure = $_
        $cleanupOutcome = 'succeeded'
        $cleanupMessage = $null
        try {
            Test-MvpStagingDirectoryReleased -StageDirectory $stageDirectory
        }
        catch {
            $cleanupOutcome = 'failed'
            $cleanupMessage = $_.Exception.Message
        }
        $failureMessage = [string]$startupFailure.Exception.Message
        $terminalOutcome = if ($startupFailure.Exception -is [TimeoutException] -or $failureMessage -match 'did not exit within') {
            'timed_out'
        }
        elseif ($startupFailure.Exception -is [OperationCanceledException]) {
            'cancelled'
        }
        else {
            'failed'
        }
        $failureKind = switch ($terminalOutcome) {
            'timed_out' { 'product_timeout' }
            'cancelled' { 'product_cancelled' }
            default { $stagingPhase + '_failed' }
        }
        $failureManifestPath = Join-ZirconWindowsPath -Path $stageDirectory -ChildPath 'staging-manifest.json'
        $failureManifestSha256 = (Get-FileSha256 -Path $failureManifestPath).ToLowerInvariant()
        try {
            Write-MvpStagingTerminalReceipt `
                -StagingRoot $stagingRootPath `
                -RunId $RunId `
                -Outcome $terminalOutcome `
                -Phase $stagingPhase `
                -StartedAtUtc $stagingStartedAtUtc `
                -StagingDirectoryPublished $true `
                -CleanupOutcome $cleanupOutcome `
                -CleanupMessage $cleanupMessage `
                -FailureKind $failureKind `
                -FailureMessage $failureMessage `
                -QualificationContextSetReceipt $processQualificationContextSetReceipt `
                -StorageCapabilityEvidence $storageCapabilityEvidence `
                -RequiredFreeSpaceBytes ([Int64]$preflight.required_free_space_bytes) `
                -StagingManifestSha256 $failureManifestSha256 | Out-Null
        }
        catch {
            throw "MVP product startup failed for staging run $($RunId): $failureMessage Terminal receipt publication also failed: $($_.Exception.Message)"
        }
        if ($terminalOutcome -eq 'cancelled') {
            throw $startupFailure
        }
        throw "MVP product startup failed for staging run $($RunId): $failureMessage"
    }

    $manifestPath = Join-ZirconWindowsPath -Path $stageDirectory -ChildPath 'staging-manifest.json'
    $manifestSha256 = Get-FileSha256 -Path $manifestPath
    $terminalReceipt = Write-MvpStagingTerminalReceipt `
        -StagingRoot $stagingRootPath `
        -RunId $RunId `
        -Outcome 'succeeded' `
        -Phase 'complete' `
        -StartedAtUtc $stagingStartedAtUtc `
        -StagingDirectoryPublished $true `
        -CleanupOutcome $(if ($NoLaunch) { 'not_required' } else { 'succeeded' }) `
        -QualificationContextSetReceipt $processQualificationContextSetReceipt `
        -StorageCapabilityEvidence $storageCapabilityEvidence `
        -RequiredFreeSpaceBytes ([Int64]$preflight.required_free_space_bytes) `
        -StagingManifestSha256 ($manifestSha256.ToLowerInvariant())
    return [ordered]@{
        run_id = $RunId
        staging_root = (Resolve-ZirconWindowsPath -Path $stageDirectory).DisplayPath
        manifest = (Resolve-ZirconWindowsPath -Path $manifestPath).DisplayPath
        tree_manifest = (Resolve-ZirconWindowsPath -Path $treeManifestPath).DisplayPath
        output_hash = $manifestSha256
        terminal_receipt = [ordered]@{
            path = (Resolve-ZirconWindowsPath -Path $terminalReceipt.path).DisplayPath
            bytes = $terminalReceipt.bytes
            sha256 = $terminalReceipt.sha256
        }
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
        project_copy_policy = $projectCopyPolicy.receipt
        scenario_registry = $scenarioRegistryReceipt
        scenario_execution_policies = $scenarioExecutionPolicyReceipts
        process_qualification_contexts = $processQualificationContexts
        process_qualification_context_set = $processQualificationContextSetReceipt
        storage_capability = $storageCapabilityEvidence
        artifact_budget = if ($null -eq $runArtifactBudgetReceipt) {
            $null
        }
        else {
            [ordered]@{
                policy = $runArtifactBudgetReceipt
                measurement = $runArtifactBudgetMeasurement
            }
        }
    }
}

function Invoke-MvpProductStaging {
    $stagingStartedAtUtc = [DateTimeOffset]::UtcNow.ToString('o')
    $stagingRootPath = Resolve-MvpStagingRoot -Path $StagingRoot
    Assert-MvpRunId -Value $RunId
    $stageDirectory = Join-ZirconWindowsPath -Path $stagingRootPath -ChildPath $RunId
    if (-not $AllowUnsafeStagingRoot) {
        try {
            Resolve-MvpArtifactStoragePath `
                -Path $stageDirectory `
                -NamespaceId 'mvp-staging-runs' | Out-Null
        }
        catch {
            throw "MVP staging run '$RunId' is outside the registered staging namespace: $($_.Exception.Message)"
        }
    }
    $cancellationState = New-MvpStagingCancellationProbeState -StagingRoot $stagingRootPath -RunId $RunId
    $terminalReceiptPath = Get-MvpStagingTerminalReceiptPath -StagingRoot $stagingRootPath -RunId $RunId
    if ([IO.File]::Exists($terminalReceiptPath)) {
        throw "MVP staging run '$RunId' already has a terminal receipt at '$terminalReceiptPath'; choose a new RunId."
    }
    try {
        return Invoke-MvpProductStagingCore `
            -StagingRootPath $stagingRootPath `
            -StagingStartedAtUtc $stagingStartedAtUtc `
            -CancellationState $cancellationState
    }
    catch {
        $stagingFailure = $_
        if (-not [IO.File]::Exists($terminalReceiptPath) -and -not [IO.Directory]::Exists($stageDirectory)) {
            $admissionOutcome = if ($stagingFailure.Exception -is [OperationCanceledException]) { 'cancelled' } else { 'failed' }
            $admissionFailureKind = if ($admissionOutcome -eq 'cancelled') { 'operation_cancelled' } else { 'admission_failed' }
            try {
                Write-MvpStagingTerminalReceipt `
                    -StagingRoot $stagingRootPath `
                    -RunId $RunId `
                    -Outcome $admissionOutcome `
                    -Phase 'admission' `
                    -StartedAtUtc $stagingStartedAtUtc `
                    -StagingDirectoryPublished $false `
                    -CleanupOutcome 'not_required' `
                    -FailureKind $admissionFailureKind `
                    -FailureMessage $stagingFailure.Exception.Message | Out-Null
            }
            catch {
                throw "MVP staging admission failed for run '$RunId': $($stagingFailure.Exception.Message) Terminal receipt publication also failed: $($_.Exception.Message)"
            }
        }
        throw $stagingFailure
    }
}

$result = Invoke-MvpProductStaging
if ($Json) {
    $result | ConvertTo-Json -Depth 64
} else {
    $result
}
