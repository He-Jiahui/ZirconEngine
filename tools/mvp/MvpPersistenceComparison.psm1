Set-StrictMode -Version Latest

Import-Module (Join-Path $PSScriptRoot 'MvpAcceptanceNativeFileSystem.psm1') -Force -DisableNameChecking -ErrorAction Stop

$mvpPersistenceComparisonUtf8NoBom = [Text.UTF8Encoding]::new($false)
$mvpPersistenceComparisonStatePropertyNames = [string[]]@(
    'project_identity',
    'manifest_identity',
    'scene_uri',
    'selected_model_resource_id',
    'selected_material_resource_id',
    'opened_project_inspection_generation',
    'snapshot'
)

function Select-MvpPersistenceState {
    param(
        [Parameter(Mandatory)]$Automation,
        [Parameter(Mandatory)][string]$Phase
    )

    $label = "$Phase automation"
    $properties = $Automation.PSObject.Properties
    $state = [ordered]@{
        schema_version = 1
        phase = $Phase
    }
    foreach ($name in $mvpPersistenceComparisonStatePropertyNames) {
        $property = $properties[$name]
        if ($null -eq $property -or $null -eq $property.Value) {
            throw "$label is missing '$name'."
        }
        $state[$name] = $property.Value
    }
    return $state
}

function Write-MvpPersistenceComparisonJson {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)]$Value,
        [string]$CompatibleWriteLeaseRoot
    )

    $contentBytes = $mvpPersistenceComparisonUtf8NoBom.GetBytes(
        (ConvertTo-Json -InputObject $Value -Depth 64))
    Write-MvpAcceptanceNewFileNoFollow `
        -Path $Path `
        -ContentBytes $contentBytes `
        -CompatibleWriteLeaseRoot $CompatibleWriteLeaseRoot
}

function Write-MvpPersistenceComparisonEvidence {
    param(
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)]$BaselineAutomation,
        [Parameter(Mandatory)]$AuthoringAutomation,
        [Parameter(Mandatory)]$ReopenAutomation,
        [string]$CompatibleWriteLeaseRoot
    )

    if (-not (Test-Path -LiteralPath $EvidenceRoot -PathType Container)) {
        throw "Persistence comparison evidence root '$EvidenceRoot' does not exist."
    }
    $reopenReports = @($ReopenAutomation)
    if ($reopenReports.Count -ne 2) {
        throw "Persistence comparison requires exactly two reopen reports; found $($reopenReports.Count)."
    }

    $before = Select-MvpPersistenceState `
        -Automation $BaselineAutomation `
        -Phase 'before-authoring'
    $after = Select-MvpPersistenceState `
        -Automation $AuthoringAutomation `
        -Phase 'after-authoring'
    $projectSaveLifecycleProperty = $AuthoringAutomation.PSObject.Properties['project_save_lifecycle']
    if ($null -eq $projectSaveLifecycleProperty -or $null -eq $projectSaveLifecycleProperty.Value) {
        throw "After-authoring automation is missing 'project_save_lifecycle'."
    }
    $after['project_save_lifecycle'] = $projectSaveLifecycleProperty.Value
    $reopened = [ordered]@{
        schema_version = 1
        phase = 'reopened'
        runs = @(
            for ($index = 0; $index -lt $reopenReports.Count; $index++) {
                $state = Select-MvpPersistenceState `
                    -Automation $reopenReports[$index] `
                    -Phase "reopen-$($index + 1)"
                [pscustomobject]$state
            }
        )
    }

    $comparisonRoot = [IO.Path]::Combine($EvidenceRoot, 'comparison')
    $null = Ensure-MvpAcceptanceDirectoryPathNoFollow `
        -RootPath $EvidenceRoot `
        -RelativePath 'comparison' `
        -CompatibleWriteLeaseRoot $CompatibleWriteLeaseRoot
    return @(
        [pscustomobject]@{
            relative_path = 'comparison/persisted-state-before.json'
            content_bytes = Write-MvpPersistenceComparisonJson `
                -Path ([IO.Path]::Combine($comparisonRoot, 'persisted-state-before.json')) `
                -Value $before `
                -CompatibleWriteLeaseRoot $CompatibleWriteLeaseRoot
        }
        [pscustomobject]@{
            relative_path = 'comparison/persisted-state-after.json'
            content_bytes = Write-MvpPersistenceComparisonJson `
                -Path ([IO.Path]::Combine($comparisonRoot, 'persisted-state-after.json')) `
                -Value $after `
                -CompatibleWriteLeaseRoot $CompatibleWriteLeaseRoot
        }
        [pscustomobject]@{
            relative_path = 'comparison/reopened-state.json'
            content_bytes = Write-MvpPersistenceComparisonJson `
                -Path ([IO.Path]::Combine($comparisonRoot, 'reopened-state.json')) `
                -Value $reopened `
                -CompatibleWriteLeaseRoot $CompatibleWriteLeaseRoot
        }
    )
}

Export-ModuleMember -Function Write-MvpPersistenceComparisonEvidence
