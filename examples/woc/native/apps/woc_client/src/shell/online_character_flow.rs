use super::{
    normalize_character_name, offline_class_presentation, CharacterEntryBlock, CharacterNameError,
    CharacterPrimaryAction, CharacterRosterEntry, CharacterRosterError, CharacterRosterModel,
    CharacterRosterScreen, CharacterSortMode, OfflinePlayerClass,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OnlineCharacterEffect {
    NavigateToLogin,
    PersistSortAndRefresh {
        mode: CharacterSortMode,
    },
    EnterWorld {
        character_id: u64,
    },
    ConfirmTakeOver {
        character_id: u64,
        character_name: String,
    },
    TakeOverAndEnter {
        character_id: u64,
    },
    Create {
        name: String,
        player_class: OfflinePlayerClass,
        skin_variant: u16,
    },
    Rename {
        character_id: u64,
        name: String,
    },
    Delete {
        character_id: u64,
        confirmation: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OnlineCharacterFlowError {
    InvalidScreen {
        action: &'static str,
        expected: CharacterRosterScreen,
        actual: CharacterRosterScreen,
    },
    Roster(CharacterRosterError),
    CharacterName(CharacterNameError),
    InvalidSkinVariant {
        player_class: OfflinePlayerClass,
        skin_variant: u16,
        skin_count: u16,
    },
    EntryBlocked(CharacterEntryBlock),
    NoPendingTakeOver,
    RenameNotRequired {
        character_id: u64,
    },
    CannotDeleteOnline {
        character_id: u64,
    },
    NoPendingDelete,
    DeleteConfirmationMismatch {
        character_id: u64,
    },
}

impl From<CharacterRosterError> for OnlineCharacterFlowError {
    fn from(error: CharacterRosterError) -> Self {
        Self::Roster(error)
    }
}

impl From<CharacterNameError> for OnlineCharacterFlowError {
    fn from(error: CharacterNameError) -> Self {
        Self::CharacterName(error)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OnlineCharacterCreateDraft {
    raw_name: String,
    player_class: OfflinePlayerClass,
    skin_variant: u16,
}

impl Default for OnlineCharacterCreateDraft {
    fn default() -> Self {
        Self {
            raw_name: String::new(),
            player_class: OfflinePlayerClass::Warrior,
            skin_variant: 0,
        }
    }
}

impl OnlineCharacterCreateDraft {
    pub fn raw_name(&self) -> &str {
        &self.raw_name
    }

    pub fn player_class(&self) -> OfflinePlayerClass {
        self.player_class
    }

    pub fn skin_variant(&self) -> u16 {
        self.skin_variant
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OnlineCharacterDeleteDialog {
    character_id: u64,
    character_name: String,
    confirmation: String,
}

impl OnlineCharacterDeleteDialog {
    pub fn character_id(&self) -> u64 {
        self.character_id
    }

    pub fn character_name(&self) -> &str {
        &self.character_name
    }

    pub fn confirmation(&self) -> &str {
        &self.confirmation
    }

    pub fn confirmation_matches(&self) -> bool {
        normalize_delete_confirmation(&self.confirmation)
            == normalize_delete_confirmation(&self.character_name)
    }
}

pub struct OnlineCharacterFlow {
    roster: CharacterRosterModel,
    screen: CharacterRosterScreen,
    create_draft: OnlineCharacterCreateDraft,
    pending_takeover: Option<u64>,
    delete_dialog: Option<OnlineCharacterDeleteDialog>,
}

impl OnlineCharacterFlow {
    pub fn new(sort_mode: CharacterSortMode) -> Self {
        Self {
            roster: CharacterRosterModel::new(sort_mode),
            screen: CharacterRosterScreen::SelectCharacter,
            create_draft: OnlineCharacterCreateDraft::default(),
            pending_takeover: None,
            delete_dialog: None,
        }
    }

    pub fn roster(&self) -> &CharacterRosterModel {
        &self.roster
    }

    pub fn screen(&self) -> CharacterRosterScreen {
        self.screen
    }

    pub fn create_draft(&self) -> &OnlineCharacterCreateDraft {
        &self.create_draft
    }

    pub fn delete_dialog(&self) -> Option<&OnlineCharacterDeleteDialog> {
        self.delete_dialog.as_ref()
    }

    pub fn replace_roster(
        &mut self,
        entries: Vec<CharacterRosterEntry>,
    ) -> Result<(), OnlineCharacterFlowError> {
        let mut roster = CharacterRosterModel::new(self.roster.sort_mode());
        roster.replace_entries(entries)?;
        self.roster = roster;
        self.screen = self.roster.screen();
        self.pending_takeover = self.pending_takeover.filter(|character_id| {
            self.roster
                .entries()
                .iter()
                .any(|entry| entry.id == *character_id && entry.online && !entry.force_rename)
        });
        let keep_delete = self.delete_dialog.as_ref().is_some_and(|dialog| {
            self.roster
                .entries()
                .iter()
                .any(|entry| entry.id == dialog.character_id && !entry.online)
        });
        if !keep_delete {
            self.delete_dialog = None;
        }
        Ok(())
    }

    pub fn select(&mut self, character_id: u64) -> Result<(), OnlineCharacterFlowError> {
        self.require_screen("select", CharacterRosterScreen::SelectCharacter)?;
        self.roster.select(character_id)?;
        Ok(())
    }

    pub fn set_sort_mode(
        &mut self,
        mode: CharacterSortMode,
    ) -> Result<OnlineCharacterEffect, OnlineCharacterFlowError> {
        self.require_screen("set_sort_mode", CharacterRosterScreen::SelectCharacter)?;
        self.roster.set_sort_mode(mode);
        Ok(OnlineCharacterEffect::PersistSortAndRefresh { mode })
    }

    pub fn open_create(&mut self) -> Result<(), OnlineCharacterFlowError> {
        self.require_screen("open_create", CharacterRosterScreen::SelectCharacter)?;
        self.screen = CharacterRosterScreen::CreateCharacter;
        Ok(())
    }

    pub fn back(&mut self) -> Result<Option<OnlineCharacterEffect>, OnlineCharacterFlowError> {
        match self.screen {
            CharacterRosterScreen::CreateCharacter => {
                self.screen = CharacterRosterScreen::SelectCharacter;
                Ok(None)
            }
            CharacterRosterScreen::SelectCharacter => {
                Ok(Some(OnlineCharacterEffect::NavigateToLogin))
            }
        }
    }

    pub fn primary_action(&mut self) -> Result<OnlineCharacterEffect, OnlineCharacterFlowError> {
        self.require_screen("primary_action", CharacterRosterScreen::SelectCharacter)?;
        match self.roster.primary_action() {
            CharacterPrimaryAction::Disabled(block) => {
                Err(OnlineCharacterFlowError::EntryBlocked(block))
            }
            CharacterPrimaryAction::EnterWorld { character_id } => {
                Ok(OnlineCharacterEffect::EnterWorld { character_id })
            }
            CharacterPrimaryAction::TakeOver { character_id } => {
                let character_name = self
                    .roster
                    .selected()
                    .expect("TakeOver action must have a selected roster entry")
                    .name
                    .clone();
                self.pending_takeover = Some(character_id);
                Ok(OnlineCharacterEffect::ConfirmTakeOver {
                    character_id,
                    character_name,
                })
            }
        }
    }

    pub fn cancel_takeover(&mut self) {
        self.pending_takeover = None;
    }

    pub fn confirm_takeover(&mut self) -> Result<OnlineCharacterEffect, OnlineCharacterFlowError> {
        let character_id = self
            .pending_takeover
            .take()
            .ok_or(OnlineCharacterFlowError::NoPendingTakeOver)?;
        Ok(OnlineCharacterEffect::TakeOverAndEnter { character_id })
    }

    pub fn set_create_name(
        &mut self,
        raw_name: impl Into<String>,
    ) -> Result<(), OnlineCharacterFlowError> {
        self.require_screen("set_create_name", CharacterRosterScreen::CreateCharacter)?;
        self.create_draft.raw_name = raw_name.into();
        Ok(())
    }

    pub fn set_create_class(
        &mut self,
        player_class: OfflinePlayerClass,
    ) -> Result<(), OnlineCharacterFlowError> {
        self.require_screen("set_create_class", CharacterRosterScreen::CreateCharacter)?;
        self.create_draft.player_class = player_class;
        self.create_draft.skin_variant = 0;
        Ok(())
    }

    pub fn set_create_skin(&mut self, skin_variant: u16) -> Result<(), OnlineCharacterFlowError> {
        self.require_screen("set_create_skin", CharacterRosterScreen::CreateCharacter)?;
        let player_class = self.create_draft.player_class;
        let skin_count = offline_class_presentation(player_class).skin_count;
        if skin_variant >= skin_count {
            return Err(OnlineCharacterFlowError::InvalidSkinVariant {
                player_class,
                skin_variant,
                skin_count,
            });
        }
        self.create_draft.skin_variant = skin_variant;
        Ok(())
    }

    pub fn submit_create(&self) -> Result<OnlineCharacterEffect, OnlineCharacterFlowError> {
        self.require_screen("submit_create", CharacterRosterScreen::CreateCharacter)?;
        Ok(OnlineCharacterEffect::Create {
            name: normalize_character_name(&self.create_draft.raw_name)?,
            player_class: self.create_draft.player_class,
            skin_variant: self.create_draft.skin_variant,
        })
    }

    pub fn complete_create(&mut self) -> Result<(), OnlineCharacterFlowError> {
        self.require_screen("complete_create", CharacterRosterScreen::CreateCharacter)?;
        self.create_draft.raw_name.clear();
        self.screen = CharacterRosterScreen::SelectCharacter;
        Ok(())
    }

    pub fn submit_rename(
        &self,
        character_id: u64,
        raw_name: &str,
    ) -> Result<OnlineCharacterEffect, OnlineCharacterFlowError> {
        self.require_screen("submit_rename", CharacterRosterScreen::SelectCharacter)?;
        let entry = self.entry(character_id)?;
        if !entry.force_rename {
            return Err(OnlineCharacterFlowError::RenameNotRequired { character_id });
        }
        Ok(OnlineCharacterEffect::Rename {
            character_id,
            name: normalize_character_name(raw_name)?,
        })
    }

    pub fn open_delete(&mut self, character_id: u64) -> Result<(), OnlineCharacterFlowError> {
        self.require_screen("open_delete", CharacterRosterScreen::SelectCharacter)?;
        let entry = self.entry(character_id)?;
        if entry.online {
            return Err(OnlineCharacterFlowError::CannotDeleteOnline { character_id });
        }
        self.delete_dialog = Some(OnlineCharacterDeleteDialog {
            character_id,
            character_name: entry.name.clone(),
            confirmation: String::new(),
        });
        Ok(())
    }

    pub fn set_delete_confirmation(
        &mut self,
        confirmation: impl Into<String>,
    ) -> Result<(), OnlineCharacterFlowError> {
        let dialog = self
            .delete_dialog
            .as_mut()
            .ok_or(OnlineCharacterFlowError::NoPendingDelete)?;
        dialog.confirmation = confirmation.into();
        Ok(())
    }

    pub fn cancel_delete(&mut self) {
        self.delete_dialog = None;
    }

    pub fn submit_delete(&mut self) -> Result<OnlineCharacterEffect, OnlineCharacterFlowError> {
        let dialog = self
            .delete_dialog
            .as_ref()
            .ok_or(OnlineCharacterFlowError::NoPendingDelete)?;
        if !dialog.confirmation_matches() {
            return Err(OnlineCharacterFlowError::DeleteConfirmationMismatch {
                character_id: dialog.character_id,
            });
        }
        let effect = OnlineCharacterEffect::Delete {
            character_id: dialog.character_id,
            confirmation: dialog.confirmation.clone(),
        };
        self.delete_dialog = None;
        Ok(effect)
    }

    fn entry(&self, character_id: u64) -> Result<&CharacterRosterEntry, OnlineCharacterFlowError> {
        self.roster
            .entries()
            .iter()
            .find(|entry| entry.id == character_id)
            .ok_or_else(|| CharacterRosterError::CharacterNotFound { character_id }.into())
    }

    fn require_screen(
        &self,
        action: &'static str,
        expected: CharacterRosterScreen,
    ) -> Result<(), OnlineCharacterFlowError> {
        if self.screen == expected {
            Ok(())
        } else {
            Err(OnlineCharacterFlowError::InvalidScreen {
                action,
                expected,
                actual: self.screen,
            })
        }
    }
}

fn normalize_delete_confirmation(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}
