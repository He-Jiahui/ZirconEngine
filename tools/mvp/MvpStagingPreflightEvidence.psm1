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

function ConvertTo-MvpPreflightInt64 {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name
    )

    [Int64]$parsed = 0
    if (-not [Int64]::TryParse([string]$Value, [ref]$parsed) -or $parsed -lt 0) {
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
    $inputCopyBytes = ConvertTo-MvpPreflightInt64 `
        -Value (Get-MvpPreflightRequiredProperty -Value $preflight -Name 'input_copy_bytes' -Label 'Staging manifest preflight') `
        -Name 'input_copy_bytes'
    if ($inputCopyBytes -ne $EntryBytes) {
        throw "Staging manifest preflight input_copy_bytes '$inputCopyBytes' differs from staging entries total '$EntryBytes'."
    }
    $evidenceReserveBytes = ConvertTo-MvpPreflightInt64 `
        -Value (Get-MvpPreflightRequiredProperty -Value $preflight -Name 'evidence_reserve_bytes' -Label 'Staging manifest preflight') `
        -Name 'evidence_reserve_bytes'
    if ($evidenceReserveBytes -ne $MvpExpectedEvidenceReserveBytes) {
        throw "Staging manifest preflight evidence_reserve_bytes '$evidenceReserveBytes' differs from fixed reserve '$MvpExpectedEvidenceReserveBytes'."
    }
    $requiredFreeSpaceBytes = ConvertTo-MvpPreflightInt64 `
        -Value (Get-MvpPreflightRequiredProperty -Value $preflight -Name 'required_free_space_bytes' -Label 'Staging manifest preflight') `
        -Name 'required_free_space_bytes'
    [decimal]$expectedRequired = [decimal]$inputCopyBytes + [decimal]$evidenceReserveBytes
    if ($expectedRequired -gt [Int64]::MaxValue -or $requiredFreeSpaceBytes -ne [Int64]$expectedRequired) {
        throw "Staging manifest preflight required_free_space_bytes '$requiredFreeSpaceBytes' is inconsistent with its input and evidence budgets."
    }
    $availableFreeSpaceBytes = ConvertTo-MvpPreflightInt64 `
        -Value (Get-MvpPreflightRequiredProperty -Value $preflight -Name 'available_free_space_bytes' -Label 'Staging manifest preflight') `
        -Name 'available_free_space_bytes'
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
    foreach ($name in @('session_id', 'monitor_count')) {
        if ($null -eq $desktop.PSObject.Properties[$name]) {
            throw "Staging manifest interactive_desktop is missing '$name'."
        }
    }
    if (-not $required) {
        throw 'MVP acceptance interactive desktop evidence is required.'
    }
    if (-not $userInteractive) {
        throw 'Staging manifest interactive_desktop required an interactive user but user_interactive is false.'
    }
    foreach ($name in @('session_id', 'monitor_count')) {
        $value = ConvertTo-MvpPreflightInt64 -Value $desktop.$name -Name "interactive_desktop.$name"
        if ($value -le 0) {
            throw "Staging manifest interactive_desktop '$name' must be positive when a desktop is required."
        }
    }
}

Export-ModuleMember -Function Assert-MvpStagingPreflightEvidence
