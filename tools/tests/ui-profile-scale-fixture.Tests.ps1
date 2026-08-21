$script:RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$script:ScaleFixtureScript = Join-Path $PSScriptRoot "..\ui-profile-scale-fixture.ps1"
$script:ProfileManifestScript = Join-Path $PSScriptRoot "..\profile-capture-manifest.ps1"
if (Test-Path -LiteralPath $script:ScaleFixtureScript) {
    . $script:ScaleFixtureScript
}
if (Test-Path -LiteralPath $script:ProfileManifestScript) {
    . $script:ProfileManifestScript
}

Describe "ui profile scale fixture" {
    It "materializes an exact hierarchy scene from the canonical project template" {
        Get-Command New-ZirconUiHierarchyScaleFixture -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty

        $outputRoot = "E:\zircon-profiles\pester-scale-$([guid]::NewGuid().ToString('N'))"
        $projectRoot = Join-Path $outputRoot "ProfileCaptureProject"
        try {
            $fixture = New-ZirconUiHierarchyScaleFixture `
                -RepoRoot $script:RepoRoot `
                -ProjectRoot $projectRoot `
                -LogicalNodeCount 12

            $fixture.schema_version | Should Be 1
            $fixture.kind | Should Be "hierarchy_scene"
            $fixture.logical_node_count | Should Be 12
            $fixture.scene_entity_count | Should Be 12
            $fixture.project_root | Should Be $projectRoot
            $fixture.scene.relative_path | Should Be "assets/scenes/main.scene.toml"
            $fixture.scene.sha256 | Should Match "^[0-9a-f]{64}$"
            $fixture.scene.byte_length | Should BeGreaterThan 0
            $fixture.project_manifest.relative_path | Should Be "zircon-project.toml"
            $fixture.project_manifest.sha256 | Should Match "^[0-9a-f]{64}$"

            Test-Path -LiteralPath (Join-Path $projectRoot "zircon-project.toml") |
                Should Be $true
            $scenePath = Join-Path $projectRoot "assets\scenes\main.scene.toml"
            $sceneSource = Get-Content -LiteralPath $scenePath -Raw
            ([regex]::Matches($sceneSource, "(?m)^\[\[entities\]\]\r?$").Count) |
                Should Be 12
            $sceneSource | Should Match 'name = "Profile Hierarchy Node 000012"'
        }
        finally {
            if (Test-Path -LiteralPath $outputRoot) {
                Remove-Item -LiteralPath $outputRoot -Recurse -Force
            }
        }
    }

    It "materializes exact renderable viewport-pointer entities with explicit mobility" {
        Get-Command New-ZirconViewportPointerScaleFixture -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty

        $outputRoot = "E:\zircon-profiles\pester-viewport-pointer-$([guid]::NewGuid().ToString('N'))"
        $projectRoot = Join-Path $outputRoot "ProfileCaptureProject"
        try {
            $fixture = New-ZirconViewportPointerScaleFixture `
                -RepoRoot $script:RepoRoot `
                -ProjectRoot $projectRoot `
                -SelectableNodeCount 12 `
                -Mobility "dynamic"

            $fixture.schema_version | Should Be 1
            $fixture.kind | Should Be "viewport_pointer_scene"
            $fixture.selectable_node_count | Should Be 12
            $fixture.scene_entity_count | Should Be 14
            $fixture.mobility | Should Be "dynamic"
            $fixture.scene.relative_path | Should Be "assets/scenes/main.scene.toml"
            $fixture.scene.sha256 | Should Match "^[0-9a-f]{64}$"

            $scenePath = Join-Path $projectRoot "assets\scenes\main.scene.toml"
            $sceneSource = Get-Content -LiteralPath $scenePath -Raw
            ([regex]::Matches($sceneSource, "(?m)^\[\[entities\]\]\r?$").Count) |
                Should Be 14
            ([regex]::Matches($sceneSource, '(?m)^mobility = "Dynamic"\r?$').Count) |
                Should Be 12
            ([regex]::Matches($sceneSource, '(?m)^\[entities\.mesh\.model\]\r?$').Count) |
                Should Be 12
            $sceneSource | Should Match 'name = "Profile Viewport Node 000012"'
        }
        finally {
            if (Test-Path -LiteralPath $outputRoot) {
                Remove-Item -LiteralPath $outputRoot -Recurse -Force
            }
        }
    }

    It "fails closed for unsafe roots and invalid N" {
        foreach ($case in @(
            @{ Root = "C:\zircon-profiles\unsafe"; N = 12 },
            @{ Root = (Join-Path $script:RepoRoot "target\profile-scale"); N = 12 },
            @{ Root = "E:\zircon-profiles\invalid-zero"; N = 0 },
            @{ Root = "E:\zircon-profiles\invalid-upper-bound"; N = 100001 }
        )) {
            $didThrow = $false
            try {
                New-ZirconUiHierarchyScaleFixture `
                    -RepoRoot $script:RepoRoot `
                    -ProjectRoot $case.Root `
                    -LogicalNodeCount $case.N | Out-Null
            }
            catch {
                $didThrow = $true
            }
            $didThrow | Should Be $true
        }
    }

    It "rejects Windows device namespace aliases for the C drive before writing" {
        foreach ($unsafeRoot in @(
            "\\?\C:\zircon-profiles\unsafe-device",
            "\\.\C:\zircon-profiles\unsafe-device"
        )) {
            $didThrow = $false
            try {
                Assert-ZirconUiProfileScaleProjectRoot `
                    -RepoRoot $script:RepoRoot `
                    -ProjectRoot $unsafeRoot | Out-Null
            }
            catch {
                $didThrow = $true
            }
            $didThrow | Should Be $true
        }
    }

    It "revalidates the scene fingerprint before binding it into a capture manifest" {
        Get-Command Resolve-ZirconProfileInputFixtureEvidence -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty

        $outputRoot = "E:\zircon-profiles\pester-scale-integrity-$([guid]::NewGuid().ToString('N'))"
        $projectRoot = Join-Path $outputRoot "ProfileCaptureProject"
        try {
            $fixture = New-ZirconUiHierarchyScaleFixture `
                -RepoRoot $script:RepoRoot `
                -ProjectRoot $projectRoot `
                -LogicalNodeCount 4
            $validated = Resolve-ZirconProfileInputFixtureEvidence `
                -RepoRoot $script:RepoRoot `
                -InputFixture $fixture

            $validated.logical_node_count | Should Be 4
            $validated.scene.sha256 | Should Be $fixture.scene.sha256
            $validated.project_manifest.sha256 | Should Be $fixture.project_manifest.sha256

            $manifestBytes = [System.IO.File]::ReadAllBytes($fixture.project_manifest.path)
            try {
                Add-Content -LiteralPath $fixture.project_manifest.path -Value "# changed after fixture materialization"
                {
                    Resolve-ZirconProfileInputFixtureEvidence `
                        -RepoRoot $script:RepoRoot `
                        -InputFixture $fixture
                } | Should Throw "UI profile input fixture changed after materialization."
            }
            finally {
                [System.IO.File]::WriteAllBytes($fixture.project_manifest.path, $manifestBytes)
            }

            Add-Content -LiteralPath $fixture.scene.path -Value "# changed after fixture materialization"
            {
                Resolve-ZirconProfileInputFixtureEvidence `
                    -RepoRoot $script:RepoRoot `
                    -InputFixture $fixture
            } | Should Throw "UI profile input fixture changed after materialization."
        }
        finally {
            if (Test-Path -LiteralPath $outputRoot) {
                Remove-Item -LiteralPath $outputRoot -Recurse -Force
            }
        }
    }

    It "revalidates viewport pointer mobility and selectable count before manifest binding" {
        $outputRoot = "E:\zircon-profiles\pester-viewport-pointer-integrity-$([guid]::NewGuid().ToString('N'))"
        $projectRoot = Join-Path $outputRoot "ProfileCaptureProject"
        try {
            $fixture = New-ZirconViewportPointerScaleFixture `
                -RepoRoot $script:RepoRoot `
                -ProjectRoot $projectRoot `
                -SelectableNodeCount 4 `
                -Mobility "static"
            $validated = Resolve-ZirconProfileInputFixtureEvidence `
                -RepoRoot $script:RepoRoot `
                -InputFixture $fixture

            $validated.kind | Should Be "viewport_pointer_scene"
            $validated.selectable_node_count | Should Be 4
            $validated.scene_entity_count | Should Be 6
            $validated.mobility | Should Be "static"

            $fixture.mobility = "dynamic"
            {
                Resolve-ZirconProfileInputFixtureEvidence `
                    -RepoRoot $script:RepoRoot `
                    -InputFixture $fixture
            } | Should Throw "UI profile viewport pointer fixture mobility is inconsistent."
        }
        finally {
            if (Test-Path -LiteralPath $outputRoot) {
                Remove-Item -LiteralPath $outputRoot -Recurse -Force
            }
        }
    }

    It "materializes an exact importable JSON asset catalog set" {
        Get-Command New-ZirconUiAssetCatalogScaleFixture -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty

        $outputRoot = "E:\zircon-profiles\pester-asset-scale-$([guid]::NewGuid().ToString('N'))"
        $projectRoot = Join-Path $outputRoot "ProfileCaptureProject"
        try {
            $fixture = New-ZirconUiAssetCatalogScaleFixture `
                -RepoRoot $script:RepoRoot `
                -ProjectRoot $projectRoot `
                -AssetItemCount 12

            $fixture.schema_version | Should Be 1
            $fixture.kind | Should Be "asset_catalog_json"
            $fixture.asset_item_count | Should Be 12
            $fixture.source_extension | Should Be "json"
            $fixture.asset_sources.file_count | Should Be 12
            $fixture.asset_sources.relative_directory | Should Be "assets"
            $fixture.asset_sources.file_name_prefix | Should Be "profile_catalog_asset_"
            $fixture.asset_sources.sha256 | Should Match "^[0-9a-f]{64}$"
            $fixture.asset_sources.total_byte_length | Should BeGreaterThan 0

            $sources = @(Get-ChildItem -LiteralPath (Join-Path $projectRoot "assets") `
                    -Filter "profile_catalog_asset_*.json" -File |
                    Sort-Object Name)
            $sources.Count | Should Be 12
            $sources[0].Name | Should Be "profile_catalog_asset_000001.json"
            $sources[11].Name | Should Be "profile_catalog_asset_000012.json"
            foreach ($source in $sources) {
                { Get-Content -LiteralPath $source.FullName -Raw | ConvertFrom-Json } |
                    Should Not Throw
                (Test-Path -LiteralPath "$($source.FullName).zmeta") | Should Be $false
            }
        }
        finally {
            if (Test-Path -LiteralPath $outputRoot) {
                Remove-Item -LiteralPath $outputRoot -Recurse -Force
            }
        }
    }

    It "revalidates every generated asset source before manifest binding" {
        $outputRoot = "E:\zircon-profiles\pester-asset-scale-integrity-$([guid]::NewGuid().ToString('N'))"
        $projectRoot = Join-Path $outputRoot "ProfileCaptureProject"
        try {
            $fixture = New-ZirconUiAssetCatalogScaleFixture `
                -RepoRoot $script:RepoRoot `
                -ProjectRoot $projectRoot `
                -AssetItemCount 4
            $validated = Resolve-ZirconProfileInputFixtureEvidence `
                -RepoRoot $script:RepoRoot `
                -InputFixture $fixture

            $validated.asset_item_count | Should Be 4
            $validated.asset_sources.sha256 | Should Be $fixture.asset_sources.sha256

            $firstSource = Join-Path $projectRoot "assets\profile_catalog_asset_000001.json"
            Set-Content -LiteralPath $firstSource -Value '{"profile_asset_index":999}' -Encoding ASCII
            {
                Resolve-ZirconProfileInputFixtureEvidence `
                    -RepoRoot $script:RepoRoot `
                    -InputFixture $fixture
            } | Should Throw "UI profile input fixture asset set changed after materialization."
        }
        finally {
            if (Test-Path -LiteralPath $outputRoot) {
                Remove-Item -LiteralPath $outputRoot -Recurse -Force
            }
        }
    }

    It "caps the asset catalog fixture at the accepted 10k scale" {
        $didThrow = $false
        try {
            New-ZirconUiAssetCatalogScaleFixture `
                -RepoRoot $script:RepoRoot `
                -ProjectRoot "E:\zircon-profiles\invalid-asset-upper-bound" `
                -AssetItemCount 10001 | Out-Null
        }
        catch {
            $didThrow = $true
        }
        $didThrow | Should Be $true
        (Test-Path -LiteralPath "E:\zircon-profiles\invalid-asset-upper-bound") |
            Should Be $false
    }
}
