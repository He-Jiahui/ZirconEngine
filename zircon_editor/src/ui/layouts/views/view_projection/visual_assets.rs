use zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata;

use crate::ui::retained_host::primitives::Image;

use super::string_attribute;

pub(crate) struct ViewTemplateVisualAssets {
    pub(crate) media_source: String,
    pub(crate) icon_name: String,
    pub(crate) preview_image: Image,
    pub(crate) has_preview_image: bool,
}

pub(crate) fn resolve_visual_assets(metadata: &UiTemplateNodeMetadata) -> ViewTemplateVisualAssets {
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
    let has_preview_image = !media_source.trim().is_empty() || !icon_name.trim().is_empty();

    ViewTemplateVisualAssets {
        media_source,
        icon_name,
        preview_image: Image::default(),
        has_preview_image,
    }
}
