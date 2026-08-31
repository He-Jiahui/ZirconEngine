$script:RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$script:CaptureScript = Join-Path $script:RepoRoot "tools\capture-editor-ui-visual.ps1"
$script:Source = Get-Content -LiteralPath $script:CaptureScript -Raw
$script:SourceBindingScript = Join-Path $script:RepoRoot "tools\editor-ui-visual-source-binding.ps1"
$script:SourceBindingSource = Get-Content -LiteralPath $script:SourceBindingScript -Raw
$script:InteractionScript = Join-Path $script:RepoRoot "tools\editor-ui-visual-interactions.ps1"

Describe "editor UI native visual capture" {
    It "parses as PowerShell" {
        $tokens = $null
        $errors = $null
        [System.Management.Automation.Language.Parser]::ParseFile(
            $script:CaptureScript,
            [ref]$tokens,
            [ref]$errors) | Out-Null

        @($errors).Count | Should Be 0

        $tokens = $null
        $errors = $null
        [System.Management.Automation.Language.Parser]::ParseFile(
            $script:SourceBindingScript,
            [ref]$tokens,
            [ref]$errors) | Out-Null

        @($errors).Count | Should Be 0

        $tokens = $null
        $errors = $null
        [System.Management.Automation.Language.Parser]::ParseFile(
            $script:InteractionScript,
            [ref]$tokens,
            [ref]$errors) | Out-Null

        @($errors).Count | Should Be 0
    }

    It "requires explicit product bundle and output paths" {
        $script:Source | Should Match '\[Parameter\(Mandatory = \$true\)\]\s*\[string\]\$BundleDirectory'
        $script:Source | Should Match '\[Parameter\(Mandatory = \$true\)\]\s*\[string\]\$OutputDirectory'
        $script:Source | Should Match '\[Parameter\(Mandatory = \$true\)\][\s\S]{0,160}\$ExpectedEditorSha256'
        $script:Source | Should Match '\[Parameter\(Mandatory = \$true\)\][\s\S]{0,160}\$ExpectedRuntimeSha256'
        $script:Source | Should Match '\[Parameter\(Mandatory = \$true\)\][\s\S]{0,160}\$ExpectedSourceSha256'
        $script:Source | Should Not Match 'AllowSoftwareFallback'
        $script:Source | Should Match 'Join-Path \$PSScriptRoot ''\.\.'''
        $script:Source | Should Not Match 'Join-Path \$PSScriptRoot ''\.\.\\\.\.'''
    }

    It "rejects a product bundle that differs from the managed build receipt" {
        $fixtureRoot = Join-Path $TestDrive 'hash-mismatch'
        $bundle = Join-Path $fixtureRoot 'bundle'
        $output = Join-Path $fixtureRoot 'output'
        New-Item -ItemType Directory -Force -Path $bundle | Out-Null
        Set-Content -LiteralPath (Join-Path $bundle 'zircon_editor.exe') -Value 'wrong editor'
        Set-Content -LiteralPath (Join-Path $bundle 'zircon_runtime.dll') -Value 'wrong runtime'

        $caught = $null
        try {
            & $script:CaptureScript `
                -BundleDirectory $bundle `
                -OutputDirectory $output `
                -ExpectedEditorSha256 ('0' * 64) `
                -ExpectedRuntimeSha256 ('1' * 64) `
                -ExpectedSourceSha256 ('2' * 64) `
                -SkipVisualOracle
        }
        catch {
            $caught = $_
        }

        $caught | Should Not BeNullOrEmpty
        $caught.Exception.Message | Should Match 'managed build receipt'
        Test-Path -LiteralPath $output | Should Be $false
    }

    It "rejects product bundle assets that differ from their source fingerprints" {
        . (Join-Path $script:RepoRoot 'tools\profile-capture-manifest.ps1')
        . $script:SourceBindingScript
        $bundle = Join-Path $TestDrive 'asset-bundle'
        $asset = Join-Path $bundle 'assets\ui\editor\fixture.zui'
        New-Item -ItemType Directory -Force -Path (Split-Path $asset -Parent) | Out-Null
        [System.IO.File]::WriteAllText(
            $asset,
            '[meta]' + [Environment]::NewLine,
            [System.Text.UTF8Encoding]::new($false))
        $fingerprint = Get-ZirconProfileRequiredFileFingerprint `
            -Path $asset `
            -Description 'fixture asset'
        $sourceBinding = [pscustomobject]@{
            critical_source_files = @(
                [pscustomobject]@{
                    relative_path = 'zircon_editor/assets/ui/editor/fixture.zui'
                    sha256 = $fingerprint.sha256
                    byte_length = $fingerprint.byte_length
                }
            )
        }

        $binding = Get-ZirconEditorVisualBundleAssetBinding `
            -BundleDirectory $bundle `
            -SourceBinding $sourceBinding
        $binding.bundle_asset_file_count | Should Be 1
        $binding.bundle_asset_sha256 | Should Match '^[0-9a-f]{64}$'

        Add-Content -LiteralPath $asset -Value 'changed = true'
        $caught = $null
        try {
            Get-ZirconEditorVisualBundleAssetBinding `
                -BundleDirectory $bundle `
                -SourceBinding $sourceBinding | Out-Null
        }
        catch {
            $caught = $_
        }
        $caught | Should Not BeNullOrEmpty
        $caught.Exception.Message | Should Match 'differs from current source'
    }

    It "captures exactly one GPU process for every accepted physical extent" {
        foreach ($extent in @(
            '@{ Width = 640; Height = 520 }',
            '@{ Width = 900; Height = 620 }',
            '@{ Width = 1672; Height = 941 }'
        )) {
            $script:Source | Should Match ([regex]::Escape($extent))
        }
        $script:Source | Should Match '\$captureResults = foreach \(\$extent in @\('
        $script:Source | Should Match 'Start-ZirconEditorVisualProcess[\s\S]*ZIRCON_PROFILE_INITIAL_CLIENT_WIDTH'
        $script:Source | Should Match 'ZIRCON_PROFILE_INITIAL_CLIENT_HEIGHT'
        $script:Source | Should Not Match 'Start-Process[\s\S]{0,300}-Environment'
        $script:Source | Should Match "presenter_backend -ne 'gpu'"
        $script:Source | Should Not Match 'WindowStyle Hidden'
    }

    It "binds desktop pixels to native DPI and the presented client extent" {
        $script:Source | Should Match 'GetDpiForWindow'
        $script:Source | Should Match 'profileGeometry\.window_client_size\.width -ne \$extent\.Width'
        $script:Source | Should Match 'profileGeometry\.window_client_size\.height -ne \$extent\.Height'
        $script:Source | Should Match 'Save-ZirconEditorVisualClientScreenshot'
        $script:Source | Should Match 'CopyFromScreen'
        $script:Source | Should Match 'Captured image is blank or low-information'
        $script:Source | Should Match 'sha256 = Get-ZirconProfileFileSha256 -Path \$Path'
        $script:Source | Should Match 'profile_geometry_sha256 = Get-ZirconProfileFileSha256'
    }

    It "publishes a stable manifest before running the pixel oracle" {
        $manifestWrite = $script:Source.IndexOf("'capture-manifest.json'")
        $oracleRun = $script:Source.IndexOf("'tools\zircon_editor_ui_visual_oracle.py'")

        $manifestWrite | Should BeGreaterThan -1
        $oracleRun | Should BeGreaterThan $manifestWrite
        $script:Source | Should Match '\$manifestJson = \$manifest \| ConvertTo-Json -Depth 8'
        $script:Source | Should Match '--capture-manifest \$manifestPath'
        $script:Source | Should Match '--output-directory \(Join-Path \$OutputDirectory ''visual-oracle''\)'
    }

    It "binds the capture manifest to repository sources and verified binaries" {
        $combinedSource = $script:Source + $script:SourceBindingSource
        $combinedSource | Should Match 'profile-capture-manifest\.ps1'
        $combinedSource | Should Match 'Get-ZirconProfileGitMetadata'
        $combinedSource | Should Match 'Get-ZirconProfileCriticalSourcePaths'
        $combinedSource | Should Match 'Get-ZirconProfileCaptureToolPaths'
        $script:SourceBindingSource | Should Match 'tools/capture-editor-ui-visual\.ps1'
        $script:SourceBindingSource | Should Match 'tools/editor-ui-visual-interactions\.ps1'
        $script:SourceBindingSource | Should Match 'tools/editor-ui-visual-source-binding\.ps1'
        $script:SourceBindingSource | Should Match 'tools/zircon_editor_ui_visual_oracle\.py'
        $script:SourceBindingSource | Should Match 'zircon_editor\\assets'
        $script:SourceBindingSource | Should Match 'zircon_runtime\\assets'
        $combinedSource | Should Match 'Get-ZirconEditorVisualBundleAssetBinding'
        $combinedSource | Should Match 'bundle_asset_sha256'
        $combinedSource | Should Match 'bundle_asset_file_count'
        $combinedSource | Should Match 'Get-ZirconEditorVisualSourceBinding'
        $script:Source | Should Match 'source_sha256'
        $script:Source | Should Match 'critical_source_files'
        $script:Source | Should Match 'expected_sha256'
        $script:Source | Should Match 'actual_sha256'
        $script:Source | Should Match 'repository\s*=\s*\[pscustomobject\]'
        $script:Source | Should Match 'binaries\s*=\s*\[pscustomobject\]'
    }

    It "uses source-bound native pointer input for the regular module Details state" {
        (Test-Path -LiteralPath $script:InteractionScript -PathType Leaf) | Should Be $true
        if (Test-Path -LiteralPath $script:InteractionScript -PathType Leaf) {
            $interactionSource = Get-Content -LiteralPath $script:InteractionScript -Raw
            $interactionSource | Should Match 'Get-ZirconEditorVisualProfileControlCenter'
            $interactionSource | Should Match 'Invoke-ZirconEditorVisualPointerMove'
            $interactionSource | Should Match 'Invoke-ZirconEditorVisualControlHover'
            $interactionSource | Should Match 'Invoke-ZirconEditorVisualControlClick'
            $interactionSource | Should Match '0x0200'
            $interactionSource | Should Match '0x0201'
            $interactionSource | Should Match '0x0202'
            $interactionSource | Should Match 'Measure-ZirconEditorVisualRegionDifference'
            $interactionSource | Should Match '\$RegionRight'
            $interactionSource | Should Match '\$RegionBottom'
            $interactionSource | Should Not Match '\[double\]::IsFinite'
            $interactionSource | Should Not Match '\[Math\]::Clamp'
        }

        $script:Source | Should Match 'editor-ui-visual-interactions\.ps1'
        $script:Source | Should Match "'WorkbenchToolbarMenu'"
        $script:Source | Should Match "'editor-900x620-main-menu\.png'"
        $script:Source | Should Match "'editor-900x620-main-menu-dismissed\.png'"
        $script:Source | Should Match 'main_menu_interaction\s*=\s*\$mainMenuInteraction'
        $script:Source | Should Match '\[switch\]\$PreservePointerPosition'
        $script:Source | Should Match "'editor-900x620-module-details-tooltip\.png'"
        $script:Source | Should Match "'editor-900x620-module-details-tooltip-dismissed\.png'"
        $script:Source | Should Match 'module_details_tooltip_interaction\s*=\s*\$moduleDetailsTooltipInteraction'
        $script:Source | Should Match "'WorkbenchModuleDetailsDrawerToggle'"
        $script:Source | Should Match "'editor-900x620-module-details\.png'"
        $script:Source | Should Match 'module_details_interaction\s*=\s*\$moduleDetailsInteraction'
        $script:Source | Should Match "source_geometry_scope\s*=\s*'pre_interaction_trigger_only'"
        $script:Source | Should Match '\$extent\.Width -eq 900'
        $script:Source | Should Match '\$extent\.Height -eq 620'
    }

    It "resolves a control center and measures a material right-region visual change" {
        if (-not (Test-Path -LiteralPath $script:InteractionScript -PathType Leaf)) {
            return
        }
        . $script:InteractionScript
        $profile = [pscustomobject]@{
            template_controls = @(
                [pscustomobject]@{
                    id = 'WorkbenchModuleDetailsDrawerToggle'
                    frame = [pscustomobject]@{ x = 840.0; y = 12.0; width = 32.0; height = 32.0 }
                }
            )
            viewport_toolbar_controls = @()
        }
        $center = Get-ZirconEditorVisualProfileControlCenter `
            -ProfileGeometry $profile `
            -ControlId 'WorkbenchModuleDetailsDrawerToggle'
        $center.X | Should Be 856
        $center.Y | Should Be 28

        Add-Type -AssemblyName System.Drawing
        $beforePath = Join-Path $TestDrive 'before.png'
        $afterPath = Join-Path $TestDrive 'after.png'
        $before = [System.Drawing.Bitmap]::new(32, 24)
        $after = [System.Drawing.Bitmap]::new(32, 24)
        try {
            for ($y = 0; $y -lt 24; $y += 1) {
                for ($x = 0; $x -lt 32; $x += 1) {
                    $before.SetPixel($x, $y, [System.Drawing.Color]::Black)
                    $after.SetPixel(
                        $x,
                        $y,
                        $(if ($x -ge 20) { [System.Drawing.Color]::White } else { [System.Drawing.Color]::Black }))
                }
            }
            $before.Save($beforePath, [System.Drawing.Imaging.ImageFormat]::Png)
            $after.Save($afterPath, [System.Drawing.Imaging.ImageFormat]::Png)
        }
        finally {
            $before.Dispose()
            $after.Dispose()
        }

        $difference = Measure-ZirconEditorVisualRegionDifference `
            -BeforePath $beforePath `
            -AfterPath $afterPath `
            -RegionLeft 20 `
            -RegionTop 0 `
            -RegionRight 28 `
            -RegionBottom 12 `
            -Stride 1
        $difference.region_right | Should Be 28
        $difference.region_bottom | Should Be 12
        $difference.sampled_pixels | Should Be 96
        $difference.different_pixels | Should Be 96
        $difference.different_pixel_ratio | Should Be 1.0
    }
}
