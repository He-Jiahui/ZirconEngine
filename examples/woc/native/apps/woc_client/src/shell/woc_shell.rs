use super::{
    AuthCompletion, AuthFlowError, CharacterRosterEntry, CharacterSortMode, ModeSelectionEffect,
    ModeSelectionError, ModeSelectionModel, OfflinePlayerClass, OfflineSessionLaunch,
    OfflineShellController, OfflineShellError, OfflineShellState, OnlineEntryState,
    OnlineShellController, OnlineShellEffect, OnlineShellError, OnlineShellScreen, RealmDefinition,
    ServerMode,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WocShellScreen {
    ModeSelection,
    Authentication,
    RealmDirectory,
    CharacterSelection,
    CharacterCreation,
    OfflinePicker,
    Welcome,
    Loading,
    InWorld,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WocShellEffect {
    ProbeOnlineSession,
    Online(OnlineShellEffect),
    PrepareOfflineSession { launch: OfflineSessionLaunch },
    StartOfflineWorld { launch: OfflineSessionLaunch },
    EnterOfflineWorld { launch: OfflineSessionLaunch },
    NavigateToModeSelection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WocShellError {
    InvalidActiveFlow {
        action: &'static str,
        screen: WocShellScreen,
    },
    Mode(ModeSelectionError),
    Online(OnlineShellError),
    Offline(OfflineShellError),
    Auth(AuthFlowError),
}

impl From<ModeSelectionError> for WocShellError {
    fn from(error: ModeSelectionError) -> Self {
        Self::Mode(error)
    }
}

impl From<OnlineShellError> for WocShellError {
    fn from(error: OnlineShellError) -> Self {
        Self::Online(error)
    }
}

impl From<OfflineShellError> for WocShellError {
    fn from(error: OfflineShellError) -> Self {
        Self::Offline(error)
    }
}

impl From<AuthFlowError> for WocShellError {
    fn from(error: AuthFlowError) -> Self {
        Self::Auth(error)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActiveFlow {
    ModeSelection,
    Online,
    Offline,
}

pub struct WocShellController {
    active_flow: ActiveFlow,
    mode: ModeSelectionModel,
    online: OnlineShellController,
    offline: OfflineShellController,
}

impl WocShellController {
    pub fn new(offline_available: bool, sort_mode: CharacterSortMode) -> Self {
        Self {
            active_flow: ActiveFlow::ModeSelection,
            mode: ModeSelectionModel::new(offline_available),
            online: OnlineShellController::new(sort_mode),
            offline: OfflineShellController::default(),
        }
    }

    pub fn screen(&self) -> WocShellScreen {
        match self.active_flow {
            ActiveFlow::ModeSelection => WocShellScreen::ModeSelection,
            ActiveFlow::Online => online_screen(self.online.screen()),
            ActiveFlow::Offline => offline_screen(self.offline.state()),
        }
    }

    pub fn mode(&self) -> &ModeSelectionModel {
        &self.mode
    }

    pub fn online(&self) -> &OnlineShellController {
        &self.online
    }

    pub(super) fn online_mut(&mut self) -> &mut OnlineShellController {
        &mut self.online
    }

    pub fn offline(&self) -> &OfflineShellController {
        &self.offline
    }

    pub fn select_mode(&mut self, mode: ServerMode) -> Result<(), WocShellError> {
        self.require_active("select_mode", ActiveFlow::ModeSelection)?;
        self.mode.select_mode(mode)?;
        Ok(())
    }

    pub(super) fn toggle_mode_menu(&mut self) -> Result<(), WocShellError> {
        self.require_active("toggle_mode_menu", ActiveFlow::ModeSelection)?;
        self.mode.toggle_menu();
        Ok(())
    }

    pub fn play(&mut self) -> Result<Option<WocShellEffect>, WocShellError> {
        self.require_active("play", ActiveFlow::ModeSelection)?;
        match self.mode.play() {
            ModeSelectionEffect::OpenOnlineFlow => {
                self.active_flow = ActiveFlow::Online;
                Ok(Some(WocShellEffect::ProbeOnlineSession))
            }
            ModeSelectionEffect::OpenOfflinePicker => {
                self.offline.open_offline_picker()?;
                self.active_flow = ActiveFlow::Offline;
                Ok(None)
            }
        }
    }

    pub fn resolve_online_entry(
        &mut self,
        entry_state: OnlineEntryState,
    ) -> Result<Option<WocShellEffect>, WocShellError> {
        self.require_active("resolve_online_entry", ActiveFlow::Online)?;
        Ok(self
            .online
            .enter_online(entry_state)?
            .map(WocShellEffect::Online))
    }

    pub fn complete_auth(
        &mut self,
        completion: AuthCompletion,
    ) -> Result<Option<WocShellEffect>, WocShellError> {
        self.require_screen("complete_auth", WocShellScreen::Authentication)?;
        Ok(self
            .online
            .complete_auth(completion)?
            .map(WocShellEffect::Online))
    }

    pub fn replace_realm_directory(
        &mut self,
        definitions: Vec<RealmDefinition>,
        remembered_realm: Option<&str>,
    ) -> Result<Option<WocShellEffect>, WocShellError> {
        self.require_screen("replace_realm_directory", WocShellScreen::RealmDirectory)?;
        Ok(self
            .online
            .replace_realm_directory(definitions, remembered_realm)?
            .map(WocShellEffect::Online))
    }

    pub fn select_realm(&mut self, realm_name: &str) -> Result<WocShellEffect, WocShellError> {
        self.require_screen("select_realm", WocShellScreen::RealmDirectory)?;
        Ok(WocShellEffect::Online(
            self.online.select_realm(realm_name)?,
        ))
    }

    pub fn replace_characters(
        &mut self,
        entries: Vec<CharacterRosterEntry>,
    ) -> Result<(), WocShellError> {
        self.require_screen("replace_characters", WocShellScreen::CharacterSelection)?;
        self.online.replace_characters(entries)?;
        Ok(())
    }

    pub fn back_from_authentication(&mut self) -> Result<(), WocShellError> {
        self.require_screen("back_from_authentication", WocShellScreen::Authentication)?;
        let effect = self.online.auth_mut().back()?;
        if !matches!(effect, Some(super::AuthFlowEffect::NavigateToModeSelection)) {
            unreachable!("authentication Back has a fixed mode-selection route");
        }
        self.active_flow = ActiveFlow::ModeSelection;
        Ok(())
    }

    pub fn back_from_realms(&mut self) -> Result<Option<WocShellEffect>, WocShellError> {
        self.require_screen("back_from_realms", WocShellScreen::RealmDirectory)?;
        match self.online.back_from_realms()? {
            Some(OnlineShellEffect::NavigateToModeSelection) => {
                self.active_flow = ActiveFlow::ModeSelection;
                Ok(Some(WocShellEffect::NavigateToModeSelection))
            }
            None => Ok(None),
            Some(effect) => Ok(Some(WocShellEffect::Online(effect))),
        }
    }

    pub fn back_from_characters(&mut self) -> Result<Option<WocShellEffect>, WocShellError> {
        self.require_active("back_from_characters", ActiveFlow::Online)?;
        Ok(self
            .online
            .back_from_characters()?
            .map(WocShellEffect::Online))
    }

    pub fn set_offline_class(
        &mut self,
        player_class: OfflinePlayerClass,
    ) -> Result<(), WocShellError> {
        self.require_screen("set_offline_class", WocShellScreen::OfflinePicker)?;
        self.offline.set_class(player_class)?;
        Ok(())
    }

    pub fn set_offline_name(&mut self, name: impl Into<String>) -> Result<(), WocShellError> {
        self.require_screen("set_offline_name", WocShellScreen::OfflinePicker)?;
        self.offline.set_name(name)?;
        Ok(())
    }

    pub fn set_offline_skin(&mut self, skin_variant: u16) -> Result<(), WocShellError> {
        self.require_screen("set_offline_skin", WocShellScreen::OfflinePicker)?;
        self.offline.set_skin(skin_variant)?;
        Ok(())
    }

    pub fn submit_offline_picker(&mut self) -> Result<WocShellEffect, WocShellError> {
        self.require_screen("submit_offline_picker", WocShellScreen::OfflinePicker)?;
        let launch = self.offline.submit_offline_picker()?;
        Ok(WocShellEffect::PrepareOfflineSession { launch })
    }

    pub fn back_from_offline_picker(&mut self) -> Result<(), WocShellError> {
        self.require_screen("back_from_offline_picker", WocShellScreen::OfflinePicker)?;
        self.offline.back_to_mode_selection()?;
        self.active_flow = ActiveFlow::ModeSelection;
        Ok(())
    }

    pub fn continue_offline_welcome(&mut self) -> Result<WocShellEffect, WocShellError> {
        self.require_screen("continue_offline_welcome", WocShellScreen::Welcome)?;
        let launch = self.offline.continue_from_welcome()?;
        Ok(WocShellEffect::StartOfflineWorld { launch })
    }

    pub fn finish_offline_loading(&mut self) -> Result<WocShellEffect, WocShellError> {
        self.require_screen("finish_offline_loading", WocShellScreen::Loading)?;
        let launch = self.offline.finish_loading()?;
        Ok(WocShellEffect::EnterOfflineWorld { launch })
    }

    fn require_active(
        &self,
        action: &'static str,
        expected: ActiveFlow,
    ) -> Result<(), WocShellError> {
        if self.active_flow == expected {
            Ok(())
        } else {
            Err(WocShellError::InvalidActiveFlow {
                action,
                screen: self.screen(),
            })
        }
    }

    fn require_screen(
        &self,
        action: &'static str,
        expected: WocShellScreen,
    ) -> Result<(), WocShellError> {
        let actual = self.screen();
        if actual == expected {
            Ok(())
        } else {
            Err(WocShellError::InvalidActiveFlow {
                action,
                screen: actual,
            })
        }
    }
}

fn online_screen(screen: OnlineShellScreen) -> WocShellScreen {
    match screen {
        OnlineShellScreen::ModeSelection => WocShellScreen::ModeSelection,
        OnlineShellScreen::Authentication => WocShellScreen::Authentication,
        OnlineShellScreen::RealmDirectory => WocShellScreen::RealmDirectory,
        OnlineShellScreen::CharacterSelection => WocShellScreen::CharacterSelection,
        OnlineShellScreen::CharacterCreation => WocShellScreen::CharacterCreation,
    }
}

fn offline_screen(screen: OfflineShellState) -> WocShellScreen {
    match screen {
        OfflineShellState::ModeSelection => WocShellScreen::ModeSelection,
        OfflineShellState::OfflinePicker => WocShellScreen::OfflinePicker,
        OfflineShellState::Welcome => WocShellScreen::Welcome,
        OfflineShellState::Loading => WocShellScreen::Loading,
        OfflineShellState::InWorld => WocShellScreen::InWorld,
    }
}
