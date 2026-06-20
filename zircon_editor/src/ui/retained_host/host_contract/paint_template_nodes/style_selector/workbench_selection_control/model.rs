use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum WorkbenchSelectionControlKind
{
    Checkbox,
    Radio,
    Toggle,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchSelectionControlStyle
{
    pub surface: [u8; 4],
    pub border: [u8; 4],
    pub thumb: [u8; 4],
    pub accent: [u8; 4],
    pub text: [u8; 4],
    pub label: [u8; 4],
    pub state: UiPainterResolvedState,
}
