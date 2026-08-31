use toml::Value;
use zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ChromeKind {
    Shell,
    ActivityRail,
    Toolbar,
    StatusBar,
    Panel,
    Viewport,
}

pub(super) fn chrome_kind(metadata: &UiTemplateNodeMetadata) -> Option<ChromeKind> {
    match metadata.component.as_str() {
        "WorkbenchShell" | "Shell" | "WorkbenchWindow" => Some(ChromeKind::Shell),
        "ActivityRail" | "ActivityRailPanel" => Some(ChromeKind::ActivityRail),
        "TopToolbar" | "Toolbar" | "MenuBar" | "WorkbenchMenuBar" => Some(ChromeKind::Toolbar),
        "StatusBar" | "BottomStatusBar" => Some(ChromeKind::StatusBar),
        "SceneTreePanel" | "InspectorPanel" | "Panel" | "DockPanel" | "ToolWindowStack" => {
            Some(ChromeKind::Panel)
        }
        "ViewportPanel" | "Viewport" | "SceneViewport" | "DocumentViewport" => {
            Some(ChromeKind::Viewport)
        }
        _ => match control_id(metadata) {
            Some(id) if id.contains("ActivityRail") => Some(ChromeKind::ActivityRail),
            Some(id) if id.contains("Toolbar") || id.contains("MenuBar") => {
                Some(ChromeKind::Toolbar)
            }
            Some(id) if id.contains("StatusBar") => Some(ChromeKind::StatusBar),
            Some(id) if id.contains("Viewport") => Some(ChromeKind::Viewport),
            Some(id) if id.contains("Panel") || id.contains("Dock") => Some(ChromeKind::Panel),
            Some(id) if id.contains("WorkbenchShell") || id.contains("WorkbenchWindow") => {
                Some(ChromeKind::Shell)
            }
            _ => None,
        },
    }
}

pub(super) fn chrome_label(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    ["label", "title", "text", "value_text"]
        .iter()
        .find_map(|key| string_attribute(metadata, key))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

pub(super) fn chrome_icon(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    string_attribute(metadata, "icon")
        .or_else(|| string_attribute(metadata, "leading_icon"))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn control_id(metadata: &UiTemplateNodeMetadata) -> Option<&str> {
    metadata.control_id.as_deref()
}

pub(super) fn color_attribute<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    key: &str,
) -> Option<&'a str> {
    metadata
        .style_overrides
        .get(key)
        .or_else(|| metadata.attributes.get(key))
        .and_then(Value::as_str)
        .filter(|color| !color.trim().is_empty())
}

pub(super) fn string_attribute<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    key: &str,
) -> Option<&'a str> {
    metadata.attributes.get(key).and_then(Value::as_str)
}

pub(super) fn number_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<f32> {
    metadata
        .style_overrides
        .get(key)
        .or_else(|| metadata.attributes.get(key))
        .and_then(value_as_f32)
}

pub(super) fn metric_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<f32> {
    number_attribute(metadata, key).filter(|value| value.is_finite())
}

fn value_as_f32(value: &Value) -> Option<f32> {
    let value = match value {
        Value::Integer(value) => *value as f64,
        Value::Float(value) if value.is_finite() => *value,
        _ => return None,
    } as f32;
    value.is_finite().then_some(value)
}
