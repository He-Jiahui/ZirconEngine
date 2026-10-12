//! 轴标签文字和比例链接图标共用当前宿主密度快照，以维持字体和链接自然尺寸的同源关系。

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct AxisLabelMetrics {
    pub font_size: f32,
    pub line_height: f32,
    pub link_lobe_width: f32,
    pub link_lobe_height: f32,
    pub link_lobe_radius: f32,
    pub link_overlap: f32,
    pub link_connector_width: f32,
}
