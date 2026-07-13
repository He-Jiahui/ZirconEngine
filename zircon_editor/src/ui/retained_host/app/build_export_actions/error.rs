use std::{io, path::PathBuf};

use thiserror::Error;
use zircon_runtime::{asset::project::ProjectManifestError, scene::world::SceneProjectError};

#[derive(Debug, Error)]
pub(super) enum DesktopExportActionError {
    #[error(transparent)]
    ProjectRoot(#[from] SceneProjectError),
    #[error("unknown desktop export profile {profile_name}")]
    UnknownProfile { profile_name: String },
    #[error("failed to load desktop export project manifest: {source}")]
    Manifest {
        #[source]
        source: ProjectManifestError,
    },
    #[error("desktop folder selection is unsupported on this host")]
    PickerUnsupported,
    #[error("no desktop folder picker command was available: {programs:?}")]
    PickerUnavailable { programs: Vec<&'static str> },
    #[error("folder picker {program} failed with status {status_code:?}: {stderr}")]
    PickerExit {
        program: &'static str,
        status_code: Option<i32>,
        stderr: String,
    },
    #[error("failed to start folder picker {program}: {source}")]
    PickerSpawn {
        program: &'static str,
        #[source]
        source: io::Error,
    },
    #[error("opening desktop export output folders is unsupported on this host")]
    RevealUnsupported,
    #[error("failed to create desktop export output folder {}: {source}", path.display())]
    CreateOutput {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to open desktop export output folder {}: {source}", path.display())]
    RevealSpawn {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}
