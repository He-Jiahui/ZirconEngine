use super::{
    AuthCompletion, AuthFlow, AuthFlowEffect, AuthFlowError, CharacterRosterEntry,
    CharacterRosterScreen, CharacterSortMode, OfflinePlayerClass, OnlineCharacterEffect,
    OnlineCharacterFlow, OnlineCharacterFlowError, RealmDefinition, RealmDirectoryEffect,
    RealmDirectoryError, RealmDirectoryModel,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OnlineEntryState {
    AuthenticationRequired,
    Authenticated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OnlineShellScreen {
    ModeSelection,
    Authentication,
    RealmDirectory,
    CharacterSelection,
    CharacterCreation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OnlineShellEffect {
    Authentication(AuthFlowEffect),
    LoadRealmDirectory,
    SelectRealmAndLoadCharacters {
        realm_name: String,
        base_url: String,
    },
    Character(OnlineCharacterEffect),
    NavigateToModeSelection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OnlineShellError {
    InvalidScreen {
        action: &'static str,
        expected: OnlineShellScreen,
        actual: OnlineShellScreen,
    },
    Auth(AuthFlowError),
    Realm(RealmDirectoryError),
    Characters(OnlineCharacterFlowError),
}

impl From<AuthFlowError> for OnlineShellError {
    fn from(error: AuthFlowError) -> Self {
        Self::Auth(error)
    }
}

impl From<RealmDirectoryError> for OnlineShellError {
    fn from(error: RealmDirectoryError) -> Self {
        Self::Realm(error)
    }
}

impl From<OnlineCharacterFlowError> for OnlineShellError {
    fn from(error: OnlineCharacterFlowError) -> Self {
        Self::Characters(error)
    }
}

pub struct OnlineShellController {
    screen: OnlineShellScreen,
    auth: AuthFlow,
    realms: RealmDirectoryModel,
    characters: OnlineCharacterFlow,
}

impl OnlineShellController {
    pub fn new(sort_mode: CharacterSortMode) -> Self {
        Self {
            screen: OnlineShellScreen::ModeSelection,
            auth: AuthFlow::new(),
            realms: RealmDirectoryModel::default(),
            characters: OnlineCharacterFlow::new(sort_mode),
        }
    }

    pub const fn screen(&self) -> OnlineShellScreen {
        self.screen
    }

    pub fn auth(&self) -> &AuthFlow {
        &self.auth
    }

    pub fn auth_mut(&mut self) -> &mut AuthFlow {
        &mut self.auth
    }

    pub fn realms(&self) -> &RealmDirectoryModel {
        &self.realms
    }

    pub fn characters(&self) -> &OnlineCharacterFlow {
        &self.characters
    }

    /// The mode selector's Online choice is resolved by a host-provided session state.
    pub fn enter_online(
        &mut self,
        entry_state: OnlineEntryState,
    ) -> Result<Option<OnlineShellEffect>, OnlineShellError> {
        match entry_state {
            OnlineEntryState::AuthenticationRequired => {
                self.auth.set_auth_mode(super::AuthMode::Login);
                self.screen = OnlineShellScreen::Authentication;
                Ok(None)
            }
            OnlineEntryState::Authenticated => {
                self.screen = OnlineShellScreen::RealmDirectory;
                Ok(Some(OnlineShellEffect::LoadRealmDirectory))
            }
        }
    }

    pub fn submit_auth(&self) -> Result<OnlineShellEffect, OnlineShellError> {
        self.require_screen("submit_auth", OnlineShellScreen::Authentication)?;
        Ok(OnlineShellEffect::Authentication(self.auth.submit_auth()?))
    }

    pub fn complete_auth(
        &mut self,
        completion: AuthCompletion,
    ) -> Result<Option<OnlineShellEffect>, OnlineShellError> {
        self.require_screen("complete_auth", OnlineShellScreen::Authentication)?;
        match self.auth.complete_auth(completion) {
            Some(AuthFlowEffect::NavigateToRealmDirectory) => {
                self.screen = OnlineShellScreen::RealmDirectory;
                Ok(Some(OnlineShellEffect::LoadRealmDirectory))
            }
            None => Ok(None),
            Some(effect) => Ok(Some(OnlineShellEffect::Authentication(effect))),
        }
    }

    pub fn replace_realm_directory(
        &mut self,
        definitions: Vec<RealmDefinition>,
        remembered_realm: Option<&str>,
    ) -> Result<Option<OnlineShellEffect>, OnlineShellError> {
        self.require_screen("replace_realm_directory", OnlineShellScreen::RealmDirectory)?;
        let effect = self
            .realms
            .replace_directory(definitions, remembered_realm)?;
        self.apply_realm_effect(effect)
    }

    pub fn select_realm(
        &mut self,
        realm_name: &str,
    ) -> Result<OnlineShellEffect, OnlineShellError> {
        self.require_screen("select_realm", OnlineShellScreen::RealmDirectory)?;
        let effect = self.realms.select(realm_name)?;
        match self.apply_realm_effect(effect)? {
            Some(effect) => Ok(effect),
            None => unreachable!("realm selection always produces a host refresh"),
        }
    }

    pub fn back_from_realms(&mut self) -> Result<Option<OnlineShellEffect>, OnlineShellError> {
        self.require_screen("back_from_realms", OnlineShellScreen::RealmDirectory)?;
        match self.realms.back() {
            RealmDirectoryEffect::NavigateToModeSelection => {
                self.screen = OnlineShellScreen::ModeSelection;
                Ok(Some(OnlineShellEffect::NavigateToModeSelection))
            }
            RealmDirectoryEffect::ShowList | RealmDirectoryEffect::SelectRealm { .. } => {
                unreachable!("realm Back has a fixed mode-selection route")
            }
        }
    }

    pub fn replace_characters(
        &mut self,
        entries: Vec<CharacterRosterEntry>,
    ) -> Result<(), OnlineShellError> {
        self.require_screen("replace_characters", OnlineShellScreen::CharacterSelection)?;
        self.characters.replace_roster(entries)?;
        self.screen = screen_for_character_roster(self.characters.screen());
        Ok(())
    }

    pub fn open_character_create(&mut self) -> Result<(), OnlineShellError> {
        self.require_screen(
            "open_character_create",
            OnlineShellScreen::CharacterSelection,
        )?;
        self.characters.open_create()?;
        self.screen = OnlineShellScreen::CharacterCreation;
        Ok(())
    }

    pub fn change_realm(&mut self) -> Result<(), OnlineShellError> {
        self.require_screen("change_realm", OnlineShellScreen::CharacterSelection)?;
        self.screen = OnlineShellScreen::RealmDirectory;
        Ok(())
    }

    pub fn set_character_sort_mode(
        &mut self,
        mode: CharacterSortMode,
    ) -> Result<OnlineShellEffect, OnlineShellError> {
        self.require_screen(
            "set_character_sort_mode",
            OnlineShellScreen::CharacterSelection,
        )?;
        Ok(OnlineShellEffect::Character(
            self.characters.set_sort_mode(mode)?,
        ))
    }

    pub fn back_from_characters(&mut self) -> Result<Option<OnlineShellEffect>, OnlineShellError> {
        match self.screen {
            OnlineShellScreen::CharacterSelection | OnlineShellScreen::CharacterCreation => {}
            screen => {
                return Err(OnlineShellError::InvalidScreen {
                    action: "back_from_characters",
                    expected: OnlineShellScreen::CharacterSelection,
                    actual: screen,
                });
            }
        }
        match self.characters.back()? {
            Some(OnlineCharacterEffect::NavigateToLogin) => {
                self.screen = OnlineShellScreen::Authentication;
                Ok(None)
            }
            None => {
                self.screen = screen_for_character_roster(self.characters.screen());
                Ok(None)
            }
            Some(effect) => Ok(Some(OnlineShellEffect::Character(effect))),
        }
    }

    pub fn character_primary_action(&mut self) -> Result<OnlineShellEffect, OnlineShellError> {
        self.require_screen(
            "character_primary_action",
            OnlineShellScreen::CharacterSelection,
        )?;
        Ok(OnlineShellEffect::Character(
            self.characters.primary_action()?,
        ))
    }

    pub fn cancel_character_takeover(&mut self) -> Result<(), OnlineShellError> {
        self.require_screen(
            "cancel_character_takeover",
            OnlineShellScreen::CharacterSelection,
        )?;
        self.characters.cancel_takeover();
        Ok(())
    }

    pub fn confirm_character_takeover(&mut self) -> Result<OnlineShellEffect, OnlineShellError> {
        self.require_screen(
            "confirm_character_takeover",
            OnlineShellScreen::CharacterSelection,
        )?;
        Ok(OnlineShellEffect::Character(
            self.characters.confirm_takeover()?,
        ))
    }

    pub fn set_character_delete_confirmation(
        &mut self,
        confirmation: impl Into<String>,
    ) -> Result<(), OnlineShellError> {
        self.require_screen(
            "set_character_delete_confirmation",
            OnlineShellScreen::CharacterSelection,
        )?;
        self.characters.set_delete_confirmation(confirmation)?;
        Ok(())
    }

    pub fn cancel_character_delete(&mut self) -> Result<(), OnlineShellError> {
        self.require_screen(
            "cancel_character_delete",
            OnlineShellScreen::CharacterSelection,
        )?;
        self.characters.cancel_delete();
        Ok(())
    }

    pub fn submit_character_delete(&mut self) -> Result<OnlineShellEffect, OnlineShellError> {
        self.require_screen(
            "submit_character_delete",
            OnlineShellScreen::CharacterSelection,
        )?;
        Ok(OnlineShellEffect::Character(
            self.characters.submit_delete()?,
        ))
    }

    pub fn set_character_create_name(
        &mut self,
        name: impl Into<String>,
    ) -> Result<(), OnlineShellError> {
        self.require_screen(
            "set_character_create_name",
            OnlineShellScreen::CharacterCreation,
        )?;
        self.characters.set_create_name(name)?;
        Ok(())
    }

    pub fn set_character_create_class(
        &mut self,
        player_class: OfflinePlayerClass,
    ) -> Result<(), OnlineShellError> {
        self.require_screen(
            "set_character_create_class",
            OnlineShellScreen::CharacterCreation,
        )?;
        self.characters.set_create_class(player_class)?;
        Ok(())
    }

    pub fn set_character_create_skin(&mut self, skin_variant: u16) -> Result<(), OnlineShellError> {
        self.require_screen(
            "set_character_create_skin",
            OnlineShellScreen::CharacterCreation,
        )?;
        self.characters.set_create_skin(skin_variant)?;
        Ok(())
    }

    pub fn submit_character_create(&self) -> Result<OnlineShellEffect, OnlineShellError> {
        self.require_screen(
            "submit_character_create",
            OnlineShellScreen::CharacterCreation,
        )?;
        Ok(OnlineShellEffect::Character(
            self.characters.submit_create()?,
        ))
    }

    fn apply_realm_effect(
        &mut self,
        effect: RealmDirectoryEffect,
    ) -> Result<Option<OnlineShellEffect>, OnlineShellError> {
        match effect {
            RealmDirectoryEffect::ShowList => Ok(None),
            RealmDirectoryEffect::SelectRealm {
                realm_name,
                base_url,
            } => {
                self.screen = OnlineShellScreen::CharacterSelection;
                Ok(Some(OnlineShellEffect::SelectRealmAndLoadCharacters {
                    realm_name,
                    base_url,
                }))
            }
            RealmDirectoryEffect::NavigateToModeSelection => {
                self.screen = OnlineShellScreen::ModeSelection;
                Ok(Some(OnlineShellEffect::NavigateToModeSelection))
            }
        }
    }

    fn require_screen(
        &self,
        action: &'static str,
        expected: OnlineShellScreen,
    ) -> Result<(), OnlineShellError> {
        if self.screen == expected {
            Ok(())
        } else {
            Err(OnlineShellError::InvalidScreen {
                action,
                expected,
                actual: self.screen,
            })
        }
    }
}

fn screen_for_character_roster(screen: CharacterRosterScreen) -> OnlineShellScreen {
    match screen {
        CharacterRosterScreen::SelectCharacter => OnlineShellScreen::CharacterSelection,
        CharacterRosterScreen::CreateCharacter => OnlineShellScreen::CharacterCreation,
    }
}
