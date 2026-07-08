use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;

mod geometry;
mod layers;
mod style;

use geometry::{
    folder_body_rect, folder_tab_rect, save_body_rect, save_bottom_cutout_rect,
    save_top_cutout_rect,
};
use layers::save_cutout_order;
use style::popup_adornment_asset_style;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_folder_adornment(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let style = popup_adornment_asset_style(color);
    commands.push(HostPaintCommand::quad(
        folder_body_rect(rect),
        Some(clip.clone()),
        order,
        Some(style.fill),
        style.border,
        style.border_width,
        style.body_radius,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        folder_tab_rect(rect),
        Some(clip.clone()),
        order,
        Some(style.fill),
        style.border,
        style.border_width,
        style.tab_radius,
        opacity,
    ));
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_save_adornment(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let style = popup_adornment_asset_style(color);
    commands.push(HostPaintCommand::quad(
        save_body_rect(rect),
        Some(clip.clone()),
        order,
        Some(style.fill),
        style.border,
        style.border_width,
        style.body_radius,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        save_top_cutout_rect(rect),
        Some(clip.clone()),
        save_cutout_order(order),
        Some(style.cutout_fill),
        style.border,
        style.border_width,
        style.cutout_radius,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        save_bottom_cutout_rect(rect),
        Some(clip.clone()),
        save_cutout_order(order),
        Some(style.cutout_fill),
        style.border,
        style.border_width,
        style.cutout_radius,
        opacity,
    ));
}
