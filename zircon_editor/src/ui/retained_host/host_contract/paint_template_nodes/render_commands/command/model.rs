use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::super::data::FrameRect;
use super::super::super::visual_assets::HostPaintImagePixels;
use super::kind::HostPaintCommandKind;

#[derive(Clone)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct HostPaintCommand {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) kind:
        HostPaintCommandKind,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) frame: FrameRect,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) clip_frame:
        Option<FrameRect>,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) z_index: i32,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) background_color:
        Option<[u8; 4]>,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) foreground_color:
        Option<[u8; 4]>,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) border_color:
        Option<[u8; 4]>,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) border_width: f32,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) corner_radius: f32,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) text: Option<String>,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) font_size: f32,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) line_height: f32,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) text_style:
        UiTextRunPaintStyle,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) image_key: Option<String>,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) image_pixels:
        Option<HostPaintImagePixels>,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) opacity: f32,
}
