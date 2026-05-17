use std::sync::Arc;

use zircon_runtime::engine_module::EngineModule;

use super::{PluginGroup, PluginGroupBuilder, PluginGroupError};

#[derive(Clone, Copy, Debug, Default)]
pub struct MinimalPlugins;

#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultPlugins;

#[derive(Clone, Copy, Debug, Default)]
pub struct DevPlugins;

#[derive(Clone, Copy, Debug, Default)]
pub struct HeadlessPlugins;

impl PluginGroup for MinimalPlugins {
    fn build(self) -> Result<PluginGroupBuilder, PluginGroupError> {
        PluginGroupBuilder::from_modules(
            "MinimalPlugins",
            [
                Arc::new(zircon_runtime::foundation::FoundationModule) as Arc<dyn EngineModule>,
                Arc::new(zircon_runtime::core::modules::TasksModule),
                Arc::new(zircon_runtime::core::modules::TimeModule),
                Arc::new(zircon_runtime::core::modules::FrameCountModule),
                Arc::new(zircon_runtime::core::modules::DiagnosticsCoreModule),
            ],
        )
    }
}

impl PluginGroup for DefaultPlugins {
    fn build(self) -> Result<PluginGroupBuilder, PluginGroupError> {
        default_modules("DefaultPlugins", true)
    }
}

impl PluginGroup for DevPlugins {
    fn build(self) -> Result<PluginGroupBuilder, PluginGroupError> {
        default_modules("DevPlugins", true)?.add_after(
            zircon_runtime::core::modules::DIAGNOSTICS_CORE_MODULE_NAME,
            Arc::new(zircon_runtime::core::modules::LogDiagnosticsModule),
        )
    }
}

impl PluginGroup for HeadlessPlugins {
    fn build(self) -> Result<PluginGroupBuilder, PluginGroupError> {
        default_modules("HeadlessPlugins", false)
    }
}

fn default_modules(
    group_name: &'static str,
    include_graphics: bool,
) -> Result<PluginGroupBuilder, PluginGroupError> {
    let mut modules: Vec<Arc<dyn EngineModule>> = vec![
        Arc::new(zircon_runtime::foundation::FoundationModule),
        Arc::new(zircon_runtime::core::modules::LogModule),
        Arc::new(zircon_runtime::core::modules::TasksModule),
        Arc::new(zircon_runtime::core::modules::TimeModule),
        Arc::new(zircon_runtime::core::modules::FrameCountModule),
        Arc::new(zircon_runtime::core::modules::DiagnosticsCoreModule),
        Arc::new(zircon_runtime::platform::PlatformModule),
        Arc::new(zircon_runtime::input::InputModule),
        Arc::new(zircon_runtime::asset::AssetModule::default()),
        Arc::new(zircon_runtime::scene::SceneModule),
    ];
    if include_graphics {
        modules.push(Arc::new(zircon_runtime::graphics::GraphicsModule::default()));
    }
    modules.push(Arc::new(zircon_runtime::script::ScriptModule));
    #[cfg(feature = "plugin-ui")]
    if include_graphics {
        modules.push(Arc::new(zircon_runtime::ui::UiModule));
    }

    PluginGroupBuilder::from_modules(group_name, modules)
}
