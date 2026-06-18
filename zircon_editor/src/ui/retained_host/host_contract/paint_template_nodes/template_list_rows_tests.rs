use super::super::super::data::TemplateNodeFrameData;
use super::super::paint_theme::PALETTE;
use super::super::template_list_row_glyphs::{list_row_adornment_kind, ListRowAdornmentKind};
use super::super::template_nodes::paint_template_nodes_for_test;
use super::*;
use crate::ui::layouts::common::model_rc;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[test]
fn list_row_adornment_kind_prefers_disabled_then_selected_then_chevron() {
    assert_eq!(
        list_row_adornment_kind(&TemplatePaneNodeData {
            disabled: true,
            selected: true,
            checked: true,
            ..TemplatePaneNodeData::default()
        }),
        ListRowAdornmentKind::DisabledDiamond
    );
    let mut loading_selected = TemplatePaneNodeData {
        selected: true,
        checked: true,
        ..TemplatePaneNodeData::default()
    };
    loading_selected.button_style.loading = true;
    assert_eq!(
        list_row_adornment_kind(&loading_selected),
        ListRowAdornmentKind::DisabledDiamond
    );
    assert_eq!(
        list_row_adornment_kind(&TemplatePaneNodeData {
            selected: true,
            ..TemplatePaneNodeData::default()
        }),
        ListRowAdornmentKind::Check
    );
    assert_eq!(
        list_row_adornment_kind(&TemplatePaneNodeData::default()),
        ListRowAdornmentKind::Chevron
    );
}

#[test]
fn selected_list_row_paints_surface_and_right_check() {
    let bytes = paint_template_nodes_for_test(160, 40, model_rc(vec![list_node(true, false)]));

    assert_ne!(pixel_at(&bytes, 160, 12, 18), [0, 0, 0, 255]);
    assert!(changed_pixel_count(&bytes, 160, 135, 12, 16, 16) > 0);
}

#[test]
fn selected_list_row_uses_declared_surface_text_and_adornment_colors() {
    let mut node = list_node(true, false);
    node.value_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(53, 199, 208);
    node.icon_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(122, 230, 240);
    node.button_style.element.background_color =
        Some(zircon_runtime_interface::ui::style::UiStyleColor::Rgba(
            zircon_runtime_interface::ui::style::UiRgbaColor::from_u8(13, 65, 73, 255),
        ));

    assert_eq!(list_row_background(&node), Some([13, 65, 73, 255]));
    assert_eq!(list_row_text_color(&node), [53, 199, 208, 255]);
    assert_eq!(list_row_adornment_color(&node), [122, 230, 240, 255]);
}

#[test]
fn list_row_style_uses_shared_state_priority() {
    let mut node = list_node(false, true);
    node.hovered = true;
    node.focused = true;
    node.pressed = true;

    let disabled = list_row_style(&node);
    assert_eq!(disabled.state, UiPainterResolvedState::Disabled);
    assert_eq!(disabled.background, None);
    assert_eq!(disabled.border, None);
    assert_eq!(disabled.text, PALETTE.text_disabled);

    node.disabled = false;
    let pressed = list_row_style(&node);
    assert_eq!(pressed.state, UiPainterResolvedState::Pressed);
    assert_eq!(pressed.background, Some(PALETTE.surface_pressed));
    assert_eq!(pressed.border, Some(PALETTE.focus_ring));
    assert_eq!(pressed.border_width, 1.0);

    node.pressed = false;
    node.focused = false;
    node.hovered = false;
    node.selected = true;
    node.checked = true;
    let selected = list_row_style(&node);
    assert_eq!(selected.state, UiPainterResolvedState::Selected);
    assert_eq!(selected.background, Some(PALETTE.surface_selected));
    assert_eq!(selected.text, PALETTE.text);
    assert_eq!(selected.adornment, PALETTE.focus_ring);
}

#[test]
fn disabled_list_row_keeps_background_empty_and_draws_disabled_adornment() {
    let bytes = paint_template_nodes_for_test(160, 40, model_rc(vec![list_node(false, true)]));

    assert_eq!(pixel_at(&bytes, 160, 12, 18), [0, 0, 0, 255]);
    assert!(changed_pixel_count(&bytes, 160, 135, 12, 16, 16) > 0);
}

fn list_node(selected: bool, disabled: bool) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: if disabled {
            "WorkbenchListDisabled".into()
        } else {
            "WorkbenchListSelected".into()
        },
        role: "ListRow".into(),
        component_role: "list-row".into(),
        text: if disabled {
            "Disabled item".into()
        } else {
            "Selected item".into()
        },
        selected,
        checked: selected,
        disabled,
        frame: TemplateNodeFrameData {
            x: 4.0,
            y: 4.0,
            width: 148.0,
            height: 32.0,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn changed_pixel_count(
    bytes: &[u8],
    frame_width: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> usize {
    let mut changed = 0;
    for py in y..(y + height) {
        for px in x..(x + width) {
            let index = ((py as usize * frame_width as usize) + px as usize) * 4;
            if bytes[index..index + 4] != [0, 0, 0, 255] {
                changed += 1;
            }
        }
    }
    changed
}

fn pixel_at(bytes: &[u8], frame_width: u32, x: u32, y: u32) -> [u8; 4] {
    let index = ((y as usize * frame_width as usize) + x as usize) * 4;
    [
        bytes[index],
        bytes[index + 1],
        bytes[index + 2],
        bytes[index + 3],
    ]
}
