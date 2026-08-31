use std::path::{Path, PathBuf};
use std::ptr::NonNull;

use libloading::Library;
use zircon_runtime_interface::runtime_api::ZrRuntimeProfileControlFnV2;
use zircon_runtime_interface::runtime_api::{
    ZrRuntimeCancelViewportPickFnV1, ZrRuntimeCaptureFrameFnV2, ZrRuntimeCreateSessionFnV3,
    ZrRuntimeDestroySessionFnV1, ZrRuntimeDrainHostRequestsFnV2, ZrRuntimeDrainPluginEventsFnV2,
    ZrRuntimeDrainWorldInvalidationsFnV2, ZrRuntimeHandleEventFnV1, ZrRuntimeHarvestOperationFnV2,
    ZrRuntimePollOperationFnV2, ZrRuntimePollViewportPickFnV1, ZrRuntimeQueryWorldFnV2,
    ZrRuntimeReleaseAllocationFnV2, ZrRuntimeRequestViewportPickFnV1,
    ZrRuntimeSubmitHighlightSetFnV1, ZrRuntimeSubmitOperationFnV1,
    ZrRuntimeSubscribePluginEventFnV1, ZrRuntimeTickFrameFnV2, ZrRuntimeUnsubscribePluginEventFnV1,
    ZrRuntimeUnwatchWorldFnV1, ZrRuntimeWatchWorldFnV1,
};
use zircon_runtime_interface::runtime_build_set::ZrRuntimeBuildSetId;
use zircon_runtime_interface::{
    ZrHostApiV1, ZrRuntimeApiV8, ZrRuntimeBindViewportSurfaceFnV1, ZrRuntimeGetApiFnV8,
    ZrRuntimePresentViewportFnV1, ZrRuntimeUnbindViewportSurfaceFnV1,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZIRCON_RUNTIME_API_VERSION_V8, ZR_RUNTIME_GET_API_SYMBOL_V8,
};

use super::{
    artifact_manifest::validate_runtime_library_artifact, default_runtime_library_path,
    RuntimeLibraryError, RuntimeLibraryPathError, RuntimeLibraryPathSelection,
};

pub(crate) struct LoadedRuntime {
    _library: Library,
    _artifact_manifest:
        Option<zircon_runtime_interface::runtime_build_set::ZrRuntimeArtifactManifestV1>,
    api: NonNull<ZrRuntimeApiV8>,
    size_bytes: usize,
    required: RequiredRuntimeApiV8,
}

/// A data-only validation receipt for the runtime artifact selected for a
/// project startup. It deliberately retains no loaded library handle.
#[derive(Clone, Debug)]
pub(crate) struct RuntimeLibraryPreflight {
    path: PathBuf,
    requested_path: String,
    build_set_id: ZrRuntimeBuildSetId,
}

#[derive(Clone, Copy)]
struct RequiredRuntimeApiV8 {
    create_session: ZrRuntimeCreateSessionFnV3,
    destroy_session: ZrRuntimeDestroySessionFnV1,
    handle_event: ZrRuntimeHandleEventFnV1,
    release_allocation: ZrRuntimeReleaseAllocationFnV2,
    capture_frame: ZrRuntimeCaptureFrameFnV2,
    submit_highlight_set: ZrRuntimeSubmitHighlightSetFnV1,
    subscribe_plugin_event: ZrRuntimeSubscribePluginEventFnV1,
    unsubscribe_plugin_event: ZrRuntimeUnsubscribePluginEventFnV1,
    drain_plugin_events: ZrRuntimeDrainPluginEventsFnV2,
    submit_operation: ZrRuntimeSubmitOperationFnV1,
    poll_operation: ZrRuntimePollOperationFnV2,
    harvest_operation: ZrRuntimeHarvestOperationFnV2,
    tick_frame: ZrRuntimeTickFrameFnV2,
    query_world: ZrRuntimeQueryWorldFnV2,
    watch_world: ZrRuntimeWatchWorldFnV1,
    unwatch_world: ZrRuntimeUnwatchWorldFnV1,
    drain_world_invalidations: ZrRuntimeDrainWorldInvalidationsFnV2,
    request_viewport_pick: ZrRuntimeRequestViewportPickFnV1,
    poll_viewport_pick: ZrRuntimePollViewportPickFnV1,
    cancel_viewport_pick: ZrRuntimeCancelViewportPickFnV1,
}

struct ValidatedRuntimeApiV8 {
    size_bytes: usize,
    required: RequiredRuntimeApiV8,
}

// The selected API table is immutable and remains valid while `_library` is held.
unsafe impl Send for LoadedRuntime {}
unsafe impl Sync for LoadedRuntime {}

impl LoadedRuntime {
    pub(crate) fn load_default() -> Result<Self, RuntimeLibraryError> {
        match default_runtime_library_path() {
            Ok(RuntimeLibraryPathSelection::EnvironmentOverride { path, request }) => {
                Self::load_for_request(&path, request)
            }
            Ok(RuntimeLibraryPathSelection::Default(path)) => {
                Self::load_for_request(&path, path.display().to_string())
            }
            Err(RuntimeLibraryPathError::EnvironmentOverride(error)) => return Err(error),
            Err(RuntimeLibraryPathError::DefaultResolution(error)) => {
                return Err(runtime_library_startup_error_for_request(
                    "<runtime-library-default>",
                    error,
                ));
            }
        }
    }

    /// Validate the selected runtime BuildSet before project materialization.
    ///
    /// This performs filesystem and sidecar validation only. Consumers must
    /// call [`RuntimeLibraryPreflight::load_after_preflight`] later; that
    /// method validates again immediately before `Library::new` so this
    /// receipt is never used to bypass a time-of-check/time-of-use boundary.
    pub(crate) fn preflight_default() -> Result<RuntimeLibraryPreflight, RuntimeLibraryError> {
        match default_runtime_library_path() {
            Ok(RuntimeLibraryPathSelection::EnvironmentOverride { path, request }) => {
                Self::preflight_for_request(&path, request)
            }
            Ok(RuntimeLibraryPathSelection::Default(path)) => {
                Self::preflight_for_request(&path, path.display().to_string())
            }
            Err(RuntimeLibraryPathError::EnvironmentOverride(error)) => Err(error),
            Err(RuntimeLibraryPathError::DefaultResolution(error)) => Err(
                runtime_library_startup_error_for_request("<runtime-library-default>", error),
            ),
        }
    }

    #[cfg(test)]
    pub(crate) fn load(path: impl AsRef<std::path::Path>) -> Result<Self, RuntimeLibraryError> {
        let path = path.as_ref();
        Self::load_dynamic_library(path, path.display().to_string(), None)
    }

    pub(super) fn preflight_for_request(
        path: &Path,
        requested_path: String,
    ) -> Result<RuntimeLibraryPreflight, RuntimeLibraryError> {
        let artifact_manifest = validate_runtime_library_artifact(path)
            .map_err(|error| runtime_library_startup_error_for_request(&requested_path, error))?;
        Ok(RuntimeLibraryPreflight {
            path: path.to_path_buf(),
            requested_path,
            build_set_id: artifact_manifest.build_set_id,
        })
    }

    pub(super) fn load_for_request(
        path: &Path,
        requested_path: String,
    ) -> Result<Self, RuntimeLibraryError> {
        let artifact_manifest = validate_runtime_library_artifact(path)
            .map_err(|error| runtime_library_startup_error_for_request(&requested_path, error))?;
        Self::load_dynamic_library(path, requested_path, Some(artifact_manifest))
    }

    fn load_dynamic_library(
        path: &Path,
        requested_path: String,
        artifact_manifest: Option<
            zircon_runtime_interface::runtime_build_set::ZrRuntimeArtifactManifestV1,
        >,
    ) -> Result<Self, RuntimeLibraryError> {
        let library = unsafe { Library::new(path) }
            .map_err(|error| runtime_library_startup_error_for_request(&requested_path, error))?;
        let host = ZrHostApiV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);
        let api = unsafe {
            let get_api = library
                .get::<ZrRuntimeGetApiFnV8>(ZR_RUNTIME_GET_API_SYMBOL_V8)
                .map_err(|error| {
                    runtime_library_startup_error_for_request(
                        &requested_path,
                        format!("failed to resolve zircon runtime API V8 symbol: {error}"),
                    )
                })?;
            NonNull::new(get_api(&host) as *mut ZrRuntimeApiV8).ok_or_else(|| {
                runtime_library_startup_error_for_request(
                    &requested_path,
                    "runtime library rejected host ABI version",
                )
            })?
        };
        // `library` remains owned by LoadedRuntime for every table access.
        let validated = unsafe { validate_v8_api(api) }
            .map_err(|error| runtime_library_startup_error_for_request(&requested_path, error))?;
        Ok(Self {
            _library: library,
            _artifact_manifest: artifact_manifest,
            api,
            size_bytes: validated.size_bytes,
            required: validated.required,
        })
    }

    pub(crate) fn create_session(&self) -> ZrRuntimeCreateSessionFnV3 {
        self.required.create_session
    }

    pub(crate) fn destroy_session(&self) -> ZrRuntimeDestroySessionFnV1 {
        self.required.destroy_session
    }

    pub(crate) fn handle_event(&self) -> ZrRuntimeHandleEventFnV1 {
        self.required.handle_event
    }

    pub(crate) fn release_allocation(&self) -> ZrRuntimeReleaseAllocationFnV2 {
        self.required.release_allocation
    }

    pub(crate) fn capture_frame(&self) -> ZrRuntimeCaptureFrameFnV2 {
        self.required.capture_frame
    }

    pub(crate) fn submit_highlight_set(&self) -> ZrRuntimeSubmitHighlightSetFnV1 {
        self.required.submit_highlight_set
    }

    pub(crate) fn request_viewport_pick(&self) -> ZrRuntimeRequestViewportPickFnV1 {
        self.required.request_viewport_pick
    }

    pub(crate) fn poll_viewport_pick(&self) -> ZrRuntimePollViewportPickFnV1 {
        self.required.poll_viewport_pick
    }

    pub(crate) fn cancel_viewport_pick(&self) -> ZrRuntimeCancelViewportPickFnV1 {
        self.required.cancel_viewport_pick
    }

    pub(crate) fn bind_viewport_surface(&self) -> Option<ZrRuntimeBindViewportSurfaceFnV1> {
        self.api_function_field(core::mem::offset_of!(ZrRuntimeApiV8, bind_viewport_surface))
    }

    pub(crate) fn unbind_viewport_surface(&self) -> Option<ZrRuntimeUnbindViewportSurfaceFnV1> {
        self.api_function_field(core::mem::offset_of!(
            ZrRuntimeApiV8,
            unbind_viewport_surface
        ))
    }

    pub(crate) fn present_viewport(&self) -> Option<ZrRuntimePresentViewportFnV1> {
        self.api_function_field(core::mem::offset_of!(ZrRuntimeApiV8, present_viewport))
    }

    pub(crate) fn tick_frame(&self) -> ZrRuntimeTickFrameFnV2 {
        self.required.tick_frame
    }

    pub(crate) fn drain_host_requests(&self) -> Option<ZrRuntimeDrainHostRequestsFnV2> {
        self.api_function_field(core::mem::offset_of!(ZrRuntimeApiV8, drain_host_requests))
    }

    pub(crate) fn subscribe_plugin_event(&self) -> ZrRuntimeSubscribePluginEventFnV1 {
        self.required.subscribe_plugin_event
    }

    pub(crate) fn unsubscribe_plugin_event(&self) -> ZrRuntimeUnsubscribePluginEventFnV1 {
        self.required.unsubscribe_plugin_event
    }

    pub(crate) fn drain_plugin_events(&self) -> ZrRuntimeDrainPluginEventsFnV2 {
        self.required.drain_plugin_events
    }

    pub(crate) fn submit_operation(&self) -> ZrRuntimeSubmitOperationFnV1 {
        self.required.submit_operation
    }

    pub(crate) fn poll_operation(&self) -> ZrRuntimePollOperationFnV2 {
        self.required.poll_operation
    }

    pub(crate) fn harvest_operation(&self) -> ZrRuntimeHarvestOperationFnV2 {
        self.required.harvest_operation
    }

    pub(crate) fn query_world(&self) -> ZrRuntimeQueryWorldFnV2 {
        self.required.query_world
    }

    pub(crate) fn watch_world(&self) -> ZrRuntimeWatchWorldFnV1 {
        self.required.watch_world
    }

    pub(crate) fn unwatch_world(&self) -> ZrRuntimeUnwatchWorldFnV1 {
        self.required.unwatch_world
    }

    pub(crate) fn drain_world_invalidations(&self) -> ZrRuntimeDrainWorldInvalidationsFnV2 {
        self.required.drain_world_invalidations
    }

    pub(crate) fn profile_control(&self) -> Option<ZrRuntimeProfileControlFnV2> {
        self.api_function_field(core::mem::offset_of!(ZrRuntimeApiV8, profile_control))
    }

    pub(crate) fn supports_viewport_surface_present(&self) -> bool {
        runtime_api_supports_viewport_surface_present(
            self.size_bytes,
            self.bind_viewport_surface(),
            self.unbind_viewport_surface(),
            self.present_viewport(),
        )
    }

    #[cfg(feature = "target-editor-host")]
    pub(crate) fn editor_gateway_api_table(&self) -> ZrRuntimeApiV8 {
        let mut api = ZrRuntimeApiV8::empty();
        api.release_allocation = Some(self.release_allocation());
        api.handle_event = Some(self.handle_event());
        api.capture_frame = Some(self.capture_frame());
        api.submit_highlight_set = Some(self.submit_highlight_set());
        api.request_viewport_pick = Some(self.request_viewport_pick());
        api.poll_viewport_pick = Some(self.poll_viewport_pick());
        api.cancel_viewport_pick = Some(self.cancel_viewport_pick());
        api.profile_control = self.profile_control();
        api.tick_frame = Some(self.tick_frame());
        api.subscribe_plugin_event = Some(self.subscribe_plugin_event());
        api.unsubscribe_plugin_event = Some(self.unsubscribe_plugin_event());
        api.drain_plugin_events = Some(self.drain_plugin_events());
        api.submit_operation = Some(self.submit_operation());
        api.poll_operation = Some(self.poll_operation());
        api.harvest_operation = Some(self.harvest_operation());
        api.query_world = Some(self.query_world());
        api.watch_world = Some(self.watch_world());
        api.unwatch_world = Some(self.unwatch_world());
        api.drain_world_invalidations = Some(self.drain_world_invalidations());
        api.bind_viewport_surface = self.bind_viewport_surface();
        api.unbind_viewport_surface = self.unbind_viewport_surface();
        api.present_viewport = self.present_viewport();
        api
    }

    fn api_function_field<T: Copy>(&self, field_offset: usize) -> Option<T> {
        // Construction validates the table, and `_library` keeps dynamic storage alive.
        unsafe { read_api_function_field(self.api, self.size_bytes, field_offset) }
    }
}

impl RuntimeLibraryPreflight {
    /// Returns the BuildSet authenticated by the data-only sidecar validation.
    pub(crate) fn build_set_id(&self) -> ZrRuntimeBuildSetId {
        self.build_set_id.clone()
    }

    /// Load the preflighted library only after validating its current sidecar
    /// and artifact digest again immediately before dynamic code can execute.
    pub(crate) fn load_after_preflight(&self) -> Result<LoadedRuntime, RuntimeLibraryError> {
        let artifact_manifest = validate_runtime_library_artifact(&self.path).map_err(|error| {
            runtime_library_startup_error_for_request(&self.requested_path, error)
        })?;
        if artifact_manifest.build_set_id != self.build_set_id {
            return Err(runtime_library_startup_error_for_request(
                &self.requested_path,
                format!(
                    "runtime BuildSet changed after preflight: expected {}, found {}",
                    self.build_set_id.as_str(),
                    artifact_manifest.build_set_id.as_str()
                ),
            ));
        }
        LoadedRuntime::load_dynamic_library(
            &self.path,
            self.requested_path.clone(),
            Some(artifact_manifest),
        )
    }
}

pub(super) fn runtime_library_startup_error_for_request(
    requested_path: impl std::fmt::Display,
    cause: impl std::fmt::Display,
) -> RuntimeLibraryError {
    RuntimeLibraryError::new(format!(
        "runtime startup diagnostic: component=runtime_library requested_path={} cause={} recovery=stage the runtime library beside the product executable or set ZIRCON_RUNTIME_LIBRARY to a compatible path relative to the product executable or an absolute path",
        requested_path, cause
    ))
}

/// Validates a runtime-owned API table pointer.
///
/// # Safety
///
/// A non-null, aligned `api` must remain readable as a `ZrRuntimeApiV8` for
/// the duration of this call.
pub(super) unsafe fn validate_runtime_api_pointer(
    api: *const ZrRuntimeApiV8,
) -> Result<usize, RuntimeLibraryError> {
    let api = NonNull::new(api as *mut ZrRuntimeApiV8)
        .ok_or_else(|| RuntimeLibraryError::new("runtime library rejected host ABI version"))?;
    Ok(unsafe { validate_v8_api(api) }?.size_bytes)
}

/// # Safety
///
/// An aligned `api` must point to a readable `ZrRuntimeApiV8` that remains
/// alive for the duration of this call. Misaligned pointers are rejected
/// before they are read.
unsafe fn validate_v8_api(
    api: NonNull<ZrRuntimeApiV8>,
) -> Result<ValidatedRuntimeApiV8, RuntimeLibraryError> {
    let required_alignment = core::mem::align_of::<ZrRuntimeApiV8>();
    if api.as_ptr() as usize % required_alignment != 0 {
        return Err(RuntimeLibraryError::new(format!(
            "runtime API table pointer is not aligned to {required_alignment} bytes"
        )));
    }

    let abi_version = unsafe {
        read_api_field_unchecked::<u32>(api, core::mem::offset_of!(ZrRuntimeApiV8, abi_version))
    };
    if abi_version != ZIRCON_RUNTIME_API_VERSION_V8 {
        return Err(RuntimeLibraryError::new(format!(
            "unsupported runtime API table version {abi_version}"
        )));
    }

    let size_bytes = unsafe {
        read_api_field_unchecked::<usize>(api, core::mem::offset_of!(ZrRuntimeApiV8, size_bytes))
    };
    if !runtime_api_required_layout_available(size_bytes) {
        return Err(RuntimeLibraryError::new(format!(
            "runtime API table is shorter than required v8 layout: {size_bytes} bytes"
        )));
    }
    let expected_size = core::mem::size_of::<ZrRuntimeApiV8>();
    if size_bytes != expected_size {
        return Err(RuntimeLibraryError::new(format!(
            "runtime API table size {size_bytes} does not match frozen v8 layout of {expected_size} bytes"
        )));
    }

    let required = RequiredRuntimeApiV8 {
        create_session: unsafe {
            required_api_function_field(
                api,
                size_bytes,
                core::mem::offset_of!(ZrRuntimeApiV8, create_session),
                "create_session",
            )
        }?,
        destroy_session: unsafe {
            required_api_function_field(
                api,
                size_bytes,
                core::mem::offset_of!(ZrRuntimeApiV8, destroy_session),
                "destroy_session",
            )
        }?,
        release_allocation: unsafe {
            required_api_function_field(
                api,
                size_bytes,
                core::mem::offset_of!(ZrRuntimeApiV8, release_allocation),
                "release_allocation",
            )
        }?,
        handle_event: unsafe {
            required_api_function_field(
                api,
                size_bytes,
                core::mem::offset_of!(ZrRuntimeApiV8, handle_event),
                "handle_event",
            )
        }?,
        capture_frame: unsafe {
            required_api_function_field(
                api,
                size_bytes,
                core::mem::offset_of!(ZrRuntimeApiV8, capture_frame),
                "capture_frame",
            )
        }?,
        submit_highlight_set: unsafe {
            required_api_function_field(
                api,
                size_bytes,
                core::mem::offset_of!(ZrRuntimeApiV8, submit_highlight_set),
                "submit_highlight_set",
            )
        }?,
        subscribe_plugin_event: unsafe {
            required_api_function_field(
                api,
                size_bytes,
                core::mem::offset_of!(ZrRuntimeApiV8, subscribe_plugin_event),
                "subscribe_plugin_event",
            )
        }?,
        unsubscribe_plugin_event: unsafe {
            required_api_function_field(
                api,
                size_bytes,
                core::mem::offset_of!(ZrRuntimeApiV8, unsubscribe_plugin_event),
                "unsubscribe_plugin_event",
            )
        }?,
        drain_plugin_events: unsafe {
            required_api_function_field(
                api,
                size_bytes,
                core::mem::offset_of!(ZrRuntimeApiV8, drain_plugin_events),
                "drain_plugin_events",
            )
        }?,
        submit_operation: unsafe {
            required_api_function_field(
                api,
                size_bytes,
                core::mem::offset_of!(ZrRuntimeApiV8, submit_operation),
                "submit_operation",
            )
        }?,
        poll_operation: unsafe {
            required_api_function_field(
                api,
                size_bytes,
                core::mem::offset_of!(ZrRuntimeApiV8, poll_operation),
                "poll_operation",
            )
        }?,
        harvest_operation: unsafe {
            required_api_function_field(
                api,
                size_bytes,
                core::mem::offset_of!(ZrRuntimeApiV8, harvest_operation),
                "harvest_operation",
            )
        }?,
        tick_frame: unsafe {
            required_api_function_field(
                api,
                size_bytes,
                core::mem::offset_of!(ZrRuntimeApiV8, tick_frame),
                "tick_frame",
            )
        }?,
        query_world: unsafe {
            required_api_function_field(
                api,
                size_bytes,
                core::mem::offset_of!(ZrRuntimeApiV8, query_world),
                "query_world",
            )
        }?,
        watch_world: unsafe {
            required_api_function_field(
                api,
                size_bytes,
                core::mem::offset_of!(ZrRuntimeApiV8, watch_world),
                "watch_world",
            )
        }?,
        unwatch_world: unsafe {
            required_api_function_field(
                api,
                size_bytes,
                core::mem::offset_of!(ZrRuntimeApiV8, unwatch_world),
                "unwatch_world",
            )
        }?,
        drain_world_invalidations: unsafe {
            required_api_function_field(
                api,
                size_bytes,
                core::mem::offset_of!(ZrRuntimeApiV8, drain_world_invalidations),
                "drain_world_invalidations",
            )
        }?,
        request_viewport_pick: unsafe {
            required_api_function_field(
                api,
                size_bytes,
                core::mem::offset_of!(ZrRuntimeApiV8, request_viewport_pick),
                "request_viewport_pick",
            )
        }?,
        poll_viewport_pick: unsafe {
            required_api_function_field(
                api,
                size_bytes,
                core::mem::offset_of!(ZrRuntimeApiV8, poll_viewport_pick),
                "poll_viewport_pick",
            )
        }?,
        cancel_viewport_pick: unsafe {
            required_api_function_field(
                api,
                size_bytes,
                core::mem::offset_of!(ZrRuntimeApiV8, cancel_viewport_pick),
                "cancel_viewport_pick",
            )
        }?,
    };
    Ok(ValidatedRuntimeApiV8 {
        size_bytes,
        required,
    })
}

unsafe fn required_api_function_field<T: Copy>(
    api: NonNull<impl Sized>,
    size_bytes: usize,
    field_offset: usize,
    field_name: &'static str,
) -> Result<T, RuntimeLibraryError> {
    unsafe { read_api_function_field(api, size_bytes, field_offset) }.ok_or_else(|| {
        RuntimeLibraryError::new(format!(
            "runtime API table is missing required function `{field_name}`"
        ))
    })
}

unsafe fn read_api_function_field<T: Copy>(
    api: NonNull<impl Sized>,
    size_bytes: usize,
    field_offset: usize,
) -> Option<T> {
    unsafe {
        read_api_field_sized::<Option<T>>(
            api,
            size_bytes,
            field_offset,
            core::mem::size_of::<Option<T>>(),
        )
    }
    .flatten()
}

unsafe fn read_api_field_sized<T: Copy>(
    api: NonNull<impl Sized>,
    size_bytes: usize,
    field_offset: usize,
    field_size: usize,
) -> Option<T> {
    if runtime_api_field_available(size_bytes, field_offset, field_size) {
        Some(unsafe { read_api_field_unchecked(api, field_offset) })
    } else {
        None
    }
}

unsafe fn read_api_field_unchecked<T: Copy>(api: NonNull<impl Sized>, field_offset: usize) -> T {
    // Callers either read the fixed ABI header or prove the advertised table covers this field.
    unsafe {
        api.as_ptr()
            .cast::<u8>()
            .add(field_offset)
            .cast::<T>()
            .read()
    }
}

pub(super) const fn runtime_api_field_available(
    size_bytes: usize,
    field_offset: usize,
    field_size: usize,
) -> bool {
    match field_offset.checked_add(field_size) {
        Some(required_size) => size_bytes >= required_size,
        None => false,
    }
}

pub(super) const fn runtime_api_required_layout_available(size_bytes: usize) -> bool {
    runtime_api_field_available(
        size_bytes,
        core::mem::offset_of!(ZrRuntimeApiV8, cancel_viewport_pick),
        core::mem::size_of::<Option<ZrRuntimeCancelViewportPickFnV1>>(),
    )
}

pub(super) fn runtime_api_supports_viewport_surface_present(
    size_bytes: usize,
    bind_viewport_surface: Option<ZrRuntimeBindViewportSurfaceFnV1>,
    unbind_viewport_surface: Option<ZrRuntimeUnbindViewportSurfaceFnV1>,
    present_viewport: Option<ZrRuntimePresentViewportFnV1>,
) -> bool {
    runtime_api_field_available(
        size_bytes,
        core::mem::offset_of!(ZrRuntimeApiV8, bind_viewport_surface),
        core::mem::size_of::<Option<ZrRuntimeBindViewportSurfaceFnV1>>(),
    ) && bind_viewport_surface.is_some()
        && runtime_api_field_available(
            size_bytes,
            core::mem::offset_of!(ZrRuntimeApiV8, unbind_viewport_surface),
            core::mem::size_of::<Option<ZrRuntimeUnbindViewportSurfaceFnV1>>(),
        )
        && unbind_viewport_surface.is_some()
        && runtime_api_field_available(
            size_bytes,
            core::mem::offset_of!(ZrRuntimeApiV8, present_viewport),
            core::mem::size_of::<Option<ZrRuntimePresentViewportFnV1>>(),
        )
        && present_viewport.is_some()
}

#[cfg(test)]
mod tests {
    use super::LoadedRuntime;
    use crate::entry::runtime_library::runtime_library_environment_override_request;

    #[test]
    fn environment_override_load_failure_keeps_the_override_request_provenance() {
        let path = std::env::temp_dir().join(format!(
            "zircon_missing_runtime_override_{}_{}.dll",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time after Unix epoch")
                .as_nanos()
        ));
        let request = runtime_library_environment_override_request(&path);
        let error = match LoadedRuntime::load_for_request(&path, request.clone()) {
            Ok(_) => panic!("a nonexistent environment override must fail to load"),
            Err(error) => error,
        };
        let diagnostic = error.to_string();

        assert!(diagnostic.contains(&format!("requested_path={request}")));
        assert!(diagnostic.contains("cause="));
        assert!(diagnostic.contains("recovery=stage the runtime library"));
    }
}
