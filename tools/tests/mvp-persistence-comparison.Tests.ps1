Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$modulePath = Join-Path $repoRoot 'tools\mvp\MvpPersistenceComparison.psm1'
$moduleSource = Get-Content -LiteralPath $modulePath -Raw
Import-Module $modulePath -Force -ErrorAction Stop

function New-MvpPersistenceAutomation {
    return [pscustomobject]@{
        project_identity = 'project-identity'
        manifest_identity = 'manifest-identity'
        scene_uri = 'res://scenes/main.scene.toml'
        selected_model_resource_id = 'model-resource'
        selected_material_resource_id = 'material-resource'
        opened_project_inspection_generation = 1
        snapshot = [ordered]@{ schema_version = 1; selected_node_id = 3 }
        project_save_lifecycle = [ordered]@{ schema_version = 1; save_mark = 'Marked' }
    }
}

Describe 'MVP persistence-comparison evidence' {
    It 'reuses one BOM-less UTF-8 encoder across all comparison files' {
        $moduleSource | Should Match '\$mvpPersistenceComparisonUtf8NoBom\s*=\s*\[Text\.UTF8Encoding\]::new\(\$false\)'
        $moduleSource | Should Match '\$mvpPersistenceComparisonUtf8NoBom\.GetBytes\('
    }

    It 'projects each persistence state from one cached property collection' {
        $moduleSource | Should Match '\$mvpPersistenceComparisonStatePropertyNames\s*=\s*\[string\[\]\]@\('
        $moduleSource | Should Match '\$properties\s*=\s*\$Automation\.PSObject\.Properties'
        $moduleSource | Should Match 'foreach \(\$name in \$mvpPersistenceComparisonStatePropertyNames\)'
        $moduleSource | Should Not Match 'function Get-MvpPersistenceComparisonValue'
    }

    It 'composes the fixed comparison publication paths through System.IO' {
        $moduleSource | Should Match '\$comparisonRoot\s*=\s*\[IO\.Path\]::Combine\(\$EvidenceRoot, ''comparison''\)'
        ([regex]::Matches($moduleSource, '\[IO\.Path\]::Combine\(\$comparisonRoot,')).Count | Should Be 3
        $moduleSource | Should Not Match 'Join-Path\s+\$comparisonRoot'
    }

    It 'discards directory admission output without an Out-Null pipeline' {
        $moduleSource | Should Match '\$null\s*=\s*Ensure-MvpAcceptanceDirectoryPathNoFollow'
        $moduleSource | Should Not Match 'Ensure-MvpAcceptanceDirectoryPathNoFollow(?s).*?\|\s*Out-Null'
    }

    It 'publishes the before after and reopened evidence files' {
        $evidenceRoot = Join-Path $TestDrive 'persistence-evidence'
        $null = New-Item -ItemType Directory -Path $evidenceRoot
        $automation = New-MvpPersistenceAutomation

        $receipts = @(Write-MvpPersistenceComparisonEvidence `
                -EvidenceRoot $evidenceRoot `
                -BaselineAutomation $automation `
                -AuthoringAutomation $automation `
                -ReopenAutomation @($automation, $automation))

        $receipts.Count | Should Be 3
        (@($receipts.relative_path) -join ',') | Should Be 'comparison/persisted-state-before.json,comparison/persisted-state-after.json,comparison/reopened-state.json'
        $before = Get-Content -LiteralPath (Join-Path $evidenceRoot 'comparison\persisted-state-before.json') -Raw | ConvertFrom-Json
        $after = Get-Content -LiteralPath (Join-Path $evidenceRoot 'comparison\persisted-state-after.json') -Raw | ConvertFrom-Json
        $reopened = Get-Content -LiteralPath (Join-Path $evidenceRoot 'comparison\reopened-state.json') -Raw | ConvertFrom-Json
        $before.phase | Should Be 'before-authoring'
        $after.project_save_lifecycle.save_mark | Should Be 'Marked'
        @($reopened.runs).Count | Should Be 2
    }

    It 'rejects an incomplete reopen set before publication' {
        $evidenceRoot = Join-Path $TestDrive 'incomplete-reopen'
        $null = New-Item -ItemType Directory -Path $evidenceRoot
        $automation = New-MvpPersistenceAutomation

        {
            Write-MvpPersistenceComparisonEvidence `
                -EvidenceRoot $evidenceRoot `
                -BaselineAutomation $automation `
                -AuthoringAutomation $automation `
                -ReopenAutomation @($automation)
        } | Should Throw 'requires exactly two reopen reports'
    }

    It 'rejects a persistence state with a missing required property' {
        $evidenceRoot = Join-Path $TestDrive 'missing-state-property'
        $null = New-Item -ItemType Directory -Path $evidenceRoot
        $automation = New-MvpPersistenceAutomation
        $automation.PSObject.Properties.Remove('manifest_identity')

        {
            Write-MvpPersistenceComparisonEvidence `
                -EvidenceRoot $evidenceRoot `
                -BaselineAutomation $automation `
                -AuthoringAutomation $automation `
                -ReopenAutomation @($automation, $automation)
        } | Should Throw "is missing 'manifest_identity'"
    }
}
