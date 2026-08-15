use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::core::asset::AssetWriteAccess;
use crate::core::editor_message::{PlayStateKind, SceneModeId};
use crate::scene::selection::WorldDomain;

use super::{DocumentKind, PlayModePredicate};

/// Structured, serializable enablement predicate shared by every command surface.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum WhenClause {
    #[default]
    Always,
    ProjectOpen,
    UndoAvailable,
    RedoAvailable,
    FocusedDocumentKind(DocumentKind),
    SceneModeActive(SceneModeId),
    SelectionNonEmpty,
    AssetWritable,
    PlayMode(PlayModePredicate),
    Capability(String),
    All(Vec<WhenClause>),
    Any(Vec<WhenClause>),
    Not(Box<WhenClause>),
}

impl WhenClause {
    pub fn eval(&self, context: &CommandEvalCtx) -> bool {
        self.eval_applicable(context).unwrap_or(false)
    }

    pub(crate) fn all(clauses: impl IntoIterator<Item = WhenClause>) -> Self {
        let mut flattened = Vec::new();
        for clause in clauses {
            match clause {
                Self::Always => {}
                Self::All(nested) => flattened.extend(nested),
                other => flattened.push(other),
            }
        }
        flattened.sort();
        flattened.dedup();
        match flattened.len() {
            0 => Self::Always,
            1 => flattened.pop().expect("single when clause"),
            _ => Self::All(flattened),
        }
    }

    fn eval_applicable(&self, context: &CommandEvalCtx) -> Option<bool> {
        match self {
            Self::Always => Some(true),
            Self::Capability(capability) => Some(context.has_capability(capability)),
            Self::ProjectOpen => context.interactive.then_some(context.project_open),
            Self::UndoAvailable => context.interactive.then_some(context.undo_available),
            Self::RedoAvailable => context.interactive.then_some(context.redo_available),
            Self::FocusedDocumentKind(kind) => context
                .interactive
                .then_some(context.focused_document_kind.as_ref() == Some(kind)),
            Self::SceneModeActive(mode) => context
                .interactive
                .then_some(context.scene_mode.as_ref() == Some(mode)),
            Self::SelectionNonEmpty => context.interactive.then_some(context.selection_count > 0),
            Self::AssetWritable => Some(context.asset_write_access == AssetWriteAccess::Writable),
            Self::PlayMode(predicate) => context
                .interactive
                .then_some(predicate.matches(context.play_state)),
            Self::All(clauses) => eval_all(clauses, context),
            Self::Any(clauses) => eval_any(clauses, context),
            Self::Not(clause) => clause.eval_applicable(context).map(|value| !value),
        }
    }
}

fn eval_all(clauses: &[WhenClause], context: &CommandEvalCtx) -> Option<bool> {
    let mut value = true;
    for clause in clauses {
        match clause.eval_applicable(context) {
            Some(clause_value) => value &= clause_value,
            None => return None,
        }
    }
    Some(value)
}

fn eval_any(clauses: &[WhenClause], context: &CommandEvalCtx) -> Option<bool> {
    let mut inapplicable = false;
    for clause in clauses {
        match clause.eval_applicable(context) {
            Some(true) => return Some(true),
            Some(false) => {}
            None => inapplicable = true,
        }
    }
    (!inapplicable).then_some(false)
}

/// Immutable command-evaluation snapshot. Headless snapshots deliberately lack UI state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandEvalCtx {
    interactive: bool,
    project_open: bool,
    undo_available: bool,
    redo_available: bool,
    focused_document_kind: Option<DocumentKind>,
    scene_mode: Option<SceneModeId>,
    selection_count: usize,
    #[serde(default)]
    selection_domain: WorldDomain,
    #[serde(default)]
    selection_revision: u64,
    #[serde(default)]
    scene_mode_revision: u64,
    #[serde(default)]
    asset_write_access: AssetWriteAccess,
    play_state: PlayStateKind,
    capabilities: BTreeSet<String>,
}

impl CommandEvalCtx {
    pub fn interactive() -> Self {
        Self {
            interactive: true,
            project_open: false,
            undo_available: false,
            redo_available: false,
            focused_document_kind: None,
            scene_mode: None,
            selection_count: 0,
            selection_domain: WorldDomain::default(),
            selection_revision: 0,
            scene_mode_revision: 0,
            asset_write_access: AssetWriteAccess::ReadOnly,
            play_state: PlayStateKind::Edit,
            capabilities: BTreeSet::new(),
        }
    }

    pub fn headless<I, S>(capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            interactive: false,
            capabilities: normalize_capabilities(capabilities),
            ..Self::interactive()
        }
    }

    pub fn with_project_open(mut self, value: bool) -> Self {
        self.project_open = value;
        self
    }

    pub fn with_undo_available(mut self, value: bool) -> Self {
        self.undo_available = value;
        self
    }

    pub fn with_redo_available(mut self, value: bool) -> Self {
        self.redo_available = value;
        self
    }

    pub fn with_focused_document_kind(mut self, kind: DocumentKind) -> Self {
        self.focused_document_kind = Some(kind);
        self
    }

    pub fn with_optional_focused_document_kind(mut self, kind: Option<DocumentKind>) -> Self {
        self.focused_document_kind = kind;
        self
    }

    pub fn focused_document_kind(&self) -> Option<&DocumentKind> {
        self.focused_document_kind.as_ref()
    }

    pub fn with_scene_mode(mut self, mode: SceneModeId) -> Self {
        self.scene_mode = Some(mode);
        self
    }

    pub fn with_selection_count(mut self, count: usize) -> Self {
        self.selection_count = count;
        self
    }

    /// Binds this snapshot to the active authoring or play selection domain.
    pub fn with_selection_domain(mut self, domain: WorldDomain) -> Self {
        self.selection_domain = domain;
        self
    }

    /// Binds this snapshot to the authoritative selection identity, not only its cardinality.
    pub fn with_selection_revision(mut self, revision: u64) -> Self {
        self.selection_revision = revision;
        self
    }

    /// Binds this snapshot to the active scene-mode topology generation.
    pub fn with_scene_mode_revision(mut self, revision: u64) -> Self {
        self.scene_mode_revision = revision;
        self
    }

    pub fn with_asset_write_access(mut self, access: AssetWriteAccess) -> Self {
        self.asset_write_access = access;
        self
    }

    pub fn asset_write_access(&self) -> AssetWriteAccess {
        self.asset_write_access
    }

    pub fn with_play_state(mut self, state: PlayStateKind) -> Self {
        self.play_state = state;
        self
    }

    pub fn with_capabilities<I, S>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.capabilities = normalize_capabilities(capabilities);
        self
    }

    pub fn is_interactive(&self) -> bool {
        self.interactive
    }

    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.contains(capability)
    }

    pub fn capabilities(&self) -> &BTreeSet<String> {
        &self.capabilities
    }
}

impl Default for CommandEvalCtx {
    fn default() -> Self {
        Self::headless(std::iter::empty::<String>())
    }
}

fn normalize_capabilities<I, S>(capabilities: I) -> BTreeSet<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    capabilities.into_iter().map(Into::into).collect()
}
