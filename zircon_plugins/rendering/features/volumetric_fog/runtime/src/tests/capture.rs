//! Windows 产品测试的 RenderDoc FFI 桥；只在显式捕获测试中调用，要求宿主进程已注入兼容 DLL。
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
// 按 RenderDoc 1.0 函数表布局读取捕获入口；布局单测只验证偏移，不证明 DLL 在引用存活期不会卸载。
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

// 捕获会话借用 API 表；异常退出路径依赖 Drop 结束当前帧，但不负责 DLL 生命周期。
struct ActiveCapture<'a> {
    api: &'a RenderDocApi100,
    capture_index: u32,
    ended: bool,
}

impl ActiveCapture<'_> {
    fn end(mut self) -> Result<PathBuf, String> {
        // SAFETY: 此守卫由成功的开始捕获创建，空设备/窗口沿用本模块的离屏约定；API 表持有期仍须满足下方 TODO。
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
            // SAFETY: 未显式结束时仍使用开始捕获所借用的同一 API 表和离屏参数；DLL 持有期见下方 TODO。
            unsafe {
                (self.api.end_frame_capture)(null_mut(), null_mut());
            }
        }
    }
}

// 先设置独立路径再包围帧提交，返回的路径要在注册与文件落盘后使用；进程内捕获有外部文件副作用。
pub(super) fn capture_offscreen_frame<T>(
    capture_template: &Path,
    render: impl FnOnce() -> T,
) -> Result<(T, PathBuf), String> {
    let api = renderdoc_api()?;
    let capture_template = CString::new(capture_template.to_string_lossy().as_bytes())
        .map_err(|_| "RenderDoc capture template contains an embedded NUL".to_owned())?;
    // SAFETY: CString 已排除内嵌 NUL，并在调用期间保持缓冲区有效；API 表来自版本协商，DLL 持有期见下方 TODO。
    unsafe {
        (api.set_capture_file_path_template)(capture_template.as_ptr());
    }

    // SAFETY: 计数查询无外部指针参数，表来自版本协商；查询与后续捕获要求同一 DLL 保持加载。
    let capture_index = unsafe { (api.get_num_captures)() };
    // SAFETY: 使用已协商的函数表开始离屏捕获，两个空指针不借用 Rust 对象；随后立即查询是否成功。
    unsafe {
        (api.start_frame_capture)(null_mut(), null_mut());
    }
    // SAFETY: 状态查询无外部指针参数，使用刚开始捕获的同一 API 表；模块持有期见下方 TODO。
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

// API 指针借用已加载模块；GetModuleHandle 不增加引用计数，必须由外层保持模块加载。
fn renderdoc_api() -> Result<&'static RenderDocApi100, String> {
    let module_name: Vec<u16> = "renderdoc.dll".encode_utf16().chain([0]).collect();
    // SAFETY: module_name 是当前调用期间有效且以 NUL 结尾的 UTF-16 缓冲区；取得的是借用模块句柄。
    let module = unsafe { GetModuleHandleW(module_name.as_ptr()) };
    if module.is_null() {
        return Err("renderdoc.dll is not injected into the product test process".to_owned());
    }

    // SAFETY: 模块已通过非空检查，导出名是静态 NUL 字符串；模块不得并发卸载的约束仍待下方 TODO 证实。
    let get_api_address = unsafe { GetProcAddress(module, c"RENDERDOC_GetAPI".as_ptr().cast()) };
    if get_api_address.is_null() {
        return Err("renderdoc.dll does not export RENDERDOC_GetAPI".to_owned());
    }
    // SAFETY: 已排除空导出地址，签名按此 DLL 的 C ABI 获取 API 约定解释；兼容 DLL 与持续加载是调用前提。
    let get_api: RenderDocGetApi = unsafe { std::mem::transmute(get_api_address) };
    let mut api = null_mut();
    // SAFETY: api 是可写的局部输出指针，版本请求与本文件函数表一致；成功返回和非空检查后才读取表。
    if unsafe { get_api(RENDERDOC_API_VERSION_1_0_0, &mut api) } != 1 || api.is_null() {
        return Err("RenderDoc rejected API version 1.0.0".to_owned());
    }
    // TODO: [CR-PLUGIN-RENDERING-0010] 确认 RenderDoc DLL 在返回的静态 API 引用存活期间由谁保持加载；GetModuleHandleW 不取得模块所有权，本文件没有持有句柄；下一步核对测试运行器的注入与卸载契约。
    // SAFETY: 版本协商成功且表指针非空；本地布局测试约束函数偏移，静态借用所需的 DLL 持有期尚待上方 TODO 证实。
    Ok(unsafe { &*api.cast::<RenderDocApi100>() })
}

// Capture 成功返回后还需等待 API 注册索引并取得实际路径，不能从模板猜测最终文件名。
fn capture_path(api: &RenderDocApi100, capture_index: u32) -> Result<PathBuf, String> {
    let registration_deadline = Instant::now() + CAPTURE_REGISTRATION_WAIT_TIMEOUT;
    if !capture_registration_available(
        capture_index,
        // SAFETY: 无指针参数的查询闭包只借用当前 API 表；重试期间也必须保持该 DLL 加载。
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
    // SAFETY: 此次只查询长度，长度输出是有效可写变量；空路径/时间戳不借用缓冲区，索引已等到登记。
    if unsafe { (api.get_capture)(capture_index, null_mut(), &mut path_length, null_mut()) } == 0
        || path_length == 0
    {
        return Err("RenderDoc did not expose the capture path length".to_owned());
    }
    let mut path = vec![0_u8; path_length as usize];
    // SAFETY: path 已按第一次查询的容量分配，长度指针与缓冲区在本次调用期间有效；读取失败或缺少 NUL 会被拒绝。
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

// 允许 RenderDoc 异步登记捕获；生产调用方提供有截止时间的重试函数。
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

// API 路径不等于文件已落盘；显式截止时间防止产品测试无限等待。
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
// SAFETY: Win32 导入按系统调用约定声明；两处调用分别传入存活的 UTF-16 名称和静态 NUL 导出名。
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
