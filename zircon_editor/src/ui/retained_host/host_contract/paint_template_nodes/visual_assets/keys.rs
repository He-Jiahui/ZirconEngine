use zircon_runtime_interface::ui::surface::UiVisualAssetRef;

use crate::ui::retained_host::primitives::image_content_fingerprint;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn visual_asset_cache_key(
    asset: &UiVisualAssetRef,
) -> String {
    match asset {
        UiVisualAssetRef::Icon(icon_name) => format!("icon:{icon_name}"),
        UiVisualAssetRef::Image(source) => format!("image:{source}"),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn template_image_cache_key(
    source: &str,
    icon_name: &str,
) -> String {
    if !source.trim().is_empty() {
        return format!("template-image:{source}");
    }
    format!("template-icon:{icon_name}")
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn retained_image_resource_key(
    width: u32,
    height: u32,
    rgba: &[u8],
) -> String {
    retained_image_resource_key_from_fingerprint(
        width,
        height,
        image_content_fingerprint(width, height, rgba),
    )
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn retained_image_resource_key_from_fingerprint(
    width: u32,
    height: u32,
    content_fingerprint: u64,
) -> String {
    format!("retained-image:{width}x{height}:{content_fingerprint:016x}")
}

#[cfg(test)]
mod tests {
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
}
