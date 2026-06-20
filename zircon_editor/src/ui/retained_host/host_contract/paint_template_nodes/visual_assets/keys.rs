use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use zircon_runtime_interface::ui::surface::UiVisualAssetRef;

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
    if !icon_name.is_empty() {
        return format!("template-icon:{icon_name}");
    }
    format!("template-image:{source}")
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn retained_image_resource_key(
    width: u32,
    height: u32,
    rgba: &[u8],
) -> String {
    let mut hasher = DefaultHasher::new();
    width.hash(&mut hasher);
    height.hash(&mut hasher);
    rgba.hash(&mut hasher);
    format!("retained-image:{width}x{height}:{:016x}", hasher.finish())
}
