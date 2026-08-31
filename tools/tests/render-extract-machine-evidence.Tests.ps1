$script:MachineEvidenceModule = Join-Path $PSScriptRoot '..\mvp\RenderExtractMachineEvidence.psm1'

function New-RenderExtractMachineManifestFixture {
    param(
        [Parameter(Mandatory)][string]$Path,
        [switch]$Incomplete
    )

    $categories = @(
        'cpu', 'gpu', 'memory', 'bios', 'os', 'display_modes', 'power_policy',
        'thermal_frequency', 'background_load', 'virtualization'
    )
    $manifest = [ordered]@{
        schema_version = 1
        manifest_kind = 'zircon_performance_machine_snapshot'
        captured_utc = '2026-08-26T00:00:00.0000000Z'
        required_categories = $categories
        all_required_observed = -not $Incomplete
    }
    foreach ($category in $categories) {
        $manifest[$category] = if ($Incomplete -and $category -eq 'thermal_frequency') {
            [ordered]@{ status = 'unavailable'; reason = 'fixture sensor unavailable' }
        }
        else {
            [ordered]@{ status = 'captured'; data = @([ordered]@{ fixture = $category }) }
        }
    }
    [IO.File]::WriteAllText($Path, ($manifest | ConvertTo-Json -Depth 8), [Text.UTF8Encoding]::new($false))
}

function Get-RenderExtractMachineFixtureSha256 {
    param([Parameter(Mandatory)][string]$Path)

    $stream = [IO.File]::OpenRead($Path)
    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        return [BitConverter]::ToString($hasher.ComputeHash($stream)).Replace('-', '')
    }
    finally {
        $hasher.Dispose()
        $stream.Dispose()
    }
}

Describe 'render-extract machine evidence binding' {
    BeforeAll {
        Import-Module $script:MachineEvidenceModule -Force -DisableNameChecking -ErrorAction Stop
    }

    AfterAll {
        Remove-Module RenderExtractMachineEvidence -ErrorAction SilentlyContinue
    }

    It 'resolves the exact hashed machine snapshot inside the evidence directory' {
        $directory = Join-Path $TestDrive 'complete-machine'
        [IO.Directory]::CreateDirectory($directory) | Out-Null
        $path = Join-Path $directory 'machine-manifest.json'
        New-RenderExtractMachineManifestFixture -Path $path
        $snapshotSha256 = Get-RenderExtractMachineFixtureSha256 -Path $path

        $resolved = Resolve-RenderExtractMachineEvidence `
            -Reference ([pscustomobject]@{ path = $path; sha256 = $snapshotSha256 }) `
            -EvidenceDirectory $directory

        $resolved.sha256 | Should Be $snapshotSha256
        $resolved.all_required_observed | Should Be $true
        $resolved.manifest.required_categories.Count | Should Be 10
    }

    It 'preserves an incomplete snapshot without treating unavailable probes as captured' {
        $directory = Join-Path $TestDrive 'incomplete-machine'
        [IO.Directory]::CreateDirectory($directory) | Out-Null
        $path = Join-Path $directory 'machine-manifest.json'
        New-RenderExtractMachineManifestFixture -Path $path -Incomplete
        $snapshotSha256 = Get-RenderExtractMachineFixtureSha256 -Path $path

        $resolved = Resolve-RenderExtractMachineEvidence `
            -Reference ([pscustomobject]@{ path = $path; sha256 = $snapshotSha256 }) `
            -EvidenceDirectory $directory

        $resolved.all_required_observed | Should Be $false
        $resolved.manifest.thermal_frequency.status | Should Be 'unavailable'
    }

    It 'rejects bytes changed after the summary bound the machine snapshot' {
        $directory = Join-Path $TestDrive 'tampered-machine'
        [IO.Directory]::CreateDirectory($directory) | Out-Null
        $path = Join-Path $directory 'machine-manifest.json'
        New-RenderExtractMachineManifestFixture -Path $path
        $snapshotSha256 = Get-RenderExtractMachineFixtureSha256 -Path $path
        [IO.File]::AppendAllText($path, ' ', [Text.UTF8Encoding]::new($false))

        {
            Resolve-RenderExtractMachineEvidence `
                -Reference ([pscustomobject]@{ path = $path; sha256 = $snapshotSha256 }) `
                -EvidenceDirectory $directory
        } | Should Throw 'SHA-256 does not match'
    }

    It 'requires capture and reporter to preserve the machine snapshot binding' {
        $captureSource = Get-Content (Join-Path $PSScriptRoot '..\mvp\Capture-RenderExtractBaseline.ps1') -Raw
        $reportSource = Get-Content (Join-Path $PSScriptRoot '..\mvp\Write-RenderExtractBaselineReport.ps1') -Raw

        $captureSource | Should Match 'New-ZirconPerformanceMachineManifest'
        $captureSource | Should Match "machine-manifest\.json"
        $captureSource | Should Match 'machine_manifest = \$machineEvidence'
        $reportSource | Should Match 'Resolve-RenderExtractMachineEvidence'
        $reportSource | Should Match 'machine_manifest = \$machineEvidence\.manifest'
    }
}
