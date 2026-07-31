use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;

use toml::{Table, Value};
use zircon_runtime_interface::ui::dispatch::UiKeyboardInputEvent;

use crate::core::editor_operation::{EditorOperationPath, EditorOperationPathError};
use crate::core::settings::EditorKeymapOverrides;

use super::{EditorKeyChord, EditorKeyChordSignature, EditorKeyboardChordInput};

#[cfg(test)]
mod tests;

const DEFAULT_KEYMAP_TOML: &str =
    include_str!("../../../assets/ui/editor/keymap/default.keymap.toml");

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EditorKeymap {
    bindings: Vec<EditorKeyBinding>,
    base_bindings: BTreeMap<EditorOperationPath, EditorKeyChord>,
    signature_index: HashMap<EditorKeyChordSignature, Vec<usize>>,
}

impl EditorKeymap {
    pub fn default_workbench() -> Self {
        Self::from_toml(DEFAULT_KEYMAP_TOML).expect("default editor keymap asset is valid")
    }

    fn from_toml(source: &str) -> Result<Self, EditorKeymapError> {
        let table = source
            .parse::<Table>()
            .map_err(|error| EditorKeymapError::InvalidToml(error.to_string()))?;
        let binding_table = table
            .get("bindings")
            .and_then(Value::as_table)
            .ok_or(EditorKeymapError::MissingBindingsTable)?;
        let mut bindings = BTreeMap::new();
        for (command_id, chord_value) in binding_table {
            let command_id = EditorOperationPath::parse(command_id.clone())
                .map_err(EditorKeymapError::InvalidOperationPath)?;
            let chord = chord_value
                .as_str()
                .ok_or_else(|| EditorKeymapError::InvalidChordValue(command_id.to_string()))?;
            bindings.insert(
                command_id,
                EditorKeyChord::from_str(chord).map_err(EditorKeymapError::InvalidChord)?,
            );
        }
        Ok(Self::from_base_and_overrides(bindings, BTreeMap::new()))
    }

    pub fn bindings(&self) -> &[EditorKeyBinding] {
        &self.bindings
    }

    pub fn resolve(&self, chord: &EditorKeyChord) -> Option<&str> {
        self.signature_index
            .get(&chord.signature())?
            .iter()
            .copied()
            .find_map(|index| {
                let binding = self.bindings.get(index)?;
                (binding.chord == *chord).then_some(binding.command_id.as_str())
            })
    }

    pub fn resolve_keyboard_input(&self, keyboard: &UiKeyboardInputEvent) -> Option<&str> {
        let input = EditorKeyboardChordInput::from_keyboard_input(keyboard)?;
        self.signature_index
            .get(&input.signature())?
            .iter()
            .copied()
            .find_map(|index| {
                let binding = self.bindings.get(index)?;
                input
                    .matches(&binding.chord)
                    .then_some(binding.command_id.as_str())
            })
    }

    pub fn chord_for_command(&self, command_id: &str) -> Option<&EditorKeyChord> {
        let index = self
            .bindings
            .binary_search_by(|binding| binding.command_id.as_str().cmp(command_id))
            .ok()?;
        Some(&self.bindings[index].chord)
    }

    /// Applies a typed settings delta over this immutable built-in preset.
    pub fn with_overrides(&self, overrides: &EditorKeymapOverrides) -> Self {
        Self::from_base_and_overrides(self.base_bindings.clone(), overrides.bindings().clone())
    }

    /// Returns all effective bindings which share the same chord.
    pub fn conflicts(&self) -> Vec<EditorKeymapConflict> {
        let mut commands_by_chord = BTreeMap::<EditorKeyChord, Vec<EditorOperationPath>>::new();
        for binding in &self.bindings {
            commands_by_chord
                .entry(binding.chord.clone())
                .or_default()
                .push(binding.command_id.clone());
        }
        commands_by_chord
            .into_iter()
            .filter_map(|(chord, command_ids)| {
                (command_ids.len() > 1).then_some(EditorKeymapConflict { chord, command_ids })
            })
            .collect()
    }

    fn from_base_and_overrides(
        base_bindings: BTreeMap<EditorOperationPath, EditorKeyChord>,
        overrides: BTreeMap<EditorOperationPath, Option<EditorKeyChord>>,
    ) -> Self {
        let mut effective = base_bindings.clone();
        for (command_id, chord) in &overrides {
            match chord {
                Some(chord) => {
                    effective.insert(command_id.clone(), chord.clone());
                }
                None => {
                    effective.remove(command_id);
                }
            }
        }
        let bindings = effective
            .into_iter()
            .map(|(command_id, chord)| EditorKeyBinding { command_id, chord })
            .collect::<Vec<_>>();
        let mut signature_index = HashMap::<EditorKeyChordSignature, Vec<usize>>::new();
        for (index, binding) in bindings.iter().enumerate() {
            signature_index
                .entry(binding.chord.signature())
                .or_default()
                .push(index);
        }
        Self {
            bindings,
            base_bindings,
            signature_index,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorKeyBinding {
    command_id: EditorOperationPath,
    chord: EditorKeyChord,
}

impl EditorKeyBinding {
    pub fn command_id(&self) -> &str {
        self.command_id.as_str()
    }

    pub fn chord(&self) -> &EditorKeyChord {
        &self.chord
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorKeymapConflict {
    chord: EditorKeyChord,
    command_ids: Vec<EditorOperationPath>,
}

impl EditorKeymapConflict {
    pub fn chord(&self) -> &EditorKeyChord {
        &self.chord
    }

    pub fn command_ids(&self) -> &[EditorOperationPath] {
        &self.command_ids
    }
}

#[derive(Debug)]
pub enum EditorKeymapError {
    InvalidToml(String),
    MissingBindingsTable,
    InvalidChordValue(String),
    InvalidChord(super::EditorKeyChordParseError),
    InvalidOperationPath(EditorOperationPathError),
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
            Self::InvalidOperationPath(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for EditorKeymapError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidChord(error) => Some(error),
            Self::InvalidOperationPath(error) => Some(error),
            Self::InvalidToml(_) | Self::MissingBindingsTable | Self::InvalidChordValue(_) => None,
        }
    }
}
