//! 输出策略分别标明目标写入与共享视口产品归属；纹理堆栈终结不等于视口最终呈现。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ViewportCameraStackOutputPolicy {
    viewport_submission_start: bool,
    stack_terminal: bool,
    viewport_terminal: bool,
}

impl ViewportCameraStackOutputPolicy {
    pub(crate) const fn new(stack_terminal: bool, viewport_terminal: bool) -> Self {
        Self {
            viewport_submission_start: false,
            stack_terminal,
            viewport_terminal,
        }
    }

    pub(crate) const fn with_viewport_submission_start(mut self, start: bool) -> Self {
        self.viewport_submission_start = start;
        self
    }

    pub(crate) const fn stack_terminal() -> Self {
        Self::new(true, true).with_viewport_submission_start(true)
    }

    pub(crate) const fn starts_viewport_submission(self) -> bool {
        self.viewport_submission_start
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

    pub(crate) fn owns_shared_viewport_products(self) -> bool {
        self.owns_viewport_submission()
    }

    pub(crate) fn owns_final_target_output(self) -> bool {
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

#[cfg(test)]
#[path = "tests/viewport_camera_stack_output_policy.rs"]
mod tests;
