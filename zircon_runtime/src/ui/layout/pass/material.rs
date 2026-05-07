use toml::Value;
use zircon_runtime_interface::ui::{layout::UiSize, tree::UiTemplateNodeMetadata};

const LAYOUT_METRIC_KEYS: &[&str] = &[
    "layout_padding_left",
    "layout_padding_right",
    "layout_padding_top",
    "layout_padding_bottom",
    "layout_spacing",
    "layout_min_width",
    "layout_min_height",
    "layout_icon_size",
    "layout_leading_slot_width",
    "layout_trailing_slot_width",
];

pub(super) struct MaterialLayoutMetrics {
    pub padding_left: f32,
    pub padding_right: f32,
    pub padding_top: f32,
    pub padding_bottom: f32,
    pub spacing: f32,
    pub min_width: f32,
    pub min_height: f32,
    pub icon_size: f32,
    pub leading_slot_width: f32,
    pub trailing_slot_width: f32,
}

impl MaterialLayoutMetrics {
    fn resolve(metadata: &UiTemplateNodeMetadata) -> Self {
        Self {
            padding_left: number_attr(metadata, "layout_padding_left")
                .unwrap_or(0.0)
                .max(0.0),
            padding_right: number_attr(metadata, "layout_padding_right")
                .unwrap_or(0.0)
                .max(0.0),
            padding_top: number_attr(metadata, "layout_padding_top")
                .unwrap_or(0.0)
                .max(0.0),
            padding_bottom: number_attr(metadata, "layout_padding_bottom")
                .unwrap_or(0.0)
                .max(0.0),
            spacing: number_attr(metadata, "layout_spacing")
                .unwrap_or(0.0)
                .max(0.0),
            min_width: number_attr(metadata, "layout_min_width")
                .unwrap_or(0.0)
                .max(0.0),
            min_height: number_attr(metadata, "layout_min_height")
                .unwrap_or(0.0)
                .max(0.0),
            icon_size: number_attr(metadata, "layout_icon_size")
                .unwrap_or(0.0)
                .max(0.0),
            leading_slot_width: number_attr(metadata, "layout_leading_slot_width")
                .unwrap_or(0.0)
                .max(0.0),
            trailing_slot_width: number_attr(metadata, "layout_trailing_slot_width")
                .unwrap_or(0.0)
                .max(0.0),
        }
    }

    fn has_layout_attribute(metadata: &UiTemplateNodeMetadata) -> bool {
        LAYOUT_METRIC_KEYS
            .iter()
            .any(|key| metadata.attributes.contains_key(*key))
    }

    fn apply_to_content(&self, content: UiSize, has_icon: bool) -> UiSize {
        let icon_size = if has_icon { self.icon_size } else { 0.0 };
        let icon_text_spacing = if has_icon && content.width > 0.0 {
            self.spacing
        } else {
            0.0
        };
        let width = content.width
            + icon_size
            + icon_text_spacing
            + self.leading_slot_width
            + self.trailing_slot_width
            + self.padding_left
            + self.padding_right;
        let height = content.height.max(icon_size) + self.padding_top + self.padding_bottom;
        UiSize::new(width.max(self.min_width), height.max(self.min_height))
    }
}

pub(super) fn measure_material_content(
    metadata: Option<&UiTemplateNodeMetadata>,
    content: UiSize,
) -> Option<UiSize> {
    let metadata = metadata?;
    if !MaterialLayoutMetrics::has_layout_attribute(metadata)
        || !supports_material_layout(metadata.component.as_str())
    {
        return None;
    }

    let metrics = MaterialLayoutMetrics::resolve(metadata);
    let has_icon = has_icon_attribute(metadata)
        || (metadata.component == "IconButton" && metrics.icon_size > 0.0);
    Some(metrics.apply_to_content(content, has_icon))
}

fn supports_material_layout(component: &str) -> bool {
    matches!(
        component,
        "Button"
            | "IconButton"
            | "ToggleButton"
            | "Checkbox"
            | "InputField"
            | "TextField"
            | "ListRow"
            | "ComboBox"
            | "RangeField"
            | "NumberField"
            | "ProgressBar"
            | "Spinner"
            | "Switch"
            | "ContextActionMenu"
            | "MenuItem"
            | "Tab"
            | "TableRow"
            | "VirtualList"
            | "ColorField"
            | "Vector2Field"
            | "Vector3Field"
            | "Vector4Field"
            | "Label"
    )
}

fn has_icon_attribute(metadata: &UiTemplateNodeMetadata) -> bool {
    ["icon", "image", "media", "source"].iter().any(|key| {
        metadata
            .attributes
            .get(*key)
            .and_then(|value| value.as_str())
            .is_some_and(|value| !value.is_empty())
    })
}

fn number_attr(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<f32> {
    metadata.attributes.get(key).and_then(value_as_f32)
}

fn value_as_f32(value: &Value) -> Option<f32> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|value| value as f64))
        .map(|value| value as f32)
}
