//! 滑块绘制配方分开提供轨道、拇指、数值框和文字；绘制端负责数值到几何位置的换算。

use zircon_runtime_interface::ui::style::UiPainterResolvedState;

/// 供轨道、拇指、数值框与文字绘制端共享的配方；不包含数值或几何求解。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchSliderStyle {
    pub track: [u8; 4],
    pub fill: [u8; 4],
    pub thumb: [u8; 4],
    pub thumb_outline: [u8; 4],
    pub thumb_halo: Option<[u8; 4]>,
    pub value_surface: [u8; 4],
    pub value_border: [u8; 4],
    pub range_value_border: [u8; 4],
    pub label_text: [u8; 4],
    pub value_text: [u8; 4],
    pub tick: [u8; 4],
    pub state: UiPainterResolvedState,
}
