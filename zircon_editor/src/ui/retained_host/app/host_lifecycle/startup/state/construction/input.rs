use std::sync::Arc;

use super::super::super::super::super::*;
use super::super::super::resources::StartupManagers;
use super::super::super::template_bridges::StartupTemplateBridges;

pub(in crate::ui::retained_host::app::host_lifecycle::startup) struct StartupHostConstruction {
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) ui: UiHostWindow,
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) runtime:
        EditorHostEventController,
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) startup_managers:
        StartupManagers,
    #[cfg(feature = "profiling")]
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) runtime_gateway:
        SharedEditorRuntimeGateway,
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) native_plugin_live_host:
        Arc<zircon_runtime::plugin::native::NativePluginLiveHost>,
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) viewport:
        RetainedViewportController,
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) startup_session:
        EditorStartupSessionDocument,
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) viewport_size: UVec2,
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) shell_size: ShellSizePx,
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) shell_scale_factor: f32,
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) template_bridges:
        StartupTemplateBridges,
}
