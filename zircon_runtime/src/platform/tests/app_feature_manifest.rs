fn app_manifest() -> toml::Table {
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../zircon_app/Cargo.toml"
    ))
    .parse::<toml::Table>()
    .expect("zircon_app Cargo manifest should parse as TOML")
}

fn manifest_features(manifest: &toml::Table) -> &toml::Table {
    manifest
        .get("features")
        .and_then(toml::Value::as_table)
        .expect("zircon_app manifest should declare [features]")
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

fn dependency<'a>(manifest: &'a toml::Table, name: &str) -> &'a toml::Table {
    manifest
        .get("dependencies")
        .and_then(toml::Value::as_table)
        .and_then(|dependencies| dependencies.get(name))
        .and_then(toml::Value::as_table)
        .unwrap_or_else(|| panic!("dependency `{name}` should be a table"))
}

fn dependency_bool(manifest: &toml::Table, name: &str, key: &str) -> bool {
    dependency(manifest, name)
        .get(key)
        .and_then(toml::Value::as_bool)
        .unwrap_or_else(|| panic!("dependency `{name}` should declare `{key}` as bool"))
}

#[test]
fn app_default_platform_forwards_runtime_default_and_desktop_host_features() {
    let manifest = app_manifest();
    let features = manifest_features(&manifest);

    assert_feature_contains_all(
        features,
        "default-platform",
        &[
            "zircon_runtime/default-platform",
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
        features,
        "default-platform",
        &["platform-headless", "gamepad-browser"],
    );
}

#[test]
fn app_target_profiles_keep_client_and_editor_windowed_while_server_stays_headless() {
    let manifest = app_manifest();
    let features = manifest_features(&manifest);

    assert_eq!(feature_entries(features, "default"), vec!["target-client"]);
    assert_feature_contains_all(
        features,
        "target-client",
        &[
            "zircon_runtime/target-client",
            "default-platform",
            "plugin-ui",
        ],
    );
    assert_feature_contains_all(
        features,
        "target-editor-host",
        &[
            "zircon_runtime/target-editor-host",
            "dep:zircon_editor",
            "default-platform",
        ],
    );
    assert_feature_contains_all(
        features,
        "target-server",
        &["zircon_runtime/target-server", "platform-headless"],
    );
    assert_feature_excludes_all(
        features,
        "target-server",
        &[
            "default-platform",
            "plugin-ui",
            "platform-winit",
            "gamepad-gilrs",
        ],
    );
}

#[test]
fn app_winit_features_forward_native_protocols_to_runtime_and_host_crate() {
    let manifest = app_manifest();
    let features = manifest_features(&manifest);

    assert_feature_contains_all(
        features,
        "platform-winit",
        &[
            "platform-window",
            "zircon_runtime/platform-winit",
            "dep:winit",
        ],
    );
    assert_feature_contains_all(
        features,
        "platform-x11",
        &["platform-winit", "zircon_runtime/platform-x11", "winit/x11"],
    );
    assert_feature_contains_all(
        features,
        "platform-wayland",
        &[
            "platform-winit",
            "zircon_runtime/platform-wayland",
            "winit/wayland",
            "winit/wayland-dlopen",
            "winit/wayland-csd-adwaita",
        ],
    );
    assert_feature_contains_all(
        features,
        "platform-android-game-activity",
        &[
            "platform-winit",
            "zircon_runtime/platform-android-game-activity",
            "winit/android-game-activity",
        ],
    );
    assert_feature_contains_all(
        features,
        "platform-android-native-activity",
        &[
            "platform-winit",
            "zircon_runtime/platform-android-native-activity",
            "winit/android-native-activity",
        ],
    );
}

#[test]
fn app_gamepad_features_keep_gilrs_and_browser_backends_separate() {
    let manifest = app_manifest();
    let features = manifest_features(&manifest);

    assert_feature_contains_all(features, "input-gamepad", &["zircon_runtime/input-gamepad"]);
    assert_feature_contains_all(
        features,
        "gamepad-gilrs",
        &["zircon_runtime/gamepad-gilrs", "dep:gilrs"],
    );
    assert_feature_contains_all(
        features,
        "gamepad-browser",
        &["zircon_runtime/gamepad-browser", "input-gamepad"],
    );
    assert_feature_excludes_all(features, "gamepad-gilrs", &["gamepad-browser"]);
    assert_feature_excludes_all(features, "gamepad-browser", &["gamepad-gilrs", "dep:gilrs"]);
}

#[test]
fn app_host_backend_dependencies_stay_optional_and_feature_owned() {
    let manifest = app_manifest();

    assert!(dependency_bool(&manifest, "winit", "optional"));
    assert!(!dependency_bool(&manifest, "winit", "default-features"));
    assert!(dependency_bool(&manifest, "softbuffer", "optional"));
    assert!(dependency_bool(&manifest, "gilrs", "optional"));
    assert!(!dependency_bool(
        &manifest,
        "zircon_runtime",
        "default-features"
    ));
}
