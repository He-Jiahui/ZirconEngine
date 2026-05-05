use toml::Value;

use zircon_runtime_interface::ui::layout::{
    Anchor, BoxConstraints, LayoutBoundary, Pivot, Position, UiAxis, UiContainerKind,
    UiScrollableBoxConfig,
};
use zircon_runtime_interface::ui::template::UiTemplateNode;
use zircon_runtime_interface::ui::tree::UiInputPolicy;

use super::build_error::UiTemplateBuildError;
use super::parsers::{
    parse_axis_constraint, parse_bool, parse_container, parse_i32, parse_input_policy,
    parse_layout_boundary, parse_point,
};

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct TemplateLayoutContract {
    pub(super) constraints: BoxConstraints,
    pub(super) anchor: Anchor,
    pub(super) pivot: Pivot,
    pub(super) position: Position,
    pub(super) container: Option<UiContainerKind>,
    pub(super) input_policy: Option<UiInputPolicy>,
    pub(super) clip_to_bounds: bool,
    pub(super) layout_boundary: LayoutBoundary,
    pub(super) stretch_width: bool,
    pub(super) stretch_height: bool,
    pub(super) z_index: i32,
}

pub(super) fn infer_layout_contract(
    node: &UiTemplateNode,
    path: &str,
    parent_container: Option<UiContainerKind>,
) -> Result<TemplateLayoutContract, UiTemplateBuildError> {
    let Some(layout) = merged_layout_table(node, parent_container) else {
        return Ok(TemplateLayoutContract::default());
    };

    Ok(TemplateLayoutContract {
        constraints: BoxConstraints {
            width: parse_axis_constraint(layout.get("width"), path, "width")?,
            height: parse_axis_constraint(layout.get("height"), path, "height")?,
        },
        anchor: parse_point(layout.get("anchor"), path, "anchor")?
            .map(|(x, y)| Anchor::new(x, y))
            .unwrap_or_default(),
        pivot: parse_point(layout.get("pivot"), path, "pivot")?
            .map(|(x, y)| Pivot::new(x, y))
            .unwrap_or_default(),
        position: parse_point(layout.get("position"), path, "position")?
            .map(|(x, y)| Position::new(x, y))
            .unwrap_or_default(),
        container: parse_container(layout.get("container"), path)?,
        input_policy: parse_input_policy(layout.get("input_policy"), path)?,
        clip_to_bounds: parse_bool(layout.get("clip"))
            .or_else(|| parse_bool(layout.get("clip_to_bounds")))
            .unwrap_or(false),
        layout_boundary: parse_layout_boundary(layout.get("boundary"), path)?.unwrap_or_default(),
        stretch_width: is_explicit_stretch_axis(layout.get("width")),
        stretch_height: is_explicit_stretch_axis(layout.get("height")),
        z_index: parse_i32(layout.get("z_index"), path, "z_index")?.unwrap_or_default(),
    })
}

fn is_explicit_stretch_axis(value: Option<&Value>) -> bool {
    value
        .and_then(Value::as_table)
        .and_then(|table| table.get("stretch"))
        .and_then(Value::as_str)
        == Some("Stretch")
}

fn merged_layout_table(
    node: &UiTemplateNode,
    parent_container: Option<UiContainerKind>,
) -> Option<toml::map::Map<String, Value>> {
    let self_layout = node.attributes.get("layout").and_then(Value::as_table);
    let slot_layout = node.slot_attributes.get("layout").and_then(Value::as_table);
    match (self_layout, slot_layout) {
        (None, None) => None,
        (Some(layout), None) | (None, Some(layout)) => Some(layout.clone()),
        (Some(self_layout), Some(slot_layout)) => {
            let mut merged = self_layout.clone();
            for (key, value) in slot_layout {
                let _ = merged.insert(key.clone(), value.clone());
            }
            match parent_container {
                Some(UiContainerKind::HorizontalBox(_))
                | Some(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                    axis: UiAxis::Horizontal,
                    ..
                })) => {
                    restore_axis(&mut merged, self_layout, "width");
                }
                Some(UiContainerKind::VerticalBox(_))
                | Some(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                    axis: UiAxis::Vertical,
                    ..
                })) => {
                    restore_axis(&mut merged, self_layout, "height");
                }
                _ => {}
            }
            Some(merged)
        }
    }
}

fn restore_axis(
    target: &mut toml::map::Map<String, Value>,
    source: &toml::map::Map<String, Value>,
    axis: &str,
) {
    if let Some(value) = source.get(axis) {
        let _ = target.insert(axis.to_string(), value.clone());
    }
}
