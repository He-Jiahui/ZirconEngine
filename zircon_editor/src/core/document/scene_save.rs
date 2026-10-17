use std::fmt;
use std::path::Path;

use zircon_runtime::scene::world::{ScenePublicationReceipt, SceneSourceRevision};

use super::{ActiveSceneDocumentIdentity, DocumentLifecycleAuthority};
use crate::core::editor_message::DocumentMessage;

/// Consumed by one explicit Save decision for the source observed by that prompt.
/// It deliberately cannot be cloned or installed as the document's ordinary save baseline.
#[derive(Debug)]
pub struct OverwriteLease {
    identity: ActiveSceneDocumentIdentity,
    observed: SceneSourceRevision,
}

impl OverwriteLease {
    pub(crate) fn for_observed_source(
        identity: ActiveSceneDocumentIdentity,
        observed: SceneSourceRevision,
    ) -> Self {
        Self { identity, observed }
    }
}

#[must_use]
#[derive(Clone, Debug)]
pub enum SceneSaveOutcome {
    Persisted {
        receipt: ScenePublicationReceipt,
    },
    PersistedProjectionFailed {
        receipt: ScenePublicationReceipt,
        phase: &'static str,
    },
    RecoveryDeferred {
        receipt: ScenePublicationReceipt,
    },
}

impl SceneSaveOutcome {
    pub fn receipt(&self) -> &ScenePublicationReceipt {
        match self {
            Self::Persisted { receipt }
            | Self::PersistedProjectionFailed { receipt, .. }
            | Self::RecoveryDeferred { receipt } => receipt,
        }
    }

    pub fn is_durable(&self) -> bool {
        self.receipt().is_durable()
    }
}

#[derive(Debug)]
pub enum SceneSaveError<E> {
    NoActiveScene,
    MissingSourceRevision,
    StaleOverwriteLease,
    Persistence(E),
}

impl<E: fmt::Display> fmt::Display for SceneSaveError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoActiveScene => f.write_str("no active scene document is available to save"),
            Self::MissingSourceRevision => {
                f.write_str("the active scene has no loaded source revision")
            }
            Self::StaleOverwriteLease => {
                f.write_str("the scene changed after this overwrite decision was prepared")
            }
            Self::Persistence(error) => error.fmt(f),
        }
    }
}

impl DocumentLifecycleAuthority {
    /// Serializes admission, publication and baseline advancement with document routing.
    /// The persistence closure must return the actual staged-object receipt.
    pub fn save_active_scene<E>(
        &self,
        root: &Path,
        overwrite: Option<OverwriteLease>,
        persist: impl FnOnce(
            &ActiveSceneDocumentIdentity,
            &SceneSourceRevision,
        ) -> Result<SceneSaveOutcome, E>,
    ) -> Result<(SceneSaveOutcome, Option<DocumentMessage>), SceneSaveError<E>> {
        self.with_scene_route(|| {
            let identity = self
                .active_scene_identity_while_routed(root)
                .ok_or(SceneSaveError::NoActiveScene)?;
            let expected = match overwrite {
                Some(lease) => {
                    if lease.identity != identity {
                        return Err(SceneSaveError::StaleOverwriteLease);
                    }
                    lease.observed
                }
                None => identity
                    .source_revision()
                    .cloned()
                    .ok_or(SceneSaveError::MissingSourceRevision)?,
            };
            let outcome = persist(&identity, &expected).map_err(SceneSaveError::Persistence)?;
            if outcome.is_durable() {
                let updated = self.replace_scene_source_revision_while_routed(
                    &identity,
                    outcome.receipt().source_revision().clone(),
                );
                debug_assert!(updated, "scene route remains owned throughout publication");
            }
            let saved =
                if outcome.is_durable() && matches!(outcome, SceneSaveOutcome::Persisted { .. }) {
                    Some(DocumentMessage::Saved {
                        doc: identity.document(),
                    })
                } else {
                    None
                };
            Ok((outcome, saved))
        })
    }
}
