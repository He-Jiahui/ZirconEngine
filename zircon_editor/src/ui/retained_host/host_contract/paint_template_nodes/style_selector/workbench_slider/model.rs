use zircon_runtime_interface::ui::style::UiPainterResolvedState;

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
