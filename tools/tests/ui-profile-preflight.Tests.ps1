$script:PreflightScript = Join-Path $PSScriptRoot "..\ui-profile-preflight.ps1"
if (Test-Path -LiteralPath $script:PreflightScript) {
    . $script:PreflightScript
}

function New-ZirconUiProfilePreflightFixture {
    $fixtureRoot = "E:\zircon-profiles\pester-ui-profile-preflight-$([guid]::NewGuid().ToString('N'))"
    $repoRoot = Join-Path $fixtureRoot "repo"
    $managedRoot = Join-Path $fixtureRoot "managed-targets"
    $targetDir = Join-Path $managedRoot "profiling"
    $sourcePath = Join-Path $repoRoot "src\ui.rs"
    New-Item -ItemType Directory -Path (Split-Path $sourcePath), $targetDir -Force | Out-Null
    "fn ui() {}" | Set-Content -LiteralPath $sourcePath -Encoding UTF8
    foreach ($binaryName in @("zircon_editor.exe", "zircon_runtime.dll")) {
        $peBytes = [byte[]]::new(128)
        $peBytes[0] = 0x4d
        $peBytes[1] = 0x5a
        $peBytes[0x3c] = 0x40
        $peBytes[0x40] = 0x50
        $peBytes[0x41] = 0x45
        [System.IO.File]::WriteAllBytes((Join-Path $targetDir $binaryName), $peBytes)
    }
    $sourceTime = [datetime]::UtcNow.AddMinutes(-2)
    $binaryTime = [datetime]::UtcNow.AddMinutes(-1)
    (Get-Item -LiteralPath $sourcePath).LastWriteTimeUtc = $sourceTime
    (Get-Item -LiteralPath (Join-Path $targetDir "zircon_editor.exe")).LastWriteTimeUtc = $binaryTime
    (Get-Item -LiteralPath (Join-Path $targetDir "zircon_runtime.dll")).LastWriteTimeUtc = $binaryTime
    return [pscustomobject]@{
        root = $fixtureRoot
        repo_root = $repoRoot
        managed_root = $managedRoot
        target_dir = $targetDir
        source_path = $sourcePath
    }
}

Describe "UI profile preflight" {
    It "accepts a source-bound managed profiling product without launching it" {
        Get-Command Get-ZirconUiProfilePreflight -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty

        $fixture = New-ZirconUiProfilePreflightFixture
        try {
            $result = Get-ZirconUiProfilePreflight `
                -RepoRoot $fixture.repo_root `
                -ProfilingTargetDir $fixture.target_dir `
                -CriticalSourcePaths @("src/ui.rs") `
                -ManagedTargetRoots @($fixture.managed_root)

            $result.schema_version | Should Be 1
            $result.ready | Should Be $true
            @($result.blockers).Count | Should Be 0
            $result.source_binding.critical_source_count | Should Be 1
            $result.binaries.editor.sha256 | Should Not BeNullOrEmpty
            $result.binaries.runtime.sha256 | Should Not BeNullOrEmpty
            $result.tool_binding.preflight.sha256 | Should Not BeNullOrEmpty
            $result.tool_binding.profile_manifest.sha256 | Should Not BeNullOrEmpty
        }
        finally {
            if (Test-Path -LiteralPath $fixture.root) {
                Remove-Item -LiteralPath $fixture.root -Recurse -Force
            }
        }
    }

    It "reports typed missing and stale binary blockers" {
        $fixture = New-ZirconUiProfilePreflightFixture
        try {
            Remove-Item -LiteralPath (Join-Path $fixture.target_dir "zircon_runtime.dll") -Force
            (Get-Item -LiteralPath $fixture.source_path).LastWriteTimeUtc = [datetime]::UtcNow

            $result = Get-ZirconUiProfilePreflight `
                -RepoRoot $fixture.repo_root `
                -ProfilingTargetDir $fixture.target_dir `
                -CriticalSourcePaths @("src/ui.rs") `
                -ManagedTargetRoots @($fixture.managed_root)

            $result.ready | Should Be $false
            (@($result.blockers.code) -contains "stale_editor_binary") | Should Be $true
            (@($result.blockers.code) -contains "missing_runtime_binary") | Should Be $true
        }
        finally {
            if (Test-Path -LiteralPath $fixture.root) {
                Remove-Item -LiteralPath $fixture.root -Recurse -Force
            }
        }
    }

    It "rejects an existing non-PE product artifact" {
        $fixture = New-ZirconUiProfilePreflightFixture
        try {
            "placeholder" | Set-Content -LiteralPath (Join-Path $fixture.target_dir "zircon_runtime.dll") -Encoding UTF8

            $result = Get-ZirconUiProfilePreflight `
                -RepoRoot $fixture.repo_root `
                -ProfilingTargetDir $fixture.target_dir `
                -CriticalSourcePaths @("src/ui.rs") `
                -ManagedTargetRoots @($fixture.managed_root)

            $result.ready | Should Be $false
            (@($result.blockers.code) -contains "invalid_runtime_binary_format") | Should Be $true
        }
        finally {
            if (Test-Path -LiteralPath $fixture.root) {
                Remove-Item -LiteralPath $fixture.root -Recurse -Force
            }
        }
    }

    It "rejects a profiling target outside the managed roots" {
        $fixture = New-ZirconUiProfilePreflightFixture
        try {
            $result = Get-ZirconUiProfilePreflight `
                -RepoRoot $fixture.repo_root `
                -ProfilingTargetDir $fixture.target_dir `
                -CriticalSourcePaths @("src/ui.rs") `
                -ManagedTargetRoots @((Join-Path $fixture.root "different-managed-root"))

            $result.ready | Should Be $false
            (@($result.blockers.code) -contains "unmanaged_profiling_target") | Should Be $true
        }
        finally {
            if (Test-Path -LiteralPath $fixture.root) {
                Remove-Item -LiteralPath $fixture.root -Recurse -Force
            }
        }
    }

    It "fails closed when WPR analysis is requested without system profile privilege" {
        $fixture = New-ZirconUiProfilePreflightFixture
        try {
            Mock Get-ZirconUiProfileToolCapability {
                param([string]$Name)
                return [pscustomobject]@{
                    available = $Name -in @("wpr.exe", "xperf.exe")
                    path = "D:\tools\$Name"
                }
            }
            Mock Test-ZirconUiSystemProfilePrivilege { return $false }

            $result = Get-ZirconUiProfilePreflight `
                -RepoRoot $fixture.repo_root `
                -ProfilingTargetDir $fixture.target_dir `
                -CriticalSourcePaths @("src/ui.rs") `
                -ManagedTargetRoots @($fixture.managed_root) `
                -RequireWpr

            $result.ready | Should Be $false
            (@($result.blockers.code) -contains "wpr_system_profile_privilege_missing") |
                Should Be $true
            $result.tools.wpr.available | Should Be $true
            $result.tools.wpr.system_profile_privilege | Should Be $false
            $result.tools.xperf.available | Should Be $true
        }
        finally {
            if (Test-Path -LiteralPath $fixture.root) {
                Remove-Item -LiteralPath $fixture.root -Recurse -Force
            }
        }
    }

    It "requires xperf whenever WPR sampled CPU evidence is requested" {
        $fixture = New-ZirconUiProfilePreflightFixture
        try {
            Mock Get-ZirconUiProfileToolCapability {
                param([string]$Name)
                return [pscustomobject]@{
                    available = $Name -eq "wpr.exe"
                    path = if ($Name -eq "wpr.exe") { "D:\tools\wpr.exe" } else { $null }
                }
            }
            Mock Test-ZirconUiSystemProfilePrivilege { return $true }

            $result = Get-ZirconUiProfilePreflight `
                -RepoRoot $fixture.repo_root `
                -ProfilingTargetDir $fixture.target_dir `
                -CriticalSourcePaths @("src/ui.rs") `
                -ManagedTargetRoots @($fixture.managed_root) `
                -RequireWpr

            $result.ready | Should Be $false
            (@($result.blockers.code) -contains "xperf_unavailable") | Should Be $true
        }
        finally {
            if (Test-Path -LiteralPath $fixture.root) {
                Remove-Item -LiteralPath $fixture.root -Recurse -Force
            }
        }
    }

    It "exports evidence only to D E or F" {
        Get-Command Export-ZirconUiProfilePreflight -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty

        $fixture = New-ZirconUiProfilePreflightFixture
        try {
            $outputPath = Join-Path $fixture.root "preflight.json"
            $receipt = Export-ZirconUiProfilePreflight `
                -RepoRoot $fixture.repo_root `
                -ProfilingTargetDir $fixture.target_dir `
                -OutputPath $outputPath `
                -CriticalSourcePaths @("src/ui.rs") `
                -ManagedTargetRoots @($fixture.managed_root)

            Test-Path -LiteralPath $receipt.output_path | Should Be $true
            $receipt.ready | Should Be $true

            $errorRecord = $null
            try {
                Export-ZirconUiProfilePreflight `
                    -RepoRoot $fixture.repo_root `
                    -ProfilingTargetDir $fixture.target_dir `
                    -OutputPath "C:\zircon-ui-profile-preflight-forbidden.json" `
                    -CriticalSourcePaths @("src/ui.rs") `
                    -ManagedTargetRoots @($fixture.managed_root)
            }
            catch {
                $errorRecord = $_
            }
            $errorRecord | Should Not BeNullOrEmpty
            $errorRecord.Exception.Message | Should Match "D:, E:, or F:"
        }
        finally {
            if (Test-Path -LiteralPath $fixture.root) {
                Remove-Item -LiteralPath $fixture.root -Recurse -Force
            }
        }
    }
}
