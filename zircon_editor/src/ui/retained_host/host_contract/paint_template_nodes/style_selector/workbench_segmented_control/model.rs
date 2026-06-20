use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum WorkbenchSegmentedControlKind
{
    SegmentedControl,
    Tab,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchSegmentedControlStyle
{
    pub background: Option<[u8; 4]>,
    pub border: Option<[u8; 4]>,
    pub border_width: f32,
    pub selected_surface: [u8; 4],
    pub selected_border: [u8; 4],
    pub selected_border_width: f32,
    pub selected_underline: [u8; 4],
    pub selected_underline_height: f32,
    pub selected_text: [u8; 4],
    pub idle_text: [u8; 4],
    pub group_label: [u8; 4],
    pub state: UiPainterResolvedState,
}
