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
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
    use crate::ui::retained_host::primitives::Color;

    macro_rules! state_node {
        ($($field:ident),* $(,)?) => {
            TemplatePaneNodeData {
                state_layer_enabled: true,
                $($field: true,)*
                ..TemplatePaneNodeData::default()
            }
        };
    }

    #[test]
    fn idle_state_layer_fallback_projects_neutral_content_color() {
        let mut palette = PALETTE;
        palette.text = [10, 11, 12, 255];
        let node = TemplatePaneNodeData::default();

        assert_eq!(
            state_layer_color_from_host(&node, palette),
            [10, 11, 12, 255]
        );
    }

    #[test]
    fn state_layer_declared_color_overrides_palette_when_available() {
        let palette = PALETTE;
        let mut node = TemplatePaneNodeData::default();
        node.state_layer_color = Color::from_argb_u8(128, 20, 21, 22);

        assert_eq!(
            state_layer_color_from_host(&node, palette),
            [20, 21, 22, 128]
        );
    }

    #[test]
    fn material_state_layer_resolves_exact_interaction_priority() {
        let mut gated_off = state_node!(
            disabled,
            pressed,
            enter_pressed,
            dragging,
            focused,
            selected,
            checked,
            hovered,
            drop_hovered,
            active_drag_target,
        );
        gated_off.state_layer_enabled = false;

        let cases = [
            (
                "default state layer has no overlay",
                TemplatePaneNodeData::default(),
                None,
                None,
            ),
            (
                "disabled state layer suppresses every interaction",
                gated_off,
                None,
                None,
            ),
            ("enabled idle state has no layer", state_node!(), None, None),
            (
                "disabled wins over every interaction",
                state_node!(
                    disabled,
                    pressed,
                    enter_pressed,
                    dragging,
                    focused,
                    selected,
                    checked,
                    hovered,
                    drop_hovered,
                    active_drag_target,
                ),
                Some(MaterialStateLayerResolvedState::Disabled),
                Some(MATERIAL_STATE_LAYER_OPACITY_FOCUS),
            ),
            (
                "drop target wins over press drag focus and hover",
                state_node!(drop_hovered, pressed, dragging, focused, selected, hovered,),
                Some(MaterialStateLayerResolvedState::DropTarget),
                Some(MATERIAL_STATE_LAYER_OPACITY_DRAG),
            ),
            (
                "pressed wins over drag focus and hover",
                state_node!(pressed, dragging, focused, hovered),
                Some(MaterialStateLayerResolvedState::Pressed),
                Some(MATERIAL_STATE_LAYER_OPACITY_PRESS),
            ),
            (
                "dragging wins over focus selection checked and hover",
                state_node!(dragging, focused, selected, checked, hovered),
                Some(MaterialStateLayerResolvedState::Dragging),
                Some(MATERIAL_STATE_LAYER_OPACITY_DRAG),
            ),
            (
                "focused wins over selection and hover",
                state_node!(focused, selected, checked, hovered),
                Some(MaterialStateLayerResolvedState::Focused),
                Some(MATERIAL_STATE_LAYER_OPACITY_FOCUS),
            ),
            (
                "selected wins over hover",
                state_node!(selected, hovered),
                Some(MaterialStateLayerResolvedState::Selected),
                Some(MATERIAL_STATE_LAYER_OPACITY_FOCUS),
            ),
            (
                "hovered resolves hover",
                state_node!(hovered),
                Some(MaterialStateLayerResolvedState::Hovered),
                Some(MATERIAL_STATE_LAYER_OPACITY_HOVER),
            ),
        ];

        for (label, node, expected_state, expected_opacity) in cases {
            assert_eq!(
                MaterialStateLayerResolvedState::resolve(&node),
                expected_state,
                "{label}"
            );
            assert_eq!(state_layer_opacity(&node), expected_opacity, "{label}");
        }
    }

    #[test]
    fn state_layer_resolves_interaction_aliases() {
        let cases = [
            (
                state_node!(enter_pressed, dragging, focused, hovered),
                MaterialStateLayerResolvedState::Pressed,
            ),
            (
                state_node!(selected, hovered),
                MaterialStateLayerResolvedState::Selected,
            ),
            (
                state_node!(checked, hovered),
                MaterialStateLayerResolvedState::Selected,
            ),
            (
                state_node!(drop_hovered),
                MaterialStateLayerResolvedState::DropTarget,
            ),
            (
                state_node!(active_drag_target),
                MaterialStateLayerResolvedState::DropTarget,
            ),
        ];

        for (node, expected) in cases {
            assert_eq!(
                MaterialStateLayerResolvedState::resolve(&node),
                Some(expected)
            );
        }
    }

    #[test]
    fn selection_uses_neutral_surface_while_visible_focus_uses_focus_ring() {
        let mut palette = PALETTE;
        palette.surface_selected = [10, 11, 12, 255];
        palette.focus_ring = [20, 21, 22, 255];
        let selected = state_node!(selected);
        let focused = state_node!(focused, selected);
        let pointer_focused_selection = TemplatePaneNodeData {
            state_layer_enabled: true,
            focused: true,
            focus_visible: false,
            focus_visible_known: true,
            selected: true,
            ..TemplatePaneNodeData::default()
        };

        assert_eq!(
            state_layer_color_from_host(&selected, palette),
            palette.surface_selected
        );
        assert_eq!(
            state_layer_color_from_host(&focused, palette),
            palette.focus_ring
        );
        assert_eq!(
            MaterialStateLayerResolvedState::resolve(&pointer_focused_selection),
            Some(MaterialStateLayerResolvedState::Selected)
        );
        assert_eq!(
            state_layer_color_from_host(&pointer_focused_selection, palette),
            palette.surface_selected
        );
    }

    #[test]
    fn interaction_layers_use_neutral_content_or_accent_without_focus_color() {
        let mut palette = PALETTE;
        palette.text = [10, 11, 12, 255];
        palette.accent = [20, 21, 22, 255];
        palette.focus_ring = [30, 31, 32, 255];

        for node in [
            state_node!(hovered),
            state_node!(pressed),
            state_node!(dragging),
        ] {
            assert_eq!(state_layer_color_from_host(&node, palette), palette.text);
            assert_ne!(
                state_layer_color_from_host(&node, palette),
                palette.focus_ring
            );
        }

        let drop_target = state_node!(drop_hovered);
        assert_eq!(
            state_layer_color_from_host(&drop_target, palette),
            palette.accent
        );
        assert_ne!(
            state_layer_color_from_host(&drop_target, palette),
            palette.focus_ring
        );
    }
}
