Set-StrictMode -Version Latest

$script:MvpStagingCancellationRequestSchemaVersion = 1
$script:MvpStagingCancellationRequestKind = 'zircon.mvp-staging-cancellation-request'
$script:MvpStagingCancellationRequestMaximumBytes = 4096
$script:MvpStagingCancellationRequestHexDigits = [char[]]'0123456789abcdef'

function ConvertTo-MvpStagingCancellationLowerHex {
    param([Parameter(Mandatory)][AllowEmptyCollection()][byte[]]$Bytes)

    $characters = [char[]]::new($Bytes.Length * 2)
    $index = 0
    foreach ($byte in $Bytes) {
        $characters[$index] = $script:MvpStagingCancellationRequestHexDigits[$byte -shr 4]
        $characters[$index + 1] = $script:MvpStagingCancellationRequestHexDigits[$byte -band 0x0F]
        $index += 2
    }
    return [string]::new($characters)
}

function Get-MvpStagingCancellationSha256 {
    param([Parameter(Mandatory)][AllowEmptyCollection()][byte[]]$Bytes)

    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        return ConvertTo-MvpStagingCancellationLowerHex -Bytes $hasher.ComputeHash($Bytes)
    }
    finally {
        $hasher.Dispose()
    }
}

function Get-MvpStagingCancellationRequestPath {
    param(
        [Parameter(Mandatory)][string]$StagingRoot,
        [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$')][string]$RunId
    )

    $root = [IO.Path]::GetFullPath($StagingRoot)
    return [IO.Path]::GetFullPath([IO.Path]::Combine($root, '.mvp-staging-cancellations', "$RunId.json"))
}

function Assert-MvpStagingCancellationExactProperties {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string[]]$ExpectedNames
    )

    $actualNames = @($Value.PSObject.Properties | ForEach-Object { $_.Name })
    if ($actualNames.Count -ne $ExpectedNames.Count) {
        throw "MVP staging cancellation request property count differs from $($ExpectedNames.Count)."
    }
    $expected = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($name in $ExpectedNames) {
        $expected.Add($name) | Out-Null
    }
    foreach ($name in $actualNames) {
        if (-not $expected.Contains($name)) {
            throw "MVP staging cancellation request contains unknown property '$name'."
        }
    }
}

function Read-MvpStagingCancellationRequest {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$ExpectedRunId
    )

    $file = [IO.FileInfo]::new($Path)
    if (($file.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "MVP staging cancellation request '$Path' is a reparse point."
    }
    $stream = [IO.File]::Open($Path, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
    try {
        $length = $stream.Length
        if ($length -eq 0 -or $length -gt $script:MvpStagingCancellationRequestMaximumBytes) {
            throw "MVP staging cancellation request must contain 1..$($script:MvpStagingCancellationRequestMaximumBytes) bytes."
        }
        $bytes = [byte[]]::new([int]$length)
        $offset = 0
        while ($offset -lt $bytes.Length) {
            $read = $stream.Read($bytes, $offset, $bytes.Length - $offset)
            if ($read -eq 0) {
                throw "MVP staging cancellation request changed while its bounded snapshot was read."
            }
            $offset += $read
        }
        if ($stream.ReadByte() -ne -1) {
            throw "MVP staging cancellation request exceeds its $($script:MvpStagingCancellationRequestMaximumBytes)-byte budget."
        }
    }
    finally {
        $stream.Dispose()
    }
    try {
        $request = [Text.UTF8Encoding]::new($false, $true).GetString($bytes) | ConvertFrom-Json
    }
    catch {
        throw "MVP staging cancellation request is not strict UTF-8 JSON: $($_.Exception.Message)"
    }
    Assert-MvpStagingCancellationExactProperties `
        -Value $request `
        -ExpectedNames @('schema_version', 'request_kind', 'run_id', 'reason', 'requested_at_utc')
    if (($request.schema_version -isnot [int] -and $request.schema_version -isnot [long]) -or
        [Int64]$request.schema_version -ne $script:MvpStagingCancellationRequestSchemaVersion) {
        throw "MVP staging cancellation request has unsupported schema version '$($request.schema_version)'."
    }
    if ([string]$request.request_kind -cne $script:MvpStagingCancellationRequestKind) {
        throw "MVP staging cancellation request has unsupported kind '$($request.request_kind)'."
    }
    if ([string]$request.run_id -cne $ExpectedRunId) {
        throw "MVP staging cancellation request run '$($request.run_id)' differs from '$ExpectedRunId'."
    }
    if ([string]$request.reason -notmatch '^[a-z0-9][a-z0-9._-]{0,127}$') {
        throw "MVP staging cancellation request has invalid reason '$($request.reason)'."
    }
    [DateTimeOffset]$requestedAt = [DateTimeOffset]::MinValue
    if ($request.requested_at_utc -is [DateTime]) {
        $requestedAt = [DateTimeOffset]::new(([DateTime]$request.requested_at_utc).ToUniversalTime())
    }
    elseif (-not [DateTimeOffset]::TryParse(
            [string]$request.requested_at_utc,
            [Globalization.CultureInfo]::InvariantCulture,
            [Globalization.DateTimeStyles]::AllowWhiteSpaces -bor [Globalization.DateTimeStyles]::RoundtripKind,
            [ref]$requestedAt)) {
        throw 'MVP staging cancellation request must contain a parseable requested_at_utc timestamp.'
    }
    return [pscustomobject][ordered]@{
        schema_version = [int]$request.schema_version
        request_kind = [string]$request.request_kind
        run_id = [string]$request.run_id
        reason = [string]$request.reason
        requested_at_utc = $requestedAt.ToUniversalTime().ToString('o')
        sha256 = Get-MvpStagingCancellationSha256 -Bytes $bytes
        bytes = $bytes.Length
    }
}

function New-MvpStagingCancellationProbeState {
    param(
        [Parameter(Mandatory)][string]$StagingRoot,
        [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$')][string]$RunId
    )

    return [pscustomobject]@{
        request_path = Get-MvpStagingCancellationRequestPath -StagingRoot $StagingRoot -RunId $RunId
        run_id = $RunId
        requested = $false
        request = $null
    }
}

function Test-MvpStagingCancellationRequested {
    param([Parameter(Mandatory)]$State)

    if ($State.requested) {
        return $true
    }
    if (-not [IO.File]::Exists($State.request_path)) {
        return $false
    }
    $State.request = Read-MvpStagingCancellationRequest `
        -Path $State.request_path `
        -ExpectedRunId $State.run_id
    $State.requested = $true
    return $true
}

function Write-MvpStagingCancellationRequest {
    param(
        [Parameter(Mandatory)][string]$StagingRoot,
        [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$')][string]$RunId,
        [Parameter(Mandatory)][ValidatePattern('^[a-z0-9][a-z0-9._-]{0,127}$')][string]$Reason
    )

    $path = Get-MvpStagingCancellationRequestPath -StagingRoot $StagingRoot -RunId $RunId
    $directory = [IO.Path]::GetDirectoryName($path)
    [IO.Directory]::CreateDirectory($directory) | Out-Null
    if (([IO.DirectoryInfo]::new($directory).Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "MVP staging cancellation request directory '$directory' is a reparse point."
    }
    if ([IO.File]::Exists($path)) {
        throw "MVP staging cancellation request '$path' already exists."
    }
    $request = [ordered]@{
        schema_version = $script:MvpStagingCancellationRequestSchemaVersion
        request_kind = $script:MvpStagingCancellationRequestKind
        run_id = $RunId
        reason = $Reason
        requested_at_utc = [DateTimeOffset]::UtcNow.ToString('o')
    }
    $bytes = [Text.UTF8Encoding]::new($false).GetBytes(
        (($request | ConvertTo-Json -Depth 8 -Compress) + [Environment]::NewLine)
    )
    if ($bytes.Length -gt $script:MvpStagingCancellationRequestMaximumBytes) {
        throw "MVP staging cancellation request exceeds its $($script:MvpStagingCancellationRequestMaximumBytes)-byte budget."
    }
    $temporaryPath = "$path.tmp-$([guid]::NewGuid().ToString('N'))"
    try {
        $stream = [IO.FileStream]::new(
            $temporaryPath,
            [IO.FileMode]::CreateNew,
            [IO.FileAccess]::Write,
            [IO.FileShare]::None,
            4096,
            [IO.FileOptions]::WriteThrough
        )
        try {
            $stream.Write($bytes, 0, $bytes.Length)
            $stream.Flush($true)
        }
        finally {
            $stream.Dispose()
        }
        [IO.File]::Move($temporaryPath, $path)
    }
    catch {
        [IO.File]::Delete($temporaryPath)
        throw
    }
    return [pscustomobject]@{
        path = $path
        bytes = $bytes.Length
        sha256 = Get-MvpStagingCancellationSha256 -Bytes $bytes
    }
}

Export-ModuleMember -Function @(
    'Get-MvpStagingCancellationRequestPath',
    'New-MvpStagingCancellationProbeState',
    'Test-MvpStagingCancellationRequested',
    'Write-MvpStagingCancellationRequest'
)
