$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$policyModule = Join-Path $repoRoot 'tools\mvp\MvpArtifactStoragePolicy.psm1'
$policyPath = Join-Path $repoRoot 'tools\mvp\mvp-artifact-storage-policy.json'
$productBuilder = Join-Path $repoRoot 'tools\mvp\Build-MvpProductInputs.ps1'
$profilingBuilder = Join-Path $repoRoot 'tools\mvp\Build-RenderExtractProfilingInputs.ps1'
$resourceGenerator = Join-Path $repoRoot 'tools\mvp\New-ResourceManagementScaleProject.ps1'
$resourceChangeSet = Join-Path $repoRoot 'tools\mvp\Set-ResourceManagementScaleProjectChangeSet.ps1'
$resourceBaselinePlan = Join-Path $repoRoot 'tools\mvp\New-ResourceManagementBaselinePlan.ps1'
$resourceBaselineReporter = Join-Path $repoRoot 'tools\mvp\Write-ResourceManagementBaselineReport.ps1'
$resourceComparisonWriter = Join-Path $repoRoot 'tools\mvp\Write-ResourceManagementComparisonReport.ps1'
$renderScaleGenerator = Join-Path $repoRoot 'tools\mvp\New-RenderExtractScaleProject.ps1'
$renderBaselineCapture = Join-Path $repoRoot 'tools\mvp\Capture-RenderExtractBaseline.ps1'
$renderBaselineEvidence = Join-Path $repoRoot 'tools\mvp\RenderExtractBaselineEvidence.psm1'
$mvpStager = Join-Path $repoRoot 'tools\mvp\Stage-MvpProducts.ps1'
$mvpAcceptance = Join-Path $repoRoot 'tools\mvp\Invoke-MvpAcceptance.ps1'
$mvpFixturePaths = Join-Path $repoRoot 'tools\mvp\MvpTestFixturePaths.psm1'

Describe 'MVP artifact storage policy' {
    It 'loads one versioned Windows root and namespace authority' {
        (Test-Path -LiteralPath $policyModule -PathType Leaf) | Should Be $true
        (Test-Path -LiteralPath $policyPath -PathType Leaf) | Should Be $true
        Import-Module $policyModule -Force -ErrorAction Stop

        $snapshot = Get-MvpArtifactStoragePolicySnapshot

        $snapshot.platform | Should Be 'windows'
        $snapshot.default_root_id | Should Be 'windows-local-e'
        @($snapshot.roots).Count | Should Be 3
        @($snapshot.namespaces).Count | Should Be 11
        (@($snapshot.roots.root_id) -join ',') | Should Be 'windows-local-d,windows-local-e,windows-local-f'
        @($snapshot.roots | Where-Object { $_.capability_class -ne 'windows-local-artifact' }).Count | Should Be 0
        (@($snapshot.namespaces.namespace_id) -join ',') | Should Be 'mvp-product-inputs,render-extract-profiling-inputs,resource-management-projects,resource-management-baselines,resource-management-reports,resource-management-comparisons,render-extract-scale-projects,render-extract-baselines,mvp-staging-runs,mvp-acceptance-evidence,mvp-test-fixtures'
        Get-MvpArtifactStorageDefaultRootPath -CapabilityClass 'windows-local-artifact' |
            Should Be 'E:\ZirconBuilds'
    }

    It 'freezes the exact policy bytes into one reusable receipt' {
        Import-Module $policyModule -Force -ErrorAction Stop

        $snapshot = Get-MvpArtifactStoragePolicySnapshot

        @($snapshot.receipt.PSObject.Properties).Count | Should Be 4
        $snapshot.receipt.schema_version | Should Be 1
        $snapshot.receipt.policy_kind | Should Be 'zircon.mvp-artifact-storage-policy'
        $snapshot.receipt.sha256 | Should Be (Get-FileHash -LiteralPath $policyPath -Algorithm SHA256).Hash
        $snapshot.receipt.size_bytes | Should Be ([IO.FileInfo]::new($policyPath).Length)
    }

    It 'creates defaults through the registered root and namespace prefix' {
        Import-Module $policyModule -Force -ErrorAction Stop

        New-MvpArtifactStoragePath -NamespaceId 'mvp-product-inputs' -InstanceId 'fixture' |
            Should Be 'E:\ZirconBuilds\mvp-product-inputs-fixture'
        New-MvpArtifactStoragePath -NamespaceId 'render-extract-profiling-inputs' -InstanceId 'fixture' |
            Should Be 'E:\ZirconBuilds\mvp-product-inputs-profile-fixture'
        New-MvpArtifactStoragePath -NamespaceId 'resource-management-projects' -InstanceId 'fixture' |
            Should Be 'E:\ZirconBuilds\mvp-resource-management-project-fixture'
        New-MvpArtifactStoragePath -NamespaceId 'resource-management-baselines' -InstanceId 'fixture' |
            Should Be 'E:\ZirconBuilds\mvp-resource-management-baseline-fixture'
        New-MvpArtifactStoragePath -NamespaceId 'resource-management-reports' -InstanceId 'fixture' |
            Should Be 'E:\ZirconBuilds\mvp-resource-management-report-fixture'
        New-MvpArtifactStoragePath -NamespaceId 'resource-management-comparisons' -InstanceId 'fixture' |
            Should Be 'E:\ZirconBuilds\mvp-resource-management-comparison-fixture'
        New-MvpArtifactStoragePath -NamespaceId 'render-extract-scale-projects' -InstanceId 'fixture' |
            Should Be 'E:\ZirconBuilds\mvp-render-extract-scale-project-fixture'
        New-MvpArtifactStoragePath -NamespaceId 'render-extract-baselines' -InstanceId 'fixture' |
            Should Be 'E:\ZirconBuilds\mvp-render-extract-baseline-fixture'
        New-MvpArtifactStoragePath -NamespaceId 'mvp-staging-runs' -InstanceId 'fixture' |
            Should Be 'E:\ZirconBuilds\mvp-f0-fixture'
        New-MvpArtifactStoragePath -NamespaceId 'mvp-acceptance-evidence' -InstanceId 'fixture' |
            Should Be 'E:\ZirconBuilds\mvp-f5-evidence-fixture'
        New-MvpArtifactStoragePath -NamespaceId 'mvp-test-fixtures' -InstanceId 'fixture' |
            Should Be 'E:\ZirconBuilds\mvp-test-fixtures-fixture'
    }

    It 'resolves all approved roots with typed policy identity' {
        Import-Module $policyModule -Force -ErrorAction Stop

        foreach ($drive in @('D', 'E', 'F')) {
            $path = "${drive}:\ZirconBuilds\mvp-product-inputs-fixture"
            $resolution = Resolve-MvpArtifactStoragePath -Path $path -NamespaceId 'mvp-product-inputs'

            $resolution.display_path | Should Be $path
            $resolution.root_id | Should Be "windows-local-$($drive.ToLowerInvariant())"
            $resolution.capability_class | Should Be 'windows-local-artifact'
            $resolution.namespace_id | Should Be 'mvp-product-inputs'
            [string]::IsNullOrWhiteSpace($resolution.operation_path) | Should Be $false

            $rootResolution = Resolve-MvpArtifactStorageRootPath `
                -Path "${drive}:\ZirconBuilds\render-extract-project" `
                -CapabilityClass 'windows-local-artifact'
            $rootResolution.root_id | Should Be "windows-local-$($drive.ToLowerInvariant())"
            $rootResolution.capability_class | Should Be 'windows-local-artifact'
            [string]::IsNullOrWhiteSpace($rootResolution.operation_path) | Should Be $false
        }
    }

    It 'probes real filesystem free-space durable-flush and same-volume move capabilities' {
        Import-Module $policyModule -Force -ErrorAction Stop
        $fixtureRoot = Join-Path $TestDrive 'capability-root'
        [IO.Directory]::CreateDirectory($fixtureRoot) | Out-Null
        $fixturePolicyReceipt = [pscustomobject][ordered]@{
            schema_version = 1
            policy_kind = 'zircon.mvp-artifact-storage-policy'
            sha256 = ('A' * 64)
            size_bytes = 1
        }
        $fixtureSnapshot = [pscustomobject][ordered]@{
            receipt = $fixturePolicyReceipt
            platform = 'windows'
            default_root_id = 'fixture-root'
            roots = @([pscustomobject][ordered]@{
                    root_id = 'fixture-root'
                    display_path = [IO.Path]::GetFullPath($fixtureRoot)
                    capability_class = 'windows-local-artifact'
                })
            namespaces = @()
        }

        $evidence = Get-MvpArtifactStorageCapabilityEvidence `
            -RootPath $fixtureRoot `
            -CapabilityClass 'windows-local-artifact' `
            -RequiredFreeSpaceBytes 1 `
            -PolicySnapshot $fixtureSnapshot

        $evidence.schema_version | Should Be 1
        $evidence.capability_kind | Should Be 'zircon.mvp-artifact-storage-capability'
        $evidence.policy.sha256 | Should Be ('A' * 64)
        $evidence.root_id | Should Be 'fixture-root'
        $evidence.capability_class | Should Be 'windows-local-artifact'
        $evidence.required_free_space_bytes | Should Be 1
        ($evidence.available_free_space_bytes -ge $evidence.required_free_space_bytes) | Should Be $true
        [string]::IsNullOrWhiteSpace([string]$evidence.drive_root) | Should Be $false
        [string]::IsNullOrWhiteSpace([string]$evidence.drive_type) | Should Be $false
        [string]::IsNullOrWhiteSpace([string]$evidence.file_system) | Should Be $false
        $evidence.durable_file_flush_supported | Should Be $true
        $evidence.same_volume_atomic_move_supported | Should Be $true
        ([DateTimeOffset]::Parse($evidence.captured_at_utc).Offset -eq [TimeSpan]::Zero) | Should Be $true
        @(Get-ChildItem -LiteralPath $fixtureRoot -Force).Count | Should Be 0

        { Get-MvpArtifactStorageCapabilityEvidence `
                -RootPath $fixtureRoot `
                -CapabilityClass 'windows-local-artifact' `
                -RequiredFreeSpaceBytes ([Int64]::MaxValue) `
                -PolicySnapshot $fixtureSnapshot } | Should Throw 'requires at least'
        @(Get-ChildItem -LiteralPath $fixtureRoot -Force).Count | Should Be 0
    }

    It 'revalidates capability evidence against current policy root and byte budget' {
        Import-Module $policyModule -Force -ErrorAction Stop
        $fixtureRoot = Join-Path $TestDrive 'capability-validation-root'
        [IO.Directory]::CreateDirectory($fixtureRoot) | Out-Null
        $fixtureSnapshot = [pscustomobject][ordered]@{
            receipt = [pscustomobject][ordered]@{
                schema_version = 1
                policy_kind = 'zircon.mvp-artifact-storage-policy'
                sha256 = ('C' * 64)
                size_bytes = 7
            }
            platform = 'windows'
            default_root_id = 'fixture-root'
            roots = @([pscustomobject][ordered]@{
                    root_id = 'fixture-root'
                    display_path = [IO.Path]::GetFullPath($fixtureRoot)
                    capability_class = 'windows-local-artifact'
                })
            namespaces = @()
        }
        $evidence = Get-MvpArtifactStorageCapabilityEvidence `
            -RootPath $fixtureRoot `
            -CapabilityClass 'windows-local-artifact' `
            -RequiredFreeSpaceBytes 1 `
            -PolicySnapshot $fixtureSnapshot

        $validated = Assert-MvpArtifactStorageCapabilityEvidence `
            -Evidence $evidence `
            -ExpectedPath $fixtureRoot `
            -ExpectedRequiredFreeSpaceBytes 1 `
            -PolicySnapshot $fixtureSnapshot
        $validated.root_id | Should Be 'fixture-root'

        $failures = [Collections.Generic.List[object]]::new()
        foreach ($mutation in @('policy', 'free-space', 'move', 'utc')) {
            $candidate = $evidence | ConvertTo-Json -Depth 8 | ConvertFrom-Json
            switch ($mutation) {
                'policy' { $candidate.policy.sha256 = ('D' * 64) }
                'free-space' { $candidate.available_free_space_bytes = 0 }
                'move' { $candidate.same_volume_atomic_move_supported = $false }
                'utc' { $candidate.captured_at_utc = '2026-08-26T08:00:00+08:00' }
            }
            try {
                Assert-MvpArtifactStorageCapabilityEvidence `
                    -Evidence $candidate `
                    -ExpectedPath $fixtureRoot `
                    -ExpectedRequiredFreeSpaceBytes 1 `
                    -PolicySnapshot $fixtureSnapshot | Out-Null
            }
            catch {
                $failures.Add($_) | Out-Null
            }
        }
        $failures.Count | Should Be 4
        ($failures.Exception.Message -join '|') | Should Match 'policy'
        ($failures.Exception.Message -join '|') | Should Match 'available_free_space_bytes'
        ($failures.Exception.Message -join '|') | Should Match 'same_volume_atomic_move_supported'
        ($failures.Exception.Message -join '|') | Should Match 'UTC'
    }

    It 'rejects unapproved roots bare prefixes and cross-namespace paths' {
        Import-Module $policyModule -Force -ErrorAction Stop
        $failures = [Collections.Generic.List[object]]::new()
        foreach ($case in @(
                @{ Path = 'C:\ZirconBuilds\mvp-product-inputs-fixture'; Namespace = 'mvp-product-inputs' },
                @{ Path = 'E:\ZirconBuilds\mvp-product-inputs-'; Namespace = 'mvp-product-inputs' },
                @{ Path = 'E:\ZirconBuilds\mvp-product-inputs-fixture'; Namespace = 'render-extract-profiling-inputs' }
            )) {
            try {
                Resolve-MvpArtifactStoragePath -Path $case.Path -NamespaceId $case.Namespace | Out-Null
            }
            catch {
                $failures.Add($_) | Out-Null
            }
        }

        $failures.Count | Should Be 3
        { Resolve-MvpArtifactStorageRootPath -Path 'C:\ZirconBuilds\render-extract-project' -CapabilityClass 'windows-local-artifact' } |
            Should Throw 'approved'
    }

    It 'rejects unknown properties and duplicate approved root paths' {
        Import-Module $policyModule -Force -ErrorAction Stop
        $failures = [Collections.Generic.List[object]]::new()
        foreach ($mutation in @('unknown-property', 'duplicate-root')) {
            $fixturePath = Join-Path $TestDrive "$mutation.json"
            $fixture = Get-Content -LiteralPath $policyPath -Raw -Encoding UTF8 | ConvertFrom-Json
            if ($mutation -eq 'unknown-property') {
                $fixture | Add-Member -NotePropertyName unexpected_property -NotePropertyValue 'must-fail'
            }
            else {
                $fixture.roots[1].display_path = $fixture.roots[0].display_path
            }
            [IO.File]::WriteAllText($fixturePath, ($fixture | ConvertTo-Json -Depth 8), [Text.UTF8Encoding]::new($false))
            try {
                Get-MvpArtifactStoragePolicySnapshot -PolicyPath $fixturePath | Out-Null
            }
            catch {
                $failures.Add($_) | Out-Null
            }
        }

        $failures.Count | Should Be 2
        $failures[0].Exception.Message | Should Match 'unknown property'
        $failures[1].Exception.Message | Should Match 'duplicate approved root path'
    }

    It 'keeps builder business logic free of physical root literals' {
        $productSource = Get-Content -LiteralPath $productBuilder -Raw
        $profilingSource = Get-Content -LiteralPath $profilingBuilder -Raw
        $resourceGeneratorSource = Get-Content -LiteralPath $resourceGenerator -Raw
        $resourceChangeSetSource = Get-Content -LiteralPath $resourceChangeSet -Raw
        $resourceBaselinePlanSource = Get-Content -LiteralPath $resourceBaselinePlan -Raw
        $resourceBaselineReporterSource = Get-Content -LiteralPath $resourceBaselineReporter -Raw
        $resourceComparisonWriterSource = Get-Content -LiteralPath $resourceComparisonWriter -Raw
        $renderScaleGeneratorSource = Get-Content -LiteralPath $renderScaleGenerator -Raw
        $renderBaselineCaptureSource = Get-Content -LiteralPath $renderBaselineCapture -Raw
        $renderBaselineEvidenceSource = Get-Content -LiteralPath $renderBaselineEvidence -Raw
        $mvpStagerSource = Get-Content -LiteralPath $mvpStager -Raw
        $mvpAcceptanceSource = Get-Content -LiteralPath $mvpAcceptance -Raw
        $mvpFixturePathsSource = Get-Content -LiteralPath $mvpFixturePaths -Raw

        foreach ($source in @(
                $productSource,
                $profilingSource,
                $resourceGeneratorSource,
                $resourceChangeSetSource,
                $resourceBaselinePlanSource,
                $resourceBaselineReporterSource,
                $resourceComparisonWriterSource,
                $renderScaleGeneratorSource,
                $renderBaselineCaptureSource,
                $renderBaselineEvidenceSource,
                $mvpStagerSource,
                $mvpAcceptanceSource,
                $mvpFixturePathsSource
            )) {
            $source | Should Match 'Import-Module .*MvpArtifactStoragePolicy\.psm1'
            $source | Should Not Match '[D-F]:\\ZirconBuilds'
            $source | Should Not Match '\^\[D-F\]:'
        }
        foreach ($source in @(
                $productSource,
                $profilingSource,
                $resourceGeneratorSource,
                $resourceChangeSetSource,
                $resourceBaselinePlanSource,
                $resourceBaselineReporterSource,
                $resourceComparisonWriterSource,
                $renderScaleGeneratorSource,
                $renderBaselineCaptureSource,
                $renderBaselineEvidenceSource
            )) {
            $source | Should Match 'Resolve-MvpArtifactStoragePath'
        }
        $productSource | Should Match 'New-MvpArtifactStoragePath'
        $profilingSource | Should Match 'New-MvpArtifactStoragePath'
        $resourceGeneratorSource | Should Match 'New-MvpArtifactStoragePath'
        $resourceBaselineReporterSource | Should Match 'New-MvpArtifactStoragePath'
        $renderScaleGeneratorSource | Should Match 'New-MvpArtifactStoragePath'
        $renderBaselineCaptureSource | Should Match 'New-MvpArtifactStoragePath'
        $renderBaselineCaptureSource | Should Match 'Resolve-MvpArtifactStorageRootPath'
        $mvpStagerSource | Should Match 'Get-MvpArtifactStorageDefaultRootPath'
        $mvpStagerSource | Should Match 'Resolve-MvpArtifactStorageRootPath'
        $mvpStagerSource | Should Match 'Get-MvpArtifactStorageCapabilityEvidence'
        $mvpStagerSource | Should Match 'storage_capability = \$storageCapabilityEvidence'
        $mvpStagerSource | Should Match "Resolve-MvpArtifactStoragePath.*(?s).*mvp-staging-runs"
        $mvpAcceptanceSource | Should Match 'Resolve-MvpArtifactStoragePath'
        $mvpAcceptanceSource | Should Match 'Assert-MvpArtifactStorageCapabilityEvidence'
        $mvpAcceptanceSource | Should Match 'mvp-staging-runs'
        $mvpAcceptanceSource | Should Match 'mvp-acceptance-evidence'
        $mvpAcceptanceSource | Should Match 'mvp-test-fixtures'
        $mvpFixturePathsSource | Should Match 'Resolve-MvpArtifactStorageRootPath'
        $mvpFixturePathsSource | Should Match "Resolve-MvpArtifactStoragePath.*(?s).*mvp-test-fixtures"
    }
}
