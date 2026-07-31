use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::commands::EditorKeyChord;
use crate::core::editor_operation::EditorOperationPath;

/// Typed per-command keymap delta persisted by the User settings layer.
///
/// `None` is an explicit tombstone, so a user can keep a command unbound even
/// when a later built-in preset adds a binding for it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EditorKeymapOverrides(BTreeMap<EditorOperationPath, Option<EditorKeyChord>>);

impl EditorKeymapOverrides {
    pub fn new(bindings: BTreeMap<EditorOperationPath, Option<EditorKeyChord>>) -> Self {
        Self(bindings)
    }

    pub fn bindings(&self) -> &BTreeMap<EditorOperationPath, Option<EditorKeyChord>> {
        &self.0
    }
}
