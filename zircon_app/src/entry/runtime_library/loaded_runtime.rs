use std::ptr::NonNull;

use libloading::Library;
use zircon_runtime_interface::{
    ZrHostApiV1, ZrRuntimeApiV1, ZrRuntimeGetApiFnV1, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZR_RUNTIME_GET_API_SYMBOL_V1,
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
