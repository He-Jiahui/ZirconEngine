use zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata;

use crate::ui::retained_host::primitives::Image;

use super::super::{load_preview_image, load_preview_image_for_generation};
use super::string_attribute;

pub(crate) struct ViewTemplateVisualAssets {
    pub(crate) media_source: String,
    pub(crate) icon_name: String,
    pub(crate) preview_image: Image,
    pub(crate) has_preview_image: bool,
}

pub(crate) fn resolve_visual_assets(metadata: &UiTemplateNodeMetadata) -> ViewTemplateVisualAssets {
    resolve_visual_assets_for_generation(metadata, 0)
}

pub(super) fn resolve_visual_assets_for_generation(
    metadata: &UiTemplateNodeMetadata,
    resource_generation: u64,
) -> ViewTemplateVisualAssets {
    let media_source = string_attribute(metadata, "image")
        .or_else(|| string_attribute(metadata, "source"))
        .or_else(|| string_attribute(metadata, "media"))
        .or_else(|| {
            matches!(metadata.component.as_str(), "Image" | "SvgIcon")
                .then(|| string_attribute(metadata, "value"))
                .flatten()
        })
        .unwrap_or_default();
    let icon_name = string_attribute(metadata, "icon")
        .or_else(|| {
            (metadata.component.as_str() == "Icon")
                .then(|| string_attribute(metadata, "value"))
                .flatten()
        })
        .unwrap_or_default();
    let preview_image = if resource_generation == 0 {
        load_preview_image(&media_source, &icon_name)
    } else {
        load_preview_image_for_generation(&media_source, &icon_name, resource_generation)
    };
    let preview_size = preview_image.size();
    let has_preview_image = preview_size.width > 0 && preview_size.height > 0;

    ViewTemplateVisualAssets {
        media_source,
        icon_name,
        preview_image,
        has_preview_image,
    }
}
