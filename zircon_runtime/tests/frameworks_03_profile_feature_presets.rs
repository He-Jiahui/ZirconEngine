#[path = "../build.rs"]
mod runtime_profile_preset_codegen;

use zircon_runtime::builtin::BuiltinRuntimeModuleId;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::RuntimeProfileId;
use zircon_runtime::plugin::{
    PluginMaturity, RuntimeProfileDescriptor, RuntimeProfileFeaturePreset,
    RUNTIME_PROFILE_FEATURE_PRESETS,
};

fn feature_preset_for(profile_id: RuntimeProfileId) -> RuntimeProfileFeaturePreset {
    RUNTIME_PROFILE_FEATURE_PRESETS
        .iter()
        .copied()
        .find(|preset| preset.id == profile_id)
        .unwrap_or_else(|| panic!("missing feature preset for runtime profile {profile_id:?}"))
}

#[test]
fn runtime_profile_feature_presets_cover_all_builtin_profiles_in_stable_order() {
    let presets = RUNTIME_PROFILE_FEATURE_PRESETS;

    assert_eq!(
        presets.iter().map(|preset| preset.id).collect::<Vec<_>>(),
        [
            RuntimeProfileId::Minimal,
            RuntimeProfileId::Client2d,
            RuntimeProfileId::Client3d,
            RuntimeProfileId::Editor,
            RuntimeProfileId::Dev,
            RuntimeProfileId::Server,
        ]
    );
    assert_eq!(
        feature_preset_for(RuntimeProfileId::Minimal).cargo_feature,
        "core-min"
    );
    assert_eq!(
        feature_preset_for(RuntimeProfileId::Minimal).app_features,
        ["zircon_runtime/core-min"]
    );
    assert_eq!(
        feature_preset_for(RuntimeProfileId::Client2d).cargo_feature,
        "target-client"
    );
    assert_eq!(
        feature_preset_for(RuntimeProfileId::Client3d).cargo_feature,
        "target-client"
    );
    assert_eq!(
        feature_preset_for(RuntimeProfileId::Editor).cargo_feature,
        "target-editor-host"
    );
    assert_eq!(
        feature_preset_for(RuntimeProfileId::Dev).cargo_feature,
        "target-editor-host"
    );
    assert_eq!(
        feature_preset_for(RuntimeProfileId::Server).cargo_feature,
        "target-server"
    );
}

#[test]
fn runtime_profile_feature_presets_expose_compilation_requirements() {
    let client = feature_preset_for(RuntimeProfileId::Client3d);
    for feature in [
        "core-min",
        "graphics",
        "ui",
        "animation",
        "navigation",
        "script",
        "ai-contracts",
        "net-contracts",
        "physics-contracts",
        "sound-contracts",
    ] {
        assert!(
            client.runtime_features.contains(&feature),
            "Client3d should require compiled feature {feature}"
        );
    }

    let server = feature_preset_for(RuntimeProfileId::Server);
    assert_eq!(
        server.runtime_features,
        [
            "core-min",
            "diagnostic-log",
            "platform-headless",
            "dep:naga",
        ]
    );
    assert!(!server.runtime_features.contains(&"graphics"));
    assert!(!server.runtime_features.contains(&"ui"));
    assert!(!server.runtime_features.contains(&"script"));
    assert_eq!(
        server.app_features,
        [
            "zircon_runtime/target-server",
            "diagnostic-log",
            "platform-headless",
        ]
    );
}

#[test]
fn generated_runtime_profile_assembly_preserves_all_builtin_descriptor_fields() {
    struct ExpectedProfile {
        id: RuntimeProfileId,
        name: &'static str,
        target_mode: RuntimeTargetMode,
        modules: Vec<BuiltinRuntimeModuleId>,
        maturity: PluginMaturity,
        default_plugins: &'static [(&'static str, bool)],
        optional_plugins: &'static [&'static str],
        capabilities: &'static [&'static str],
    }

    let server_modules = vec![
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
    ];
    let mut client_modules = server_modules.clone();
    #[cfg(feature = "graphics")]
    client_modules.push(BuiltinRuntimeModuleId::Graphics);
    #[cfg(feature = "script")]
    client_modules.push(BuiltinRuntimeModuleId::Script);

    let expected = [
        ExpectedProfile {
            id: RuntimeProfileId::Minimal,
            name: "minimal",
            target_mode: RuntimeTargetMode::ClientRuntime,
            modules: vec![
                BuiltinRuntimeModuleId::Foundation,
                BuiltinRuntimeModuleId::Tasks,
                BuiltinRuntimeModuleId::Time,
                BuiltinRuntimeModuleId::FrameCount,
                BuiltinRuntimeModuleId::DiagnosticsCore,
            ],
            maturity: PluginMaturity::Core,
            default_plugins: &[],
            optional_plugins: &[],
            capabilities: &[
                "runtime.core.lifecycle",
                "runtime.core.tasks",
                "runtime.core.time",
                "runtime.core.frame_count",
                "runtime.core.diagnostics",
            ],
        },
        ExpectedProfile {
            id: RuntimeProfileId::Client2d,
            name: "client_2d",
            target_mode: RuntimeTargetMode::ClientRuntime,
            modules: client_modules.clone(),
            maturity: PluginMaturity::Beta,
            default_plugins: &[
                ("ui", true),
                ("sound", true),
                ("rendering", true),
                ("texture", false),
            ],
            optional_plugins: &["tilemap_2d", "particles", "animation"],
            capabilities: &[
                "runtime.core.asset",
                "runtime.core.scene",
                "runtime.core.render.base",
                "runtime.plugin.sound",
                "runtime.plugin.rendering",
            ],
        },
        ExpectedProfile {
            id: RuntimeProfileId::Client3d,
            name: "client_3d",
            target_mode: RuntimeTargetMode::ClientRuntime,
            modules: client_modules.clone(),
            maturity: PluginMaturity::Beta,
            default_plugins: &[
                ("ui", true),
                ("sound", true),
                ("rendering", true),
                ("texture", false),
            ],
            optional_plugins: &[
                "animation",
                "ai",
                "navigation",
                "particles",
                "virtual_geometry",
                "hybrid_gi",
                "solari",
            ],
            capabilities: &[
                "runtime.core.asset",
                "runtime.core.scene",
                "runtime.core.render.base",
                "runtime.plugin.sound",
                "runtime.plugin.rendering",
            ],
        },
        ExpectedProfile {
            id: RuntimeProfileId::Editor,
            name: "editor",
            target_mode: RuntimeTargetMode::EditorHost,
            modules: client_modules.clone(),
            maturity: PluginMaturity::Beta,
            default_plugins: &[
                ("ui", true),
                ("sound", true),
                ("rendering", true),
                ("texture", false),
            ],
            optional_plugins: &["animation", "navigation", "particles", "net"],
            capabilities: &["editor.host.ui_shell", "editor.host.plugin_management"],
        },
        ExpectedProfile {
            id: RuntimeProfileId::Dev,
            name: "dev",
            target_mode: RuntimeTargetMode::EditorHost,
            modules: client_modules,
            maturity: PluginMaturity::Experimental,
            default_plugins: &[
                ("ui", true),
                ("sound", true),
                ("rendering", true),
                ("texture", false),
                ("net", false),
            ],
            optional_plugins: &[
                "ai",
                "animation",
                "navigation",
                "particles",
                "virtual_geometry",
                "hybrid_gi",
                "solari",
            ],
            capabilities: &["runtime.core.diagnostics", "editor.host.plugin_management"],
        },
        ExpectedProfile {
            id: RuntimeProfileId::Server,
            name: "server",
            target_mode: RuntimeTargetMode::ServerRuntime,
            modules: server_modules,
            maturity: PluginMaturity::Beta,
            default_plugins: &[("net", false)],
            optional_plugins: &["ai", "physics", "animation", "navigation"],
            capabilities: &["runtime.core.lifecycle", "runtime.core.scene"],
        },
    ];

    let profiles = RuntimeProfileDescriptor::builtin_profiles();
    assert_eq!(profiles.len(), expected.len());
    for (profile, expected) in profiles.iter().zip(expected) {
        assert_eq!(profile, &RuntimeProfileDescriptor::for_id(expected.id));
        assert_eq!(profile.id, expected.id);
        assert_eq!(profile.name, expected.name);
        assert_eq!(profile.target_mode, expected.target_mode);
        assert_eq!(profile.builtin_modules, expected.modules);
        assert_eq!(profile.minimum_maturity, expected.maturity);
        assert_eq!(
            profile
                .default_plugins
                .iter()
                .map(|plugin| (plugin.id.as_str(), plugin.required))
                .collect::<Vec<_>>(),
            expected.default_plugins
        );
        assert_eq!(
            profile
                .optional_plugins
                .iter()
                .map(|plugin| plugin.as_str())
                .collect::<Vec<_>>(),
            expected.optional_plugins
        );
        assert_eq!(
            profile
                .required_capabilities
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            expected.capabilities
        );
        assert!(!profile.allow_externalized_required_plugins);
    }
}
