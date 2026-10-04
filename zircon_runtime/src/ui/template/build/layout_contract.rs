use toml::Value;

use zircon_runtime_interface::ui::layout::{
    Anchor, BoxConstraints, LayoutBoundary, Pivot, Position, UiAxis, UiContainerKind, UiMargin,
    UiScrollableBoxConfig,
};
use zircon_runtime_interface::ui::template::UiTemplateNode;
use zircon_runtime_interface::ui::tree::UiInputPolicy;

use super::build_error::UiTemplateBuildError;
use super::parsers::{
    parse_axis_constraint, parse_bool, parse_container, parse_f32, parse_i32, parse_input_policy,
    parse_layout_boundary, parse_point,
};

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct TemplateLayoutContract {
    pub(super) constraints: BoxConstraints,
    pub(super) anchor: Anchor,
    pub(super) pivot: Pivot,
    pub(super) position: Position,
    pub(super) container: Option<UiContainerKind>,
    pub(super) padding: UiMargin,
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
        // Padding is a node-owned content inset. Keep slot padding on the
        // parent/child edge so the two policies cannot be applied twice.
        padding: parse_margin(layout.self_value("padding"), path, "padding")?,
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

fn parse_margin(
    value: Option<&Value>,
    node_path: &str,
    field: &str,
) -> Result<UiMargin, UiTemplateBuildError> {
    let Some(value) = value else {
        return Ok(UiMargin::default());
    };
    let table = value
        .as_table()
        .ok_or_else(|| UiTemplateBuildError::InvalidLayoutContract {
            node_path: node_path.to_string(),
            detail: format!("{field} must be a table"),
        })?;
    Ok(UiMargin::new(
        parse_f32(table.get("left")).unwrap_or(0.0),
        parse_f32(table.get("top")).unwrap_or(0.0),
        parse_f32(table.get("right")).unwrap_or(0.0),
        parse_f32(table.get("bottom")).unwrap_or(0.0),
    ))
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
) -> Option<LayeredLayoutTable<'_>> {
    let self_layout = node.attributes.get("layout").and_then(Value::as_table);
    let slot_layout = node.slot_attributes.get("layout").and_then(Value::as_table);
    if self_layout.is_none() && slot_layout.is_none() {
        return None;
    }
    Some(LayeredLayoutTable::new(
        self_layout,
        slot_layout,
        parent_container,
    ))
}

#[derive(Clone, Copy)]
struct LayeredLayoutTable<'a> {
    self_layout: Option<&'a toml::map::Map<String, Value>>,
    slot_layout: Option<&'a toml::map::Map<String, Value>>,
    restored_axis: Option<&'static str>,
}

impl<'a> LayeredLayoutTable<'a> {
    fn new(
        self_layout: Option<&'a toml::map::Map<String, Value>>,
        slot_layout: Option<&'a toml::map::Map<String, Value>>,
        parent_container: Option<UiContainerKind>,
    ) -> Self {
        let restored_axis = match parent_container {
            Some(UiContainerKind::HorizontalBox(_))
            | Some(UiContainerKind::WrapBox(_))
            | Some(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                axis: UiAxis::Horizontal,
                ..
            })) => Some("width"),
            Some(UiContainerKind::VerticalBox(_))
            | Some(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                axis: UiAxis::Vertical,
                ..
            })) => Some("height"),
            _ => None,
        };
        Self {
            self_layout,
            slot_layout,
            restored_axis,
        }
    }

    // 已识别流向的主轴优先采用节点自身尺寸，缺失时回退到挂载边；普通字段由挂载边覆盖，借用视图不复制表。
    fn get(&self, key: &str) -> Option<&'a Value> {
        if self.restored_axis == Some(key) {
            if let Some(value) = self.self_layout.and_then(|layout| layout.get(key)) {
                return Some(value);
            }
        }
        self.slot_layout
            .and_then(|layout| layout.get(key))
            .or_else(|| self.self_layout.and_then(|layout| layout.get(key)))
    }

    fn self_value(&self, key: &str) -> Option<&'a Value> {
        self.self_layout.and_then(|layout| layout.get(key))
    }
}

#[cfg(test)]
#[path = "tests/layout_contract.rs"]
mod tests;
