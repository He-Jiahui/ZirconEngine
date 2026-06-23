use super::*;
use std::collections::BTreeSet;
use zircon_runtime::core::framework::sound::{
    AUDIO_LISTENER_COMPONENT_TYPE, AUDIO_SOURCE_COMPONENT_TYPE, AUDIO_VOLUME_COMPONENT_TYPE,
};

#[test]
fn sound_editor_plugin_contributes_authoring_extensions() {
    let registration = plugin_registration();

    assert!(registration.is_success(), "{:?}", registration.diagnostics);
    assert!(registration
        .capabilities
        .contains(&SOUND_AUTHORING_CAPABILITY.to_string()));
    assert!(registration
        .extensions
        .views()
        .iter()
        .any(|view| view.id() == SOUND_AUTHORING_VIEW_ID));
    assert!(registration
        .extensions
        .views()
        .iter()
        .any(|view| view.id() == SOUND_ACOUSTIC_DEBUG_VIEW_ID));
    assert!(registration
        .extensions
        .drawers()
        .iter()
        .any(|drawer| drawer.id() == SOUND_DRAWER_ID));
    assert!(registration
        .extensions
        .ui_templates()
        .iter()
        .any(|template| template.id() == SOUND_TEMPLATE_ID));
    assert!(registration
        .extensions
        .ui_templates()
        .iter()
        .any(|template| template.id() == SOUND_ACOUSTIC_DEBUG_TEMPLATE_ID));
    assert!(registration
        .extensions
        .component_drawers()
        .iter()
        .any(|drawer| {
            drawer.component_type() == AUDIO_SOURCE_COMPONENT_TYPE
                && drawer.controller() == SOUND_AUDIO_SOURCE_DRAWER_ID
                && drawer
                    .bindings()
                    .iter()
                    .map(String::as_str)
                    .eq(SOUND_AUDIO_SOURCE_OPERATION_PATHS.iter().copied())
        }));
    assert!(registration
        .extensions
        .component_drawers()
        .iter()
        .any(|drawer| {
            drawer.component_type() == AUDIO_LISTENER_COMPONENT_TYPE
                && drawer.controller() == SOUND_AUDIO_LISTENER_DRAWER_ID
                && drawer
                    .bindings()
                    .iter()
                    .map(String::as_str)
                    .eq(SOUND_AUDIO_LISTENER_OPERATION_PATHS.iter().copied())
        }));
    assert!(registration
        .extensions
        .component_drawers()
        .iter()
        .any(|drawer| {
            drawer.component_type() == AUDIO_VOLUME_COMPONENT_TYPE
                && drawer.controller() == SOUND_AUDIO_VOLUME_DRAWER_ID
                && drawer
                    .bindings()
                    .iter()
                    .map(String::as_str)
                    .eq(SOUND_AUDIO_VOLUME_OPERATION_PATHS.iter().copied())
        }));
    assert!(registration
        .extensions
        .menu_items()
        .iter()
        .any(|menu| menu.operation().as_str() == "View.sound.mixer_console.Open"));
    assert!(registration
        .extensions
        .menu_items()
        .iter()
        .any(|menu| menu.operation().as_str() == "View.sound.acoustic_debug.Open"));
    assert!(registration
        .extensions
        .operations()
        .descriptors()
        .any(|operation| operation.path().as_str() == "View.sound.mixer_console.Open"));
    for path in SOUND_MIXER_OPERATION_PATHS
        .iter()
        .chain(SOUND_AUDIO_SOURCE_OPERATION_PATHS)
        .chain(SOUND_AUDIO_LISTENER_OPERATION_PATHS)
        .chain(SOUND_AUDIO_VOLUME_OPERATION_PATHS)
    {
        let operation = registration
            .extensions
            .operations()
            .descriptors()
            .find(|operation| operation.path().as_str() == *path)
            .unwrap_or_else(|| panic!("missing sound editor operation {path}"));
        assert!(operation
            .payload_schema_id()
            .is_some_and(|schema| schema.starts_with("sound.")));
    }
    let create_track = registration
        .extensions
        .operations()
        .descriptors()
        .find(|operation| operation.path().as_str() == "Sound.Mixer.Track.Create")
        .expect("create track operation");
    assert!(create_track.undoable().is_some());
}

#[test]
fn sound_editor_ui_template_routes_are_registered_operations() {
    let registration = plugin_registration();
    let registered_operations = registration
        .extensions
        .operations()
        .descriptors()
        .map(|operation| operation.path().as_str().to_string())
        .collect::<BTreeSet<_>>();
    let routes = ui_template_routes(&[
        (
            "mixer_console.v2.ui.toml",
            include_str!("../mixer_console.v2.ui.toml"),
        ),
        (
            "acoustic_debug.v2.ui.toml",
            include_str!("../acoustic_debug.v2.ui.toml"),
        ),
        (
            "audio_source.drawer.v2.ui.toml",
            include_str!("../audio_source.drawer.v2.ui.toml"),
        ),
        (
            "audio_listener.drawer.v2.ui.toml",
            include_str!("../audio_listener.drawer.v2.ui.toml"),
        ),
        (
            "audio_volume.drawer.v2.ui.toml",
            include_str!("../audio_volume.drawer.v2.ui.toml"),
        ),
    ]);

    assert!(
        !routes.is_empty(),
        "sound editor templates should expose at least one routed control"
    );
    for (template, route) in routes {
        assert!(
            registered_operations.contains(&route),
            "sound editor template {template} routes to unregistered operation {route}"
        );
    }
}

#[test]
fn sound_editor_ui_template_asset_ids_match_registered_surfaces() {
    assert_template_asset_id(
        "mixer_console.v2.ui.toml",
        include_str!("../mixer_console.v2.ui.toml"),
        SOUND_TEMPLATE_ID,
    );
    assert_template_asset_id(
        "acoustic_debug.v2.ui.toml",
        include_str!("../acoustic_debug.v2.ui.toml"),
        SOUND_ACOUSTIC_DEBUG_TEMPLATE_ID,
    );
    assert_template_asset_id(
        "audio_source.drawer.v2.ui.toml",
        include_str!("../audio_source.drawer.v2.ui.toml"),
        "sound.audio_source.drawer",
    );
    assert_template_asset_id(
        "audio_listener.drawer.v2.ui.toml",
        include_str!("../audio_listener.drawer.v2.ui.toml"),
        "sound.audio_listener.drawer",
    );
    assert_template_asset_id(
        "audio_volume.drawer.v2.ui.toml",
        include_str!("../audio_volume.drawer.v2.ui.toml"),
        "sound.audio_volume.drawer",
    );
}

fn ui_template_routes(templates: &[(&'static str, &'static str)]) -> Vec<(&'static str, String)> {
    let mut routes = Vec::new();
    for (template, source) in templates {
        let mut remaining = *source;
        while let Some(index) = remaining.find("route = \"") {
            let route_start = index + "route = \"".len();
            let route_source = &remaining[route_start..];
            let route_end = route_source
                .find('"')
                .expect("sound editor template route should close string");
            routes.push((*template, route_source[..route_end].to_string()));
            remaining = &route_source[route_end..];
        }
    }
    routes
}

fn assert_template_asset_id(template: &str, source: &str, expected: &str) {
    assert_eq!(
        template_asset_id(source).as_deref(),
        Some(expected),
        "sound editor template {template} should keep its asset id aligned"
    );
}

fn template_asset_id(source: &str) -> Option<String> {
    source.lines().map(str::trim).find_map(|line| {
        line.strip_prefix("id = \"")
            .and_then(|value| value.strip_suffix('"'))
            .map(str::to_string)
    })
}
