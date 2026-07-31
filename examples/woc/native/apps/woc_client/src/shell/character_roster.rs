use std::cmp::Ordering;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacterAppearanceRig {
    Class,
    Mech,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterRosterEntry {
    pub id: u64,
    pub name: String,
    pub class_id: String,
    pub level: u16,
    pub skin_variant: u16,
    pub appearance_rig: CharacterAppearanceRig,
    pub mainhand_item_id: Option<String>,
    pub offhand_item_id: Option<String>,
    pub online: bool,
    pub force_rename: bool,
    pub last_played_epoch_ms: Option<i64>,
    pub playtime_seconds: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacterSortMode {
    Level,
    Name,
    Recent,
    Playtime,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacterRosterScreen {
    CreateCharacter,
    SelectCharacter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacterEntryBlock {
    NoSelection,
    RenameRequired { character_id: u64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacterPrimaryAction {
    Disabled(CharacterEntryBlock),
    EnterWorld { character_id: u64 },
    TakeOver { character_id: u64 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CharacterRosterError {
    DuplicateCharacterId {
        character_id: u64,
    },
    InvalidField {
        character_id: u64,
        field: &'static str,
    },
    CharacterNotFound {
        character_id: u64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacterNameError {
    Empty,
    InvalidLength { actual: usize },
    FirstCharacter,
    InvalidCharacter { index: usize },
}

pub struct CharacterRosterModel {
    entries: Vec<CharacterRosterEntry>,
    sort_mode: CharacterSortMode,
    selected_id: Option<u64>,
}

impl CharacterRosterModel {
    pub fn new(sort_mode: CharacterSortMode) -> Self {
        Self {
            entries: Vec::new(),
            sort_mode,
            selected_id: None,
        }
    }

    pub fn replace_entries(
        &mut self,
        mut entries: Vec<CharacterRosterEntry>,
    ) -> Result<(), CharacterRosterError> {
        validate_entries(&entries)?;
        sort_entries(&mut entries, self.sort_mode);
        let selected_id = self
            .selected_id
            .filter(|selected| entries.iter().any(|entry| entry.id == *selected))
            .or_else(|| entries.first().map(|entry| entry.id));
        self.entries = entries;
        self.selected_id = selected_id;
        Ok(())
    }

    pub fn set_sort_mode(&mut self, sort_mode: CharacterSortMode) {
        self.sort_mode = sort_mode;
        sort_entries(&mut self.entries, sort_mode);
    }

    pub fn select(&mut self, character_id: u64) -> Result<(), CharacterRosterError> {
        if self.entries.iter().any(|entry| entry.id == character_id) {
            self.selected_id = Some(character_id);
            Ok(())
        } else {
            Err(CharacterRosterError::CharacterNotFound { character_id })
        }
    }

    pub fn entries(&self) -> &[CharacterRosterEntry] {
        &self.entries
    }

    pub fn sort_mode(&self) -> CharacterSortMode {
        self.sort_mode
    }

    pub fn selected_id(&self) -> Option<u64> {
        self.selected_id
    }

    pub fn selected(&self) -> Option<&CharacterRosterEntry> {
        self.selected_id
            .and_then(|selected| self.entries.iter().find(|entry| entry.id == selected))
    }

    pub fn screen(&self) -> CharacterRosterScreen {
        if self.entries.is_empty() {
            CharacterRosterScreen::CreateCharacter
        } else {
            CharacterRosterScreen::SelectCharacter
        }
    }

    pub fn primary_action(&self) -> CharacterPrimaryAction {
        let Some(selected) = self.selected() else {
            return CharacterPrimaryAction::Disabled(CharacterEntryBlock::NoSelection);
        };
        if selected.force_rename {
            CharacterPrimaryAction::Disabled(CharacterEntryBlock::RenameRequired {
                character_id: selected.id,
            })
        } else if selected.online {
            CharacterPrimaryAction::TakeOver {
                character_id: selected.id,
            }
        } else {
            CharacterPrimaryAction::EnterWorld {
                character_id: selected.id,
            }
        }
    }
}

fn validate_entries(entries: &[CharacterRosterEntry]) -> Result<(), CharacterRosterError> {
    let mut ids = BTreeSet::new();
    for entry in entries {
        for (field, invalid) in [
            ("id", entry.id == 0),
            ("name", entry.name.is_empty()),
            ("class_id", entry.class_id.is_empty()),
            ("level", entry.level == 0),
            (
                "mainhand_item_id",
                entry
                    .mainhand_item_id
                    .as_ref()
                    .is_some_and(String::is_empty),
            ),
            (
                "offhand_item_id",
                entry.offhand_item_id.as_ref().is_some_and(String::is_empty),
            ),
        ] {
            if invalid {
                return Err(CharacterRosterError::InvalidField {
                    character_id: entry.id,
                    field,
                });
            }
        }
        if !ids.insert(entry.id) {
            return Err(CharacterRosterError::DuplicateCharacterId {
                character_id: entry.id,
            });
        }
    }
    Ok(())
}

fn sort_entries(entries: &mut [CharacterRosterEntry], sort_mode: CharacterSortMode) {
    entries.sort_by(|left, right| {
        let primary = match sort_mode {
            CharacterSortMode::Level => right.level.cmp(&left.level),
            CharacterSortMode::Name => compare_name(left, right),
            CharacterSortMode::Recent => right
                .last_played_epoch_ms
                .unwrap_or(0)
                .cmp(&left.last_played_epoch_ms.unwrap_or(0)),
            CharacterSortMode::Playtime => right.playtime_seconds.cmp(&left.playtime_seconds),
        };
        primary
            .then_with(|| compare_name(left, right))
            .then_with(|| left.id.cmp(&right.id))
    });
}

fn compare_name(left: &CharacterRosterEntry, right: &CharacterRosterEntry) -> Ordering {
    left.name
        .to_ascii_lowercase()
        .cmp(&right.name.to_ascii_lowercase())
        .then_with(|| left.name.cmp(&right.name))
}

pub fn normalize_character_name(raw: &str) -> Result<String, CharacterNameError> {
    let normalized = raw.trim().to_string();
    if normalized.is_empty() {
        return Err(CharacterNameError::Empty);
    }
    let bytes = normalized.as_bytes();
    if !(2..=16).contains(&bytes.len()) {
        return Err(CharacterNameError::InvalidLength {
            actual: bytes.len(),
        });
    }
    if !bytes[0].is_ascii_alphabetic() {
        return Err(CharacterNameError::FirstCharacter);
    }
    if let Some((index, _)) = bytes
        .iter()
        .copied()
        .enumerate()
        .skip(1)
        .find(|(_, byte)| !byte.is_ascii_alphabetic() && !matches!(byte, b'\'' | b' ' | b'-'))
    {
        return Err(CharacterNameError::InvalidCharacter { index });
    }
    Ok(normalized)
}
