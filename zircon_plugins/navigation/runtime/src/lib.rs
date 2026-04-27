use std::sync::Arc;

use zircon_runtime::core::{
    ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject, StartupMode,
};
use zircon_runtime::engine_module::{factory, qualified_name};

pub const PLUGIN_ID: &str = "navigation";
pub const NAVIGATION_MODULE_NAME: &str = "NavigationModule";
pub const NAVIGATION_MANAGER_NAME: &str = "NavigationModule.Manager.NavigationManager";

#[derive(Clone, Debug, PartialEq)]
pub struct NavigationPath {
    pub points: Vec<[f32; 3]>,
    pub length: f32,
}

#[derive(Clone, Debug, Default)]
pub struct DefaultNavigationManager;

impl DefaultNavigationManager {
    pub fn straight_path(&self, from: [f32; 3], to: [f32; 3]) -> NavigationPath {
        let delta = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
        let length = (delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2]).sqrt();
        NavigationPath {
            points: vec![from, to],
            length,
        }
    }
}

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        NAVIGATION_MODULE_NAME,
        "Navigation path query and nav data runtime plugin",
    )
    .with_manager(ManagerDescriptor::new(
        qualified_name(
            NAVIGATION_MODULE_NAME,
            ServiceKind::Manager,
            "NavigationManager",
        ),
        StartupMode::Lazy,
        Vec::new(),
        factory(|_| Ok(Arc::new(DefaultNavigationManager) as ServiceObject)),
    ))
}

#[derive(Clone, Debug)]
pub struct NavigationRuntimePlugin {
    descriptor: zircon_runtime::RuntimePluginDescriptor,
}

impl NavigationRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl zircon_runtime::RuntimePlugin for NavigationRuntimePlugin {
    fn descriptor(&self) -> &zircon_runtime::RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register_runtime_extensions(
        &self,
        registry: &mut zircon_runtime::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::RuntimeExtensionRegistryError> {
        registry.register_module(module_descriptor())
    }
}

pub fn runtime_plugin_descriptor() -> zircon_runtime::RuntimePluginDescriptor {
    zircon_runtime::RuntimePluginDescriptor::new(
        PLUGIN_ID,
        "Navigation",
        zircon_runtime::RuntimePluginId::Navigation,
        "zircon_plugin_navigation_runtime",
    )
    .with_target_modes([
        zircon_runtime::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::RuntimeTargetMode::ServerRuntime,
        zircon_runtime::RuntimeTargetMode::EditorHost,
    ])
    .with_capability("runtime.plugin.navigation")
}

pub fn runtime_plugin() -> NavigationRuntimePlugin {
    NavigationRuntimePlugin::new()
}

pub fn package_manifest() -> zircon_runtime::PluginPackageManifest {
    zircon_runtime::RuntimePlugin::package_manifest(&runtime_plugin())
}

pub fn runtime_selection() -> zircon_runtime::ProjectPluginSelection {
    zircon_runtime::RuntimePlugin::project_selection(&runtime_plugin())
}

pub fn plugin_registration() -> zircon_runtime::RuntimePluginRegistrationReport {
    zircon_runtime::RuntimePluginRegistrationReport::from_plugin(&runtime_plugin())
}

pub fn runtime_capabilities() -> &'static [&'static str] {
    &["runtime.plugin.navigation"]
}

#[cfg(test)]
mod tests {
    use zircon_runtime::core::CoreRuntime;

    use super::*;

    #[test]
    fn navigation_registration_contributes_runtime_module() {
        let report = plugin_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report
            .extensions
            .modules()
            .iter()
            .any(|module| module.name == NAVIGATION_MODULE_NAME));
        assert_eq!(
            report.package_manifest.modules[0].target_modes,
            vec![
                zircon_runtime::RuntimeTargetMode::ClientRuntime,
                zircon_runtime::RuntimeTargetMode::ServerRuntime,
                zircon_runtime::RuntimeTargetMode::EditorHost,
            ]
        );
    }

    #[test]
    fn navigation_module_resolves_manager_and_builds_straight_path() {
        let runtime = CoreRuntime::new();
        runtime.register_module(module_descriptor()).unwrap();
        runtime.activate_module(NAVIGATION_MODULE_NAME).unwrap();
        let manager = runtime
            .handle()
            .resolve_manager::<DefaultNavigationManager>(NAVIGATION_MANAGER_NAME)
            .unwrap();

        let path = manager.straight_path([0.0, 0.0, 0.0], [3.0, 4.0, 0.0]);

        assert_eq!(path.points.len(), 2);
        assert_eq!(path.length, 5.0);
    }
}
