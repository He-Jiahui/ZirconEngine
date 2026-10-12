//! 将可能同时成立的输入状态收敛为一个反馈优先级，再投影颜色与强度。
//! 键盘可见焦点由共享选择器判定，避免鼠标获得焦点时无意增强状态层。

use super::super::super::data::TemplatePaneNodeData;
use super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::super::style_selector::focus_visible_for_node;
use super::super::template_style::is_button_disabled;

const MATERIAL_STATE_LAYER_OPACITY_HOVER: f32 = 0.08;
const MATERIAL_STATE_LAYER_OPACITY_FOCUS: f32 = 0.10;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const MATERIAL_STATE_LAYER_OPACITY_PRESS: f32 = 0.10;
const MATERIAL_STATE_LAYER_OPACITY_DRAG: f32 = 0.16;

/// Owns the retained Material state-layer priority before opacity projection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MaterialStateLayerResolvedState {
    Disabled,
    DropTarget,
    Pressed,
    Dragging,
    Focused,
    Selected,
    Hovered,
}

impl MaterialStateLayerResolvedState {
    // 禁用与拖放目标优先于按压，选中优先于悬停；颜色和透明度必须消费同一次序。
    fn resolve(node: &TemplatePaneNodeData) -> Option<Self> {
        if !node.state_layer_enabled {
            None
        } else if is_button_disabled(node) {
            Some(Self::Disabled)
        } else if node.drop_hovered || node.active_drag_target {
            Some(Self::DropTarget)
        } else if node.pressed || node.enter_pressed {
            Some(Self::Pressed)
        } else if node.dragging {
            Some(Self::Dragging)
        } else if focus_visible_for_node(node) {
            Some(Self::Focused)
        } else if node.selected || node.checked {
            Some(Self::Selected)
        } else if node.hovered {
            Some(Self::Hovered)
        } else {
            None
        }
    }

    const fn opacity(self) -> f32 {
        match self {
            Self::Disabled | Self::Focused | Self::Selected => MATERIAL_STATE_LAYER_OPACITY_FOCUS,
            Self::Pressed => MATERIAL_STATE_LAYER_OPACITY_PRESS,
            Self::DropTarget | Self::Dragging => MATERIAL_STATE_LAYER_OPACITY_DRAG,
            Self::Hovered => MATERIAL_STATE_LAYER_OPACITY_HOVER,
        }
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn state_layer_opacity(
    node: &TemplatePaneNodeData,
) -> Option<f32> {
    MaterialStateLayerResolvedState::resolve(node).map(MaterialStateLayerResolvedState::opacity)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn state_layer_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    state_layer_color_from_host(node, current_host_palette())
}

fn state_layer_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    if node.state_layer_color.a > 0 {
        [
            node.state_layer_color.r,
            node.state_layer_color.g,
            node.state_layer_color.b,
            node.state_layer_color.a,
        ]
    } else {
        match MaterialStateLayerResolvedState::resolve(node) {
            Some(MaterialStateLayerResolvedState::Disabled) => palette.text_disabled,
            Some(MaterialStateLayerResolvedState::DropTarget) => palette.accent,
            Some(MaterialStateLayerResolvedState::Pressed)
            | Some(MaterialStateLayerResolvedState::Dragging)
            | Some(MaterialStateLayerResolvedState::Hovered)
            | None => palette.text,
            Some(MaterialStateLayerResolvedState::Focused) => palette.focus_ring,
            Some(MaterialStateLayerResolvedState::Selected) => palette.surface_selected,
        }
    }
}

#[cfg(test)]
#[path = "tests/state.rs"]
mod tests;
