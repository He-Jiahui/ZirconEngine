Set-StrictMode -Version Latest

function Get-MvpProjectOpenEvidenceValue {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Label
    )

    if ($Value -is [System.Collections.IDictionary]) {
        if (-not $Value.Contains($Name) -or $null -eq $Value[$Name] -or
            ($Value[$Name] -is [string] -and [string]::IsNullOrWhiteSpace($Value[$Name]))) {
            throw "$Label is missing '$Name'."
        }
        return $Value[$Name]
    }

    $property = $Value.PSObject.Properties[$Name]
    if ($null -eq $property -or $null -eq $property.Value -or
        ($property.Value -is [string] -and [string]::IsNullOrWhiteSpace($property.Value))) {
        throw "$Label is missing '$Name'."
    }
    return $property.Value
}

function Get-MvpProjectOpenDiagnosticToken {
    param(
        [Parameter(Mandatory)][string]$Diagnostic,
        [Parameter(Mandatory)][string]$Name
    )

    $match = [regex]::Match(
        $Diagnostic,
        '(?:^|\s)' + [regex]::Escape($Name) + '=([^\s]+)'
    )
    if (-not $match.Success) {
        throw "Editor project-open diagnostic is missing '$Name': $Diagnostic"
    }
    return $match.Groups[1].Value
}

function ConvertFrom-MvpProjectOpenDiagnosticToken {
    param(
        [Parameter(Mandatory)][string]$Value,
        [Parameter(Mandatory)][string]$Name
    )

    if ($Value -match '%(?![0-9A-Fa-f]{2})') {
        throw "Editor project-open diagnostic has malformed percent encoding for '$Name': $Value"
    }
    $decoded = [Uri]::UnescapeDataString($Value)
    if ([string]::IsNullOrWhiteSpace($decoded)) {
        throw "Editor project-open diagnostic has an empty '$Name' value."
    }
    return $decoded
}

function ConvertTo-MvpProjectOpenUInt64 {
    param(
        [Parameter(Mandatory)][string]$Value,
        [Parameter(Mandatory)][string]$Name
    )

    [UInt64]$parsed = 0
    if (-not [UInt64]::TryParse($Value, [ref]$parsed)) {
        throw "Editor project-open diagnostic has non-numeric '$Name' value '$Value'."
    }
    return $parsed
}

function Get-MvpProjectOpenRelativePath {
    param(
        [Parameter(Mandatory)][string]$StagingRoot,
        [Parameter(Mandatory)][string]$ProjectRoot
    )

    $resolvedStagingRoot = [IO.Path]::GetFullPath($StagingRoot).TrimEnd([char[]]@('\', '/'))
    $resolvedProjectRoot = [IO.Path]::GetFullPath($ProjectRoot)
    $prefix = $resolvedStagingRoot + [IO.Path]::DirectorySeparatorChar
    if (-not $resolvedProjectRoot.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Editor project-open diagnostic project root '$resolvedProjectRoot' escapes staging root '$resolvedStagingRoot'."
    }
    return $resolvedProjectRoot.Substring($prefix.Length).Replace('\', '/')
}

function Get-MvpEditorProjectOpenEvidence {
    param(
        [Parameter(Mandatory)][string]$DiagnosticText,
        [Parameter(Mandatory)][string]$StagingRoot,
        [Parameter(Mandatory)][string]$ProjectRoot
    )

    $diagnosticLines = @(
        $DiagnosticText -split '\r?\n' |
            Where-Object { $_.IndexOf('editor_project_open ', [StringComparison]::Ordinal) -ge 0 }
    )
    if ($diagnosticLines.Count -eq 0) {
        throw 'Editor project creation did not emit the editor_project_open diagnostic.'
    }
    $diagnostic = $diagnosticLines[$diagnosticLines.Count - 1]
    if ((Get-MvpProjectOpenDiagnosticToken -Diagnostic $diagnostic -Name 'result') -ne 'completed') {
        throw "Editor project-open diagnostic did not complete successfully: $diagnostic"
    }

    $reportedProjectRoot = ConvertFrom-MvpProjectOpenDiagnosticToken `
        -Value (Get-MvpProjectOpenDiagnosticToken -Diagnostic $diagnostic -Name 'project_root') `
        -Name 'project_root'
    $expectedProjectRoot = [IO.Path]::GetFullPath($ProjectRoot)
    $resolvedReportedProjectRoot = [IO.Path]::GetFullPath($reportedProjectRoot)
    if (-not $resolvedReportedProjectRoot.Equals($expectedProjectRoot, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Editor project-open diagnostic project_root '$reportedProjectRoot' differs from staged project '$expectedProjectRoot'."
    }

    $manifestIdentity = ConvertFrom-MvpProjectOpenDiagnosticToken `
        -Value (Get-MvpProjectOpenDiagnosticToken -Diagnostic $diagnostic -Name 'manifest_identity') `
        -Name 'manifest_identity'
    if ($manifestIdentity -notmatch '^.+@v[1-9][0-9]*$') {
        throw "Editor project-open diagnostic has invalid manifest_identity '$manifestIdentity'."
    }
    $sceneUri = ConvertFrom-MvpProjectOpenDiagnosticToken `
        -Value (Get-MvpProjectOpenDiagnosticToken -Diagnostic $diagnostic -Name 'scene_uri') `
        -Name 'scene_uri'
    if ($sceneUri -ne 'res://scenes/main.scene.toml') {
        throw "Editor project-open diagnostic scene_uri '$sceneUri' differs from the RenderableEmpty default scene."
    }
    $settingsSource = ConvertFrom-MvpProjectOpenDiagnosticToken `
        -Value (Get-MvpProjectOpenDiagnosticToken -Diagnostic $diagnostic -Name 'settings_source') `
        -Name 'settings_source'
    if ($settingsSource -notmatch '^persisted-[A-Za-z0-9._-]+$') {
        throw "Editor project-open diagnostic settings_source '$settingsSource' is not persisted project settings evidence."
    }

    $counts = [ordered]@{}
    foreach ($name in @(
        'registry_asset_count',
        'registry_ready_asset_count',
        'registry_failed_asset_count',
        'registry_diagnostic_count',
        'project_generation',
        'project_generation_publish_epoch',
        'catalog_asset_count'
    )) {
        $counts[$name] = ConvertTo-MvpProjectOpenUInt64 `
            -Value (Get-MvpProjectOpenDiagnosticToken -Diagnostic $diagnostic -Name $name) `
            -Name $name
    }
    if ($counts.registry_asset_count -lt 4 -or $counts.registry_ready_asset_count -lt 4) {
        throw "Editor project-open diagnostic does not report all required starter assets as ready: assets=$($counts.registry_asset_count) ready=$($counts.registry_ready_asset_count)."
    }
    if ($counts.registry_ready_asset_count -gt $counts.registry_asset_count) {
        throw 'Editor project-open diagnostic reports more ready assets than registry assets.'
    }
    if ($counts.registry_failed_asset_count -ne 0) {
        throw "Editor project-open diagnostic reports failed starter assets: $($counts.registry_failed_asset_count)."
    }
    if ($counts.project_generation -eq 0 -or $counts.project_generation_publish_epoch -eq 0) {
        throw 'Editor project-open diagnostic reports an uninitialized project generation.'
    }
    if ($counts.catalog_asset_count -lt 4) {
        throw "Editor project-open diagnostic has incomplete editor catalog evidence: $($counts.catalog_asset_count)."
    }

    return [pscustomobject][ordered]@{
        project_root = Get-MvpProjectOpenRelativePath -StagingRoot $StagingRoot -ProjectRoot $expectedProjectRoot
        manifest_identity = $manifestIdentity
        scene_uri = $sceneUri
        registry_asset_count = $counts.registry_asset_count
        registry_ready_asset_count = $counts.registry_ready_asset_count
        registry_failed_asset_count = $counts.registry_failed_asset_count
        registry_diagnostic_count = $counts.registry_diagnostic_count
        project_generation = $counts.project_generation
        project_generation_publish_epoch = $counts.project_generation_publish_epoch
        catalog_asset_count = $counts.catalog_asset_count
        settings_source = $settingsSource
    }
}

function Assert-MvpEditorProjectOpenEvidence {
    param(
        [Parameter(Mandatory)]$Evidence,
        [Parameter(Mandatory)][string]$DiagnosticText,
        [Parameter(Mandatory)][string]$StagingRoot,
        [Parameter(Mandatory)][string]$ProjectRoot
    )

    $parsed = Get-MvpEditorProjectOpenEvidence `
        -DiagnosticText $DiagnosticText `
        -StagingRoot $StagingRoot `
        -ProjectRoot $ProjectRoot
    foreach ($name in @(
        'project_root',
        'manifest_identity',
        'scene_uri',
        'registry_asset_count',
        'registry_ready_asset_count',
        'registry_failed_asset_count',
        'registry_diagnostic_count',
        'project_generation',
        'project_generation_publish_epoch',
        'catalog_asset_count',
        'settings_source'
    )) {
        $recorded = Get-MvpProjectOpenEvidenceValue -Value $Evidence -Name $name -Label 'Project creation project_open'
        if ([string]$recorded -ne [string]$parsed.$name) {
            throw "Project creation project_open '$name' differs from its captured diagnostic."
        }
    }
    return $parsed
}

Export-ModuleMember -Function Get-MvpEditorProjectOpenEvidence, Assert-MvpEditorProjectOpenEvidence
