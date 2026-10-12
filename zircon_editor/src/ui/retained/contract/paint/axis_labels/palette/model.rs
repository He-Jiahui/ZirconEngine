//! 轴、Scale 轴与链接标记分别持有正常和禁用色，供同一变换面板中的文本及资源图标共用。

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct AxisLabelPalette {
    pub axis: [u8; 4],
    pub scale_axis: [u8; 4],
    pub disabled_axis: [u8; 4],
    pub scale_link: [u8; 4],
    pub disabled_scale_link: [u8; 4],
}
