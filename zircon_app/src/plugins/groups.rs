mod resolution;

use zircon_runtime::core::framework::project::RuntimeProfileId;

use super::{PluginGroup, PluginGroupBuilder, PluginGroupError};
use resolution::{resolve_builtin_plugin_group, BuiltinPluginGroupFeature};

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
        resolve_builtin_plugin_group("MinimalPlugins", RuntimeProfileId::Minimal, [])
    }
}

impl PluginGroup for DefaultPlugins {
    fn build(self) -> Result<PluginGroupBuilder, PluginGroupError> {
        resolve_builtin_plugin_group(
            "DefaultPlugins",
            RuntimeProfileId::Client3d,
            [BuiltinPluginGroupFeature::Ui],
        )
    }
}

impl PluginGroup for DevPlugins {
    fn build(self) -> Result<PluginGroupBuilder, PluginGroupError> {
        resolve_builtin_plugin_group(
            "DevPlugins",
            RuntimeProfileId::Dev,
            [
                BuiltinPluginGroupFeature::Ui,
                BuiltinPluginGroupFeature::LogDiagnostics,
            ],
        )
    }
}

impl PluginGroup for HeadlessPlugins {
    fn build(self) -> Result<PluginGroupBuilder, PluginGroupError> {
        resolve_builtin_plugin_group("HeadlessPlugins", RuntimeProfileId::Server, [])
    }
}
