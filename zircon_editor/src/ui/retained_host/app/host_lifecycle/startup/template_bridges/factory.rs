use std::error::Error;
use std::sync::Arc;

use super::super::super::super::*;
use super::bundle::StartupTemplateBridges;

pub(in crate::ui::retained_host::app::host_lifecycle::startup) fn create_startup_template_bridges(
    shell_size: ShellSizePx,
) -> Result<StartupTemplateBridges, Box<dyn Error>> {
    let builtin_template_runtime = {
        zircon_runtime::profile_scope!(
            "editor",
            "retained_host",
            "new_load_shared_builtin_templates"
        );
        Arc::new(callback_dispatch::load_startup_builtin_template_runtime()?)
    };
    let template_size = UiSize::new(shell_size.width, shell_size.height);
    let template_bridge = {
        zircon_runtime::profile_scope!("editor", "retained_host", "new_template_bridge");
        callback_dispatch::BuiltinHostWindowTemplateBridge::new_with_runtime(
            builtin_template_runtime.clone(),
            template_size,
        )?
    };
    let workbench_window_bridge = {
        zircon_runtime::profile_scope!("editor", "retained_host", "new_workbench_window_bridge");
        callback_dispatch::BuiltinWorkbenchWindowTemplateSurfaceBridge::new_with_runtime(
            builtin_template_runtime.clone(),
            template_size,
        )?
    };
    let floating_window_source_bridge = {
        zircon_runtime::profile_scope!(
            "editor",
            "retained_host",
            "new_floating_window_source_bridge"
        );
        callback_dispatch::BuiltinFloatingWindowSourceTemplateBridge::new_with_runtime(
            builtin_template_runtime.as_ref(),
            template_size,
        )?
    };
    let viewport_toolbar_bridge = {
        zircon_runtime::profile_scope!("editor", "retained_host", "new_viewport_toolbar_bridge");
        callback_dispatch::BuiltinViewportToolbarTemplateBridge::new_with_runtime(
            builtin_template_runtime.clone(),
        )?
    };
    let inspector_surface_bridge = {
        zircon_runtime::profile_scope!("editor", "retained_host", "new_inspector_surface_bridge");
        callback_dispatch::BuiltinInspectorSurfaceTemplateBridge::new_with_runtime(
            builtin_template_runtime.as_ref(),
        )?
    };
    let pane_surface_bridge = {
        zircon_runtime::profile_scope!("editor", "retained_host", "new_pane_surface_bridge");
        callback_dispatch::BuiltinPaneSurfaceTemplateBridge::new_with_runtime(
            builtin_template_runtime.as_ref(),
        )?
    };
    let component_showcase_runtime = {
        zircon_runtime::profile_scope!("editor", "retained_host", "new_component_runtime");
        EditorUiHostRuntime::default()
    };

    Ok(StartupTemplateBridges {
        builtin_template_runtime,
        template_bridge,
        workbench_window_bridge,
        floating_window_source_bridge,
        viewport_toolbar_bridge,
        inspector_surface_bridge,
        pane_surface_bridge,
        component_showcase_runtime,
    })
}
