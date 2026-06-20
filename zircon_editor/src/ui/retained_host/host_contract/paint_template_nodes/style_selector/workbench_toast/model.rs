use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchToastStyle {
    pub surface: [u8; 4],
    pub border: [u8; 4],
    pub text: [u8; 4],
    pub mark: [u8; 4],
    pub action: [u8; 4],
    pub close: [u8; 4],
    pub state: UiPainterResolvedState,
}
