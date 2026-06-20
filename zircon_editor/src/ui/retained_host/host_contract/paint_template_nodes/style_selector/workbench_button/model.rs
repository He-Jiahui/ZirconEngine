use zircon_runtime_interface::ui::style::ButtonInteractionState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum WorkbenchButtonKind {
    Primary,
    Secondary,
    Tertiary,
    Danger,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchButtonStyle {
    pub surface: [u8; 4],
    pub border: [u8; 4],
    pub border_width: f32,
    pub text: [u8; 4],
    pub glyph: [u8; 4],
    pub interaction: ButtonInteractionState,
}
