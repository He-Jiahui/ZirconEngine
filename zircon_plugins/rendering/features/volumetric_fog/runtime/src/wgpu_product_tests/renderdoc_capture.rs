use std::ffi::{c_char, c_void, CStr, CString};
use std::path::{Path, PathBuf};
use std::ptr::null_mut;
use std::thread;
use std::time::{Duration, Instant};

const RENDERDOC_API_VERSION_1_0_0: i32 = 10_000;
const CAPTURE_REGISTRATION_WAIT_TIMEOUT: Duration = Duration::from_secs(10);
const CAPTURE_FILE_WAIT_TIMEOUT: Duration = Duration::from_secs(10);
const CAPTURE_FILE_POLL_INTERVAL: Duration = Duration::from_millis(20);

type RenderDocGetApi = unsafe extern "C" fn(i32, *mut *mut c_void) -> i32;
type SetCaptureFilePathTemplate = unsafe extern "C" fn(*const c_char);
type GetNumCaptures = unsafe extern "C" fn() -> u32;
type GetCapture = unsafe extern "C" fn(u32, *mut c_char, *mut u32, *mut u64) -> u32;
type StartFrameCapture = unsafe extern "C" fn(*mut c_void, *mut c_void);
type IsFrameCapturing = unsafe extern "C" fn() -> u32;
type EndFrameCapture = unsafe extern "C" fn(*mut c_void, *mut c_void) -> u32;

#[repr(C)]
struct RenderDocApi100 {
    _get_api_version: *const c_void,
    _set_capture_option_u32: *const c_void,
    _set_capture_option_f32: *const c_void,
    _get_capture_option_u32: *const c_void,
    _get_capture_option_f32: *const c_void,
    _set_focus_toggle_keys: *const c_void,
    _set_capture_keys: *const c_void,
    _get_overlay_bits: *const c_void,
    _mask_overlay_bits: *const c_void,
    _shutdown: *const c_void,
    _unload_crash_handler: *const c_void,
    set_capture_file_path_template: SetCaptureFilePathTemplate,
    _get_capture_file_path_template: *const c_void,
    get_num_captures: GetNumCaptures,
    get_capture: GetCapture,
    _trigger_capture: *const c_void,
    _is_target_control_connected: *const c_void,
    _launch_replay_ui: *const c_void,
    _set_active_window: *const c_void,
    start_frame_capture: StartFrameCapture,
    is_frame_capturing: IsFrameCapturing,
    end_frame_capture: EndFrameCapture,
}

struct ActiveCapture<'a> {
    api: &'a RenderDocApi100,
    capture_index: u32,
    ended: bool,
}

impl ActiveCapture<'_> {
    fn end(mut self) -> Result<PathBuf, String> {
        let captured = unsafe { (self.api.end_frame_capture)(null_mut(), null_mut()) };
        self.ended = true;
        if captured == 0 {
            return Err("RenderDoc rejected the offscreen frame capture".to_owned());
        }
        capture_path(self.api, self.capture_index)
    }
}

impl Drop for ActiveCapture<'_> {
    fn drop(&mut self) {
        if !self.ended {
            unsafe {
                (self.api.end_frame_capture)(null_mut(), null_mut());
            }
        }
    }
}

pub(super) fn capture_offscreen_frame<T>(
    capture_template: &Path,
    render: impl FnOnce() -> T,
) -> Result<(T, PathBuf), String> {
    let api = renderdoc_api()?;
    let capture_template = CString::new(capture_template.to_string_lossy().as_bytes())
        .map_err(|_| "RenderDoc capture template contains an embedded NUL".to_owned())?;
    unsafe {
        (api.set_capture_file_path_template)(capture_template.as_ptr());
    }

    let capture_index = unsafe { (api.get_num_captures)() };
    unsafe {
        (api.start_frame_capture)(null_mut(), null_mut());
    }
    if unsafe { (api.is_frame_capturing)() } == 0 {
        return Err("RenderDoc did not start a headless frame capture".to_owned());
    }

    let active_capture = ActiveCapture {
        api,
        capture_index,
        ended: false,
    };
    let rendered = render();
    let capture_path = active_capture.end()?;
    Ok((rendered, capture_path))
}

fn renderdoc_api() -> Result<&'static RenderDocApi100, String> {
    let module_name: Vec<u16> = "renderdoc.dll".encode_utf16().chain([0]).collect();
    let module = unsafe { GetModuleHandleW(module_name.as_ptr()) };
    if module.is_null() {
        return Err("renderdoc.dll is not injected into the product test process".to_owned());
    }

    let get_api_address = unsafe { GetProcAddress(module, c"RENDERDOC_GetAPI".as_ptr().cast()) };
    if get_api_address.is_null() {
        return Err("renderdoc.dll does not export RENDERDOC_GetAPI".to_owned());
    }
    let get_api: RenderDocGetApi = unsafe { std::mem::transmute(get_api_address) };
    let mut api = null_mut();
    if unsafe { get_api(RENDERDOC_API_VERSION_1_0_0, &mut api) } != 1 || api.is_null() {
        return Err("RenderDoc rejected API version 1.0.0".to_owned());
    }
    Ok(unsafe { &*api.cast::<RenderDocApi100>() })
}

fn capture_path(api: &RenderDocApi100, capture_index: u32) -> Result<PathBuf, String> {
    let registration_deadline = Instant::now() + CAPTURE_REGISTRATION_WAIT_TIMEOUT;
    if !capture_registration_available(
        capture_index,
        || unsafe { (api.get_num_captures)() },
        || {
            if Instant::now() >= registration_deadline {
                return false;
            }
            thread::sleep(CAPTURE_FILE_POLL_INTERVAL);
            true
        },
    ) {
        return Err("RenderDoc ended the frame without registering a capture".to_owned());
    }

    let mut path_length = 0;
    if unsafe { (api.get_capture)(capture_index, null_mut(), &mut path_length, null_mut()) } == 0
        || path_length == 0
    {
        return Err("RenderDoc did not expose the capture path length".to_owned());
    }
    let mut path = vec![0_u8; path_length as usize];
    if unsafe {
        (api.get_capture)(
            capture_index,
            path.as_mut_ptr().cast(),
            &mut path_length,
            null_mut(),
        )
    } == 0
    {
        return Err("RenderDoc did not expose the capture path".to_owned());
    }
    let capture_path = CStr::from_bytes_until_nul(&path)
        .map_err(|_| "RenderDoc returned a non-terminated capture path".to_owned())?
        .to_str()
        .map_err(|_| "RenderDoc returned a non-UTF-8 capture path".to_owned())?;
    wait_for_capture_file(PathBuf::from(capture_path))
}

fn capture_registration_available(
    capture_index: u32,
    mut capture_count: impl FnMut() -> u32,
    mut wait_for_retry: impl FnMut() -> bool,
) -> bool {
    loop {
        if capture_count() > capture_index {
            return true;
        }
        if !wait_for_retry() {
            return false;
        }
    }
}

fn wait_for_capture_file(capture_path: PathBuf) -> Result<PathBuf, String> {
    let deadline = Instant::now() + CAPTURE_FILE_WAIT_TIMEOUT;
    while !capture_path.is_file() {
        if Instant::now() >= deadline {
            return Err(format!(
                "RenderDoc capture was registered but not written to {}",
                capture_path.display()
            ));
        }
        thread::sleep(CAPTURE_FILE_POLL_INTERVAL);
    }
    Ok(capture_path)
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(module_name: *const u16) -> *mut c_void;
    fn GetProcAddress(module: *mut c_void, proc_name: *const u8) -> *const c_void;
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::mem::{offset_of, size_of};

    use super::{capture_registration_available, RenderDocApi100};

    #[test]
    fn api_1_0_layout_keeps_offscreen_capture_function_offsets() {
        let pointer_size = size_of::<*const ()>();
        assert_eq!(
            offset_of!(RenderDocApi100, set_capture_file_path_template),
            11 * pointer_size
        );
        assert_eq!(
            offset_of!(RenderDocApi100, start_frame_capture),
            19 * pointer_size
        );
        assert_eq!(
            offset_of!(RenderDocApi100, is_frame_capturing),
            20 * pointer_size
        );
        assert_eq!(
            offset_of!(RenderDocApi100, end_frame_capture),
            21 * pointer_size
        );
        assert_eq!(size_of::<RenderDocApi100>(), 22 * pointer_size);
    }

    #[test]
    fn capture_registration_waits_for_async_capture_list_update() {
        let polls = Cell::new(0_u32);
        let retries = Cell::new(0_u32);

        let registered = capture_registration_available(
            0,
            || {
                let poll = polls.get() + 1;
                polls.set(poll);
                u32::from(poll >= 3)
            },
            || {
                retries.set(retries.get() + 1);
                true
            },
        );

        assert!(registered);
        assert_eq!(polls.get(), 3);
        assert_eq!(retries.get(), 2);

        polls.set(0);
        retries.set(0);

        let registered = capture_registration_available(
            0,
            || {
                polls.set(polls.get() + 1);
                0
            },
            || {
                retries.set(retries.get() + 1);
                false
            },
        );

        assert!(!registered);
        assert_eq!(polls.get(), 1);
        assert_eq!(retries.get(), 1);
    }
}
