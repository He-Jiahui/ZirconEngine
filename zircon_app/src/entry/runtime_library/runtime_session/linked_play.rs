use super::*;
use crate::entry::runtime_library::RuntimeLibraryPreflight;
use zircon_runtime::plugin::RuntimePluginRegistrationReport;

impl RuntimeSession {
    pub(in crate::entry) fn create_linked_play(
        preflight: &RuntimeLibraryPreflight,
        project_root: &Path,
        scene: &RelPath,
        registrations: Vec<RuntimePluginRegistrationReport>,
    ) -> Result<Self, RuntimeSessionCreateFailure> {
        RuntimeSessionCreateFailure::ensure_available()?;
        let runtime = preflight.linked_after_preflight()?;
        let project_identity = project_root_for_abi(Some(project_root))?.map(Arc::<str>::from);
        let handle = zircon_runtime::dynamic_api::create_linked_runtime_play_session(
            b"runtime",
            project_root,
            scene,
            registrations,
        )
        .map_err(|error| RuntimeLibraryError::new(error.to_string()))?;
        // This owner uses the same linked table/registry that created the handle.
        // Receipt errors retain the original session for creator-thread cleanup.
        Self::adopt_handle(runtime, b"runtime", handle, project_identity, None)
    }
}
