use zircon_runtime::plugin::{RuntimeProfileId, RUNTIME_PROFILE_FEATURE_PRESETS};

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
        RuntimeProfileId::Minimal.feature_preset().cargo_feature,
        "core-min"
    );
    assert_eq!(
        RuntimeProfileId::Minimal.feature_preset().app_features,
        ["zircon_runtime/core-min"]
    );
    assert_eq!(
        RuntimeProfileId::Client2d.feature_preset().cargo_feature,
        "target-client"
    );
    assert_eq!(
        RuntimeProfileId::Client3d.feature_preset().cargo_feature,
        "target-client"
    );
    assert_eq!(
        RuntimeProfileId::Editor.feature_preset().cargo_feature,
        "target-editor-host"
    );
    assert_eq!(
        RuntimeProfileId::Dev.feature_preset().cargo_feature,
        "target-editor-host"
    );
    assert_eq!(
        RuntimeProfileId::Server.feature_preset().cargo_feature,
        "target-server"
    );
}

#[test]
fn runtime_profile_feature_presets_expose_compilation_requirements() {
    let client = RuntimeProfileId::Client3d.feature_preset();
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

    let server = RuntimeProfileId::Server.feature_preset();
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
