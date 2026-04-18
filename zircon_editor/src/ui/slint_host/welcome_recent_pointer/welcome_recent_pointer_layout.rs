use zircon_ui::UiSize;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WelcomeRecentPointerLayout {
    pub pane_size: UiSize,
    pub recent_project_paths: Vec<String>,
}

impl Default for WelcomeRecentPointerLayout {
    fn default() -> Self {
        Self {
            pane_size: UiSize::new(0.0, 0.0),
            recent_project_paths: Vec::new(),
        }
    }
}
