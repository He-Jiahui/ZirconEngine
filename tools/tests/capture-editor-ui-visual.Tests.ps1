# 编辑器视觉采集门禁混合源码契约与隔离夹具；夹具实际调用产物指纹拒绝和区域变化测量，完整编辑器截图仍需由产品采集流程验证。
[CmdletBinding()]
param(
    [string]$RepositoryRoot
)
$script:RequestedRepositoryRoot = $RepositoryRoot
$script:TestScriptPath = $MyInvocation.MyCommand.Path

function Resolve-ZirconEditorVisualTestRepositoryRoot {
    param(
        [string]$ExplicitRoot,
        [string]$TestScriptRoot,
        [string]$TestScriptPath
    )

    $candidateRoot = $ExplicitRoot
    if ([string]::IsNullOrWhiteSpace($candidateRoot)) {
        if (-not [string]::IsNullOrWhiteSpace($TestScriptRoot)) {
            $candidateRoot = Join-Path $TestScriptRoot '..\..'
        }
        elseif (-not [string]::IsNullOrWhiteSpace($TestScriptPath)) {
            $testScriptRoot = Split-Path -Parent $TestScriptPath
            $candidateRoot = Join-Path $testScriptRoot '..\..'
        }
        else {
            throw 'Pass -RepositoryRoot when this PowerShell host does not expose the test script path.'
        }
    }

    if (-not [System.IO.Directory]::Exists($candidateRoot)) {
        throw "RepositoryRoot does not exist as a directory: $candidateRoot"
    }
    $resolvedRoot = (Resolve-Path -LiteralPath $candidateRoot -ErrorAction Stop).Path
    foreach ($requiredRepositoryFile in @(
            (Join-Path $resolvedRoot 'Cargo.toml'),
            (Join-Path $resolvedRoot 'tools\analysis\visual\capture-editor-ui-visual.ps1')
        )) {
        if (-not [System.IO.File]::Exists($requiredRepositoryFile)) {
            throw "RepositoryRoot does not identify a ZirconEngine checkout: $resolvedRoot"
        }
    }
    return $resolvedRoot
}
$script:RepoRoot = Resolve-ZirconEditorVisualTestRepositoryRoot -ExplicitRoot $RepositoryRoot -TestScriptRoot $PSScriptRoot -TestScriptPath $script:TestScriptPath
$script:CaptureScript = Join-Path $script:RepoRoot "tools\analysis\visual\capture-editor-ui-visual.ps1"
$script:Source = Get-Content -LiteralPath $script:CaptureScript -Raw
$script:SourceBindingScript = Join-Path $script:RepoRoot "tools\analysis\visual\editor-ui-visual-source-binding.ps1"
$script:SourceBindingSource = Get-Content -LiteralPath $script:SourceBindingScript -Raw
$script:InteractionScript = Join-Path $script:RepoRoot "tools\analysis\visual\editor-ui-visual-interactions.ps1"
$script:MatrixScript = Join-Path $script:RepoRoot "tools\analysis\visual\editor-ui-visual-matrix.ps1"
. $script:MatrixScript

Describe "editor UI native visual capture" {
    It "uses the supplied repository root and validates both portable fallbacks" {
        $script:RepoRoot | Should Be (Resolve-Path -LiteralPath $script:RequestedRepositoryRoot).Path
        (Resolve-ZirconEditorVisualTestRepositoryRoot -TestScriptRoot (Join-Path $script:RepoRoot 'tools\tests')) | Should Be $script:RepoRoot
        (Resolve-ZirconEditorVisualTestRepositoryRoot -TestScriptPath $script:TestScriptPath) | Should Be $script:RepoRoot
        (Test-Path -LiteralPath (Join-Path $script:RepoRoot 'Cargo.toml') -PathType Leaf) | Should Be $true
        (Test-Path -LiteralPath $script:CaptureScript -PathType Leaf) | Should Be $true
    }

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
        . (Join-Path $script:RepoRoot 'tools\analysis\profiling\shared\profile-capture-manifest.ps1')
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

    It "captures all three logical extents at a real 100 percent display scale" {
        $cases = @(Get-ZirconEditorVisualCaptureCases -DpiProfile '100')
        $cases.Count | Should Be 3
        ($cases.CaseId -join ',') | Should Be '1280x800,900x620,640x520'
        ($cases | ForEach-Object { '{0}x{1}' -f $_.Width, $_.Height }) -join ',' | Should Be '1280x800,900x620,640x520'
        foreach ($case in $cases) {
            $case.Width | Should Be $case.LogicalWidth
            $case.Height | Should Be $case.LogicalHeight
            $case.ExpectedWindowDpi | Should Be 96
            $case.ExpectedWinitScaleFactor | Should Be 1.0
        }
    }

    It "keeps the 150 percent logical extent and native physical extent distinct" {
        $cases = @(Get-ZirconEditorVisualCaptureCases -DpiProfile '150')
        $cases.Count | Should Be 1
        $cases[0].CaseId | Should Be '1280x800-dpi150'
        $cases[0].LogicalWidth | Should Be 1280
        $cases[0].LogicalHeight | Should Be 800
        $cases[0].Width | Should Be 1920
        $cases[0].Height | Should Be 1200
        $cases[0].ExpectedWindowDpi | Should Be 144
        $cases[0].ExpectedWinitScaleFactor | Should Be 1.5
    }

    It "rejects unsupported display profiles before a product capture" {
        $caught = $null
        try { Get-ZirconEditorVisualCaptureCases -DpiProfile '200' | Out-Null }
        catch { $caught = $_ }
        $caught | Should Not BeNullOrEmpty
        $caught.Exception.Message | Should Match '100|150'
    }

    It "captures exactly one GPU process for every case in the selected DPI profile" {
        $script:Source | Should Match 'editor-ui-visual-matrix\.ps1'
        $script:Source | Should Match 'Get-ZirconEditorVisualCaptureCases -DpiProfile \$DpiProfile'
        $script:Source | Should Match '\$captureResults = foreach \(\$extent in \$captureCases\)'
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
        $oracleRun = $script:Source.IndexOf("'tools\analysis\visual\zircon_editor_ui_visual_oracle.py'")

        $manifestWrite | Should BeGreaterThan -1
        $oracleRun | Should BeGreaterThan $manifestWrite
        $script:Source | Should Match '\$manifestJson = \$manifest \| ConvertTo-Json -Depth 8'
        $script:Source.Contains("'--capture-manifest',") | Should Be $true
        $script:Source.Contains('$caseManifestPath') | Should Be $true
        $script:Source.Contains("'--output-directory',") | Should Be $true
        $script:Source.Contains('(Join-Path $caseOutput ''analysis'')') | Should Be $true
        $script:Source | Should Match '& python @oracleArguments'
    }

    It "binds the capture manifest to repository sources and verified binaries" {
        $combinedSource = $script:Source + $script:SourceBindingSource
        $combinedSource | Should Match 'profile-capture-manifest\.ps1'
        $combinedSource | Should Match 'Get-ZirconProfileGitMetadata'
        $combinedSource | Should Match 'Get-ZirconProfileCriticalSourcePaths'
        $combinedSource | Should Match 'Get-ZirconProfileCaptureToolPaths'
        $script:SourceBindingSource | Should Match 'tools/capture-editor-ui-visual\.ps1'
        $script:SourceBindingSource | Should Match 'tools/editor-ui-visual-interactions\.ps1'
        $script:SourceBindingSource | Should Match 'tools/editor-ui-visual-matrix\.ps1'
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

    It "includes the DPI module bytes in the actual visual source binding" {
        . (Join-Path $script:RepoRoot 'tools\analysis\profiling\shared\profile-capture-manifest.ps1')
        . $script:SourceBindingScript
        # Limit unrelated repository inputs; exercise the production binding and file hashing.
        Mock Get-ZirconProfileCriticalSourcePaths { @() }
        Mock Get-ZirconProfileCaptureToolPaths { @() }
        Mock Get-ZirconProfileGitMetadata { [pscustomobject]@{ revision = 'dpi-binding-fixture' } }
        $fixtureRoot = Join-Path $TestDrive 'dpi-source-binding'
        foreach ($directory in @('tools', 'zircon_editor\assets', 'zircon_runtime\assets')) {
            New-Item -ItemType Directory -Force -Path (Join-Path $fixtureRoot $directory) | Out-Null
        }
        foreach ($path in @(
                'tools/analysis/visual/capture-editor-ui-visual.ps1',
                'tools/analysis/visual/editor-ui-visual-interactions.ps1',
                'tools/analysis/visual/editor-ui-visual-matrix.ps1',
                'tools/analysis/visual/editor-ui-visual-source-binding.ps1',
                'tools/analysis/visual/zircon_editor_ui_visual_oracle.py')) {
            Copy-Item -LiteralPath (Join-Path $script:RepoRoot $path) -Destination (Join-Path $fixtureRoot $path)
        }
        $before = Get-ZirconEditorVisualSourceBinding -RepositoryRoot $fixtureRoot
        $matrixBefore = @($before.critical_source_files | Where-Object { $_.relative_path -eq 'tools/analysis/visual/editor-ui-visual-matrix.ps1' })
        $matrixBefore.Count | Should Be 1
        Add-Content -LiteralPath (Join-Path $fixtureRoot 'tools/analysis/visual/editor-ui-visual-matrix.ps1') -Value '# DPI source fingerprint regression probe'
        $after = Get-ZirconEditorVisualSourceBinding -RepositoryRoot $fixtureRoot
        $matrixAfter = @($after.critical_source_files | Where-Object { $_.relative_path -eq 'tools/analysis/visual/editor-ui-visual-matrix.ps1' })
        $matrixAfter.Count | Should Be 1
        $matrixAfter[0].sha256 | Should Not Be $matrixBefore[0].sha256
        $after.source_sha256 | Should Not Be $before.source_sha256
    }

    It "routes an optional normal project and proves its source stayed unchanged" {
        $script:Source | Should Match '\[string\]\$ProjectRoot'
        $script:Source | Should Match 'Get-ZirconEditorVisualProjectSnapshot -Root \$ProjectRoot'
        $script:Source.Contains('default_scene\s*=') | Should Be $true
        $script:Source.Contains('asset_roots\s*=') | Should Be $true
        $script:Source | Should Match "'--project'"
        $script:Source | Should Match 'ArgumentList = \$ArgumentList'
        $script:Source | Should Match 'project_manifest_sha256_before'
        $script:Source | Should Match 'default_scene_sha256_after'
        $script:Source | Should Match 'source_sha256_before'
        $script:Source | Should Match 'source_sha256_after'
        $script:Source | Should Match 'unchanged_before_after'
        $script:Source | Should Match 'input_project = \$projectProvenance'
    }

    It "pins runtime and asset lookup to the verified product bundle" {
        $script:Source | Should Match 'runtimeManifestPath = "\$runtime\.manifest\.json"'
        $script:Source | Should Match 'declared_build_set_id'
        $script:Source | Should Match 'app_preflight_authenticated = \$false'
        $script:Source | Should Match 'ZIRCON_RUNTIME_LIBRARY = \$runtime'
        $script:Source | Should Match 'ZIRCON_ASSET_ROOT = \$bundleAssetRoot'
        $script:Source | Should Match 'zircon_runtime_library = \$runtime'
        $script:Source | Should Match 'zircon_asset_root = \$bundleAssetRoot'
        $script:Source | Should Match 'GetEnvironmentVariable\(''TMP'''
        $script:Source | Should Match 'GetEnvironmentVariable\(''TEMP'''
    }

    It "can capture an untouched initial window without moving the pointer" {
        $script:Source | Should Match '\[switch\]\$SkipInteractions'
        $script:Source | Should Match '\[switch\]\$PreservePointerPosition'
        $script:Source | Should Match '\$extent\.CaseId -eq ''900x620'' -and -not \$SkipInteractions'
        $script:Source | Should Match '\$preservePointerForCapture = \$PreservePointerPosition -or \$SkipInteractions'
        $script:Source | Should Match '-PreservePointerPosition:\$preservePointerForCapture'
    }

    It "compares artifact timestamps only with compiled Rust and Cargo inputs" {
        $script:Source | Should Match 'relative_path -match'
        $script:Source.Contains('Cargo\.toml') | Should Be $true
        $script:Source.Contains('Cargo\.lock') | Should Be $true
        $script:Source.Contains('\.rs$') | Should Be $true
        $script:Source | Should Match 'current compiled Rust/Cargo source'
        $script:Source | Should Match 'Get-ZirconEditorVisualBundleAssetBinding'
        $script:Source | Should Match 'ExpectedSourceSha256'
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
        $script:Source.Contains('$extent.CaseId -eq ''900x620''') | Should Be $true
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
