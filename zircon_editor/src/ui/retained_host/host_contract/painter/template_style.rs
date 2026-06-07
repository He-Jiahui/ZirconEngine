use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::style_selector::painter_state_for_node;
use super::theme::PALETTE;
use zircon_runtime_interface::ui::style::{
    ButtonColor, ButtonInteractionState, ButtonVariant, UiStyleColor,
};

const MATERIAL_ELEVATION_SHADOW_OFFSET: f32 = 2.0;
const MUI_TOOLTIP_BG: [u8; 4] = [97, 97, 97, 255];
const MUI_SNACKBAR_BG: [u8; 4] = [50, 50, 50, 255];
const MUI_ON_DARK: [u8; 4] = [255, 255, 255, 255];

pub(super) fn template_border_width(node: &TemplatePaneNodeData) -> f32 {
    let width = node
        .border_width
        .max(node.button_style.element.border_width)
        .max(0.0);
    if matches!(
        button_interaction_state(node),
        ButtonInteractionState::Pressed | ButtonInteractionState::Focused
    ) || node.selected
        || node.checked
    {
        width.max(2.0)
    } else {
        width
    }
}

pub(super) fn template_corner_radius(node: &TemplatePaneNodeData) -> f32 {
    node.corner_radius
        .max(node.button_style.element.corner_radius)
        .max(0.0)
}

pub(super) fn draws_elevation_shadow(node: &TemplatePaneNodeData) -> bool {
    node.elevation > 0.0 && !is_button_disabled(node)
}

pub(super) fn elevation_shadow_rect(rect: &FrameRect, elevation: f32) -> FrameRect {
    let offset = elevation.max(1.0) * MATERIAL_ELEVATION_SHADOW_OFFSET;
    FrameRect {
        x: rect.x + offset,
        y: rect.y + offset,
        width: rect.width,
        height: rect.height,
    }
}

pub(super) fn surface_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if is_button_disabled(node) {
        return PALETTE.surface_disabled;
    }
    if matches!(node.validation_level.as_str(), "error" | "danger")
        || matches!(node.surface_variant.as_str(), "danger" | "error")
    {
        return PALETTE.error_container;
    }
    if node.validation_level.as_str() == "warning" {
        return PALETTE.warning_container;
    }
    if node.validation_level.as_str() == "success" || node.surface_variant.as_str() == "success" {
        return PALETTE.success_container;
    }
    if node.validation_level.as_str() == "info" || node.surface_variant.as_str() == "info" {
        return PALETTE.info_container;
    }
    match button_interaction_state(node) {
        ButtonInteractionState::Pressed => return PALETTE.surface_pressed,
        ButtonInteractionState::Focused => return PALETTE.surface_selected,
        ButtonInteractionState::Hover => {
            return if is_primary_contained_button(node) {
                PALETTE.accent_soft
            } else {
                PALETTE.surface_hover
            };
        }
        ButtonInteractionState::Disabled => return PALETTE.surface_disabled,
        ButtonInteractionState::Loading | ButtonInteractionState::Normal => {}
    }
    if let Some(color) = resolved_style_color(node.button_style.element.background_color.as_ref()) {
        return color;
    }
    if let Some(color) = typed_button_variant_background(node) {
        return color;
    }
    match node.surface_variant.as_str() {
        "tooltip" => return MUI_TOOLTIP_BG,
        "snackbar" => return MUI_SNACKBAR_BG,
        "paper" | "paper-outlined" | "dialog" | "popover" => return PALETTE.popup,
        _ => {}
    }
    if matches!(node.button_variant.as_str(), "primary" | "filled")
        || matches!(node.surface_variant.as_str(), "accent" | "primary")
    {
        return PALETTE.accent;
    }
    match node.surface_variant.as_str() {
        "inset" | "scroll-body" | "asset-tree-row" | "reference-row" => PALETTE.surface_inset,
        "popup" | "elevated" => PALETTE.popup,
        "panel" | "asset-preview" | "asset-preview-visual" => PALETTE.surface,
        "shell" => PALETTE.shell_background,
        _ => match node.role.as_str() {
            "Button" if node.surface_variant.is_empty() && is_explicit_text_button(node) => {
                [0, 0, 0, 0]
            }
            "Button" if node.surface_variant.is_empty() => PALETTE.surface_hover,
            _ => PALETTE.surface,
        },
    }
}

pub(super) fn border_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if is_button_disabled(node) {
        return PALETTE.border_disabled;
    }
    if matches!(node.validation_level.as_str(), "error" | "danger")
        || matches!(node.surface_variant.as_str(), "danger" | "error")
    {
        return PALETTE.error;
    }
    if node.validation_level.as_str() == "warning" {
        return PALETTE.warning;
    }
    if node.validation_level.as_str() == "success" || node.surface_variant.as_str() == "success" {
        return PALETTE.success;
    }
    if node.validation_level.as_str() == "info" || node.surface_variant.as_str() == "info" {
        return PALETTE.info;
    }
    if let Some(color) = resolved_style_color(node.button_style.element.border_color.as_ref()) {
        return color;
    }
    if matches!(
        button_interaction_state(node),
        ButtonInteractionState::Pressed | ButtonInteractionState::Focused
    ) || node.selected
        || node.checked
    {
        PALETTE.focus_ring
    } else if let Some(color) = typed_button_tone_color(node) {
        color
    } else if matches!(node.button_variant.as_str(), "primary" | "filled")
        || matches!(node.surface_variant.as_str(), "accent" | "primary")
        || matches!(
            button_interaction_state(node),
            ButtonInteractionState::Hover
        )
    {
        PALETTE.focus_ring
    } else {
        PALETTE.border
    }
}

pub(super) fn text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if is_button_disabled(node) {
        return PALETTE.text_disabled;
    }
    if let Some(color) = resolved_style_color(node.button_style.element.foreground_color.as_ref()) {
        return color;
    }
    if is_primary_contained_button(node)
        && matches!(
            button_interaction_state(node),
            ButtonInteractionState::Normal | ButtonInteractionState::Hover
        )
    {
        return [8, 20, 22, 255];
    }
    match node.text_tone.as_str() {
        "inverse" | "on-dark" | "tooltip" | "snackbar" => MUI_ON_DARK,
        "muted" | "subtle" => PALETTE.text_muted,
        "accent" | "primary" | "default" => PALETTE.focus_ring,
        "warning" => PALETTE.warning,
        "error" | "danger" => PALETTE.error,
        "success" => PALETTE.success,
        "info" => PALETTE.info,
        _ => PALETTE.text,
    }
}

pub(super) fn is_mui_overlay_surface_node(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.component_role.as_str(),
        "paper"
            | "dialog"
            | "alert-dialog"
            | "popover"
            | "menu"
            | "tooltip"
            | "snackbar"
            | "drawer"
    )
}

pub(super) fn is_button_disabled(node: &TemplatePaneNodeData) -> bool {
    node.disabled
        || node.button_style.disabled
        || matches!(
            node.button_style.interaction_state,
            ButtonInteractionState::Disabled
        )
}

pub(super) fn resolved_style_color(color: Option<&UiStyleColor>) -> Option<[u8; 4]> {
    match color? {
        UiStyleColor::Rgba(color) => Some(color.to_u8()),
        UiStyleColor::Transparent => Some([0, 0, 0, 0]),
        UiStyleColor::Inherit => None,
        UiStyleColor::Role(role) => material_role_color(role),
    }
}

fn button_interaction_state(node: &TemplatePaneNodeData) -> ButtonInteractionState {
    painter_state_for_node(node).button_interaction_state()
}

fn typed_button_variant_background(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    if !matches!(node.role.as_str(), "Button" | "IconButton") {
        return None;
    }
    match node.button_style.variant.normalized() {
        ButtonVariant::Contained => Some(button_container_color(&node.button_style.color)),
        ButtonVariant::Outlined => Some(PALETTE.surface_inset),
        ButtonVariant::Text | ButtonVariant::Default => None,
    }
}

fn is_explicit_text_button(node: &TemplatePaneNodeData) -> bool {
    matches!(node.button_variant.as_str(), "default" | "text")
        || (!node.button_variant.is_empty()
            && node.button_style.variant.normalized() == ButtonVariant::Text)
}

fn is_primary_contained_button(node: &TemplatePaneNodeData) -> bool {
    (node.button_style.variant.normalized() == ButtonVariant::Contained
        && is_primary_button_color(&node.button_style.color))
        || matches!(node.button_variant.as_str(), "primary" | "filled")
        || matches!(node.surface_variant.as_str(), "accent" | "primary")
}

fn button_container_color(color: &ButtonColor) -> [u8; 4] {
    match color {
        ButtonColor::Warning => PALETTE.warning_container,
        ButtonColor::Error => PALETTE.error_container,
        ButtonColor::Success => PALETTE.success_container,
        ButtonColor::Info => PALETTE.info_container,
        ButtonColor::Custom(color) => color.to_u8(),
        ButtonColor::Style(role) => material_role_color(role).unwrap_or(PALETTE.surface_selected),
        ButtonColor::Default | ButtonColor::Primary => PALETTE.accent,
        ButtonColor::Secondary | ButtonColor::Inherit => PALETTE.surface_selected,
    }
}

fn typed_button_tone_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    if !matches!(node.role.as_str(), "Button" | "IconButton") {
        return None;
    }
    match &node.button_style.color {
        ButtonColor::Warning => Some(PALETTE.warning),
        ButtonColor::Error => Some(PALETTE.error),
        ButtonColor::Success => Some(PALETTE.success),
        ButtonColor::Info => Some(PALETTE.info),
        ButtonColor::Custom(color) => Some(color.to_u8()),
        ButtonColor::Style(role) => material_role_color(role),
        ButtonColor::Default | ButtonColor::Primary
            if matches!(
                node.button_style.variant.normalized(),
                ButtonVariant::Contained | ButtonVariant::Outlined
            ) =>
        {
            Some(PALETTE.focus_ring)
        }
        ButtonColor::Secondary
        | ButtonColor::Inherit
        | ButtonColor::Default
        | ButtonColor::Primary => None,
    }
}

fn is_primary_button_color(color: &ButtonColor) -> bool {
    matches!(color, ButtonColor::Default | ButtonColor::Primary)
}

fn material_role_color(role: &str) -> Option<[u8; 4]> {
    match role {
        "primary" | "accent" | "material.primary" | "material_color_primary" => {
            Some(PALETTE.accent)
        }
        "on_primary" | "material.on_primary" | "material_color_on_primary" => {
            Some([8, 20, 22, 255])
        }
        "surface" | "material.surface" => Some(PALETTE.surface),
        "surface_inset" | "material.surface_inset" => Some(PALETTE.surface_inset),
        "surface_hover" | "material.surface_hover" => Some(PALETTE.surface_hover),
        "surface_pressed" | "material.surface_pressed" => Some(PALETTE.surface_pressed),
        "surface_selected" | "material.surface_selected" => Some(PALETTE.surface_selected),
        "disabled" | "material.disabled" => Some(PALETTE.surface_disabled),
        "border" | "outline" | "material.outline" => Some(PALETTE.border),
        "focus" | "focus_ring" | "material.focus_ring" => Some(PALETTE.focus_ring),
        "text" | "on_surface" | "material.text" | "material.on_surface" => Some(PALETTE.text),
        "text_muted" | "muted" | "material.text_muted" => Some(PALETTE.text_muted),
        "text_disabled" | "material.text_disabled" => Some(PALETTE.text_disabled),
        "warning" | "material.warning" => Some(PALETTE.warning),
        "error" | "danger" | "material.error" => Some(PALETTE.error),
        "success" | "material.success" => Some(PALETTE.success),
        "info" | "material.info" => Some(PALETTE.info),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime_interface::ui::style::{
        ButtonInteractionState, ResolvedButtonStyle, UiResolvedElementStyle, UiRgbaColor,
        UiStyleColor,
    };

    #[test]
    fn native_template_button_state_uses_shared_painter_priority() {
        let mut node = button_node();
        node.focused = true;
        node.pressed = true;

        assert_eq!(
            button_interaction_state(&node),
            ButtonInteractionState::Pressed
        );
        assert_eq!(surface_color(&node), PALETTE.surface_pressed);

        node.button_style.loading = true;
        assert_eq!(
            button_interaction_state(&node),
            ButtonInteractionState::Loading
        );

        node.disabled = true;
        assert_eq!(
            button_interaction_state(&node),
            ButtonInteractionState::Disabled
        );
        assert_eq!(surface_color(&node), PALETTE.surface_disabled);
    }

    #[test]
    fn native_template_button_style_state_values_feed_shared_priority() {
        let mut node = button_node();
        node.button_style.interaction_state = ButtonInteractionState::Pressed;
        assert_eq!(
            button_interaction_state(&node),
            ButtonInteractionState::Pressed
        );

        node.button_style.interaction_state = ButtonInteractionState::Loading;
        assert_eq!(
            button_interaction_state(&node),
            ButtonInteractionState::Loading
        );

        node.button_style.interaction_state = ButtonInteractionState::Disabled;
        assert_eq!(
            button_interaction_state(&node),
            ButtonInteractionState::Disabled
        );
    }

    #[test]
    fn native_template_button_style_keeps_declared_colors_after_state_resolution() {
        let mut node = button_node();
        node.hovered = true;
        node.button_style = resolved_background([11, 22, 33, 255]);

        assert_eq!(
            button_interaction_state(&node),
            ButtonInteractionState::Hover
        );
        assert_eq!(surface_color(&node), PALETTE.surface_hover);

        node.hovered = false;
        assert_eq!(surface_color(&node), [11, 22, 33, 255]);
    }

    fn button_node() -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            role: "Button".into(),
            control_id: "TemplateStyleButton".into(),
            ..TemplatePaneNodeData::default()
        }
    }

    fn resolved_background(color: [u8; 4]) -> ResolvedButtonStyle {
        ResolvedButtonStyle {
            element: UiResolvedElementStyle {
                background_color: Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(
                    color[0], color[1], color[2], color[3],
                ))),
                ..UiResolvedElementStyle::default()
            },
            ..ResolvedButtonStyle::default()
        }
    }
}
