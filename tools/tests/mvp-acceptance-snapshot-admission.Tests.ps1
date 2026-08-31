Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$admissionModule = Join-Path $repoRoot 'tools\mvp\MvpAcceptanceSnapshotAdmission.psm1'
$snapshotModule = Join-Path $repoRoot 'tools\mvp\MvpAcceptanceStagingSnapshot.psm1'
$treeManifestModule = Join-Path $repoRoot 'tools\mvp\MvpAcceptanceStagingTreeManifest.psm1'
Import-Module $admissionModule -Force -ErrorAction Stop
Import-Module $treeManifestModule -Force -ErrorAction Stop

function New-FixtureAdmissionEntry {
    param(
        [Parameter(Mandatory)][string]$RelativePath,
        [Parameter(Mandatory)][ValidateSet('file', 'directory')][string]$Kind,
        [Int64]$SizeBytes = 0
    )

    return [pscustomobject]@{
        relative_path = $RelativePath
        kind = $Kind
        size_bytes = if ($Kind -eq 'file') { $SizeBytes } else { $null }
    }
}

Describe 'MVP acceptance snapshot admission' {
    It 'emits one exact versioned receipt with the required default budgets' {
        $startedAt = [DateTimeOffset]::UtcNow
        $entries = @(
            New-FixtureAdmissionEntry -RelativePath 'runtime' -Kind directory
            New-FixtureAdmissionEntry -RelativePath 'runtime/host.exe' -Kind file -SizeBytes 7
            New-FixtureAdmissionEntry -RelativePath 'logs/nested/startup.log' -Kind file -SizeBytes 5
        )

        $receipt = New-MvpAcceptanceSnapshotAdmission `
            -Entries $entries `
            -RootPath (Join-Path $TestDrive 'source') `
            -StartedAtUtc $startedAt

        @($receipt.PSObject.Properties.Name) -join ',' |
            Should Be 'schema_version,receipt_kind,source_root_name,started_at_utc,deadline_utc,limits,observed'
        $receipt.schema_version | Should Be 1
        $receipt.receipt_kind | Should Be 'zircon.mvp-acceptance-snapshot-admission'
        $receipt.source_root_name | Should Be 'source'
        [DateTimeOffset]::Parse($receipt.started_at_utc) | Should Be $startedAt
        [DateTimeOffset]::Parse($receipt.deadline_utc) | Should Be $startedAt.AddSeconds(600)
        @($receipt.limits.PSObject.Properties.Name) -join ',' |
            Should Be 'maximum_manifest_bytes,maximum_entry_count,maximum_total_file_bytes,maximum_depth,maximum_duration_seconds'
        $receipt.limits.maximum_manifest_bytes | Should Be 67108864
        $receipt.limits.maximum_entry_count | Should Be 100000
        $receipt.limits.maximum_total_file_bytes | Should Be 17179869184
        $receipt.limits.maximum_depth | Should Be 64
        $receipt.limits.maximum_duration_seconds | Should Be 600
        @($receipt.observed.PSObject.Properties.Name) -join ',' |
            Should Be 'manifest_size_bytes,entry_count,file_count,directory_count,total_file_bytes,maximum_depth'
        $receipt.observed.manifest_size_bytes | Should Be 0
        $receipt.observed.entry_count | Should Be 3
        $receipt.observed.file_count | Should Be 2
        $receipt.observed.directory_count | Should Be 1
        $receipt.observed.total_file_bytes | Should Be 12
        $receipt.observed.maximum_depth | Should Be 3
    }

    It 'rejects a manifest above its entry-count budget' {
        $entries = @(
            New-FixtureAdmissionEntry -RelativePath 'runtime' -Kind directory
            New-FixtureAdmissionEntry -RelativePath 'runtime/host.exe' -Kind file -SizeBytes 1
        )

        { New-MvpAcceptanceSnapshotAdmission -Entries $entries -RootPath $TestDrive -MaximumEntryCount 1 } |
            Should Throw 'entry-count budget of 1'

        $manifestRoot = Join-Path $TestDrive 'entry-count-manifest'
        [IO.Directory]::CreateDirectory($manifestRoot) | Out-Null
        $manifest = [ordered]@{
            schema_version = 1
            entries = @(
                [ordered]@{ path = 'runtime'; kind = 'directory' }
                [ordered]@{ path = 'logs'; kind = 'directory' }
            )
        }
        [IO.File]::WriteAllText(
            (Join-Path $manifestRoot 'staging-tree-manifest.json'),
            ($manifest | ConvertTo-Json -Depth 4),
            [Text.UTF8Encoding]::new($false))
        { Read-MvpAcceptanceStagingTreeManifest -StagingRoot $manifestRoot -MaximumEntryCount 1 } |
            Should Throw 'entry-count budget of 1'
    }

    It 'rejects total file bytes above the budget without integer wraparound' {
        $overBudget = @(
            New-FixtureAdmissionEntry -RelativePath 'runtime/host.exe' -Kind file -SizeBytes 11
        )
        { New-MvpAcceptanceSnapshotAdmission -Entries $overBudget -RootPath $TestDrive -MaximumTotalFileBytes 10 } |
            Should Throw 'file-byte budget of 10'

        $overflow = @(
            New-FixtureAdmissionEntry -RelativePath 'runtime/host.exe' -Kind file -SizeBytes ([Int64]::MaxValue)
            New-FixtureAdmissionEntry -RelativePath 'runtime/host.pdb' -Kind file -SizeBytes 1
        )
        { New-MvpAcceptanceSnapshotAdmission -Entries $overflow -RootPath $TestDrive -MaximumTotalFileBytes ([Int64]::MaxValue) } |
            Should Throw 'file-byte budget of 9223372036854775807'

        $manifestRoot = Join-Path $TestDrive 'byte-budget-manifest'
        [IO.Directory]::CreateDirectory($manifestRoot) | Out-Null
        [IO.File]::WriteAllText(
            (Join-Path $manifestRoot 'staging-tree-manifest.json'),
            '{"schema_version":1,"entries":[]}',
            [Text.UTF8Encoding]::new($false))
        { Read-MvpAcceptanceStagingTreeManifest -StagingRoot $manifestRoot -MaximumManifestBytes 8 } |
            Should Throw 'manifest-byte budget of 8'
    }

    It 'rejects a manifest path above its depth budget' {
        $entries = @(
            New-FixtureAdmissionEntry -RelativePath 'runtime/bin/editor/host.exe' -Kind file -SizeBytes 1
        )

        { New-MvpAcceptanceSnapshotAdmission -Entries $entries -RootPath $TestDrive -MaximumDepth 3 } |
            Should Throw 'depth budget of 3'
    }

    It 'accepts an active receipt and rejects it after its deadline' {
        $startedAt = [DateTimeOffset]::UtcNow
        $receipt = New-MvpAcceptanceSnapshotAdmission `
            -Entries @() `
            -RootPath $TestDrive `
            -MaximumDurationSeconds 10 `
            -StartedAtUtc $startedAt

        { Assert-MvpAcceptanceSnapshotAdmissionActive -Admission $receipt -Phase 'fixture-copy' -NowUtc $startedAt.AddSeconds(10) } |
            Should Not Throw
        { Assert-MvpAcceptanceSnapshotAdmissionActive -Admission $receipt -Phase 'fixture-copy' -NowUtc $startedAt.AddTicks(100000001) } |
            Should Throw "deadline exceeded during 'fixture-copy'"
    }

    It 'wires one manifest read and the same admission through lease census and recursive copy' {
        $source = Get-Content -LiteralPath $snapshotModule -Raw

        $source | Should Match 'Import-Module \(Join-Path \$PSScriptRoot ''MvpAcceptanceSnapshotAdmission\.psm1''\)'
        @([regex]::Matches($source, 'Read-MvpAcceptanceStagingTreeManifest\s+`?\s+-StagingRoot')).Count | Should Be 1
        $source | Should Match 'Open-MvpAcceptanceStagingTreeManifestEntryLeases[\s\S]+-ManifestEntries \$manifestEntries[\s\S]+-Admission \$admission'
        $source | Should Match 'Get-MvpAcceptanceSnapshotAdmissionDefaultLimits'
        $source | Should Match 'Read-MvpAcceptanceStagingTreeManifest\s+`[\s\S]+-MaximumManifestBytes \$admissionLimits\.maximum_manifest_bytes[\s\S]+-MaximumEntryCount \$admissionLimits\.maximum_entry_count'
        $source | Should Match 'admission = \$admission'
        $source | Should Match 'Copy-MvpAcceptanceStagingTree[\s\S]+\[Parameter\(Mandatory\)\]\$Admission'
        $source | Should Match 'Copy-MvpAcceptanceStagingTree[\s\S]+-Admission \$Admission'
        $source | Should Match '-Admission \$SourceSnapshotLease\.admission'
        @([regex]::Matches($source, 'Assert-MvpAcceptanceSnapshotAdmissionActive')).Count |
            Should BeGreaterThan 6
    }

    It 'reuses one bounded copy buffer across the complete recursive snapshot materialization' {
        $tokens = $null
        $errors = $null
        $ast = [Management.Automation.Language.Parser]::ParseFile(
            $snapshotModule,
            [ref]$tokens,
            [ref]$errors)
        $errors.Count | Should Be 0
        $copyTree = $ast.Find(
            { param($node) $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -eq 'Copy-MvpAcceptanceStagingTree' },
            $true)
        $copyItems = $ast.Find(
            { param($node) $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -eq 'Copy-MvpAcceptanceStagingItems' },
            $true)

        $copyItems.Extent.Text | Should Match '\$copyBuffer\s*=\s*\[byte\[\]\]::new\(1048576\)'
        $copyItems.Extent.Text | Should Match 'Copy-MvpAcceptanceStagingTree[\s\S]+-CopyBuffer \$copyBuffer'
        $copyTree.Extent.Text | Should Match '\[Parameter\(Mandatory\)\]\[byte\[\]\]\$CopyBuffer'
        $copyTree.Extent.Text | Should Match 'Copy-MvpAcceptanceStagingTree[\s\S]+-CopyBuffer \$CopyBuffer'
        $copyTree.Extent.Text | Should Not Match '\[byte\[\]\]::new\(1048576\)'
    }
}
