use toml::Value;
use zircon_runtime_interface::ui::{layout::UiSize, tree::UiTemplateNodeMetadata};

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
    fn resolve(metadata: &UiTemplateNodeMetadata) -> Option<Self> {
        let mut authored_metric = false;
        let mut metrics = Self {
            padding_left: 0.0,
            padding_right: 0.0,
            padding_top: 0.0,
            padding_bottom: 0.0,
            spacing: 0.0,
            min_width: 0.0,
            min_height: 0.0,
            icon_size: 0.0,
            leading_slot_width: 0.0,
            trailing_slot_width: 0.0,
        };
        for (key, value) in &metadata.attributes {
            let target = match key.as_str() {
                "layout_padding_left" => &mut metrics.padding_left,
                "layout_padding_right" => &mut metrics.padding_right,
                "layout_padding_top" => &mut metrics.padding_top,
                "layout_padding_bottom" => &mut metrics.padding_bottom,
                "layout_spacing" => &mut metrics.spacing,
                "layout_min_width" => &mut metrics.min_width,
                "layout_min_height" => &mut metrics.min_height,
                "layout_icon_size" => &mut metrics.icon_size,
                "layout_leading_slot_width" => &mut metrics.leading_slot_width,
                "layout_trailing_slot_width" => &mut metrics.trailing_slot_width,
                _ => continue,
            };
            authored_metric = true;
            *target = value_as_f32(value).unwrap_or(0.0).max(0.0);
        }
        authored_metric.then_some(metrics)
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
    if !supports_material_layout(metadata.component.as_str()) {
        return None;
    }

    let Some(metrics) = MaterialLayoutMetrics::resolve(metadata) else {
        return None;
    };
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
            | "Progress"
            | "ProgressBar"
            | "LinearProgress"
            | "CircularProgress"
            | "Spinner"
            | "Skeleton"
            | "Backdrop"
            | "Paper"
            | "Modal"
            | "Dialog"
            | "AlertDialog"
            | "Popover"
            | "Popper"
            | "Tooltip"
            | "Snackbar"
            | "Menu"
            | "Drawer"
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

fn value_as_f32(value: &Value) -> Option<f32> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|value| value as f64))
        .map(|value| value as f32)
}

#[cfg(test)]
#[path = "material/single_pass_tests.rs"]
mod single_pass_tests;
