use super::template_source_is_vector;

#[test]
fn primary_media_source_wins_over_the_icon_fallback_kind() {
    assert!(!template_source_is_vector(
        "asset://previews/material.png",
        "folder-open-outline",
        false
    ));
    assert!(template_source_is_vector(
        "asset://previews/material.svg",
        "fallback.png",
        false
    ));
}

#[test]
fn svg_role_hint_and_semantic_icon_names_preserve_vector_identity() {
    assert!(template_source_is_vector("", "save", false));
    assert!(template_source_is_vector("preview.png", "", true));
    assert!(!template_source_is_vector("", "preview.png", false));
}
