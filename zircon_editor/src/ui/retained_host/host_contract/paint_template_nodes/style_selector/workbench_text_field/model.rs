use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchTextFieldStyle
{
    pub surface: [u8; 4],
    pub border: [u8; 4],
    pub text: [u8; 4],
    pub stepper: [u8; 4],
    pub stepper_divider: [u8; 4],
    pub state: UiPainterResolvedState,
}
