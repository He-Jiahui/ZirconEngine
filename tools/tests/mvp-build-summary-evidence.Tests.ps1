Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$buildSummaryModule = Join-Path $repoRoot 'tools\mvp\MvpBuildSummaryEvidence.psm1'
$buildGateRegistryModule = Join-Path $repoRoot 'tools\mvp\MvpBuildGateRegistry.psm1'
$buildGateRegistryPath = Join-Path $repoRoot 'tools\mvp\mvp-build-gate-registry.json'
Import-Module $buildSummaryModule -Force -ErrorAction Stop

Describe 'MVP build-summary evidence' {
    It 'encodes validated SHA-256 values through one fixed-size uppercase buffer' {
        $module = Get-Module -Name MvpBuildSummaryEvidence -ErrorAction Stop
        $bytes = [byte[]]@(0x00, 0x0F, 0x10, 0x7F, 0x80, 0xF0, 0xFF)

        $encoded = & $module {
            param([byte[]]$Value)

            ConvertTo-MvpBuildSummaryUpperHex -Bytes $Value
        } $bytes

        $encoded | Should Be '000F107F80F0FF'
        $moduleSource = Get-Content -LiteralPath $buildSummaryModule -Raw
        $moduleSource | Should Match '\[char\[\]\]::new\(\$Bytes\.Length \* 2\)'
        $moduleSource | Should Not Match 'ForEach-Object \{ \$_.ToString\(''X2''\) \}'
    }

    It 'rejects an unknown build-summary schema property' {
        $module = Get-Module -Name MvpBuildSummaryEvidence -ErrorAction Stop
        $failure = $null

        try {
            & $module {
                param($Value)

                Assert-MvpBuildSummaryExactProperties `
                    -Value $Value `
                    -ExpectedNames @('schema_version') `
                    -Label 'fixture build summary'
            } ([pscustomobject]@{
                    schema_version = 1
                    unexpected_property = 'must-fail'
                })
        }
        catch {
            $failure = $_
        }

        $failure | Should Not BeNullOrEmpty
        $failure.Exception.Message | Should Match "unknown property 'unexpected_property'"
        $moduleSource = Get-Content -LiteralPath $buildSummaryModule -Raw
        $moduleSource | Should Match 'Assert-MvpBuildSummaryExactProperties\s+`\s+-Value \$summary'
        $moduleSource | Should Match 'Assert-MvpBuildSummaryExactProperties\s+`\s+-Value \$gate'
        $moduleSource | Should Match 'Assert-MvpBuildSummaryExactProperties\s+`\s+-Value \$evidence'
    }

    It 'rejects build evidence above its caller-owned byte budget' {
        $module = Get-Module -Name MvpBuildSummaryEvidence -ErrorAction Stop
        $path = Join-Path $TestDrive 'oversized-build-evidence.log'
        [IO.File]::WriteAllBytes($path, [byte[]]::new(64))
        $failure = $null

        try {
            & $module {
                param([string]$Path)

                Read-MvpBuildSummaryBoundedBytes `
                    -Path $Path `
                    -MaximumBytes 32 `
                    -Label 'Fixture build evidence'
            } $path
        }
        catch {
            $failure = $_
        }

        $failure | Should Not BeNullOrEmpty
        $failure.Exception.Message | Should Match 'byte budget of 32 bytes'
        $moduleSource = Get-Content -LiteralPath $buildSummaryModule -Raw
        $moduleSource | Should Not Match '\[IO\.File\]::ReadAllBytes'
    }

    It 'loads the exact versioned profile and workspace gate sets from one registry' {
        (Test-Path -LiteralPath $buildGateRegistryModule -PathType Leaf) | Should Be $true
        (Test-Path -LiteralPath $buildGateRegistryPath -PathType Leaf) | Should Be $true
        Import-Module $buildGateRegistryModule -Force -ErrorAction Stop

        $profile = @(Get-MvpBuildGateContract -SummaryKind 'profile-contract')
        $workspace = @(Get-MvpBuildGateContract -SummaryKind 'workspace')

        $profile.Count | Should Be 7
        $workspace.Count | Should Be 2
        (@($profile.gate_id) -join ',') | Should Be 'zircon-app-target-server,zircon-app-target-client-platform,zircon-app-target-editor-host,zircon-app-target-client-shader-pbr-viewer,zircon-runtime-target-client,zircon-runtime-target-editor-host,zircon-runtime-target-server'
        (@($workspace.gate_id) -join ',') | Should Be 'workspace-build,workspace-test'
        $profile[0].command | Should Be 'cargo check -p zircon_app --no-default-features --features target-server --locked'
        $workspace[1].command | Should Be 'cargo test --workspace --locked'
        @($profile + $workspace | Where-Object { $_.cargo_arguments[-1] -ne '--locked' }).Count | Should Be 0
        $acceptanceTestSource = Get-Content -LiteralPath (Join-Path $repoRoot 'tools\tests\mvp-acceptance.Tests.ps1') -Raw
        $acceptanceTestSource | Should Match 'Get-MvpBuildGateRegistrySnapshot'
        $acceptanceTestSource | Should Match 'Get-MvpBuildGateContract\s+`\s+-SummaryKind \$SummaryKind\s+`\s+-RegistrySnapshot \$gateRegistrySnapshot'
        $acceptanceTestSource | Should Not Match "\[ordered\]@\{ gate_id = 'zircon-app-target-server'"
    }

    It 'rejects an unknown build gate registry property' {
        Import-Module $buildGateRegistryModule -Force -ErrorAction Stop
        $fixturePath = Join-Path $TestDrive 'unknown-property-gate-registry.json'
        $fixture = Get-Content -LiteralPath $buildGateRegistryPath -Raw -Encoding UTF8 | ConvertFrom-Json
        $fixture | Add-Member -NotePropertyName unexpected_property -NotePropertyValue 'must-fail'
        [IO.File]::WriteAllText($fixturePath, ($fixture | ConvertTo-Json -Depth 8), [Text.UTF8Encoding]::new($false))

        { Get-MvpBuildGateContract -SummaryKind 'workspace' -RegistryPath $fixturePath } |
            Should Throw "unknown property 'unexpected_property'"
    }

    It 'rejects duplicate build gate IDs across summary groups' {
        Import-Module $buildGateRegistryModule -Force -ErrorAction Stop
        $fixturePath = Join-Path $TestDrive 'duplicate-gate-registry.json'
        $fixture = Get-Content -LiteralPath $buildGateRegistryPath -Raw -Encoding UTF8 | ConvertFrom-Json
        $fixture.summaries[1].gates[0].gate_id = $fixture.summaries[0].gates[0].gate_id
        [IO.File]::WriteAllText($fixturePath, ($fixture | ConvertTo-Json -Depth 8), [Text.UTF8Encoding]::new($false))

        { Get-MvpBuildGateContract -SummaryKind 'workspace' -RegistryPath $fixturePath } |
            Should Throw 'duplicate gate_id'
    }

    It 'rejects a Cargo argument that can collapse the registered argv boundary' {
        Import-Module $buildGateRegistryModule -Force -ErrorAction Stop
        $fixturePath = Join-Path $TestDrive 'unsafe-argv-gate-registry.json'
        $fixture = Get-Content -LiteralPath $buildGateRegistryPath -Raw -Encoding UTF8 | ConvertFrom-Json
        $fixture.summaries[0].gates[0].cargo_arguments[0] = 'check --workspace'
        [IO.File]::WriteAllText($fixturePath, ($fixture | ConvertTo-Json -Depth 8), [Text.UTF8Encoding]::new($false))

        { Get-MvpBuildGateContract -SummaryKind 'profile-contract' -RegistryPath $fixturePath } |
            Should Throw 'must not contain whitespace'
    }

    It 'freezes the exact registry bytes and SHA-256 into one reusable snapshot receipt' {
        Import-Module $buildGateRegistryModule -Force -ErrorAction Stop

        $snapshot = Get-MvpBuildGateRegistrySnapshot
        $profile = @(Get-MvpBuildGateContract -SummaryKind 'profile-contract' -RegistrySnapshot $snapshot)
        $workspace = @(Get-MvpBuildGateContract -SummaryKind 'workspace' -RegistrySnapshot $snapshot)

        @($snapshot.receipt.PSObject.Properties).Count | Should Be 4
        $snapshot.receipt.schema_version | Should Be 1
        $snapshot.receipt.registry_kind | Should Be 'zircon.mvp-build-gate-registry'
        $snapshot.receipt.sha256 | Should Be (Get-FileHash -LiteralPath $buildGateRegistryPath -Algorithm SHA256).Hash
        $snapshot.receipt.size_bytes | Should Be ([IO.FileInfo]::new($buildGateRegistryPath).Length)
        $profile.Count | Should Be 7
        $workspace.Count | Should Be 2
    }

    It 'rejects a build summary receipt detached from the current gate registry snapshot' {
        Import-Module $buildGateRegistryModule -Force -ErrorAction Stop
        $snapshot = Get-MvpBuildGateRegistrySnapshot
        $receipt = ($snapshot.receipt | ConvertTo-Json | ConvertFrom-Json)
        $receipt.sha256 = ('0' * 64)
        $module = Get-Module -Name MvpBuildSummaryEvidence -ErrorAction Stop
        $failure = $null

        try {
            & $module {
                param($Receipt, $Snapshot)
                Assert-MvpBuildGateRegistryReceipt -Receipt $Receipt -ExpectedSnapshot $Snapshot
            } $receipt $snapshot
        }
        catch {
            $failure = $_
        }

        $failure | Should Not BeNullOrEmpty
        $failure.Exception.Message | Should Match 'gate registry receipt sha256 differs'
        $moduleSource = Get-Content -LiteralPath $buildSummaryModule -Raw
        $moduleSource | Should Match "ExpectedNames @\('schema_version', 'summary_kind', 'source_fingerprint', 'status', 'gate_registry', 'gates'\)"
        $moduleSource | Should Match 'schema_version must be the JSON integer 2'
    }

    It 'round-trips one receipt-bound schema v2 summary through the production validator' {
        Import-Module $buildGateRegistryModule -Force -ErrorAction Stop
        $snapshot = Get-MvpBuildGateRegistrySnapshot
        $contracts = @(Get-MvpBuildGateContract -SummaryKind 'workspace' -RegistrySnapshot $snapshot)
        $summaryRoot = Join-Path $TestDrive 'receipt-bound-summary'
        $logRoot = Join-Path $summaryRoot 'logs'
        [IO.Directory]::CreateDirectory($logRoot) | Out-Null
        $gates = @()
        for ($index = 0; $index -lt $contracts.Count; $index++) {
            $contract = $contracts[$index]
            $logPath = Join-Path $logRoot "$($contract.gate_id).log"
            [IO.File]::WriteAllText($logPath, "fixture $($contract.gate_id)`n", [Text.UTF8Encoding]::new($false))
            $startedAt = [DateTimeOffset]::Parse('2026-08-01T00:00:00Z').AddSeconds($index)
            $gates += [ordered]@{
                gate_id = $contract.gate_id
                command = $contract.command
                status = 'passed'
                started_at_utc = $startedAt.ToString('o')
                ended_at_utc = $startedAt.AddMilliseconds(10).ToString('o')
                exit_code = 0
                evidence = [ordered]@{
                    path = "logs/$($contract.gate_id).log"
                    sha256 = (Get-FileHash -LiteralPath $logPath -Algorithm SHA256).Hash
                    size_bytes = [IO.FileInfo]::new($logPath).Length
                }
            }
        }
        $summaryPath = Join-Path $summaryRoot 'workspace-summary.json'
        $summary = [ordered]@{
            schema_version = 2
            summary_kind = 'workspace'
            source_fingerprint = 'fixture-source-fingerprint'
            status = 'passed'
            gate_registry = $snapshot.receipt
            gates = $gates
        }
        [IO.File]::WriteAllText(
            $summaryPath,
            ($summary | ConvertTo-Json -Depth 8),
            [Text.UTF8Encoding]::new($false))

        $validated = Assert-MvpBuildSummaryEvidence `
            -Path $summaryPath `
            -ExpectedKind 'workspace' `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint'

        $validated.manifest_evidence.schema_version | Should Be 2
        $validated.manifest_evidence.gate_count | Should Be 2
        $validated.manifest_evidence.gate_registry.sha256 | Should Be $snapshot.receipt.sha256
        @($validated.gate_artifacts).Count | Should Be 2
    }
}
