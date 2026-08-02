use crate::plugin::PluginMaturity;
use crate::{
    builtin::{BuiltinRuntimeModuleId, RuntimePluginId},
    core::framework::{platform::RuntimeTargetMode, project::RuntimeProfileId},
};

use super::descriptor::RuntimeProfileDescriptor;

impl RuntimeProfileDescriptor {
    pub fn for_id(id: RuntimeProfileId) -> Self {
        match id {
            RuntimeProfileId::Minimal => Self::minimal(),
            RuntimeProfileId::Client2d => Self::client_2d(),
            RuntimeProfileId::Client3d => Self::client_3d(),
            RuntimeProfileId::Editor => Self::editor(),
            RuntimeProfileId::Dev => Self::dev(),
            RuntimeProfileId::Server => Self::server(),
        }
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
        .with_builtin_modules([
            BuiltinRuntimeModuleId::Foundation,
            BuiltinRuntimeModuleId::Tasks,
            BuiltinRuntimeModuleId::Time,
            BuiltinRuntimeModuleId::FrameCount,
            BuiltinRuntimeModuleId::DiagnosticsCore,
        ])
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
        .with_builtin_modules(client_runtime_builtin_modules())
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
        .with_builtin_modules(client_runtime_builtin_modules())
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
        .with_builtin_modules(client_runtime_builtin_modules())
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
            .with_builtin_modules(client_runtime_builtin_modules())
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
        .with_builtin_modules(server_runtime_builtin_modules())
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

fn server_runtime_builtin_modules() -> Vec<BuiltinRuntimeModuleId> {
    vec![
        BuiltinRuntimeModuleId::Foundation,
        BuiltinRuntimeModuleId::Log,
        BuiltinRuntimeModuleId::Tasks,
        BuiltinRuntimeModuleId::Time,
        BuiltinRuntimeModuleId::FrameCount,
        BuiltinRuntimeModuleId::DiagnosticsCore,
        BuiltinRuntimeModuleId::Platform,
        BuiltinRuntimeModuleId::Input,
        BuiltinRuntimeModuleId::Asset,
        BuiltinRuntimeModuleId::Scene,
    ]
}

fn client_runtime_builtin_modules() -> Vec<BuiltinRuntimeModuleId> {
    let mut modules = server_runtime_builtin_modules();
    #[cfg(feature = "graphics")]
    modules.push(BuiltinRuntimeModuleId::Graphics);
    #[cfg(feature = "script")]
    modules.push(BuiltinRuntimeModuleId::Script);
    modules
}
