use zircon_runtime_interface::ui::layout::UiFrame;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WelcomeRecentPointerLayout {
    pub viewport: UiFrame,
    pub recent_project_paths: Vec<String>,
}

impl Default for WelcomeRecentPointerLayout {
    fn default() -> Self {
        Self {
            viewport: UiFrame::default(),
            recent_project_paths: Vec::new(),
        }
    }
}
