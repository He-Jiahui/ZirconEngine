use std::error::Error;
use std::sync::Arc;

use super::super::super::super::*;
use super::bundle::StartupTemplateBridges;
use crate::core::i18n::EditorI18nService;

// 启动时共享内建模板 Runtime，再按 shell 装载尺寸建立各表面桥；后续重算复用这些桥的已挂载状态。
pub(in crate::ui::retained_host::app::host_lifecycle::startup) fn create_startup_template_bridges(
    shell_size: ShellSizePx,
    i18n: Arc<EditorI18nService>,
) -> Result<StartupTemplateBridges, Box<dyn Error + Send + Sync>> {
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
        let mount_frame = template_bridge
            .root_shell_frames()
            .componentized_workbench_mount_frame(template_size);
        callback_dispatch::BuiltinWorkbenchWindowTemplateSurfaceBridge::new_mounted_with_runtime_and_i18n(
            builtin_template_runtime.clone(),
            mount_frame,
            i18n,
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
