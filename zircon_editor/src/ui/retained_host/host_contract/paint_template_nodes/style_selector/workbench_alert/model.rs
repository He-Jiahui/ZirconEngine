use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum WorkbenchAlertTone {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchAlertStyle {
    pub surface: [u8; 4],
    pub border: [u8; 4],
    pub mark: [u8; 4],
    pub text: [u8; 4],
    pub state: UiPainterResolvedState,
}
