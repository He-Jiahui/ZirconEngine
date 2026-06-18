#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ViewportCameraStackOutputPolicy {
    stack_terminal: bool,
    viewport_terminal: bool,
}

impl ViewportCameraStackOutputPolicy {
    pub(crate) const fn new(stack_terminal: bool, viewport_terminal: bool) -> Self {
        Self {
            stack_terminal,
            viewport_terminal,
        }
    }

    pub(crate) const fn stack_terminal() -> Self {
        Self::new(true, true)
    }

    #[cfg(test)]
    pub(crate) const fn is_stack_terminal(self) -> bool {
        self.stack_terminal
    }

    #[cfg(test)]
    pub(crate) const fn is_viewport_terminal(self) -> bool {
        self.viewport_terminal
    }

    pub(crate) fn owns_viewport_submission(self) -> bool {
        debug_assert!(
            !self.viewport_terminal || self.stack_terminal,
            "viewport terminal camera must also be stack terminal"
        );
        self.viewport_terminal
    }

    pub(crate) fn writes_output_target(self) -> bool {
        debug_assert!(
            !self.viewport_terminal || self.stack_terminal,
            "viewport terminal camera must also be stack terminal"
        );
        self.stack_terminal
    }
}

impl Default for ViewportCameraStackOutputPolicy {
    fn default() -> Self {
        Self::stack_terminal()
    }
}
