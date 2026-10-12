use super::super::super::super::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::{
    ResolvedButtonStyle, UiResolvedElementStyle, UiRgbaColor, UiStyleColor,
};

pub(super) fn button_node() -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        role: "Button".into(),
        control_id: "TemplateStyleButton".into(),
        ..TemplatePaneNodeData::default()
    }
}

pub(super) fn panel_node(surface_variant: &str) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        role: "Panel".into(),
        control_id: "TemplateStylePanel".into(),
        surface_variant: surface_variant.into(),
        ..TemplatePaneNodeData::default()
    }
}

pub(super) fn resolved_background(color: [u8; 4]) -> ResolvedButtonStyle {
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
