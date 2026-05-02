[CmdletBinding(SupportsShouldProcess = $true)]
param(
    [int]$OlderThanHours = 2
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Get-CargoManifestDirectory {
    param(
        [Parameter(Mandatory = $true)]
        [System.IO.DirectoryInfo]$TargetDirectory
    )

    $cursor = $TargetDirectory.Parent
    while ($null -ne $cursor) {
        $manifestPath = Join-Path -Path $cursor.FullName -ChildPath "Cargo.toml"
        if (Test-Path -LiteralPath $manifestPath) {
            return $cursor
        }

        $cursor = $cursor.Parent
    }

    return $null
}

function Get-TargetDirectoriesFromCandidate {
    param(
        [Parameter(Mandatory = $true)]
        [System.IO.DirectoryInfo]$Candidate
    )

    $targets = [System.Collections.Generic.List[System.IO.DirectoryInfo]]::new()

    $looksLikeTargetRoot = $Candidate.Name -eq "target" -or
        $Candidate.Name -like "cargo-targets*" -or
        (Test-Path -LiteralPath (Join-Path -Path $Candidate.FullName -ChildPath "debug")) -or
        (Test-Path -LiteralPath (Join-Path -Path $Candidate.FullName -ChildPath "release")) -or
        (Test-Path -LiteralPath (Join-Path -Path $Candidate.FullName -ChildPath ".fingerprint"))

    if ($looksLikeTargetRoot) {
        $targets.Add($Candidate)
    }

    $nestedTargetPath = Join-Path -Path $Candidate.FullName -ChildPath "target"
    if (Test-Path -LiteralPath $nestedTargetPath) {
        $nestedTarget = Get-Item -LiteralPath $nestedTargetPath
        if ($nestedTarget -is [System.IO.DirectoryInfo]) {
            $targets.Add($nestedTarget)
        }
    }

    return $targets
}

function Invoke-TargetCleanup {
    param(
        [Parameter(Mandatory = $true)]
        [System.IO.DirectoryInfo]$TargetDirectory,
        [Parameter(Mandatory = $true)]
        [datetime]$CutoffTime,
        [switch]$CargoAvailable
    )

    $staleDirectories = @(
        Get-ChildItem -LiteralPath $TargetDirectory.FullName -Directory -Force |
            Where-Object { $_.LastWriteTime -lt $CutoffTime }
    )

    if ($staleDirectories.Count -eq 0) {
        Write-Host "[Skip] No stale directories: $($TargetDirectory.FullName)"
        return
    }

    Write-Host "[Scan] Found $($staleDirectories.Count) stale directories: $($TargetDirectory.FullName)"
    $manifestDirectory = Get-CargoManifestDirectory -TargetDirectory $TargetDirectory

    $manifestPath = $null
    if ($null -ne $manifestDirectory) {
        $manifestPath = Join-Path -Path $manifestDirectory.FullName -ChildPath "Cargo.toml"
    }

    foreach ($directory in $staleDirectories) {
        $cleaned = $false

        if ($CargoAvailable -and $null -ne $manifestPath) {
            $cargoArgs = @(
                "clean"
                "--manifest-path", $manifestPath
                "--target-dir", $directory.FullName
            )

            if ($PSCmdlet.ShouldProcess($directory.FullName, "cargo $($cargoArgs -join ' ')")) {
                try {
                    Write-Host "[Rust] Running cargo clean: $($directory.FullName)"
                    & cargo @cargoArgs
                    if ($LASTEXITCODE -eq 0) {
                        $cleaned = $true
                    }
                    else {
                        Write-Warning "cargo clean failed for $($directory.FullName), will try direct deletion."
                    }
                }
                catch {
                    Write-Warning "cargo clean threw an exception for $($directory.FullName): $($_.Exception.Message)"
                }
            }
        }

        if (-not $cleaned) {
            if ($PSCmdlet.ShouldProcess($directory.FullName, "Remove-Item -Recurse -Force")) {
                try {
                    Write-Host "[Delete] Removing stale directory: $($directory.FullName)"
                    Remove-Item -LiteralPath $directory.FullName -Recurse -Force
                }
                catch {
                    Write-Warning "Failed to remove $($directory.FullName): $($_.Exception.Message)"
                }
            }
        }
    }
}

$now = Get-Date
$cutoffTime = $now.AddHours(-$OlderThanHours)

$searchCandidates = [System.Collections.Generic.List[System.IO.DirectoryInfo]]::new()

$workspaceTargetPath = Join-Path -Path (Get-Location).Path -ChildPath "target"
if (Test-Path -LiteralPath $workspaceTargetPath) {
    $searchCandidates.Add((Get-Item -LiteralPath $workspaceTargetPath))
}

foreach ($drive in @("D:\", "E:\", "F:\")) {
    if (-not (Test-Path -LiteralPath $drive)) {
        continue
    }

    $rootMatches = Get-ChildItem -LiteralPath $drive -Directory -Force |
        Where-Object {
            $_.Name -like "codex*" -or
            $_.Name -like "target*" -or
            $_.Name -like "cargo-targets*"
        }

    foreach ($match in $rootMatches) {
        $searchCandidates.Add($match)
    }
}

if ($searchCandidates.Count -eq 0) {
    Write-Host "No candidate directories found."
    exit 0
}

$cargoAvailable = $null -ne (Get-Command cargo -ErrorAction SilentlyContinue)
if (-not $cargoAvailable) {
    Write-Warning "cargo is not available; Rust-like targets will fall back to direct stale-directory deletion."
}

$targetDirectories = [System.Collections.Generic.Dictionary[string, System.IO.DirectoryInfo]]::new([System.StringComparer]::OrdinalIgnoreCase)
foreach ($candidate in $searchCandidates) {
    foreach ($target in (Get-TargetDirectoriesFromCandidate -Candidate $candidate)) {
        if (-not $targetDirectories.ContainsKey($target.FullName)) {
            $targetDirectories.Add($target.FullName, $target)
        }
    }
}

if ($targetDirectories.Count -eq 0) {
    Write-Host "No target directories detected."
    exit 0
}

Write-Host "Cleanup started. Threshold: $OlderThanHours hour(s), older than $cutoffTime."
Write-Host "[Roots] Scanning target roots:"
foreach ($target in $targetDirectories.Values) {
    Write-Host "  - $($target.FullName)"
    Invoke-TargetCleanup -TargetDirectory $target -CutoffTime $cutoffTime -CargoAvailable:$cargoAvailable
}

Write-Host "Cleanup completed."
