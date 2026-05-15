use crate::ui::retained_host::primitives::ModelRc;

use super::super::data::{FrameRect, HostTextInputFocusData, TemplatePaneNodeData};
use super::frame::HostRgbaFrame;
use super::geometry::{frame_from_template, intersect, is_visible_frame, translated};
use super::render_commands::{draw_host_paint_commands, HostPaintCommand};
use super::theme::PALETTE;
use super::visual_assets::{raster_size_from_frame, template_image_pixels, template_image_tint};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const DEFAULT_TEMPLATE_FONT_SIZE: f32 = 12.0;
const TEXT_HORIZONTAL_INSET: f32 = 5.0;
const TEXT_VERTICAL_INSET: f32 = 5.0;
const MIN_TEXT_RECT_HEIGHT: f32 = 12.0;

pub(super) fn draw_template_nodes(
    frame: &mut HostRgbaFrame,
    nodes: &ModelRc<TemplatePaneNodeData>,
    origin: &FrameRect,
    clip: &FrameRect,
    text_input_focus: Option<&HostTextInputFocusData>,
) -> bool {
    let Some(effective_clip) = effective_template_clip(frame, clip) else {
        return false;
    };

    let mut commands = Vec::new();
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "template_nodes_collect_commands");
        zircon_runtime::profile_counter!("editor", "template_node_count", nodes.row_count());
        for row in 0..nodes.row_count() {
            let Some(node) = nodes.row_data(row) else {
                continue;
            };
            // Region repaint must avoid generating commands for off-damage nodes:
            // image commands can rasterize previews before the final primitive clip runs.
            push_template_node_commands(
                &mut commands,
                &node,
                origin,
                &effective_clip,
                text_input_focus,
                row as i32,
            );
        }
    }
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "template_nodes_draw_commands");
        zircon_runtime::profile_counter!("editor", "template_command_count", commands.len());
        draw_host_paint_commands(frame, &commands)
    }
}

fn effective_template_clip(frame: &HostRgbaFrame, clip: &FrameRect) -> Option<FrameRect> {
    match frame.paint_clip() {
        Some(active_clip) => intersect(active_clip, clip),
        None if is_visible_frame(clip) => Some(clip.clone()),
        None => None,
    }
}

pub(super) fn has_template_nodes(nodes: &ModelRc<TemplatePaneNodeData>) -> bool {
    nodes.row_count() > 0
}

#[cfg(test)]
pub(crate) fn paint_template_nodes_for_test(
    width: u32,
    height: u32,
    nodes: ModelRc<TemplatePaneNodeData>,
) -> Vec<u8> {
    let mut frame = HostRgbaFrame::filled(width, height, [0, 0, 0, 255]);
    let bounds = FrameRect {
        x: 0.0,
        y: 0.0,
        width: width as f32,
        height: height as f32,
    };
    draw_template_nodes(&mut frame, &nodes, &bounds, &bounds, None);
    frame.into_bytes()
}

fn push_template_node_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    origin: &FrameRect,
    clip: &FrameRect,
    text_input_focus: Option<&HostTextInputFocusData>,
    order: i32,
) {
    let local = frame_from_template(&node.frame);
    let rect = translated(&local, origin.x, origin.y);
    if !is_visible_frame(&rect) {
        return;
    }
    let Some(node_clip) = template_node_clip(node, origin, clip) else {
        return;
    };
    if intersect(&rect, &node_clip).is_none() {
        return;
    }

    if draws_surface(node) {
        commands.push(HostPaintCommand::quad(
            rect.clone(),
            Some(node_clip.clone()),
            order * 4,
            Some(surface_color(node)),
            draws_border(node).then_some(border_color(node)),
            node.border_width.max(0.0),
            node.corner_radius.max(0.0),
            1.0,
        ));
    }

    push_template_image_command(commands, node, &rect, &node_clip, order * 4 + 1);

    let label = node_label(node, text_input_focus);
    if (!label.is_empty() && !is_icon_only_node(node))
        || matches!(node.role.as_str(), "Label" | "Button")
    {
        let text_rect = text_rect_for_node(node, &rect);
        let font_size = node_font_size(node, text_rect.height);
        commands.push(HostPaintCommand::text(
            FrameRect {
                x: text_rect.x,
                y: text_rect.y,
                width: text_rect.width,
                height: text_rect.height,
            },
            Some(node_clip),
            order * 4 + 2,
            label,
            text_color(node),
            font_size,
            font_size * 1.2,
            UiTextRunPaintStyle::default(),
            1.0,
        ));
    }
}

fn template_node_clip(
    node: &TemplatePaneNodeData,
    origin: &FrameRect,
    pane_clip: &FrameRect,
) -> Option<FrameRect> {
    let node_clip = if node.has_clip_frame {
        translated(&frame_from_template(&node.clip_frame), origin.x, origin.y)
    } else {
        pane_clip.clone()
    };
    intersect(&node_clip, pane_clip)
}

fn push_template_image_command(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
) {
    if !node.has_preview_image {
        return;
    }
    let preview_size = node.preview_image.size();
    let image_rect = image_rect_for_node(node, rect, preview_size.width, preview_size.height);
    if !is_visible_frame(&image_rect) {
        return;
    }
    if intersect(&image_rect, clip).is_none() {
        return;
    }
    let Some((target_width, target_height)) =
        raster_size_from_frame(image_rect.width, image_rect.height)
    else {
        return;
    };
    let tint = template_image_tint(
        is_icon_node(node),
        node.selected || node.focused || node.pressed,
        node.disabled,
        node.text_tone.as_str(),
        node.validation_level.as_str(),
    );
    let image = {
        zircon_runtime::profile_scope!("editor", "host_painter", "template_node_image_pixels");
        template_image_pixels(
            &node.preview_image,
            node.media_source.as_str(),
            node.icon_name.as_str(),
            target_width,
            target_height,
            tint,
            !is_icon_node(node),
        )
    };
    let Some(image) = image else {
        return;
    };
    commands.push(HostPaintCommand::image_pixels(
        image_rect,
        Some(clip.clone()),
        order,
        image.resource_key,
        image.width,
        image.height,
        image.rgba,
        1.0,
    ));
}

fn image_rect_for_node(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    image_width: u32,
    image_height: u32,
) -> FrameRect {
    if is_icon_node(node) {
        let label = node_label(node, None);
        if !label.is_empty() && !is_icon_only_node(node) {
            let size = leading_icon_size(rect);
            return FrameRect {
                x: rect.x + TEXT_HORIZONTAL_INSET,
                y: rect.y + (rect.height - size) * 0.5,
                width: size,
                height: size,
            };
        }
        let inset = (rect.width.min(rect.height) * 0.16).min(4.0).max(0.0);
        let size = (rect.width.min(rect.height) - inset * 2.0).max(1.0);
        return FrameRect {
            x: rect.x + (rect.width - size) * 0.5,
            y: rect.y + (rect.height - size) * 0.5,
            width: size,
            height: size,
        };
    }
    fitted_image_rect(rect, image_width, image_height)
}

fn is_icon_node(node: &TemplatePaneNodeData) -> bool {
    matches!(node.role.as_str(), "Icon" | "IconButton" | "SvgIcon") || !node.icon_name.is_empty()
}

fn is_icon_only_node(node: &TemplatePaneNodeData) -> bool {
    matches!(node.role.as_str(), "Icon" | "IconButton" | "SvgIcon")
}

fn fitted_image_rect(rect: &FrameRect, image_width: u32, image_height: u32) -> FrameRect {
    if image_width == 0 || image_height == 0 || rect.width <= 0.0 || rect.height <= 0.0 {
        return rect.clone();
    }
    let image_aspect = image_width as f32 / image_height as f32;
    let rect_aspect = rect.width / rect.height;
    if rect_aspect > image_aspect {
        let height = rect.height;
        let width = height * image_aspect;
        FrameRect {
            x: rect.x + (rect.width - width) * 0.5,
            y: rect.y,
            width,
            height,
        }
    } else {
        let width = rect.width;
        let height = width / image_aspect;
        FrameRect {
            x: rect.x,
            y: rect.y + (rect.height - height) * 0.5,
            width,
            height,
        }
    }
}

fn text_rect_for_node(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    let horizontal = TEXT_HORIZONTAL_INSET
        .min((rect.width * 0.25).max(0.0))
        .max(0.0);
    let vertical = TEXT_VERTICAL_INSET
        .min(((rect.height - MIN_TEXT_RECT_HEIGHT) * 0.5).max(1.0))
        .max(0.0);
    let mut x = rect.x + horizontal;
    let mut width = (rect.width - horizontal * 2.0).max(0.0);
    if is_leading_icon_text_node(node) {
        let leading = (leading_icon_size(rect) + TEXT_HORIZONTAL_INSET).min(width);
        x += leading;
        width = (width - leading).max(0.0);
    }
    FrameRect {
        x,
        y: rect.y + vertical,
        width,
        height: (rect.height - vertical * 2.0).max(0.0),
    }
}

fn is_leading_icon_text_node(node: &TemplatePaneNodeData) -> bool {
    is_icon_node(node) && !is_icon_only_node(node) && !node_label(node, None).is_empty()
}

fn leading_icon_size(rect: &FrameRect) -> f32 {
    (rect.height - TEXT_VERTICAL_INSET * 2.0)
        .min(rect.width * 0.28)
        .max(1.0)
}

fn node_font_size(node: &TemplatePaneNodeData, available_height: f32) -> f32 {
    let requested = if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
    } else {
        DEFAULT_TEMPLATE_FONT_SIZE
    };
    requested.min(available_height.max(1.0)).max(1.0)
}

fn draws_surface(node: &TemplatePaneNodeData) -> bool {
    matches!(node.role.as_str(), "Panel" | "Button" | "Mount")
        || !node.surface_variant.is_empty()
        || !node.button_variant.is_empty()
        || node.border_width > 0.0
        || node.corner_radius > 0.0
        || node.selected
        || node.hovered
        || node.pressed
        || node.focused
        || node.disabled
}

fn draws_border(node: &TemplatePaneNodeData) -> bool {
    node.border_width > 0.0
        || node.corner_radius > 0.0
        || node.selected
        || node.checked
        || node.focused
        || node.hovered
        || node.pressed
        || node.drop_hovered
        || node.active_drag_target
        || matches!(node.role.as_str(), "Button" | "Mount")
}

fn surface_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
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
    if node.pressed {
        return PALETTE.surface_pressed;
    }
    if node.selected || node.checked || node.focused {
        return PALETTE.surface_selected;
    }
    if matches!(node.button_variant.as_str(), "primary" | "filled")
        || matches!(node.surface_variant.as_str(), "accent" | "primary")
    {
        if node.hovered || node.drop_hovered || node.active_drag_target {
            return PALETTE.accent_soft;
        }
        return PALETTE.accent;
    }
    if node.hovered || node.drop_hovered || node.active_drag_target {
        return PALETTE.surface_hover;
    }
    match node.surface_variant.as_str() {
        "inset" | "scroll-body" | "asset-tree-row" | "reference-row" => PALETTE.surface_inset,
        "popup" | "elevated" => PALETTE.popup,
        "panel" | "asset-preview" | "asset-preview-visual" => PALETTE.surface,
        "shell" => PALETTE.shell_background,
        _ => match node.role.as_str() {
            "Button" if node.surface_variant.is_empty() => PALETTE.surface_hover,
            _ => PALETTE.surface,
        },
    }
}

fn border_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
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
    if node.selected || node.checked || node.focused || node.pressed {
        PALETTE.focus_ring
    } else if matches!(node.button_variant.as_str(), "primary" | "filled")
        || matches!(node.surface_variant.as_str(), "accent" | "primary")
        || node.hovered
        || node.drop_hovered
        || node.active_drag_target
    {
        PALETTE.focus_ring
    } else {
        PALETTE.border
    }
}

fn text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        return PALETTE.text_disabled;
    }
    match node.text_tone.as_str() {
        "muted" | "subtle" => PALETTE.text_muted,
        "accent" | "primary" | "default" => PALETTE.focus_ring,
        "warning" => PALETTE.warning,
        "error" | "danger" => PALETTE.error,
        "success" => PALETTE.success,
        "info" => PALETTE.info,
        _ => PALETTE.text,
    }
}

fn node_label(
    node: &TemplatePaneNodeData,
    text_input_focus: Option<&HostTextInputFocusData>,
) -> String {
    if let Some(focus) = focused_text_value(node, text_input_focus) {
        return focus.to_string();
    }
    let values = if is_text_input_node(node) {
        [
            node.value_text.as_str(),
            node.text.as_str(),
            node.options_text.as_str(),
        ]
    } else {
        [
            node.text.as_str(),
            node.value_text.as_str(),
            node.options_text.as_str(),
        ]
    };
    first_non_empty(&values).to_string()
}

fn focused_text_value<'a>(
    node: &TemplatePaneNodeData,
    text_input_focus: Option<&'a HostTextInputFocusData>,
) -> Option<&'a str> {
    let focus = text_input_focus?;
    (focus.is_active() && focus.control_id.as_str() == node.control_id.as_str())
        .then_some(focus.value_text.as_str())
}

fn is_text_input_node(node: &TemplatePaneNodeData) -> bool {
    matches!(node.component_role.as_str(), "input-field" | "number-field")
        || matches!(node.role.as_str(), "InputField" | "LineEdit")
        || !node.edit_action_id.is_empty()
        || !node.commit_action_id.is_empty()
}

fn first_non_empty<'a>(values: &[&'a str]) -> &'a str {
    values
        .iter()
        .copied()
        .find(|value| !value.trim().is_empty())
        .unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::super::super::data::TemplateNodeFrameData;
    use super::*;
    use crate::ui::layouts::common::model_rc;

    #[test]
    fn template_nodes_skip_when_active_paint_clip_misses_template_clip() {
        let mut frame = HostRgbaFrame::filled(32, 32, [1, 2, 3, 255]);
        let before = frame.as_bytes().to_vec();
        frame.replace_paint_clip(Some(rect(24.0, 24.0, 4.0, 4.0)));

        let bounds = rect(0.0, 0.0, 16.0, 16.0);
        let painted = draw_template_nodes(
            &mut frame,
            &model_rc(vec![panel_node("outside", 0.0, 0.0, 8.0, 8.0)]),
            &bounds,
            &bounds,
            None,
        );

        assert!(!painted);
        assert_eq!(frame.as_bytes(), before.as_slice());
    }

    #[test]
    fn template_nodes_only_paint_nodes_intersecting_active_damage_clip() {
        let mut frame = HostRgbaFrame::filled(40, 20, [0, 0, 0, 255]);
        frame.replace_paint_clip(Some(rect(20.0, 0.0, 10.0, 10.0)));

        let bounds = rect(0.0, 0.0, 40.0, 20.0);
        let painted = draw_template_nodes(
            &mut frame,
            &model_rc(vec![
                panel_node("left", 0.0, 0.0, 10.0, 10.0),
                panel_node("damage", 20.0, 0.0, 10.0, 10.0),
            ]),
            &bounds,
            &bounds,
            None,
        );

        assert!(painted);
        assert_eq!(changed_pixel_count(frame.as_bytes(), 40, 0, 0, 10, 10), 0);
        assert!(changed_pixel_count(frame.as_bytes(), 40, 20, 0, 10, 10) > 0);
    }

    fn panel_node(
        control_id: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: control_id.into(),
            role: "Panel".into(),
            surface_variant: "panel".into(),
            frame: TemplateNodeFrameData {
                x,
                y,
                width,
                height,
            },
            ..TemplatePaneNodeData::default()
        }
    }

    fn rect(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
        FrameRect {
            x,
            y,
            width,
            height,
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
}
