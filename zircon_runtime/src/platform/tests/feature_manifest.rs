fn manifest_features() -> toml::Table {
    let manifest = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"));
    let manifest = manifest
        .parse::<toml::Table>()
        .expect("zircon_runtime Cargo manifest should parse as TOML");
    manifest
        .get("features")
        .and_then(toml::Value::as_table)
        .expect("zircon_runtime Cargo manifest should declare [features]")
        .clone()
}

fn feature_entries<'a>(features: &'a toml::Table, feature: &str) -> Vec<&'a str> {
    features
        .get(feature)
        .and_then(toml::Value::as_array)
        .unwrap_or_else(|| panic!("feature `{feature}` should exist"))
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .unwrap_or_else(|| panic!("feature `{feature}` entries should be strings"))
        })
        .collect()
}

fn assert_feature_contains_all(features: &toml::Table, feature: &str, expected: &[&str]) {
    let entries = feature_entries(features, feature);
    for expected_entry in expected {
        assert!(
            entries.contains(expected_entry),
            "feature `{feature}` should contain `{expected_entry}`; entries were {entries:?}"
        );
    }
}

fn assert_feature_excludes_all(features: &toml::Table, feature: &str, forbidden: &[&str]) {
    let entries = feature_entries(features, feature);
    for forbidden_entry in forbidden {
        assert!(
            !entries.contains(forbidden_entry),
            "feature `{feature}` should not contain `{forbidden_entry}`; entries were {entries:?}"
        );
    }
}

#[test]
fn default_platform_feature_declares_window_winit_and_neutral_input_baseline() {
    let features = manifest_features();

    assert_feature_contains_all(
        &features,
        "default-platform",
        &[
            "platform-window",
            "platform-winit",
            "platform-x11",
            "platform-wayland",
            "input-mouse",
            "input-keyboard",
            "input-touch",
            "input-gamepad",
            "gamepad-gilrs",
        ],
    );
    assert_feature_excludes_all(
        &features,
        "default-platform",
        &["platform-headless", "input-gestures", "gamepad-browser"],
    );
}

#[test]
fn target_profiles_keep_client_and_editor_windowed_while_server_stays_headless() {
    let features = manifest_features();

    assert_feature_contains_all(
        &features,
        "target-client",
        &["core-min", "plugin-ui", "default-platform"],
    );
    assert_feature_contains_all(
        &features,
        "target-editor-host",
        &["core-min", "plugin-ui", "default-platform"],
    );
    assert_feature_contains_all(
        &features,
        "target-server",
        &["core-min", "platform-headless"],
    );
    assert_feature_excludes_all(
        &features,
        "target-server",
        &[
            "default-platform",
            "plugin-ui",
            "platform-window",
            "platform-winit",
            "gamepad-gilrs",
        ],
    );
}

#[test]
fn winit_platform_features_cascade_through_the_window_backend_gate() {
    let features = manifest_features();

    assert_eq!(
        feature_entries(&features, "platform-window"),
        Vec::<&str>::new()
    );
    assert_feature_contains_all(
        &features,
        "platform-winit",
        &["platform-window", "dep:winit"],
    );
    assert_feature_contains_all(&features, "platform-x11", &["platform-winit", "winit/x11"]);
    assert_feature_contains_all(
        &features,
        "platform-wayland",
        &[
            "platform-winit",
            "winit/wayland",
            "winit/wayland-dlopen",
            "winit/wayland-csd-adwaita",
        ],
    );
    assert_feature_contains_all(
        &features,
        "platform-android-game-activity",
        &["platform-winit", "winit/android-game-activity"],
    );
    assert_feature_contains_all(
        &features,
        "platform-android-native-activity",
        &["platform-winit", "winit/android-native-activity"],
    );
    assert_feature_contains_all(
        &features,
        "platform-web",
        &["platform-window", "platform-winit"],
    );
}

#[test]
fn gamepad_backend_features_stay_separated_from_neutral_gamepad_input() {
    let features = manifest_features();

    assert_eq!(
        feature_entries(&features, "input-gamepad"),
        Vec::<&str>::new()
    );
    assert_feature_contains_all(&features, "gamepad-gilrs", &["input-gamepad"]);
    assert_feature_contains_all(&features, "gamepad-browser", &["input-gamepad"]);
    assert_feature_excludes_all(&features, "gamepad-gilrs", &["gamepad-browser"]);
    assert_feature_excludes_all(&features, "gamepad-browser", &["gamepad-gilrs"]);
}

#[test]
fn input_source_features_remain_independent_feature_gates() {
    let features = manifest_features();

    for feature in [
        "input-mouse",
        "input-keyboard",
        "input-touch",
        "input-gestures",
    ] {
        assert_eq!(feature_entries(&features, feature), Vec::<&str>::new());
    }
}
