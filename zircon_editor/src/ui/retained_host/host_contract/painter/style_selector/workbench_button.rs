use super::super::super::data::TemplatePaneNodeData;
use super::super::template_style::resolved_style_color;
use super::super::theme::PALETTE;
use super::painter_state_for_node;
use zircon_runtime_interface::ui::style::{ButtonInteractionState, UiStyleColor};

pub(in crate::ui::retained_host::host_contract::painter) const PRIMARY_SURFACE: [u8; 4] =
    [50, 184, 197, 255];
const PRIMARY_SURFACE_HOVER: [u8; 4] = [67, 204, 216, 255];
const PRIMARY_SURFACE_PRESSED: [u8; 4] = [30, 140, 153, 255];
const PRIMARY_TEXT: [u8; 4] = [8, 24, 27, 255];
pub(in crate::ui::retained_host::host_contract::painter) const OUTLINED_SURFACE: [u8; 4] =
    [25, 31, 35, 255];
const OUTLINED_SURFACE_HOVER: [u8; 4] = [32, 40, 45, 255];
const OUTLINED_SURFACE_PRESSED: [u8; 4] = [18, 52, 61, 255];
pub(in crate::ui::retained_host::host_contract::painter) const OUTLINED_BORDER: [u8; 4] =
    [58, 70, 78, 255];
pub(in crate::ui::retained_host::host_contract::painter) const OUTLINED_TEXT: [u8; 4] =
    [201, 213, 218, 255];
const TERTIARY_TEXT: [u8; 4] = [152, 166, 174, 255];
const DANGER_SURFACE: [u8; 4] = [72, 32, 36, 255];
const DANGER_BORDER: [u8; 4] = [122, 57, 55, 255];
const DANGER_TEXT: [u8; 4] = [239, 112, 102, 255];
pub(in crate::ui::retained_host::host_contract::painter) const ADD_COMPONENT_TEXT: [u8; 4] =
    [186, 196, 201, 255];
pub(in crate::ui::retained_host::host_contract::painter) const ADD_COMPONENT_GLYPH: [u8; 4] =
    [197, 206, 210, 255];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::painter) enum WorkbenchButtonKind {
    Primary,
    Secondary,
    Tertiary,
    Danger,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::painter) struct WorkbenchButtonStyle {
    pub surface: [u8; 4],
    pub border: [u8; 4],
    pub border_width: f32,
    pub text: [u8; 4],
    pub glyph: [u8; 4],
    pub interaction: ButtonInteractionState,
}

pub(in crate::ui::retained_host::host_contract::painter) fn select_workbench_button_style(
    node: &TemplatePaneNodeData,
    kind: WorkbenchButtonKind,
    is_add_component_button: bool,
) -> WorkbenchButtonStyle {
    let state = painter_state_for_node(node);
    let interaction = state.button_interaction_state();
    let mut style = base_button_style(kind, interaction);
    style.interaction = interaction;

    if is_unavailable_button_interaction(interaction) {
        return style;
    }

    if let Some(surface) =
        declared_button_style_color(node.button_style.element.background_color.as_ref())
    {
        style.surface = surface;
    }
    if let Some(border) =
        declared_button_style_color(node.button_style.element.border_color.as_ref())
    {
        style.border = border;
    }
    if let Some(text) =
        declared_button_style_color(node.button_style.element.foreground_color.as_ref())
    {
        style.text = text;
        style.glyph = text;
    }
    if is_add_component_button {
        style.text = ADD_COMPONENT_TEXT;
        style.glyph = ADD_COMPONENT_GLYPH;
    }
    apply_visual_brightness(style, node.label_brightness)
}

fn base_button_style(
    kind: WorkbenchButtonKind,
    interaction: ButtonInteractionState,
) -> WorkbenchButtonStyle {
    match interaction {
        ButtonInteractionState::Disabled | ButtonInteractionState::Loading => {
            unavailable_button_style(interaction)
        }
        ButtonInteractionState::Normal => normal_button_style(kind, interaction),
        ButtonInteractionState::Hover => hover_button_style(kind, interaction),
        ButtonInteractionState::Pressed => pressed_button_style(kind, interaction),
        ButtonInteractionState::Focused => focused_button_style(kind, interaction),
    }
}

fn unavailable_button_style(interaction: ButtonInteractionState) -> WorkbenchButtonStyle {
    WorkbenchButtonStyle {
        surface: PALETTE.surface_disabled,
        border: PALETTE.border_disabled,
        border_width: 1.0,
        text: PALETTE.text_disabled,
        glyph: PALETTE.text_disabled,
        interaction,
    }
}

fn normal_button_style(
    kind: WorkbenchButtonKind,
    interaction: ButtonInteractionState,
) -> WorkbenchButtonStyle {
    match kind {
        WorkbenchButtonKind::Primary => WorkbenchButtonStyle {
            surface: PRIMARY_SURFACE,
            border: [36, 154, 166, 255],
            border_width: 1.0,
            text: PRIMARY_TEXT,
            glyph: PRIMARY_TEXT,
            interaction,
        },
        WorkbenchButtonKind::Secondary => WorkbenchButtonStyle {
            surface: OUTLINED_SURFACE,
            border: OUTLINED_BORDER,
            border_width: 1.0,
            text: OUTLINED_TEXT,
            glyph: OUTLINED_TEXT,
            interaction,
        },
        WorkbenchButtonKind::Tertiary => WorkbenchButtonStyle {
            surface: PALETTE.surface_inset,
            border: PALETTE.border,
            border_width: 1.0,
            text: TERTIARY_TEXT,
            glyph: TERTIARY_TEXT,
            interaction,
        },
        WorkbenchButtonKind::Danger => WorkbenchButtonStyle {
            surface: DANGER_SURFACE,
            border: DANGER_BORDER,
            border_width: 1.0,
            text: DANGER_TEXT,
            glyph: DANGER_TEXT,
            interaction,
        },
    }
}

fn hover_button_style(
    kind: WorkbenchButtonKind,
    interaction: ButtonInteractionState,
) -> WorkbenchButtonStyle {
    let mut style = normal_button_style(kind, interaction);
    match kind {
        WorkbenchButtonKind::Primary => {
            style.surface = PRIMARY_SURFACE_HOVER;
        }
        WorkbenchButtonKind::Secondary => {
            style.surface = OUTLINED_SURFACE_HOVER;
        }
        WorkbenchButtonKind::Tertiary => {
            style.surface = PALETTE.surface_hover;
        }
        WorkbenchButtonKind::Danger => {
            style.surface = [88, 40, 43, 255];
        }
    }
    style
}

fn pressed_button_style(
    kind: WorkbenchButtonKind,
    interaction: ButtonInteractionState,
) -> WorkbenchButtonStyle {
    let mut style = normal_button_style(kind, interaction);
    match kind {
        WorkbenchButtonKind::Primary => {
            style.surface = PRIMARY_SURFACE_PRESSED;
            style.border = PALETTE.focus_ring;
        }
        WorkbenchButtonKind::Secondary => {
            style.surface = OUTLINED_SURFACE_PRESSED;
            style.border = PALETTE.focus_ring;
        }
        WorkbenchButtonKind::Tertiary => {
            style.surface = PALETTE.surface_pressed;
            style.border = PALETTE.focus_ring;
        }
        WorkbenchButtonKind::Danger => {
            style.surface = [82, 37, 40, 255];
            style.border = PALETTE.error;
        }
    }
    style
}

fn focused_button_style(
    kind: WorkbenchButtonKind,
    interaction: ButtonInteractionState,
) -> WorkbenchButtonStyle {
    let mut style = hover_button_style(kind, interaction);
    style.border = if kind == WorkbenchButtonKind::Danger {
        PALETTE.error
    } else {
        PALETTE.focus_ring
    };
    style
}

fn declared_button_style_color(color: Option<&UiStyleColor>) -> Option<[u8; 4]> {
    resolved_style_color(color).filter(|color| color[3] > 0)
}

fn is_unavailable_button_interaction(interaction: ButtonInteractionState) -> bool {
    matches!(
        interaction,
        ButtonInteractionState::Disabled | ButtonInteractionState::Loading
    )
}

fn apply_visual_brightness(style: WorkbenchButtonStyle, brightness: f32) -> WorkbenchButtonStyle {
    if !brightness.is_finite() || brightness <= 0.0 || (brightness - 1.0).abs() < 0.001 {
        return style;
    }
    let brightness = brightness.clamp(0.0, 4.0);
    WorkbenchButtonStyle {
        surface: scaled_color(style.surface, brightness),
        border: scaled_color(style.border, brightness),
        border_width: style.border_width,
        text: scaled_color(style.text, brightness),
        glyph: scaled_color(style.glyph, brightness),
        interaction: style.interaction,
    }
}

fn scaled_color(color: [u8; 4], brightness: f32) -> [u8; 4] {
    [
        scaled_channel(color[0], brightness),
        scaled_channel(color[1], brightness),
        scaled_channel(color[2], brightness),
        color[3],
    ]
}

fn scaled_channel(value: u8, brightness: f32) -> u8 {
    (f32::from(value) * brightness).round().clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

    #[test]
    fn button_loading_state_uses_unavailable_visuals() {
        let mut node = TemplatePaneNodeData::default();
        node.hovered = true;
        node.focused = true;
        node.pressed = true;
        node.button_style.loading = true;
        node.label_brightness = 1.5;
        node.button_style.element.background_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(41, 164, 184, 255)));
        node.button_style.element.border_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(28, 135, 152, 255)));
        node.button_style.element.foreground_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(8, 24, 27, 255)));

        let style = select_workbench_button_style(&node, WorkbenchButtonKind::Primary, true);

        assert_eq!(style.interaction, ButtonInteractionState::Loading);
        assert_eq!(style.surface, PALETTE.surface_disabled);
        assert_eq!(style.border, PALETTE.border_disabled);
        assert_eq!(style.border_width, 1.0);
        assert_eq!(style.text, PALETTE.text_disabled);
        assert_eq!(style.glyph, PALETTE.text_disabled);
    }
}
