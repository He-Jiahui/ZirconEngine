use crate::core::gui_startup_request::EditorGuiStartupRequest;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EditorHostRunConfig {
    startup_request: Option<EditorGuiStartupRequest>,
    exit_after_first_presented_frame: bool,
}

impl EditorHostRunConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_startup_request(mut self, request: Option<EditorGuiStartupRequest>) -> Self {
        self.startup_request = request;
        self
    }

    pub fn with_exit_after_first_presented_frame(mut self, exit: bool) -> Self {
        self.exit_after_first_presented_frame = exit;
        self
    }

    pub fn startup_request(&self) -> Option<&EditorGuiStartupRequest> {
        self.startup_request.as_ref()
    }

    pub fn exit_after_first_presented_frame(&self) -> bool {
        self.exit_after_first_presented_frame
    }

    pub(crate) fn into_startup_request(self) -> Option<EditorGuiStartupRequest> {
        self.startup_request
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_host_run_config_defaults_to_normal_interactive_startup() {
        let config = EditorHostRunConfig::new();

        assert_eq!(config.startup_request(), None);
        assert!(!config.exit_after_first_presented_frame());
    }

    #[test]
    fn editor_host_run_config_can_request_first_frame_exit() {
        let config = EditorHostRunConfig::new()
            .with_startup_request(Some(EditorGuiStartupRequest::open_builtin_view(
                "editor.material_component_lab",
            )))
            .with_exit_after_first_presented_frame(true);

        assert_eq!(
            config.startup_request(),
            Some(&EditorGuiStartupRequest::open_builtin_view(
                "editor.material_component_lab"
            ))
        );
        assert!(config.exit_after_first_presented_frame());
    }
}
