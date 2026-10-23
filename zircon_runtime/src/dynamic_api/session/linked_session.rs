use std::path::Path;

use zircon_runtime_interface::project::RelPath;
use zircon_runtime_interface::ZrRuntimeSessionHandle;

use crate::plugin::{RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport};

use super::profile::RuntimeDynamicSessionProfile;
use super::project::RuntimeProjectConfig;
use super::registry::{try_create_linked_session, SessionRegistryInsertError};
use super::RuntimeDynamicSessionError;

pub fn create_linked_runtime_session(
    profile: &[u8],
    project_root: Option<&Path>,
    registrations: Vec<RuntimePluginRegistrationReport>,
) -> Result<ZrRuntimeSessionHandle, RuntimeDynamicSessionError> {
    let profile = parse_profile(profile)?;
    let project_config = project_root
        .map(RuntimeProjectConfig::from_root)
        .transpose()
        .map_err(|source| RuntimeDynamicSessionError::ProjectStep {
            step: "resolve linked runtime project root",
            source,
        })?;
    create_linked_session(profile, project_config, registrations, Vec::new())
}

/// Creates a linked session from the editor's materialized Play snapshot.
/// Project preparation applies the same path and snapshot admission as the ABI path.
pub fn create_linked_runtime_play_session(
    profile: &[u8],
    project_root: &Path,
    play_scene: &RelPath,
    registrations: Vec<RuntimePluginRegistrationReport>,
    feature_registrations: Vec<RuntimePluginFeatureRegistrationReport>,
) -> Result<ZrRuntimeSessionHandle, RuntimeDynamicSessionError> {
    let profile = parse_profile(profile)?;
    let project = RuntimeProjectConfig::from_root(project_root)
        .map_err(|source| RuntimeDynamicSessionError::ProjectStep {
            step: "resolve linked Play project root",
            source,
        })?
        .with_play_scene(play_scene.clone());
    create_linked_session(profile, Some(project), registrations, feature_registrations)
}

fn create_linked_session(
    profile: RuntimeDynamicSessionProfile,
    project_config: Option<RuntimeProjectConfig>,
    registrations: Vec<RuntimePluginRegistrationReport>,
    feature_registrations: Vec<RuntimePluginFeatureRegistrationReport>,
) -> Result<ZrRuntimeSessionHandle, RuntimeDynamicSessionError> {
    try_create_linked_session(
        profile,
        project_config,
        registrations,
        feature_registrations,
    )
    .map_err(|error| match error {
        SessionRegistryInsertError::HandleSpaceExhausted => {
            RuntimeDynamicSessionError::SessionHandleSpaceExhausted
        }
        SessionRegistryInsertError::OwnerUnavailable(message) => {
            RuntimeDynamicSessionError::ModuleDiscovery { message }
        }
    })
}

fn parse_profile(
    profile: &[u8],
) -> Result<RuntimeDynamicSessionProfile, RuntimeDynamicSessionError> {
    RuntimeDynamicSessionProfile::from_bytes(profile).ok_or_else(|| {
        RuntimeDynamicSessionError::UnknownProfile {
            profile: String::from_utf8_lossy(profile).into_owned(),
        }
    })
}
