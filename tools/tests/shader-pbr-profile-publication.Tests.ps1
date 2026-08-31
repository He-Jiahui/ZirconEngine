$script:ProfileCaptureManifest = Join-Path $PSScriptRoot "..\profile-capture-manifest.ps1"
$script:PublicationScript = Join-Path $PSScriptRoot "..\shader-pbr-profile-publication.ps1"

if (Test-Path -LiteralPath $script:ProfileCaptureManifest) {
    . $script:ProfileCaptureManifest
}
if (Test-Path -LiteralPath $script:PublicationScript) {
    . $script:PublicationScript
}

Describe "shader PBR profile publication contract" {
    It "creates a contained unique staging root for a stable profile id" {
        Get-Command New-ZirconShaderPbrProfileStagingRoot -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty

        $capturesRoot = Join-Path $TestDrive "profile-captures"
        New-Item -ItemType Directory -Force -Path $capturesRoot | Out-Null
        $stagingRoot = New-ZirconShaderPbrProfileStagingRoot `
            -ProfileCapturesRoot $capturesRoot `
            -ProfileId "shader-pbr-20260824-000001"

        $stagingRoot | Should Be (Join-Path $capturesRoot ".staging\shader-pbr-20260824-000001")
        Test-Path -LiteralPath $stagingRoot -PathType Container | Should Be $true
        {
            New-ZirconShaderPbrProfileStagingRoot `
                -ProfileCapturesRoot $capturesRoot `
                -ProfileId "..\escape"
        } | Should Throw "stable profile id"
    }

    It "holds one profile lease outside the staged artifact closure" {
        Get-Command New-ZirconShaderPbrProfileRunLease -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty
        Get-Command Update-ZirconShaderPbrProfileRunLeaseHeartbeat -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty

        $capturesRoot = Join-Path $TestDrive "profile-captures"
        $profileId = "shader-pbr-20260825-000006"
        $stagingRoot = New-ZirconShaderPbrProfileStagingRoot `
            -ProfileCapturesRoot $capturesRoot `
            -ProfileId $profileId
        $lease = New-ZirconShaderPbrProfileRunLease `
            -ProfileCapturesRoot $capturesRoot `
            -ProfileRoot $stagingRoot `
            -ProfileId $profileId
        try {
            $leasePath = Update-ZirconShaderPbrProfileRunLeaseHeartbeat -Lease $lease
            $state = Get-ZirconShaderPbrProfileRunLeaseState `
                -ProfileCapturesRoot $capturesRoot `
                -ProfileId $profileId

            Test-Path -LiteralPath $leasePath -PathType Leaf | Should Be $true
            @(
                Get-ChildItem -LiteralPath (Split-Path -Parent $leasePath) -File |
                    Where-Object { $_.Name -like '*.backup' }
            ).Count | Should Be 0
            $state.status | Should Be "running"
            $state.profile_root | Should Be ([System.IO.Path]::GetFullPath($stagingRoot))
            $state.lease_token | Should Be $lease.lease_token
            Test-Path -LiteralPath (Join-Path $stagingRoot "profile_lease.json") | Should Be $false
            {
                New-ZirconShaderPbrProfileRunLease `
                    -ProfileCapturesRoot $capturesRoot `
                    -ProfileRoot $stagingRoot `
                    -ProfileId $profileId
            } | Should Throw "already active"
        }
        finally {
            Close-ZirconShaderPbrProfileRunLease -Lease $lease
        }
    }

    It "quarantines a stale lease only after its owner lock is released" {
        Get-Command Invoke-ZirconShaderPbrProfileStaleRunScavenger -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty

        $capturesRoot = Join-Path $TestDrive "profile-captures"
        $profileId = "shader-pbr-20260825-000007"
        $stagingRoot = New-ZirconShaderPbrProfileStagingRoot `
            -ProfileCapturesRoot $capturesRoot `
            -ProfileId $profileId
        Set-Content -LiteralPath (Join-Path $stagingRoot "partial_run.json") -Value "partial" -Encoding UTF8
        $lease = New-ZirconShaderPbrProfileRunLease `
            -ProfileCapturesRoot $capturesRoot `
            -ProfileRoot $stagingRoot `
            -ProfileId $profileId
        Close-ZirconShaderPbrProfileRunLease -Lease $lease

        $paths = Resolve-ZirconShaderPbrProfileLeasePaths `
            -ProfileCapturesRoot $capturesRoot `
            -ProfileId $profileId
        $state = Get-ZirconShaderPbrProfileRunLeaseState `
            -ProfileCapturesRoot $capturesRoot `
            -ProfileId $profileId
        $state.heartbeat_utc = "2000-01-01T00:00:00.0000000Z"
        Set-ZirconShaderPbrProfileRunLeaseState `
            -State $state `
            -Paths $paths `
            -ProfileId $profileId | Out-Null

        $result = @(Invoke-ZirconShaderPbrProfileStaleRunScavenger `
            -ProfileCapturesRoot $capturesRoot `
            -StaleAfterSeconds 60)
        $updated = Get-ZirconShaderPbrProfileRunLeaseState `
            -ProfileCapturesRoot $capturesRoot `
            -ProfileId $profileId

        @($result | Where-Object { $_.profile_id -eq $profileId -and $_.action -eq "quarantined_stale_run" }).Count |
            Should Be 1
        $updated.status | Should Be "quarantined"
        Test-Path -LiteralPath $stagingRoot | Should Be $false
        Test-Path -LiteralPath $updated.quarantine_root -PathType Container | Should Be $true

        $secondResult = @(Invoke-ZirconShaderPbrProfileStaleRunScavenger `
            -ProfileCapturesRoot $capturesRoot `
            -StaleAfterSeconds 60)
        @($secondResult | Where-Object { $_.profile_id -eq $profileId -and $_.action -eq "quarantine_retained" }).Count |
            Should Be 1
    }

    It "recovers a published completion receipt left by a released running lease" {
        $capturesRoot = Join-Path $TestDrive "profile-captures"
        $profileId = "shader-pbr-20260825-000008"
        $stagingRoot = New-ZirconShaderPbrProfileStagingRoot `
            -ProfileCapturesRoot $capturesRoot `
            -ProfileId $profileId
        Set-Content -LiteralPath (Join-Path $stagingRoot "profile_summary.json") -Value "summary" -Encoding UTF8
        $lease = New-ZirconShaderPbrProfileRunLease `
            -ProfileCapturesRoot $capturesRoot `
            -ProfileRoot $stagingRoot `
            -ProfileId $profileId
        $receiptPath = Publish-ZirconShaderPbrProfileCompletion `
            -ProfileCapturesRoot $capturesRoot `
            -ProfileRoot $stagingRoot `
            -ProfileId $profileId
        Close-ZirconShaderPbrProfileRunLease -Lease $lease

        $result = @(Invoke-ZirconShaderPbrProfileStaleRunScavenger `
            -ProfileCapturesRoot $capturesRoot `
            -StaleAfterSeconds 60)
        $updated = Get-ZirconShaderPbrProfileRunLeaseState `
            -ProfileCapturesRoot $capturesRoot `
            -ProfileId $profileId

        @($result | Where-Object { $_.profile_id -eq $profileId -and $_.action -eq "recovered_committed_receipt" }).Count |
            Should Be 1
        $updated.status | Should Be "committed"
        $updated.receipt_path | Should Be ([System.IO.Path]::GetFullPath($receiptPath))
        $updated.quarantine_root | Should Be $null
    }

    It "finishes a quarantined staging move interrupted after terminal state persistence" {
        $capturesRoot = Join-Path $TestDrive "profile-captures"
        $profileId = "shader-pbr-20260825-000009"
        $stagingRoot = New-ZirconShaderPbrProfileStagingRoot `
            -ProfileCapturesRoot $capturesRoot `
            -ProfileId $profileId
        Set-Content -LiteralPath (Join-Path $stagingRoot "partial_run.json") -Value "partial" -Encoding UTF8
        $lease = New-ZirconShaderPbrProfileRunLease `
            -ProfileCapturesRoot $capturesRoot `
            -ProfileRoot $stagingRoot `
            -ProfileId $profileId
        Close-ZirconShaderPbrProfileRunLease -Lease $lease

        $paths = Resolve-ZirconShaderPbrProfileLeasePaths `
            -ProfileCapturesRoot $capturesRoot `
            -ProfileId $profileId
        $state = Get-ZirconShaderPbrProfileRunLeaseState `
            -ProfileCapturesRoot $capturesRoot `
            -ProfileId $profileId
        $state.status = "quarantined"
        $state.terminal_utc = (Get-Date).ToUniversalTime().ToString("o")
        $state.failure = "simulated interruption after quarantine state persistence"
        $state.quarantine_root = Resolve-ZirconShaderPbrPublicationChildPath `
            -Root (Resolve-ZirconShaderPbrPublicationChildPath -Root $capturesRoot -Child ".quarantine") `
            -Child ("{0}-{1}" -f $profileId, $state.lease_token)
        Set-ZirconShaderPbrProfileRunLeaseState `
            -State $state `
            -Paths $paths `
            -ProfileId $profileId | Out-Null

        $result = @(Invoke-ZirconShaderPbrProfileStaleRunScavenger `
            -ProfileCapturesRoot $capturesRoot `
            -StaleAfterSeconds 60)
        $updated = Get-ZirconShaderPbrProfileRunLeaseState `
            -ProfileCapturesRoot $capturesRoot `
            -ProfileId $profileId

        @($result | Where-Object { $_.profile_id -eq $profileId -and $_.action -eq "quarantine_retained" }).Count |
            Should Be 1
        $updated.status | Should Be "quarantined"
        Test-Path -LiteralPath $stagingRoot | Should Be $false
        Test-Path -LiteralPath $updated.quarantine_root -PathType Container | Should Be $true
    }

    It "rejects mutually contradictory terminal lease fields" {
        $capturesRoot = Join-Path $TestDrive "profile-captures"
        $profileId = "shader-pbr-20260825-000010"
        $stagingRoot = New-ZirconShaderPbrProfileStagingRoot `
            -ProfileCapturesRoot $capturesRoot `
            -ProfileId $profileId
        $lease = New-ZirconShaderPbrProfileRunLease `
            -ProfileCapturesRoot $capturesRoot `
            -ProfileRoot $stagingRoot `
            -ProfileId $profileId
        try {
            $paths = Resolve-ZirconShaderPbrProfileLeasePaths `
                -ProfileCapturesRoot $capturesRoot `
                -ProfileId $profileId
            $state = Get-ZirconShaderPbrProfileRunLeaseState `
                -ProfileCapturesRoot $capturesRoot `
                -ProfileId $profileId
            $state.status = "committed"
            $state.terminal_utc = (Get-Date).ToUniversalTime().ToString("o")
            $state.receipt_path = $paths.receipt_path
            $state.failure = "must not coexist with a committed receipt"

            {
                Set-ZirconShaderPbrProfileRunLeaseState `
                    -State $state `
                    -Paths $paths `
                    -ProfileId $profileId
            } | Should Throw "committed lease contains an unexpected terminal field"
        }
        finally {
            Close-ZirconShaderPbrProfileRunLease -Lease $lease
        }
    }

    It "publishes one create-new completion receipt that hashes every staged artifact" {
        Get-Command Publish-ZirconShaderPbrProfileCompletion -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty

        $capturesRoot = Join-Path $TestDrive "profile-captures"
        $profileId = "shader-pbr-20260824-000002"
        $stagingRoot = Join-Path $capturesRoot ".staging\$profileId"
        New-Item -ItemType Directory -Force -Path $stagingRoot | Out-Null
        Set-Content -LiteralPath (Join-Path $stagingRoot "profile_summary.json") -Value "summary" -Encoding UTF8
        Set-Content -LiteralPath (Join-Path $stagingRoot "profile_analysis.json") -Value "analysis" -Encoding UTF8

        $receiptPath = Publish-ZirconShaderPbrProfileCompletion `
            -ProfileCapturesRoot $capturesRoot `
            -ProfileRoot $stagingRoot `
            -ProfileId $profileId

        Test-Path -LiteralPath $receiptPath -PathType Leaf | Should Be $true
        $receipt = Get-Content -LiteralPath $receiptPath -Raw | ConvertFrom-Json
        $receipt.schema_version | Should Be 1
        $receipt.receipt_kind | Should Be "zircon_shader_pbr_profile_completion"
        $receipt.status | Should Be "completed"
        $receipt.profile_id | Should Be $profileId
        $receipt.profile_root | Should Be ([System.IO.Path]::GetFullPath($stagingRoot))
        @($receipt.artifacts).Count | Should Be 2
        @($receipt.artifacts.relative_path) | Should Be @("profile_analysis.json", "profile_summary.json")
        @($receipt.artifacts | Where-Object { $_.sha256 -notmatch '^[0-9a-f]{64}$' }).Count | Should Be 0

        {
            Publish-ZirconShaderPbrProfileCompletion `
                -ProfileCapturesRoot $capturesRoot `
                -ProfileRoot $stagingRoot `
                -ProfileId $profileId
        } | Should Throw "must not overwrite"
    }

    It "rejects a completion receipt when its staging artifact closure changes" {
        Get-Command Assert-ZirconShaderPbrProfileCompletion -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty

        $capturesRoot = Join-Path $TestDrive "profile-captures"
        $profileId = "shader-pbr-20260825-000005"
        $stagingRoot = Join-Path $capturesRoot ".staging\$profileId"
        New-Item -ItemType Directory -Force -Path $stagingRoot | Out-Null
        $artifactPath = Join-Path $stagingRoot "profile_summary.json"
        Set-Content -LiteralPath $artifactPath -Value "summary" -Encoding UTF8
        $receiptPath = Publish-ZirconShaderPbrProfileCompletion `
            -ProfileCapturesRoot $capturesRoot `
            -ProfileRoot $stagingRoot `
            -ProfileId $profileId

        Assert-ZirconShaderPbrProfileCompletion `
            -ProfileCapturesRoot $capturesRoot `
            -ProfileRoot $stagingRoot `
            -ProfileId $profileId `
            -ReceiptPath $receiptPath | Should Be $receiptPath

        Set-Content -LiteralPath $artifactPath -Value "tampered" -Encoding UTF8
        {
            Assert-ZirconShaderPbrProfileCompletion `
                -ProfileCapturesRoot $capturesRoot `
                -ProfileRoot $stagingRoot `
                -ProfileId $profileId `
                -ReceiptPath $receiptPath
        } | Should Throw "SHA-256"
    }

    It "orders staged artifact paths with ordinal semantics" {
        $stagingRoot = Join-Path $TestDrive ".staging\shader-pbr-20260824-000004"
        New-Item -ItemType Directory -Force -Path $stagingRoot | Out-Null
        Set-Content -LiteralPath (Join-Path $stagingRoot "alpha.bin") -Value "alpha" -Encoding UTF8
        Set-Content -LiteralPath (Join-Path $stagingRoot "Zebra.bin") -Value "zebra" -Encoding UTF8

        @((Get-ZirconShaderPbrProfileStagedArtifacts -ProfileRoot $stagingRoot).relative_path) |
            Should Be @("Zebra.bin", "alpha.bin")
    }

    It "writes an immutable incomplete receipt inside the staging root" {
        Get-Command Write-ZirconShaderPbrProfileIncompleteReceipt -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty

        $stagingRoot = Join-Path $TestDrive ".staging\shader-pbr-20260824-000003"
        New-Item -ItemType Directory -Force -Path $stagingRoot | Out-Null
        $receiptPath = Write-ZirconShaderPbrProfileIncompleteReceipt `
            -ProfileRoot $stagingRoot `
            -ProfileId "shader-pbr-20260824-000003" `
            -FailureMessage "simulated capture failure"
        $firstBytes = [System.IO.File]::ReadAllBytes($receiptPath)
        Write-ZirconShaderPbrProfileIncompleteReceipt `
            -ProfileRoot $stagingRoot `
            -ProfileId "shader-pbr-20260824-000003" `
            -FailureMessage "must not replace the original receipt" | Should Be $receiptPath

        [System.IO.File]::ReadAllBytes($receiptPath) | Should Be $firstBytes
        (Get-Content -LiteralPath $receiptPath -Raw | ConvertFrom-Json).status | Should Be "incomplete"
    }
}
