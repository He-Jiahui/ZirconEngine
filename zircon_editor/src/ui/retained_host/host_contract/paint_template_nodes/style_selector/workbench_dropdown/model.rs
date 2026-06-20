use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchDropdownStyle
{
    pub surface: [u8; 4],
    pub border: [u8; 4],
    pub text: [u8; 4],
    pub chevron: [u8; 4],
    pub state: UiPainterResolvedState,
}
