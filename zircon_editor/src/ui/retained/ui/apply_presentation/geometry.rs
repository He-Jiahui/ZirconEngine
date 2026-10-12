use super::*;

/// Publishes a resize frame from the last stable shell product. The semantic
/// pane payloads remain owned by the current host generation; only geometry and
/// hit regions are rebuilt for the new extent.
pub(crate) fn apply_window_metrics_geometry_presentation(
    ui: &UiHostWindow,
    cached: &ShellPresentation,
    model: &WorkbenchViewModel,
    geometry: &WorkbenchShellGeometry,
    componentized_workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames,
    root_template_projection: Option<&RetainedUiHostProjection>,
    workbench_window_projection: Option<&RetainedUiHostProjection>,
    workbench_geometry_patch_indices: Option<&[usize]>,
    template_scale_factor: f32,
    floating_window_projection_bundle: &FloatingWindowProjectionBundle,
) -> bool {
    zircon_runtime::profile_scope!(
        "editor",
        "retained_host",
        "apply_window_metrics_geometry_presentation"
    );
    zircon_runtime::profile_counter!(
        "editor",
        "ui.window_metrics.geometry_fast_path_attempt_count",
        1
    );

    let mut host_shell = cached.host_shell.clone();
    host_shell.shell_min_width_px = geometry.window_min_width;
    host_shell.shell_min_height_px = geometry.window_min_height;
    host_shell.viewport_label = model.status_bar.viewport_label.clone().into();
    host_shell.status_secondary = model
        .status_bar
        .secondary_text
        .clone()
        .unwrap_or_default()
        .into();
    let host_layout = host_window_layout(componentized_workbench_layout_frames);
    let Some(retained_scene_data) = cached.retained_scene_data.as_deref() else {
        record_geometry_fast_path_fallback();
        return false;
    };
    let host_scene_data = host_window::build_host_scene_geometry(
        retained_scene_data,
        &cached.host_surface_data,
        &host_shell,
        &host_layout,
        &cached.status_primary,
        floating_window_projection_bundle,
    );
    let host_welcome_pane = {
        let welcome_pane = project_welcome_pane(&cached.welcome.pane, &host_scene_data);
        to_host_contract_welcome_pane(&welcome_pane, &cached.welcome.recent_projects)
    };
    let native_floating_surface = build_native_floating_surface_data(&host_scene_data, &host_shell);
    let current_generation = ui.get_host_presentation_generation();
    let current_structure = current_generation.structure();
    let host_scene_data =
        scene_conversion::to_host_contract_host_scene_geometry_with_retained_panes(
            &host_scene_data,
            &current_structure.host_scene_data,
        );
    let Some(workbench_window_projection) = workbench_window_projection else {
        record_geometry_fast_path_fallback();
        return false;
    };
    let Some(workbench_geometry_patch_indices) = workbench_geometry_patch_indices else {
        record_geometry_fast_path_fallback();
        return false;
    };
    let Some(workbench_patch) =
        build_host_contract_workbench_window_geometry_patch_at_mount_and_scale(
            workbench_window_projection,
            workbench_geometry_patch_indices,
            &current_structure.workbench_window_nodes,
            componentized_workbench_layout_frames.mount_frame,
            template_scale_factor,
        )
    else {
        record_geometry_fast_path_fallback();
        return false;
    };
    let root_template_nodes = root_template_projection
        .map(|projection| {
            to_host_contract_root_template_overlay_nodes_at_scale(
                Some(projection),
                template_scale_factor,
            )
        })
        .unwrap_or_else(|| current_structure.root_template_nodes.clone());
    let window_size = ui.window().size();
    let native_floating_surface_data =
        scene_conversion::to_host_contract_native_floating_surface_geometry_with_retained_panes(
            &native_floating_surface,
            &current_structure.native_floating_surface_data,
        );
    let geometry_presentation = HostWindowGeometryPresentationData {
        host_scene_data,
        host_shell: to_host_contract_host_shell(&host_shell),
        host_layout: to_host_contract_host_window_layout(&host_layout),
        asset_deletion_blocker: current_structure
            .asset_deletion_blocker
            .relayout(window_size.width as f32, window_size.height as f32),
        root_template_nodes,
        workbench_window_nodes: workbench_patch.nodes,
        native_floating_surface_data,
    };
    drop(current_generation);
    if !ui.set_host_geometry_presentation(geometry_presentation, &workbench_patch.changed_rows) {
        record_geometry_fast_path_fallback();
        return false;
    }
    zircon_runtime::profile_counter!("editor", "ui.window_metrics.geometry_fast_path_count", 1_u8);
    ui.global::<host_contract::PaneSurfaceHostContext>()
        .set_welcome_pane(host_welcome_pane);
    true
}

fn record_geometry_fast_path_fallback() {
    zircon_runtime::profile_counter!(
        "editor",
        "ui.window_metrics.geometry_fast_path_fallback_count",
        1_u8
    );
}
