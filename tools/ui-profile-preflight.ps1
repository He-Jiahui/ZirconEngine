[CmdletBinding()]
param(
    [string]$RepoRoot,
    [string]$ProfilingTargetDir,
    [string]$OutputPath = "E:\zircon-profiles\ui-profile-preflight.json",
    [switch]$RequireWpr
)

$script:ZirconUiProfilePreflightPath = $PSCommandPath
$manifestScript = Join-Path $PSScriptRoot "profile-capture-manifest.ps1"
$script:ZirconUiProfileManifestPath = $manifestScript
if (Test-Path -LiteralPath $manifestScript -PathType Leaf) {
    . $manifestScript
}
$wprScript = Join-Path $PSScriptRoot "ui-profile-wpr.ps1"
$script:ZirconUiProfileWprPath = $wprScript
if (Test-Path -LiteralPath $wprScript -PathType Leaf) {
    . $wprScript
}

function Get-ZirconUiManagedProfileTargetRoots {
    return @(
        "D:\cargo-targets",
        "E:\cargo-targets",
        "F:\cargo-targets",
        "D:\targets",
        "E:\targets",
        "F:\targets",
        "D:\ZirconBuilds",
        "E:\ZirconBuilds",
        "F:\ZirconBuilds"
    )
}

function Test-ZirconUiProfileTargetIsManaged {
    param(
        [Parameter(Mandatory = $true)][string]$TargetDir,
        [Parameter(Mandatory = $true)][string[]]$ManagedTargetRoots
    )

    $resolvedTarget = [System.IO.Path]::GetFullPath($TargetDir).TrimEnd("\")
    foreach ($root in $ManagedTargetRoots) {
        $resolvedRoot = [System.IO.Path]::GetFullPath($root).TrimEnd("\")
        if ($resolvedTarget.Equals($resolvedRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
            return $true
        }
        if ($resolvedTarget.StartsWith(
                $resolvedRoot + [System.IO.Path]::DirectorySeparatorChar,
                [System.StringComparison]::OrdinalIgnoreCase
            )) {
            return $true
        }
    }
    return $false
}

function Get-ZirconUiProfileGitBinding {
    param([Parameter(Mandatory = $true)][string]$ResolvedRepoRoot)

    $head = @(& git -C $ResolvedRepoRoot rev-parse HEAD 2>$null)[0]
    if (-not $head) {
        return [pscustomobject]@{
            head_commit = $null
            dirty_path_count = $null
        }
    }
    $dirtyPaths = @(& git -C $ResolvedRepoRoot status --porcelain --untracked-files=all 2>$null)
    return [pscustomobject]@{
        head_commit = $head.Trim()
        dirty_path_count = $dirtyPaths.Count
    }
}

function Get-ZirconUiProfileToolCapability {
    param([Parameter(Mandatory = $true)][string]$Name)

    $command = Get-Command $Name -ErrorAction SilentlyContinue | Select-Object -First 1
    return [pscustomobject]@{
        available = $null -ne $command
        path = if ($null -ne $command) { $command.Source } else { $null }
    }
}

function Test-ZirconUiProfilePeBinary {
    param([Parameter(Mandatory = $true)][string]$Path)

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        return $false
    }

    $stream = $null
    $reader = $null
    try {
        $stream = [System.IO.File]::OpenRead($Path)
        if ($stream.Length -lt 64) {
            return $false
        }
        $reader = [System.IO.BinaryReader]::new($stream)
        if ($reader.ReadUInt16() -ne 0x5a4d) {
            return $false
        }
        $stream.Position = 0x3c
        $peOffset = [int64]$reader.ReadInt32()
        if ($peOffset -lt 64 -or $peOffset -gt ($stream.Length - 4)) {
            return $false
        }
        $stream.Position = $peOffset
        return $reader.ReadUInt32() -eq 0x00004550
    }
    catch [System.IO.IOException] {
        return $false
    }
    catch [System.ArgumentException] {
        return $false
    }
    finally {
        if ($null -ne $reader) {
            $reader.Dispose()
        }
        elseif ($null -ne $stream) {
            $stream.Dispose()
        }
    }
}

function Get-ZirconUiProfilePreflight {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)][string]$RepoRoot,
        [Parameter(Mandatory = $true)][string]$ProfilingTargetDir,
        [string[]]$CriticalSourcePaths = @(),
        [string[]]$ManagedTargetRoots = @(),
        [switch]$RequireWpr
    )

    $resolvedRepoRoot = (Resolve-Path -LiteralPath $RepoRoot -ErrorAction Stop).Path
    $resolvedTargetDir = [System.IO.Path]::GetFullPath($ProfilingTargetDir)
    if ($CriticalSourcePaths.Count -eq 0) {
        $CriticalSourcePaths = @(Get-ZirconProfileCriticalSourcePaths)
    }
    if ($ManagedTargetRoots.Count -eq 0) {
        $ManagedTargetRoots = @(Get-ZirconUiManagedProfileTargetRoots)
    }

    $blockers = [System.Collections.Generic.List[object]]::new()
    if (-not (Test-ZirconUiProfileTargetIsManaged `
                -TargetDir $resolvedTargetDir `
                -ManagedTargetRoots $ManagedTargetRoots)) {
        $blockers.Add([pscustomobject]@{
                code = "unmanaged_profiling_target"
                path = $resolvedTargetDir
                detail = "Profiling products must reside below a coordinator-managed target root."
            })
    }

    $sourceFingerprints = [System.Collections.Generic.List[object]]::new()
    $newestSourceWriteUtc = $null
    foreach ($relativePath in $CriticalSourcePaths) {
        $absolutePath = Join-Path $resolvedRepoRoot $relativePath
        $fingerprint = Get-ZirconProfileFileFingerprint -Path $absolutePath
        if ($null -eq $fingerprint) {
            $blockers.Add([pscustomobject]@{
                    code = "missing_critical_source"
                    path = $absolutePath
                    detail = "A source-bound profile cannot omit a critical source file."
                })
            continue
        }
        $sourceFingerprints.Add([pscustomobject]@{
                path = $relativePath.Replace("\", "/")
                sha256 = $fingerprint.sha256
                byte_length = $fingerprint.byte_length
                last_write_utc = $fingerprint.last_write_utc
            })
        $writeUtc = [datetime]$fingerprint.last_write_utc
        if ($null -eq $newestSourceWriteUtc -or $writeUtc -gt $newestSourceWriteUtc) {
            $newestSourceWriteUtc = $writeUtc
        }
    }

    $editorPath = Join-Path $resolvedTargetDir "zircon_editor.exe"
    $runtimePath = Join-Path $resolvedTargetDir "zircon_runtime.dll"
    $editorFingerprint = Get-ZirconProfileFileFingerprint -Path $editorPath
    $runtimeFingerprint = Get-ZirconProfileFileFingerprint -Path $runtimePath
    foreach ($binary in @(
            [pscustomobject]@{ name = "editor"; path = $editorPath; fingerprint = $editorFingerprint },
            [pscustomobject]@{ name = "runtime"; path = $runtimePath; fingerprint = $runtimeFingerprint }
        )) {
        if ($null -eq $binary.fingerprint) {
            $blockers.Add([pscustomobject]@{
                    code = "missing_$($binary.name)_binary"
                    path = $binary.path
                    detail = "The managed profiling product is missing."
                })
            continue
        }
        if (-not (Test-ZirconUiProfilePeBinary -Path $binary.path)) {
            $blockers.Add([pscustomobject]@{
                    code = "invalid_$($binary.name)_binary_format"
                    path = $binary.path
                    detail = "The profiling product is not a readable Windows PE executable or DLL."
                })
        }
        if ($null -ne $newestSourceWriteUtc -and
            [datetime]$binary.fingerprint.last_write_utc -lt $newestSourceWriteUtc) {
            $blockers.Add([pscustomobject]@{
                    code = "stale_$($binary.name)_binary"
                    path = $binary.path
                    detail = "The binary predates the newest critical UI source."
                })
        }
    }

    $wpr = Get-ZirconUiProfileToolCapability -Name "wpr.exe"
    $xperf = Get-ZirconUiProfileToolCapability -Name "xperf.exe"
    $wpaExporter = Get-ZirconUiProfileToolCapability -Name "wpaexporter.exe"
    $systemProfilePrivilege = Test-ZirconUiSystemProfilePrivilege
    if ($RequireWpr -and -not $wpr.available) {
        $blockers.Add([pscustomobject]@{
                code = "wpr_unavailable"
                path = $null
                detail = "WPR was required for this capture but is unavailable."
            })
    }
    if ($RequireWpr -and -not $xperf.available) {
        $blockers.Add([pscustomobject]@{
                code = "xperf_unavailable"
                path = $null
                detail = "xperf is required to export function/module sampled CPU evidence from WPR."
            })
    }
    if ($RequireWpr -and -not $systemProfilePrivilege) {
        $blockers.Add([pscustomobject]@{
                code = "wpr_system_profile_privilege_missing"
                path = $null
                detail = "WPR CPU sampling requires an elevated Windows terminal with the system performance profile privilege."
            })
    }

    $gitBinding = Get-ZirconUiProfileGitBinding -ResolvedRepoRoot $resolvedRepoRoot
    return [pscustomobject]@{
        schema_version = 1
        ready = $blockers.Count -eq 0
        source_binding = [pscustomobject]@{
            repo_root = $resolvedRepoRoot
            head_commit = $gitBinding.head_commit
            dirty_path_count = $gitBinding.dirty_path_count
            critical_source_count = $sourceFingerprints.Count
            newest_critical_source_write_utc = if ($null -ne $newestSourceWriteUtc) {
                $newestSourceWriteUtc.ToString("o")
            }
            else {
                $null
            }
            critical_source_files = @($sourceFingerprints)
        }
        profiling_target = [pscustomobject]@{
            path = $resolvedTargetDir
            managed = Test-ZirconUiProfileTargetIsManaged `
                -TargetDir $resolvedTargetDir `
                -ManagedTargetRoots $ManagedTargetRoots
        }
        binaries = [pscustomobject]@{
            editor = $editorFingerprint
            runtime = $runtimeFingerprint
        }
        tools = [pscustomobject]@{
            wpr = [pscustomobject]@{
                available = $wpr.available
                path = $wpr.path
                system_profile_privilege = $systemProfilePrivilege
            }
            xperf = $xperf
            wpaexporter = $wpaExporter
        }
        tool_binding = [pscustomobject]@{
            preflight = Get-ZirconProfileFileFingerprint `
                -Path $script:ZirconUiProfilePreflightPath
            profile_manifest = Get-ZirconProfileFileFingerprint `
                -Path $script:ZirconUiProfileManifestPath
            wpr_capture = Get-ZirconProfileFileFingerprint `
                -Path $script:ZirconUiProfileWprPath
        }
        blockers = @($blockers)
    }
}

function Export-ZirconUiProfilePreflight {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)][string]$RepoRoot,
        [Parameter(Mandatory = $true)][string]$ProfilingTargetDir,
        [Parameter(Mandatory = $true)][string]$OutputPath,
        [string[]]$CriticalSourcePaths = @(),
        [string[]]$ManagedTargetRoots = @(),
        [switch]$RequireWpr
    )

    $resolvedOutputPath = [System.IO.Path]::GetFullPath($OutputPath)
    $outputDrive = [System.IO.Path]::GetPathRoot($resolvedOutputPath).TrimEnd("\")
    if ($outputDrive -notin @("D:", "E:", "F:")) {
        throw "UI profile preflight artifacts must be written below D:, E:, or F:."
    }
    $result = Get-ZirconUiProfilePreflight `
        -RepoRoot $RepoRoot `
        -ProfilingTargetDir $ProfilingTargetDir `
        -CriticalSourcePaths $CriticalSourcePaths `
        -ManagedTargetRoots $ManagedTargetRoots `
        -RequireWpr:$RequireWpr
    New-Item -ItemType Directory -Path (Split-Path $resolvedOutputPath) -Force | Out-Null
    $result | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $resolvedOutputPath -Encoding UTF8
    return [pscustomobject]@{
        schema_version = $result.schema_version
        ready = $result.ready
        blocker_count = @($result.blockers).Count
        output_path = $resolvedOutputPath
    }
}

if ($MyInvocation.InvocationName -ne ".") {
    if ([string]::IsNullOrWhiteSpace($RepoRoot) -or
        [string]::IsNullOrWhiteSpace($ProfilingTargetDir)) {
        throw "RepoRoot and ProfilingTargetDir are required when invoking this script directly."
    }
    Export-ZirconUiProfilePreflight `
        -RepoRoot $RepoRoot `
        -ProfilingTargetDir $ProfilingTargetDir `
        -OutputPath $OutputPath `
        -RequireWpr:$RequireWpr
}
