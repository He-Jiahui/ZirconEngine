<#
.SYNOPSIS
Builds a runnable Zircon editor debug bundle for Windows.

.DESCRIPTION
Builds zircon_editor.exe and zircon_runtime.dll through the repository-managed
Cargo validator, copies runtime assets, and publishes the bundle only after all
steps succeed. The final directory is never overwritten.

.PARAMETER OutputDirectory
Final bundle directory. It must be on a local drive outside the coordinator-
managed D:, E:, and F: roots. Relative paths are resolved from the repository
root. When omitted, a unique directory under %USERPROFILE%\ZirconBuilds is used.

.PARAMETER SkipSmokeTest
Skips the zircon_editor.exe --help launch check. Intended for script tests only.

.EXAMPLE
.\tools\build-editor.ps1

.EXAMPLE
.\tools\build-editor.ps1 -OutputDirectory C:\ZirconBuilds\editor-debug-local
#>
[CmdletBinding()]
param(
    [string]$OutputDirectory,
    [switch]$SkipSmokeTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Invoke-ManagedBuild {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Validator,

        [Parameter(Mandatory = $true)]
        [string]$RepositoryRoot,

        [Parameter(Mandatory = $true)]
        [string]$Package,

        [string]$Binary,

        [Parameter(Mandatory = $true)]
        [string]$ArtifactOutputDirectory,

        [Parameter(Mandatory = $true)]
        [string]$Artifact
    )

    $arguments = @(
        '-NoProfile'
        '-ExecutionPolicy', 'Bypass'
        '-File', $Validator
        '-RepoRoot', $RepositoryRoot
        '-Package', $Package
    )
    if (-not [string]::IsNullOrWhiteSpace($Binary)) {
        $arguments += @('-Bin', $Binary)
    }
    $arguments += @(
        '-NoDefaultFeatures'
        '-Features', 'target-editor-host'
        '-SkipTest'
        '-ArtifactOutputDirectory', $ArtifactOutputDirectory
        '-PublishArtifact', $Artifact
    )

    & powershell.exe @arguments
    $exitCode = $LASTEXITCODE
    if ($exitCode -ne 0) {
        throw "Managed build failed for package '$Package' with exit code $exitCode."
    }
}

function Resolve-BundleOutputDirectory {
    param(
        [string]$RequestedPath,
        [Parameter(Mandatory = $true)]
        [string]$RepositoryRoot
    )

    if ([string]::IsNullOrWhiteSpace($RequestedPath)) {
        $userProfile = [Environment]::GetFolderPath([Environment+SpecialFolder]::UserProfile)
        if ([string]::IsNullOrWhiteSpace($userProfile)) {
            throw 'Unable to resolve the current user profile for the default output directory.'
        }

        $suffix = '{0}-{1}' -f (Get-Date -Format 'yyyyMMdd-HHmmss'), ([guid]::NewGuid().ToString('N').Substring(0, 8))
        $RequestedPath = Join-Path $userProfile "ZirconBuilds\editor-debug-$suffix"
    }
    elseif (-not [System.IO.Path]::IsPathRooted($RequestedPath)) {
        $RequestedPath = Join-Path $RepositoryRoot $RequestedPath
    }

    $resolvedPath = [System.IO.Path]::GetFullPath($RequestedPath)
    $driveRoot = [System.IO.Path]::GetPathRoot($resolvedPath)
    if ($driveRoot -notmatch '^[A-Za-z]:\\$') {
        throw "OutputDirectory must resolve to a local drive: $resolvedPath"
    }
    if ($driveRoot -in @('D:\', 'E:\', 'F:\')) {
        throw "OutputDirectory must be outside coordinator-managed D/E/F roots: $resolvedPath"
    }

    return $resolvedPath.TrimEnd('\')
}

function Get-Sha256Hex {
    param(
        [Parameter(Mandatory = $true)]
        [string]$LiteralPath
    )

    $stream = [System.IO.File]::OpenRead($LiteralPath)
    $sha256 = [System.Security.Cryptography.SHA256]::Create()
    try {
        return ([System.BitConverter]::ToString($sha256.ComputeHash($stream))).Replace('-', '')
    }
    finally {
        $sha256.Dispose()
        $stream.Dispose()
    }
}

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$validator = Join-Path $repoRoot '.codex\skills\zircon-dev\scripts\validate-matrix.ps1'
$assetSource = Join-Path $repoRoot 'zircon_runtime\assets'
$finalDirectory = Resolve-BundleOutputDirectory -RequestedPath $OutputDirectory -RepositoryRoot $repoRoot
$finalParent = [System.IO.Path]::GetDirectoryName($finalDirectory)
$finalLeaf = [System.IO.Path]::GetFileName($finalDirectory)

if (-not (Test-Path -LiteralPath $validator -PathType Leaf)) {
    throw "Managed Cargo validator was not found: $validator"
}
if (-not (Test-Path -LiteralPath $assetSource -PathType Container)) {
    throw "Runtime asset directory was not found: $assetSource"
}
if ([string]::IsNullOrWhiteSpace($finalParent) -or [string]::IsNullOrWhiteSpace($finalLeaf)) {
    throw "OutputDirectory must name a bundle directory: $finalDirectory"
}
if (Test-Path -LiteralPath $finalDirectory) {
    throw "Refusing to overwrite existing output: $finalDirectory"
}

$partialLeaf = '{0}.partial-{1}' -f $finalLeaf, ([guid]::NewGuid().ToString('N'))
$partialDirectory = Join-Path $finalParent $partialLeaf
$partialCreated = $false

try {
    [System.IO.Directory]::CreateDirectory($finalParent) | Out-Null
    [System.IO.Directory]::CreateDirectory($partialDirectory) | Out-Null
    $partialCreated = $true

    Write-Host 'Building Zircon editor executable...' -ForegroundColor Cyan
    Invoke-ManagedBuild `
        -Validator $validator `
        -RepositoryRoot $repoRoot `
        -Package 'zircon_app' `
        -Binary 'zircon_editor' `
        -ArtifactOutputDirectory $partialDirectory `
        -Artifact 'zircon_editor.exe'

    Write-Host 'Building Zircon runtime library...' -ForegroundColor Cyan
    Invoke-ManagedBuild `
        -Validator $validator `
        -RepositoryRoot $repoRoot `
        -Package 'zircon_runtime' `
        -ArtifactOutputDirectory $partialDirectory `
        -Artifact 'zircon_runtime.dll'

    Write-Host 'Copying runtime assets...' -ForegroundColor Cyan
    Copy-Item -LiteralPath $assetSource -Destination $partialDirectory -Recurse

    $editorPath = Join-Path $partialDirectory 'zircon_editor.exe'
    $runtimePath = Join-Path $partialDirectory 'zircon_runtime.dll'
    $assetPath = Join-Path $partialDirectory 'assets'
    foreach ($requiredPath in @($editorPath, $runtimePath, $assetPath)) {
        if (-not (Test-Path -LiteralPath $requiredPath)) {
            throw "Required bundle content was not produced: $requiredPath"
        }
    }

    if (-not $SkipSmokeTest) {
        Write-Host 'Running editor launch smoke test...' -ForegroundColor Cyan
        Push-Location $partialDirectory
        try {
            $helpOutput = @(& $editorPath --help 2>&1)
            $helpExitCode = $LASTEXITCODE
        }
        finally {
            Pop-Location
        }

        if ($helpExitCode -ne 0) {
            throw "Editor smoke test failed with exit code $helpExitCode."
        }
        if (($helpOutput -join [Environment]::NewLine) -notmatch '(?m)^Usage:\s+zircon_editor') {
            throw 'Editor smoke test did not report the expected zircon_editor usage text.'
        }
    }

    if (Test-Path -LiteralPath $finalDirectory) {
        throw "Output appeared while the build was running; refusing to overwrite it: $finalDirectory"
    }

    Move-Item -LiteralPath $partialDirectory -Destination $finalDirectory
    $partialCreated = $false

    $finalEditor = Join-Path $finalDirectory 'zircon_editor.exe'
    $finalRuntime = Join-Path $finalDirectory 'zircon_runtime.dll'
    $editorInfo = Get-Item -LiteralPath $finalEditor
    $runtimeInfo = Get-Item -LiteralPath $finalRuntime
    $assetCount = @(Get-ChildItem -LiteralPath (Join-Path $finalDirectory 'assets') -File -Recurse).Count

    Write-Host ''
    Write-Host "Editor bundle ready: $finalDirectory" -ForegroundColor Green
    [pscustomobject]@{
        OutputDirectory = $finalDirectory
        EditorBytes = $editorInfo.Length
        EditorSha256 = Get-Sha256Hex -LiteralPath $finalEditor
        RuntimeBytes = $runtimeInfo.Length
        RuntimeSha256 = Get-Sha256Hex -LiteralPath $finalRuntime
        AssetFiles = $assetCount
        SmokeTested = -not $SkipSmokeTest.IsPresent
    }
}
catch {
    if ($partialCreated -and (Test-Path -LiteralPath $partialDirectory)) {
        $partialParent = [System.IO.Path]::GetDirectoryName($partialDirectory)
        $isExpectedParent = [string]::Equals(
            $partialParent,
            $finalParent,
            [System.StringComparison]::OrdinalIgnoreCase)
        $isExpectedLeaf = $partialLeaf.StartsWith("$finalLeaf.partial-", [System.StringComparison]::Ordinal)
        if ($isExpectedParent -and $isExpectedLeaf) {
            Remove-Item -LiteralPath $partialDirectory -Recurse -Force
        }
    }
    throw
}
