use super::*;

impl RuntimeLibraryPreflight {
    pub(in crate::entry) fn linked_after_preflight(
        &self,
    ) -> Result<LoadedRuntime, RuntimeLibraryError> {
        // Scripted Play still requires the admitted editor staging identity.
        // Its actual linked module composition is checked by RuntimeSession.
        self.validate_after_preflight()?;
        let host = ZrHostApiV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);
        // SAFETY: host lives through this synchronous call; the returned table
        // belongs to the linked runtime and remains valid until process exit.
        let api = NonNull::new(unsafe {
            zircon_runtime::dynamic_api::zircon_runtime_get_api_v8(&host) as *mut ZrRuntimeApiV8
        })
        .ok_or_else(|| RuntimeLibraryError::new("linked runtime rejected the host ABI"))?;
        // SAFETY: the linked getter supplies the process-lifetime V8 table.
        let validated = unsafe { validate_v8_api(api) }?;
        Ok(LoadedRuntime {
            _library: None,
            _artifact_manifest: None,
            api,
            size_bytes: validated.size_bytes,
            required: validated.required,
            app_session_configuration: None,
            viewport_surface_binding_v2: None,
            runtime_instance: allocate_runtime_instance_id(&NEXT_RUNTIME_INSTANCE_ID)?,
        })
    }
}
