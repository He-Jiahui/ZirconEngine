use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_DIAGNOSTIC_SIGNAL_VARIANT:
    &str = "diagnostic_signal";
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_SEMANTIC_STATUS_SIGNAL_VARIANT:
    &str = "semantic_status_signal";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum WorkbenchStatusSignalKind
{
    Ready,
    Success,
    Warning,
    Info,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchStatusSignalStyle
{
    pub icon_fill: [u8; 4],
    pub text: [u8; 4],
    pub state: UiPainterResolvedState,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchStatusChipStyle
{
    pub background: [u8; 4],
    pub border: [u8; 4],
    pub label_text: [u8; 4],
    pub value_text: [u8; 4],
    pub state: UiPainterResolvedState,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchStatusIconButtonStyle
{
    pub background: [u8; 4],
    pub border: [u8; 4],
    pub glyph: [u8; 4],
    pub state: UiPainterResolvedState,
}
