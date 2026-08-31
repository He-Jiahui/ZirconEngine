Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$modulePath = Join-Path $repoRoot 'tools\mvp\MvpScenePersistenceEvidence.psm1'
$moduleSource = Get-Content -LiteralPath $modulePath -Raw
Import-Module $modulePath -Force -ErrorAction Stop

function New-MvpSceneSnapshot {
    param(
        [int]$NodeCount = 30,
        [switch]$MissingParent,
        [switch]$ParentCycle
    )

    $nodes = [Collections.Generic.List[object]]::new()
    for ($id = 1; $id -le $NodeCount; $id++) {
        $parent = if ($id -eq 1) { $null } else { $id - 1 }
        $nodes.Add([pscustomobject]@{
                id = $id
                name = "Node$id"
                kind = 'Entity'
                camera = $null
                mesh = $null
                directional_light = $null
                active = $true
                render_layer_mask = 1
                mobility = 'Static'
                parent = $parent
                transform = [pscustomobject]@{
                    translation = @(0.0, 0.0, 0.0)
                    rotation = @(0.0, 0.0, 0.0, 1.0)
                    scale = @(1.0, 1.0, 1.0)
                }
            })
    }
    if ($MissingParent) {
        $nodes[$NodeCount - 1].parent = $NodeCount + 100
    }
    if ($ParentCycle) {
        $nodes[0].parent = $NodeCount
    }
    return [pscustomobject]@{ scene_nodes = $nodes }
}

function Assert-MvpSceneFixture {
    param([Parameter(Mandatory)]$Snapshot)

    Assert-MvpAutomationSceneSnapshot `
        -Snapshot $Snapshot `
        -SceneEntryCount $Snapshot.scene_nodes.Count `
        -SelectedNodeId $Snapshot.scene_nodes.Count `
        -SelectedNodeName "Node$($Snapshot.scene_nodes.Count)" `
        -InspectorTranslation @(0.0, 0.0, 0.0) `
        -InspectorScale @(1.0, 1.0, 1.0) `
        -Label 'fixture'
}

function New-MvpSceneDeltaFixture {
    $baselineNodes = @(
        [pscustomobject][ordered]@{ id = 1; name = 'Root'; transform = [pscustomobject][ordered]@{ translation = @(0.0, 0.0, 0.0); rotation = @(0.0, 0.0, 0.0, 1.0); scale = @(1.0, 1.0, 1.0) } },
        [pscustomobject][ordered]@{ id = 2; name = 'Cube'; transform = [pscustomobject][ordered]@{ translation = @(0.0, 0.0, 0.0); rotation = @(0.0, 0.0, 0.0, 1.0); scale = @(1.0, 1.0, 1.0) } }
    )
    $authoringNodes = ConvertTo-Json -InputObject $baselineNodes -Depth 8 | ConvertFrom-Json
    $authoringNodes[1].transform.translation[0] = 42.0
    $authoringNodes[1].transform.scale[0] = 1.25
    return [pscustomobject]@{
        baseline = [pscustomobject]@{ scene_nodes = $baselineNodes; selected_node_id = 2 }
        authoring = [pscustomobject]@{ scene_nodes = $authoringNodes; selected_node_id = 2 }
    }
}

Describe 'MVP scene-persistence evidence' {
    It 'uses one parent map as both node identity set and graph storage' {
        $moduleSource | Should Match '\$parentsByNode\s*=\s*@\{\}'
        $moduleSource | Should Not Match '\$nodeEvidence'
        $moduleSource | Should Not Match '\[Collections\.Generic\.HashSet\[UInt64\]\]::new\(\)'
    }

    It 'validates the parent graph once with shared tri-color visit state' {
        $moduleSource | Should Match 'function Assert-MvpSceneParentGraph'
        $moduleSource | Should Match '\$visitState\s*=\s*@\{\}'
        $moduleSource | Should Match '\$path\s*=\s*\[Collections\.Generic\.List\[UInt64\]\]::new\(\$ParentsByNode\.Count\)'
        $moduleSource | Should Match '\$path\.Clear\(\)'
    }

    It 'clones only authoring nodes while comparing the expected delta' {
        ([regex]::Matches($moduleSource, '\| ConvertFrom-Json')).Count | Should Be 1
        $moduleSource | Should Match '\$baselineNodes\s*=\s*\$BaselineSnapshot\.scene_nodes'
        $moduleSource | Should Not Match '\$baselineNodes\s*=\s*@\(\$BaselineSnapshot\.scene_nodes\)'
    }

    It 'accepts a deep acyclic parent chain' {
        { Assert-MvpSceneFixture -Snapshot (New-MvpSceneSnapshot) } | Should Not Throw
    }

    It 'rejects a missing parent' {
        { Assert-MvpSceneFixture -Snapshot (New-MvpSceneSnapshot -MissingParent) } |
            Should Throw 'references missing parent'
    }

    It 'rejects a parent cycle' {
        { Assert-MvpSceneFixture -Snapshot (New-MvpSceneSnapshot -ParentCycle) } |
            Should Throw 'contains a parent cycle'
    }

    It 'accepts only the requested Cube delta without mutating either snapshot' {
        $fixture = New-MvpSceneDeltaFixture

        { Assert-MvpExpectedAuthoringSceneDelta -BaselineSnapshot $fixture.baseline -AuthoringSnapshot $fixture.authoring } |
            Should Not Throw

        $fixture.baseline.scene_nodes[1].transform.translation[0] | Should Be 0.0
        $fixture.authoring.scene_nodes[1].transform.translation[0] | Should Be 42.0
        $fixture.authoring.scene_nodes[1].transform.scale[0] | Should Be 1.25
    }

    It 'rejects an authoring change outside the requested Cube delta' {
        $fixture = New-MvpSceneDeltaFixture
        $fixture.authoring.scene_nodes[0].name = 'ChangedRoot'

        { Assert-MvpExpectedAuthoringSceneDelta -BaselineSnapshot $fixture.baseline -AuthoringSnapshot $fixture.authoring } |
            Should Throw 'differs from the baseline outside'
    }
}
