use super::super::super::super::super::data::FrameRect;

// 根标签和覆盖层共用已测量文本框结构，将区域和字号交给各自命令生成器。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct BadgeTextFrame {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) rect: FrameRect,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) font_size: f32,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) line_height: f32,
}
