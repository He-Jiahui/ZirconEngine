use super::{
    offline_class_presentation, OfflinePlayerClass, OfflineSessionDraft, OfflineSessionError,
    OfflineSessionLaunch,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OfflineShellState {
    #[default]
    ModeSelection,
    OfflinePicker,
    Welcome,
    Loading,
    InWorld,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OfflineShellAction {
    OpenOfflinePicker,
    BackToModeSelection,
    SetClass,
    SetName,
    SetSkin,
    SubmitOfflinePicker,
    ContinueWelcome,
    FinishLoading,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OfflineShellError {
    InvalidTransition {
        action: OfflineShellAction,
        state: OfflineShellState,
    },
    Session(OfflineSessionError),
}

impl From<OfflineSessionError> for OfflineShellError {
    fn from(error: OfflineSessionError) -> Self {
        Self::Session(error)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OfflineShellController {
    state: OfflineShellState,
    draft: OfflineSessionDraft,
    prepared_launch: Option<OfflineSessionLaunch>,
}

impl OfflineShellController {
    pub fn state(&self) -> OfflineShellState {
        self.state
    }

    pub fn draft(&self) -> &OfflineSessionDraft {
        &self.draft
    }

    pub fn prepared_launch(&self) -> Option<&OfflineSessionLaunch> {
        self.prepared_launch.as_ref()
    }

    pub fn open_offline_picker(&mut self) -> Result<(), OfflineShellError> {
        self.require_state(
            OfflineShellAction::OpenOfflinePicker,
            OfflineShellState::ModeSelection,
        )?;
        self.draft = OfflineSessionDraft::default();
        self.prepared_launch = None;
        self.state = OfflineShellState::OfflinePicker;
        Ok(())
    }

    pub fn back_to_mode_selection(&mut self) -> Result<(), OfflineShellError> {
        self.require_state(
            OfflineShellAction::BackToModeSelection,
            OfflineShellState::OfflinePicker,
        )?;
        self.draft.set_raw_name("");
        self.state = OfflineShellState::ModeSelection;
        Ok(())
    }

    pub fn set_class(&mut self, player_class: OfflinePlayerClass) -> Result<(), OfflineShellError> {
        self.require_picker(OfflineShellAction::SetClass)?;
        self.draft.set_player_class(player_class);
        Ok(())
    }

    pub fn set_name(&mut self, raw_name: impl Into<String>) -> Result<(), OfflineShellError> {
        self.require_picker(OfflineShellAction::SetName)?;
        self.draft.set_raw_name(raw_name);
        Ok(())
    }

    pub fn set_skin(&mut self, skin_variant: u16) -> Result<(), OfflineShellError> {
        self.require_picker(OfflineShellAction::SetSkin)?;
        let player_class = self
            .draft
            .player_class()
            .ok_or(OfflineSessionError::MissingPlayerClass)?;
        let skin_count = offline_class_presentation(player_class).skin_count;
        if skin_variant >= skin_count {
            return Err(OfflineSessionError::InvalidSkinVariant {
                player_class,
                skin_variant,
                skin_count,
            }
            .into());
        }
        self.draft.set_skin_variant(skin_variant);
        Ok(())
    }

    pub fn submit_offline_picker(&mut self) -> Result<OfflineSessionLaunch, OfflineShellError> {
        self.require_picker(OfflineShellAction::SubmitOfflinePicker)?;
        let launch = self.draft.launch()?;
        self.prepared_launch = Some(launch.clone());
        self.state = OfflineShellState::Welcome;
        Ok(launch)
    }

    pub fn continue_from_welcome(&mut self) -> Result<OfflineSessionLaunch, OfflineShellError> {
        self.require_state(
            OfflineShellAction::ContinueWelcome,
            OfflineShellState::Welcome,
        )?;
        let launch = self
            .prepared_launch
            .as_ref()
            .expect("Welcome state must own a prepared offline launch")
            .clone();
        self.state = OfflineShellState::Loading;
        Ok(launch)
    }

    pub fn finish_loading(&mut self) -> Result<OfflineSessionLaunch, OfflineShellError> {
        self.require_state(
            OfflineShellAction::FinishLoading,
            OfflineShellState::Loading,
        )?;
        let launch = self
            .prepared_launch
            .as_ref()
            .expect("Loading state must own a prepared offline launch")
            .clone();
        self.state = OfflineShellState::InWorld;
        Ok(launch)
    }

    fn require_picker(&self, action: OfflineShellAction) -> Result<(), OfflineShellError> {
        self.require_state(action, OfflineShellState::OfflinePicker)
    }

    fn require_state(
        &self,
        action: OfflineShellAction,
        expected: OfflineShellState,
    ) -> Result<(), OfflineShellError> {
        if self.state == expected {
            Ok(())
        } else {
            Err(OfflineShellError::InvalidTransition {
                action,
                state: self.state,
            })
        }
    }
}
