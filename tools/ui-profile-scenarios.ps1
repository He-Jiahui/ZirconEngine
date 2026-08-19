function Get-ZirconUiProfileCaptureScenarioDefinitions {
    return @(
        [pscustomobject]@{ id = 'manual'; instruction = 'Perform the target interaction, then close the editor to export the report.'; include_in_all = $false }
        [pscustomobject]@{ id = 'startup'; instruction = 'Launch, wait until the first editor frame is stable, then close the editor.'; include_in_all = $true }
        [pscustomobject]@{ id = 'material_lab_startup'; instruction = 'Launch the Material Component Lab, wait until the first frame is stable, then close the editor.'; include_in_all = $true }
        [pscustomobject]@{ id = 'material_lab_hover'; instruction = 'Launch the Material Component Lab, move the pointer across prototype controls, then close.'; include_in_all = $true }
        [pscustomobject]@{ id = 'material_lab_click'; instruction = 'Launch the Material Component Lab, click representative prototype controls, then close.'; include_in_all = $true }
        [pscustomobject]@{ id = 'idle_hover'; instruction = 'Move the pointer slowly across toolbar, hierarchy rows, inspector fields, and tabs for several seconds, then close.'; include_in_all = $true }
        [pscustomobject]@{ id = 'click'; instruction = 'Click toolbar buttons, hierarchy rows, tabs, and inspector controls, then close.'; include_in_all = $true }
        [pscustomobject]@{ id = 'viewport_toolbar_click'; instruction = 'Click source-bound controls in the live scene viewport toolbar, then close.'; include_in_all = $true }
        [pscustomobject]@{ id = 'drag'; instruction = 'Drag selection or draggable editor controls where available, then close.'; include_in_all = $true }
        [pscustomobject]@{ id = 'drawer_resize'; instruction = 'Drag side or bottom pane splitters repeatedly, then close.'; include_in_all = $true }
        [pscustomobject]@{ id = 'window_resize'; instruction = 'Resize the native editor window repeatedly, restore its original extent, then close.'; include_in_all = $true }
        [pscustomobject]@{ id = 'hierarchy_scroll'; instruction = 'Scroll the live hierarchy pane in alternating directions, then close.'; include_in_all = $true }
        [pscustomobject]@{ id = 'welcome_recent_scroll'; instruction = 'Scroll the Welcome recent-project viewport in alternating directions, then close.'; include_in_all = $true }
        [pscustomobject]@{ id = 'asset_refresh'; instruction = 'Trigger an asset refresh or reopen the project/asset pane, then close.'; include_in_all = $true }
        [pscustomobject]@{ id = 'viewport_image'; instruction = 'Let the scene/game viewport update for several seconds, orbit if useful, then close.'; include_in_all = $true }
    )
}

function Resolve-ZirconUiProfileCaptureScenarios {
    param(
        [string]$Scenario = 'manual',
        [string[]]$ScenarioList = @(),
        [switch]$AllUiScenarios
    )

    $definitions = @(Get-ZirconUiProfileCaptureScenarioDefinitions)
    if ($AllUiScenarios) {
        return @($definitions | Where-Object { $_.include_in_all } | ForEach-Object { $_.id })
    }

    $requestedNames = @()
    $sources = if ($ScenarioList.Count -gt 0) { $ScenarioList } else { @($Scenario) }
    foreach ($source in $sources) {
        foreach ($part in ([string]$source -split ',')) {
            $name = $part.Trim()
            if ([string]::IsNullOrWhiteSpace($name)) {
                continue
            }
            $requestedNames += $name
        }
    }
    if ($requestedNames.Count -eq 0) {
        throw 'At least one UI profile scenario must be specified.'
    }

    $resolved = @()
    foreach ($requestedName in $requestedNames) {
        $definition = $definitions |
            Where-Object { $_.id.Equals($requestedName, [System.StringComparison]::OrdinalIgnoreCase) } |
            Select-Object -First 1
        if ($null -eq $definition) {
            throw "Unsupported UI profile scenario: $requestedName"
        }
        $resolved += $definition.id
    }
    return $resolved
}

function Get-ZirconUiProfileCaptureScenarioInstruction {
    param([Parameter(Mandatory = $true)][string]$ScenarioId)

    $definition = @(Get-ZirconUiProfileCaptureScenarioDefinitions) |
        Where-Object { $_.id.Equals($ScenarioId, [System.StringComparison]::OrdinalIgnoreCase) } |
        Select-Object -First 1
    if ($null -eq $definition) {
        throw "Unsupported UI profile scenario: $ScenarioId"
    }
    return $definition.instruction
}
