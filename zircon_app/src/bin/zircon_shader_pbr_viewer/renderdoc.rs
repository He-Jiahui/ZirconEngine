use std::error::Error;
use std::ffi::{CStr, CString, c_char, c_void};
use std::path::{Path, PathBuf};

use libloading::{Library, Symbol};

#[repr(C)]
struct RenderDocApi141 {
    // `RENDERDOC_API_1_4_1` exposes SetCaptureFilePathTemplate as its twelfth pointer-sized
    // entry. The viewer needs only that stable public ABI prefix, avoiding a new direct crate
    // dependency solely for a diagnostic tool integration.
    _functions_before_capture_template: [usize; 11],
    set_capture_file_path_template: Option<unsafe extern "C" fn(*const c_char)>,
    _get_capture_file_path_template: usize,
    get_num_captures: Option<unsafe extern "C" fn() -> u32>,
    get_capture: Option<unsafe extern "C" fn(u32, *mut c_char, *mut u32, *mut u64) -> u32>,
}

pub(crate) struct RenderDocBridge {
    library: Library,
}

pub(crate) struct RenderDocCaptureReport {
    capture_count: u32,
    latest_capture_path: Option<PathBuf>,
}

impl RenderDocCaptureReport {
    pub(crate) const fn capture_count(&self) -> u32 {
        self.capture_count
    }

    pub(crate) fn latest_capture_path(&self) -> Option<&Path> {
        self.latest_capture_path.as_deref()
    }
}

pub(crate) fn preload_renderdoc_dll(
    path: Option<&Path>,
    capture_path: Option<&Path>,
) -> Result<Option<RenderDocBridge>, Box<dyn Error>> {
    let Some(path) = path else {
        return Ok(None);
    };

    if !path.is_file() {
        return Err(format!("RenderDoc DLL does not exist: {}", path.display()).into());
    }

    // wgpu's Windows integration only accepts a RenderDoc module already loaded into this
    // process. Keep the handle alive through the event loop so that it remains available when
    // SceneRenderer creates its D3D12 device on the background loader thread.
    let bridge = RenderDocBridge {
        library: unsafe { Library::new(path) }.map_err(|error| {
            format!(
                "failed to preload RenderDoc DLL {}: {error}",
                path.display()
            )
        })?,
    };
    if let Some(capture_path) = capture_path {
        configure_capture_path(&bridge, capture_path)?;
    }
    Ok(Some(bridge))
}

impl RenderDocBridge {
    pub(crate) fn capture_report(&self) -> Result<RenderDocCaptureReport, String> {
        let api = self.api()?;
        let get_num_captures = api
            .get_num_captures
            .ok_or("RenderDoc API lacks GetNumCaptures")?;
        let capture_count = unsafe { get_num_captures() };
        let latest_capture_path = if capture_count == 0 {
            None
        } else {
            Some(self.capture_path(api, capture_count - 1)?)
        };
        Ok(RenderDocCaptureReport {
            capture_count,
            latest_capture_path,
        })
    }

    fn api(&self) -> Result<&RenderDocApi141, String> {
        type RenderDocGetApi = unsafe extern "C" fn(u32, *mut *mut c_void) -> i32;

        let get_api: Symbol<RenderDocGetApi> = unsafe { self.library.get(b"RENDERDOC_GetAPI\0") }
            .map_err(|error| {
            format!("RenderDoc DLL does not export RENDERDOC_GetAPI: {error}")
        })?;
        let mut api = std::ptr::null_mut();
        let result = unsafe { get_api(10401, &mut api) };
        if result != 1 || api.is_null() {
            return Err(format!(
                "RenderDoc API 1.4.1 is unavailable (result {result})"
            ));
        }
        Ok(unsafe { &*api.cast::<RenderDocApi141>() })
    }

    fn capture_path(&self, api: &RenderDocApi141, index: u32) -> Result<PathBuf, String> {
        let get_capture = api.get_capture.ok_or("RenderDoc API lacks GetCapture")?;
        let mut path_length = 0;
        let mut timestamp = 0;
        unsafe {
            get_capture(
                index,
                std::ptr::null_mut(),
                &mut path_length,
                &mut timestamp,
            )
        };
        if path_length == 0 {
            return Err("RenderDoc reported a capture without a file path".to_owned());
        }

        let mut path = vec![0_i8; path_length as usize];
        let success =
            unsafe { get_capture(index, path.as_mut_ptr(), &mut path_length, &mut timestamp) };
        if success == 0 {
            return Err("RenderDoc could not read the latest capture path".to_owned());
        }
        let path = unsafe { CStr::from_ptr(path.as_ptr()) };
        Ok(PathBuf::from(path.to_string_lossy().as_ref()))
    }
}

fn configure_capture_path(
    bridge: &RenderDocBridge,
    capture_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let Some(parent) = capture_path.parent() else {
        return Err(format!(
            "RenderDoc capture template must have a parent directory: {}",
            capture_path.display()
        )
        .into());
    };
    if !parent.is_dir() {
        return Err(format!(
            "RenderDoc capture template directory does not exist: {}",
            parent.display()
        )
        .into());
    }

    let capture_path = CString::new(capture_path.to_string_lossy().as_bytes())?;
    let api = bridge
        .api()
        .map_err(|error| -> Box<dyn Error> { error.into() })?;
    let set_capture_path = api
        .set_capture_file_path_template
        .ok_or("RenderDoc API lacks SetCaptureFilePathTemplate")?;
    unsafe { set_capture_path(capture_path.as_ptr()) };
    Ok(())
}
