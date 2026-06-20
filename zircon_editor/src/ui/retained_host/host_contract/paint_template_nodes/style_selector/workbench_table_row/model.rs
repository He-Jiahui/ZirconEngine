use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchTableRowStyle
{
    pub background: [u8; 4],
    pub border: Option<[u8; 4]>,
    pub border_width: f32,
    pub separator: [u8; 4],
    pub action: [u8; 4],
    pub state: UiPainterResolvedState,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) text: [u8; 4],
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) muted_text: [u8; 4],
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) tail_value_text: [u8; 4],
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) header: bool,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) tail: bool,
}
