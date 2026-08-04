Set-StrictMode -Version Latest

Import-Module (Join-Path $PSScriptRoot 'MvpAcceptanceNativeFileSystem.psm1') -Force -DisableNameChecking -ErrorAction Stop

function Get-MvpPersistenceComparisonValue {
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

function Select-MvpPersistenceState {
    param(
        [Parameter(Mandatory)]$Automation,
        [Parameter(Mandatory)][string]$Phase
    )

    $label = "$Phase automation"
    return [ordered]@{
        schema_version = 1
        phase = $Phase
        project_identity = Get-MvpPersistenceComparisonValue -Value $Automation -Name 'project_identity' -Label $label
        manifest_identity = Get-MvpPersistenceComparisonValue -Value $Automation -Name 'manifest_identity' -Label $label
        scene_uri = Get-MvpPersistenceComparisonValue -Value $Automation -Name 'scene_uri' -Label $label
        selected_model_resource_id = Get-MvpPersistenceComparisonValue -Value $Automation -Name 'selected_model_resource_id' -Label $label
        selected_material_resource_id = Get-MvpPersistenceComparisonValue -Value $Automation -Name 'selected_material_resource_id' -Label $label
        opened_project_inspection_generation = Get-MvpPersistenceComparisonValue -Value $Automation -Name 'opened_project_inspection_generation' -Label $label
        snapshot = Get-MvpPersistenceComparisonValue -Value $Automation -Name 'snapshot' -Label $label
    }
}

function Write-MvpPersistenceComparisonJson {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)]$Value,
        [string]$CompatibleWriteLeaseRoot
    )

    $contentBytes = [Text.UTF8Encoding]::new($false).GetBytes(
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
    $after['project_save_lifecycle'] = Get-MvpPersistenceComparisonValue `
        -Value $AuthoringAutomation `
        -Name 'project_save_lifecycle' `
        -Label 'After-authoring automation'
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

    $comparisonRoot = Join-Path $EvidenceRoot 'comparison'
    Ensure-MvpAcceptanceDirectoryPathNoFollow `
        -RootPath $EvidenceRoot `
        -RelativePath 'comparison' `
        -CompatibleWriteLeaseRoot $CompatibleWriteLeaseRoot | Out-Null
    return @(
        [pscustomobject]@{
            relative_path = 'comparison/persisted-state-before.json'
            content_bytes = Write-MvpPersistenceComparisonJson `
                -Path (Join-Path $comparisonRoot 'persisted-state-before.json') `
                -Value $before `
                -CompatibleWriteLeaseRoot $CompatibleWriteLeaseRoot
        }
        [pscustomobject]@{
            relative_path = 'comparison/persisted-state-after.json'
            content_bytes = Write-MvpPersistenceComparisonJson `
                -Path (Join-Path $comparisonRoot 'persisted-state-after.json') `
                -Value $after `
                -CompatibleWriteLeaseRoot $CompatibleWriteLeaseRoot
        }
        [pscustomobject]@{
            relative_path = 'comparison/reopened-state.json'
            content_bytes = Write-MvpPersistenceComparisonJson `
                -Path (Join-Path $comparisonRoot 'reopened-state.json') `
                -Value $reopened `
                -CompatibleWriteLeaseRoot $CompatibleWriteLeaseRoot
        }
    )
}

Export-ModuleMember -Function Write-MvpPersistenceComparisonEvidence
