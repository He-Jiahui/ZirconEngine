$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$modulePath = Join-Path $repoRoot 'tools\mvp\MvpRunArtifactBudget.psm1'

Import-Module $modulePath -Force -ErrorAction Stop

Describe 'MVP run artifact budget' {
    It 'captures a versioned baseline with no initial artifact growth' {
        $root = Join-Path $TestDrive 'baseline-root'
        [IO.Directory]::CreateDirectory($root) | Out-Null
        [IO.File]::WriteAllBytes((Join-Path $root 'product.bin'), [byte[]](1..16))

        $budget = New-MvpRunArtifactBudget `
            -Root $root `
            -PolicyId 'test.artifacts.v1' `
            -MaximumAdditionalBytes 1024 `
            -MaximumAdditionalFileCount 4
        $measurement = Measure-MvpRunArtifactBudget -Budget $budget
        $receipt = Get-MvpRunArtifactBudgetPolicyReceipt -Budget $budget

        $budget.schema_version | Should Be 1
        $budget.policy_kind | Should Be 'zircon.mvp-run-artifact-budget'
        $budget.policy_id | Should Be 'test.artifacts.v1'
        $budget.baseline_file_count | Should Be 1
        $budget.baseline_bytes | Should Be 16
        $budget.baseline_sha256 | Should Match '^[0-9a-f]{64}$'
        (@($receipt.PSObject.Properties.Name) -contains 'baseline_lengths') | Should Be $false
        $receipt.baseline_sha256 | Should Be $budget.baseline_sha256
        $measurement.additional_bytes | Should Be 0
        $measurement.additional_file_count | Should Be 0
        $measurement.remaining_bytes | Should Be 1024
        $measurement.remaining_file_count | Should Be 4
    }

    It 'charges file growth and new files without crediting baseline deletions' {
        $root = Join-Path $TestDrive 'growth-root'
        [IO.Directory]::CreateDirectory($root) | Out-Null
        [IO.File]::WriteAllBytes((Join-Path $root 'grows.bin'), [byte[]](1..10))
        [IO.File]::WriteAllBytes((Join-Path $root 'deleted.bin'), [byte[]](1..20))
        $budget = New-MvpRunArtifactBudget `
            -Root $root `
            -PolicyId 'test.artifacts.growth.v1' `
            -MaximumAdditionalBytes 1024 `
            -MaximumAdditionalFileCount 4

        [IO.File]::WriteAllBytes((Join-Path $root 'grows.bin'), [byte[]](1..15))
        [IO.File]::Delete((Join-Path $root 'deleted.bin'))
        [IO.File]::WriteAllBytes((Join-Path $root 'new.bin'), [byte[]](1..7))
        $measurement = Measure-MvpRunArtifactBudget -Budget $budget

        $measurement.additional_bytes | Should Be 12
        $measurement.additional_file_count | Should Be 1
        $measurement.current_file_count | Should Be 2
        $measurement.current_bytes | Should Be 22
    }

    It 'rejects byte and file-count quota overruns' {
        $byteRoot = Join-Path $TestDrive 'byte-limit-root'
        [IO.Directory]::CreateDirectory($byteRoot) | Out-Null
        $byteBudget = New-MvpRunArtifactBudget `
            -Root $byteRoot `
            -PolicyId 'test.artifacts.byte-limit.v1' `
            -MaximumAdditionalBytes 4 `
            -MaximumAdditionalFileCount 4
        [IO.File]::WriteAllBytes((Join-Path $byteRoot 'too-large.bin'), [byte[]](1..5))
        $byteQuotaRejected = $false
        try {
            Assert-MvpRunArtifactBudget -Budget $byteBudget | Out-Null
        }
        catch {
            $byteQuotaRejected = $_.Exception.Message -match 'byte quota'
        }
        $byteQuotaRejected | Should Be $true

        $fileRoot = Join-Path $TestDrive 'file-limit-root'
        [IO.Directory]::CreateDirectory($fileRoot) | Out-Null
        $fileBudget = New-MvpRunArtifactBudget `
            -Root $fileRoot `
            -PolicyId 'test.artifacts.file-limit.v1' `
            -MaximumAdditionalBytes 1024 `
            -MaximumAdditionalFileCount 1
        [IO.File]::WriteAllBytes((Join-Path $fileRoot 'one.bin'), [byte[]]@(1))
        [IO.File]::WriteAllBytes((Join-Path $fileRoot 'two.bin'), [byte[]]@(2))
        $fileQuotaRejected = $false
        try {
            Assert-MvpRunArtifactBudget -Budget $fileBudget | Out-Null
        }
        catch {
            $fileQuotaRejected = $_.Exception.Message -match 'file-count quota'
        }
        $fileQuotaRejected | Should Be $true
    }

    It 'rejects a reparse directory introduced below the budget root' {
        $root = Join-Path $TestDrive 'reparse-root'
        $outside = Join-Path $TestDrive 'reparse-outside'
        [IO.Directory]::CreateDirectory($root) | Out-Null
        [IO.Directory]::CreateDirectory($outside) | Out-Null
        [IO.File]::WriteAllBytes((Join-Path $outside 'outside.bin'), [byte[]](1..8))
        $budget = New-MvpRunArtifactBudget `
            -Root $root `
            -PolicyId 'test.artifacts.reparse.v1' `
            -MaximumAdditionalBytes 1024 `
            -MaximumAdditionalFileCount 4
        $junction = Join-Path $root 'escaped'
        $junctionCommand = Start-Process `
            -FilePath $env:ComSpec `
            -ArgumentList @('/d', '/c', "mklink /J `"$junction`" `"$outside`"") `
            -NoNewWindow `
            -Wait `
            -PassThru
        $junctionCommand.ExitCode | Should Be 0

        $reparseRejected = $false
        try {
            Measure-MvpRunArtifactBudget -Budget $budget | Out-Null
        }
        catch {
            $reparseRejected = $_.Exception.Message -match 'reparse'
        }
        $reparseRejected | Should Be $true
    }

    It 'reuses enumerated file metadata and typed directory stacks during every scan' {
        $source = Get-Content -LiteralPath $modulePath -Raw

        $source | Should Match '\[Collections\.Generic\.Stack\[IO\.DirectoryInfo\]\]::new\(\)'
        $source | Should Match '\[Collections\.Generic\.Stack\[int\]\]::new\(\)'
        $source | Should Match '\$length = \[Int64\]\$entry\.Length'
        $source | Should Match '\$directories\.Push\(\[IO\.DirectoryInfo\]\$entry\)'
        $source | Should Not Match '\[Collections\.Generic\.Stack\[string\]\]::new\(\)'
        $source | Should Not Match '\[Collections\.Generic\.Stack\[object\]\]::new\(\)'
        $source | Should Not Match '\[IO\.FileInfo\]::new\(\$fullPath\)\.Length'
    }

    It 'accumulates heartbeat growth during the filesystem scan instead of a second dictionary pass' {
        $source = Get-Content -LiteralPath $modulePath -Raw

        $source | Should Match '(?s)Get-MvpRunArtifactBudgetFileLengths.*?-RootDirectory \$rootDirectory.*?-BaselineLengths \$Budget\.baseline_lengths'
        $source | Should Match '\[Collections\.Generic\.HashSet\[string\]\]::new\(\[StringComparer\]::OrdinalIgnoreCase\)'
        $source | Should Match 'additional_bytes = \$additionalBytes'
        $source | Should Match 'additional_file_count = \$additionalFileCount'
        $source | Should Not Match 'foreach \(\$entry in \$current\.lengths\.GetEnumerator\(\)\)'
    }

    It 'reuses run-owned traversal and duplicate-detection scratch across heartbeat scans' {
        $root = Join-Path $TestDrive 'scratch-reuse-root'
        [IO.Directory]::CreateDirectory((Join-Path $root 'nested')) | Out-Null
        [IO.File]::WriteAllBytes((Join-Path $root 'nested\fixture.bin'), [byte[]](1..8))
        $budget = New-MvpRunArtifactBudget `
            -Root $root `
            -PolicyId 'test.artifacts.scratch.v1' `
            -MaximumAdditionalBytes 1024 `
            -MaximumAdditionalFileCount 4
        $directories = $budget.scan_directories_scratch
        $directoryDepths = $budget.scan_directory_depths_scratch
        $seenPaths = $budget.scan_seen_paths_scratch

        Measure-MvpRunArtifactBudget -Budget $budget | Out-Null
        Measure-MvpRunArtifactBudget -Budget $budget | Out-Null
        $source = Get-Content -LiteralPath $modulePath -Raw

        ($directories -is [Collections.Generic.Stack[IO.DirectoryInfo]]) | Should Be $true
        ($directoryDepths -is [Collections.Generic.Stack[int]]) | Should Be $true
        ($seenPaths -is [Collections.Generic.HashSet[string]]) | Should Be $true
        [Object]::ReferenceEquals($directories, $budget.scan_directories_scratch) | Should Be $true
        [Object]::ReferenceEquals($directoryDepths, $budget.scan_directory_depths_scratch) | Should Be $true
        [Object]::ReferenceEquals($seenPaths, $budget.scan_seen_paths_scratch) | Should Be $true
        $source | Should Match '-DirectoriesScratch \$Budget\.scan_directories_scratch'
        $source | Should Match '-SeenPathsScratch \$Budget\.scan_seen_paths_scratch'
    }

    It 'reuses one run-owned internal scan result across heartbeat measurements' {
        $root = Join-Path $TestDrive 'result-scratch-reuse-root'
        [IO.Directory]::CreateDirectory($root) | Out-Null
        [IO.File]::WriteAllBytes((Join-Path $root 'fixture.bin'), [byte[]](1..8))
        $budget = New-MvpRunArtifactBudget `
            -Root $root `
            -PolicyId 'test.artifacts.result-scratch.v1' `
            -MaximumAdditionalBytes 1024 `
            -MaximumAdditionalFileCount 4
        $scanResult = $budget.scan_result_scratch

        Measure-MvpRunArtifactBudget -Budget $budget | Out-Null
        Measure-MvpRunArtifactBudget -Budget $budget | Out-Null
        $source = Get-Content -LiteralPath $modulePath -Raw

        ($scanResult -is [pscustomobject]) | Should Be $true
        [Object]::ReferenceEquals($scanResult, $budget.scan_result_scratch) | Should Be $true
        $source | Should Match '-ResultScratch \$Budget\.scan_result_scratch'
        $source | Should Match '\$ResultScratch\.additional_bytes = \$additionalBytes'
    }

    It 'reuses resolved root metadata and enumerated absolute file paths during scans' {
        $source = Get-Content -LiteralPath $modulePath -Raw

        $source | Should Match 'return \$rootInfo'
        $source | Should Match '\[IO\.DirectoryInfo\]\$RootDirectory'
        $source | Should Match '\$directories\.Push\(\$RootDirectory\)'
        $source | Should Match '\$fullPath = \$entry\.FullName'
        $source | Should Not Match '\[IO\.Path\]::GetFullPath\(\$entry\.FullName\)'
    }

    It 'hashes baseline paths through one pooled UTF8 buffer without changing the receipt digest' {
        $module = Get-Module MvpRunArtifactBudget
        $digest = & $module {
            $lengths = [Collections.Generic.Dictionary[string, Int64]]::new(
                [StringComparer]::OrdinalIgnoreCase)
            $lengths.Add('alpha.txt', [Int64]3)
            $lengths.Add('nested/beta.bin', [Int64]5)
            Get-MvpRunArtifactBudgetBaselineSha256 -Lengths $lengths
        }
        $source = Get-Content -LiteralPath $modulePath -Raw

        $digest | Should Be 'a74ae683bf9584b9338c9e438036144519c8957cd613850a1df9fb9850d59d20'
        $source | Should Match "'System\.Buffers\.ArrayPool\x601\[System\.Byte\]' -as \[type\]"
        $source | Should Match 'MvpRunArtifactBudgetByteArrayPool\.Rent'
        $source | Should Match '\[byte\[\]\]::new\(\$pathBufferLength\)'
        $source | Should Match '\[IO\.BinaryWriter\]::new'
        $source | Should Not Match '\[Text\.Encoding\]::UTF8\.GetBytes\(\$path\)'
        $source | Should Not Match '\[BitConverter\]::GetBytes'
    }

    It 'reuses one run-owned root object and containment prefix across heartbeat scans' {
        $root = Join-Path $TestDrive 'root-object-reuse'
        [IO.Directory]::CreateDirectory($root) | Out-Null
        [IO.File]::WriteAllBytes((Join-Path $root 'fixture.bin'), [byte[]](1..8))
        $budget = New-MvpRunArtifactBudget `
            -Root $root `
            -PolicyId 'test.artifacts.root-reuse.v1' `
            -MaximumAdditionalBytes 1024 `
            -MaximumAdditionalFileCount 4
        $rootDirectory = $budget.root_directory
        $rootPrefix = $budget.root_prefix

        Measure-MvpRunArtifactBudget -Budget $budget | Out-Null
        Measure-MvpRunArtifactBudget -Budget $budget | Out-Null
        $source = Get-Content -LiteralPath $modulePath -Raw

        ($rootDirectory -is [IO.DirectoryInfo]) | Should Be $true
        [Object]::ReferenceEquals($rootDirectory, $budget.root_directory) | Should Be $true
        $budget.root_prefix | Should Be $rootPrefix
        $source | Should Match '\$Budget\.root_directory\.Refresh\(\)'
        $source | Should Match '-RootDirectory \$Budget\.root_directory'
        $source | Should Match '-RootPrefix \$Budget\.root_prefix'
    }

    It 'updates a caller-owned measurement object across heartbeat scans' {
        $root = Join-Path $TestDrive 'measurement-object-reuse'
        [IO.Directory]::CreateDirectory($root) | Out-Null
        $budget = New-MvpRunArtifactBudget `
            -Root $root `
            -PolicyId 'test.artifacts.measurement-reuse.v1' `
            -MaximumAdditionalBytes 1024 `
            -MaximumAdditionalFileCount 4
        $measurement = Measure-MvpRunArtifactBudget -Budget $budget
        [IO.File]::WriteAllBytes((Join-Path $root 'fixture.bin'), [byte[]](1..8))

        $updated = Measure-MvpRunArtifactBudget `
            -Budget $budget `
            -ResultScratch $measurement
        $source = Get-Content -LiteralPath $modulePath -Raw
        $supervisorSource = Get-Content `
            -LiteralPath (Join-Path $repoRoot 'tools\mvp\StagedProcessSupervisor.psm1') `
            -Raw

        [Object]::ReferenceEquals($measurement, $updated) | Should Be $true
        $updated.additional_bytes | Should Be 8
        $updated.additional_file_count | Should Be 1
        $source | Should Match '\[AllowNull\(\)\]\$ResultScratch'
        $source | Should Match '\$ResultScratch\.measured_at_utc = '
        $supervisorSource | Should Match '-ResultScratch \$ProcessState\.artifact_budget_measurement'
    }
}

Describe 'MVP staging artifact-budget wiring' {
    It 'shares one run baseline across every supervised product phase' {
        $stagerPath = Join-Path $repoRoot 'tools\mvp\Stage-MvpProducts.ps1'
        $stagerSource = [IO.File]::ReadAllText($stagerPath)

        ([regex]::Matches($stagerSource, 'New-MvpRunArtifactBudget').Count) | Should Be 1
        $stagerSource | Should Match "MvpRunArtifactBudgetPolicyId = 'mvp.staging-run-artifacts.v1'"
        $stagerSource | Should Match '-MaximumAdditionalBytes \(\[Int64\]\$preflight\.evidence_reserve_bytes\)'
        $stagerSource | Should Match '-MaximumAdditionalFileCount \$script:MvpMaximumAdditionalArtifactFileCount'
        ([regex]::Matches($stagerSource, '-ArtifactBudget \$runArtifactBudget').Count) | Should Be 7
        ([regex]::Matches($stagerSource, 'ArtifactBudget = \$runArtifactBudget').Count) | Should Be 1
        $stagerSource | Should Match 'Get-MvpRunArtifactBudgetPolicyReceipt -Budget \$runArtifactBudget'
        $stagerSource | Should Match '\$runArtifactBudgetMeasurement = Assert-MvpRunArtifactBudget -Budget \$runArtifactBudget'
        $stagerSource | Should Match 'policy = \$runArtifactBudgetReceipt'
        $stagerSource | Should Match 'measurement = \$runArtifactBudgetMeasurement'
    }
}
