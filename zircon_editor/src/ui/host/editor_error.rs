use std::path::PathBuf;

use thiserror::Error;

use zircon_runtime::asset::AssetImportError;
use zircon_runtime::core::resource::ResourceLocatorError;
use zircon_runtime::core::CoreError;
use zircon_runtime::scene::world::SceneProjectError;

use crate::core::asset::DirtyRegistryError;
use crate::core::extension::{
    SaveError, ToolkitInstanceIdError, ToolkitLayoutError, ToolkitRegistryError,
};
use crate::core::project::ProjectAuthorityError;
use crate::core::recovery::DocumentJournalCoordinatorError;
use crate::ui::animation_editor::AnimationEditorCommandDiagnostic;
use crate::ui::workbench::layout_persistence_document::LayoutPersistenceDocumentError;

use super::editor_save_batch::EditorDirtySaveError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationEditorTargetKind {
    Sequence,
    Graph,
    StateMachine,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationEditorTargetUnavailableReason {
    NoFocusedView,
    MissingFocusedView,
    WrongFocusedViewKind,
    WrongDocumentKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimationEditorTargetDiagnostic {
    target: AnimationEditorTargetKind,
    reason: AnimationEditorTargetUnavailableReason,
}

impl AnimationEditorTargetDiagnostic {
    pub const fn new(
        target: AnimationEditorTargetKind,
        reason: AnimationEditorTargetUnavailableReason,
    ) -> Self {
        Self { target, reason }
    }

    pub const fn code(self) -> &'static str {
        match self.target {
            AnimationEditorTargetKind::Sequence => match self.reason {
                AnimationEditorTargetUnavailableReason::NoFocusedView => "ZR-ANIM-TARGET-001",
                AnimationEditorTargetUnavailableReason::MissingFocusedView => "ZR-ANIM-TARGET-002",
                AnimationEditorTargetUnavailableReason::WrongFocusedViewKind => {
                    "ZR-ANIM-TARGET-003"
                }
                AnimationEditorTargetUnavailableReason::WrongDocumentKind => "ZR-ANIM-TARGET-007",
            },
            AnimationEditorTargetKind::Graph => match self.reason {
                AnimationEditorTargetUnavailableReason::NoFocusedView => "ZR-ANIM-TARGET-004",
                AnimationEditorTargetUnavailableReason::MissingFocusedView => "ZR-ANIM-TARGET-005",
                AnimationEditorTargetUnavailableReason::WrongFocusedViewKind => {
                    "ZR-ANIM-TARGET-006"
                }
                AnimationEditorTargetUnavailableReason::WrongDocumentKind => "ZR-ANIM-TARGET-008",
            },
            AnimationEditorTargetKind::StateMachine => match self.reason {
                AnimationEditorTargetUnavailableReason::NoFocusedView => "ZR-ANIM-TARGET-009",
                AnimationEditorTargetUnavailableReason::MissingFocusedView => "ZR-ANIM-TARGET-010",
                AnimationEditorTargetUnavailableReason::WrongFocusedViewKind => {
                    "ZR-ANIM-TARGET-011"
                }
                AnimationEditorTargetUnavailableReason::WrongDocumentKind => "ZR-ANIM-TARGET-012",
            },
        }
    }

    pub const fn message(self) -> &'static str {
        match self.target {
            AnimationEditorTargetKind::Sequence => match self.reason {
                AnimationEditorTargetUnavailableReason::NoFocusedView => {
                    "no focused animation sequence editor"
                }
                AnimationEditorTargetUnavailableReason::MissingFocusedView => {
                    "focused animation sequence view is missing"
                }
                AnimationEditorTargetUnavailableReason::WrongFocusedViewKind => {
                    "focused view is not an animation sequence editor"
                }
                AnimationEditorTargetUnavailableReason::WrongDocumentKind => {
                    "focused animation document is not a sequence"
                }
            },
            AnimationEditorTargetKind::Graph => match self.reason {
                AnimationEditorTargetUnavailableReason::NoFocusedView => {
                    "no focused animation graph editor"
                }
                AnimationEditorTargetUnavailableReason::MissingFocusedView => {
                    "focused animation graph view is missing"
                }
                AnimationEditorTargetUnavailableReason::WrongFocusedViewKind => {
                    "focused view is not an animation graph editor"
                }
                AnimationEditorTargetUnavailableReason::WrongDocumentKind => {
                    "focused animation document is not a graph"
                }
            },
            AnimationEditorTargetKind::StateMachine => match self.reason {
                AnimationEditorTargetUnavailableReason::NoFocusedView => {
                    "no focused animation state-machine editor"
                }
                AnimationEditorTargetUnavailableReason::MissingFocusedView => {
                    "focused animation state-machine view is missing"
                }
                AnimationEditorTargetUnavailableReason::WrongFocusedViewKind => {
                    "focused view is not an animation state-machine editor"
                }
                AnimationEditorTargetUnavailableReason::WrongDocumentKind => {
                    "focused animation document is not a state machine"
                }
            },
        }
    }
}

impl std::fmt::Display for AnimationEditorTargetDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code(), self.message())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimationEditorDocumentLoadDiagnostic {
    expected: &'static str,
    actual: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiAssetSaveStage {
    SourceRead,
    SourceRemoval,
    AtomicCommit,
    DurabilityBarrier,
    LocalCopyPublish,
}

impl std::fmt::Display for UiAssetSaveStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::SourceRead => "source read",
            Self::SourceRemoval => "source removal",
            Self::AtomicCommit => "atomic commit",
            Self::DurabilityBarrier => "durability barrier",
            Self::LocalCopyPublish => "local-copy publication",
        };
        f.write_str(label)
    }
}

impl AnimationEditorDocumentLoadDiagnostic {
    pub const fn binary_kind_mismatch(expected: &'static str, actual: &'static str) -> Self {
        Self { expected, actual }
    }

    pub const fn code(self) -> &'static str {
        "ZR-ANIM-LOAD-001"
    }

    pub const fn expected(self) -> &'static str {
        self.expected
    }

    pub const fn actual(self) -> &'static str {
        self.actual
    }
}

impl std::fmt::Display for AnimationEditorDocumentLoadDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] animation binary kind mismatch: expected {}, found {}",
            self.code(),
            self.expected,
            self.actual
        )
    }
}

#[derive(Debug, Error)]
pub enum EditorError {
    #[error("{0}")]
    Layout(String),
    #[error("workbench layout persistence failed: {source}")]
    LayoutPersistence {
        #[from]
        #[source]
        source: LayoutPersistenceDocumentError,
    },
    #[error("{0}")]
    Registry(String),
    #[error("document toolkit instance {instance:?} is not registered")]
    DocumentToolkitNotRegistered { instance: String },
    #[error("document toolkit registry failed: {source}")]
    DocumentToolkitRegistry {
        #[from]
        #[source]
        source: ToolkitRegistryError,
    },
    #[error("document toolkit save failed: {source}")]
    DocumentToolkitSave {
        #[from]
        #[source]
        source: SaveError,
    },
    #[error("document toolkit instance id failed: {source}")]
    DocumentToolkitInstanceId {
        #[from]
        #[source]
        source: ToolkitInstanceIdError,
    },
    #[error("document toolkit layout failed: {source}")]
    DocumentToolkitLayout {
        #[from]
        #[source]
        source: ToolkitLayoutError,
    },
    #[error("document dirty registry failed: {source}")]
    DirtyRegistry {
        #[from]
        #[source]
        source: DirtyRegistryError,
    },
    #[error("dirty document batch save failed: {source}")]
    DirtySave {
        #[from]
        #[source]
        source: EditorDirtySaveError,
    },
    #[error("document journal coordination failed: {source}")]
    DocumentJournal {
        #[from]
        #[source]
        source: DocumentJournalCoordinatorError,
    },
    #[error("{0}")]
    Project(String),
    #[error("Hub focus was forwarded to active editor process {process_id}")]
    HubFocusForwarded { process_id: u32 },
    #[error("project authority failed: {source}")]
    ProjectAuthority {
        #[from]
        #[source]
        source: ProjectAuthorityError,
    },
    #[error("{0}")]
    UiAsset(String),
    #[error("UI asset source changed before save: {source_path}")]
    UiAssetSourceConflict {
        asset_id: String,
        source_path: PathBuf,
        expected_digest: blake3::Hash,
        actual_digest: blake3::Hash,
    },
    #[error("document source changed before save: {source_path}")]
    DocumentSourceChanged { source_path: PathBuf },
    #[error("UI asset {stage} failed for {source_path}: {source}")]
    UiAssetSaveIo {
        stage: UiAssetSaveStage,
        source_path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{diagnostic}")]
    AnimationTargetUnavailable {
        diagnostic: AnimationEditorTargetDiagnostic,
    },
    #[error("{diagnostic}")]
    AnimationDocumentLoad {
        diagnostic: AnimationEditorDocumentLoadDiagnostic,
    },
    #[error("{diagnostic}")]
    AnimationCommandUnavailable {
        diagnostic: AnimationEditorCommandDiagnostic,
    },
    #[error("asset import failed: {source}")]
    AssetImport {
        #[from]
        #[source]
        source: AssetImportError,
    },
    #[error("resource locator failed: {source}")]
    ResourceLocator {
        #[from]
        #[source]
        source: ResourceLocatorError,
    },
    #[error("runtime service failed: {source}")]
    Core {
        #[from]
        #[source]
        source: CoreError,
    },
    #[error("project document failed: {source}")]
    SceneProject {
        #[from]
        #[source]
        source: SceneProjectError,
    },
    #[error("asset watcher failed: {source}")]
    AssetWatcher {
        #[from]
        #[source]
        source: notify::Error,
    },
}

impl EditorError {
    pub const fn animation_target_diagnostic(&self) -> Option<AnimationEditorTargetDiagnostic> {
        match self {
            Self::AnimationTargetUnavailable { diagnostic } => Some(*diagnostic),
            _ => None,
        }
    }

    pub const fn animation_document_load_diagnostic(
        &self,
    ) -> Option<AnimationEditorDocumentLoadDiagnostic> {
        match self {
            Self::AnimationDocumentLoad { diagnostic } => Some(*diagnostic),
            _ => None,
        }
    }

    pub fn animation_command_diagnostic(&self) -> Option<&AnimationEditorCommandDiagnostic> {
        match self {
            Self::AnimationCommandUnavailable { diagnostic } => Some(diagnostic),
            _ => None,
        }
    }

    pub const fn ui_asset_source_conflict(&self) -> Option<(&str, &PathBuf)> {
        match self {
            Self::UiAssetSourceConflict {
                asset_id,
                source_path,
                ..
            } => Some((asset_id, source_path)),
            _ => None,
        }
    }

    pub(crate) const fn hub_focus_forwarded_process_id(&self) -> Option<u32> {
        match self {
            Self::HubFocusForwarded { process_id } => Some(*process_id),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::{
        AnimationEditorDocumentLoadDiagnostic, AnimationEditorTargetDiagnostic,
        AnimationEditorTargetKind, AnimationEditorTargetUnavailableReason, EditorError,
    };
    use zircon_runtime::asset::AssetImportError;
    use zircon_runtime::core::resource::{ResourceLocator, ResourceLocatorError};

    #[test]
    fn typed_asset_and_uri_errors_remain_in_the_editor_error_source_chain() {
        let asset_error: EditorError = AssetImportError::MissingProjectAssetRoot.into();
        assert!(asset_error
            .source()
            .is_some_and(|source| source.downcast_ref::<AssetImportError>().is_some()));

        let uri_source = ResourceLocator::parse("not-a-resource-uri").unwrap_err();
        let uri_error: EditorError = uri_source.into();
        assert!(uri_error
            .source()
            .is_some_and(|source| source.downcast_ref::<ResourceLocatorError>().is_some()));
    }

    #[test]
    fn hub_focus_forwarding_exposes_the_existing_editor_process_id() {
        let error = EditorError::HubFocusForwarded { process_id: 913 };

        assert_eq!(error.hub_focus_forwarded_process_id(), Some(913));
        assert_eq!(
            EditorError::Project("other".to_string()).hub_focus_forwarded_process_id(),
            None
        );
    }

    #[test]
    fn animation_target_diagnostics_expose_stable_codes_without_message_matching() {
        let diagnostic = AnimationEditorTargetDiagnostic::new(
            AnimationEditorTargetKind::Graph,
            AnimationEditorTargetUnavailableReason::WrongFocusedViewKind,
        );
        let error = EditorError::AnimationTargetUnavailable { diagnostic };

        assert_eq!(diagnostic.code(), "ZR-ANIM-TARGET-006");
        assert_eq!(
            diagnostic.to_string(),
            "[ZR-ANIM-TARGET-006] focused view is not an animation graph editor"
        );
        assert_eq!(error.animation_target_diagnostic(), Some(diagnostic));
    }

    #[test]
    fn animation_document_load_diagnostic_exposes_binary_kind_mismatch_contract() {
        let diagnostic =
            AnimationEditorDocumentLoadDiagnostic::binary_kind_mismatch("sequence", "graph");
        let error = EditorError::AnimationDocumentLoad { diagnostic };

        assert_eq!(diagnostic.code(), "ZR-ANIM-LOAD-001");
        assert_eq!(diagnostic.expected(), "sequence");
        assert_eq!(diagnostic.actual(), "graph");
        assert_eq!(
            diagnostic.to_string(),
            "[ZR-ANIM-LOAD-001] animation binary kind mismatch: expected sequence, found graph"
        );
        assert_eq!(error.animation_document_load_diagnostic(), Some(diagnostic));
    }
}
