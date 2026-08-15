use std::path::Path;

use zircon_runtime_interface::ZrRuntimeSessionHandle;

use crate::plugin::RuntimePluginRegistrationReport;

use super::profile::RuntimeDynamicSessionProfile;
use super::project::RuntimeProjectConfig;
use super::registry::insert_session;
use super::{RuntimeDynamicSession, RuntimeDynamicSessionError};

pub fn create_linked_runtime_session(
    profile: &[u8],
    project_root: Option<&Path>,
    registrations: Vec<RuntimePluginRegistrationReport>,
) -> Result<ZrRuntimeSessionHandle, RuntimeDynamicSessionError> {
    let profile = RuntimeDynamicSessionProfile::from_bytes(profile).ok_or_else(|| {
        RuntimeDynamicSessionError::UnknownProfile {
            profile: String::from_utf8_lossy(profile).into_owned(),
        }
    })?;
    let project_config = project_root
        .map(RuntimeProjectConfig::from_root)
        .transpose()
        .map_err(|source| RuntimeDynamicSessionError::ProjectStep {
            step: "resolve linked runtime project root",
            source,
        })?;
    RuntimeDynamicSession::new_with_linked_plugins(profile, project_config, registrations)
        .map(insert_session)
}
