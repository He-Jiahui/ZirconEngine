use std::ptr::NonNull;

use libloading::Library;
use zircon_runtime_interface::{
    ZrHostApiV1, ZrRuntimeApiV1, ZrRuntimeBindViewportSurfaceFnV1, ZrRuntimeGetApiFnV1,
    ZrRuntimePresentViewportFnV1, ZrRuntimeUnbindViewportSurfaceFnV1,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_GET_API_SYMBOL_V1,
};

use super::{default_runtime_library_path, RuntimeLibraryError};

pub(crate) struct LoadedRuntime {
    _library: Library,
    api: NonNull<ZrRuntimeApiV1>,
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
        let loaded = Self {
            _library: library,
            api,
        };
        loaded.validate_api()?;
        Ok(loaded)
    }

    pub(crate) fn api(&self) -> &ZrRuntimeApiV1 {
        unsafe { self.api.as_ref() }
    }

    pub(crate) fn bind_viewport_surface(&self) -> Option<ZrRuntimeBindViewportSurfaceFnV1> {
        self.optional_api_field(
            core::mem::offset_of!(ZrRuntimeApiV1, bind_viewport_surface),
            core::mem::size_of::<Option<ZrRuntimeBindViewportSurfaceFnV1>>(),
            |api| api.bind_viewport_surface,
        )
    }

    pub(crate) fn unbind_viewport_surface(&self) -> Option<ZrRuntimeUnbindViewportSurfaceFnV1> {
        self.optional_api_field(
            core::mem::offset_of!(ZrRuntimeApiV1, unbind_viewport_surface),
            core::mem::size_of::<Option<ZrRuntimeUnbindViewportSurfaceFnV1>>(),
            |api| api.unbind_viewport_surface,
        )
    }

    pub(crate) fn present_viewport(&self) -> Option<ZrRuntimePresentViewportFnV1> {
        self.optional_api_field(
            core::mem::offset_of!(ZrRuntimeApiV1, present_viewport),
            core::mem::size_of::<Option<ZrRuntimePresentViewportFnV1>>(),
            |api| api.present_viewport,
        )
    }

    pub(crate) fn supports_viewport_surface_present(&self) -> bool {
        runtime_api_supports_viewport_surface_present(self.api())
    }

    fn optional_api_field<T: Copy>(
        &self,
        field_offset: usize,
        field_size: usize,
        read: impl FnOnce(&ZrRuntimeApiV1) -> Option<T>,
    ) -> Option<T> {
        let api = self.api();
        if runtime_api_field_available(api.size_bytes, field_offset, field_size) {
            read(api)
        } else {
            None
        }
    }

    fn validate_api(&self) -> Result<(), RuntimeLibraryError> {
        let api = self.api();
        if api.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
            return Err(RuntimeLibraryError::new(format!(
                "unsupported runtime ABI version {}",
                api.abi_version
            )));
        }
        if api.create_session.is_none()
            || api.destroy_session.is_none()
            || api.handle_event.is_none()
            || api.capture_frame.is_none()
        {
            return Err(RuntimeLibraryError::new(
                "runtime API table is missing required functions",
            ));
        }
        Ok(())
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

pub(super) fn runtime_api_supports_viewport_surface_present(api: &ZrRuntimeApiV1) -> bool {
    runtime_api_field_available(
        api.size_bytes,
        core::mem::offset_of!(ZrRuntimeApiV1, bind_viewport_surface),
        core::mem::size_of::<Option<ZrRuntimeBindViewportSurfaceFnV1>>(),
    ) && api.bind_viewport_surface.is_some()
        && runtime_api_field_available(
            api.size_bytes,
            core::mem::offset_of!(ZrRuntimeApiV1, unbind_viewport_surface),
            core::mem::size_of::<Option<ZrRuntimeUnbindViewportSurfaceFnV1>>(),
        )
        && api.unbind_viewport_surface.is_some()
        && runtime_api_field_available(
            api.size_bytes,
            core::mem::offset_of!(ZrRuntimeApiV1, present_viewport),
            core::mem::size_of::<Option<ZrRuntimePresentViewportFnV1>>(),
        )
        && api.present_viewport.is_some()
}
