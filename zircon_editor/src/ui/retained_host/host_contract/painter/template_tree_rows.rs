use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::template_component_family::{is_component_family, TemplateComponentFamily};
use super::render_commands::HostPaintCommand;
use super::style_selector::{select_workbench_tree_row_style, WorkbenchTreeRowStyle};
use super::template_node_labels::template_node_label;
use super::theme::PALETTE;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const TREE_FONT_SIZE: f32 = 12.0;
const TREE_BASE_INSET_X: f32 = 12.0;
const TREE_DISCLOSURE_SIZE: f32 = 12.0;
const TREE_ICON_SIZE: f32 = 14.0;
const TREE_TEXT_GAP: f32 = 7.0;
const TREE_RIGHT_INSET: f32 = 12.0;
const TREE_ACTION_SIZE: f32 = 14.0;
const TREE_ACTION_GAP: f32 = 16.0;
const TREE_ROW_RADIUS: f32 = 5.0;
const TREE_GUIDE_STEP: f32 = 18.0;
const TREE_GUIDE_COLOR: [u8; 4] = [42, 55, 64, 255];
const TREE_OBJECT_BLUE: [u8; 4] = [82, 148, 240, 255];

pub(super) fn push_tree_row_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_workbench_tree_row(node) {
        return false;
    }

    push_tree_row_surface(commands, node, rect, clip, order, opacity);
    push_tree_indent_guides(commands, node, rect, clip, order + 1, opacity);

    let disclosure = tree_disclosure_rect(node, rect);
    push_tree_disclosure(
        commands,
        node,
        &disclosure,
        clip,
        order + 2,
        tree_secondary_color(node),
        opacity,
    );

    let icon = tree_icon_rect(&disclosure);
    push_tree_object_icon(
        commands,
        node,
        &icon,
        clip,
        order + 3,
        tree_icon_color(node),
        opacity,
    );
    push_tree_label(commands, node, rect, &icon, clip, order + 4, opacity);
    push_tree_actions(commands, node, rect, clip, order + 5, opacity);
    true
}

fn is_workbench_tree_row(node: &TemplatePaneNodeData) -> bool {
    is_component_family(node, TemplateComponentFamily::TreeRow)
}

fn push_tree_row_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let Some(background) = tree_row_background(node) else {
        return;
    };
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(background),
        tree_row_border(node),
        tree_row_border_width(node),
        TREE_ROW_RADIUS,
        opacity,
    ));
}

fn push_tree_indent_guides(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let depth = node.tree_depth.max(0) as usize;
    for level in 0..depth {
        let guide_x = rect.x + TREE_BASE_INSET_X + 5.0 + (level as f32 * TREE_GUIDE_STEP);
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: guide_x,
                y: rect.y - 1.0,
                width: 1.0,
                height: rect.height + 2.0,
            },
            Some(clip.clone()),
            order,
            Some(TREE_GUIDE_COLOR),
            None,
            0.0,
            0.0,
            opacity * 0.78,
        ));
    }
}

fn push_tree_disclosure(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    if node.expanded {
        push_down_chevron(commands, rect, clip, order, color, opacity);
    } else {
        push_right_chevron(commands, rect, clip, order, color, opacity);
    }
}

fn push_tree_object_icon(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    match tree_icon_kind(node) {
        TreeIconKind::Audio => push_audio_icon(commands, rect, clip, order, color, opacity),
        TreeIconKind::PlayerStart => {
            push_player_start_icon(commands, rect, clip, order, TREE_OBJECT_BLUE, opacity)
        }
        TreeIconKind::Cube => push_cube_icon(commands, rect, clip, order, color, opacity),
    }
}

fn push_tree_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    icon: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let label = template_node_label(node, None);
    if label.trim().is_empty() {
        return;
    }

    let line_height = TREE_FONT_SIZE * 1.2;
    let text_x = icon.x + icon.width + TREE_TEXT_GAP;
    let right_reserve = TREE_RIGHT_INSET + TREE_ACTION_SIZE * 2.0 + TREE_ACTION_GAP;
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: text_x,
            y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
            width: (rect.x + rect.width - text_x - right_reserve).max(1.0),
            height: line_height,
        },
        Some(clip.clone()),
        order,
        label,
        tree_text_color(node),
        TREE_FONT_SIZE,
        line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn push_tree_actions(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let eye = tree_action_rect(rect, 1);
    push_eye_icon(
        commands,
        &eye,
        clip,
        order,
        tree_action_color(node),
        opacity,
    );

    let secondary = tree_action_rect(rect, 0);
    if node.selected || node.checked {
        push_kebab_icon(
            commands,
            &secondary,
            clip,
            order + 1,
            tree_action_color(node),
            opacity,
        );
    } else if shows_tree_lock_action(node) {
        push_lock_icon(
            commands,
            &secondary,
            clip,
            order + 1,
            tree_action_color(node),
            opacity,
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TreeIconKind {
    Cube,
    PlayerStart,
    Audio,
}

fn tree_icon_kind(node: &TemplatePaneNodeData) -> TreeIconKind {
    let id = node.control_id.as_str();
    let label = node.text.as_str();
    if id.contains("Audio") || label.contains("Audio") {
        TreeIconKind::Audio
    } else if id.contains("Player") || label.contains("Player") {
        TreeIconKind::PlayerStart
    } else {
        TreeIconKind::Cube
    }
}

fn tree_disclosure_rect(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    let indent = if node.tree_indent_px.is_finite() && node.tree_indent_px > 0.0 {
        node.tree_indent_px
    } else {
        node.tree_depth.max(0) as f32 * TREE_GUIDE_STEP
    };
    FrameRect {
        x: rect.x + TREE_BASE_INSET_X + indent,
        y: rect.y + (rect.height - TREE_DISCLOSURE_SIZE).max(0.0) * 0.5,
        width: TREE_DISCLOSURE_SIZE,
        height: TREE_DISCLOSURE_SIZE,
    }
}

fn tree_icon_rect(disclosure: &FrameRect) -> FrameRect {
    FrameRect {
        x: disclosure.x + disclosure.width + 4.0,
        y: disclosure.y + (disclosure.height - TREE_ICON_SIZE).max(0.0) * 0.5,
        width: TREE_ICON_SIZE,
        height: TREE_ICON_SIZE,
    }
}

fn tree_action_rect(rect: &FrameRect, index_from_right: usize) -> FrameRect {
    let stride = TREE_ACTION_SIZE + TREE_ACTION_GAP;
    FrameRect {
        x: rect.x + rect.width
            - TREE_RIGHT_INSET
            - TREE_ACTION_SIZE
            - index_from_right as f32 * stride,
        y: rect.y + (rect.height - TREE_ACTION_SIZE).max(0.0) * 0.5,
        width: TREE_ACTION_SIZE,
        height: TREE_ACTION_SIZE,
    }
}

fn tree_row_background(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    tree_row_style(node).background
}

fn tree_row_border(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    tree_row_style(node).border
}

fn tree_row_border_width(node: &TemplatePaneNodeData) -> f32 {
    tree_row_style(node).border_width
}

fn tree_text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    tree_row_style(node).text
}

fn tree_icon_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    tree_row_style(node).icon
}

fn tree_secondary_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    tree_row_style(node).secondary
}

fn tree_action_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    tree_row_style(node).action
}

fn tree_row_style(node: &TemplatePaneNodeData) -> WorkbenchTreeRowStyle {
    select_workbench_tree_row_style(node)
}

fn shows_tree_lock_action(node: &TemplatePaneNodeData) -> bool {
    let id = node.control_id.as_str();
    node.tree_depth <= 1
        || id.contains("Audio")
        || id.contains("Root")
        || id.contains("Environment")
}

fn push_cube_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        clip,
        order,
        color,
        opacity,
        &[
            local_rect(rect, 3.0, 2.0, 8.0, 1.0),
            local_rect(rect, 2.0, 3.0, 1.0, 7.0),
            local_rect(rect, 11.0, 3.0, 1.0, 7.0),
            local_rect(rect, 3.0, 10.0, 8.0, 1.0),
            local_rect(rect, 6.0, 0.0, 1.0, 3.0),
            local_rect(rect, 6.0, 10.0, 1.0, 3.0),
            local_rect(rect, 2.0, 6.0, 10.0, 1.0),
        ],
    );
}

fn push_player_start_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        clip,
        order,
        color,
        opacity,
        &[
            local_rect(rect, 6.0, 1.0, 2.0, 3.0),
            local_rect(rect, 3.0, 4.0, 8.0, 2.0),
            local_rect(rect, 2.0, 7.0, 4.0, 4.0),
            local_rect(rect, 8.0, 7.0, 4.0, 4.0),
            local_rect(rect, 5.0, 11.0, 4.0, 2.0),
        ],
    );
}

fn push_audio_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        clip,
        order,
        color,
        opacity,
        &[
            local_rect(rect, 2.0, 5.0, 3.0, 4.0),
            local_rect(rect, 5.0, 3.0, 2.0, 8.0),
            local_rect(rect, 8.0, 4.0, 1.0, 2.0),
            local_rect(rect, 10.0, 3.0, 1.0, 4.0),
            local_rect(rect, 8.0, 8.0, 1.0, 2.0),
            local_rect(rect, 10.0, 7.0, 1.0, 4.0),
        ],
    );
}

fn push_eye_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        clip,
        order,
        color,
        opacity,
        &[
            local_rect(rect, 2.0, 6.0, 2.0, 2.0),
            local_rect(rect, 4.0, 4.0, 6.0, 1.0),
            local_rect(rect, 4.0, 9.0, 6.0, 1.0),
            local_rect(rect, 10.0, 6.0, 2.0, 2.0),
            local_rect(rect, 6.0, 6.0, 2.0, 2.0),
        ],
    );
}

fn push_lock_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        clip,
        order,
        color,
        opacity,
        &[
            local_rect(rect, 4.0, 6.0, 7.0, 6.0),
            local_rect(rect, 5.0, 3.0, 5.0, 1.0),
            local_rect(rect, 4.0, 4.0, 1.0, 3.0),
            local_rect(rect, 10.0, 4.0, 1.0, 3.0),
        ],
    );
}

fn push_kebab_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        clip,
        order,
        color,
        opacity,
        &[
            local_rect(rect, 6.0, 2.0, 2.0, 2.0),
            local_rect(rect, 6.0, 6.0, 2.0, 2.0),
            local_rect(rect, 6.0, 10.0, 2.0, 2.0),
        ],
    );
}

fn push_down_chevron(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        clip,
        order,
        color,
        opacity,
        &[
            local_rect(rect, 3.0, 4.0, 2.0, 2.0),
            local_rect(rect, 5.0, 6.0, 2.0, 2.0),
            local_rect(rect, 7.0, 4.0, 2.0, 2.0),
        ],
    );
}

fn push_right_chevron(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        clip,
        order,
        color,
        opacity,
        &[
            local_rect(rect, 4.0, 3.0, 2.0, 3.0),
            local_rect(rect, 6.0, 6.0, 2.0, 2.0),
            local_rect(rect, 4.0, 8.0, 2.0, 3.0),
        ],
    );
}

fn push_segments(
    commands: &mut Vec<HostPaintCommand>,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
    segments: &[FrameRect],
) {
    for segment in segments {
        commands.push(HostPaintCommand::quad(
            segment.clone(),
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

fn local_rect(origin: &FrameRect, x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x: origin.x + x,
        y: origin.y + y,
        width,
        height,
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::data::TemplateNodeFrameData;
    use super::super::style_selector::WORKBENCH_TREE_ROW_TEXT_SELECTED as TREE_TEXT_SELECTED;
    use super::super::template_nodes::paint_template_nodes_for_test;
    use super::*;
    use crate::ui::layouts::common::model_rc;
    use zircon_runtime_interface::ui::style::UiPainterResolvedState;

    #[test]
    fn tree_row_kind_matches_roles_and_scene_ids() {
        assert!(is_workbench_tree_row(&tree_node(
            "Custom", "TreeRow", "", "Root", 0, false
        )));
        assert!(is_workbench_tree_row(&tree_node(
            "WorkbenchScenePropsItem",
            "",
            "",
            "Props",
            2,
            true
        )));
        assert!(is_workbench_tree_row(&tree_node(
            "Custom", "", "tree-row", "Node", 0, false
        )));
        assert!(!is_workbench_tree_row(&TemplatePaneNodeData {
            control_id: "WorkbenchListSelected".into(),
            role: "ListRow".into(),
            component_role: "list-row".into(),
            ..TemplatePaneNodeData::default()
        }));
    }

    #[test]
    fn selected_tree_row_paints_surface_indent_icon_and_actions() {
        let bytes = paint_template_nodes_for_test(
            280,
            48,
            model_rc(vec![tree_node(
                "WorkbenchScenePropsItem",
                "TreeRow",
                "tree-row",
                "Props",
                2,
                true,
            )]),
        );

        assert_ne!(pixel_at(&bytes, 280, 14, 19), [0, 0, 0, 255]);
        assert!(changed_pixel_count(&bytes, 280, 50, 10, 40, 24) > 0);
        assert!(changed_pixel_count(&bytes, 280, 230, 13, 40, 18) > 0);
    }

    #[test]
    fn nested_tree_row_draws_indent_guides_without_full_surface() {
        let bytes = paint_template_nodes_for_test(
            240,
            42,
            model_rc(vec![tree_node(
                "WorkbenchSceneEnvironmentItem",
                "TreeRow",
                "tree-row",
                "Environment",
                1,
                false,
            )]),
        );

        assert_eq!(pixel_at(&bytes, 240, 8, 18), [0, 0, 0, 255]);
        assert_ne!(pixel_at(&bytes, 240, 21, 18), [0, 0, 0, 255]);
        assert!(changed_pixel_count(&bytes, 240, 32, 10, 48, 22) > 0);
    }

    #[test]
    fn tree_row_style_uses_shared_state_priority() {
        let mut node = tree_node(
            "WorkbenchScenePropsItem",
            "TreeRow",
            "tree-row",
            "Props",
            2,
            true,
        );
        node.hovered = true;
        node.focused = true;
        node.pressed = true;

        let pressed = tree_row_style(&node);
        assert_eq!(pressed.state, UiPainterResolvedState::Pressed);
        assert_eq!(pressed.background, Some(PALETTE.surface_selected));
        assert_eq!(pressed.border, Some(PALETTE.focus_ring));
        assert_eq!(pressed.text, TREE_TEXT_SELECTED);

        node.pressed = false;
        node.selected = false;
        node.checked = false;
        let focused = tree_row_style(&node);
        assert_eq!(focused.state, UiPainterResolvedState::Focused);
        assert_eq!(focused.background, Some(PALETTE.surface_hover));
        assert_eq!(focused.border, Some(PALETTE.focus_ring));

        node.disabled = true;
        let disabled = tree_row_style(&node);
        assert_eq!(disabled.state, UiPainterResolvedState::Disabled);
        assert_eq!(disabled.background, None);
        assert_eq!(disabled.border, None);
        assert_eq!(disabled.text, PALETTE.text_disabled);
    }

    #[test]
    fn collapsed_tree_row_paints_right_chevron() {
        let bytes = paint_template_nodes_for_test(
            240,
            42,
            model_rc(vec![tree_node(
                "WorkbenchScenePlayerStartItem",
                "TreeRow",
                "tree-row",
                "PlayerStart",
                0,
                false,
            )]),
        );

        assert!(changed_pixel_count(&bytes, 240, 14, 11, 14, 16) > 0);
        assert!(changed_pixel_count(&bytes, 240, 32, 10, 28, 22) > 0);
    }

    fn tree_node(
        control_id: &str,
        role: &str,
        component_role: &str,
        text: &str,
        depth: i32,
        selected: bool,
    ) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: control_id.into(),
            role: role.into(),
            component_role: component_role.into(),
            text: text.into(),
            tree_depth: depth,
            tree_indent_px: if selected { 40.0 } else { 0.0 },
            selected,
            checked: selected,
            expanded: !text.contains("Player"),
            frame: TemplateNodeFrameData {
                x: 4.0,
                y: 6.0,
                width: 268.0,
                height: 30.0,
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
