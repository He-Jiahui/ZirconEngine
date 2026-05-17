use std::ptr::NonNull;

use libloading::Library;
#[cfg(feature = "target-editor-host")]
use zircon_runtime_interface::runtime_api::ZrRuntimeProfileControlFnV1;
use zircon_runtime_interface::runtime_api::{
    ZrRuntimeCaptureFrameFnV1, ZrRuntimeCreateSessionFnV1, ZrRuntimeDestroySessionFnV1,
    ZrRuntimeDrainHostRequestsFnV1, ZrRuntimeHandleEventFnV1, ZrRuntimeTickFrameFnV1,
};
use zircon_runtime_interface::{
    ZrHostApiV1, ZrRuntimeApiV1, ZrRuntimeBindViewportSurfaceFnV1, ZrRuntimeGetApiFnV1,
    ZrRuntimePresentViewportFnV1, ZrRuntimeUnbindViewportSurfaceFnV1,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_GET_API_SYMBOL_V1,
};

use super::{default_runtime_library_path, RuntimeLibraryError};

pub(crate) struct LoadedRuntime {
    _library: Library,
    api: NonNull<ZrRuntimeApiV1>,
    size_bytes: usize,
}

impl LoadedRuntime {
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
                .get::<ZrRuntimeGetApiFnV1>(ZR_RUNTIME_GET_API_SYMBOL_V1)
                .map_err(|error| {
                    RuntimeLibraryError::new(format!(
                        "failed to resolve zircon runtime API symbol: {error}"
                    ))
                })?;
            get_api(&host)
        };
        let api = NonNull::new(api as *mut ZrRuntimeApiV1)
            .ok_or_else(|| RuntimeLibraryError::new("runtime library rejected host ABI version"))?;
        let size_bytes = validate_api(api)?;
        let loaded = Self {
            _library: library,
            api,
            size_bytes,
        };
        Ok(loaded)
    }

    pub(crate) fn create_session(&self) -> Option<ZrRuntimeCreateSessionFnV1> {
        self.api_function_field(core::mem::offset_of!(ZrRuntimeApiV1, create_session))
    }

    pub(crate) fn destroy_session(&self) -> Option<ZrRuntimeDestroySessionFnV1> {
        self.api_function_field(core::mem::offset_of!(ZrRuntimeApiV1, destroy_session))
    }

    pub(crate) fn handle_event(&self) -> Option<ZrRuntimeHandleEventFnV1> {
        self.api_function_field(core::mem::offset_of!(ZrRuntimeApiV1, handle_event))
    }

    pub(crate) fn capture_frame(&self) -> Option<ZrRuntimeCaptureFrameFnV1> {
        self.api_function_field(core::mem::offset_of!(ZrRuntimeApiV1, capture_frame))
    }

    pub(crate) fn bind_viewport_surface(&self) -> Option<ZrRuntimeBindViewportSurfaceFnV1> {
        self.api_function_field(core::mem::offset_of!(ZrRuntimeApiV1, bind_viewport_surface))
    }

    pub(crate) fn unbind_viewport_surface(&self) -> Option<ZrRuntimeUnbindViewportSurfaceFnV1> {
        self.api_function_field(core::mem::offset_of!(
            ZrRuntimeApiV1,
            unbind_viewport_surface
        ))
    }

    pub(crate) fn present_viewport(&self) -> Option<ZrRuntimePresentViewportFnV1> {
        self.api_function_field(core::mem::offset_of!(ZrRuntimeApiV1, present_viewport))
    }

    pub(crate) fn tick_frame(&self) -> Option<ZrRuntimeTickFrameFnV1> {
        self.api_function_field(core::mem::offset_of!(ZrRuntimeApiV1, tick_frame))
    }

    pub(crate) fn drain_host_requests(&self) -> Option<ZrRuntimeDrainHostRequestsFnV1> {
        self.api_function_field(core::mem::offset_of!(ZrRuntimeApiV1, drain_host_requests))
    }

    #[cfg(feature = "target-editor-host")]
    pub(crate) fn profile_control(&self) -> Option<ZrRuntimeProfileControlFnV1> {
        self.api_function_field(core::mem::offset_of!(ZrRuntimeApiV1, profile_control))
    }

    pub(crate) fn supports_viewport_surface_present(&self) -> bool {
        runtime_api_supports_viewport_surface_present(
            self.size_bytes,
            self.bind_viewport_surface(),
            self.unbind_viewport_surface(),
            self.present_viewport(),
        )
    }

    fn api_function_field<T: Copy>(&self, field_offset: usize) -> Option<T> {
        read_api_function_field(self.api, self.size_bytes, field_offset)
    }
}

fn validate_api(api: NonNull<ZrRuntimeApiV1>) -> Result<usize, RuntimeLibraryError> {
    let abi_version =
        read_api_field_unchecked::<u32>(api, core::mem::offset_of!(ZrRuntimeApiV1, abi_version));
    if abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
        return Err(RuntimeLibraryError::new(format!(
            "unsupported runtime ABI version {abi_version}"
        )));
    }

    let size_bytes =
        read_api_field_unchecked::<usize>(api, core::mem::offset_of!(ZrRuntimeApiV1, size_bytes));
    if !runtime_api_required_prefix_available(size_bytes) {
        return Err(RuntimeLibraryError::new(format!(
            "runtime API table is shorter than required v1 prefix: {size_bytes} bytes"
        )));
    }

    let create_session = read_api_function_field::<ZrRuntimeCreateSessionFnV1>(
        api,
        size_bytes,
        core::mem::offset_of!(ZrRuntimeApiV1, create_session),
    );
    let destroy_session = read_api_function_field::<ZrRuntimeDestroySessionFnV1>(
        api,
        size_bytes,
        core::mem::offset_of!(ZrRuntimeApiV1, destroy_session),
    );
    let handle_event = read_api_function_field::<ZrRuntimeHandleEventFnV1>(
        api,
        size_bytes,
        core::mem::offset_of!(ZrRuntimeApiV1, handle_event),
    );
    let capture_frame = read_api_function_field::<ZrRuntimeCaptureFrameFnV1>(
        api,
        size_bytes,
        core::mem::offset_of!(ZrRuntimeApiV1, capture_frame),
    );
    if create_session.is_none()
        || destroy_session.is_none()
        || handle_event.is_none()
        || capture_frame.is_none()
    {
        return Err(RuntimeLibraryError::new(
            "runtime API table is missing required functions",
        ));
    }
    Ok(size_bytes)
}

fn read_api_function_field<T: Copy>(
    api: NonNull<ZrRuntimeApiV1>,
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
    api: NonNull<ZrRuntimeApiV1>,
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

fn read_api_field_unchecked<T: Copy>(api: NonNull<ZrRuntimeApiV1>, field_offset: usize) -> T {
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

pub(super) const fn runtime_api_required_prefix_available(size_bytes: usize) -> bool {
    runtime_api_field_available(
        size_bytes,
        core::mem::offset_of!(ZrRuntimeApiV1, capture_frame),
        core::mem::size_of::<Option<ZrRuntimeCaptureFrameFnV1>>(),
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
        core::mem::offset_of!(ZrRuntimeApiV1, bind_viewport_surface),
        core::mem::size_of::<Option<ZrRuntimeBindViewportSurfaceFnV1>>(),
    ) && bind_viewport_surface.is_some()
        && runtime_api_field_available(
            size_bytes,
            core::mem::offset_of!(ZrRuntimeApiV1, unbind_viewport_surface),
            core::mem::size_of::<Option<ZrRuntimeUnbindViewportSurfaceFnV1>>(),
        )
        && unbind_viewport_surface.is_some()
        && runtime_api_field_available(
            size_bytes,
            core::mem::offset_of!(ZrRuntimeApiV1, present_viewport),
            core::mem::size_of::<Option<ZrRuntimePresentViewportFnV1>>(),
        )
        && present_viewport.is_some()
}
