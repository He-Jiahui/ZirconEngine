Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:MvpBuildGateRegistryPath = Join-Path $PSScriptRoot 'mvp-build-gate-registry.json'
$script:MvpBuildGateRegistryMaximumBytes = 64KB
$script:MvpBuildGateRegistrySummaryKinds = @('profile-contract', 'workspace')
$script:MvpBuildGateRegistryUpperHexDigits = [char[]]'0123456789ABCDEF'

function ConvertTo-MvpBuildGateRegistryUpperHex {
    param([Parameter(Mandatory)][byte[]]$Bytes)

    $characters = [char[]]::new($Bytes.Length * 2)
    for ($index = 0; $index -lt $Bytes.Length; $index++) {
        $value = $Bytes[$index]
        $characters[$index * 2] = $script:MvpBuildGateRegistryUpperHexDigits[$value -shr 4]
        $characters[$index * 2 + 1] = $script:MvpBuildGateRegistryUpperHexDigits[$value -band 0x0F]
    }
    return [string]::new($characters)
}

function Get-MvpBuildGateRegistryBytesSha256 {
    param([Parameter(Mandatory)][byte[]]$Bytes)

    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        return ConvertTo-MvpBuildGateRegistryUpperHex -Bytes $hasher.ComputeHash($Bytes)
    }
    finally {
        $hasher.Dispose()
    }
}

function Get-MvpBuildGateRegistryProperty {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Label
    )

    $property = $Value.PSObject.Properties[$Name]
    if ($null -eq $property -or $null -eq $property.Value) {
        throw "$Label is missing '$Name'."
    }
    return $property.Value
}

function Assert-MvpBuildGateRegistryExactProperties {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string[]]$ExpectedNames,
        [Parameter(Mandatory)][string]$Label
    )

    if ($Value -is [array] -or $Value -is [string] -or $Value -is [ValueType]) {
        throw "$Label must be one JSON object."
    }
    foreach ($name in $ExpectedNames) {
        if ($null -eq $Value.PSObject.Properties[$name]) {
            throw "$Label is missing '$name'."
        }
    }
    foreach ($property in $Value.PSObject.Properties) {
        if ($ExpectedNames -notcontains $property.Name) {
            throw "$Label contains unknown property '$($property.Name)'."
        }
    }
}

function Read-MvpBuildGateRegistry {
    param([Parameter(Mandatory)][string]$Path)

    $resolvedPath = [IO.Path]::GetFullPath($Path)
    if (-not [IO.File]::Exists($resolvedPath)) {
        throw "MVP build gate registry does not exist or is not a file: $Path"
    }
    $stream = [IO.File]::Open($resolvedPath, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
    try {
        if ($stream.Length -gt $script:MvpBuildGateRegistryMaximumBytes) {
            throw "MVP build gate registry exceeds its byte budget of $($script:MvpBuildGateRegistryMaximumBytes) bytes."
        }
        [byte[]]$bytes = [byte[]]::new([int]$stream.Length)
        $offset = 0
        while ($offset -lt $bytes.Length) {
            $read = $stream.Read($bytes, $offset, $bytes.Length - $offset)
            if ($read -eq 0) {
                throw 'MVP build gate registry changed while it was being read.'
            }
            $offset += $read
        }
    }
    finally {
        $stream.Dispose()
    }
    try {
        $registry = ([Text.UTF8Encoding]::new($false, $true)).GetString($bytes) | ConvertFrom-Json
    }
    catch {
        throw "MVP build gate registry is not valid UTF-8 JSON: $($_.Exception.Message)"
    }

    Assert-MvpBuildGateRegistryExactProperties `
        -Value $registry `
        -ExpectedNames @('schema_version', 'registry_kind', 'summaries') `
        -Label 'MVP build gate registry'
    $schemaVersion = Get-MvpBuildGateRegistryProperty -Value $registry -Name 'schema_version' -Label 'MVP build gate registry'
    if (-not ($schemaVersion -is [int] -or $schemaVersion -is [long]) -or [long]$schemaVersion -ne 1) {
        throw "MVP build gate registry schema_version must be the JSON integer 1; found '$schemaVersion'."
    }
    $registryKind = [string](Get-MvpBuildGateRegistryProperty -Value $registry -Name 'registry_kind' -Label 'MVP build gate registry')
    if (-not $registryKind.Equals('zircon.mvp-build-gate-registry', [StringComparison]::Ordinal)) {
        throw "MVP build gate registry has unsupported registry_kind '$registryKind'."
    }
    $summaries = @(Get-MvpBuildGateRegistryProperty -Value $registry -Name 'summaries' -Label 'MVP build gate registry')
    if ($summaries.Count -ne $script:MvpBuildGateRegistrySummaryKinds.Count) {
        throw "MVP build gate registry must contain exactly $($script:MvpBuildGateRegistrySummaryKinds.Count) summary groups."
    }

    $resolvedSummaries = [Collections.Generic.List[object]]::new()
    $seenGateIds = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    for ($summaryIndex = 0; $summaryIndex -lt $summaries.Count; $summaryIndex++) {
        $summary = $summaries[$summaryIndex]
        Assert-MvpBuildGateRegistryExactProperties `
            -Value $summary `
            -ExpectedNames @('summary_kind', 'gates') `
            -Label 'MVP build gate registry summary'
        $summaryKind = [string](Get-MvpBuildGateRegistryProperty -Value $summary -Name 'summary_kind' -Label 'MVP build gate registry summary')
        $expectedSummaryKind = $script:MvpBuildGateRegistrySummaryKinds[$summaryIndex]
        if (-not $summaryKind.Equals($expectedSummaryKind, [StringComparison]::Ordinal)) {
            throw "MVP build gate registry summary $summaryIndex must be '$expectedSummaryKind'; found '$summaryKind'."
        }
        $gates = @(Get-MvpBuildGateRegistryProperty -Value $summary -Name 'gates' -Label "MVP build gate registry '$summaryKind' summary")
        if ($gates.Count -eq 0) {
            throw "MVP build gate registry '$summaryKind' summary must contain at least one gate."
        }
        $resolvedGates = [Collections.Generic.List[object]]::new()
        foreach ($gate in $gates) {
            Assert-MvpBuildGateRegistryExactProperties `
                -Value $gate `
                -ExpectedNames @('gate_id', 'cargo_arguments') `
                -Label "MVP build gate registry '$summaryKind' gate"
            $gateId = [string](Get-MvpBuildGateRegistryProperty -Value $gate -Name 'gate_id' -Label "MVP build gate registry '$summaryKind' gate")
            if ($gateId -notmatch '^[a-z0-9][a-z0-9-]{0,127}$') {
                throw "MVP build gate registry gate_id '$gateId' is invalid."
            }
            if (-not $seenGateIds.Add($gateId)) {
                throw "MVP build gate registry contains duplicate gate_id '$gateId'."
            }
            $rawArguments = Get-MvpBuildGateRegistryProperty -Value $gate -Name 'cargo_arguments' -Label "MVP build gate registry gate '$gateId'"
            if ($rawArguments -isnot [array]) {
                throw "MVP build gate registry gate '$gateId' cargo_arguments must be one JSON array."
            }
            $cargoArguments = @($rawArguments)
            if ($cargoArguments.Count -eq 0) {
                throw "MVP build gate registry gate '$gateId' cargo_arguments must not be empty."
            }
            foreach ($argument in $cargoArguments) {
                if ($argument -isnot [string] -or [string]::IsNullOrWhiteSpace($argument)) {
                    throw "MVP build gate registry gate '$gateId' cargo_arguments must contain non-empty strings."
                }
                if ($argument -match '[\s\x00]') {
                    throw "MVP build gate registry gate '$gateId' Cargo argument '$argument' must not contain whitespace or NUL."
                }
            }
            if ($cargoArguments[-1] -ne '--locked') {
                throw "MVP build gate registry gate '$gateId' must terminate cargo_arguments with '--locked'."
            }
            if ($summaryKind -eq 'profile-contract' -and $cargoArguments[0] -ne 'check') {
                throw "MVP build gate registry profile gate '$gateId' must invoke cargo check."
            }
            if ($summaryKind -eq 'workspace' -and $cargoArguments[0] -notin @('build', 'test')) {
                throw "MVP build gate registry workspace gate '$gateId' must invoke cargo build or test."
            }
            [string[]]$typedArguments = @($cargoArguments | ForEach-Object { [string]$_ })
            $resolvedGates.Add([pscustomobject]@{
                    gate_id = $gateId
                    cargo_arguments = $typedArguments
                    command = 'cargo ' + ($typedArguments -join ' ')
                }) | Out-Null
        }
        $resolvedSummaries.Add([pscustomobject]@{
                summary_kind = $summaryKind
                gates = $resolvedGates.ToArray()
            }) | Out-Null
    }
    return [pscustomobject][ordered]@{
        receipt = [pscustomobject][ordered]@{
            schema_version = 1
            registry_kind = $registryKind
            sha256 = Get-MvpBuildGateRegistryBytesSha256 -Bytes $bytes
            size_bytes = [Int64]$bytes.LongLength
        }
        summaries = $resolvedSummaries.ToArray()
    }
}

function Get-MvpBuildGateRegistrySnapshot {
    param([string]$RegistryPath = $script:MvpBuildGateRegistryPath)

    return Read-MvpBuildGateRegistry -Path $RegistryPath
}

function Get-MvpBuildGateContract {
    param(
        [Parameter(Mandatory)][ValidateSet('profile-contract', 'workspace')][string]$SummaryKind,
        [string]$RegistryPath = $script:MvpBuildGateRegistryPath,
        [AllowNull()]$RegistrySnapshot
    )

    $snapshot = if ($null -eq $RegistrySnapshot) {
        Get-MvpBuildGateRegistrySnapshot -RegistryPath $RegistryPath
    }
    else {
        $RegistrySnapshot
    }
    Assert-MvpBuildGateRegistryExactProperties `
        -Value $snapshot `
        -ExpectedNames @('receipt', 'summaries') `
        -Label 'MVP build gate registry snapshot'
    $summaries = @($snapshot.summaries)
    $matches = @($summaries | Where-Object { $_.summary_kind -eq $SummaryKind })
    if ($matches.Count -ne 1) {
        throw "MVP build gate registry must contain one '$SummaryKind' summary."
    }
    return @($matches[0].gates)
}

Export-ModuleMember -Function Get-MvpBuildGateRegistrySnapshot, Get-MvpBuildGateContract
