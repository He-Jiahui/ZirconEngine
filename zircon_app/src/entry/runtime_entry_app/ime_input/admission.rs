#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::entry::runtime_entry_app) struct RuntimeImeInputAdmission {
    window_focused: bool,
    ime_enabled: bool,
}

impl RuntimeImeInputAdmission {
    pub(in crate::entry::runtime_entry_app) const fn new(window_focused: bool) -> Self {
        Self {
            window_focused,
            ime_enabled: false,
        }
    }

    /// Returns whether the active composition must be cancelled before focus loss is forwarded.
    pub(in crate::entry::runtime_entry_app) fn set_window_focused(
        &mut self,
        focused: bool,
    ) -> bool {
        self.window_focused = focused;
        if focused {
            return false;
        }
        let cancel_active_composition = self.ime_enabled;
        self.ime_enabled = false;
        cancel_active_composition
    }

    pub(in crate::entry::runtime_entry_app) fn enable(&mut self) -> bool {
        if !self.window_focused {
            self.ime_enabled = false;
            return false;
        }
        self.ime_enabled = true;
        true
    }

    pub(in crate::entry::runtime_entry_app) fn disable(&mut self) -> bool {
        let was_enabled = self.ime_enabled;
        self.ime_enabled = false;
        was_enabled
    }

    pub(in crate::entry::runtime_entry_app) const fn admits_composition(self) -> bool {
        self.window_focused && self.ime_enabled
    }

    pub(in crate::entry::runtime_entry_app) const fn window_focused(self) -> bool {
        self.window_focused
    }
}
