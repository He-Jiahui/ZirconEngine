use crate::ui::retained_host::primitives::ModelRc;
use std::f32::consts::PI;

use super::super::data::{FrameRect, HostTextInputFocusData, TemplatePaneNodeData};
use super::frame::HostRgbaFrame;
use super::geometry::{frame_from_template, intersect, is_visible_frame, translated};
use super::material_primitives::{
    push_material_primitive_commands, push_material_text_field_surface_commands,
};
use super::material_state_layer::push_state_layer_commands;
use super::mui_x_primitives::push_mui_x_primitive_commands;
use super::render_commands::{draw_host_paint_commands, HostPaintCommand};
use super::template_alerts::push_alert_commands;
use super::template_axis_labels::push_axis_label_commands;
use super::template_axis_value_fields::push_axis_value_field_commands;
use super::template_buttons::push_button_commands;
use super::template_chips::push_chip_commands;
use super::template_dropdowns::{dropdown_paint_rect, push_dropdown_commands};
use super::template_fields::push_field_commands;
use super::template_icon_buttons::push_icon_button_commands;
use super::template_inspector_rows::push_inspector_row_commands;
use super::template_list_rows::push_list_row_commands;
use super::template_node_labels::template_node_label;
use super::template_popup_rows::push_template_popup_row_commands;
use super::template_property_rows::push_property_row_text_commands;
use super::template_section_titles::push_section_title_commands;
use super::template_segmented_controls::push_segmented_control_commands;
use super::template_selection_controls::push_selection_control_commands;
use super::template_shell_panels::push_shell_panel_commands;
use super::template_sliders::push_slider_commands;
use super::template_status_controls::push_status_control_commands;
use super::template_style::{
    border_color, draws_elevation_shadow, elevation_shadow_rect, is_mui_overlay_surface_node,
    resolved_style_color, surface_color, template_border_width, template_corner_radius, text_color,
};
use super::template_table_rows::{push_table_row_commands, push_table_row_text_commands};
use super::template_tooltips::push_tooltip_commands;
use super::template_tree_rows::push_tree_row_commands;
use super::template_viewport_scene::push_viewport_scene_commands;
use super::theme::PALETTE;
use super::visual_assets::{raster_size_from_frame, template_image_pixels, template_image_tint};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const DEFAULT_TEMPLATE_FONT_SIZE: f32 = 12.0;
const TEXT_HORIZONTAL_INSET: f32 = 5.0;
const TEXT_VERTICAL_INSET: f32 = 5.0;
const MIN_TEXT_RECT_HEIGHT: f32 = 12.0;
const MATERIAL_ELEVATION_SHADOW_OPACITY: f32 = 0.72;
const MATERIAL_PROGRESS_TRACK: [u8; 4] = [42, 52, 60, 255];
const MUI_BACKDROP_SCRIM: [u8; 4] = [0, 0, 0, 128];
const TEMPLATE_NODE_ORDER_STRIDE: i32 = 4;
const TEMPLATE_NODE_Z_LAYER_STRIDE: i32 = 100_000;

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
    paint_template_nodes_for_test_with_background(width, height, [0, 0, 0, 255], nodes)
}

#[cfg(test)]
pub(crate) fn paint_template_nodes_for_test_with_background(
    width: u32,
    height: u32,
    background: [u8; 4],
    nodes: ModelRc<TemplatePaneNodeData>,
) -> Vec<u8> {
    let mut frame = HostRgbaFrame::filled(width, height, background);
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

    let order = template_node_paint_order(node, order);
    let opacity = template_node_transition_opacity(node);
    if opacity <= 0.0 {
        return;
    }

    if push_material_feedback_primitive_commands(commands, node, &rect, &node_clip, order, opacity)
    {
        return;
    }

    if push_shell_panel_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_selection_control_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_segmented_control_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_button_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_dropdown_commands(commands, node, &rect, &node_clip, order, opacity) {
        let popup_anchor = dropdown_paint_rect(node, &rect);
        push_template_popup_row_commands(
            commands,
            node,
            &popup_anchor,
            origin,
            clip,
            order,
            opacity,
        );
        return;
    }

    if push_tree_row_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_list_row_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_table_row_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_status_control_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_chip_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_viewport_scene_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_section_title_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_icon_button_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_inspector_row_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_axis_value_field_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_axis_label_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_field_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_slider_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_alert_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_tooltip_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_material_primitive_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    let draws_mui_x_primitive =
        push_mui_x_primitive_commands(commands, node, &rect, &node_clip, order, opacity);

    let draws_text_field_surface = push_material_text_field_surface_commands(
        commands, node, &rect, &node_clip, order, opacity,
    );

    if !draws_mui_x_primitive && !draws_text_field_surface && draws_surface(node) {
        let border_width = template_border_width(node);
        let corner_radius = template_corner_radius(node);
        if draws_elevation_shadow(node) {
            commands.push(HostPaintCommand::quad(
                elevation_shadow_rect(&rect, node.elevation),
                Some(node_clip.clone()),
                order - 1,
                Some(PALETTE.shadow),
                None,
                0.0,
                corner_radius,
                MATERIAL_ELEVATION_SHADOW_OPACITY * opacity,
            ));
        }
        commands.push(HostPaintCommand::quad(
            rect.clone(),
            Some(node_clip.clone()),
            order,
            Some(surface_color(node)),
            draws_border(node).then_some(border_color(node)),
            border_width,
            corner_radius,
            opacity,
        ));
        push_state_layer_commands(
            commands,
            node,
            &rect,
            &node_clip,
            corner_radius,
            order + 1,
            opacity,
        );
    }

    push_template_image_command(commands, node, &rect, &node_clip, order + 2, opacity);

    let property_row_text_painted =
        push_property_row_text_commands(commands, node, &rect, &node_clip, order + 3, opacity);
    let table_row_text_painted = !property_row_text_painted
        && push_table_row_text_commands(commands, node, &rect, &node_clip, order + 3, opacity);

    let label = template_node_label(node, text_input_focus);
    if !property_row_text_painted
        && !table_row_text_painted
        && ((!label.is_empty() && !is_icon_only_node(node))
            || matches!(node.role.as_str(), "Label" | "Button"))
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
            order + 3,
            label,
            text_color(node),
            font_size,
            font_size * 1.2,
            UiTextRunPaintStyle::default(),
            opacity,
        ));
    }

    push_template_popup_row_commands(commands, node, &rect, origin, clip, order, opacity);
}

fn template_node_paint_order(node: &TemplatePaneNodeData, row_order: i32) -> i32 {
    node.z_index
        .saturating_mul(TEMPLATE_NODE_Z_LAYER_STRIDE)
        .saturating_add(row_order.saturating_mul(TEMPLATE_NODE_ORDER_STRIDE))
}

fn template_node_transition_opacity(node: &TemplatePaneNodeData) -> f32 {
    match node.transition_kind.as_str() {
        "fade" | "grow" | "zoom" => node.transition_progress.clamp(0.0, 1.0),
        _ => 1.0,
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
    opacity: f32,
) {
    if !template_node_has_image_source(node) {
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
        resolved_style_color(node.button_style.element.foreground_color.as_ref()),
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
        image.atlas,
        opacity,
    ));
}

fn template_node_has_image_source(node: &TemplatePaneNodeData) -> bool {
    node.has_preview_image || !node.media_source.is_empty() || !node.icon_name.is_empty()
}

fn push_material_feedback_primitive_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if is_material_backdrop_node(node) {
        push_material_backdrop_commands(commands, node, rect, clip, order, opacity);
        return true;
    }
    if is_material_progress_node(node) {
        push_material_progress_commands(commands, node, rect, clip, order, opacity);
        return true;
    }
    false
}

fn push_material_backdrop_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if !node.popup_open
        && node.surface_variant.as_str() != "backdrop"
        && !component_variant_contains(node, "open")
    {
        return;
    }
    if component_variant_contains(node, "invisible") {
        return;
    }
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(
            resolved_style_color(node.button_style.element.background_color.as_ref())
                .unwrap_or(MUI_BACKDROP_SCRIM),
        ),
        None,
        0.0,
        0.0,
        opacity,
    ));
}

fn push_material_progress_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if progress_is_circular(node) {
        push_circular_progress_command(commands, node, rect, clip, order, opacity);
    } else {
        push_linear_progress_commands(commands, node, rect, clip, order, opacity);
    }
}

fn push_linear_progress_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let radius = template_corner_radius(node)
        .max((rect.height * 0.5).min(2.0))
        .max(0.0);
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(progress_track_color(node)),
        None,
        0.0,
        radius,
        opacity,
    ));

    let fill = progress_fill_color(node);
    if progress_is_indeterminate(node) {
        for (x_factor, width_factor) in [(0.12, 0.36), (0.62, 0.24)] {
            let bar = FrameRect {
                x: rect.x + rect.width * x_factor,
                y: rect.y,
                width: (rect.width * width_factor).max(1.0),
                height: rect.height,
            };
            commands.push(HostPaintCommand::quad(
                bar,
                Some(clip.clone()),
                order + 1,
                Some(fill),
                None,
                0.0,
                radius,
                opacity,
            ));
        }
        return;
    }

    let width = rect.width * progress_percent(node);
    if width <= 0.0 {
        return;
    }
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x,
            y: rect.y,
            width: width.max(1.0),
            height: rect.height,
        },
        Some(clip.clone()),
        order + 1,
        Some(fill),
        None,
        0.0,
        radius,
        opacity,
    ));
}

fn push_circular_progress_command(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let image_rect = circular_progress_rect(rect);
    let Some((width, height)) = raster_size_from_frame(image_rect.width, image_rect.height) else {
        return;
    };
    let size = width.min(height);
    if size == 0 {
        return;
    }
    let rgba = circular_progress_pixels(
        size,
        if progress_is_indeterminate(node) {
            0.58
        } else {
            progress_percent(node)
        },
        progress_track_color(node),
        progress_fill_color(node),
    );
    commands.push(HostPaintCommand::image_pixels(
        image_rect,
        Some(clip.clone()),
        order,
        format!(
            "mui-circular-progress:{size}:{:.3}:{}:{}",
            progress_percent(node),
            progress_track_color(node)[0],
            progress_fill_color(node)[0]
        ),
        size,
        size,
        rgba,
        None,
        opacity,
    ));
}

fn is_material_progress_node(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.component_role.as_str(),
        "progress" | "progress-bar" | "linear-progress" | "circular-progress" | "spinner"
    ) || matches!(
        node.role.as_str(),
        "Progress" | "ProgressBar" | "LinearProgress" | "CircularProgress" | "Spinner"
    )
}

fn is_material_backdrop_node(node: &TemplatePaneNodeData) -> bool {
    node.component_role.as_str() == "backdrop"
        || node.role.as_str() == "Backdrop"
        || node.surface_variant.as_str() == "backdrop"
}

fn progress_is_circular(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.component_role.as_str(),
        "circular-progress" | "spinner"
    ) || matches!(node.role.as_str(), "CircularProgress" | "Spinner")
        || component_variant_contains(node, "circular")
}

fn progress_is_indeterminate(node: &TemplatePaneNodeData) -> bool {
    matches!(node.component_role.as_str(), "spinner")
        || component_variant_contains(node, "indeterminate")
}

fn progress_percent(node: &TemplatePaneNodeData) -> f32 {
    if node.value_percent.is_finite() {
        node.value_percent.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn progress_track_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        return PALETTE.surface_disabled;
    }
    resolved_style_color(node.button_style.element.background_color.as_ref())
        .unwrap_or(MATERIAL_PROGRESS_TRACK)
}

fn progress_fill_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        return PALETTE.text_disabled;
    }
    resolved_style_color(node.button_style.element.foreground_color.as_ref())
        .or_else(|| material_tone_color(node))
        .unwrap_or(PALETTE.accent)
}

fn material_tone_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    let tone = [node.validation_level.as_str(), node.text_tone.as_str()]
        .into_iter()
        .find(|value| !value.trim().is_empty())
        .unwrap_or("");
    match tone {
        "warning" => Some(PALETTE.warning),
        "error" | "danger" => Some(PALETTE.error),
        "success" => Some(PALETTE.success),
        "info" => Some(PALETTE.info),
        "accent" | "primary" => Some(PALETTE.accent),
        _ => None,
    }
}

fn circular_progress_rect(rect: &FrameRect) -> FrameRect {
    let size = rect.width.min(rect.height).max(1.0);
    FrameRect {
        x: rect.x + (rect.width - size) * 0.5,
        y: rect.y + (rect.height - size) * 0.5,
        width: size,
        height: size,
    }
}

fn circular_progress_pixels(size: u32, percent: f32, track: [u8; 4], fill: [u8; 4]) -> Vec<u8> {
    let mut rgba = vec![0; size as usize * size as usize * 4];
    let center = size as f32 * 0.5;
    let radius = (size as f32 * 0.5 - 0.5).max(1.0);
    let thickness = (size as f32 * 0.16).clamp(3.0, 6.0);
    let inner = (radius - thickness).max(0.0);
    let percent = percent.clamp(0.0, 1.0);
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 + 0.5 - center;
            let dy = y as f32 + 0.5 - center;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance < inner || distance > radius {
                continue;
            }
            let angle = dy.atan2(dx);
            let turn = ((angle + PI * 0.5).rem_euclid(PI * 2.0)) / (PI * 2.0);
            let color = if turn <= percent { fill } else { track };
            let offset = ((y as usize * size as usize) + x as usize) * 4;
            rgba[offset..offset + 4].copy_from_slice(&color);
        }
    }
    rgba
}

fn component_variant_contains(node: &TemplatePaneNodeData, expected: &str) -> bool {
    node.component_variant
        .as_str()
        .split(|character: char| {
            character.is_ascii_whitespace() || matches!(character, ',' | '/' | '|' | ':' | ';')
        })
        .any(|part| part.eq_ignore_ascii_case(expected))
}

fn image_rect_for_node(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    image_width: u32,
    image_height: u32,
) -> FrameRect {
    if is_icon_node(node) {
        let label = template_node_label(node, None);
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
    is_icon_node(node) && !is_icon_only_node(node) && !template_node_label(node, None).is_empty()
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
        || is_mui_overlay_surface_node(node)
        || !node.surface_variant.is_empty()
        || !node.button_variant.is_empty()
        || node.button_style.element.background_color.is_some()
        || node.button_style.element.border_color.is_some()
        || node.button_style.element.border_width > 0.0
        || node.button_style.element.corner_radius > 0.0
        || node.border_width > 0.0
        || node.corner_radius > 0.0
        || node.selected
        || node.hovered
        || node.pressed
        || node.focused
        || node.state_layer_enabled
        || node.ripple_enabled
        || node.disabled
}

fn draws_border(node: &TemplatePaneNodeData) -> bool {
    node.button_style.element.border_width > 0.0
        || node.button_style.element.border_color.is_some()
        || node.border_width > 0.0
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

#[cfg(test)]
mod tests {
    use super::super::super::data::{
        TemplateNodeFrameData, TemplatePaneMenuItemData, TemplatePaneOptionData,
    };
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

    #[test]
    fn template_nodes_paint_open_dropdown_option_rows_below_control() {
        let bytes = paint_template_nodes_for_test(128, 128, model_rc(vec![dropdown_node()]));

        assert!(changed_pixel_count(&bytes, 128, 12, 48, 112, 66) > 0);
    }

    #[test]
    fn template_nodes_anchor_workbench_dropdown_popup_to_declared_layout_offset() {
        let mut node = dropdown_node();
        node.control_id = "WorkbenchInputDropdown".into();
        node.layout_offset_x = 10.0;
        node.layout_offset_y = 6.0;
        let bytes = paint_template_nodes_for_test(160, 160, model_rc(vec![node]));

        assert!(changed_pixel_count(&bytes, 160, 22, 54, 112, 66) > 0);
        assert_eq!(changed_pixel_count(&bytes, 160, 12, 44, 8, 84), 0);
    }

    #[test]
    fn template_nodes_paint_open_dropdown_option_rows_above_control_when_below_clipped() {
        let bytes =
            paint_template_nodes_for_test(160, 160, model_rc(vec![dropdown_near_bottom_node()]));

        assert!(changed_pixel_count(&bytes, 160, 20, 32, 100, 84) > 0);
        assert_eq!(changed_pixel_count(&bytes, 160, 20, 152, 100, 8), 0);
    }

    #[test]
    fn template_nodes_paint_open_popup_menu_rows_inside_menu_frame() {
        let bytes = paint_template_nodes_for_test(160, 128, model_rc(vec![popup_menu_node()]));

        assert!(changed_pixel_count(&bytes, 160, 16, 16, 128, 96) > 0);
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

    fn dropdown_node() -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: "Dropdown".into(),
            role: "Dropdown".into(),
            component_role: "dropdown".into(),
            popup_open: true,
            frame: TemplateNodeFrameData {
                x: 12.0,
                y: 12.0,
                width: 112.0,
                height: 28.0,
            },
            structured_options: model_rc(vec![
                option("dropdown", true, false, false, false),
                option("option_a", false, true, true, false),
                option("option_b", false, false, false, true),
            ]),
            ..TemplatePaneNodeData::default()
        }
    }

    fn dropdown_near_bottom_node() -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: "Dropdown".into(),
            role: "Dropdown".into(),
            component_role: "dropdown".into(),
            popup_open: true,
            frame: TemplateNodeFrameData {
                x: 20.0,
                y: 120.0,
                width: 100.0,
                height: 28.0,
            },
            structured_options: model_rc(vec![
                option("dropdown", true, false, false, false),
                option("option_a", false, true, true, false),
                option("option_b", false, false, false, false),
            ]),
            ..TemplatePaneNodeData::default()
        }
    }

    fn popup_menu_node() -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: "PopupMenu".into(),
            role: "Menu".into(),
            component_role: "menu".into(),
            popup_open: true,
            frame: TemplateNodeFrameData {
                x: 16.0,
                y: 16.0,
                width: 128.0,
                height: 96.0,
            },
            structured_menu_items: model_rc(vec![
                menu_item("New", false, false, false),
                menu_item("Open", false, false, false),
                menu_item("Save", true, false, false),
                menu_item("", false, true, false),
                menu_item("Delete", false, false, true),
            ]),
            ..TemplatePaneNodeData::default()
        }
    }

    fn option(
        id: &str,
        selected: bool,
        hovered: bool,
        special: bool,
        disabled: bool,
    ) -> TemplatePaneOptionData {
        TemplatePaneOptionData {
            id: id.into(),
            label: id.into(),
            selected,
            hovered,
            special,
            disabled,
            ..TemplatePaneOptionData::default()
        }
    }

    fn menu_item(
        action_id: &str,
        checked: bool,
        separator: bool,
        hovered: bool,
    ) -> TemplatePaneMenuItemData {
        TemplatePaneMenuItemData {
            action_id: action_id.into(),
            label: action_id.into(),
            checked,
            separator,
            disabled: separator,
            hovered,
            ..TemplatePaneMenuItemData::default()
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
