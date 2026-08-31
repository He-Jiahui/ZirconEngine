use std::fmt::{self, Display, Formatter};

use crate::core::editor_message::DocumentId;

use super::{ActiveSceneDocumentIdentity, DocumentLifecycleAuthority};

/// Host port for a prepared active-scene reload.
///
/// The coordinator owns identity serialization. The host remains responsible for checking dirty
/// state and atomically replacing its authoring world.
pub trait ActiveSceneReloader {
    type Error;

    fn prepare_active_scene_reload(&mut self) -> Result<(), Self::Error>;

    fn install_active_scene_reload(&mut self) -> Result<(), Self::Error>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneDocumentReloadOutcome {
    Reloaded { document: DocumentId },
    Superseded,
}

#[derive(Debug)]
pub enum SceneDocumentReloadError<ReloadError> {
    Transition(ReloadError),
    Install(ReloadError),
}

impl<ReloadError: Display> Display for SceneDocumentReloadError<ReloadError> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transition(error) => {
                write!(formatter, "active scene reload was not admitted: {error}")
            }
            Self::Install(error) => write!(formatter, "active scene reload failed: {error}"),
        }
    }
}

impl<ReloadError: std::error::Error + 'static> std::error::Error
    for SceneDocumentReloadError<ReloadError>
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Transition(error) | Self::Install(error) => Some(error),
        }
    }
}

/// Serializes same-scene reloads with open/create/project-session transitions.
pub struct SceneDocumentReloadCoordinator<'a> {
    lifecycle: &'a DocumentLifecycleAuthority,
}

impl<'a> SceneDocumentReloadCoordinator<'a> {
    pub const fn new(lifecycle: &'a DocumentLifecycleAuthority) -> Self {
        Self { lifecycle }
    }

    pub fn reload<Reloader>(
        &self,
        expected: &ActiveSceneDocumentIdentity,
        reloader: &mut Reloader,
    ) -> Result<SceneDocumentReloadOutcome, SceneDocumentReloadError<Reloader::Error>>
    where
        Reloader: ActiveSceneReloader,
    {
        self.lifecycle.with_scene_route(|| {
            if self
                .lifecycle
                .active_scene_identity_while_routed(expected.project_root())
                .as_ref()
                != Some(expected)
            {
                return Ok(SceneDocumentReloadOutcome::Superseded);
            }
            reloader
                .prepare_active_scene_reload()
                .map_err(SceneDocumentReloadError::Transition)?;
            reloader
                .install_active_scene_reload()
                .map_err(SceneDocumentReloadError::Install)?;
            Ok(SceneDocumentReloadOutcome::Reloaded {
                document: expected.document(),
            })
        })
    }
}
