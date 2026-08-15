use std::collections::BTreeMap;

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::welcome_presentation;
use crate::ui::layouts::windows::workbench_host_window::{
    self as host_window, build_host_dock_surface_patch, HostDockSurfaceId, HostDockSurfacePatch,
    HostWindowSurfaceData, ShellPresentation,
};
use crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowLayoutFrames;
use crate::ui::retained_host::host_contract::{
    FrameRect, HostBottomDockSurfaceData, HostDockPresentationPatch, HostSideDockSurfaceData,
    HostWindowLayoutData, HostWindowPresentationData, HostWindowShellData, PaneData,
    TemplatePaneNodeData, UiHostWindow,
};
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::retained_host::HostShellContentScope;
use crate::ui::template_runtime::EditorUiHostRuntime;
use crate::ui::workbench::autolayout::WorkbenchShellGeometry;
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::{EditorChromeSnapshot, ViewContentKind};

use super::apply_presentation_impl::{
    host_window_layout, to_host_contract_bottom_dock, to_host_contract_host_shell,
    to_host_contract_host_window_layout, to_host_contract_side_dock,
};

#[derive(Clone)]
pub(crate) struct ShellContentTarget {
    pub(crate) dock: HostDockSurfaceId,
    pub(crate) content_kind: ViewContentKind,
    pub(crate) instance_id: Option<String>,
}

pub(crate) fn shell_content_target(
    scope: &HostShellContentScope,
    model: &WorkbenchViewModel,
) -> Option<ShellContentTarget> {
    let slot = scope.slot.canonical();
    let dock = dock_for_drawer_slot(slot);
    let slots: &[ActivityDrawerSlot] = match dock {
        HostDockSurfaceId::Left => &[ActivityDrawerSlot::LeftTop, ActivityDrawerSlot::LeftBottom],
        HostDockSurfaceId::Right => &[
            ActivityDrawerSlot::RightTop,
            ActivityDrawerSlot::RightBottom,
        ],
        HostDockSurfaceId::Bottom => &[ActivityDrawerSlot::Bottom],
    };
    let selection = host_window::side_pane_selection(model, slots)?;
    if selection.stack.slot.canonical() != slot
        || selection.tab.instance_id != scope.instance_id
        || selection.stack.active_tab.as_ref() != Some(&scope.instance_id)
    {
        return None;
    }
    Some(ShellContentTarget {
        dock,
        content_kind: selection.tab.content_kind,
        instance_id: Some(selection.tab.instance_id.0.clone()),
    })
}

fn dock_for_drawer_slot(slot: ActivityDrawerSlot) -> HostDockSurfaceId {
    match slot.canonical() {
        ActivityDrawerSlot::LeftTop | ActivityDrawerSlot::LeftBottom => HostDockSurfaceId::Left,
        ActivityDrawerSlot::RightTop | ActivityDrawerSlot::RightBottom => HostDockSurfaceId::Right,
        ActivityDrawerSlot::Bottom
        | ActivityDrawerSlot::BottomLeft
        | ActivityDrawerSlot::BottomRight => HostDockSurfaceId::Bottom,
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn patch_shell_content_presentation_from_state(
    ui: &UiHostWindow,
    target: ShellContentTarget,
    model: &WorkbenchViewModel,
    chrome: &EditorChromeSnapshot,
    geometry: &WorkbenchShellGeometry,
    preset_names: &[String],
    active_preset_name: Option<&str>,
    ui_asset_panes: &BTreeMap<String, crate::ui::asset_editor::UiAssetEditorPanePresentation>,
    animation_panes: &BTreeMap<
        String,
        crate::ui::animation_editor::AnimationEditorPanePresentation,
    >,
    runtime_diagnostics: &zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot,
    module_plugins: &host_window::ModulePluginsPaneViewData,
    build_export: &host_window::BuildExportPaneViewData,
    template_v2_data: &BTreeMap<
        String,
        crate::core::editor_extension::EditorUiTemplatePaneDataSnapshot,
    >,
    componentized_workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames,
    component_showcase_runtime: Option<&EditorUiHostRuntime>,
    hierarchy_filter_query: &str,
    chrome_projection_cache: &mut host_window::HostChromeProjectionCache,
) -> bool {
    zircon_runtime::profile_scope!(
        "editor",
        "retained_host",
        "apply_early_shell_content_presentation_patch"
    );
    let current_generation = ui.get_host_presentation_generation();
    let current = current_generation.structure();
    let host_shell = host_window::build_host_window_shell_data(
        model,
        chrome,
        geometry,
        preset_names,
        active_preset_name,
        chrome_projection_cache,
    );
    let host_layout = host_window_layout(componentized_workbench_layout_frames);
    let next_shell = to_host_contract_host_shell(&host_shell);
    let next_layout = to_host_contract_host_window_layout(&host_layout);
    if !validate_shell_content_context(
        current,
        &next_shell,
        &next_layout,
        chrome.status_line.as_str(),
        host_window::document_pane_selection(model)
            .map(|tab| tab.instance_id.0.as_str())
            .unwrap_or(""),
    ) {
        return false;
    }

    let left_tabs = chrome_projection_cache.left_tabs(model);
    let right_tabs = chrome_projection_cache.right_tabs(model);
    let bottom_tabs = chrome_projection_cache.bottom_tabs(model);
    let target_pane = |slots: &[ActivityDrawerSlot]| {
        host_window::side_pane_with_template_v2_data(
            model,
            chrome,
            slots,
            ui_asset_panes,
            animation_panes,
            Some(runtime_diagnostics),
            module_plugins,
            build_export,
            template_v2_data,
        )
    };
    let (left_pane, right_pane, bottom_pane) = match target.dock {
        HostDockSurfaceId::Left => (
            target_pane(&[ActivityDrawerSlot::LeftTop, ActivityDrawerSlot::LeftBottom]),
            host_window::blank_pane(),
            host_window::blank_pane(),
        ),
        HostDockSurfaceId::Right => (
            host_window::blank_pane(),
            target_pane(&[
                ActivityDrawerSlot::RightTop,
                ActivityDrawerSlot::RightBottom,
            ]),
            host_window::blank_pane(),
        ),
        HostDockSurfaceId::Bottom => (
            host_window::blank_pane(),
            host_window::blank_pane(),
            target_pane(&[ActivityDrawerSlot::Bottom]),
        ),
    };
    let host_surface_data = HostWindowSurfaceData {
        host_tabs: chrome_projection_cache.host_tabs(model),
        left_tabs,
        right_tabs,
        bottom_tabs,
        document_tabs: chrome_projection_cache.document_tabs(model),
        floating_windows: model_rc(Vec::new()),
        left_pane,
        right_pane,
        bottom_pane,
        document_pane: host_window::blank_pane(),
    };
    if changed_dock_for_model(current, model) != Some(target.dock) {
        record_shell_content_patch_fallback(ShellContentPatchFallback::DockCardinality);
        return false;
    }
    let patch = build_host_dock_surface_patch(
        &host_surface_data,
        &host_shell,
        &host_layout,
        &chrome.project_overview,
        chrome,
        target.dock,
    );
    let welcome = welcome_presentation(&chrome.welcome);
    let (damage, patch) = convert_patch(
        patch,
        component_showcase_runtime,
        Some(&welcome),
        hierarchy_filter_query,
    );
    let commit_context = shell_content_commit_context(&current_generation, target.dock);
    drop(current_generation);
    commit_shell_content_patch(
        ui,
        commit_context,
        next_shell,
        next_layout,
        target.dock,
        damage,
        patch,
    )
}

fn changed_dock_for_model(
    current: &HostWindowPresentationData,
    model: &WorkbenchViewModel,
) -> Option<HostDockSurfaceId> {
    let pane_id = |slots: &[ActivityDrawerSlot]| {
        host_window::side_pane_selection(model, slots)
            .map(|selection| selection.tab.instance_id.0.as_str())
            .unwrap_or("")
    };
    changed_dock_from_pane_ids(
        current.host_scene_data.left_dock.pane.id.as_str(),
        pane_id(&[ActivityDrawerSlot::LeftTop, ActivityDrawerSlot::LeftBottom]),
        current.host_scene_data.right_dock.pane.id.as_str(),
        pane_id(&[
            ActivityDrawerSlot::RightTop,
            ActivityDrawerSlot::RightBottom,
        ]),
        current.host_scene_data.bottom_dock.pane.id.as_str(),
        pane_id(&[ActivityDrawerSlot::Bottom]),
    )
}

pub(super) fn patch_shell_content_presentation(
    ui: &UiHostWindow,
    presentation: &ShellPresentation,
    host_layout: &host_window::HostWindowLayoutData,
    chrome: &crate::ui::workbench::snapshot::EditorChromeSnapshot,
    component_showcase_runtime: Option<&EditorUiHostRuntime>,
    hierarchy_filter_query: &str,
) -> bool {
    zircon_runtime::profile_scope!(
        "editor",
        "retained_host",
        "apply_shell_content_presentation_patch"
    );
    let current_generation = ui.get_host_presentation_generation();
    let current = current_generation.structure();
    let next_shell = to_host_contract_host_shell(&presentation.host_shell);
    let next_layout = to_host_contract_host_window_layout(host_layout);
    if !validate_shell_content_context(
        current,
        &next_shell,
        &next_layout,
        presentation.status_primary.as_str(),
        presentation.host_surface_data.document_pane.id.as_str(),
    ) {
        return false;
    }

    let Some(target) = changed_dock(current, presentation) else {
        record_shell_content_patch_fallback(ShellContentPatchFallback::DockCardinality);
        return false;
    };
    let patch = build_host_dock_surface_patch(
        &presentation.host_surface_data,
        &presentation.host_shell,
        host_layout,
        &chrome.project_overview,
        chrome,
        target,
    );
    let (damage, patch) = convert_patch(
        patch,
        component_showcase_runtime,
        Some(&presentation.welcome),
        hierarchy_filter_query,
    );
    let commit_context = shell_content_commit_context(&current_generation, target);
    drop(current_generation);
    commit_shell_content_patch(
        ui,
        commit_context,
        next_shell,
        next_layout,
        target,
        damage,
        patch,
    )
}

struct ShellContentCommitContext {
    expected_structure_generation: u64,
    debug_refresh_rate: String,
    previous_models: Vec<ModelRc<TemplatePaneNodeData>>,
}

fn shell_content_commit_context(
    generation: &crate::ui::retained_host::host_contract::HostPresentationGeneration,
    target: HostDockSurfaceId,
) -> ShellContentCommitContext {
    let current = generation.structure();
    ShellContentCommitContext {
        expected_structure_generation: generation.structure_generation(),
        debug_refresh_rate: current.host_shell.debug_refresh_rate.clone(),
        previous_models: presentation_paint_models_for_target(current, target),
    }
}

fn validate_shell_content_context(
    current: &HostWindowPresentationData,
    next_shell: &HostWindowShellData,
    next_layout: &HostWindowLayoutData,
    status_primary: &str,
    document_pane_id: &str,
) -> bool {
    if let Some(reason) = shell_mismatch(&current.host_shell, next_shell) {
        record_shell_content_patch_fallback(reason);
        return false;
    }
    if !same_layout(&current.host_layout, next_layout) {
        record_shell_content_patch_fallback(ShellContentPatchFallback::Layout);
        return false;
    }
    if current.host_scene_data.status_bar.status_primary.as_str() != status_primary {
        record_shell_content_patch_fallback(ShellContentPatchFallback::Status);
        return false;
    }
    if current.host_scene_data.document_dock.pane.id.as_str() != document_pane_id {
        record_shell_content_patch_fallback(ShellContentPatchFallback::Document);
        return false;
    }
    true
}

fn commit_shell_content_patch(
    ui: &UiHostWindow,
    context: ShellContentCommitContext,
    mut next_shell: HostWindowShellData,
    next_layout: HostWindowLayoutData,
    target: HostDockSurfaceId,
    damage: FrameRect,
    patch: ConvertedDockPatch,
) -> bool {
    let next_models = converted_patch_paint_models(&patch);
    if context.previous_models.len() != next_models.len() {
        record_shell_content_patch_fallback(ShellContentPatchFallback::HitIndex);
        return false;
    }
    let replacements = context
        .previous_models
        .into_iter()
        .zip(next_models)
        .collect::<Vec<_>>();
    next_shell.debug_refresh_rate = context.debug_refresh_rate;
    let patch = match patch {
        ConvertedDockPatch::Left(dock) => HostDockPresentationPatch::Left(dock),
        ConvertedDockPatch::Right(dock) => HostDockPresentationPatch::Right(dock),
        ConvertedDockPatch::Bottom(dock) => HostDockPresentationPatch::Bottom(dock),
    };
    if !ui.patch_host_presentation_dock(
        context.expected_structure_generation,
        next_shell,
        next_layout,
        patch,
        &replacements,
    ) {
        record_shell_content_patch_fallback(ShellContentPatchFallback::HitIndex);
        return false;
    }
    ui.request_frame_update_region(damage);
    record_shell_content_patch_hit();
    true
}

enum ConvertedDockPatch {
    Left(crate::ui::retained_host::host_contract::HostSideDockSurfaceData),
    Right(crate::ui::retained_host::host_contract::HostSideDockSurfaceData),
    Bottom(crate::ui::retained_host::host_contract::HostBottomDockSurfaceData),
}

fn convert_patch(
    patch: HostDockSurfacePatch,
    component_showcase_runtime: Option<&EditorUiHostRuntime>,
    welcome: Option<&crate::ui::layouts::views::WelcomePresentation>,
    hierarchy_filter_query: &str,
) -> (FrameRect, ConvertedDockPatch) {
    match patch {
        HostDockSurfacePatch::Left(dock) => {
            let converted = to_host_contract_side_dock(
                &dock,
                component_showcase_runtime,
                welcome,
                hierarchy_filter_query,
            );
            (
                converted.region_frame.clone(),
                ConvertedDockPatch::Left(converted),
            )
        }
        HostDockSurfacePatch::Right(dock) => {
            let converted = to_host_contract_side_dock(
                &dock,
                component_showcase_runtime,
                welcome,
                hierarchy_filter_query,
            );
            (
                converted.region_frame.clone(),
                ConvertedDockPatch::Right(converted),
            )
        }
        HostDockSurfacePatch::Bottom(dock) => {
            let converted = to_host_contract_bottom_dock(
                &dock,
                component_showcase_runtime,
                welcome,
                hierarchy_filter_query,
            );
            (
                converted.region_frame.clone(),
                ConvertedDockPatch::Bottom(converted),
            )
        }
    }
}

fn changed_dock(
    current: &HostWindowPresentationData,
    next: &ShellPresentation,
) -> Option<HostDockSurfaceId> {
    changed_dock_from_pane_ids(
        current.host_scene_data.left_dock.pane.id.as_str(),
        next.host_surface_data.left_pane.id.as_str(),
        current.host_scene_data.right_dock.pane.id.as_str(),
        next.host_surface_data.right_pane.id.as_str(),
        current.host_scene_data.bottom_dock.pane.id.as_str(),
        next.host_surface_data.bottom_pane.id.as_str(),
    )
}

fn changed_dock_from_pane_ids(
    current_left: &str,
    next_left: &str,
    current_right: &str,
    next_right: &str,
    current_bottom: &str,
    next_bottom: &str,
) -> Option<HostDockSurfaceId> {
    let mut changed = [
        (current_left != next_left).then_some(HostDockSurfaceId::Left),
        (current_right != next_right).then_some(HostDockSurfaceId::Right),
        (current_bottom != next_bottom).then_some(HostDockSurfaceId::Bottom),
    ]
    .into_iter()
    .flatten();
    let target = changed.next()?;
    changed.next().is_none().then_some(target)
}

fn same_layout(current: &HostWindowLayoutData, next: &HostWindowLayoutData) -> bool {
    current.center_band_frame == next.center_band_frame
        && current.status_bar_frame == next.status_bar_frame
        && current.left_region_frame == next.left_region_frame
        && current.document_region_frame == next.document_region_frame
        && current.right_region_frame == next.right_region_frame
        && current.bottom_region_frame == next.bottom_region_frame
        && current.left_splitter_frame == next.left_splitter_frame
        && current.right_splitter_frame == next.right_splitter_frame
        && current.bottom_splitter_frame == next.bottom_splitter_frame
        && current.viewport_content_frame == next.viewport_content_frame
}

fn same_shell(current: &HostWindowShellData, next: &HostWindowShellData) -> bool {
    shell_mismatch(current, next).is_none()
}

fn shell_mismatch(
    current: &HostWindowShellData,
    next: &HostWindowShellData,
) -> Option<ShellContentPatchFallback> {
    if current.project_path != next.project_path {
        return Some(ShellContentPatchFallback::ShellIdentity);
    }
    if current.status_secondary != next.status_secondary
        || current.viewport_label != next.viewport_label
    {
        return Some(ShellContentPatchFallback::ShellStatus);
    }
    if current.save_project_enabled != next.save_project_enabled
        || current.undo_enabled != next.undo_enabled
        || current.redo_enabled != next.redo_enabled
    {
        return Some(ShellContentPatchFallback::ShellCommands);
    }
    if !current.preset_names.shares_values_with(&next.preset_names)
        || current.active_preset_name != next.active_preset_name
    {
        return Some(ShellContentPatchFallback::ShellPresets);
    }
    if current.skin_id != next.skin_id
        || current.panel_preset_id != next.panel_preset_id
        || current.shell_preset_id != next.shell_preset_id
        || current.window_model_preset_id != next.window_model_preset_id
    {
        return Some(ShellContentPatchFallback::ShellTheme);
    }
    if current.native_floating_window_mode != next.native_floating_window_mode
        || current.native_floating_window_id != next.native_floating_window_id
        || current.native_surface_tree_id != next.native_surface_tree_id
        || current.native_window_title != next.native_window_title
        || current.native_window_bounds != next.native_window_bounds
    {
        return Some(ShellContentPatchFallback::ShellNative);
    }
    None
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShellContentPatchFallback {
    ShellIdentity,
    ShellStatus,
    ShellCommands,
    ShellPresets,
    ShellTheme,
    ShellNative,
    Layout,
    Status,
    Document,
    DockCardinality,
    HitIndex,
}

fn record_shell_content_patch_hit() {
    zircon_runtime::profile_counter!("editor", "ui.shell_content.presentation_patch_count", 1);
}

fn record_shell_content_patch_fallback(reason: ShellContentPatchFallback) {
    zircon_runtime::profile_counter!(
        "editor",
        "ui.shell_content.presentation_patch_fallback_count",
        1
    );
    if matches!(
        reason,
        ShellContentPatchFallback::ShellIdentity
            | ShellContentPatchFallback::ShellStatus
            | ShellContentPatchFallback::ShellCommands
            | ShellContentPatchFallback::ShellPresets
            | ShellContentPatchFallback::ShellTheme
            | ShellContentPatchFallback::ShellNative
    ) {
        zircon_runtime::profile_counter!(
            "editor",
            "ui.shell_content.presentation_patch_fallback_shell_count",
            1
        );
    }
    match reason {
        ShellContentPatchFallback::ShellIdentity => {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.shell_content.presentation_patch_fallback_shell_identity_count",
                1
            );
        }
        ShellContentPatchFallback::ShellStatus => {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.shell_content.presentation_patch_fallback_shell_status_count",
                1
            );
        }
        ShellContentPatchFallback::ShellCommands => {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.shell_content.presentation_patch_fallback_shell_commands_count",
                1
            );
        }
        ShellContentPatchFallback::ShellPresets => {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.shell_content.presentation_patch_fallback_shell_presets_count",
                1
            );
        }
        ShellContentPatchFallback::ShellTheme => {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.shell_content.presentation_patch_fallback_shell_theme_count",
                1
            );
        }
        ShellContentPatchFallback::ShellNative => {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.shell_content.presentation_patch_fallback_shell_native_count",
                1
            );
        }
        ShellContentPatchFallback::Layout => {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.shell_content.presentation_patch_fallback_layout_count",
                1
            );
        }
        ShellContentPatchFallback::Status => {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.shell_content.presentation_patch_fallback_status_count",
                1
            );
        }
        ShellContentPatchFallback::Document => {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.shell_content.presentation_patch_fallback_document_count",
                1
            );
        }
        ShellContentPatchFallback::DockCardinality => {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.shell_content.presentation_patch_fallback_dock_cardinality_count",
                1
            );
        }
        ShellContentPatchFallback::HitIndex => {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.shell_content.presentation_patch_fallback_hit_index_count",
                1
            );
        }
    }
}

fn presentation_paint_models_for_target(
    presentation: &HostWindowPresentationData,
    target: HostDockSurfaceId,
) -> Vec<ModelRc<TemplatePaneNodeData>> {
    match target {
        HostDockSurfaceId::Left => side_dock_paint_models(&presentation.host_scene_data.left_dock),
        HostDockSurfaceId::Right => {
            side_dock_paint_models(&presentation.host_scene_data.right_dock)
        }
        HostDockSurfaceId::Bottom => {
            bottom_dock_paint_models(&presentation.host_scene_data.bottom_dock)
        }
    }
}

fn converted_patch_paint_models(patch: &ConvertedDockPatch) -> Vec<ModelRc<TemplatePaneNodeData>> {
    match patch {
        ConvertedDockPatch::Left(dock) | ConvertedDockPatch::Right(dock) => {
            side_dock_paint_models(dock)
        }
        ConvertedDockPatch::Bottom(dock) => bottom_dock_paint_models(dock),
    }
}

fn side_dock_paint_models(dock: &HostSideDockSurfaceData) -> Vec<ModelRc<TemplatePaneNodeData>> {
    let mut models = Vec::with_capacity(3);
    push_paint_model(&mut models, &dock.rail_nodes);
    push_paint_model(&mut models, &dock.header_nodes);
    push_pane_paint_model(&mut models, &dock.pane);
    models
}

fn bottom_dock_paint_models(
    dock: &HostBottomDockSurfaceData,
) -> Vec<ModelRc<TemplatePaneNodeData>> {
    let mut models = Vec::with_capacity(2);
    push_paint_model(&mut models, &dock.header_nodes);
    push_pane_paint_model(&mut models, &dock.pane);
    models
}

fn push_pane_paint_model(models: &mut Vec<ModelRc<TemplatePaneNodeData>>, pane: &PaneData) {
    if let Some(nodes) = pane_paint_nodes(pane) {
        push_paint_model(models, nodes);
    }
}

fn pane_paint_nodes(pane: &PaneData) -> Option<&ModelRc<TemplatePaneNodeData>> {
    if pane.template_v2.nodes.row_count() > 0 {
        return Some(&pane.template_v2.nodes);
    }
    match pane.kind.as_str() {
        "Hierarchy" => Some(&pane.hierarchy.nodes),
        "Inspector" => Some(&pane.inspector.nodes),
        "Console" => Some(&pane.console.nodes),
        "Assets" => Some(&pane.assets_activity.nodes),
        "AssetBrowser" => Some(&pane.asset_browser.nodes),
        "Welcome" => Some(&pane.welcome.nodes),
        "Project" | "UiComponentShowcase" => Some(&pane.project_overview.nodes),
        "RuntimeDiagnostics" => Some(&pane.runtime_diagnostics.nodes),
        "PerformanceTimeline" => Some(&pane.performance_timeline.nodes),
        "ModulePlugins" => Some(&pane.module_plugins.nodes),
        "BuildExport" => Some(&pane.build_export.nodes),
        "GeneratedBottom" => Some(&pane.generated_bottom.nodes),
        "UiAssetEditor" => Some(&pane.ui_asset.nodes),
        "AnimationSequenceEditor" | "AnimationGraphEditor" => Some(&pane.animation.nodes),
        _ => None,
    }
}

fn push_paint_model(
    models: &mut Vec<ModelRc<TemplatePaneNodeData>>,
    nodes: &ModelRc<TemplatePaneNodeData>,
) {
    if nodes.row_count() > 0 {
        models.push(nodes.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::{
        changed_dock_from_pane_ids, same_shell, shell_mismatch, HostDockSurfaceId,
        ShellContentPatchFallback,
    };
    use crate::ui::retained_host::host_contract::HostWindowShellData;

    #[test]
    fn one_changed_pane_selects_one_dock() {
        assert_eq!(
            changed_dock_from_pane_ids(
                "hierarchy",
                "assets",
                "inspector",
                "inspector",
                "console",
                "console"
            ),
            Some(HostDockSurfaceId::Left)
        );
    }

    #[test]
    fn no_change_or_multiple_changes_force_the_full_presentation_path() {
        assert_eq!(
            changed_dock_from_pane_ids("a", "a", "b", "b", "c", "c"),
            None
        );
        assert_eq!(
            changed_dock_from_pane_ids("a", "x", "b", "y", "c", "c"),
            None
        );
    }

    #[test]
    fn shell_content_patch_records_specific_fallback_reasons() {
        let source = include_str!("shell_content_presentation.rs");
        for counter in [
            "presentation_patch_fallback_shell_count",
            "presentation_patch_fallback_shell_identity_count",
            "presentation_patch_fallback_shell_status_count",
            "presentation_patch_fallback_shell_commands_count",
            "presentation_patch_fallback_shell_presets_count",
            "presentation_patch_fallback_shell_theme_count",
            "presentation_patch_fallback_shell_native_count",
            "presentation_patch_fallback_layout_count",
            "presentation_patch_fallback_status_count",
            "presentation_patch_fallback_document_count",
            "presentation_patch_fallback_dock_cardinality_count",
            "presentation_patch_fallback_hit_index_count",
        ] {
            assert!(
                source.contains(counter),
                "missing fallback counter {counter}"
            );
        }
    }

    #[test]
    fn shell_mismatch_reports_the_changed_ownership_group() {
        let current = HostWindowShellData::default();

        let mut next = current.clone();
        next.project_path = "project".into();
        assert_eq!(
            shell_mismatch(&current, &next),
            Some(ShellContentPatchFallback::ShellIdentity)
        );

        let mut next = current.clone();
        next.status_secondary = "Selected Node".into();
        assert_eq!(
            shell_mismatch(&current, &next),
            Some(ShellContentPatchFallback::ShellStatus)
        );

        let mut next = current.clone();
        next.save_project_enabled = true;
        assert_eq!(
            shell_mismatch(&current, &next),
            Some(ShellContentPatchFallback::ShellCommands)
        );

        let mut next = current.clone();
        next.active_preset_name = "Editing".into();
        assert_eq!(
            shell_mismatch(&current, &next),
            Some(ShellContentPatchFallback::ShellPresets)
        );

        let mut next = current.clone();
        next.skin_id = "dark".into();
        assert_eq!(
            shell_mismatch(&current, &next),
            Some(ShellContentPatchFallback::ShellTheme)
        );

        let mut next = current.clone();
        next.native_window_title = "Floating".into();
        assert_eq!(
            shell_mismatch(&current, &next),
            Some(ShellContentPatchFallback::ShellNative)
        );
    }

    #[test]
    fn native_window_minimums_or_drawer_state_do_not_invalidate_mounted_shell_content() {
        let current = HostWindowShellData::default();
        let mut next = current.clone();
        next.shell_min_width_px = 840.0;
        next.shell_min_height_px = 520.0;

        assert!(same_shell(&current, &next));

        next.left_expanded = true;
        next.drawers_visible = true;
        assert!(same_shell(&current, &next));

        next.debug_refresh_rate =
            "present 42 | pixels 4096 | slow 0 | render 0 | paint-only 42".into();
        assert!(same_shell(&current, &next));

        next.viewport_label = "Scene".into();
        assert!(!same_shell(&current, &next));
    }
}
