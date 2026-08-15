$scaleFixtureScript = Join-Path $PSScriptRoot "ui-profile-scale-fixture.ps1"
if (Test-Path -LiteralPath $scaleFixtureScript -PathType Leaf) {
    . $scaleFixtureScript
}

function Get-ZirconProfileFileFingerprint {
    param([string]$Path)

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        return $null
    }

    $item = Get-Item -LiteralPath $Path
    $hash = Get-FileHash -LiteralPath $Path -Algorithm SHA256
    return [pscustomobject]@{
        path = $item.FullName
        sha256 = $hash.Hash.ToLowerInvariant()
        byte_length = [int64]$item.Length
        last_write_utc = $item.LastWriteTimeUtc.ToString("o")
    }
}

function Get-ZirconProfileRequiredFileFingerprint {
    param(
        [string]$Path,
        [string]$Description
    )

    $fingerprint = Get-ZirconProfileFileFingerprint -Path $Path
    if ($null -eq $fingerprint) {
        throw "Source-bound profile capture requires ${Description}: $Path"
    }
    return $fingerprint
}

function Get-ZirconProfileCriticalSourcePaths {
    return @(
        "zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute.rs",
        "zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute_viewport.rs",
        "zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/presentation.rs",
        "zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/invalidation/decision.rs",
        "zircon_editor/src/ui/retained_host/app/profiling/snapshot_merge.rs",
        "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events/resize.rs",
        "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/input_outcome.rs",
        "zircon_editor/src/ui/retained_host/host_contract/window/event_loop.rs",
        "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/lifecycle.rs",
        "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/lifecycle/presenter.rs",
        "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/redraw.rs",
        "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/redraw/present.rs",
        "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/platform_input.rs",
        "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/profile_capture.rs",
        "zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts.rs",
        "zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/environment.rs",
        "zircon_editor/src/ui/retained_host/host_contract/window/event_wake.rs",
        "zircon_editor/src/ui/retained_host/host_contract/presenter/factory.rs",
        "zircon_editor/src/ui/retained_host/host_contract/presenter/runtime_factory.rs",
        "zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/lifecycle.rs",
        "zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/present.rs",
        "zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/stats.rs",
        "zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/present.rs",
        "zircon_editor/src/ui/retained_host/viewport/presenter_factory.rs",
        "zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/runtime_draw_list.rs",
        "zircon_editor/src/ui/retained_host/host_contract/native_pointer/move_dispatch/entry/body.rs",
        "zircon_editor/src/ui/retained_host/host_contract/native_pointer/scroll_dispatch/entry.rs",
        "zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/index.rs",
        "zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/pane_index.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/loading/cache.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/loading/pixels.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/svg/cache.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/mui_icons/rendering.rs",
        "zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/icon_atlas.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout/cache.rs",
        "zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry.rs",
        "zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/pane_profile_controls.rs",
        "zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/pane_frames/pane.rs",
        "zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/schema/geometry.rs",
        "zircon_editor/src/ui/retained_host/app/viewport/toolbar_pointer/click.rs",
        "zircon_editor/src/ui/retained_host/callback_dispatch/shared_pointer/viewport_toolbar.rs",
        "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/viewport_toolbar/bridge.rs",
        "zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/handle_click.rs",
        "zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/new.rs",
        "zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/rebuild_surface.rs",
        "zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/sync.rs",
        "zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/sync_surface_frame.rs",
        "zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/viewport_toolbar_pointer_bridge.rs",
        "zircon_editor/src/ui/retained_host/asset_pointer/common.rs",
        "zircon_editor/src/ui/retained_host/asset_pointer/content/bridge.rs",
        "zircon_editor/src/ui/retained_host/asset_pointer/reference/bridge.rs",
        "zircon_editor/src/ui/retained_host/asset_pointer/tree/bridge.rs",
        "zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge.rs",
        "zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_dispatch_event.rs",
        "zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_handle_scroll.rs",
        "zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_popup_items.rs",
        "zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_project_route.rs",
        "zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_rebuild_surface.rs",
        "zircon_editor/src/ui/retained_host/menu_pointer/popup_layout.rs",
        "zircon_editor/src/ui/retained_host/menu_pointer/register_handled_pointer_node.rs",
        "zircon_editor/src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge.rs",
        "zircon_editor/src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge_handle_click.rs",
        "zircon_editor/src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge_handle_move.rs",
        "zircon_editor/src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge_handle_scroll.rs",
        "zircon_editor/src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge_project_route.rs",
        "zircon_editor/src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge_rebuild_surface.rs",
        "zircon_editor/src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge_sync.rs",
        "zircon_editor/src/ui/retained_host/welcome_recent_pointer/register_handled_pointer_node.rs",
        "zircon_editor/src/ui/retained_host/hierarchy_pointer/handle_click.rs",
        "zircon_editor/src/ui/retained_host/hierarchy_pointer/handle_move.rs",
        "zircon_editor/src/ui/retained_host/hierarchy_pointer/handle_scroll.rs",
        "zircon_editor/src/ui/retained_host/hierarchy_pointer/register_handled_pointer_node.rs",
        "zircon_editor/src/ui/retained_host/hierarchy_pointer/rebuild_surface.rs",
        "zircon_editor/src/ui/retained_host/hierarchy_pointer/route_at_point.rs",
        "zircon_editor/src/ui/retained_host/hierarchy_pointer/sync.rs",
        "zircon_editor/src/ui/retained_host/shell_pointer/bridge.rs",
        "zircon_editor/src/ui/retained_host/shell_pointer/common.rs",
        "zircon_editor/src/ui/retained_host/shell_pointer/drag_frames.rs",
        "zircon_editor/src/ui/retained_host/shell_pointer/drag_surface.rs",
        "zircon_editor/src/ui/retained_host/shell_pointer/node_ids.rs",
        "zircon_editor/src/ui/retained_host/app/assets/refresh.rs",
        "zircon_editor/src/ui/retained_host/ui/apply_presentation.rs",
        "zircon_editor/src/ui/retained_host/ui_perf.rs",
        "zircon_editor/src/ui/retained_host/ui_perf/counter_batch.rs",
        "zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs",
        "zircon_runtime/src/ui/surface/surface/event_routing.rs",
        "zircon_runtime/src/ui/surface/surface/rebuild.rs",
        "zircon_runtime/src/ui/tree/node/scroll.rs",
        "zircon_runtime/src/ui/layout/pass/incremental.rs",
        "zircon_runtime/src/core/runtime/diagnostics/profiling/mod.rs",
        "zircon_runtime/src/core/runtime/diagnostics/profiling/recorder.rs",
        "zircon_runtime/src/core/runtime/diagnostics/profiling/export.rs",
        "zircon_runtime/src/core/runtime/diagnostics/profiling/ui_hotspot.rs",
        "zircon_runtime_interface/src/profiling.rs",
        "zircon_runtime/src/text/ui_style.rs",
        "zircon_runtime/crates/zr_rhi/src/ui_surface.rs",
        "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface.rs",
        "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/presentation.rs",
        "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/retained_cache.rs",
        "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/batching.rs",
        "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/image_cache.rs",
        "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/image_cache/resource.rs",
        "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/shared_image_registry.rs"
    )
}

function Get-ZirconProfileCaptureToolPaths {
    return @(
        "tools/ui-profile-capture.ps1",
        "tools/ui-profile-latency-evidence.ps1",
        "tools/ui-profile-process-evidence.ps1",
        "tools/ui-profile-native-resize.ps1",
        "tools/ui-profile-scale-fixture.ps1",
        "tools/profile-capture-paths.ps1",
        "tools/profile-capture-manifest.ps1"
    )
}

function Get-ZirconProfileGitMetadata {
    param(
        [string]$RepoRoot,
        [string]$GitExecutable = "git.exe"
    )

    $git = Get-Command $GitExecutable -ErrorAction SilentlyContinue
    if ($null -eq $git) {
        throw "Source-bound profile capture requires git.exe to record repository metadata."
    }

    $revisionLines = @(& $git.Source -C $RepoRoot rev-parse HEAD 2>$null)
    if ($LASTEXITCODE -ne 0 -or $revisionLines.Count -ne 1 -or [string]::IsNullOrWhiteSpace($revisionLines[0])) {
        throw "Source-bound profile capture requires a readable Git revision for: $RepoRoot"
    }
    $revision = $revisionLines[0].Trim()
    $dirtyEntries = @(& $git.Source -C $RepoRoot status --porcelain=v1 2>$null)
    if ($LASTEXITCODE -ne 0) {
        throw "Source-bound profile capture requires readable Git working-tree status for: $RepoRoot"
    }

    $statusBytes = [System.Text.Encoding]::UTF8.GetBytes(($dirtyEntries -join "`n"))
    $sha256 = [System.Security.Cryptography.SHA256]::Create()
    try {
        $dirtyTreeSha256 = ([System.BitConverter]::ToString($sha256.ComputeHash($statusBytes))).Replace("-", "").ToLowerInvariant()
    }
    finally {
        $sha256.Dispose()
    }

    return [pscustomobject]@{
        revision = $revision
        dirty = $dirtyEntries.Count -gt 0
        dirty_entry_count = $dirtyEntries.Count
        dirty_tree_sha256 = $dirtyTreeSha256
    }
}

function Resolve-ZirconProfileInputFixtureFileEvidence {
    param(
        [string]$ProjectRoot,
        [object]$Evidence,
        [string]$ExpectedRelativePath,
        [string]$Description
    )

    if ($null -eq $Evidence) {
        throw "UI profile input fixture $Description evidence is missing."
    }
    foreach ($field in @("relative_path", "path", "sha256", "byte_length")) {
        if ($null -eq $Evidence.PSObject.Properties[$field]) {
            throw "UI profile input fixture $Description is missing required field '$field'."
        }
    }
    $relativePath = [string]$Evidence.relative_path
    $path = [System.IO.Path]::GetFullPath([string]$Evidence.path)
    $expectedPath = [System.IO.Path]::GetFullPath((Join-Path $ProjectRoot $ExpectedRelativePath))
    if ($relativePath.Replace("\", "/") -ne $ExpectedRelativePath -or
        -not $path.Equals($expectedPath, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "UI profile input fixture $Description is outside its declared project."
    }

    $fingerprint = Get-ZirconProfileRequiredFileFingerprint `
        -Path $path `
        -Description "UI profile input fixture $Description"
    if ($fingerprint.sha256 -ne [string]$Evidence.sha256 -or
        $fingerprint.byte_length -ne [int64]$Evidence.byte_length) {
        throw "UI profile input fixture changed after materialization."
    }
    return [pscustomobject]@{
        relative_path = $ExpectedRelativePath
        path = $fingerprint.path
        sha256 = $fingerprint.sha256
        byte_length = $fingerprint.byte_length
        last_write_utc = $fingerprint.last_write_utc
    }
}

function Resolve-ZirconProfileInputFixtureEvidence {
    param(
        [string]$RepoRoot,
        [object]$InputFixture
    )

    if ($null -eq $InputFixture) {
        return $null
    }
    foreach ($field in @(
            "schema_version",
            "kind",
            "project_root",
            "template_relative_path",
            "project_manifest",
            "scene"
        )) {
        if ($null -eq $InputFixture.PSObject.Properties[$field]) {
            throw "UI profile input fixture is missing required field '$field'."
        }
    }
    $kind = [string]$InputFixture.kind
    if ([int]$InputFixture.schema_version -ne 1 -or
        $kind -notin @("hierarchy_scene", "asset_catalog_json") -or
        [string]$InputFixture.template_relative_path -ne "templates/projects/renderable-empty") {
        throw "UI profile input fixture schema or kind is unsupported."
    }

    $declaredProjectRoot = [string]$InputFixture.project_root
    if (-not [System.IO.Path]::IsPathRooted($declaredProjectRoot)) {
        throw "UI profile input fixture project root is not an allowed external path."
    }
    $projectRoot = [System.IO.Path]::GetFullPath($declaredProjectRoot).TrimEnd('\')
    $repo = [System.IO.Path]::GetFullPath($RepoRoot).TrimEnd('\')
    $repoPrefix = $repo + [System.IO.Path]::DirectorySeparatorChar
    $projectDriveRoot = [System.IO.Path]::GetPathRoot($projectRoot).Replace('/', '\')
    $isSystemDrive = $projectDriveRoot -match '^(?:[Cc]:\\|\\\\\?\\[Cc]:\\|\\\\\.\\[Cc]:\\)$'
    if ($projectRoot.Equals($repo, [System.StringComparison]::OrdinalIgnoreCase) -or
        $projectRoot.StartsWith($repoPrefix, [System.StringComparison]::OrdinalIgnoreCase) -or
        $isSystemDrive) {
        throw "UI profile input fixture project root is not an allowed external path."
    }

    $projectManifest = Resolve-ZirconProfileInputFixtureFileEvidence `
        -ProjectRoot $projectRoot `
        -Evidence $InputFixture.project_manifest `
        -ExpectedRelativePath "zircon-project.toml" `
        -Description "project manifest"
    $scene = Resolve-ZirconProfileInputFixtureFileEvidence `
        -ProjectRoot $projectRoot `
        -Evidence $InputFixture.scene `
        -ExpectedRelativePath "assets/scenes/main.scene.toml" `
        -Description "scene"

    if ($kind -eq "asset_catalog_json") {
        foreach ($field in @("asset_item_count", "source_extension", "asset_sources")) {
            if ($null -eq $InputFixture.PSObject.Properties[$field]) {
                throw "UI profile input fixture is missing required field '$field'."
            }
        }
        $assetItemCount = [int64]$InputFixture.asset_item_count
        if ($assetItemCount -lt 1 -or $assetItemCount -gt 10000 -or
            [string]$InputFixture.source_extension -ne "json") {
            throw "UI profile asset catalog fixture count or source type is unsupported."
        }
        foreach ($field in @(
                "relative_directory",
                "file_name_prefix",
                "extension",
                "file_count",
                "total_byte_length",
                "sha256"
            )) {
            if ($null -eq $InputFixture.asset_sources.PSObject.Properties[$field]) {
                throw "UI profile input fixture asset set is missing required field '$field'."
            }
        }
        try {
            $assetSources = Get-ZirconUiAssetCatalogScaleSetFingerprint `
                -ProjectRoot $projectRoot `
                -ExpectedCount ([int]$assetItemCount)
        }
        catch {
            throw "UI profile input fixture asset set changed after materialization."
        }
        if ([string]$InputFixture.asset_sources.relative_directory -ne $assetSources.relative_directory -or
            [string]$InputFixture.asset_sources.file_name_prefix -ne $assetSources.file_name_prefix -or
            [string]$InputFixture.asset_sources.extension -ne $assetSources.extension -or
            [int64]$InputFixture.asset_sources.file_count -ne $assetSources.file_count -or
            [int64]$InputFixture.asset_sources.total_byte_length -ne $assetSources.total_byte_length -or
            [string]$InputFixture.asset_sources.sha256 -ne $assetSources.sha256) {
            throw "UI profile input fixture asset set changed after materialization."
        }

        return [pscustomobject]@{
            schema_version = 1
            kind = "asset_catalog_json"
            project_root = $projectRoot
            template_relative_path = [string]$InputFixture.template_relative_path
            asset_item_count = $assetItemCount
            source_extension = "json"
            project_manifest = $projectManifest
            scene = $scene
            asset_sources = $assetSources
        }
    }

    foreach ($field in @("logical_node_count", "scene_entity_count")) {
        if ($null -eq $InputFixture.PSObject.Properties[$field]) {
            throw "UI profile input fixture is missing required field '$field'."
        }
    }
    $logicalNodeCount = [int64]$InputFixture.logical_node_count
    $sceneEntityCount = [int64]$InputFixture.scene_entity_count
    if ($logicalNodeCount -lt 1 -or $logicalNodeCount -gt 100000 -or
        $sceneEntityCount -ne $logicalNodeCount) {
        throw "UI profile input fixture N and scene entity counts are inconsistent."
    }

    return [pscustomobject]@{
        schema_version = 1
        kind = "hierarchy_scene"
        project_root = $projectRoot
        template_relative_path = [string]$InputFixture.template_relative_path
        logical_node_count = $logicalNodeCount
        scene_entity_count = $sceneEntityCount
        project_manifest = $projectManifest
        scene = $scene
    }
}

function Export-ZirconProfileCaptureManifest {
    param(
        [string]$ProfileDir,
        [string]$RepoRoot,
        [string]$OutputRoot,
        [string]$VerificationScreenshotRoot,
        [string]$TargetDir,
        [string]$SessionId,
        [string]$ScenarioName,
        [string]$EditorExe,
        [string]$RuntimeDll,
        [hashtable]$CaptureOptions,
        [object]$InputFixture = $null,
        [string]$GitExecutable = "git.exe"
    )

    $validatedInputFixture = Resolve-ZirconProfileInputFixtureEvidence `
        -RepoRoot $RepoRoot `
        -InputFixture $InputFixture
    $gitMetadata = Get-ZirconProfileGitMetadata -RepoRoot $RepoRoot -GitExecutable $GitExecutable
    $sourceFiles = Get-ZirconProfileCriticalSourcePaths | ForEach-Object {
        $relativePath = $_
        $fingerprint = Get-ZirconProfileRequiredFileFingerprint `
            -Path (Join-Path $RepoRoot $relativePath) `
            -Description "critical source file '$relativePath'"
        [pscustomobject]@{
            relative_path = $relativePath
            sha256 = $fingerprint.sha256
            byte_length = $fingerprint.byte_length
            last_write_utc = $fingerprint.last_write_utc
        }
    }
    $captureToolFiles = Get-ZirconProfileCaptureToolPaths | ForEach-Object {
        $relativePath = $_
        $fingerprint = Get-ZirconProfileRequiredFileFingerprint `
            -Path (Join-Path $RepoRoot $relativePath) `
            -Description "capture tool '$relativePath'"
        [pscustomobject]@{
            relative_path = $relativePath
            sha256 = $fingerprint.sha256
            byte_length = $fingerprint.byte_length
            last_write_utc = $fingerprint.last_write_utc
        }
    }

    $editorFingerprint = Get-ZirconProfileRequiredFileFingerprint `
        -Path $EditorExe `
        -Description "editor binary fingerprint"
    $runtimeFingerprint = Get-ZirconProfileRequiredFileFingerprint `
        -Path $RuntimeDll `
        -Description "Runtime binary fingerprint"
    $newestSourceWriteUtc = @($sourceFiles | ForEach-Object { [datetime]$_.last_write_utc } |
            Sort-Object -Descending | Select-Object -First 1)[0]
    foreach ($binary in @($editorFingerprint, $runtimeFingerprint)) {
        if ([datetime]$binary.last_write_utc -lt $newestSourceWriteUtc) {
            throw "Source-bound profile capture requires binaries built after the newest critical source change: $($binary.path)"
        }
    }

    New-Item -ItemType Directory -Force -Path $ProfileDir | Out-Null

    $manifest = [pscustomobject]@{
        schema_version = 2
        capture_started_utc = (Get-Date).ToUniversalTime().ToString("o")
        session_id = $SessionId
        scenario = $ScenarioName
        input_fixture = $validatedInputFixture
        repository = [pscustomobject]@{
            root = $RepoRoot
            git = $gitMetadata
            critical_source_files = @($sourceFiles)
        }
        binaries = [pscustomobject]@{
            editor = $editorFingerprint
            runtime = $runtimeFingerprint
        }
        capture = [pscustomobject]@{
            output_root = $OutputRoot
            target_dir = $TargetDir
            verification_screenshot_root = $VerificationScreenshotRoot
            options = $CaptureOptions
            tool_files = @($captureToolFiles)
        }
    }
    $manifestPath = Join-Path $ProfileDir "source_manifest.json"
    $manifest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $manifestPath -Encoding UTF8
    return $manifestPath
}
