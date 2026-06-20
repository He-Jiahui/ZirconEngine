use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchTooltipStyle {
    pub surface: [u8; 4],
    pub border: [u8; 4],
    pub title: [u8; 4],
    pub body: [u8; 4],
    pub arrow: [u8; 4],
    pub icon: [u8; 4],
    pub shadow: [u8; 4],
    pub state: UiPainterResolvedState,
}
