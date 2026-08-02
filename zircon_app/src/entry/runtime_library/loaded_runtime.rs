use std::ptr::NonNull;

use libloading::Library;
use zircon_runtime_interface::runtime_api::ZrRuntimeProfileControlFnV1;
use zircon_runtime_interface::runtime_api::{
    ZrRuntimeCaptureFrameFnV1, ZrRuntimeCreateSessionFnV2, ZrRuntimeDestroySessionFnV1,
    ZrRuntimeDrainHostRequestsFnV1, ZrRuntimeDrainPluginEventsFnV1, ZrRuntimeHandleEventFnV1,
    ZrRuntimeHarvestOperationFnV1, ZrRuntimePollOperationFnV2, ZrRuntimeSubmitOperationFnV1,
    ZrRuntimeSubscribePluginEventFnV1, ZrRuntimeTickFrameFnV2, ZrRuntimeUnsubscribePluginEventFnV1,
};
use zircon_runtime_interface::{
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZIRCON_RUNTIME_API_VERSION_V3, ZR_RUNTIME_GET_API_SYMBOL_V3,
    ZrHostApiV1, ZrRuntimeApiV3, ZrRuntimeBindViewportSurfaceFnV1, ZrRuntimeGetApiFnV3,
    ZrRuntimePresentViewportFnV1, ZrRuntimeUnbindViewportSurfaceFnV1,
};

use super::{
    RuntimeLibraryError, RuntimeLibraryPathError, RuntimeLibraryPathSelection,
    default_runtime_library_path, runtime_library_environment_override_request,
};

pub(crate) struct LoadedRuntime {
    _library: Option<Library>,
    api: NonNull<ZrRuntimeApiV3>,
    size_bytes: usize,
    required: RequiredRuntimeApiV3,
}

#[derive(Clone, Copy)]
struct RequiredRuntimeApiV3 {
    create_session: ZrRuntimeCreateSessionFnV2,
    destroy_session: ZrRuntimeDestroySessionFnV1,
    handle_event: ZrRuntimeHandleEventFnV1,
    capture_frame: ZrRuntimeCaptureFrameFnV1,
    subscribe_plugin_event: ZrRuntimeSubscribePluginEventFnV1,
    unsubscribe_plugin_event: ZrRuntimeUnsubscribePluginEventFnV1,
    drain_plugin_events: ZrRuntimeDrainPluginEventsFnV1,
    submit_operation: ZrRuntimeSubmitOperationFnV1,
    poll_operation: ZrRuntimePollOperationFnV2,
    harvest_operation: ZrRuntimeHarvestOperationFnV1,
    tick_frame: ZrRuntimeTickFrameFnV2,
}

struct ValidatedRuntimeApiV3 {
    size_bytes: usize,
    required: RequiredRuntimeApiV3,
}

// The selected API table is immutable and remains valid either while
// `_library` is held or for the process lifetime for the linked runtime.
unsafe impl Send for LoadedRuntime {}
unsafe impl Sync for LoadedRuntime {}

impl LoadedRuntime {
    pub(crate) fn linked() -> Result<Self, RuntimeLibraryError> {
        let host = ZrHostApiV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);
        let api = unsafe { zircon_runtime::dynamic_api::zircon_runtime_get_api_v3(&host) };
        let api = NonNull::new(api as *mut ZrRuntimeApiV3)
            .ok_or_else(|| RuntimeLibraryError::new("linked runtime rejected host ABI version"))?;
        // The linked export returns a process-lifetime static V3 table.
        let validated = unsafe { validate_v3_api(api) }?;
        Ok(Self {
            _library: None,
            api,
            size_bytes: validated.size_bytes,
            required: validated.required,
        })
    }

    pub(crate) fn load_default() -> Result<Self, RuntimeLibraryError> {
        match default_runtime_library_path() {
            Ok(RuntimeLibraryPathSelection::EnvironmentOverride(path)) => {
                Self::load_for_request(&path, runtime_library_environment_override_request(&path))
            }
            Ok(RuntimeLibraryPathSelection::Default(path)) => Self::load(path),
            Err(RuntimeLibraryPathError::EnvironmentOverride(error)) => return Err(error),
            Err(RuntimeLibraryPathError::DefaultResolution(error)) => {
                return Err(runtime_library_startup_error_for_request(
                    "<runtime-library-default>",
                    error,
                ));
            }
        }
    }

    pub(crate) fn load(path: impl AsRef<std::path::Path>) -> Result<Self, RuntimeLibraryError> {
        let path = path.as_ref();
        Self::load_for_request(path, path.display().to_string())
    }

    fn load_for_request(
        path: &std::path::Path,
        requested_path: String,
    ) -> Result<Self, RuntimeLibraryError> {
        let library = unsafe { Library::new(path) }
            .map_err(|error| runtime_library_startup_error_for_request(&requested_path, error))?;
        let host = ZrHostApiV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);
        let api = unsafe {
            let get_api = library
                .get::<ZrRuntimeGetApiFnV3>(ZR_RUNTIME_GET_API_SYMBOL_V3)
                .map_err(|error| {
                    runtime_library_startup_error_for_request(
                        &requested_path,
                        format!("failed to resolve zircon runtime API V3 symbol: {error}"),
                    )
                })?;
            NonNull::new(get_api(&host) as *mut ZrRuntimeApiV3).ok_or_else(|| {
                runtime_library_startup_error_for_request(
                    &requested_path,
                    "runtime library rejected host ABI version",
                )
            })?
        };
        // `library` remains owned by LoadedRuntime for every table access.
        let validated = unsafe { validate_v3_api(api) }
            .map_err(|error| runtime_library_startup_error_for_request(&requested_path, error))?;
        Ok(Self {
            _library: Some(library),
            api,
            size_bytes: validated.size_bytes,
            required: validated.required,
        })
    }

    pub(crate) fn create_session(&self) -> ZrRuntimeCreateSessionFnV2 {
        self.required.create_session
    }

    pub(crate) fn destroy_session(&self) -> ZrRuntimeDestroySessionFnV1 {
        self.required.destroy_session
    }

    pub(crate) fn handle_event(&self) -> ZrRuntimeHandleEventFnV1 {
        self.required.handle_event
    }

    pub(crate) fn capture_frame(&self) -> ZrRuntimeCaptureFrameFnV1 {
        self.required.capture_frame
    }

    pub(crate) fn bind_viewport_surface(&self) -> Option<ZrRuntimeBindViewportSurfaceFnV1> {
        self.api_function_field(core::mem::offset_of!(ZrRuntimeApiV3, bind_viewport_surface))
    }

    pub(crate) fn unbind_viewport_surface(&self) -> Option<ZrRuntimeUnbindViewportSurfaceFnV1> {
        self.api_function_field(core::mem::offset_of!(
            ZrRuntimeApiV3,
            unbind_viewport_surface
        ))
    }

    pub(crate) fn present_viewport(&self) -> Option<ZrRuntimePresentViewportFnV1> {
        self.api_function_field(core::mem::offset_of!(ZrRuntimeApiV3, present_viewport))
    }

    pub(crate) fn tick_frame(&self) -> ZrRuntimeTickFrameFnV2 {
        self.required.tick_frame
    }

    pub(crate) fn drain_host_requests(&self) -> Option<ZrRuntimeDrainHostRequestsFnV1> {
        self.api_function_field(core::mem::offset_of!(ZrRuntimeApiV3, drain_host_requests))
    }

    pub(crate) fn subscribe_plugin_event(&self) -> ZrRuntimeSubscribePluginEventFnV1 {
        self.required.subscribe_plugin_event
    }

    pub(crate) fn unsubscribe_plugin_event(&self) -> ZrRuntimeUnsubscribePluginEventFnV1 {
        self.required.unsubscribe_plugin_event
    }

    pub(crate) fn drain_plugin_events(&self) -> ZrRuntimeDrainPluginEventsFnV1 {
        self.required.drain_plugin_events
    }

    pub(crate) fn submit_operation(&self) -> ZrRuntimeSubmitOperationFnV1 {
        self.required.submit_operation
    }

    pub(crate) fn poll_operation(&self) -> ZrRuntimePollOperationFnV2 {
        self.required.poll_operation
    }

    pub(crate) fn harvest_operation(&self) -> ZrRuntimeHarvestOperationFnV1 {
        self.required.harvest_operation
    }

    pub(crate) fn profile_control(&self) -> Option<ZrRuntimeProfileControlFnV1> {
        self.api_function_field(core::mem::offset_of!(ZrRuntimeApiV3, profile_control))
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
    pub(crate) fn editor_gateway_api_table(&self) -> ZrRuntimeApiV3 {
        let mut api = ZrRuntimeApiV3::empty();
        api.handle_event = Some(self.handle_event());
        api.capture_frame = Some(self.capture_frame());
        api.profile_control = self.profile_control();
        api.tick_frame = Some(self.tick_frame());
        api.subscribe_plugin_event = Some(self.subscribe_plugin_event());
        api.unsubscribe_plugin_event = Some(self.unsubscribe_plugin_event());
        api.drain_plugin_events = Some(self.drain_plugin_events());
        api.submit_operation = Some(self.submit_operation());
        api.poll_operation = Some(self.poll_operation());
        api.harvest_operation = Some(self.harvest_operation());
        api
    }

    fn api_function_field<T: Copy>(&self, field_offset: usize) -> Option<T> {
        // Construction validates the table, and `_library` keeps dynamic storage alive.
        unsafe { read_api_function_field(self.api, self.size_bytes, field_offset) }
    }
}

pub(super) fn runtime_library_startup_error_for_request(
    requested_path: impl std::fmt::Display,
    cause: impl std::fmt::Display,
) -> RuntimeLibraryError {
    RuntimeLibraryError::new(format!(
        "runtime startup diagnostic: component=runtime_library requested_path={} cause={} recovery=stage the runtime library beside the product executable or set ZIRCON_RUNTIME_LIBRARY to a compatible absolute path",
        requested_path, cause
    ))
}

/// Validates a runtime-owned API table pointer.
///
/// # Safety
///
/// A non-null, aligned `api` must remain readable as a `ZrRuntimeApiV3` for
/// the duration of this call.
pub(super) unsafe fn validate_runtime_api_pointer(
    api: *const ZrRuntimeApiV3,
) -> Result<usize, RuntimeLibraryError> {
    let api = NonNull::new(api as *mut ZrRuntimeApiV3)
        .ok_or_else(|| RuntimeLibraryError::new("runtime library rejected host ABI version"))?;
    Ok(unsafe { validate_v3_api(api) }?.size_bytes)
}

/// # Safety
///
/// An aligned `api` must point to a readable `ZrRuntimeApiV3` that remains
/// alive for the duration of this call. Misaligned pointers are rejected
/// before they are read.
unsafe fn validate_v3_api(
    api: NonNull<ZrRuntimeApiV3>,
) -> Result<ValidatedRuntimeApiV3, RuntimeLibraryError> {
    let required_alignment = core::mem::align_of::<ZrRuntimeApiV3>();
    if api.as_ptr() as usize % required_alignment != 0 {
        return Err(RuntimeLibraryError::new(format!(
            "runtime API table pointer is not aligned to {required_alignment} bytes"
        )));
    }

    let abi_version = unsafe {
        read_api_field_unchecked::<u32>(api, core::mem::offset_of!(ZrRuntimeApiV3, abi_version))
    };
    if abi_version != ZIRCON_RUNTIME_API_VERSION_V3 {
        return Err(RuntimeLibraryError::new(format!(
            "unsupported runtime API table version {abi_version}"
        )));
    }

    let size_bytes = unsafe {
        read_api_field_unchecked::<usize>(api, core::mem::offset_of!(ZrRuntimeApiV3, size_bytes))
    };
    if !runtime_api_required_layout_available(size_bytes) {
        return Err(RuntimeLibraryError::new(format!(
            "runtime API table is shorter than required v3 layout: {size_bytes} bytes"
        )));
    }
    let expected_size = core::mem::size_of::<ZrRuntimeApiV3>();
    if size_bytes != expected_size {
        return Err(RuntimeLibraryError::new(format!(
            "runtime API table size {size_bytes} does not match frozen v3 layout of {expected_size} bytes"
        )));
    }

    let required = RequiredRuntimeApiV3 {
        create_session: unsafe {
            required_api_function_field(
                api,
                size_bytes,
                core::mem::offset_of!(ZrRuntimeApiV3, create_session),
                "create_session",
            )
        }?,
        destroy_session: unsafe {
            required_api_function_field(
                api,
                size_bytes,
                core::mem::offset_of!(ZrRuntimeApiV3, destroy_session),
                "destroy_session",
            )
        }?,
        handle_event: unsafe {
            required_api_function_field(
                api,
                size_bytes,
                core::mem::offset_of!(ZrRuntimeApiV3, handle_event),
                "handle_event",
            )
        }?,
        capture_frame: unsafe {
            required_api_function_field(
                api,
                size_bytes,
                core::mem::offset_of!(ZrRuntimeApiV3, capture_frame),
                "capture_frame",
            )
        }?,
        subscribe_plugin_event: unsafe {
            required_api_function_field(
                api,
                size_bytes,
                core::mem::offset_of!(ZrRuntimeApiV3, subscribe_plugin_event),
                "subscribe_plugin_event",
            )
        }?,
        unsubscribe_plugin_event: unsafe {
            required_api_function_field(
                api,
                size_bytes,
                core::mem::offset_of!(ZrRuntimeApiV3, unsubscribe_plugin_event),
                "unsubscribe_plugin_event",
            )
        }?,
        drain_plugin_events: unsafe {
            required_api_function_field(
                api,
                size_bytes,
                core::mem::offset_of!(ZrRuntimeApiV3, drain_plugin_events),
                "drain_plugin_events",
            )
        }?,
        submit_operation: unsafe {
            required_api_function_field(
                api,
                size_bytes,
                core::mem::offset_of!(ZrRuntimeApiV3, submit_operation),
                "submit_operation",
            )
        }?,
        poll_operation: unsafe {
            required_api_function_field(
                api,
                size_bytes,
                core::mem::offset_of!(ZrRuntimeApiV3, poll_operation),
                "poll_operation",
            )
        }?,
        harvest_operation: unsafe {
            required_api_function_field(
                api,
                size_bytes,
                core::mem::offset_of!(ZrRuntimeApiV3, harvest_operation),
                "harvest_operation",
            )
        }?,
        tick_frame: unsafe {
            required_api_function_field(
                api,
                size_bytes,
                core::mem::offset_of!(ZrRuntimeApiV3, tick_frame),
                "tick_frame",
            )
        }?,
    };
    Ok(ValidatedRuntimeApiV3 {
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
        core::mem::offset_of!(ZrRuntimeApiV3, harvest_operation),
        core::mem::size_of::<Option<ZrRuntimeHarvestOperationFnV1>>(),
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
        core::mem::offset_of!(ZrRuntimeApiV3, bind_viewport_surface),
        core::mem::size_of::<Option<ZrRuntimeBindViewportSurfaceFnV1>>(),
    ) && bind_viewport_surface.is_some()
        && runtime_api_field_available(
            size_bytes,
            core::mem::offset_of!(ZrRuntimeApiV3, unbind_viewport_surface),
            core::mem::size_of::<Option<ZrRuntimeUnbindViewportSurfaceFnV1>>(),
        )
        && unbind_viewport_surface.is_some()
        && runtime_api_field_available(
            size_bytes,
            core::mem::offset_of!(ZrRuntimeApiV3, present_viewport),
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
