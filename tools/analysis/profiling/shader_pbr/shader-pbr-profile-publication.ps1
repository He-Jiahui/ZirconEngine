Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Assert-ZirconShaderPbrStableProfileId {
    param([Parameter(Mandatory = $true)][string]$ProfileId)

    if ($ProfileId -notmatch '^[a-z][a-z0-9-]{2,127}$') {
        throw "Shader PBR profile publication requires a stable profile id."
    }
}

function Resolve-ZirconShaderPbrPublicationChildPath {
    param(
        [Parameter(Mandatory = $true)][string]$Root,
        [Parameter(Mandatory = $true)][string]$Child
    )

    $resolvedRoot = [System.IO.Path]::GetFullPath($Root).TrimEnd(
        [char[]]@(
            [System.IO.Path]::DirectorySeparatorChar,
            [System.IO.Path]::AltDirectorySeparatorChar
        )
    )
    $candidate = [System.IO.Path]::GetFullPath((Join-Path $resolvedRoot $Child))
    $rootPrefix = $resolvedRoot + [System.IO.Path]::DirectorySeparatorChar
    if (-not $candidate.StartsWith($rootPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Shader PBR profile publication path escapes its root: $Child"
    }
    return $candidate
}

function Get-ZirconShaderPbrPublicationRelativePath {
    param(
        [Parameter(Mandatory = $true)][string]$Root,
        [Parameter(Mandatory = $true)][string]$Path
    )

    $resolvedRoot = [System.IO.Path]::GetFullPath($Root).TrimEnd(
        [char[]]@(
            [System.IO.Path]::DirectorySeparatorChar,
            [System.IO.Path]::AltDirectorySeparatorChar
        )
    )
    $resolvedPath = [System.IO.Path]::GetFullPath($Path)
    $rootPrefix = $resolvedRoot + [System.IO.Path]::DirectorySeparatorChar
    if (-not $resolvedPath.StartsWith($rootPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Shader PBR profile staged artifact escapes its root: $resolvedPath"
    }
    return $resolvedPath.Substring($rootPrefix.Length).Replace("\", "/")
}

function Write-ZirconShaderPbrCreateNewJson {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)]$Value,
        [Parameter(Mandatory = $true)][string]$Description
    )

    if (Test-Path -LiteralPath $Path) {
        throw "Shader PBR profile $Description must not overwrite an existing receipt: $Path"
    }
    $parent = Split-Path -Parent $Path
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
    $stream = $null
    $writer = $null
    try {
        $stream = [System.IO.File]::Open(
            $Path,
            [System.IO.FileMode]::CreateNew,
            [System.IO.FileAccess]::Write,
            [System.IO.FileShare]::Read
        )
        $writer = [System.IO.StreamWriter]::new(
            $stream,
            [System.Text.UTF8Encoding]::new($false),
            1024,
            $true
        )
        $writer.Write(($Value | ConvertTo-Json -Depth 12))
        $writer.Write("`n")
        $writer.Flush()
        $stream.Flush($true)
    }
    catch [System.IO.IOException] {
        if (Test-Path -LiteralPath $Path) {
            throw "Shader PBR profile $Description must not overwrite an existing receipt: $Path"
        }
        throw
    }
    finally {
        if ($null -ne $writer) {
            $writer.Dispose()
        }
        if ($null -ne $stream) {
            $stream.Dispose()
        }
    }
    return [System.IO.Path]::GetFullPath($Path)
}

function Write-ZirconShaderPbrReplaceJson {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)]$Value,
        [Parameter(Mandatory = $true)][string]$Description
    )

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Shader PBR profile $Description cannot replace a missing receipt: $Path"
    }
    $temporaryPath = "$Path.$([guid]::NewGuid().ToString("N")).tmp"
    $backupPath = "$Path.$([guid]::NewGuid().ToString("N")).backup"
    try {
        Write-ZirconShaderPbrCreateNewJson `
            -Path $temporaryPath `
            -Value $Value `
            -Description "$Description replacement" | Out-Null
        # .NET requires a non-empty backup path on the supported PowerShell runtimes.
        # The backup is only an atomic-replace implementation detail and never published.
        [System.IO.File]::Replace($temporaryPath, $Path, $backupPath)
    }
    finally {
        if (Test-Path -LiteralPath $temporaryPath) {
            Remove-Item -LiteralPath $temporaryPath -Force
        }
        if (Test-Path -LiteralPath $backupPath) {
            Remove-Item -LiteralPath $backupPath -Force
        }
    }
    return [System.IO.Path]::GetFullPath($Path)
}

function Resolve-ZirconShaderPbrProfileLeasePaths {
    param(
        [Parameter(Mandatory = $true)][string]$ProfileCapturesRoot,
        [Parameter(Mandatory = $true)][string]$ProfileId
    )

    Assert-ZirconShaderPbrStableProfileId -ProfileId $ProfileId
    $capturesRoot = [System.IO.Path]::GetFullPath($ProfileCapturesRoot)
    $leasesRoot = Resolve-ZirconShaderPbrPublicationChildPath -Root $capturesRoot -Child ".leases"
    $locksRoot = Resolve-ZirconShaderPbrPublicationChildPath -Root $capturesRoot -Child ".lease-locks"
    return [pscustomobject]@{
        captures_root = $capturesRoot
        staging_root = Resolve-ZirconShaderPbrPublicationChildPath `
            -Root (Resolve-ZirconShaderPbrPublicationChildPath -Root $capturesRoot -Child ".staging") `
            -Child $ProfileId
        receipt_path = Resolve-ZirconShaderPbrPublicationChildPath `
            -Root (Resolve-ZirconShaderPbrPublicationChildPath -Root $capturesRoot -Child ".completed") `
            -Child "$ProfileId.json"
        leases_root = $leasesRoot
        lease_path = Resolve-ZirconShaderPbrPublicationChildPath -Root $leasesRoot -Child "$ProfileId.json"
        locks_root = $locksRoot
        lock_path = Resolve-ZirconShaderPbrPublicationChildPath -Root $locksRoot -Child "$ProfileId.lock"
    }
}

function ConvertFrom-ZirconShaderPbrLeaseUtc {
    param(
        [Parameter(Mandatory = $true)][string]$Value,
        [Parameter(Mandatory = $true)][string]$Description
    )

    $parsed = [DateTimeOffset]::MinValue
    if (-not [DateTimeOffset]::TryParse($Value, [ref]$parsed)) {
        throw "Shader PBR profile lease $Description is not an ISO timestamp: $Value"
    }
    return $parsed.ToUniversalTime()
}

function Assert-ZirconShaderPbrProfileRunLeaseState {
    param(
        [Parameter(Mandatory = $true)]$State,
        [Parameter(Mandatory = $true)]$Paths,
        [Parameter(Mandatory = $true)][string]$ProfileId
    )

    $expectedFields = @(
        "schema_version",
        "lease_kind",
        "status",
        "profile_id",
        "profile_root",
        "lease_token",
        "owner_pid",
        "started_utc",
        "heartbeat_utc",
        "terminal_utc",
        "receipt_path",
        "failure",
        "quarantine_root"
    )
    $actualFields = @($State.PSObject.Properties.Name)
    if ($actualFields.Count -ne $expectedFields.Count -or
        @($actualFields | Where-Object { $_ -notin $expectedFields }).Count -ne 0) {
        throw "Shader PBR profile lease has an unexpected schema: $($Paths.lease_path)"
    }
    if ($State.schema_version -ne 1 -or
        $State.lease_kind -ne "zircon_shader_pbr_profile_run_lease" -or
        $State.status -notin @("running", "committed", "failed", "quarantined") -or
        $State.profile_id -ne $ProfileId -or
        -not ([string]$State.profile_root).Equals($Paths.staging_root, [System.StringComparison]::OrdinalIgnoreCase) -or
        [string]$State.lease_token -notmatch '^[0-9a-f]{32}$' -or
        [int64]$State.owner_pid -lt 1) {
        throw "Shader PBR profile lease does not bind its allocated profile root: $($Paths.lease_path)"
    }
    ConvertFrom-ZirconShaderPbrLeaseUtc -Value ([string]$State.started_utc) -Description "start" | Out-Null
    ConvertFrom-ZirconShaderPbrLeaseUtc -Value ([string]$State.heartbeat_utc) -Description "heartbeat" | Out-Null
    if ($State.status -eq "running") {
        if ($null -ne $State.terminal_utc -or
            $null -ne $State.receipt_path -or
            $null -ne $State.failure -or
            $null -ne $State.quarantine_root) {
            throw "Shader PBR running lease contains terminal state: $($Paths.lease_path)"
        }
    }
    else {
        ConvertFrom-ZirconShaderPbrLeaseUtc -Value ([string]$State.terminal_utc) -Description "terminal" | Out-Null
        switch ($State.status) {
            "committed" {
                if (-not ([string]$State.receipt_path).Equals($Paths.receipt_path, [System.StringComparison]::OrdinalIgnoreCase)) {
                    throw "Shader PBR committed lease does not bind its completion receipt: $($Paths.lease_path)"
                }
                if (-not [string]::IsNullOrWhiteSpace([string]$State.failure) -or
                    -not [string]::IsNullOrWhiteSpace([string]$State.quarantine_root)) {
                    throw "Shader PBR committed lease contains an unexpected terminal field: $($Paths.lease_path)"
                }
            }
            "failed" {
                if ([string]::IsNullOrWhiteSpace([string]$State.failure)) {
                    throw "Shader PBR failed lease has no failure reason: $($Paths.lease_path)"
                }
                if (-not [string]::IsNullOrWhiteSpace([string]$State.receipt_path) -or
                    -not [string]::IsNullOrWhiteSpace([string]$State.quarantine_root)) {
                    throw "Shader PBR failed lease contains an unexpected terminal field: $($Paths.lease_path)"
                }
            }
            "quarantined" {
                if ([string]::IsNullOrWhiteSpace([string]$State.failure) -or
                    [string]::IsNullOrWhiteSpace([string]$State.quarantine_root)) {
                    throw "Shader PBR quarantined lease has incomplete terminal state: $($Paths.lease_path)"
                }
                if (-not [string]::IsNullOrWhiteSpace([string]$State.receipt_path)) {
                    throw "Shader PBR quarantined lease contains an unexpected terminal field: $($Paths.lease_path)"
                }
                $expectedQuarantineRoot = Resolve-ZirconShaderPbrPublicationChildPath `
                    -Root (Resolve-ZirconShaderPbrPublicationChildPath -Root $Paths.captures_root -Child ".quarantine") `
                    -Child ("{0}-{1}" -f $ProfileId, $State.lease_token)
                if (-not ([string]$State.quarantine_root).Equals($expectedQuarantineRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
                    throw "Shader PBR quarantined lease does not bind its allocated quarantine root: $($Paths.lease_path)"
                }
            }
        }
    }
    return $State
}

function Get-ZirconShaderPbrProfileRunLeaseState {
    param(
        [Parameter(Mandatory = $true)][string]$ProfileCapturesRoot,
        [Parameter(Mandatory = $true)][string]$ProfileId
    )

    $paths = Resolve-ZirconShaderPbrProfileLeasePaths `
        -ProfileCapturesRoot $ProfileCapturesRoot `
        -ProfileId $ProfileId
    if (-not (Test-Path -LiteralPath $paths.lease_path -PathType Leaf)) {
        throw "Shader PBR profile lease is unavailable: $($paths.lease_path)"
    }
    $item = Get-Item -LiteralPath $paths.lease_path -Force
    if (($item.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "Shader PBR profile lease must not be a reparse point: $($paths.lease_path)"
    }
    try {
        $state = Get-Content -LiteralPath $paths.lease_path -Raw | ConvertFrom-Json
    }
    catch {
        throw "Shader PBR profile lease is malformed: $($paths.lease_path)"
    }
    return Assert-ZirconShaderPbrProfileRunLeaseState `
        -State $state `
        -Paths $paths `
        -ProfileId $ProfileId
}

function Set-ZirconShaderPbrProfileRunLeaseState {
    param(
        [Parameter(Mandatory = $true)]$State,
        [Parameter(Mandatory = $true)]$Paths,
        [Parameter(Mandatory = $true)][string]$ProfileId
    )

    Assert-ZirconShaderPbrProfileRunLeaseState -State $State -Paths $Paths -ProfileId $ProfileId | Out-Null
    return Write-ZirconShaderPbrReplaceJson `
        -Path $paths.lease_path `
        -Value $State `
        -Description "run lease"
}

function New-ZirconShaderPbrProfileRunLease {
    param(
        [Parameter(Mandatory = $true)][string]$ProfileCapturesRoot,
        [Parameter(Mandatory = $true)][string]$ProfileRoot,
        [Parameter(Mandatory = $true)][string]$ProfileId
    )

    $paths = Resolve-ZirconShaderPbrProfileLeasePaths `
        -ProfileCapturesRoot $ProfileCapturesRoot `
        -ProfileId $ProfileId
    $resolvedProfileRoot = [System.IO.Path]::GetFullPath($ProfileRoot)
    if (-not $resolvedProfileRoot.Equals($paths.staging_root, [System.StringComparison]::OrdinalIgnoreCase) -or
        -not (Test-Path -LiteralPath $resolvedProfileRoot -PathType Container)) {
        throw "Shader PBR profile lease requires its allocated staging root."
    }
    New-Item -ItemType Directory -Force -Path $paths.leases_root, $paths.locks_root | Out-Null
    $lockStream = $null
    try {
        try {
            $lockStream = [System.IO.File]::Open(
                $paths.lock_path,
                [System.IO.FileMode]::OpenOrCreate,
                [System.IO.FileAccess]::ReadWrite,
                [System.IO.FileShare]::None
            )
        }
        catch [System.IO.IOException] {
            throw "Shader PBR profile lease is already active: $ProfileId"
        }
        $now = (Get-Date).ToUniversalTime().ToString("o")
        $state = [ordered]@{
            schema_version = 1
            lease_kind = "zircon_shader_pbr_profile_run_lease"
            status = "running"
            profile_id = $ProfileId
            profile_root = $resolvedProfileRoot
            lease_token = [guid]::NewGuid().ToString("N")
            owner_pid = $PID
            started_utc = $now
            heartbeat_utc = $now
            terminal_utc = $null
            receipt_path = $null
            failure = $null
            quarantine_root = $null
        }
        Assert-ZirconShaderPbrProfileRunLeaseState -State ([pscustomobject]$state) -Paths $paths -ProfileId $ProfileId | Out-Null
        Write-ZirconShaderPbrCreateNewJson `
            -Path $paths.lease_path `
            -Value $state `
            -Description "run lease" | Out-Null
        return [pscustomobject]@{
            profile_id = $ProfileId
            profile_root = $resolvedProfileRoot
            lease_path = $paths.lease_path
            lease_token = $state.lease_token
            lock_stream = $lockStream
        }
    }
    catch {
        if ($null -ne $lockStream) {
            $lockStream.Dispose()
        }
        throw
    }
}

function Assert-ZirconShaderPbrProfileRunLeaseOwner {
    param([Parameter(Mandatory = $true)]$Lease)

    if ($null -eq $Lease.lock_stream -or
        -not $Lease.lock_stream.CanWrite -or
        $Lease.lock_stream.SafeFileHandle.IsClosed) {
        throw "Shader PBR profile lease owner is no longer active: $($Lease.profile_id)"
    }
    $paths = Resolve-ZirconShaderPbrProfileLeasePaths `
        -ProfileCapturesRoot (Split-Path -Parent (Split-Path -Parent $Lease.lease_path)) `
        -ProfileId $Lease.profile_id
    $state = Get-ZirconShaderPbrProfileRunLeaseState `
        -ProfileCapturesRoot $paths.captures_root `
        -ProfileId $Lease.profile_id
    if ($state.status -ne "running" -or
        -not ([string]$state.lease_token).Equals([string]$Lease.lease_token, [System.StringComparison]::Ordinal)) {
        throw "Shader PBR profile lease owner no longer owns a running profile: $($Lease.profile_id)"
    }
    return [pscustomobject]@{ state = $state; paths = $paths }
}

function Update-ZirconShaderPbrProfileRunLeaseHeartbeat {
    param([Parameter(Mandatory = $true)]$Lease)

    $owner = Assert-ZirconShaderPbrProfileRunLeaseOwner -Lease $Lease
    $owner.state.heartbeat_utc = (Get-Date).ToUniversalTime().ToString("o")
    return Set-ZirconShaderPbrProfileRunLeaseState `
        -State $owner.state `
        -Paths $owner.paths `
        -ProfileId $Lease.profile_id
}

function Complete-ZirconShaderPbrProfileRunLease {
    param(
        [Parameter(Mandatory = $true)]$Lease,
        [Parameter(Mandatory = $true)][string]$ReceiptPath
    )

    $owner = Assert-ZirconShaderPbrProfileRunLeaseOwner -Lease $Lease
    Assert-ZirconShaderPbrProfileCompletion `
        -ProfileCapturesRoot $owner.paths.captures_root `
        -ProfileRoot $Lease.profile_root `
        -ProfileId $Lease.profile_id `
        -ReceiptPath $ReceiptPath | Out-Null
    $owner.state.status = "committed"
    $owner.state.heartbeat_utc = (Get-Date).ToUniversalTime().ToString("o")
    $owner.state.terminal_utc = $owner.state.heartbeat_utc
    $owner.state.receipt_path = [System.IO.Path]::GetFullPath($ReceiptPath)
    return Set-ZirconShaderPbrProfileRunLeaseState `
        -State $owner.state `
        -Paths $owner.paths `
        -ProfileId $Lease.profile_id
}

function Fail-ZirconShaderPbrProfileRunLease {
    param(
        [Parameter(Mandatory = $true)]$Lease,
        [Parameter(Mandatory = $true)][string]$FailureMessage
    )

    $owner = Assert-ZirconShaderPbrProfileRunLeaseOwner -Lease $Lease
    $owner.state.status = "failed"
    $owner.state.heartbeat_utc = (Get-Date).ToUniversalTime().ToString("o")
    $owner.state.terminal_utc = $owner.state.heartbeat_utc
    $owner.state.failure = $FailureMessage.Substring(0, [Math]::Min($FailureMessage.Length, 1024))
    return Set-ZirconShaderPbrProfileRunLeaseState `
        -State $owner.state `
        -Paths $owner.paths `
        -ProfileId $Lease.profile_id
}

function Close-ZirconShaderPbrProfileRunLease {
    param([Parameter(Mandatory = $true)]$Lease)

    if ($null -ne $Lease.lock_stream) {
        $Lease.lock_stream.Dispose()
        $Lease.lock_stream = $null
    }
}

function Move-ZirconShaderPbrQuarantinedProfileRoot {
    param(
        [Parameter(Mandatory = $true)]$State,
        [Parameter(Mandatory = $true)]$Paths,
        [Parameter(Mandatory = $true)][string]$ProfileId
    )

    if ($State.status -ne "quarantined") {
        throw "Shader PBR profile quarantine requires a quarantined terminal state."
    }
    $quarantineRoot = [System.IO.Path]::GetFullPath([string]$State.quarantine_root)
    if (Test-Path -LiteralPath $paths.staging_root -PathType Container) {
        if (Test-Path -LiteralPath $quarantineRoot) {
            throw "Shader PBR profile quarantine root already exists: $quarantineRoot"
        }
        New-Item -ItemType Directory -Force -Path (Split-Path -Parent $quarantineRoot) | Out-Null
        Move-Item -LiteralPath $paths.staging_root -Destination $quarantineRoot -ErrorAction Stop
    }
    elseif (-not (Test-Path -LiteralPath $quarantineRoot -PathType Container)) {
        throw "Shader PBR profile quarantine lost both staging and quarantine roots: $ProfileId"
    }
    return $quarantineRoot
}

function Invoke-ZirconShaderPbrProfileStaleRunScavenger {
    param(
        [Parameter(Mandatory = $true)][string]$ProfileCapturesRoot,
        [ValidateRange(60, 604800)][int]$StaleAfterSeconds = 600
    )

    $capturesRoot = [System.IO.Path]::GetFullPath($ProfileCapturesRoot)
    $leasesRoot = Resolve-ZirconShaderPbrPublicationChildPath -Root $capturesRoot -Child ".leases"
    $results = [System.Collections.Generic.List[object]]::new()
    if (Test-Path -LiteralPath $leasesRoot -PathType Container) {
        foreach ($leaseItem in @(Get-ChildItem -LiteralPath $leasesRoot -File -Filter "*.json")) {
            if (($leaseItem.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
                $results.Add([pscustomobject]@{ profile_id = $leaseItem.BaseName; action = "ignored_reparse_point" })
                continue
            }
            $profileId = $leaseItem.BaseName
            if ($profileId -notmatch '^[a-z][a-z0-9-]{2,127}$') {
                $results.Add([pscustomobject]@{ profile_id = $profileId; action = "ignored_invalid_profile_id" })
                continue
            }
            try {
                $paths = Resolve-ZirconShaderPbrProfileLeasePaths `
                    -ProfileCapturesRoot $capturesRoot `
                    -ProfileId $profileId
                $lockStream = $null
                try {
                    try {
                        New-Item -ItemType Directory -Force -Path $paths.locks_root | Out-Null
                        $lockStream = [System.IO.File]::Open(
                            $paths.lock_path,
                            [System.IO.FileMode]::OpenOrCreate,
                            [System.IO.FileAccess]::ReadWrite,
                            [System.IO.FileShare]::None
                        )
                    }
                    catch [System.IO.IOException] {
                        $results.Add([pscustomobject]@{ profile_id = $profileId; action = "active_lock_held" })
                        continue
                    }
                    $state = Get-ZirconShaderPbrProfileRunLeaseState `
                        -ProfileCapturesRoot $capturesRoot `
                        -ProfileId $profileId
                    if ($state.status -in @("committed", "failed")) {
                        $results.Add([pscustomobject]@{ profile_id = $profileId; action = "terminal_retained" })
                        continue
                    }
                    if ($state.status -eq "quarantined") {
                        $quarantineRoot = Move-ZirconShaderPbrQuarantinedProfileRoot `
                            -State $state `
                            -Paths $paths `
                            -ProfileId $profileId
                        $results.Add([pscustomobject]@{
                                profile_id = $profileId
                                action = "quarantine_retained"
                                quarantine_root = $quarantineRoot
                            })
                        continue
                    }
                    if ($state.status -eq "running" -and (Test-Path -LiteralPath $paths.receipt_path -PathType Leaf)) {
                        try {
                            Assert-ZirconShaderPbrProfileCompletion `
                                -ProfileCapturesRoot $capturesRoot `
                                -ProfileRoot $paths.staging_root `
                                -ProfileId $profileId `
                                -ReceiptPath $paths.receipt_path | Out-Null
                            $state.status = "committed"
                            $state.heartbeat_utc = (Get-Date).ToUniversalTime().ToString("o")
                            $state.terminal_utc = $state.heartbeat_utc
                            $state.receipt_path = $paths.receipt_path
                            Set-ZirconShaderPbrProfileRunLeaseState `
                                -State $state `
                                -Paths $paths `
                                -ProfileId $profileId | Out-Null
                            $results.Add([pscustomobject]@{ profile_id = $profileId; action = "recovered_committed_receipt" })
                            continue
                        }
                        catch {
                            $null = $_
                        }
                    }
                    if ($state.status -eq "running") {
                        $heartbeat = ConvertFrom-ZirconShaderPbrLeaseUtc `
                            -Value ([string]$state.heartbeat_utc) `
                            -Description "heartbeat"
                        $ageSeconds = ((Get-Date).ToUniversalTime() - $heartbeat.UtcDateTime).TotalSeconds
                        if ($ageSeconds -lt $StaleAfterSeconds) {
                            $results.Add([pscustomobject]@{ profile_id = $profileId; action = "recent_lease_retained" })
                            continue
                        }
                        $state.status = "quarantined"
                        $state.terminal_utc = (Get-Date).ToUniversalTime().ToString("o")
                        $state.failure = "stale lease exceeded $StaleAfterSeconds seconds without an active owner lock"
                        $state.quarantine_root = Resolve-ZirconShaderPbrPublicationChildPath `
                            -Root (Resolve-ZirconShaderPbrPublicationChildPath -Root $capturesRoot -Child ".quarantine") `
                            -Child ("{0}-{1}" -f $profileId, $state.lease_token)
                        Set-ZirconShaderPbrProfileRunLeaseState `
                            -State $state `
                            -Paths $paths `
                            -ProfileId $profileId | Out-Null
                    }
                    $quarantineRoot = Move-ZirconShaderPbrQuarantinedProfileRoot `
                        -State $state `
                        -Paths $paths `
                        -ProfileId $profileId
                    $results.Add([pscustomobject]@{
                            profile_id = $profileId
                            action = "quarantined_stale_run"
                            quarantine_root = $quarantineRoot
                        })
                }
                finally {
                    if ($null -ne $lockStream) {
                        $lockStream.Dispose()
                    }
                }
            }
            catch {
                $results.Add([pscustomobject]@{
                        profile_id = $profileId
                        action = "scavenge_error"
                        error = $_.Exception.Message
                    })
            }
        }
    }
    return @($results)
}

function New-ZirconShaderPbrProfileStagingRoot {
    param(
        [Parameter(Mandatory = $true)][string]$ProfileCapturesRoot,
        [Parameter(Mandatory = $true)][string]$ProfileId
    )

    Assert-ZirconShaderPbrStableProfileId -ProfileId $ProfileId
    $capturesRoot = [System.IO.Path]::GetFullPath($ProfileCapturesRoot)
    New-Item -ItemType Directory -Force -Path $capturesRoot | Out-Null
    $stagingParent = Resolve-ZirconShaderPbrPublicationChildPath -Root $capturesRoot -Child ".staging"
    New-Item -ItemType Directory -Force -Path $stagingParent | Out-Null
    $stagingRoot = Resolve-ZirconShaderPbrPublicationChildPath -Root $stagingParent -Child $ProfileId
    if (Test-Path -LiteralPath $stagingRoot) {
        throw "Shader PBR profile staging root already exists: $stagingRoot"
    }
    New-Item -ItemType Directory -Path $stagingRoot -ErrorAction Stop | Out-Null
    return $stagingRoot
}

function Get-ZirconShaderPbrProfileStagedArtifacts {
    param([Parameter(Mandatory = $true)][string]$ProfileRoot)

    $resolvedProfileRoot = [System.IO.Path]::GetFullPath($ProfileRoot)
    if (-not (Test-Path -LiteralPath $resolvedProfileRoot -PathType Container)) {
        throw "Shader PBR profile staging root is unavailable: $resolvedProfileRoot"
    }
    $reparsePoints = @(Get-ChildItem -LiteralPath $resolvedProfileRoot -Force -Recurse |
        Where-Object { ($_.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0 })
    if ($reparsePoints.Count -ne 0) {
        throw "Shader PBR profile staging root contains a reparse point: $($reparsePoints[0].FullName)"
    }
    $artifactByPath = [System.Collections.Generic.Dictionary[string, object]]::new(
        [System.StringComparer]::Ordinal
    )
    $artifactPaths = [System.Collections.Generic.List[string]]::new()
    foreach ($file in @(Get-ChildItem -LiteralPath $resolvedProfileRoot -File -Recurse)) {
        $relativePath = Get-ZirconShaderPbrPublicationRelativePath `
            -Root $resolvedProfileRoot `
            -Path $file.FullName
        if ($relativePath -notmatch '^[A-Za-z0-9._/-]+$' -or
            $relativePath.Split("/") | Where-Object { $_ -in @("", ".", "..") }) {
            throw "Shader PBR profile staged artifact has an unsafe relative path: $relativePath"
        }
        if ($artifactByPath.ContainsKey($relativePath)) {
            throw "Shader PBR profile staged artifact path is duplicated: $relativePath"
        }
        $fingerprint = Get-ZirconProfileRequiredFileFingerprint `
            -Path $file.FullName `
            -Description "staged Shader PBR artifact '$relativePath'"
        $artifactByPath[$relativePath] = [pscustomobject]@{
            relative_path = $relativePath
            sha256 = [string]$fingerprint.sha256
            byte_length = [int64]$fingerprint.byte_length
        }
        $artifactPaths.Add($relativePath)
    }
    $artifactPaths.Sort([System.StringComparer]::Ordinal)
    $artifacts = @($artifactPaths | ForEach-Object { $artifactByPath[$_] })
    if ($artifacts.Count -eq 0) {
        throw "Shader PBR profile staging root has no artifacts to publish: $resolvedProfileRoot"
    }
    return $artifacts
}

function Publish-ZirconShaderPbrProfileCompletion {
    param(
        [Parameter(Mandatory = $true)][string]$ProfileCapturesRoot,
        [Parameter(Mandatory = $true)][string]$ProfileRoot,
        [Parameter(Mandatory = $true)][string]$ProfileId
    )

    Assert-ZirconShaderPbrStableProfileId -ProfileId $ProfileId
    $capturesRoot = [System.IO.Path]::GetFullPath($ProfileCapturesRoot)
    $expectedProfileRoot = Resolve-ZirconShaderPbrPublicationChildPath `
        -Root (Resolve-ZirconShaderPbrPublicationChildPath -Root $capturesRoot -Child ".staging") `
        -Child $ProfileId
    $resolvedProfileRoot = [System.IO.Path]::GetFullPath($ProfileRoot)
    if (-not $resolvedProfileRoot.Equals($expectedProfileRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Shader PBR profile completion can only publish its allocated staging root."
    }
    $receiptRoot = Resolve-ZirconShaderPbrPublicationChildPath -Root $capturesRoot -Child ".completed"
    $receiptPath = Resolve-ZirconShaderPbrPublicationChildPath -Root $receiptRoot -Child "$ProfileId.json"
    $receipt = [ordered]@{
        schema_version = 1
        receipt_kind = "zircon_shader_pbr_profile_completion"
        status = "completed"
        profile_id = $ProfileId
        profile_root = $resolvedProfileRoot
        completed_utc = (Get-Date).ToUniversalTime().ToString("o")
        artifacts = @(Get-ZirconShaderPbrProfileStagedArtifacts -ProfileRoot $resolvedProfileRoot)
    }
    return Write-ZirconShaderPbrCreateNewJson `
        -Path $receiptPath `
        -Value $receipt `
        -Description "completion receipt"
}

function Assert-ZirconShaderPbrProfileCompletion {
    param(
        [Parameter(Mandatory = $true)][string]$ProfileCapturesRoot,
        [Parameter(Mandatory = $true)][string]$ProfileRoot,
        [Parameter(Mandatory = $true)][string]$ProfileId,
        [Parameter(Mandatory = $true)][string]$ReceiptPath
    )

    Assert-ZirconShaderPbrStableProfileId -ProfileId $ProfileId
    $capturesRoot = [System.IO.Path]::GetFullPath($ProfileCapturesRoot)
    $expectedProfileRoot = Resolve-ZirconShaderPbrPublicationChildPath `
        -Root (Resolve-ZirconShaderPbrPublicationChildPath -Root $capturesRoot -Child ".staging") `
        -Child $ProfileId
    $resolvedProfileRoot = [System.IO.Path]::GetFullPath($ProfileRoot)
    if (-not $resolvedProfileRoot.Equals($expectedProfileRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Shader PBR profile completion verification requires its allocated staging root."
    }
    $expectedReceiptPath = Resolve-ZirconShaderPbrPublicationChildPath `
        -Root (Resolve-ZirconShaderPbrPublicationChildPath -Root $capturesRoot -Child ".completed") `
        -Child "$ProfileId.json"
    $resolvedReceiptPath = [System.IO.Path]::GetFullPath($ReceiptPath)
    if (-not $resolvedReceiptPath.Equals($expectedReceiptPath, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Shader PBR profile completion receipt is outside its allocated receipt root."
    }
    if (-not (Test-Path -LiteralPath $resolvedReceiptPath -PathType Leaf)) {
        throw "Shader PBR profile completion receipt is unavailable: $resolvedReceiptPath"
    }
    $receiptItem = Get-Item -LiteralPath $resolvedReceiptPath -Force
    if (($receiptItem.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "Shader PBR profile completion receipt must not be a reparse point: $resolvedReceiptPath"
    }
    try {
        $receipt = Get-Content -LiteralPath $resolvedReceiptPath -Raw | ConvertFrom-Json
    }
    catch {
        throw "Shader PBR profile completion receipt is malformed: $resolvedReceiptPath"
    }
    $expectedFields = @(
        "schema_version",
        "receipt_kind",
        "status",
        "profile_id",
        "profile_root",
        "completed_utc",
        "artifacts"
    )
    $actualFields = @($receipt.PSObject.Properties.Name)
    if ($actualFields.Count -ne $expectedFields.Count -or
        @($actualFields | Where-Object { $_ -notin $expectedFields }).Count -ne 0) {
        throw "Shader PBR profile completion receipt has an unexpected schema: $resolvedReceiptPath"
    }
    if ($receipt.schema_version -ne 1 -or
        $receipt.receipt_kind -ne "zircon_shader_pbr_profile_completion" -or
        $receipt.status -ne "completed" -or
        $receipt.profile_id -ne $ProfileId -or
        -not ([string]$receipt.profile_root).Equals($resolvedProfileRoot, [System.StringComparison]::OrdinalIgnoreCase) -or
        [string]::IsNullOrWhiteSpace([string]$receipt.completed_utc)) {
        throw "Shader PBR profile completion receipt does not bind the requested completed profile: $resolvedReceiptPath"
    }
    $expectedArtifacts = @($receipt.artifacts)
    if ($expectedArtifacts.Count -eq 0) {
        throw "Shader PBR profile completion receipt has no artifacts: $resolvedReceiptPath"
    }
    $expectedByPath = [System.Collections.Generic.Dictionary[string, object]]::new(
        [System.StringComparer]::Ordinal
    )
    $expectedPaths = [System.Collections.Generic.List[string]]::new()
    foreach ($artifact in $expectedArtifacts) {
        $artifactFields = @($artifact.PSObject.Properties.Name)
        $requiredArtifactFields = @("relative_path", "sha256", "byte_length")
        if ($artifactFields.Count -ne $requiredArtifactFields.Count -or
            @($artifactFields | Where-Object { $_ -notin $requiredArtifactFields }).Count -ne 0) {
            throw "Shader PBR profile completion receipt artifact has an unexpected schema: $resolvedReceiptPath"
        }
        $relativePath = [string]$artifact.relative_path
        if ($relativePath -notmatch '^[A-Za-z0-9._/-]+$' -or
            @($relativePath.Split("/") | Where-Object { $_ -in @("", ".", "..") }).Count -ne 0) {
            throw "Shader PBR profile completion receipt artifact path is unsafe: $relativePath"
        }
        if ([string]$artifact.sha256 -notmatch '^[0-9a-f]{64}$' -or
            [int64]$artifact.byte_length -lt 0 -or
            $expectedByPath.ContainsKey($relativePath)) {
            throw "Shader PBR profile completion receipt artifact is malformed: $relativePath"
        }
        $expectedByPath[$relativePath] = $artifact
        $expectedPaths.Add($relativePath)
    }
    $expectedPaths.Sort([System.StringComparer]::Ordinal)
    for ($index = 0; $index -lt $expectedArtifacts.Count; $index++) {
        if ([string]$expectedArtifacts[$index].relative_path -ne $expectedPaths[$index]) {
            throw "Shader PBR profile completion receipt artifacts are not in stable ordinal order: $resolvedReceiptPath"
        }
    }
    $actualArtifacts = @(Get-ZirconShaderPbrProfileStagedArtifacts -ProfileRoot $resolvedProfileRoot)
    if ($actualArtifacts.Count -ne $expectedPaths.Count) {
        throw "Shader PBR profile completion receipt artifact closure changed: $resolvedReceiptPath"
    }
    for ($index = 0; $index -lt $actualArtifacts.Count; $index++) {
        $actual = $actualArtifacts[$index]
        $relativePath = [string]$actual.relative_path
        if ($relativePath -ne $expectedPaths[$index]) {
            throw "Shader PBR profile completion receipt artifact closure changed: $resolvedReceiptPath"
        }
        $expected = $expectedByPath[$relativePath]
        if ([string]$actual.sha256 -ne [string]$expected.sha256) {
            throw "Shader PBR profile completion receipt artifact SHA-256 changed: $relativePath"
        }
        if ([int64]$actual.byte_length -ne [int64]$expected.byte_length) {
            throw "Shader PBR profile completion receipt artifact byte length changed: $relativePath"
        }
    }
    return $resolvedReceiptPath
}

function Write-ZirconShaderPbrProfileIncompleteReceipt {
    param(
        [Parameter(Mandatory = $true)][string]$ProfileRoot,
        [Parameter(Mandatory = $true)][string]$ProfileId,
        [Parameter(Mandatory = $true)][string]$FailureMessage
    )

    Assert-ZirconShaderPbrStableProfileId -ProfileId $ProfileId
    $resolvedProfileRoot = [System.IO.Path]::GetFullPath($ProfileRoot)
    if (-not (Test-Path -LiteralPath $resolvedProfileRoot -PathType Container)) {
        throw "Shader PBR profile staging root is unavailable: $resolvedProfileRoot"
    }
    $receiptPath = Join-Path $resolvedProfileRoot "profile_incomplete.json"
    if (Test-Path -LiteralPath $receiptPath) {
        return [System.IO.Path]::GetFullPath($receiptPath)
    }
    $receipt = [ordered]@{
        schema_version = 1
        receipt_kind = "zircon_shader_pbr_profile_completion"
        status = "incomplete"
        profile_id = $ProfileId
        profile_root = $resolvedProfileRoot
        failed_utc = (Get-Date).ToUniversalTime().ToString("o")
        failure = $FailureMessage.Substring(0, [Math]::Min($FailureMessage.Length, 1024))
    }
    return Write-ZirconShaderPbrCreateNewJson `
        -Path $receiptPath `
        -Value $receipt `
        -Description "incomplete receipt"
}
