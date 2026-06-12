use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::template_component_family::{is_component_family, TemplateComponentFamily};
use super::render_commands::HostPaintCommand;
use super::style_selector::{select_workbench_list_row_style, WorkbenchListRowStyle};
use super::template_node_labels::template_node_label;
use super::theme::PALETTE;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const LIST_ROW_FONT_SIZE: f32 = 12.0;
const LIST_ROW_TEXT_INSET_X: f32 = 14.0;
const LIST_ROW_TEXT_INSET_Y: f32 = 6.0;
const LIST_ROW_RIGHT_INSET: f32 = 12.0;
const LIST_ROW_ADORNMENT_SIZE: f32 = 13.0;
const LIST_ROW_RADIUS: f32 = 4.0;
const LIST_ROW_ADORNMENT_RESERVE: f32 = 26.0;

pub(super) fn push_list_row_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_workbench_list_row(node) {
        return false;
    }

    push_list_row_surface(commands, node, rect, clip, order, opacity);
    push_list_row_label(commands, node, rect, clip, order + 2, opacity);
    push_list_row_adornment(commands, node, rect, clip, order + 3, opacity);
    true
}

fn is_workbench_list_row(node: &TemplatePaneNodeData) -> bool {
    is_component_family(node, TemplateComponentFamily::ListRow)
        && !node.control_id.as_str().ends_with("Title")
}

fn push_list_row_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let Some(background) = list_row_background(node) else {
        return;
    };
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(background),
        list_row_border(node),
        list_row_border_width(node),
        LIST_ROW_RADIUS,
        opacity,
    ));
}

fn push_list_row_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let label = template_node_label(node, None);
    if label.trim().is_empty() {
        return;
    }
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: rect.x + LIST_ROW_TEXT_INSET_X,
            y: rect.y + LIST_ROW_TEXT_INSET_Y,
            width: (rect.width - LIST_ROW_TEXT_INSET_X - LIST_ROW_ADORNMENT_RESERVE).max(1.0),
            height: (rect.height - LIST_ROW_TEXT_INSET_Y * 2.0).max(1.0),
        },
        Some(clip.clone()),
        order,
        label,
        list_row_text_color(node),
        LIST_ROW_FONT_SIZE,
        LIST_ROW_FONT_SIZE * 1.2,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn push_list_row_adornment(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let adornment = list_row_adornment_rect(rect);
    match list_row_adornment_kind(node) {
        ListRowAdornmentKind::Check => {
            push_check_mark(
                commands,
                &adornment,
                clip,
                order,
                list_row_adornment_color(node),
                opacity,
            );
        }
        ListRowAdornmentKind::Chevron => {
            push_right_chevron(
                commands,
                &adornment,
                clip,
                order,
                list_row_adornment_color(node),
                opacity,
            );
        }
        ListRowAdornmentKind::DisabledDiamond => {
            push_disabled_diamond(commands, &adornment, clip, order, opacity);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ListRowAdornmentKind {
    Check,
    Chevron,
    DisabledDiamond,
}

fn list_row_adornment_kind(node: &TemplatePaneNodeData) -> ListRowAdornmentKind {
    if is_unavailable_list_row_state(list_row_style(node).state) {
        ListRowAdornmentKind::DisabledDiamond
    } else if node.checked || node.selected {
        ListRowAdornmentKind::Check
    } else {
        ListRowAdornmentKind::Chevron
    }
}

fn list_row_background(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    list_row_style(node).background
}

fn list_row_border(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    list_row_style(node).border
}

fn list_row_border_width(node: &TemplatePaneNodeData) -> f32 {
    list_row_style(node).border_width
}

fn list_row_text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    list_row_style(node).text
}

fn list_row_adornment_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    list_row_style(node).adornment
}

fn list_row_style(node: &TemplatePaneNodeData) -> WorkbenchListRowStyle {
    select_workbench_list_row_style(node)
}

fn is_unavailable_list_row_state(state: UiPainterResolvedState) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
    )
}

fn list_row_adornment_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x + rect.width - LIST_ROW_RIGHT_INSET - LIST_ROW_ADORNMENT_SIZE,
        y: rect.y + (rect.height - LIST_ROW_ADORNMENT_SIZE).max(0.0) * 0.5,
        width: LIST_ROW_ADORNMENT_SIZE,
        height: LIST_ROW_ADORNMENT_SIZE,
    }
}

fn push_check_mark(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    for segment in [
        FrameRect {
            x: rect.x + 2.0,
            y: rect.y + 7.0,
            width: 3.0,
            height: 2.0,
        },
        FrameRect {
            x: rect.x + 4.0,
            y: rect.y + 9.0,
            width: 3.0,
            height: 2.0,
        },
        FrameRect {
            x: rect.x + 7.0,
            y: rect.y + 4.0,
            width: 3.0,
            height: 7.0,
        },
    ] {
        commands.push(HostPaintCommand::quad(
            segment,
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            1.0,
            opacity,
        ));
    }
}

fn push_right_chevron(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    for segment in [
        FrameRect {
            x: rect.x + 5.0,
            y: rect.y + 3.0,
            width: 2.0,
            height: 3.0,
        },
        FrameRect {
            x: rect.x + 7.0,
            y: rect.y + 6.0,
            width: 2.0,
            height: 2.0,
        },
        FrameRect {
            x: rect.x + 5.0,
            y: rect.y + 8.0,
            width: 2.0,
            height: 3.0,
        },
    ] {
        commands.push(HostPaintCommand::quad(
            segment,
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            1.0,
            opacity,
        ));
    }
}

fn push_disabled_diamond(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let color = PALETTE.text_disabled;
    let center_x = rect.x + rect.width * 0.5;
    let center_y = rect.y + rect.height * 0.5;
    for segment in [
        FrameRect {
            x: center_x - 1.0,
            y: center_y - 5.0,
            width: 2.0,
            height: 2.0,
        },
        FrameRect {
            x: center_x + 3.0,
            y: center_y - 1.0,
            width: 2.0,
            height: 2.0,
        },
        FrameRect {
            x: center_x - 1.0,
            y: center_y + 3.0,
            width: 2.0,
            height: 2.0,
        },
        FrameRect {
            x: center_x - 5.0,
            y: center_y - 1.0,
            width: 2.0,
            height: 2.0,
        },
    ] {
        commands.push(HostPaintCommand::quad(
            segment,
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            1.0,
            opacity,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::data::TemplateNodeFrameData;
    use super::super::template_nodes::paint_template_nodes_for_test;
    use super::*;
    use crate::ui::layouts::common::model_rc;

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
}
