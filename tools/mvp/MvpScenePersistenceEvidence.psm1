Set-StrictMode -Version Latest

function Get-MvpSceneRequiredProperty {
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

function ConvertTo-MvpSceneUInt64 {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Label
    )

    [UInt64]$parsed = 0
    if (-not [UInt64]::TryParse([string]$Value, [ref]$parsed)) {
        throw "$Label has non-numeric '$Name' value '$Value'."
    }
    return $parsed
}

function Assert-MvpSceneVector {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][int]$ExpectedCount,
        [Parameter(Mandatory)][string]$Label
    )

    $values = @($Value)
    if ($values.Count -ne $ExpectedCount) {
        throw "$Label must contain exactly $ExpectedCount values; found $($values.Count)."
    }
    foreach ($value in $values) {
        [double]$parsed = 0
        if (-not [double]::TryParse([string]$value, [ref]$parsed) -or
            [double]::IsNaN($parsed) -or [double]::IsInfinity($parsed)) {
            throw "$Label contains non-finite numeric value '$value'."
        }
    }
}

function Assert-MvpSceneParentGraph {
    param(
        [Parameter(Mandatory)][System.Collections.IDictionary]$ParentsByNode,
        [Parameter(Mandatory)][string]$Label
    )

    foreach ($entry in $ParentsByNode.GetEnumerator()) {
        if ($entry.Value -ne 0 -and -not $ParentsByNode.ContainsKey($entry.Value)) {
            throw "$Label scene node '$($entry.Key)' references missing parent '$($entry.Value)'."
        }
    }

    $visitState = @{}
    $path = [Collections.Generic.List[UInt64]]::new($ParentsByNode.Count)
    foreach ($nodeId in $ParentsByNode.Keys) {
        if ($visitState[$nodeId] -eq 2) {
            continue
        }

        $path.Clear()
        [UInt64]$currentNodeId = $nodeId
        while ($currentNodeId -ne 0) {
            $state = $visitState[$currentNodeId]
            if ($state -eq 1) {
                throw "$Label scene_nodes contains a parent cycle involving node '$currentNodeId'."
            }
            if ($state -eq 2) {
                break
            }
            $visitState[$currentNodeId] = 1
            $path.Add($currentNodeId)
            $currentNodeId = $ParentsByNode[$currentNodeId]
        }
        foreach ($pathNodeId in $path) {
            $visitState[$pathNodeId] = 2
        }
    }
}

function Assert-MvpAutomationSceneSnapshot {
    param(
        [Parameter(Mandatory)]$Snapshot,
        [Parameter(Mandatory)][UInt64]$SceneEntryCount,
        [Parameter(Mandatory)][UInt64]$SelectedNodeId,
        [Parameter(Mandatory)][string]$SelectedNodeName,
        [Parameter(Mandatory)]$InspectorTranslation,
        [Parameter(Mandatory)]$InspectorScale,
        [Parameter(Mandatory)][string]$Label
    )

    $sceneNodes = @(Get-MvpSceneRequiredProperty -Value $Snapshot -Name 'scene_nodes' -Label $Label)
    if ($sceneNodes.Count -ne $SceneEntryCount) {
        throw "$Label scene_nodes count '$($sceneNodes.Count)' differs from scene_entry_count '$SceneEntryCount'."
    }

    $parentsByNode = @{}
    $selectedNodeFound = $false
    foreach ($node in $sceneNodes) {
        $nodeId = ConvertTo-MvpSceneUInt64 `
            -Value (Get-MvpSceneRequiredProperty -Value $node -Name 'id' -Label "$Label scene node") `
            -Name 'id' `
            -Label "$Label scene node"
        if ($nodeId -eq 0 -or $parentsByNode.ContainsKey($nodeId)) {
            throw "$Label scene_nodes contains zero or duplicate node id '$nodeId'."
        }
        $parentsByNode[$nodeId] = [UInt64]0
        $nodeName = [string](Get-MvpSceneRequiredProperty -Value $node -Name 'name' -Label "$Label scene node $nodeId")
        $null = Get-MvpSceneRequiredProperty -Value $node -Name 'kind' -Label "$Label scene node $nodeId"
        foreach ($componentName in @('camera', 'mesh', 'directional_light')) {
            if ($null -eq $node.PSObject.Properties[$componentName]) {
                throw "$Label scene node $nodeId is missing '$componentName'."
            }
        }
        foreach ($fieldName in @('active', 'render_layer_mask', 'mobility')) {
            $null = Get-MvpSceneRequiredProperty -Value $node -Name $fieldName -Label "$Label scene node $nodeId"
        }
        if ($node.active -isnot [bool]) {
            throw "$Label scene node $nodeId has non-boolean 'active'."
        }
        $null = ConvertTo-MvpSceneUInt64 -Value $node.render_layer_mask -Name 'render_layer_mask' -Label "$Label scene node $nodeId"

        $parentProperty = $node.PSObject.Properties['parent']
        if ($null -eq $parentProperty) {
            throw "$Label scene node $nodeId is missing 'parent'."
        }
        [UInt64]$parentId = 0
        if ($null -ne $parentProperty.Value) {
            $parentId = ConvertTo-MvpSceneUInt64 -Value $parentProperty.Value -Name 'parent' -Label "$Label scene node $nodeId"
            if ($parentId -eq $nodeId) {
                throw "$Label scene node $nodeId cannot parent itself."
            }
        }
        $parentsByNode[$nodeId] = $parentId

        $transform = Get-MvpSceneRequiredProperty -Value $node -Name 'transform' -Label "$Label scene node $nodeId"
        $translation = @(Get-MvpSceneRequiredProperty -Value $transform -Name 'translation' -Label "$Label scene node $nodeId transform")
        $scale = @(Get-MvpSceneRequiredProperty -Value $transform -Name 'scale' -Label "$Label scene node $nodeId transform")
        Assert-MvpSceneVector -Value $translation -ExpectedCount 3 -Label "$Label scene node $nodeId translation"
        Assert-MvpSceneVector `
            -Value (Get-MvpSceneRequiredProperty -Value $transform -Name 'rotation' -Label "$Label scene node $nodeId transform") `
            -ExpectedCount 4 `
            -Label "$Label scene node $nodeId rotation"
        Assert-MvpSceneVector `
            -Value $scale `
            -ExpectedCount 3 `
            -Label "$Label scene node $nodeId scale"

        if ($nodeId -eq $SelectedNodeId) {
            if ($nodeName -ne $SelectedNodeName) {
                throw "$Label selected scene node name '$nodeName' differs from '$SelectedNodeName'."
            }
            for ($axis = 0; $axis -lt 3; $axis++) {
                if ([double]$translation[$axis] -ne [double]$InspectorTranslation[$axis]) {
                    throw "$Label selected scene node translation differs from inspector_translation."
                }
            }
            for ($axis = 0; $axis -lt 3; $axis++) {
                if ([double]$scale[$axis] -ne [double]$InspectorScale[$axis]) {
                    throw "$Label selected scene node scale differs from inspector_scale."
                }
            }
            $selectedNodeFound = $true
        }
    }

    if (-not $selectedNodeFound) {
        throw "$Label scene_nodes does not contain selected node '$SelectedNodeId'."
    }
    Assert-MvpSceneParentGraph -ParentsByNode $parentsByNode -Label $Label
}

function Assert-MvpSceneSnapshotMatch {
    param(
        [Parameter(Mandatory)]$ExpectedSnapshot,
        [Parameter(Mandatory)]$ActualSnapshot,
        [Parameter(Mandatory)][string]$Label
    )

    $expected = ConvertTo-Json -InputObject $ExpectedSnapshot.scene_nodes -Depth 64 -Compress
    $actual = ConvertTo-Json -InputObject $ActualSnapshot.scene_nodes -Depth 64 -Compress
    if ($actual -ne $expected) {
        throw "$Label scene_nodes differs from the authoring snapshot."
    }
}

function Assert-MvpExpectedAuthoringSceneDelta {
    param(
        [Parameter(Mandatory)]$BaselineSnapshot,
        [Parameter(Mandatory)]$AuthoringSnapshot
    )

    $baselineNodes = $BaselineSnapshot.scene_nodes
    $authoringNodes = ConvertTo-Json -InputObject $AuthoringSnapshot.scene_nodes -Depth 64 | ConvertFrom-Json
    $selectedNodeId = [UInt64]$AuthoringSnapshot.selected_node_id
    $baselineSelected = $null
    $authoringSelected = $null
    $baselineSelectedCount = 0
    $authoringSelectedCount = 0
    foreach ($node in $baselineNodes) {
        if ([UInt64]$node.id -eq $selectedNodeId) {
            $baselineSelected = $node
            $baselineSelectedCount++
        }
    }
    foreach ($node in $authoringNodes) {
        if ([UInt64]$node.id -eq $selectedNodeId) {
            $authoringSelected = $node
            $authoringSelectedCount++
        }
    }
    if ($baselineSelectedCount -ne 1 -or $authoringSelectedCount -ne 1) {
        throw 'Authoring scene delta cannot resolve the selected node in both snapshots.'
    }
    if ([double]$baselineSelected.transform.translation[0] -ne 0.0 -or
        [double]$authoringSelected.transform.translation[0] -ne 42.0 -or
        [double]$baselineSelected.transform.scale[0] -ne 1.0 -or
        [double]$authoringSelected.transform.scale[0] -ne 1.25) {
        throw 'Authoring scene delta must change the selected Cube X translation from 0 to 42 and scale from 1 to 1.25.'
    }
    $authoringSelected.transform.translation[0] = $baselineSelected.transform.translation[0]
    $authoringSelected.transform.scale[0] = $baselineSelected.transform.scale[0]
    $baseline = ConvertTo-Json -InputObject $baselineNodes -Depth 64 -Compress
    $authoring = ConvertTo-Json -InputObject $authoringNodes -Depth 64 -Compress
    if ($authoring -ne $baseline) {
        throw 'Authoring scene_nodes differs from the baseline outside the requested Cube X translation and scale.'
    }
}

Export-ModuleMember -Function @(
    'Assert-MvpAutomationSceneSnapshot',
    'Assert-MvpSceneSnapshotMatch',
    'Assert-MvpExpectedAuthoringSceneDelta'
)
