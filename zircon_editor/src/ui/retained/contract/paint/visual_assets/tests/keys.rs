use super::template_image_cache_key;

#[test]
fn template_cache_key_uses_the_selected_candidate_group_identity() {
    assert_eq!(
        template_image_cache_key("preview.png", "folder-open-outline"),
        "template-image:preview.png"
    );
    assert_ne!(
        template_image_cache_key("preview-a.png", "fallback"),
        template_image_cache_key("preview-b.png", "fallback")
    );
    assert_eq!(
        template_image_cache_key("", "folder-open-outline"),
        "template-icon:folder-open-outline"
    );
}
