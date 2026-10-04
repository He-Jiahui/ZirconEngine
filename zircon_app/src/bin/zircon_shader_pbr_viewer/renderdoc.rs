//! RenderDoc 原生诊断库的显式预加载、模板配置与捕获工件准入。
//! 库句柄随宿主保留；捕获列表与工件存在性、外部重放验证分别承担不同证据。

use std::error::Error;
use std::ffi::{c_char, c_void, CStr, CString};
use std::path::{Path, PathBuf};

use libloading::{Library, Symbol};

#[repr(C)]
struct RenderDocApi141 {
    // `RENDERDOC_API_1_4_1` exposes SetCaptureFilePathTemplate as its twelfth pointer-sized
    // entry. The viewer needs only that stable public ABI prefix, avoiding a new direct crate
    // dependency solely for a diagnostic tool integration.
    _functions_before_capture_template: [usize; 11],
    set_capture_file_path_template: Option<unsafe extern "C" fn(*const c_char)>,
    get_capture_file_path_template: Option<unsafe extern "C" fn() -> *const c_char>,
    get_num_captures: Option<unsafe extern "C" fn() -> u32>,
    get_capture: Option<unsafe extern "C" fn(u32, *mut c_char, *mut u32, *mut u64) -> u32>,
}

/// 显式加载的调试库所有权；宿主须保留到场景释放后，API 表和函数指针才保持有效。
pub(crate) struct RenderDocBridge {
    library: Library,
}

/// 捕获列表的当前快照；列表仍可能引用已被外部删除的文件，证据准入需要再次查文件。
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

    /// 捕获停止后确认最新工件仍存在且非空；这一步不证明捕获可成功重放。
    pub(crate) fn capture_path_for_evidence(&self) -> Result<&Path, String> {
        if self.capture_count == 0 {
            return Err("RenderDoc did not record a capture".to_owned());
        }
        let capture_path = self.latest_capture_path().ok_or_else(|| {
            "RenderDoc reported a capture without a latest capture path".to_owned()
        })?;
        if capture_path
            .extension()
            .and_then(|extension| extension.to_str())
            != Some("rdc")
        {
            return Err(format!(
                "RenderDoc reported a capture without a lowercase .rdc artifact: {}",
                capture_path.display()
            ));
        }
        let metadata = std::fs::metadata(capture_path).map_err(|error| {
            format!(
                "RenderDoc capture artifact is unavailable: {} ({error})",
                capture_path.display()
            )
        })?;
        if !metadata.is_file() {
            return Err(format!(
                "RenderDoc capture artifact is not a regular file: {}",
                capture_path.display()
            ));
        }
        if metadata.len() == 0 {
            return Err(format!(
                "RenderDoc capture artifact is empty: {}",
                capture_path.display()
            ));
        }
        Ok(capture_path)
    }
}

#[cfg(test)]
#[path = "tests/renderdoc.rs"]
mod tests;

/// 仅用于显式诊断捕获；指定库必须是可信 RenderDoc，模板父目录需预先存在。
/// 在图形设备初始化前调用，并让返回句柄活到所有图形所有者释放之后。
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
    // SAFETY: 调用前提是显式指定可信原生调试库；仅检查文件存在不证明可信，宿主保留句柄到场景释放后。
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
        // SAFETY: 函数来自已成功协商的 API 表且当前库仍由 self 持有；此查询无指针参数。
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

    fn capture_file_path_template(&self) -> Result<PathBuf, String> {
        let api = self.api()?;
        let get_capture_path = api
            .get_capture_file_path_template
            .ok_or("RenderDoc API lacks GetCaptureFilePathTemplate")?;
        // SAFETY: 已协商表中的非空查询函数在库存活期间返回库拥有的模板字符串。
        let capture_path = unsafe { get_capture_path() };
        if capture_path.is_null() {
            return Err("RenderDoc returned a null capture file path template".to_owned());
        }
        // SAFETY: 已排除空指针；可信 API 的返回值是 NUL 结尾 UTF-8 字符串，借用后立即复制且不改模板。
        let capture_path = unsafe { CStr::from_ptr(capture_path) };
        Ok(PathBuf::from(capture_path.to_string_lossy().as_ref()))
    }

    fn api(&self) -> Result<&RenderDocApi141, String> {
        type RenderDocGetApi = unsafe extern "C" fn(u32, *mut *mut c_void) -> i32;

        // SAFETY: 可信 RenderDoc 的具名入口采用公共 C 调用约定；Symbol 的借用绑定当前库。
        let get_api: Symbol<RenderDocGetApi> = unsafe { self.library.get(b"RENDERDOC_GetAPI\0") }
            .map_err(|error| {
            format!("RenderDoc DLL does not export RENDERDOC_GetAPI: {error}")
        })?;
        let mut api = std::ptr::null_mut();
        // SAFETY: 传入受支持的版本值和可写指针槽；宿主在设备初始化前配置，捕获结束后查询，避免并发协商。
        let result = unsafe { get_api(10401, &mut api) };
        if result != 1 || api.is_null() {
            return Err(format!(
                "RenderDoc API 1.4.1 is unavailable (result {result})"
            ));
        }
        // SAFETY: 成功且非空的公共 API 返回值具有上述 C 前缀布局；返回借用不超过持有库的 self。
        Ok(unsafe { &*api.cast::<RenderDocApi141>() })
    }

    fn capture_path(&self, api: &RenderDocApi141, index: u32) -> Result<PathBuf, String> {
        let get_capture = api.get_capture.ok_or("RenderDoc API lacks GetCapture")?;
        let mut path_length = 0;
        let mut timestamp = 0;
        // SAFETY: 库和表存活；公共 API 允许空路径缓冲只查询长度，两个输出槽均可写。
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
        // TODO: [CR-APP-VIEWER-0005] 确认两次查询间捕获索引保持同一记录；库可删记录且写入不校验容量，宿主串行仍不足以证明缓冲安全。
        // SAFETY: 在记录稳定此前提成立时，首轮长度包含 NUL，当前 Vec 为同一路径提供足够空间；此前提仍待上述审查。
        let success =
            unsafe { get_capture(index, path.as_mut_ptr(), &mut path_length, &mut timestamp) };
        if success == 0 {
            return Err("RenderDoc could not read the latest capture path".to_owned());
        }
        // SAFETY: 成功写入的路径是 NUL 结尾字符串，缓冲仍由本函数持有，随后立即复制。
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

    let capture_template = CString::new(capture_path.to_string_lossy().as_bytes())?;
    let api = bridge
        .api()
        .map_err(|error| -> Box<dyn Error> { error.into() })?;
    let set_capture_path = api
        .set_capture_file_path_template
        .ok_or("RenderDoc API lacks SetCaptureFilePathTemplate")?;
    // SAFETY: CString 在调用期间存活且 NUL 结尾；公共 setter 同步复制模板，库和 API 表仍存活。
    unsafe { set_capture_path(capture_template.as_ptr()) };
    let active_capture_template = bridge
        .capture_file_path_template()
        .map_err(|error| -> Box<dyn Error> { error.into() })?;
    let expected_capture_template = capture_template_identity(capture_path);
    if !capture_template_matches(&expected_capture_template, &active_capture_template) {
        return Err(format!(
            "RenderDoc did not apply the requested capture template: expected {}, active {}",
            expected_capture_template.display(),
            active_capture_template.display(),
        )
        .into());
    }
    println!(
        "configured RenderDoc capture template: {}",
        active_capture_template.display()
    );
    Ok(())
}

fn capture_template_identity(capture_path: &Path) -> PathBuf {
    // RenderDoc 1.44 strips only a lowercase `.rdc` suffix from the active template.
    if capture_path
        .extension()
        .and_then(|extension| extension.to_str())
        == Some("rdc")
    {
        capture_path.with_extension("")
    } else {
        capture_path.to_path_buf()
    }
}

fn capture_template_matches(expected: &Path, active: &Path) -> bool {
    let normalize = |path: &Path| {
        path.to_string_lossy()
            .replace('/', "\\")
            .to_ascii_lowercase()
    };
    normalize(expected) == normalize(active)
}
