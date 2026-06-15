use std::str::FromStr;

use toml::{Table, Value};
use zircon_runtime_interface::ui::dispatch::UiKeyboardInputEvent;

use super::EditorKeyChord;

const DEFAULT_KEYMAP_TOML: &str =
    include_str!("../../../../assets/ui/editor/keymap/default.keymap.toml");

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EditorKeymap {
    bindings: Vec<EditorKeyBinding>,
}

impl EditorKeymap {
    pub fn default_workbench() -> Self {
        Self::from_toml(DEFAULT_KEYMAP_TOML).expect("default editor keymap asset is valid")
    }

    pub fn from_toml(source: &str) -> Result<Self, EditorKeymapError> {
        let table = source
            .parse::<Table>()
            .map_err(|error| EditorKeymapError::InvalidToml(error.to_string()))?;
        let binding_table = table
            .get("bindings")
            .and_then(Value::as_table)
            .ok_or(EditorKeymapError::MissingBindingsTable)?;
        let mut bindings = Vec::new();
        for (command_id, chord_value) in binding_table {
            let chord = chord_value
                .as_str()
                .ok_or_else(|| EditorKeymapError::InvalidChordValue(command_id.clone()))?;
            bindings.push(EditorKeyBinding {
                command_id: command_id.clone(),
                chord: EditorKeyChord::from_str(chord).map_err(EditorKeymapError::InvalidChord)?,
            });
        }
        bindings.sort_by(|left, right| left.command_id.cmp(&right.command_id));
        Ok(Self { bindings })
    }

    pub fn bindings(&self) -> &[EditorKeyBinding] {
        &self.bindings
    }

    pub fn resolve(&self, chord: &EditorKeyChord) -> Option<&str> {
        self.bindings
            .iter()
            .find(|binding| &binding.chord == chord)
            .map(|binding| binding.command_id.as_str())
    }

    pub fn resolve_keyboard_input(&self, keyboard: &UiKeyboardInputEvent) -> Option<&str> {
        let chord = EditorKeyChord::from_keyboard_input(keyboard)?;
        self.resolve(&chord)
    }

    pub fn chord_for_command(&self, command_id: &str) -> Option<&EditorKeyChord> {
        self.bindings
            .iter()
            .find(|binding| binding.command_id == command_id)
            .map(|binding| &binding.chord)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorKeyBinding {
    command_id: String,
    chord: EditorKeyChord,
}

impl EditorKeyBinding {
    pub fn command_id(&self) -> &str {
        &self.command_id
    }

    pub fn chord(&self) -> &EditorKeyChord {
        &self.chord
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorKeymapError {
    InvalidToml(String),
    MissingBindingsTable,
    InvalidChordValue(String),
    InvalidChord(super::EditorKeyChordParseError),
}

impl std::fmt::Display for EditorKeymapError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidToml(error) => write!(formatter, "invalid editor keymap TOML: {error}"),
            Self::MissingBindingsTable => {
                formatter.write_str("editor keymap has no bindings table")
            }
            Self::InvalidChordValue(command_id) => {
                write!(
                    formatter,
                    "editor keymap command `{command_id}` has a non-string chord"
                )
            }
            Self::InvalidChord(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for EditorKeymapError {}
