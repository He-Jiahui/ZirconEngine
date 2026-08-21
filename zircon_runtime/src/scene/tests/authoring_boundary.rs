// Keep the tables narrow and explicit: these names describe editor authoring
// state that must stay out of runtime-owned scene/project serialization.
pub(super) const SERIALIZED_AUTHORING_TOKENS: &[&str] = &[
    "active_camera_override",
    "camera_override",
    "display_mode",
    "gizmo",
    "grid_mode",
    "overlay",
    "pane",
    "preview_lighting",
    "preview_skybox",
    "scene_gizmos",
    "SceneViewportSettings",
    "SceneViewportTool",
    "selected",
    "selection",
    "selection_anchors",
    "transform_space",
    "view_orientation",
    "viewport_camera",
    "ViewportCameraSnapshot",
];

pub(super) const SOURCE_AUTHORING_TOKENS: &[&str] = &[
    "active_camera_override",
    "camera_override",
    "display_mode",
    "GridMode",
    "GridOverlayExtract",
    "HandleOverlayExtract",
    "preview_lighting",
    "preview_skybox",
    "RenderOverlayExtract",
    "SceneGizmoKind",
    "SceneGizmoOverlayExtract",
    "scene_gizmos",
    "SceneViewportSettings",
    "SceneViewportTool",
    "selected",
    "selected_entity",
    "selected_node",
    "selection",
    "SelectionAnchorExtract",
    "selection_anchors",
    "set_selected",
    "TransformSpace",
    "ViewOrientation",
    "ViewportCameraSnapshot",
];

pub(super) const SERIALIZED_AUTHORING_SURFACES: &[&str] =
    &["versioned dynamic scene JSON", "versioned reflected JSON"];

// `schema_id` identifies a neutral wire contract; it is not editor authoring state.
pub(super) const SERIALIZATION_HEADER_TOKEN_EXEMPTIONS: &[&str] = &["schema_id"];

pub(super) fn assert_text_excludes_authoring_tokens(label: &str, text: &str, tokens: &[&str]) {
    if let Some(token) = first_authoring_token(text, tokens) {
        panic!("{label} must not contain editor authoring token {token}");
    }
}

pub(super) fn first_authoring_token<'a>(text: &str, tokens: &'a [&str]) -> Option<&'a str> {
    tokens.iter().copied().find(|token| text.contains(token))
}

#[test]
fn authoring_boundary_guard_fails_on_representative_tokens() {
    let probe = "selection overlay SceneViewportTool";
    for token in ["selection", "overlay", "SceneViewportTool"] {
        assert_eq!(first_authoring_token(probe, &[token]), Some(token));
    }
}

#[test]
fn authoring_token_tables_stay_sorted_and_deduplicated() {
    assert_sorted_and_deduplicated("serialized", SERIALIZED_AUTHORING_TOKENS);
    assert_sorted_and_deduplicated("source", SOURCE_AUTHORING_TOKENS);
    assert_sorted_and_deduplicated("serialized surfaces", SERIALIZED_AUTHORING_SURFACES);
    assert_sorted_and_deduplicated(
        "serialization header exemptions",
        SERIALIZATION_HEADER_TOKEN_EXEMPTIONS,
    );
    assert!(SERIALIZED_AUTHORING_SURFACES.contains(&"versioned dynamic scene JSON"));
    assert!(SERIALIZED_AUTHORING_SURFACES.contains(&"versioned reflected JSON"));
    assert!(SERIALIZATION_HEADER_TOKEN_EXEMPTIONS.contains(&"schema_id"));
}

fn assert_sorted_and_deduplicated(label: &str, tokens: &[&str]) {
    let mut seen = std::collections::BTreeSet::new();
    let mut previous: Option<&str> = None;

    for token in tokens {
        assert!(
            seen.insert(*token),
            "{label} authoring token table contains duplicate token {token}"
        );

        if let Some(previous) = previous {
            assert!(
                token_order_key(token) >= token_order_key(previous),
                "{label} authoring token table must stay sorted: {previous} should not appear before {token}"
            );
        }

        previous = Some(token);
    }
}

fn token_order_key(token: &str) -> String {
    token
        .chars()
        .filter(|character| *character != '_')
        .collect::<String>()
        .to_ascii_lowercase()
}
