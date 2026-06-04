use crate::{plugin::PluginMaturity, RuntimePluginId, RuntimeTargetMode};

use super::descriptor::{RuntimeProfileDescriptor, RuntimeProfileId};

impl RuntimeProfileDescriptor {
    pub fn for_id(id: RuntimeProfileId) -> Self {
        Self::builtin_profiles()
            .into_iter()
            .find(|profile| profile.id == id)
            .unwrap_or_else(|| panic!("missing built-in runtime profile {id:?}"))
    }

    pub fn builtin_profiles() -> Vec<Self> {
        vec![
            Self::minimal(),
            Self::client_2d(),
            Self::client_3d(),
            Self::editor(),
            Self::dev(),
            Self::server(),
        ]
    }

    fn minimal() -> Self {
        Self::new(
            RuntimeProfileId::Minimal,
            "minimal",
            RuntimeTargetMode::ClientRuntime,
        )
        .with_minimum_maturity(PluginMaturity::Core)
        .with_required_capability("runtime.core.lifecycle")
        .with_required_capability("runtime.core.tasks")
        .with_required_capability("runtime.core.time")
        .with_required_capability("runtime.core.frame_count")
        .with_required_capability("runtime.core.diagnostics")
    }

    fn client_2d() -> Self {
        Self::new(
            RuntimeProfileId::Client2d,
            "client_2d",
            RuntimeTargetMode::ClientRuntime,
        )
        .with_minimum_maturity(PluginMaturity::Beta)
        .with_default_plugin(RuntimePluginId::Ui, true)
        .with_default_plugin(RuntimePluginId::Sound, true)
        .with_default_plugin(RuntimePluginId::Rendering, true)
        .with_default_plugin(RuntimePluginId::Texture, false)
        .with_optional_plugin(RuntimePluginId::Tilemap2d)
        .with_optional_plugin(RuntimePluginId::Particles)
        .with_optional_plugin(RuntimePluginId::Animation)
        .with_required_capability("runtime.core.asset")
        .with_required_capability("runtime.core.scene")
        .with_required_capability("runtime.core.render.base")
        .with_required_capability("runtime.plugin.sound")
        .with_required_capability("runtime.plugin.rendering")
    }

    fn client_3d() -> Self {
        Self::new(
            RuntimeProfileId::Client3d,
            "client_3d",
            RuntimeTargetMode::ClientRuntime,
        )
        .with_minimum_maturity(PluginMaturity::Beta)
        .with_default_plugin(RuntimePluginId::Ui, true)
        .with_default_plugin(RuntimePluginId::Sound, true)
        .with_default_plugin(RuntimePluginId::Rendering, true)
        .with_default_plugin(RuntimePluginId::Texture, false)
        .with_optional_plugin(RuntimePluginId::Animation)
        .with_optional_plugin(RuntimePluginId::Ai)
        .with_optional_plugin(RuntimePluginId::Navigation)
        .with_optional_plugin(RuntimePluginId::Particles)
        .with_optional_plugin(RuntimePluginId::VirtualGeometry)
        .with_optional_plugin(RuntimePluginId::HybridGi)
        .with_optional_plugin(RuntimePluginId::Solari)
        .with_required_capability("runtime.core.asset")
        .with_required_capability("runtime.core.scene")
        .with_required_capability("runtime.core.render.base")
        .with_required_capability("runtime.plugin.sound")
        .with_required_capability("runtime.plugin.rendering")
    }

    fn editor() -> Self {
        Self::new(
            RuntimeProfileId::Editor,
            "editor",
            RuntimeTargetMode::EditorHost,
        )
        .with_minimum_maturity(PluginMaturity::Beta)
        .with_default_plugin(RuntimePluginId::Ui, true)
        .with_default_plugin(RuntimePluginId::Sound, true)
        .with_default_plugin(RuntimePluginId::Rendering, true)
        .with_default_plugin(RuntimePluginId::Texture, false)
        .with_optional_plugin(RuntimePluginId::Animation)
        .with_optional_plugin(RuntimePluginId::Navigation)
        .with_optional_plugin(RuntimePluginId::Particles)
        .with_optional_plugin(RuntimePluginId::Net)
        .with_required_capability("editor.host.ui_shell")
        .with_required_capability("editor.host.plugin_management")
    }

    fn dev() -> Self {
        Self::new(RuntimeProfileId::Dev, "dev", RuntimeTargetMode::EditorHost)
            .with_minimum_maturity(PluginMaturity::Experimental)
            .with_default_plugin(RuntimePluginId::Ui, true)
            .with_default_plugin(RuntimePluginId::Sound, true)
            .with_default_plugin(RuntimePluginId::Rendering, true)
            .with_default_plugin(RuntimePluginId::Texture, false)
            .with_default_plugin(RuntimePluginId::Net, false)
            .with_optional_plugin(RuntimePluginId::Ai)
            .with_optional_plugin(RuntimePluginId::Animation)
            .with_optional_plugin(RuntimePluginId::Navigation)
            .with_optional_plugin(RuntimePluginId::Particles)
            .with_optional_plugin(RuntimePluginId::VirtualGeometry)
            .with_optional_plugin(RuntimePluginId::HybridGi)
            .with_optional_plugin(RuntimePluginId::Solari)
            .with_required_capability("runtime.core.diagnostics")
            .with_required_capability("editor.host.plugin_management")
    }

    fn server() -> Self {
        Self::new(
            RuntimeProfileId::Server,
            "server",
            RuntimeTargetMode::ServerRuntime,
        )
        .with_minimum_maturity(PluginMaturity::Beta)
        .with_default_plugin(RuntimePluginId::Net, false)
        .with_optional_plugin(RuntimePluginId::Ai)
        .with_optional_plugin(RuntimePluginId::Physics)
        .with_optional_plugin(RuntimePluginId::Animation)
        .with_optional_plugin(RuntimePluginId::Navigation)
        .with_required_capability("runtime.core.lifecycle")
        .with_required_capability("runtime.core.scene")
    }
}
