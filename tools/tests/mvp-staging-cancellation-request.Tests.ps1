$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$modulePath = Join-Path $repoRoot 'tools\mvp\MvpStagingCancellationRequest.psm1'
$requestScript = Join-Path $repoRoot 'tools\mvp\Request-MvpStagingCancellation.ps1'

Import-Module $modulePath -Force -ErrorAction Stop

function Assert-MvpCancellationRequestThrows {
    param([Parameter(Mandatory)][scriptblock]$Action)

    $threw = $false
    try {
        & $Action | Out-Null
    }
    catch {
        $threw = $true
    }
    $threw | Should Be $true
}

function Write-MvpCancellationFixtureJson {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)]$Value
    )

    [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($Path)) | Out-Null
    [IO.File]::WriteAllText(
        $Path,
        (($Value | ConvertTo-Json -Depth 8 -Compress) + [Environment]::NewLine),
        [Text.UTF8Encoding]::new($false)
    )
}

Describe 'MVP staging cancellation request' {
    It 'reads a bounded request into one exact-size byte snapshot' {
        $source = [IO.File]::ReadAllText($modulePath)

        $source | Should Match '\[byte\[\]\]::new\(\[int\]\$length\)'
        $source | Should Match '\$stream\.Read\(\$bytes, \$offset, \$bytes\.Length - \$offset\)'
        $source | Should Not Match '\[IO\.MemoryStream\]::new\(\)'
        $source | Should Not Match '\.ToArray\(\)'
    }

    It 'atomically publishes and consumes one run-bound cancellation request' {
        $published = Write-MvpStagingCancellationRequest `
            -StagingRoot $TestDrive `
            -RunId 'cancel-fixture' `
            -Reason 'operator_requested'
        $state = New-MvpStagingCancellationProbeState `
            -StagingRoot $TestDrive `
            -RunId 'cancel-fixture'

        (Test-MvpStagingCancellationRequested -State $state) | Should Be $true
        $state.request.reason | Should Be 'operator_requested'
        $state.request.run_id | Should Be 'cancel-fixture'
        $published.bytes | Should BeGreaterThan 0
        ($published.bytes -le 4096) | Should Be $true
        $published.sha256 | Should Match '^[0-9a-f]{64}$'

        Remove-Item -LiteralPath $published.path -Force
        (Test-MvpStagingCancellationRequested -State $state) | Should Be $true
    }

    It 'does not overwrite the first immutable request for a run' {
        Write-MvpStagingCancellationRequest `
            -StagingRoot $TestDrive `
            -RunId 'immutable-cancel-fixture' `
            -Reason 'operator_requested' | Out-Null

        Assert-MvpCancellationRequestThrows { Write-MvpStagingCancellationRequest `
                -StagingRoot $TestDrive `
                -RunId 'immutable-cancel-fixture' `
                -Reason 'replacement_requested' }
    }

    It 'rejects unknown fields, a mismatched run, and oversized input' {
        $unknownPath = Get-MvpStagingCancellationRequestPath -StagingRoot $TestDrive -RunId 'unknown-field-fixture'
        Write-MvpCancellationFixtureJson -Path $unknownPath -Value ([ordered]@{
                schema_version = 1
                request_kind = 'zircon.mvp-staging-cancellation-request'
                run_id = 'unknown-field-fixture'
                reason = 'operator_requested'
                requested_at_utc = [DateTimeOffset]::UtcNow.ToString('o')
                unexpected = $true
            })
        $unknownState = New-MvpStagingCancellationProbeState -StagingRoot $TestDrive -RunId 'unknown-field-fixture'
        Assert-MvpCancellationRequestThrows { Test-MvpStagingCancellationRequested -State $unknownState }

        $mismatchPath = Get-MvpStagingCancellationRequestPath -StagingRoot $TestDrive -RunId 'expected-run'
        Write-MvpCancellationFixtureJson -Path $mismatchPath -Value ([ordered]@{
                schema_version = 1
                request_kind = 'zircon.mvp-staging-cancellation-request'
                run_id = 'different-run'
                reason = 'operator_requested'
                requested_at_utc = [DateTimeOffset]::UtcNow.ToString('o')
            })
        $mismatchState = New-MvpStagingCancellationProbeState -StagingRoot $TestDrive -RunId 'expected-run'
        Assert-MvpCancellationRequestThrows { Test-MvpStagingCancellationRequested -State $mismatchState }

        $oversizedPath = Get-MvpStagingCancellationRequestPath -StagingRoot $TestDrive -RunId 'oversized-run'
        [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($oversizedPath)) | Out-Null
        [IO.File]::WriteAllText($oversizedPath, ('x' * 4097), [Text.UTF8Encoding]::new($false))
        $oversizedState = New-MvpStagingCancellationProbeState -StagingRoot $TestDrive -RunId 'oversized-run'
        Assert-MvpCancellationRequestThrows { Test-MvpStagingCancellationRequested -State $oversizedState }
    }

    It 'exposes the immutable request publisher through the operator CLI' {
        $published = & $requestScript `
            -StagingRoot $TestDrive `
            -RunId 'cli-cancel-fixture' `
            -Reason 'operator_requested'

        Test-Path -LiteralPath $published.path -PathType Leaf | Should Be $true
        $published.sha256 | Should Match '^[0-9a-f]{64}$'
        $state = New-MvpStagingCancellationProbeState -StagingRoot $TestDrive -RunId 'cli-cancel-fixture'
        (Test-MvpStagingCancellationRequested -State $state) | Should Be $true
    }
}
