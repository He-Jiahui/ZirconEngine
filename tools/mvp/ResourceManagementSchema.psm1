Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Get-ResourceManagementSchemaProperty {
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

function Get-ResourceManagementSchemaOptionalProperty {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name
    )

    $property = $Value.PSObject.Properties[$Name]
    if ($null -eq $property) {
        return $null
    }
    return $property.Value
}

function Get-ResourceManagementSchemaArrayProperty {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Label
    )

    $items = @(Get-ResourceManagementSchemaProperty -Value $Value -Name $Name -Label $Label)
    if ($items.Count -eq 0) {
        throw "$Label has no '$Name'."
    }
    return $items
}

function Assert-ResourceManagementSchemaProperties {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string[]]$RequiredNames,
        [string[]]$OptionalNames = @(),
        [Parameter(Mandatory)][string]$Label
    )

    $actualNames = @($Value.PSObject.Properties.Name)
    foreach ($name in $RequiredNames) {
        if ($actualNames -notcontains $name) {
            throw "$Label is missing '$name'."
        }
    }
    $allowedNames = @($RequiredNames) + @($OptionalNames)
    foreach ($name in $actualNames) {
        if ($allowedNames -notcontains $name) {
            throw "$Label has unexpected property '$name'."
        }
    }
}

function Assert-ResourceManagementSchemaSha256 {
    param(
        [Parameter(Mandatory)][string]$Value,
        [Parameter(Mandatory)][string]$Label
    )

    if ($Value -notmatch '^[0-9A-F]{64}$') {
        throw "$Label must be an uppercase SHA-256 value."
    }
    return $Value
}

function Test-ResourceManagementSchemaJsonNumber {
    param([Parameter(Mandatory)]$Value)

    return (
        $Value -is [sbyte] -or
        $Value -is [byte] -or
        $Value -is [int16] -or
        $Value -is [uint16] -or
        $Value -is [int32] -or
        $Value -is [uint32] -or
        $Value -is [int64] -or
        $Value -is [uint64] -or
        $Value -is [single] -or
        $Value -is [double] -or
        $Value -is [decimal]
    )
}

function ConvertTo-ResourceManagementSchemaJsonNumber {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Label,
        [double]$Minimum = 0.0,
        [double]$Maximum = [double]::MaxValue,
        [string]$InvalidTypeMessage = "$Label must be a JSON number.",
        [string]$InvalidConversionMessage = "$Label must be numeric.",
        [string]$InvalidRangeMessage = "$Label must be a finite number between $Minimum and $Maximum."
    )

    if (-not (Test-ResourceManagementSchemaJsonNumber -Value $Value)) {
        throw $InvalidTypeMessage
    }
    try {
        $number = [double]$Value
    }
    catch {
        throw $InvalidConversionMessage
    }
    if ([double]::IsNaN($number) -or [double]::IsInfinity($number) -or
        $number -lt $Minimum -or $number -gt $Maximum) {
        throw $InvalidRangeMessage
    }
    return $number
}

function ConvertTo-ResourceManagementSchemaNonNegativeInteger {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Label,
        [string]$InvalidMessage = "$Label must be a non-negative integer."
    )

    if (-not (Test-ResourceManagementSchemaJsonNumber -Value $Value)) {
        throw $InvalidMessage
    }
    try {
        $number = [decimal]$Value
    }
    catch {
        throw $InvalidMessage
    }
    if ($number -lt 0 -or [decimal]::Truncate($number) -ne $number -or
        $number -gt [decimal][uint64]::MaxValue) {
        throw $InvalidMessage
    }
    return [uint64]$number
}

Export-ModuleMember -Function @(
    'Assert-ResourceManagementSchemaProperties',
    'Assert-ResourceManagementSchemaSha256',
    'ConvertTo-ResourceManagementSchemaJsonNumber',
    'ConvertTo-ResourceManagementSchemaNonNegativeInteger',
    'Get-ResourceManagementSchemaArrayProperty',
    'Get-ResourceManagementSchemaOptionalProperty',
    'Get-ResourceManagementSchemaProperty',
    'Test-ResourceManagementSchemaJsonNumber'
)
