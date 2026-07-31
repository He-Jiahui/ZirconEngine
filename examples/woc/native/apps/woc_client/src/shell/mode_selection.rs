#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServerMode {
    Online,
    Offline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModeMenuNavigation {
    Previous,
    Next,
    First,
    Last,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModeSelectionEffect {
    OpenOnlineFlow,
    OpenOfflinePicker,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModeSelectionError {
    OfflineUnavailable,
    MenuClosed,
}

pub struct ModeSelectionModel {
    selected_mode: ServerMode,
    offline_available: bool,
    menu_open: bool,
    active_mode: Option<ServerMode>,
}

impl ModeSelectionModel {
    pub const fn new(offline_available: bool) -> Self {
        Self {
            selected_mode: ServerMode::Online,
            offline_available,
            menu_open: false,
            active_mode: None,
        }
    }

    pub const fn selected_mode(&self) -> ServerMode {
        self.selected_mode
    }

    pub const fn offline_available(&self) -> bool {
        self.offline_available
    }

    pub const fn menu_open(&self) -> bool {
        self.menu_open
    }

    pub const fn active_mode(&self) -> Option<ServerMode> {
        self.active_mode
    }

    pub fn open_menu(&mut self) {
        self.menu_open = true;
        self.active_mode = Some(self.selected_mode);
    }

    pub fn close_menu(&mut self) {
        self.menu_open = false;
        self.active_mode = None;
    }

    pub fn toggle_menu(&mut self) {
        if self.menu_open {
            self.close_menu();
        } else {
            self.open_menu();
        }
    }

    pub fn move_active(
        &mut self,
        navigation: ModeMenuNavigation,
    ) -> Result<(), ModeSelectionError> {
        if !self.menu_open {
            return Err(ModeSelectionError::MenuClosed);
        }
        self.active_mode = Some(match navigation {
            ModeMenuNavigation::Previous | ModeMenuNavigation::First => ServerMode::Online,
            ModeMenuNavigation::Next | ModeMenuNavigation::Last if self.offline_available => {
                ServerMode::Offline
            }
            ModeMenuNavigation::Next | ModeMenuNavigation::Last => ServerMode::Online,
        });
        Ok(())
    }

    pub fn select_mode(&mut self, mode: ServerMode) -> Result<(), ModeSelectionError> {
        if mode == ServerMode::Offline && !self.offline_available {
            return Err(ModeSelectionError::OfflineUnavailable);
        }
        self.selected_mode = mode;
        self.close_menu();
        Ok(())
    }

    pub fn commit_active(&mut self) -> Result<(), ModeSelectionError> {
        if !self.menu_open {
            return Err(ModeSelectionError::MenuClosed);
        }
        let mode = self.active_mode.unwrap_or(self.selected_mode);
        self.select_mode(mode)
    }

    pub const fn play(&self) -> ModeSelectionEffect {
        match self.selected_mode {
            ServerMode::Online => ModeSelectionEffect::OpenOnlineFlow,
            ServerMode::Offline => ModeSelectionEffect::OpenOfflinePicker,
        }
    }
}
