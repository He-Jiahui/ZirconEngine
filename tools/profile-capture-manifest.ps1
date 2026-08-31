$scaleFixtureScript = Join-Path $PSScriptRoot "ui-profile-scale-fixture.ps1"
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
        "zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute.rs",
        "zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute_viewport.rs",
        "zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/presentation.rs",
        "zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/invalidation/decision.rs",
        "zircon_editor/src/ui/retained_host/app.rs",
        "zircon_editor/src/ui/retained_host/app/committed_shell_state.rs",
        "zircon_editor/src/ui/retained_host/app/host_lifecycle/invalidation_bridge/dirty_marking.rs",
        "zircon_editor/src/ui/retained_host/app/profiling/snapshot_merge.rs",
        "zircon_editor/src/ui/retained_host/app/host_lifecycle/render_submission.rs",
        "zircon_editor/src/ui/retained_host/app/runtime_diagnostics_visibility.rs",
        "zircon_editor/src/ui/retained_host/app/workspace_docking/drawer_resize/movement.rs",
        "zircon_editor/src/ui/retained_host/app/pane_surface_actions/click.rs",
        "zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/state/construction/assembly.rs",
        "zircon_editor/src/ui/retained_host/app/host_lifecycle/pane_payloads.rs",
        "zircon_editor/src/ui/retained_host/app/host_lifecycle/pane_payloads/workbench_panes.rs",
        "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events/resize.rs",
        "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/input.rs",
        "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/input_outcome.rs",
        "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/pointer_move_mailbox.rs",
        "zircon_editor/src/ui/retained_host/host_contract/window/event_loop.rs",
        "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/lifecycle.rs",
        "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/lifecycle/presenter.rs",
        "zircon_editor/src/ui/retained_host/host_contract/redraw.rs",
        "zircon_editor/src/ui/retained_host/host_contract/redraw/damage_region.rs",
        "zircon_editor/src/ui/retained_host/host_contract/redraw/request.rs",
        "zircon_editor/src/ui/retained_host/host_contract/redraw/request/constructors.rs",
        "zircon_editor/src/ui/retained_host/host_contract/redraw/request/merge.rs",
        "zircon_editor/src/ui/retained_host/host_contract/redraw/request/query.rs",
        "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/redraw.rs",
        "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/redraw/present.rs",
        "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/platform_input.rs",
        "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/profile_capture.rs",
        "zircon_editor/src/ui/retained_host/host_contract/window/metadata.rs",
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
        "zircon_editor/src/ui/retained_host/viewport/submit_extract.rs",
        "zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/mod.rs",
        "zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/runtime_draw_list.rs",
        "zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/stream/model.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_recording/record.rs",
        "zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/extraction/entry.rs",
        "zircon_editor/src/ui/retained_host/host_contract/native_pointer/move_dispatch/entry/body.rs",
        "zircon_editor/src/ui/retained_host/host_contract/native_pointer/scroll_dispatch/entry.rs",
        "zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/index.rs",
        "zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/pane_index.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline/draw.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline/transform.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/console_output.rs",
        "zircon_editor/src/ui/retained_host/console_output.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/loading/async_loader.rs",
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
        "zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/hit_controls.rs",
        "zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/surface_frames/docked.rs",
        "zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/surface_frames/floating.rs",
        "zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/surface_frames/pane_frame.rs",
        "zircon_editor/src/scene/modes/scene_mode_stack.rs",
        "zircon_editor/src/scene/modes/scene_mode_ctx.rs",
        "zircon_editor/src/scene/selection/selection_model.rs",
        "zircon_editor/src/scene/selection/domain_selection.rs",
        "zircon_editor/src/scene/viewport/pointer/precision/renderer_visible_spatial_pick_source.rs",
        "zircon_editor/src/scene/viewport/pointer/overlay_router/viewport_overlay_pointer_router_visible_spatial_query.rs",
        "zircon_editor/src/ui/retained_host/callback_dispatch/shared_pointer/viewport_toolbar.rs",
        "zircon_editor/src/ui/retained_host/callback_dispatch/workbench/pointer.rs",
        "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/pointer_feedback.rs",
        "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/search_clear_action.rs",
        "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/viewport_toolbar/bridge.rs",
        "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/viewport_toolbar/surface_frame_cache.rs",
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
        "zircon_editor/src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge_sync.rs",
        "zircon_editor/src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_layout.rs",
        "zircon_editor/src/ui/retained_host/app/pointer_layout/welcome_recent.rs",
        "zircon_editor/src/ui/retained_host/hierarchy_pointer/handle_click.rs",
        "zircon_editor/src/ui/retained_host/hierarchy_pointer/handle_move.rs",
        "zircon_editor/src/ui/retained_host/hierarchy_pointer/handle_scroll.rs",
        "zircon_editor/src/ui/retained_host/hierarchy_pointer/hierarchy_pointer_bridge.rs",
        "zircon_editor/src/ui/retained_host/hierarchy_pointer/hierarchy_pointer_layout.rs",
        "zircon_editor/src/ui/retained_host/hierarchy_pointer/route_at_point.rs",
        "zircon_editor/src/ui/retained_host/hierarchy_pointer/sync.rs",
        "zircon_editor/src/ui/retained_host/app/pointer_layout/hierarchy.rs",
        "zircon_editor/src/ui/layouts/views/asset_browser.rs",
        "zircon_editor/src/ui/layouts/views/asset_browser/logical_paint_source.rs",
        "zircon_editor/src/ui/workbench/asset_content_layout/browser_virtualization.rs",
        "zircon_editor/src/ui/workbench/project/asset_workspace_state.rs",
        "zircon_editor/src/ui/workbench/snapshot/asset/asset_workspace_item_generation.rs",
        "zircon_editor/src/ui/host/editor_event_runtime_access/asset_access.rs",
        "zircon_editor/src/ui/workbench/shell_state.rs",
        "zircon_editor/src/ui/retained_host/app/asset_content_pointer/events/motion.rs",
        "zircon_editor/src/ui/retained_host/host_contract/native_pointer/scroll_dispatch/pane/asset/content.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/asset_content/projector.rs",
        "zircon_editor/src/ui/workbench/asset_content_layout/paint_metadata.rs",
        "zircon_editor/src/ui/retained_host/shell_pointer/bridge.rs",
        "zircon_editor/src/ui/retained_host/shell_pointer/common.rs",
        "zircon_editor/src/ui/retained_host/shell_pointer/drag_frames.rs",
        "zircon_editor/src/ui/retained_host/shell_pointer/drag_surface.rs",
        "zircon_editor/src/ui/retained_host/shell_pointer/node_ids.rs",
        "zircon_editor/src/ui/retained_host/app/assets/refresh.rs",
        "zircon_editor/src/ui/retained_host/app/assets/refresh/apply.rs",
        "zircon_editor/src/ui/retained_host/app/assets/refresh/snapshots.rs",
        "zircon_editor/src/ui/retained_host/ui/apply_presentation.rs",
        "zircon_editor/src/ui/retained_host/ui/scoped_presentation.rs",
        "zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events.rs",
        "zircon_editor/src/ui/asset_editor/binding/binding_inspector.rs",
        "zircon_editor/src/ui/asset_editor/binding/binding_inspector/payload_editing.rs",
        "zircon_editor/src/ui/asset_editor/binding/schema_projection.rs",
        "zircon_editor/src/ui/asset_editor/session/binding_state.rs",
        "zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/binding/entry/lifecycle.rs",
        "zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/binding/payload.rs",
        "zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/binding/suggestions/action.rs",
        "zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/binding/suggestions/payload.rs",
        "zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/binding/suggestions/route.rs",
        "zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/collection.rs",
        "zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/component_adapter.rs",
        "zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/palette.rs",
        "zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/preview/nested.rs",
        "zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/preview/suggestions.rs",
        "zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/preview/value.rs",
        "zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/source.rs",
        "zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/structure/layout/semantic.rs",
        "zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/structure/slot/semantic.rs",
        "zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/style/class.rs",
        "zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/style/rules/declaration.rs",
        "zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/style/rules/rule.rs",
        "zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/style/theme_source.rs",
        "zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/style/tokens.rs",
        "zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/widget/promote.rs",
        "zircon_editor/src/ui/layouts/views/view_projection/projection_cache.rs",
        "zircon_editor/src/ui/layouts/views/view_projection/projection_cache/render_command_index.rs",
        "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/virtual_rows.rs",
        "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/component_property_rows.rs",
        "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/componentized_window.rs",
        "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/data_sync.rs",
        "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/scene_hierarchy_fragment.rs",
        "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/scene_hierarchy_projection.rs",
        "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/scene_tree_rows.rs",
        "zircon_editor/src/ui/retained_host/app/host_lifecycle/scene_hierarchy_refresh.rs",
        "zircon_editor/src/ui/retained_host/app/host_lifecycle/scene_hierarchy_refresh/hierarchy_row_patch.rs",
        "zircon_editor/src/ui/retained_host/app/detail_scroll_pointer/inspector.rs",
        "zircon_editor/assets/ui/editor/components/workbench/shell/workbench_inspector_panel.zui",
        "zircon_editor/src/ui/workbench/snapshot/data/scene_entry/entries.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/hierarchy.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/hierarchy/viewport.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/assets/frame.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/diagnostics.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/menus/bar.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/menus/rows.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/menus/popup.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/menus/popup/submenus.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/welcome/recent_projects/rows.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/rail.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/scene_layers/overlay/page_overflow.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows/menu/entry.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows/options/entry.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dialogs/actions/labels.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dropdowns/text.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_segmented_controls/options.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_segmented_controls/commands.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_segmented_controls/segments/body.rs",
        "zircon_editor/src/ui/retained_host/primitives.rs",
        "zircon_editor/src/ui/retained_host/persistent_row_patch_map.rs",
        "zircon_editor/src/ui/retained_host/ui_perf.rs",
        "zircon_editor/src/ui/retained_host/ui_perf/counter_batch.rs",
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
        "zircon_runtime/src/graphics/runtime/render_framework/query_visible_spatial_snapshot/query_visible_spatial_snapshot.rs",
        "zircon_runtime/src/graphics/runtime/render_framework/viewport_record/visible_spatial_query.rs",
        "zircon_runtime/src/graphics/scene/resources/ui_texture.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/background.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/plan_cache.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/record.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/resolved_layout.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/ui/screen_space_ui_renderer.rs",
        "zircon_runtime/src/graphics/types/viewport_render_frame.rs",
        "zircon_runtime/src/core/runtime/diagnostics/profiling/export.rs",
        "zircon_runtime/src/core/runtime/diagnostics/profiling/ui_hotspot.rs",
        "zircon_runtime_interface/src/profiling.rs",
        "zircon_runtime_interface/src/ui/surface/mod.rs",
        "zircon_runtime_interface/src/ui/surface/frame.rs",
        "zircon_runtime_interface/src/ui/surface/render/frame_extract.rs",
        "zircon_runtime_interface/src/ui/surface/diagnostics.rs",
        "zircon_runtime/src/text/ui_style.rs",
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
        "tools/ui-profile-capture.ps1",
        "tools/ui-profile-scenarios.ps1",
        "tools/ui-profile-latency-evidence.ps1",
        "tools/ui-profile-process-evidence.ps1",
        "tools/ui-profile-counter-evidence.ps1",
        "tools/ui-profile-workbench-pointer-evidence.ps1",
        "tools/ui-profile-native-resize.ps1",
        "tools/ui-profile-hierarchy-filter-input.ps1",
        "tools/ui-profile-hierarchy-filter-metrics.ps1",
        "tools/ui-profile-scale-fixture.ps1",
        "tools/ui-profile-surface-pipeline-metrics.ps1",
        "tools/ui-profile-chrome-paint-metrics.ps1",
        "tools/ui-profile-machine-manifest.ps1",
        "tools/performance-machine-manifest.ps1",
        "tools/profile-capture-paths.ps1",
        "tools/ui-profile-product-directory.ps1",
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
