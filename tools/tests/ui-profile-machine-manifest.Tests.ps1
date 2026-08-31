$script:RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$script:ProfileCaptureScript = Join-Path $script:RepoRoot "tools\ui-profile-capture.ps1"
$script:ProfileCaptureManifest = Join-Path $script:RepoRoot "tools\profile-capture-manifest.ps1"
$script:ProfileMachineManifest = Join-Path $script:RepoRoot "tools\ui-profile-machine-manifest.ps1"

if (Test-Path -LiteralPath $script:ProfileCaptureManifest) {
    . $script:ProfileCaptureManifest
}
if (Test-Path -LiteralPath $script:ProfileMachineManifest) {
    . $script:ProfileMachineManifest
}

function New-UiProfileMachineManifestFixture {
    $observations = [ordered]@{}
    foreach ($category in @(
            "cpu", "gpu", "memory", "bios", "os", "display_modes", "power_policy",
            "thermal_frequency", "background_load", "virtualization"
        )) {
        $observations[$category] = @{
            status = "captured"
            data = @(@{ fixture = $category })
        }
    }
    return $observations
}

Describe "UI profile machine manifest contract" {
    It "exports a complete machine snapshot beside each UI profile run" {
        Get-Command Export-ZirconUiProfileMachineManifest -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty

        $profileDir = Join-Path $TestDrive "profile"
        $path = Export-ZirconUiProfileMachineManifest `
            -ProfileDir $profileDir `
            -Observations (New-UiProfileMachineManifestFixture)
        $manifest = Get-Content -LiteralPath $path -Raw | ConvertFrom-Json

        $path | Should Be (Join-Path $profileDir "machine_manifest.json")
        $manifest.manifest_kind | Should Be "zircon_performance_machine_snapshot"
        $manifest.all_required_observed | Should Be $true
        @($manifest.required_categories).Count | Should Be 10
    }

    It "captures the machine before WPR and the editor process start" {
        $source = Get-Content -LiteralPath $script:ProfileCaptureScript -Raw
        $exportIndex = $source.IndexOf("Export-ZirconUiProfileMachineManifest")
        $wprIndex = $source.IndexOf('if ($UseWpr -and $runPhase')
        $wprCaptureArgumentIndex = $source.LastIndexOf('-WprCapture $wprCapture')
        $editorIndex = if ($wprCaptureArgumentIndex -ge 0) {
            $source.LastIndexOf("Invoke-EditorCapture", $wprCaptureArgumentIndex)
        }
        else {
            -1
        }

        ($exportIndex -ge 0) | Should Be $true
        ($exportIndex -lt $wprIndex) | Should Be $true
        ($editorIndex -ge 0) | Should Be $true
        ($exportIndex -lt $editorIndex) | Should Be $true
    }

    It "source-binds the machine manifest implementation" {
        Get-Command Get-ZirconProfileCaptureToolPaths -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty

        $toolPaths = @(Get-ZirconProfileCaptureToolPaths)
        ($toolPaths -contains "tools/performance-machine-manifest.ps1") | Should Be $true
        ($toolPaths -contains "tools/ui-profile-machine-manifest.ps1") | Should Be $true
    }
}
