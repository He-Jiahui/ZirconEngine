use zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata;

const ICON_BUTTON_CLASS: &str = "workbench-icon-button";
const RAIL_BUTTON_CLASS: &str = "workbench-rail-button";
const DEFAULT_ICON_LABEL: &str = "Tool";

pub(crate) fn workbench_icon_tooltip_text(metadata: &UiTemplateNodeMetadata) -> Option<&str> {
    if let Some(tooltip) = metadata
        .attributes
        .get("tooltip")
        .and_then(toml::Value::as_str)
        .map(str::trim)
        .filter(|tooltip| !tooltip.is_empty())
    {
        return Some(tooltip);
    }

    if !metadata
        .classes
        .iter()
        .any(|class| class.as_str() == ICON_BUTTON_CLASS || class.as_str() == RAIL_BUTTON_CLASS)
    {
        return None;
    }

    let label = metadata.attributes.get("label")?.as_str()?.trim();
    (!label.is_empty() && label != DEFAULT_ICON_LABEL).then_some(label)
}
