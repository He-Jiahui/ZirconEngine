$scaleFixtureScript = Join-Path $PSScriptRoot "..\ui\ui-profile-scale-fixture.ps1"
if (Test-Path -LiteralPath $scaleFixtureScript -PathType Leaf) {
    . $scaleFixtureScript
}

function Get-ZirconProfileFileSha256 {
    param([Parameter(Mandatory = $true)][string]$Path)

    $stream = [System.IO.File]::OpenRead($Path)
    $hasher = [System.Security.Cryptography.SHA256]::Create()
    try {
        return -join ($hasher.ComputeHash($stream) | ForEach-Object { $_.ToString('x2') })
    }
    finally {
        $hasher.Dispose()
        $stream.Dispose()
    }
}

function Get-ZirconProfileFileFingerprint {
    param([string]$Path)

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        return $null
    }

    $item = Get-Item -LiteralPath $Path
    return [pscustomobject]@{
        path = $item.FullName
        sha256 = Get-ZirconProfileFileSha256 -Path $Path
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
        "zircon_editor/src/ui/retained/app/host_lifecycle/recompute.rs",
        "zircon_editor/src/ui/retained/app/host_lifecycle/recompute_viewport.rs",
        "zircon_editor/src/ui/retained/app/host_lifecycle/recompute/presentation.rs",
        "zircon_editor/src/ui/retained/app/recompute/invalidation/decision.rs",
        "zircon_editor/src/ui/retained/app.rs",
        "zircon_editor/src/ui/retained/app/committed_shell_state.rs",
        "zircon_editor/src/ui/retained/app/invalidation_bridge/dirty_marking.rs",
        "zircon_editor/src/ui/retained/app/profiling/snapshot_merge.rs",
        "zircon_editor/src/ui/retained/app/host_lifecycle/render_submission.rs",
        "zircon_editor/src/ui/retained/app/runtime_diagnostics_visibility.rs",
        "zircon_editor/src/ui/retained/app/workspace_docking/drawer_resize/movement.rs",
        "zircon_editor/src/ui/retained/app/pane_surface_actions/click.rs",
        "zircon_editor/src/ui/retained/app/startup/state/construction/assembly.rs",
        "zircon_editor/src/ui/retained/app/host_lifecycle/pane_payloads.rs",
        "zircon_editor/src/ui/retained/app/host_lifecycle/pane_payloads/panes.rs",
        "zircon_editor/src/ui/retained/contract/window/event_loop/events/resize.rs",
        "zircon_editor/src/ui/retained/contract/window/event_loop/input.rs",
        "zircon_editor/src/ui/retained/contract/window/event_loop/input_outcome.rs",
        "zircon_editor/src/ui/retained/contract/window/event_loop/pointer_move_mailbox.rs",
        "zircon_editor/src/ui/retained/contract/window/event_loop.rs",
        "zircon_editor/src/ui/retained/contract/window/event_loop/lifecycle.rs",
        "zircon_editor/src/ui/retained/contract/window/event_loop/lifecycle/presenter.rs",
        "zircon_editor/src/ui/retained/contract/redraw.rs",
        "zircon_editor/src/ui/retained/contract/redraw/damage_region.rs",
        "zircon_editor/src/ui/retained/contract/redraw/request.rs",
        "zircon_editor/src/ui/retained/contract/redraw/request/constructors.rs",
        "zircon_editor/src/ui/retained/contract/redraw/request/merge.rs",
        "zircon_editor/src/ui/retained/contract/redraw/request/query.rs",
        "zircon_editor/src/ui/retained/contract/window/event_loop/redraw.rs",
        "zircon_editor/src/ui/retained/contract/window/event_loop/redraw/present.rs",
        "zircon_editor/src/ui/retained/contract/window/event_loop/platform_input.rs",
        "zircon_editor/src/ui/retained/contract/window/event_loop/profile_capture.rs",
        "zircon_editor/src/ui/retained/contract/window/metadata.rs",
        "zircon_editor/src/ui/retained/contract/profiling_artifacts.rs",
        "zircon_editor/src/ui/retained/contract/profiling_artifacts/environment.rs",
        "zircon_editor/src/ui/retained/contract/window/event_wake.rs",
        "zircon_editor/src/ui/retained/contract/presenter/factory.rs",
        "zircon_editor/src/ui/retained/contract/presenter/runtime_factory.rs",
        "zircon_editor/src/ui/retained/contract/presenter/gpu/lifecycle.rs",
        "zircon_editor/src/ui/retained/contract/presenter/gpu/present.rs",
        "zircon_editor/src/ui/retained/contract/presenter/gpu/stats.rs",
        "zircon_editor/src/ui/retained/contract/presenter/softbuffer/present.rs",
        "zircon_editor/src/ui/retained/viewport/presenter_factory.rs",
        "zircon_editor/src/ui/retained/viewport/submit_extract.rs",
        "zircon_editor/src/ui/retained/contract/chrome_command_stream/mod.rs",
        "zircon_editor/src/ui/contract/chrome_command_stream/runtime_draw_list.rs",
        "zircon_editor/src/ui/retained/contract/chrome_command_stream/stream/model.rs",
        "zircon_editor/src/ui/retained/contract/paint_recording/record.rs",
        "zircon_editor/src/ui/retained/contract/chrome_command_stream/extraction/entry.rs",
        "zircon_editor/src/ui/retained/contract/pointer/motion/entry/body.rs",
        "zircon_editor/src/ui/retained/contract/pointer/scroll/entry.rs",
        "zircon_editor/src/ui/retained/contract/surface_hit_test/node/index.rs",
        "zircon_editor/src/ui/retained/contract/surface_hit_test/node/pane_index.rs",
        "zircon_editor/src/ui/retained/contract/paint/node_pipeline/draw.rs",
        "zircon_editor/src/ui/retained/contract/paint/node_pipeline/transform.rs",
        "zircon_editor/src/ui/retained/contract/docks/pane/nodes/console_output.rs",
        "zircon_editor/src/ui/retained/console_output.rs",
        "zircon_editor/src/ui/retained/contract/paint/loading/async_loader.rs",
        "zircon_editor/src/ui/retained/contract/paint/visual_assets/loading/cache.rs",
        "zircon_editor/src/ui/retained/contract/paint/visual_assets/loading/pixels.rs",
        "zircon_editor/src/ui/retained/contract/paint/visual_assets/svg/cache.rs",
        "zircon_editor/src/ui/retained/contract/paint/mui_icons/rendering.rs",
        "zircon_editor/src/ui/retained/contract/chrome_command_stream/icon_atlas.rs",
        "zircon_editor/src/ui/retained/contract/paint_text/draw/layout/cache.rs",
        "zircon_editor/src/ui/retained/contract/profiling_artifacts/geometry.rs",
        "zircon_editor/src/ui/retained/contract/geometry/pane_profile_controls.rs",
        "zircon_editor/src/ui/retained/contract/geometry/pane_frames/pane.rs",
        "zircon_editor/src/ui/retained/contract/profiling_artifacts/schema/geometry.rs",
        "zircon_editor/src/ui/retained/app/viewport/toolbar_pointer/click.rs",
        "zircon_editor/src/ui/retained/app/viewport_toolbar_project/hit_controls.rs",
        "zircon_editor/src/ui/retained/app/surface_frames/docked.rs",
        "zircon_editor/src/ui/retained/app/surface_frames/floating.rs",
        "zircon_editor/src/ui/retained/app/surface_frames/pane_frame.rs",
        "zircon_editor/src/scene/modes/scene_mode_stack.rs",
        "zircon_editor/src/scene/modes/scene_mode_ctx.rs",
        "zircon_editor/src/scene/selection/selection_model.rs",
        "zircon_editor/src/scene/selection/domain_selection.rs",
        "zircon_editor/src/scene/precision/renderer_visible_spatial_pick_source.rs",
        "zircon_editor/src/scene/viewport/pointer/overlay_router/visible_spatial_query.rs",
        "zircon_editor/src/ui/retained/dispatch/shared_pointer/viewport_toolbar.rs",
        "zircon_editor/src/ui/retained/dispatch/workbench/pointer.rs",
        "zircon_editor/src/ui/retained/dispatch/templates/workbench/pointer_feedback.rs",
        "zircon_editor/src/ui/retained/dispatch/workbench/search_clear_action.rs",
        "zircon_editor/src/ui/retained/dispatch/templates/viewport_toolbar/bridge.rs",
        "zircon_editor/src/ui/retained/dispatch/viewport_toolbar/surface_frame_cache.rs",
        "zircon_editor/src/ui/retained/viewport_toolbar_pointer/handle_click.rs",
        "zircon_editor/src/ui/retained/viewport_toolbar_pointer/new.rs",
        "zircon_editor/src/ui/retained/viewport_toolbar_pointer/rebuild_surface.rs",
        "zircon_editor/src/ui/retained/viewport_toolbar_pointer/sync.rs",
        "zircon_editor/src/ui/retained/viewport_toolbar_pointer/sync_surface_frame.rs",
        "zircon_editor/src/ui/retained/viewport_toolbar_pointer/bridge.rs",
        "zircon_editor/src/ui/retained/asset_pointer/common.rs",
        "zircon_editor/src/ui/retained/asset_pointer/content/bridge.rs",
        "zircon_editor/src/ui/retained/asset_pointer/reference/bridge.rs",
        "zircon_editor/src/ui/retained/asset_pointer/tree/bridge.rs",
        "zircon_editor/src/ui/retained/menu_pointer/host_menu_pointer_bridge.rs",
        "zircon_editor/src/ui/menu_pointer/host_menu_pointer_bridge_dispatch_event.rs",
        "zircon_editor/src/ui/menu_pointer/host_menu_pointer_bridge_handle_scroll.rs",
        "zircon_editor/src/ui/menu_pointer/host_menu_pointer_bridge_popup_items.rs",
        "zircon_editor/src/ui/menu_pointer/host_menu_pointer_bridge_project_route.rs",
        "zircon_editor/src/ui/menu_pointer/host_menu_pointer_bridge_rebuild_surface.rs",
        "zircon_editor/src/ui/retained/menu_pointer/popup_layout.rs",
        "zircon_editor/src/ui/retained/menu_pointer/register_handled_pointer_node.rs",
        "zircon_editor/src/ui/retained/welcome_recent_pointer/bridge.rs",
        "zircon_editor/src/ui/retained/welcome_recent_pointer/bridge_handle_click.rs",
        "zircon_editor/src/ui/retained/welcome_recent_pointer/bridge_handle_move.rs",
        "zircon_editor/src/ui/retained/welcome_recent_pointer/bridge_handle_scroll.rs",
        "zircon_editor/src/ui/retained/welcome_recent_pointer/bridge_project_route.rs",
        "zircon_editor/src/ui/retained/welcome_recent_pointer/bridge_sync.rs",
        "zircon_editor/src/ui/retained/welcome_recent_pointer/layout.rs",
        "zircon_editor/src/ui/retained/app/pointer_layout/welcome_recent.rs",
        "zircon_editor/src/ui/retained/hierarchy_pointer/handle_click.rs",
        "zircon_editor/src/ui/retained/hierarchy_pointer/handle_move.rs",
        "zircon_editor/src/ui/retained/hierarchy_pointer/handle_scroll.rs",
        "zircon_editor/src/ui/retained/hierarchy_pointer/hierarchy_pointer_bridge.rs",
        "zircon_editor/src/ui/retained/hierarchy_pointer/hierarchy_pointer_layout.rs",
        "zircon_editor/src/ui/retained/hierarchy_pointer/route_at_point.rs",
        "zircon_editor/src/ui/retained/hierarchy_pointer/sync.rs",
        "zircon_editor/src/ui/retained/app/pointer_layout/hierarchy.rs",
        "zircon_editor/src/ui/layouts/views/asset_browser.rs",
        "zircon_editor/src/ui/layouts/views/asset_browser/logical_paint_source.rs",
        "zircon_editor/src/ui/workbench/asset_content_layout/browser_virtualization.rs",
        "zircon_editor/src/ui/workbench/project/asset_workspace_state.rs",
        "zircon_editor/src/ui/workbench/snapshot/asset/asset_workspace_item_generation.rs",
        "zircon_editor/src/ui/host/editor_event_runtime_access/asset_access.rs",
        "zircon_editor/src/ui/workbench/shell_state.rs",
        "zircon_editor/src/ui/retained/app/asset_content_pointer/events/motion.rs",
        "zircon_editor/src/ui/retained/contract/pointer/scroll/pane/asset/content.rs",
        "zircon_editor/src/ui/contract/docks/pane/nodes/asset_content/projector.rs",
        "zircon_editor/src/ui/workbench/asset_content_layout/paint_metadata.rs",
        "zircon_editor/src/ui/retained/shell_pointer/bridge.rs",
        "zircon_editor/src/ui/retained/shell_pointer/common.rs",
        "zircon_editor/src/ui/retained/shell_pointer/drag_frames.rs",
        "zircon_editor/src/ui/retained/shell_pointer/drag_surface.rs",
        "zircon_editor/src/ui/retained/shell_pointer/node_ids.rs",
        "zircon_editor/src/ui/retained/app/assets/refresh.rs",
        "zircon_editor/src/ui/retained/app/assets/refresh/apply.rs",
        "zircon_editor/src/ui/retained/app/assets/refresh/snapshots.rs",
        "zircon_editor/src/ui/retained/ui/apply_presentation.rs",
        "zircon_editor/src/ui/retained/ui/scoped_presentation.rs",
        "zircon_editor/src/ui/retained/app/asset_events.rs",
        "zircon_editor/src/ui/asset_editor/binding/binding_inspector.rs",
        "zircon_editor/src/ui/asset_editor/binding/binding_inspector/payload_editing.rs",
        "zircon_editor/src/ui/asset_editor/binding/schema_projection.rs",
        "zircon_editor/src/ui/asset_editor/session/binding_state.rs",
        "zircon_editor/src/ui/retained/app/asset_events/binding/entry/lifecycle.rs",
        "zircon_editor/src/ui/retained/app/asset_events/binding/payload.rs",
        "zircon_editor/src/ui/retained/app/asset_events/binding/suggestions/action.rs",
        "zircon_editor/src/ui/retained/app/asset_events/binding/suggestions/payload.rs",
        "zircon_editor/src/ui/retained/app/asset_events/binding/suggestions/route.rs",
        "zircon_editor/src/ui/retained/app/asset_events/collection.rs",
        "zircon_editor/src/ui/retained/app/asset_events/component_adapter.rs",
        "zircon_editor/src/ui/retained/app/asset_events/palette.rs",
        "zircon_editor/src/ui/retained/app/asset_events/preview/nested.rs",
        "zircon_editor/src/ui/retained/app/asset_events/preview/suggestions.rs",
        "zircon_editor/src/ui/retained/app/asset_events/preview/value.rs",
        "zircon_editor/src/ui/retained/app/asset_events/source.rs",
        "zircon_editor/src/ui/retained/app/asset_events/structure/layout/semantic.rs",
        "zircon_editor/src/ui/retained/app/asset_events/structure/slot/semantic.rs",
        "zircon_editor/src/ui/retained/app/asset_events/style/class.rs",
        "zircon_editor/src/ui/retained/app/asset_events/style/rules/declaration.rs",
        "zircon_editor/src/ui/retained/app/asset_events/style/rules/rule.rs",
        "zircon_editor/src/ui/retained/app/asset_events/style/theme_source.rs",
        "zircon_editor/src/ui/retained/app/asset_events/style/tokens.rs",
        "zircon_editor/src/ui/retained/app/asset_events/widget/promote.rs",
        "zircon_editor/src/ui/layouts/views/view_projection/projection_cache.rs",
        "zircon_editor/src/ui/layouts/views/projection_cache/render_command_index.rs",
        "zircon_editor/src/ui/retained/dispatch/templates/virtual_rows.rs",
        "zircon_editor/src/ui/retained/dispatch/workbench/component_property_rows.rs",
        "zircon_editor/src/ui/retained/dispatch/workbench/componentized_window.rs",
        "zircon_editor/src/ui/retained/dispatch/templates/workbench/data_sync.rs",
        "zircon_editor/src/ui/retained/dispatch/workbench/scene_hierarchy_fragment.rs",
        "zircon_editor/src/ui/retained/dispatch/workbench/scene_hierarchy_project.rs",
        "zircon_editor/src/ui/retained/dispatch/templates/workbench/scene_tree_rows.rs",
        "zircon_editor/src/ui/retained/app/host_lifecycle/scene_hierarchy_refresh.rs",
        "zircon_editor/src/ui/retained/app/scene_hierarchy_refresh/hierarchy_row_patch.rs",
        "zircon_editor/src/ui/retained/app/detail_scroll_pointer/inspector.rs",
        "zircon_editor/assets/ui/editor/unreal/components/details.zui",
        "zircon_editor/assets/ui/editor/unreal/components/content_drawer.zui",
        "zircon_editor/assets/ui/editor/unreal/components/menus.zui",
        "zircon_editor/assets/ui/editor/unreal/components/outliner.zui",
        "zircon_editor/assets/ui/editor/unreal/components/status_bar.zui",
        "zircon_editor/assets/ui/editor/unreal/components/toolbar.zui",
        "zircon_editor/assets/ui/editor/unreal/components/viewport.zui",
        "zircon_editor/assets/ui/editor/unreal/editor_shell.zui",
        "zircon_editor/assets/ui/editor/unreal/level_editor_window.zui",
        "zircon_editor/assets/ui/editor/unreal/starship.zui",
        "zircon_editor/src/ui/workbench/snapshot/data/scene_entry/entries.rs",
        "zircon_editor/src/ui/retained/contract/workbench/native_panes/hierarchy.rs",
        "zircon_editor/src/ui/retained/contract/workbench/hierarchy/viewport.rs",
        "zircon_editor/src/ui/retained/contract/workbench/native_panes/assets/frame.rs",
        "zircon_editor/src/ui/retained/contract/workbench/native_panes/diagnostics.rs",
        "zircon_editor/src/ui/retained/contract/workbench/menus/bar.rs",
        "zircon_editor/src/ui/retained/contract/workbench/menus/rows.rs",
        "zircon_editor/src/ui/retained/contract/workbench/menus/popup.rs",
        "zircon_editor/src/ui/retained/contract/workbench/menus/popup/submenus.rs",
        "zircon_editor/src/ui/retained/contract/workbench/welcome/recent_projects/rows.rs",
        "zircon_editor/src/ui/retained/contract/workbench/docks/rail.rs",
        "zircon_editor/src/ui/retained/contract/workbench/overlay/page_overflow.rs",
        "zircon_editor/src/ui/retained/contract/paint/popup_rows/menu/entry.rs",
        "zircon_editor/src/ui/retained/contract/paint/popup_rows/options/entry.rs",
        "zircon_editor/src/ui/retained/contract/paint/dialogs/actions/labels.rs",
        "zircon_editor/src/ui/retained/contract/paint/dropdowns/text.rs",
        "zircon_editor/src/ui/retained/contract/paint/segmented_controls/options.rs",
        "zircon_editor/src/ui/retained/contract/paint/segmented_controls/commands.rs",
        "zircon_editor/src/ui/retained/contract/paint/segmented_controls/segments/body.rs",
        "zircon_editor/src/ui/retained/primitives.rs",
        "zircon_editor/src/ui/retained/persistent_row_patch_map.rs",
        "zircon_editor/src/ui/retained/ui_perf.rs",
        "zircon_editor/src/ui/retained/ui_perf/counter_batch.rs",
        "zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs",
        "zircon_runtime/src/ui/surface/binding_targets.rs",
        "zircon_runtime/src/ui/surface/surface/event_routing.rs",
        "zircon_runtime/src/ui/surface/surface/rebuild.rs",
        "zircon_runtime/src/ui/surface/surface/rebuild/report.rs",
        "zircon_runtime/src/ui/surface/surface/frame_publication.rs",
        "zircon_runtime/src/ui/surface/surface.rs",
        "zircon_runtime/src/ui/surface/arranged.rs",
        "zircon_runtime/src/ui/surface/arranged_visibility.rs",
        "zircon_runtime/src/ui/surface/render/extract.rs",
        "zircon_runtime/src/ui/surface/render/extract/pixel_snapping.rs",
        "zircon_runtime/src/ui/surface/diagnostics.rs",
        "zircon_runtime/src/ui/surface/ecs_projection.rs",
        "zircon_runtime/src/ui/surface/frame_hit_test.rs",
        "zircon_runtime/src/ui/surface/navigation_index.rs",
        "zircon_runtime/src/ui/surface/navigation_index/profile.rs",
        "zircon_runtime/src/ui/surface/virtual_list_materialization.rs",
        "zircon_runtime/src/ui/surface/virtual_list_prototype_pool.rs",
        "zircon_runtime/src/ui/surface/render/cache.rs",
        "zircon_runtime/src/ui/surface/focus.rs",
        "zircon_runtime/src/ui/tree/hit_test.rs",
        "zircon_runtime/src/ui/tree/hit_test/route_index.rs",
        "zircon_runtime/src/ui/tree/node/focus.rs",
        "zircon_runtime/src/ui/tree/node/scroll.rs",
        "zircon_runtime/src/ui/layout/virtualization/materialization.rs",
        "zircon_runtime/src/ui/layout/pass/virtual_list_layout.rs",
        "zircon_runtime/src/ui/layout/pass/arrange/virtual_list.rs",
        "zircon_runtime/src/ui/layout/pass/incremental.rs",
        "zircon_runtime/src/ui/layout/pass/slot.rs",
        "zircon_runtime/src/ui/layout/pass/measure.rs",
        "zircon_runtime/src/ui/layout/pass/measure/traversal.rs",
        "zircon_runtime/src/ui/layout/pass/arrange.rs",
        "zircon_runtime/src/ui/v2/style/runtime_state.rs",
        "zircon_runtime_interface/src/ui/tree/node/layout_cache.rs",
        "zircon_runtime_interface/src/ui/tree/node/ui_tree.rs",
        "zircon_runtime_interface/src/ui/dispatch/input/metadata.rs",
        "zircon_runtime_interface/src/ui/surface/hit.rs",
        "zircon_runtime/src/core/runtime/diagnostics/profiling/mod.rs",
        "zircon_runtime/src/core/runtime/diagnostics/profiling/recorder.rs",
        "zircon_runtime/src/core/framework/render/framework.rs",
        "zircon_runtime/src/core/framework/render/ui_submission.rs",
        "zircon_runtime/src/core/framework/render/visible_spatial_query.rs",
        "zircon_runtime/src/dynamic_api/session/runtime_ui.rs",
        "zircon_runtime/src/graphics/runtime/query_visible_spatial_snapshot/snapshot.rs",
        "zircon_runtime/src/graphics/runtime/viewport_record/visible_spatial_query.rs",
        "zircon_runtime/src/graphics/scene/resources/ui_texture.rs",
        "zircon_runtime/src/graphics/scene/renderer/ui/render.rs",
        "zircon_runtime/src/graphics/scene/renderer/ui/render/background.rs",
        "zircon_runtime/src/graphics/scene/renderer/ui/render/geometry.rs",
        "zircon_runtime/src/graphics/scene/renderer/ui/render/plan_cache.rs",
        "zircon_runtime/src/graphics/scene/renderer/ui/render/record.rs",
        "zircon_runtime/src/graphics/scene/renderer/ui/render/resolved_layout.rs",
        "zircon_runtime/src/graphics/scene/renderer/ui/shaders/screen_space_ui.wgsl",
        "zircon_runtime/src/graphics/scene/renderer/ui/screen_space_ui_renderer.rs",
        "zircon_runtime/src/graphics/types/viewport_render_frame.rs",
        "zircon_runtime/src/core/runtime/diagnostics/profiling/export.rs",
        "zircon_runtime/src/core/runtime/diagnostics/profiling/ui_hotspot.rs",
        "zircon_runtime_interface/src/profiling.rs",
        "zircon_runtime_interface/src/ui/surface/mod.rs",
        "zircon_runtime_interface/src/ui/surface/frame.rs",
        "zircon_runtime_interface/src/ui/surface/render/frame_extract.rs",
        "zircon_runtime_interface/src/ui/surface/diagnostics.rs",
        "zircon_runtime/src/text/ui_style.rs",
        "zircon_editor/src/ui/retained/contract/paint/settings_window/commands.rs",
        "zircon_editor/src/ui/retained/contract/paint/section_titles/commands.rs",
        "zircon_editor/src/ui/retained/contract/paint/section_titles/geometry.rs",
        "zircon_runtime/crates/zr_rhi/src/ui_surface.rs",
        "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface.rs",
        "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/presentation.rs",
        "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/retained_cache.rs",
        "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/batching.rs",
        "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/batching/bounds_index.rs",
        "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/batching/dependency_depths.rs",
        "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/geometry.rs",
        "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/pipeline.rs",
        "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/render_pass.rs",
        "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/shaders/ui_material.wgsl",
        "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/image_cache.rs",
        "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/image_cache/resource.rs",
        "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/shared_image_registry.rs"
    )
}

function Get-ZirconProfileCaptureToolPaths {
    return @(
        "tools/analysis/profiling/ui/ui-profile-capture.ps1",
        "tools/analysis/profiling/ui/ui-profile-scenarios.ps1",
        "tools/analysis/profiling/ui/ui-profile-latency-evidence.ps1",
        "tools/analysis/profiling/ui/ui-profile-process-evidence.ps1",
        "tools/analysis/profiling/ui/ui-profile-counter-evidence.ps1",
        "tools/analysis/profiling/ui/ui-profile-workbench-pointer-evidence.ps1",
        "tools/analysis/profiling/ui/ui-profile-native-resize.ps1",
        "tools/analysis/profiling/ui/ui-profile-hierarchy-filter-input.ps1",
        "tools/analysis/profiling/ui/ui-profile-hierarchy-filter-metrics.ps1",
        "tools/analysis/profiling/ui/ui-profile-scale-fixture.ps1",
        "tools/analysis/profiling/ui/ui-profile-surface-pipeline-metrics.ps1",
        "tools/analysis/profiling/ui/ui-profile-chrome-paint-metrics.ps1",
        "tools/analysis/profiling/ui/ui-profile-machine-manifest.ps1",
        "tools/analysis/profiling/shared/performance-machine-manifest.ps1",
        "tools/analysis/profiling/shared/profile-capture-paths.ps1",
        "tools/analysis/profiling/ui/ui-profile-product-directory.ps1",
        "tools/analysis/profiling/shared/profile-capture-manifest.ps1"
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
        $kind -notin @("hierarchy_scene", "asset_catalog_json", "viewport_pointer_scene") -or
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
        foreach ($field in @("asset_item_count", "source_extension", "workspace", "asset_sources")) {
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
        $workspace = Resolve-ZirconProfileInputFixtureFileEvidence `
            -ProjectRoot $projectRoot `
            -Evidence $InputFixture.workspace `
            -ExpectedRelativePath ".zircon/editor-workspace.json" `
            -Description "editor workspace"

        return [pscustomobject]@{
            schema_version = 1
            kind = "asset_catalog_json"
            project_root = $projectRoot
            template_relative_path = [string]$InputFixture.template_relative_path
            asset_item_count = $assetItemCount
            source_extension = "json"
            project_manifest = $projectManifest
            scene = $scene
            workspace = $workspace
            asset_sources = $assetSources
        }
    }

    if ($kind -eq "viewport_pointer_scene") {
        foreach ($field in @("selectable_node_count", "scene_entity_count", "mobility")) {
            if ($null -eq $InputFixture.PSObject.Properties[$field]) {
                throw "UI profile input fixture is missing required field '$field'."
            }
        }
        $selectableNodeCount = [int64]$InputFixture.selectable_node_count
        $sceneEntityCount = [int64]$InputFixture.scene_entity_count
        $mobility = [string]$InputFixture.mobility
        if ($selectableNodeCount -lt 1 -or $selectableNodeCount -gt 10000 -or
            $sceneEntityCount -ne $selectableNodeCount + 2) {
            throw "UI profile viewport pointer fixture count is inconsistent."
        }
        if ($mobility -notin @("static", "dynamic")) {
            throw "UI profile viewport pointer fixture mobility is inconsistent."
        }
        $sceneSource = Get-Content -LiteralPath $scene.path -Raw
        $expectedMobility = if ($mobility -eq "static") { "Static" } else { "Dynamic" }
        $entityCount = [regex]::Matches($sceneSource, "(?m)^\[\[entities\]\]\r?$").Count
        $mobilityCount = [regex]::Matches(
            $sceneSource,
            "(?m)^mobility = `"$expectedMobility`"\r?$"
        ).Count
        $meshCount = [regex]::Matches(
            $sceneSource,
            "(?m)^\[entities\.mesh\.model\]\r?$"
        ).Count
        if ($mobilityCount -ne $selectableNodeCount) {
            throw "UI profile viewport pointer fixture mobility is inconsistent."
        }
        if ($entityCount -ne $sceneEntityCount -or $meshCount -ne $selectableNodeCount) {
            throw "UI profile viewport pointer fixture scene does not match declared count."
        }

        return [pscustomobject]@{
            schema_version = 1
            kind = "viewport_pointer_scene"
            project_root = $projectRoot
            template_relative_path = [string]$InputFixture.template_relative_path
            selectable_node_count = $selectableNodeCount
            scene_entity_count = $sceneEntityCount
            mobility = $mobility
            project_manifest = $projectManifest
            scene = $scene
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
