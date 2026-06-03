#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorWorkbenchReferencePalette {
    pub app_background: &'static str,
    pub panel_background: &'static str,
    pub panel_raised: &'static str,
    pub panel_hover: &'static str,
    pub viewport_background: &'static str,
    pub control_background: &'static str,
    pub control_background_active: &'static str,
    pub control_border: &'static str,
    pub divider: &'static str,
    pub accent: &'static str,
    pub accent_soft: &'static str,
    pub text_primary: &'static str,
    pub text_secondary: &'static str,
    pub text_muted: &'static str,
    pub warning: &'static str,
    pub success: &'static str,
}

impl Default for EditorWorkbenchReferencePalette {
    fn default() -> Self {
        Self {
            app_background: "#0d1115",
            panel_background: "#11161a",
            panel_raised: "#171d22",
            panel_hover: "#122a30",
            viewport_background: "#182026",
            control_background: "#1b2228",
            control_background_active: "#21b7c9",
            control_border: "#2a333a",
            divider: "#263039",
            accent: "#20c7d8",
            accent_soft: "#0f3d46",
            text_primary: "#e7edf2",
            text_secondary: "#aeb8c0",
            text_muted: "#78838b",
            warning: "#f1b84d",
            success: "#5bc878",
        }
    }
}
