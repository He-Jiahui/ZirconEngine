<#
.SYNOPSIS
Builds a runnable Zircon editor debug bundle for Windows.

.DESCRIPTION
Builds zircon_editor.exe and zircon_runtime.dll through the repository-managed
Cargo validator, copies runtime assets, and publishes the bundle only after all
steps succeed. The final directory is never overwritten.

.PARAMETER OutputDirectory
Final bundle directory. It must be below an approved
D:\ZirconBuilds, E:\ZirconBuilds, or F:\ZirconBuilds root. Relative paths are
resolved below the first available approved root. When omitted, a unique directory below the
first available approved root is used. A requested parent directory must already exist.

.PARAMETER SkipSmokeTest
Skips the zircon_editor.exe --help launch check. Intended for script tests only.

.EXAMPLE
.\tools\build-editor.ps1

.EXAMPLE
.\tools\build-editor.ps1 -OutputDirectory E:\ZirconBuilds\editor-debug-local
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
        '-MvpProductInputArtifactOutput'
        '-ArtifactOutputDirectory', $ArtifactOutputDirectory
        '-PublishArtifact', $Artifact
    )

    & powershell.exe @arguments
    $exitCode = $LASTEXITCODE
    if ($exitCode -ne 0) {
        throw "Managed build failed for package '$Package' with exit code $exitCode."
    }
}

function Invoke-ArtifactStagingCoordinator {
    param(
        [Parameter(Mandatory = $true)]
        [string]$CoordinatorScript,

        [Parameter(Mandatory = $true)]
        [string[]]$Arguments
    )

    $output = @(& $CoordinatorScript --json artifact @Arguments)
    $exitCode = $LASTEXITCODE
    if ($exitCode -ne 0) {
        throw "Coordinator product staging command failed with exit code ${exitCode}: $($output -join ' ')"
    }
    $json = $output -join "`n"
    if ([string]::IsNullOrWhiteSpace($json)) {
        throw 'Coordinator product staging command returned no JSON result.'
    }
    try {
        return $json | ConvertFrom-Json -ErrorAction Stop
    }
    catch {
        throw "Coordinator product staging command returned invalid JSON: $($_.Exception.Message)"
    }
}

function Assert-ArtifactStagingLease {
    param(
        [Parameter(Mandatory = $true)]
        [object]$Response,

        [Parameter(Mandatory = $true)]
        [string]$LeaseId,

        [Parameter(Mandatory = $true)]
        [string]$ExpectedStatus
    )

    if (
        $null -eq $Response.lease -or
        [string]$Response.lease.leaseId -ne $LeaseId -or
        [string]$Response.lease.status -ne $ExpectedStatus -or
        [int]$Response.lease.ownerPid -ne $PID
    ) {
        throw "Coordinator product staging returned an invalid '$ExpectedStatus' lifecycle result for lease $LeaseId."
    }
}

function Resolve-BundleOutputDirectory {
    param(
        [string]$RequestedPath,
        [Parameter(Mandatory = $true)]
        [string]$RepositoryRoot
    )

    $approvedRoots = @(
        'D:\ZirconBuilds',
        'E:\ZirconBuilds',
        'F:\ZirconBuilds'
    )
    # A configured root must retain its own physical identity; otherwise an E:/D:/F: alias
    # could make an output on another volume appear to be under an approved root.
    $approvedRootResolutions = @(
        $approvedRoots | Where-Object {
            Test-Path -LiteralPath ([System.IO.Path]::GetPathRoot($_)) -PathType Container
        } | ForEach-Object {
            $expectedDisplayPath = [System.IO.Path]::GetFullPath($_).TrimEnd('\\')
            $rootResolution = Resolve-ZirconWindowsPath -Path $_
            if ([string]::Equals(
                    $rootResolution.DisplayPath.TrimEnd('\\'),
                    $expectedDisplayPath,
                    [System.StringComparison]::OrdinalIgnoreCase)) {
                [pscustomobject]@{
                    DisplayPath = $rootResolution.DisplayPath.TrimEnd('\\')
                    OperationalPath = $rootResolution.OperationalPath.TrimEnd('\\')
                }
            }
        }
    )
    if ($approvedRootResolutions.Count -eq 0) {
        throw 'No approved D:\ZirconBuilds, E:\ZirconBuilds, or F:\ZirconBuilds artifact root is available.'
    }

    if ([string]::IsNullOrWhiteSpace($RequestedPath)) {
        $approvedRoot = $approvedRootResolutions | Select-Object -First 1
        $suffix = '{0}-{1}' -f (Get-Date -Format 'yyyyMMdd-HHmmss'), ([guid]::NewGuid().ToString('N').Substring(0, 8))
        $RequestedPath = Join-Path $approvedRoot.DisplayPath "editor-debug-$suffix"
    }
    elseif ($RequestedPath -notmatch '^[A-Za-z]:(?:$|[^\\/])' -and
            -not [System.IO.Path]::IsPathRooted($RequestedPath)) {
        $approvedRoot = $approvedRootResolutions | Select-Object -First 1
        $RequestedPath = Join-Path $approvedRoot.DisplayPath $RequestedPath
    }
    # Resolve existing junctions before comparing roots. GetFullPath alone can collapse a
    # `junction\\..` tail first, making a physical output outside the approved roots appear safe.
    $outputResolution = Resolve-ZirconWindowsPath -Path $RequestedPath -BasePath $RepositoryRoot
    $operationalPath = $outputResolution.OperationalPath.TrimEnd('\\')
    $approvedRoot = $approvedRootResolutions | Where-Object {
        $operationalPath.StartsWith(
            $_.OperationalPath + [System.IO.Path]::DirectorySeparatorChar,
            [System.StringComparison]::OrdinalIgnoreCase)
    } | Select-Object -First 1
    if ($null -eq $approvedRoot) {
        throw "OutputDirectory must resolve below an approved D:\ZirconBuilds, E:\ZirconBuilds, or F:\ZirconBuilds root: $($outputResolution.DisplayPath)"
    }

    return [pscustomobject]@{
        DisplayPath = $outputResolution.DisplayPath.TrimEnd('\\')
        OperationalPath = $operationalPath
        ApprovedRootOperationalPath = $approvedRoot.OperationalPath
    }
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

function Copy-BundleDirectoryTree {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Source,

        [Parameter(Mandatory = $true)]
        [string]$Destination
    )

    $sourceAttributes = [System.IO.File]::GetAttributes($Source)
    if (-not [bool]($sourceAttributes -band [System.IO.FileAttributes]::Directory)) {
        throw "Bundle asset source is not a directory: $Source"
    }
    if ([bool]($sourceAttributes -band [System.IO.FileAttributes]::ReparsePoint)) {
        throw "Refusing to copy a reparse-point bundle asset directory: $Source"
    }

    [System.IO.Directory]::CreateDirectory($Destination) | Out-Null
    foreach ($sourceChild in [System.IO.Directory]::EnumerateFileSystemEntries($Source)) {
        $childAttributes = [System.IO.File]::GetAttributes($sourceChild)
        $destinationChild = Join-ZirconWindowsPath `
            -Path $Destination `
            -ChildPath ([System.IO.Path]::GetFileName($sourceChild))
        if ([bool]($childAttributes -band [System.IO.FileAttributes]::Directory)) {
            if ([bool]($childAttributes -band [System.IO.FileAttributes]::ReparsePoint)) {
                throw "Refusing to copy a reparse-point bundle asset directory: $sourceChild"
            }
            Copy-BundleDirectoryTree -Source $sourceChild -Destination $destinationChild
        }
        elseif ([bool]($childAttributes -band [System.IO.FileAttributes]::ReparsePoint)) {
            throw "Refusing to copy a reparse-point bundle asset file: $sourceChild"
        }
        else {
            [System.IO.File]::Copy($sourceChild, $destinationChild, $false)
        }
    }
}

function Get-BundleDirectoryFileCount {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    $directoryAttributes = [System.IO.File]::GetAttributes($Path)
    if ([bool]($directoryAttributes -band [System.IO.FileAttributes]::ReparsePoint)) {
        throw "Refusing to enumerate a reparse-point bundle directory: $Path"
    }

    [int]$count = 0
    foreach ($childPath in [System.IO.Directory]::EnumerateFileSystemEntries($Path)) {
        $childAttributes = [System.IO.File]::GetAttributes($childPath)
        if ([bool]($childAttributes -band [System.IO.FileAttributes]::Directory)) {
            if ([bool]($childAttributes -band [System.IO.FileAttributes]::ReparsePoint)) {
                throw "Refusing to enumerate a reparse-point bundle directory: $childPath"
            }
            $count += Get-BundleDirectoryFileCount -Path $childPath
        }
        else {
            $count += 1
        }
    }
    return $count
}

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$validator = Join-Path $repoRoot '.codex\skills\zircon-dev\scripts\validate-matrix.ps1'
$pathResolver = Join-Path $repoRoot 'tools\WindowsPathResolver.psm1'
$coordinator = Join-Path $repoRoot 'tools\zircon-session.ps1'
$assetSource = Join-Path $repoRoot 'zircon_runtime\assets'

if (-not (Test-Path -LiteralPath $validator -PathType Leaf)) {
    throw "Managed Cargo validator was not found: $validator"
}
if (-not (Test-Path -LiteralPath $pathResolver -PathType Leaf)) {
    throw "Windows path resolver was not found: $pathResolver"
}
if (-not (Test-Path -LiteralPath $coordinator -PathType Leaf)) {
    throw "Session Coordinator wrapper was not found: $coordinator"
}
Import-Module $pathResolver -Force -DisableNameChecking -ErrorAction Stop

if (-not (Test-Path -LiteralPath $assetSource -PathType Container)) {
    throw "Runtime asset directory was not found: $assetSource"
}
$assetSourceOperationalPath = (Resolve-ZirconWindowsPath -Path $assetSource).OperationalPath
$bundleOutput = Resolve-BundleOutputDirectory -RequestedPath $OutputDirectory -RepositoryRoot $repoRoot
$finalDisplayDirectory = $bundleOutput.DisplayPath
$finalDirectory = $bundleOutput.OperationalPath
$finalParent = [System.IO.Path]::GetDirectoryName($finalDirectory)
$finalLeaf = [System.IO.Path]::GetFileName($finalDirectory)

if ([string]::IsNullOrWhiteSpace($finalParent) -or [string]::IsNullOrWhiteSpace($finalLeaf)) {
    throw "OutputDirectory must name a bundle directory: $finalDisplayDirectory"
}
if (-not [System.IO.Directory]::Exists($finalParent)) {
    throw "OutputDirectory parent must already exist below an approved artifact root: $finalDisplayDirectory"
}
if ([System.IO.Directory]::Exists($finalDirectory) -or [System.IO.File]::Exists($finalDirectory)) {
    throw "Refusing to overwrite existing output: $finalDisplayDirectory"
}

$stagingDirectory = $null
$stagingCreated = $false
$approvedRootLease = $null
$stagingLease = $null
$cleanupRootLease = $null
$productStagingLeaseId = $null
$productStagingStatus = $null

try {
    $acquireResponse = Invoke-ArtifactStagingCoordinator `
        -CoordinatorScript $coordinator `
        -Arguments @(
            'staging-acquire'
            '--purpose', 'build-editor'
            '--final-path', $finalDisplayDirectory
            '--owner-pid', [string]$PID
        )
    if ($null -eq $acquireResponse.lease) {
        throw 'Coordinator product staging acquire response omitted its lease.'
    }
    $productStagingLeaseId = [string]$acquireResponse.lease.leaseId
    if ($productStagingLeaseId -notmatch '^[0-9a-f]{32}$') {
        throw "Coordinator product staging acquire returned an invalid lease ID: $productStagingLeaseId"
    }
    Assert-ArtifactStagingLease `
        -Response $acquireResponse `
        -LeaseId $productStagingLeaseId `
        -ExpectedStatus 'active'
    $productStagingStatus = 'active'
    if (
        [string]$acquireResponse.lease.purpose -ne 'build-editor' -or
        -not [string]::Equals(
            [string]$acquireResponse.lease.finalPath,
            $finalDisplayDirectory,
            [System.StringComparison]::OrdinalIgnoreCase)
    ) {
        throw 'Coordinator product staging lease is not bound to this build output.'
    }
    $stagingResolution = Resolve-ZirconWindowsPath -Path ([string]$acquireResponse.lease.stagingPath)
    $stagingDirectory = $stagingResolution.OperationalPath.TrimEnd('\')
    $stagingParent = [System.IO.Path]::GetDirectoryName($stagingDirectory)
    $stagingLeaf = [System.IO.Path]::GetFileName($stagingDirectory)
    if (
        -not [string]::Equals(
            $stagingParent,
            $bundleOutput.ApprovedRootOperationalPath,
            [System.StringComparison]::OrdinalIgnoreCase) -or
        $stagingLeaf -ne "mvp-product-inputs-build-editor-$productStagingLeaseId"
    ) {
        throw "Coordinator product staging path is outside the approved root: $($stagingResolution.DisplayPath)"
    }
    $approvedRootLease = Open-ZirconWindowsDirectoryLease `
        -Path $bundleOutput.ApprovedRootOperationalPath `
        -ExpectedOperationalPath $bundleOutput.ApprovedRootOperationalPath
    if ([System.IO.Directory]::Exists($stagingDirectory) -or [System.IO.File]::Exists($stagingDirectory)) {
        throw "Generated staging directory already exists: $stagingDirectory"
    }
    [System.IO.Directory]::CreateDirectory($stagingDirectory) | Out-Null
    $stagingCreated = $true
    $stagingLease = Open-ZirconWindowsDirectoryLease `
        -Path $stagingDirectory `
        -ExpectedOperationalPath $stagingDirectory `
        -ForMove `
        -DenyWrite `
        -NoFollow

    Write-Host 'Building Zircon editor executable...' -ForegroundColor Cyan
    Invoke-ManagedBuild `
        -Validator $validator `
        -RepositoryRoot $repoRoot `
        -Package 'zircon_app' `
        -Binary 'zircon_editor' `
        -ArtifactOutputDirectory $stagingDirectory `
        -Artifact 'zircon_editor.exe'

    Write-Host 'Building Zircon runtime library...' -ForegroundColor Cyan
    Invoke-ManagedBuild `
        -Validator $validator `
        -RepositoryRoot $repoRoot `
        -Package 'zircon_runtime' `
        -ArtifactOutputDirectory $stagingDirectory `
        -Artifact 'zircon_runtime.dll'

    Write-Host 'Copying runtime assets...' -ForegroundColor Cyan
    Copy-BundleDirectoryTree `
        -Source $assetSourceOperationalPath `
        -Destination (Join-ZirconWindowsPath -Path $stagingDirectory -ChildPath 'assets')

    $editorPath = Join-ZirconWindowsPath -Path $stagingDirectory -ChildPath 'zircon_editor.exe'
    $runtimePath = Join-ZirconWindowsPath -Path $stagingDirectory -ChildPath 'zircon_runtime.dll'
    $assetPath = Join-ZirconWindowsPath -Path $stagingDirectory -ChildPath 'assets'
    if (-not [System.IO.File]::Exists($editorPath)) {
        throw "Required bundle executable was not produced: $editorPath"
    }
    if (-not [System.IO.File]::Exists($runtimePath)) {
        throw "Required bundle runtime library was not produced: $runtimePath"
    }
    if (-not [System.IO.Directory]::Exists($assetPath)) {
        throw "Required bundle asset directory was not produced: $assetPath"
    }

    if (-not $SkipSmokeTest) {
        Write-Host 'Running editor launch smoke test...' -ForegroundColor Cyan
        $startInfo = [System.Diagnostics.ProcessStartInfo]::new()
        $startInfo.FileName = $editorPath
        $startInfo.Arguments = '--help'
        $startInfo.WorkingDirectory = $stagingDirectory
        $startInfo.UseShellExecute = $false
        $startInfo.RedirectStandardOutput = $true
        $startInfo.RedirectStandardError = $true
        $smokeProcess = [System.Diagnostics.Process]::new()
        try {
            $smokeProcess.StartInfo = $startInfo
            if (-not $smokeProcess.Start()) {
                throw 'Editor smoke test process did not start.'
            }
            $helpOutput = @(
                $smokeProcess.StandardOutput.ReadToEnd()
                $smokeProcess.StandardError.ReadToEnd()
            )
            $smokeProcess.WaitForExit()
            $helpExitCode = $smokeProcess.ExitCode
        }
        finally {
            $smokeProcess.Dispose()
        }

        if ($helpExitCode -ne 0) {
            throw "Editor smoke test failed with exit code $helpExitCode."
        }
        if (($helpOutput -join [Environment]::NewLine) -notmatch '(?m)^Usage:\s+zircon_editor') {
            throw 'Editor smoke test did not report the expected zircon_editor usage text.'
        }
    }

    if ([System.IO.Directory]::Exists($finalDirectory) -or [System.IO.File]::Exists($finalDirectory)) {
        throw "Output appeared while the build was running; refusing to overwrite it: $finalDisplayDirectory"
    }

    $publishingResponse = Invoke-ArtifactStagingCoordinator `
        -CoordinatorScript $coordinator `
        -Arguments @(
            'staging-begin-publish'
            '--lease-id', $productStagingLeaseId
            '--owner-pid', [string]$PID
        )
    Assert-ArtifactStagingLease `
        -Response $publishingResponse `
        -LeaseId $productStagingLeaseId `
        -ExpectedStatus 'publishing'
    $productStagingStatus = 'publishing'

    $finalDirectory = Move-ZirconWindowsLeasedPathWithinRoot `
        -SourceLease $stagingLease `
        -Destination $finalDirectory `
        -ApprovedRoot $bundleOutput.ApprovedRootOperationalPath
    $stagingCreated = $false

    $publishedResponse = Invoke-ArtifactStagingCoordinator `
        -CoordinatorScript $coordinator `
        -Arguments @(
            'staging-complete-publish'
            '--lease-id', $productStagingLeaseId
            '--owner-pid', [string]$PID
        )
    Assert-ArtifactStagingLease `
        -Response $publishedResponse `
        -LeaseId $productStagingLeaseId `
        -ExpectedStatus 'published'
    $productStagingStatus = 'published'

    $finalEditor = Join-ZirconWindowsPath -Path $finalDirectory -ChildPath 'zircon_editor.exe'
    $finalRuntime = Join-ZirconWindowsPath -Path $finalDirectory -ChildPath 'zircon_runtime.dll'
    $editorInfo = [System.IO.FileInfo]::new($finalEditor)
    $runtimeInfo = [System.IO.FileInfo]::new($finalRuntime)
    $assetCount = Get-BundleDirectoryFileCount `
        -Path (Join-ZirconWindowsPath -Path $finalDirectory -ChildPath 'assets')

    Write-Host ''
    Write-Host "Editor bundle ready: $finalDisplayDirectory" -ForegroundColor Green
    [pscustomobject]@{
        OutputDirectory = $finalDisplayDirectory
        EditorBytes = $editorInfo.Length
        EditorSha256 = Get-Sha256Hex -LiteralPath $finalEditor
        RuntimeBytes = $runtimeInfo.Length
        RuntimeSha256 = Get-Sha256Hex -LiteralPath $finalRuntime
        AssetFiles = $assetCount
        SmokeTested = -not $SkipSmokeTest.IsPresent
    }
}
catch {
    $primaryFailure = $_
    if ($stagingCreated) {
        if ($null -eq $stagingLease) {
            Write-Warning 'Skipping staging cleanup because its original directory lease was not acquired.'
        }
        else {
            try {
                $approvedRootLease.Dispose()
                $approvedRootLease = $null
                $cleanupRootLease = Open-ZirconWindowsDirectoryLease `
                    -Path $bundleOutput.ApprovedRootOperationalPath `
                    -ExpectedOperationalPath $bundleOutput.ApprovedRootOperationalPath `
                    -DenyWrite `
                    -NoFollow
                Remove-ZirconWindowsLeasedDirectoryTree -Lease $stagingLease
                $stagingLease.Dispose()
                $stagingLease = $null
            }
            catch {
                Write-Warning "Skipping staging cleanup because its held directory could not be deleted: $($_.Exception.Message)"
            }
        }
    }
    if (
        $null -ne $productStagingLeaseId -and
        $productStagingStatus -in @('active', 'publishing') -and
        ($null -eq $stagingDirectory -or
            (-not [System.IO.Directory]::Exists($stagingDirectory) -and
             -not [System.IO.File]::Exists($stagingDirectory))) -and
        -not [System.IO.Directory]::Exists($finalDirectory) -and
        -not [System.IO.File]::Exists($finalDirectory)
    ) {
        try {
            $releaseResponse = Invoke-ArtifactStagingCoordinator `
                -CoordinatorScript $coordinator `
                -Arguments @(
                    'staging-release'
                    '--lease-id', $productStagingLeaseId
                    '--owner-pid', [string]$PID
                )
            Assert-ArtifactStagingLease `
                -Response $releaseResponse `
                -LeaseId $productStagingLeaseId `
                -ExpectedStatus 'released'
            $productStagingStatus = 'released'
        }
        catch {
            Write-Warning "Product staging lease release failed after the primary build error: $($_.Exception.Message)"
        }
    }
    throw $primaryFailure
}
finally {
    if ($null -ne $cleanupRootLease) {
        $cleanupRootLease.Dispose()
    }
    if ($null -ne $stagingLease) {
        $stagingLease.Dispose()
    }
    if ($null -ne $approvedRootLease) {
        $approvedRootLease.Dispose()
    }
}
