Set-StrictMode -Version Latest

$MvpExpectedEvidenceReserveBytes = [Int64](512MB)

function Get-MvpPreflightRequiredProperty {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Label
    )

    $property = $Value.PSObject.Properties[$Name]
    if ($null -eq $property -or $null -eq $property.Value -or
        ($property.Value -is [string] -and [string]::IsNullOrWhiteSpace($property.Value))) {
        throw "$Label is missing '$Name'."
    }
    return $property.Value
}

function Get-MvpPreflightRequiredInt64Property {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Label
    )

    $property = $Value.PSObject.Properties[$Name]
    if ($null -eq $property -or $null -eq $property.Value -or
        ($property.Value -is [string] -and [string]::IsNullOrWhiteSpace($property.Value))) {
        throw "$Label is missing '$Name'."
    }
    $propertyValue = $property.Value
    [Int64]$parsed = 0
    if ($propertyValue -is [int] -or $propertyValue -is [long]) {
        $parsed = [Int64]$propertyValue
    }
    elseif (-not [Int64]::TryParse([string]$propertyValue, [ref]$parsed)) {
        throw "Staging manifest preflight has invalid non-negative '$Name' value '$propertyValue'."
    }
    if ($parsed -lt 0) {
        throw "Staging manifest preflight has invalid non-negative '$Name' value '$propertyValue'."
    }
    return $parsed
}

function ConvertTo-MvpPreflightInt64 {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name
    )

    [Int64]$parsed = 0
    if ($Value -is [int] -or $Value -is [long]) {
        $parsed = [Int64]$Value
    }
    elseif (-not [Int64]::TryParse([string]$Value, [ref]$parsed)) {
        throw "Staging manifest preflight has invalid non-negative '$Name' value '$Value'."
    }
    if ($parsed -lt 0) {
        throw "Staging manifest preflight has invalid non-negative '$Name' value '$Value'."
    }
    return $parsed
}

function Assert-MvpStagingPreflightEvidence {
    param(
        [Parameter(Mandatory)]$Manifest,
        [Parameter(Mandatory)][Int64]$EntryBytes,
        [Parameter(Mandatory)][string]$StagingRoot
    )

    $preflight = Get-MvpPreflightRequiredProperty -Value $Manifest -Name 'preflight' -Label 'Staging manifest'
    $inputCopyBytes = Get-MvpPreflightRequiredInt64Property `
        -Value $preflight `
        -Name 'input_copy_bytes' `
        -Label 'Staging manifest preflight'
    if ($inputCopyBytes -ne $EntryBytes) {
        throw "Staging manifest preflight input_copy_bytes '$inputCopyBytes' differs from staging entries total '$EntryBytes'."
    }
    $evidenceReserveBytes = Get-MvpPreflightRequiredInt64Property `
        -Value $preflight `
        -Name 'evidence_reserve_bytes' `
        -Label 'Staging manifest preflight'
    if ($evidenceReserveBytes -ne $MvpExpectedEvidenceReserveBytes) {
        throw "Staging manifest preflight evidence_reserve_bytes '$evidenceReserveBytes' differs from fixed reserve '$MvpExpectedEvidenceReserveBytes'."
    }
    $requiredFreeSpaceBytes = Get-MvpPreflightRequiredInt64Property `
        -Value $preflight `
        -Name 'required_free_space_bytes' `
        -Label 'Staging manifest preflight'
    [decimal]$expectedRequired = [decimal]$inputCopyBytes + [decimal]$evidenceReserveBytes
    if ($expectedRequired -gt [Int64]::MaxValue -or $requiredFreeSpaceBytes -ne [Int64]$expectedRequired) {
        throw "Staging manifest preflight required_free_space_bytes '$requiredFreeSpaceBytes' is inconsistent with its input and evidence budgets."
    }
    $availableFreeSpaceBytes = Get-MvpPreflightRequiredInt64Property `
        -Value $preflight `
        -Name 'available_free_space_bytes' `
        -Label 'Staging manifest preflight'
    if ($availableFreeSpaceBytes -lt $requiredFreeSpaceBytes) {
        throw "Staging manifest preflight available_free_space_bytes '$availableFreeSpaceBytes' is below required_free_space_bytes '$requiredFreeSpaceBytes'."
    }

    $reportedDriveRoot = [string](Get-MvpPreflightRequiredProperty `
        -Value $preflight `
        -Name 'staging_drive_root' `
        -Label 'Staging manifest preflight')
    $expectedDriveRoot = [IO.Path]::GetPathRoot([IO.Path]::GetFullPath($StagingRoot))
    if (-not $reportedDriveRoot.Equals($expectedDriveRoot, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Staging manifest preflight drive root '$reportedDriveRoot' differs from staging drive '$expectedDriveRoot'."
    }

    $desktop = Get-MvpPreflightRequiredProperty -Value $preflight -Name 'interactive_desktop' -Label 'Staging manifest preflight'
    $required = Get-MvpPreflightRequiredProperty -Value $desktop -Name 'required' -Label 'Staging manifest interactive_desktop'
    $userInteractive = Get-MvpPreflightRequiredProperty -Value $desktop -Name 'user_interactive' -Label 'Staging manifest interactive_desktop'
    if ($required -isnot [bool] -or $userInteractive -isnot [bool]) {
        throw 'Staging manifest interactive_desktop required/user_interactive fields must be boolean.'
    }
    $sessionIdProperty = $desktop.PSObject.Properties['session_id']
    if ($null -eq $sessionIdProperty) {
        throw "Staging manifest interactive_desktop is missing 'session_id'."
    }
    $monitorCountProperty = $desktop.PSObject.Properties['monitor_count']
    if ($null -eq $monitorCountProperty) {
        throw "Staging manifest interactive_desktop is missing 'monitor_count'."
    }
    if (-not $required) {
        throw 'MVP acceptance interactive desktop evidence is required.'
    }
    if (-not $userInteractive) {
        throw 'Staging manifest interactive_desktop required an interactive user but user_interactive is false.'
    }
    $sessionId = ConvertTo-MvpPreflightInt64 `
        -Value $sessionIdProperty.Value `
        -Name 'interactive_desktop.session_id'
    if ($sessionId -le 0) {
        throw "Staging manifest interactive_desktop 'session_id' must be positive when a desktop is required."
    }
    $monitorCount = ConvertTo-MvpPreflightInt64 `
        -Value $monitorCountProperty.Value `
        -Name 'interactive_desktop.monitor_count'
    if ($monitorCount -le 0) {
        throw "Staging manifest interactive_desktop 'monitor_count' must be positive when a desktop is required."
    }
}

Export-ModuleMember -Function Assert-MvpStagingPreflightEvidence
