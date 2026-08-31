$script:AuditScript = Join-Path $PSScriptRoot "..\ui-structural-hotspot-audit.ps1"
if (Test-Path -LiteralPath $script:AuditScript) {
    . $script:AuditScript
}

Describe "UI structural hotspot audit" {
    It "includes screen-space renderer and RHI UI roots by default" {
        Get-Command Export-ZirconUiStructuralHotspotAudit -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty

        $fixtureRoot = "E:\zircon-profiles\pester-ui-audit-default-roots-$([guid]::NewGuid().ToString('N'))"
        $outputRoot = Join-Path $fixtureRoot "evidence"
        try {
            $sourceFiles = @(
                "zircon_runtime/src/ui/surface.rs",
                "zircon_runtime/src/graphics/scene/scene_renderer/ui/image.rs",
                "zircon_runtime/crates/zr_rhi/src/ui_surface.rs",
                "zircon_runtime/crates/zr_rhi/src/ui_surface/image_resources.rs",
                "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface.rs",
                "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/image_cache.rs",
                "zircon_runtime_interface/src/ui/surface.rs",
                "zircon_editor/src/ui/retained_host.rs"
            )
            foreach ($relativePath in $sourceFiles) {
                $absolutePath = Join-Path $fixtureRoot $relativePath
                New-Item -ItemType Directory -Path (Split-Path -Parent $absolutePath) -Force |
                    Out-Null
                "fn retained_ui_product() { consume(source.clone()); }" |
                    Set-Content -LiteralPath $absolutePath -Encoding UTF8
            }

            $ignoredTest = Join-Path $fixtureRoot `
                "zircon_runtime/src/graphics/scene/scene_renderer/ui/tests/noise.rs"
            New-Item -ItemType Directory -Path (Split-Path -Parent $ignoredTest) -Force |
                Out-Null
            "fn ignored() { consume(source.clone()); }" |
                Set-Content -LiteralPath $ignoredTest -Encoding UTF8

            $receipt = Export-ZirconUiStructuralHotspotAudit `
                -RepoRoot $fixtureRoot `
                -OutputDirectory $outputRoot

            $report = Get-Content -LiteralPath $receipt.json_path -Raw | ConvertFrom-Json
            $receipt.file_count | Should Be $sourceFiles.Count
            @($report.hotspots.path | Sort-Object) |
                Should Be @($sourceFiles | Sort-Object)
            @($report.hotspots | Where-Object { $_.path -match "ui_surface" }).Count |
                Should Be 4
            @($report.hotspots | Where-Object { $_.path -match "ui_surface" }).domain |
                ForEach-Object { $_ | Should Be "render" }
            @($report.hotspots.path) -contains `
                "zircon_runtime/src/graphics/scene/scene_renderer/ui/tests/noise.rs" |
                Should Be $false
        }
        finally {
            if (Test-Path -LiteralPath $fixtureRoot) {
                Remove-Item -LiteralPath $fixtureRoot -Recurse -Force
            }
        }
    }

    It "exports deterministic production-source hotspot evidence on an approved drive" {
        Get-Command Export-ZirconUiStructuralHotspotAudit -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty

        $fixtureRoot = "E:\zircon-profiles\pester-ui-audit-$([guid]::NewGuid().ToString('N'))"
        $outputRoot = Join-Path $fixtureRoot "evidence"
        try {
            $runtimeRoot = Join-Path $fixtureRoot "zircon_runtime\src\ui\layout"
            $editorRoot = Join-Path $fixtureRoot "zircon_editor\src\ui\retained_host"
            $ignoredRoot = Join-Path $fixtureRoot "zircon_runtime\src\ui\tests"
            $ignoredSuffixRoot = Join-Path $fixtureRoot "zircon_editor\src\ui\retained_host\paint_tests"
            New-Item -ItemType Directory `
                -Path $runtimeRoot, $editorRoot, $ignoredRoot, $ignoredSuffixRoot `
                -Force |
                Out-Null
            @'
fn recompute(nodes: &[Node]) {
    let cloned = nodes.to_vec();
    let mut names = nodes.iter().map(|node| node.name.to_string()).collect::<Vec<_>>();
    names.sort();
    for node in nodes.iter() {
        consume(node.clone());
    }
}
'@ | Set-Content -LiteralPath (Join-Path $runtimeRoot "incremental.rs") -Encoding UTF8
            @'
fn publish(commands: &Commands) {
    for command in commands.iter() {
        submit(command.clone());
    }
}
'@ | Set-Content -LiteralPath (Join-Path $editorRoot "publication.rs") -Encoding UTF8
            @'
fn ignored_test_noise(value: &Value) {
    consume(value.clone());
    consume(value.clone());
}
'@ | Set-Content -LiteralPath (Join-Path $ignoredRoot "noise.rs") -Encoding UTF8
            @'
fn ignored_suffix_test_noise(value: &Value) {
    consume(value.clone());
    consume(value.clone());
}
'@ | Set-Content -LiteralPath (Join-Path $ignoredSuffixRoot "paint.rs") -Encoding UTF8

            $receipt = Export-ZirconUiStructuralHotspotAudit `
                -RepoRoot $fixtureRoot `
                -OutputDirectory $outputRoot `
                -SourceRoots @("zircon_runtime/src/ui", "zircon_editor/src/ui")

            $receipt.schema_version | Should Be 1
            $receipt.file_count | Should Be 2
            Test-Path -LiteralPath $receipt.json_path | Should Be $true
            Test-Path -LiteralPath $receipt.csv_path | Should Be $true

            $report = Get-Content -LiteralPath $receipt.json_path -Raw | ConvertFrom-Json
            $report.schema_version | Should Be 1
            $report.source_binding.repo_root | Should Be $fixtureRoot
            $report.summary.file_count | Should Be 2
            $report.summary.clone_calls | Should Be 2
            $report.summary.vec_materializations | Should Be 2
            $report.summary.sort_calls | Should Be 1
            $report.summary.string_allocations | Should Be 1
            $report.summary.traversal_signals | Should Be 3
            $report.summary.dirty_hotspots | Should Be 0
            @($report.hotspots).Count | Should Be 2
            $report.hotspots[0].risk_score | Should BeGreaterThan 0
            @($report.hotspots.dirty | Where-Object { $_ -eq $true }).Count | Should Be 0
            $report.hotspots[0].path | Should Be "zircon_runtime/src/ui/layout/incremental.rs"
            @($report.hotspots.path) -contains "zircon_runtime/src/ui/tests/noise.rs" |
                Should Be $false
            @($report.hotspots.path) -contains `
                "zircon_editor/src/ui/retained_host/paint_tests/paint.rs" |
                Should Be $false
        }
        finally {
            if (Test-Path -LiteralPath $fixtureRoot) {
                Remove-Item -LiteralPath $fixtureRoot -Recurse -Force
            }
        }
    }

    It "does not rank allocation-free empty Vec defaults as materializations" {
        Get-Command Export-ZirconUiStructuralHotspotAudit -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty

        $fixtureRoot = "E:\zircon-profiles\pester-ui-audit-empty-vec-$([guid]::NewGuid().ToString('N'))"
        $outputRoot = Join-Path $fixtureRoot "evidence"
        try {
            $runtimeRoot = Join-Path $fixtureRoot "zircon_runtime\src\ui"
            New-Item -ItemType Directory -Path $runtimeRoot -Force | Out-Null
            @'
struct LayoutState {
    rows: Vec<Row>,
    columns: Vec<Column>,
}

fn empty_defaults() -> (Vec<Row>, Vec<Column>, Vec<Item>) {
    (Vec::new(), Vec::new(), vec![])
}

fn materialized_rows(row: Row) -> (Vec<Row>, Vec<Row>) {
    (Vec::with_capacity(8), vec![row])
}
'@ | Set-Content -LiteralPath (Join-Path $runtimeRoot "layout_state.rs") -Encoding UTF8

            $receipt = Export-ZirconUiStructuralHotspotAudit `
                -RepoRoot $fixtureRoot `
                -OutputDirectory $outputRoot `
                -SourceRoots @("zircon_runtime/src/ui")

            $report = Get-Content -LiteralPath $receipt.json_path -Raw | ConvertFrom-Json
            $report.summary.vec_materializations | Should Be 2
            $report.hotspots[0].vec_materializations | Should Be 2
        }
        finally {
            if (Test-Path -LiteralPath $fixtureRoot) {
                Remove-Item -LiteralPath $fixtureRoot -Recurse -Force
            }
        }
    }

    It "excludes inline cfg-test signals from production hotspot counts" {
        Get-Command Export-ZirconUiStructuralHotspotAudit -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty

        $fixtureRoot = "E:\zircon-profiles\pester-ui-audit-inline-tests-$([guid]::NewGuid().ToString('N'))"
        $outputRoot = Join-Path $fixtureRoot "evidence"
        try {
            $runtimeRoot = Join-Path $fixtureRoot "zircon_runtime\src\ui"
            New-Item -ItemType Directory -Path $runtimeRoot -Force | Out-Null
            @'
fn publish(row: Row, source: &Source) -> Vec<Row> {
    consume(source.clone());
    vec![row]
}

#[cfg(test)]
mod tests {
    fn assertion_noise(source: &Source) {
        consume(source.clone());
        consume(source.clone());
        consume(vec![Row::default(), Row::default()]);
        consume("noise".to_string());
    }
}
'@ | Set-Content -LiteralPath (Join-Path $runtimeRoot "publication.rs") -Encoding UTF8

            $receipt = Export-ZirconUiStructuralHotspotAudit `
                -RepoRoot $fixtureRoot `
                -OutputDirectory $outputRoot `
                -SourceRoots @("zircon_runtime/src/ui")

            $report = Get-Content -LiteralPath $receipt.json_path -Raw | ConvertFrom-Json
            $report.summary.clone_calls | Should Be 1
            $report.summary.vec_materializations | Should Be 1
            $report.summary.string_allocations | Should Be 0
        }
        finally {
            if (Test-Path -LiteralPath $fixtureRoot) {
                Remove-Item -LiteralPath $fixtureRoot -Recurse -Force
            }
        }
    }

    It "treats a test-only source file as a zero-signal production prefix" {
        Get-Command Export-ZirconUiStructuralHotspotAudit -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty

        $fixtureRoot = "E:\zircon-profiles\pester-ui-audit-test-only-$([guid]::NewGuid().ToString('N'))"
        $outputRoot = Join-Path $fixtureRoot "evidence"
        try {
            $runtimeRoot = Join-Path $fixtureRoot "zircon_runtime\src\ui"
            New-Item -ItemType Directory -Path $runtimeRoot -Force | Out-Null
            @'
#[cfg(test)]
mod tests {
    fn assertion_noise(source: &Source) {
        consume(source.clone());
        consume(vec![Row::default()]);
    }
}
'@ | Set-Content -LiteralPath (Join-Path $runtimeRoot "test_only.rs") -Encoding UTF8

            $report = & {
                $ErrorActionPreference = "Stop"
                $receipt = Export-ZirconUiStructuralHotspotAudit `
                    -RepoRoot $fixtureRoot `
                    -OutputDirectory $outputRoot `
                    -SourceRoots @("zircon_runtime/src/ui")
                Get-Content -LiteralPath $receipt.json_path -Raw | ConvertFrom-Json
            }

            $report.summary.clone_calls | Should Be 0
            $report.summary.vec_materializations | Should Be 0
            $report.hotspots[0].line_count | Should Be 0
            $report.hotspots[0].risk_score | Should Be 0
        }
        finally {
            if (Test-Path -LiteralPath $fixtureRoot) {
                Remove-Item -LiteralPath $fixtureRoot -Recurse -Force
            }
        }
    }

    It "binds the report to a deterministic production-source manifest" {
        Get-Command Export-ZirconUiStructuralHotspotAudit -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty

        $fixtureRoot = "E:\zircon-profiles\pester-ui-audit-source-binding-$([guid]::NewGuid().ToString('N'))"
        try {
            $runtimeRoot = Join-Path $fixtureRoot "zircon_runtime\src\ui"
            $sourcePath = Join-Path $runtimeRoot "frame.rs"
            New-Item -ItemType Directory -Path $runtimeRoot -Force | Out-Null
            @'
fn publish(source: &Source) {
    consume(source.clone());
}

#[cfg(test)]
mod tests {
    fn original_test_noise() {}
}
'@ | Set-Content -LiteralPath $sourcePath -Encoding UTF8

            $first = Export-ZirconUiStructuralHotspotAudit `
                -RepoRoot $fixtureRoot `
                -OutputDirectory (Join-Path $fixtureRoot "first") `
                -SourceRoots @("zircon_runtime/src/ui")
            $firstReport = Get-Content -LiteralPath $first.json_path -Raw | ConvertFrom-Json

            @'
fn publish(source: &Source) {
    consume(source.clone());
}

#[cfg(test)]
mod tests {
    fn changed_test_noise() {
        consume(vec![1, 2, 3]);
    }
}
'@ | Set-Content -LiteralPath $sourcePath -Encoding UTF8
            $testOnlyChange = Export-ZirconUiStructuralHotspotAudit `
                -RepoRoot $fixtureRoot `
                -OutputDirectory (Join-Path $fixtureRoot "test-only-change") `
                -SourceRoots @("zircon_runtime/src/ui")
            $testOnlyReport = Get-Content -LiteralPath $testOnlyChange.json_path -Raw |
                ConvertFrom-Json

            @'
fn publish(source: &Source) {
    consume(source.clone());
    consume(source.clone());
}
'@ | Set-Content -LiteralPath $sourcePath -Encoding UTF8
            $productionChange = Export-ZirconUiStructuralHotspotAudit `
                -RepoRoot $fixtureRoot `
                -OutputDirectory (Join-Path $fixtureRoot "production-change") `
                -SourceRoots @("zircon_runtime/src/ui")
            $productionReport = Get-Content -LiteralPath $productionChange.json_path -Raw |
                ConvertFrom-Json

            $firstReport.source_binding.production_source_manifest_sha256 |
                Should Match "^[0-9A-F]{64}$"
            $testOnlyReport.source_binding.production_source_manifest_sha256 |
                Should Be $firstReport.source_binding.production_source_manifest_sha256
            $productionReport.source_binding.production_source_manifest_sha256 |
                Should Not Be $firstReport.source_binding.production_source_manifest_sha256
        }
        finally {
            if (Test-Path -LiteralPath $fixtureRoot) {
                Remove-Item -LiteralPath $fixtureRoot -Recurse -Force
            }
        }
    }

    It "rejects C drive artifact output" {
        Get-Command Export-ZirconUiStructuralHotspotAudit -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty

        $errorRecord = $null
        try {
            Export-ZirconUiStructuralHotspotAudit `
                -RepoRoot "E:\Git\ZirconEngine" `
                -OutputDirectory "C:\zircon-ui-audit-forbidden" `
                -SourceRoots @("zircon_runtime/src/ui")
        }
        catch {
            $errorRecord = $_
        }

        $errorRecord | Should Not BeNullOrEmpty
        $errorRecord.Exception.Message | Should Match "D:, E:, or F:"
    }
}
