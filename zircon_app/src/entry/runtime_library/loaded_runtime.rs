use std::ptr::NonNull;

use libloading::Library;
#[cfg(feature = "target-editor-host")]
use zircon_runtime_interface::runtime_api::ZrRuntimeProfileControlFnV1;
use zircon_runtime_interface::runtime_api::{
    ZrRuntimeCaptureFrameFnV1, ZrRuntimeCreateSessionFnV1, ZrRuntimeDestroySessionFnV1,
    ZrRuntimeDrainHostRequestsFnV1, ZrRuntimeDrainPluginEventsFnV1, ZrRuntimeHandleEventFnV1,
    ZrRuntimeHarvestOperationFnV1, ZrRuntimePollOperationFnV1, ZrRuntimeSubmitOperationFnV1,
    ZrRuntimeSubscribePluginEventFnV1, ZrRuntimeTickFrameFnV1, ZrRuntimeUnsubscribePluginEventFnV1,
};
use zircon_runtime_interface::{
    ZrHostApiV1, ZrRuntimeApiV2, ZrRuntimeBindViewportSurfaceFnV1, ZrRuntimeGetApiFnV2,
    ZrRuntimePresentViewportFnV1, ZrRuntimeUnbindViewportSurfaceFnV1,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZIRCON_RUNTIME_API_VERSION_V2, ZR_RUNTIME_GET_API_SYMBOL_V2,
};

use super::{default_runtime_library_path, RuntimeLibraryError};

pub(crate) struct LoadedRuntime {
    _library: Option<Library>,
    api: NonNull<ZrRuntimeApiV2>,
    size_bytes: usize,
    required: RequiredRuntimeApiV2,
}

#[derive(Clone, Copy)]
struct RequiredRuntimeApiV2 {
    create_session: ZrRuntimeCreateSessionFnV1,
    destroy_session: ZrRuntimeDestroySessionFnV1,
    handle_event: ZrRuntimeHandleEventFnV1,
    capture_frame: ZrRuntimeCaptureFrameFnV1,
    subscribe_plugin_event: ZrRuntimeSubscribePluginEventFnV1,
    unsubscribe_plugin_event: ZrRuntimeUnsubscribePluginEventFnV1,
    drain_plugin_events: ZrRuntimeDrainPluginEventsFnV1,
    submit_operation: ZrRuntimeSubmitOperationFnV1,
    poll_operation: ZrRuntimePollOperationFnV1,
    harvest_operation: ZrRuntimeHarvestOperationFnV1,
}

struct ValidatedRuntimeApiV2 {
    size_bytes: usize,
    required: RequiredRuntimeApiV2,
}

// The selected API table is immutable and remains valid either while
// `_library` is held or for the process lifetime for the linked runtime.
unsafe impl Send for LoadedRuntime {}
unsafe impl Sync for LoadedRuntime {}

impl LoadedRuntime {
    pub(crate) fn linked() -> Result<Self, RuntimeLibraryError> {
        let host = ZrHostApiV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);
        let api = unsafe { zircon_runtime::dynamic_api::zircon_runtime_get_api_v2(&host) };
        let api = NonNull::new(api as *mut ZrRuntimeApiV2)
            .ok_or_else(|| RuntimeLibraryError::new("linked runtime rejected host ABI version"))?;
        let validated = validate_v2_api(api)?;
        Ok(Self {
            _library: None,
            api,
            size_bytes: validated.size_bytes,
            required: validated.required,
        })
    }

    pub(crate) fn load_default() -> Result<Self, RuntimeLibraryError> {
        let path = default_runtime_library_path()?;
        Self::load(path)
    }

    pub(crate) fn load(path: impl AsRef<std::path::Path>) -> Result<Self, RuntimeLibraryError> {
        let path = path.as_ref();
        let library = unsafe { Library::new(path) }.map_err(|error| {
            RuntimeLibraryError::new(format!(
                "failed to load runtime library {}: {error}",
                path.display()
            ))
        })?;
        let host = ZrHostApiV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);
        let api = unsafe {
            let get_api = library
                .get::<ZrRuntimeGetApiFnV2>(ZR_RUNTIME_GET_API_SYMBOL_V2)
                .map_err(|error| {
                    RuntimeLibraryError::new(format!(
                        "failed to resolve zircon runtime API V2 symbol: {error}"
                    ))
                })?;
            NonNull::new(get_api(&host) as *mut ZrRuntimeApiV2).ok_or_else(|| {
                RuntimeLibraryError::new("runtime library rejected host ABI version")
            })?
        };
        let validated = validate_v2_api(api)?;
        Ok(Self {
            _library: Some(library),
            api,
            size_bytes: validated.size_bytes,
            required: validated.required,
        })
    }

    pub(crate) fn create_session(&self) -> ZrRuntimeCreateSessionFnV1 {
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
        self.api_function_field(core::mem::offset_of!(ZrRuntimeApiV2, bind_viewport_surface))
    }

    pub(crate) fn unbind_viewport_surface(&self) -> Option<ZrRuntimeUnbindViewportSurfaceFnV1> {
        self.api_function_field(core::mem::offset_of!(
            ZrRuntimeApiV2,
            unbind_viewport_surface
        ))
    }

    pub(crate) fn present_viewport(&self) -> Option<ZrRuntimePresentViewportFnV1> {
        self.api_function_field(core::mem::offset_of!(ZrRuntimeApiV2, present_viewport))
    }

    pub(crate) fn tick_frame(&self) -> Option<ZrRuntimeTickFrameFnV1> {
        self.api_function_field(core::mem::offset_of!(ZrRuntimeApiV2, tick_frame))
    }

    pub(crate) fn drain_host_requests(&self) -> Option<ZrRuntimeDrainHostRequestsFnV1> {
        self.api_function_field(core::mem::offset_of!(ZrRuntimeApiV2, drain_host_requests))
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

    pub(crate) fn poll_operation(&self) -> ZrRuntimePollOperationFnV1 {
        self.required.poll_operation
    }

    pub(crate) fn harvest_operation(&self) -> ZrRuntimeHarvestOperationFnV1 {
        self.required.harvest_operation
    }

    #[cfg(feature = "target-editor-host")]
    pub(crate) fn profile_control(&self) -> Option<ZrRuntimeProfileControlFnV1> {
        self.api_function_field(core::mem::offset_of!(ZrRuntimeApiV2, profile_control))
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
    pub(crate) fn editor_gateway_api_table(&self) -> ZrRuntimeApiV2 {
        let mut api = ZrRuntimeApiV2::empty();
        api.handle_event = Some(self.handle_event());
        api.capture_frame = Some(self.capture_frame());
        api.bind_viewport_surface = self.bind_viewport_surface();
        api.unbind_viewport_surface = self.unbind_viewport_surface();
        api.present_viewport = self.present_viewport();
        api.profile_control = self.profile_control();
        api.tick_frame = self.tick_frame();
        api.drain_host_requests = self.drain_host_requests();
        api.subscribe_plugin_event = Some(self.subscribe_plugin_event());
        api.unsubscribe_plugin_event = Some(self.unsubscribe_plugin_event());
        api.drain_plugin_events = Some(self.drain_plugin_events());
        api.submit_operation = Some(self.submit_operation());
        api.poll_operation = Some(self.poll_operation());
        api.harvest_operation = Some(self.harvest_operation());
        api
    }

    fn api_function_field<T: Copy>(&self, field_offset: usize) -> Option<T> {
        read_api_function_field(self.api, self.size_bytes, field_offset)
    }
}

pub(super) fn validate_runtime_api_pointer(
    api: *const ZrRuntimeApiV2,
) -> Result<usize, RuntimeLibraryError> {
    let api = NonNull::new(api as *mut ZrRuntimeApiV2)
        .ok_or_else(|| RuntimeLibraryError::new("runtime library rejected host ABI version"))?;
    Ok(validate_v2_api(api)?.size_bytes)
}

fn validate_v2_api(
    api: NonNull<ZrRuntimeApiV2>,
) -> Result<ValidatedRuntimeApiV2, RuntimeLibraryError> {
    let abi_version =
        read_api_field_unchecked::<u32>(api, core::mem::offset_of!(ZrRuntimeApiV2, abi_version));
    if abi_version != ZIRCON_RUNTIME_API_VERSION_V2 {
        return Err(RuntimeLibraryError::new(format!(
            "unsupported runtime API table version {abi_version}"
        )));
    }

    let size_bytes =
        read_api_field_unchecked::<usize>(api, core::mem::offset_of!(ZrRuntimeApiV2, size_bytes));
    if !runtime_api_required_layout_available(size_bytes) {
        return Err(RuntimeLibraryError::new(format!(
            "runtime API table is shorter than required v2 layout: {size_bytes} bytes"
        )));
    }

    let required = RequiredRuntimeApiV2 {
        create_session: required_api_function_field(
            api,
            size_bytes,
            core::mem::offset_of!(ZrRuntimeApiV2, create_session),
        )?,
        destroy_session: required_api_function_field(
            api,
            size_bytes,
            core::mem::offset_of!(ZrRuntimeApiV2, destroy_session),
        )?,
        handle_event: required_api_function_field(
            api,
            size_bytes,
            core::mem::offset_of!(ZrRuntimeApiV2, handle_event),
        )?,
        capture_frame: required_api_function_field(
            api,
            size_bytes,
            core::mem::offset_of!(ZrRuntimeApiV2, capture_frame),
        )?,
        subscribe_plugin_event: required_api_function_field(
            api,
            size_bytes,
            core::mem::offset_of!(ZrRuntimeApiV2, subscribe_plugin_event),
        )?,
        unsubscribe_plugin_event: required_api_function_field(
            api,
            size_bytes,
            core::mem::offset_of!(ZrRuntimeApiV2, unsubscribe_plugin_event),
        )?,
        drain_plugin_events: required_api_function_field(
            api,
            size_bytes,
            core::mem::offset_of!(ZrRuntimeApiV2, drain_plugin_events),
        )?,
        submit_operation: required_api_function_field(
            api,
            size_bytes,
            core::mem::offset_of!(ZrRuntimeApiV2, submit_operation),
        )?,
        poll_operation: required_api_function_field(
            api,
            size_bytes,
            core::mem::offset_of!(ZrRuntimeApiV2, poll_operation),
        )?,
        harvest_operation: required_api_function_field(
            api,
            size_bytes,
            core::mem::offset_of!(ZrRuntimeApiV2, harvest_operation),
        )?,
    };
    Ok(ValidatedRuntimeApiV2 {
        size_bytes,
        required,
    })
}

fn required_api_function_field<T: Copy>(
    api: NonNull<impl Sized>,
    size_bytes: usize,
    field_offset: usize,
) -> Result<T, RuntimeLibraryError> {
    read_api_function_field(api, size_bytes, field_offset)
        .ok_or_else(|| RuntimeLibraryError::new("runtime API table is missing required functions"))
}

fn read_api_function_field<T: Copy>(
    api: NonNull<impl Sized>,
    size_bytes: usize,
    field_offset: usize,
) -> Option<T> {
    read_api_field_sized::<Option<T>>(
        api,
        size_bytes,
        field_offset,
        core::mem::size_of::<Option<T>>(),
    )
    .flatten()
}

fn read_api_field_sized<T: Copy>(
    api: NonNull<impl Sized>,
    size_bytes: usize,
    field_offset: usize,
    field_size: usize,
) -> Option<T> {
    if runtime_api_field_available(size_bytes, field_offset, field_size) {
        Some(read_api_field_unchecked(api, field_offset))
    } else {
        None
    }
}

fn read_api_field_unchecked<T: Copy>(api: NonNull<impl Sized>, field_offset: usize) -> T {
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
        core::mem::offset_of!(ZrRuntimeApiV2, harvest_operation),
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
        core::mem::offset_of!(ZrRuntimeApiV2, bind_viewport_surface),
        core::mem::size_of::<Option<ZrRuntimeBindViewportSurfaceFnV1>>(),
    ) && bind_viewport_surface.is_some()
        && runtime_api_field_available(
            size_bytes,
            core::mem::offset_of!(ZrRuntimeApiV2, unbind_viewport_surface),
            core::mem::size_of::<Option<ZrRuntimeUnbindViewportSurfaceFnV1>>(),
        )
        && unbind_viewport_surface.is_some()
        && runtime_api_field_available(
            size_bytes,
            core::mem::offset_of!(ZrRuntimeApiV2, present_viewport),
            core::mem::size_of::<Option<ZrRuntimePresentViewportFnV1>>(),
        )
        && present_viewport.is_some()
}
