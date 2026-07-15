use zircon_runtime::core::framework::project::RuntimeProfileId;
use zircon_runtime::plugin::{RuntimeProfileFeaturePreset, RUNTIME_PROFILE_FEATURE_PRESETS};

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
        ["core-min", "diagnostic-log", "platform-headless"]
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
