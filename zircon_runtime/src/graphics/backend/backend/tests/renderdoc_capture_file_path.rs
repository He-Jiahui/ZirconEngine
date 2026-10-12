use std::path::Path;

#[cfg(windows)]
use std::sync::{Mutex, OnceLock};

use crate::graphics::types::GraphicsError;

/// Configures an already injected RenderDoc instance before WGPU creates a device.
///
/// This is test-only because normal runtime capture may use the debugger's own output policy.
/// Returning `false` means no RenderDoc module was injected into the current process.
pub(crate) fn configure_renderdoc_capture_file_path_template(
    path_template: &Path,
) -> Result<bool, GraphicsError> {
    configure_if_renderdoc_is_injected(path_template)
}

#[cfg(not(windows))]
fn configure_if_renderdoc_is_injected(_path_template: &Path) -> Result<bool, GraphicsError> {
    Ok(false)
}

#[cfg(windows)]
fn configure_if_renderdoc_is_injected(path_template: &Path) -> Result<bool, GraphicsError> {
    use std::ffi::{c_char, c_void, CString};

    use libloading::os::windows::Library;

    // These external enum/layout values are definition-bound to renderdoc_app.h, not renderer
    // policy, so they intentionally remain private to this ABI adapter.
    const RENDERDOC_API_VERSION_1_0_0: i32 = 10_000;
    const SET_CAPTURE_FILE_PATH_TEMPLATE_INDEX: usize = 11;

    type RenderdocGetApi = unsafe extern "C" fn(i32, *mut *mut c_void) -> i32;
    type RenderdocSetCaptureFilePathTemplate = unsafe extern "C" fn(*const c_char);
    type RenderdocApiFunction = unsafe extern "C" fn();

    #[repr(C)]
    struct RenderdocApiV1 {
        // RenderDoc keeps the API table append-only. The capture-path setter is the twelfth
        // function pointer in every compatible v1 table.
        _before_capture_file_path_template:
            [RenderdocApiFunction; SET_CAPTURE_FILE_PATH_TEMPLATE_INDEX],
        set_capture_file_path_template: RenderdocSetCaptureFilePathTemplate,
    }

    let path_template = path_template.to_str().ok_or_else(|| {
        GraphicsError::GraphicsDebugger("RenderDoc capture template must be UTF-8".to_owned())
    })?;
    let path_template = CString::new(path_template).map_err(|_| {
        GraphicsError::GraphicsDebugger(
            "RenderDoc capture template must not contain an interior NUL".to_owned(),
        )
    })?;
    let configuration_lock = renderdoc_api_configuration_lock();
    let _configuration_guard = configuration_lock
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Ok(library) = Library::open_already_loaded("renderdoc.dll") else {
        return Ok(false);
    };
    let Ok(get_api) = (unsafe { library.get::<RenderdocGetApi>(b"RENDERDOC_GetAPI\0") }) else {
        return Ok(false);
    };
    let mut api_pointer = std::ptr::null_mut::<c_void>();
    let api_available = unsafe { get_api(RENDERDOC_API_VERSION_1_0_0, &mut api_pointer) } == 1;
    if !api_available || api_pointer.is_null() {
        return Ok(false);
    }

    let api = unsafe { &*api_pointer.cast::<RenderdocApiV1>() };
    unsafe { (api.set_capture_file_path_template)(path_template.as_ptr()) };
    Ok(true)
}

#[cfg(windows)]
fn renderdoc_api_configuration_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}
