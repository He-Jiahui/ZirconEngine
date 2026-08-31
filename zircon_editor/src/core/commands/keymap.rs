use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;

use toml::{Table, Value};
use zircon_runtime_interface::ui::dispatch::UiKeyboardInputEvent;

use crate::core::editor_operation::{EditorOperationPath, EditorOperationPathError};
use crate::core::settings::EditorKeymapOverrides;

use super::{EditorKeyChord, EditorKeyChordSignature, EditorKeyboardChordInput, WhenClause};

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

    /// Resolves one enabled command from the matching signature bucket.
    ///
    /// The caller owns command-registry evaluation. Returning `None` for two enabled candidates
    /// keeps a malformed keymap from selecting a command by operation-path order.
    pub fn resolve_keyboard_input_when(
        &self,
        keyboard: &UiKeyboardInputEvent,
        mut is_enabled: impl FnMut(&str) -> bool,
    ) -> Option<&str> {
        let input = EditorKeyboardChordInput::from_keyboard_input(keyboard)?;
        let mut resolved = None;
        for index in self
            .signature_index
            .get(&input.signature())?
            .iter()
            .copied()
        {
            let binding = self.bindings.get(index)?;
            if !input.matches(&binding.chord) || !is_enabled(binding.command_id.as_str()) {
                continue;
            }
            if resolved.is_some() {
                return None;
            }
            resolved = Some(binding.command_id.as_str());
        }
        resolved
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

    /// Returns pairwise chord collisions whose effective `when` predicates can overlap.
    ///
    /// A missing command predicate is conservatively treated as a conflict so a stale keymap
    /// binding can never hide an ambiguous dispatch behind an absent registry entry.
    pub fn conflicts_with_when(
        &self,
        mut effective_when_for_command: impl FnMut(&str) -> Option<WhenClause>,
    ) -> Vec<EditorKeymapConflict> {
        let mut commands_by_chord = BTreeMap::<EditorKeyChord, Vec<&EditorKeyBinding>>::new();
        for binding in &self.bindings {
            commands_by_chord
                .entry(binding.chord.clone())
                .or_default()
                .push(binding);
        }
        let mut conflicts = Vec::new();
        for (chord, bindings) in commands_by_chord {
            for left_index in 0..bindings.len() {
                let left = bindings[left_index];
                let left_when = effective_when_for_command(left.command_id.as_str());
                for right in bindings.iter().skip(left_index + 1).copied() {
                    let right_when = effective_when_for_command(right.command_id.as_str());
                    let overlaps = match (left_when.as_ref(), right_when.as_ref()) {
                        (Some(left_when), Some(right_when)) => {
                            left_when.can_overlap_in_interactive_context(right_when)
                        }
                        (None, _) | (_, None) => true,
                    };
                    if overlaps {
                        conflicts.push(EditorKeymapConflict {
                            chord: chord.clone(),
                            first_command_id: left.command_id.clone(),
                            second_command_id: right.command_id.clone(),
                        });
                    }
                }
            }
        }
        conflicts
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
    first_command_id: EditorOperationPath,
    second_command_id: EditorOperationPath,
}

impl EditorKeymapConflict {
    pub fn chord(&self) -> &EditorKeyChord {
        &self.chord
    }

    pub fn first_command_id(&self) -> &EditorOperationPath {
        &self.first_command_id
    }

    pub fn second_command_id(&self) -> &EditorOperationPath {
        &self.second_command_id
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
