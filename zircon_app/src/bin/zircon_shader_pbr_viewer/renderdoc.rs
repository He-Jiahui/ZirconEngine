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
mod tests {
    use super::RenderDocCaptureReport;

    fn temporary_capture_path(extension: &str) -> std::path::PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time must be after the Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "zircon-renderdoc-capture-evidence-{}-{nonce}.{extension}",
            std::process::id()
        ))
    }

    #[test]
    fn renderdoc_capture_report_rejects_an_empty_capture_list() {
        let report = RenderDocCaptureReport {
            capture_count: 0,
            latest_capture_path: None,
        };

        let error = report
            .capture_path_for_evidence()
            .expect_err("a completed bridge capture requires an actual RenderDoc record");

        assert!(error.contains("did not record a capture"));
    }

    #[test]
    fn renderdoc_capture_report_rejects_a_missing_latest_path() {
        let report = RenderDocCaptureReport {
            capture_count: 1,
            latest_capture_path: None,
        };

        let error = report
            .capture_path_for_evidence()
            .expect_err("a completed bridge capture requires the latest capture path");

        assert!(error.contains("without a latest capture path"));
    }

    #[test]
    fn renderdoc_capture_report_rejects_a_missing_capture_artifact() {
        let capture_path = temporary_capture_path("rdc");
        let report = RenderDocCaptureReport {
            capture_count: 1,
            latest_capture_path: Some(capture_path),
        };

        let error = report
            .capture_path_for_evidence()
            .expect_err("a missing RenderDoc capture must not be accepted as evidence");

        assert!(error.contains("artifact is unavailable"));
    }

    #[test]
    fn renderdoc_capture_report_rejects_non_lowercase_rdc_artifacts() {
        let capture_path = temporary_capture_path("RDC");
        std::fs::write(&capture_path, b"RenderDoc capture").expect("write capture fixture");
        let report = RenderDocCaptureReport {
            capture_count: 1,
            latest_capture_path: Some(capture_path.clone()),
        };

        let result = report.capture_path_for_evidence();
        std::fs::remove_file(&capture_path).expect("remove capture fixture");

        let error = result.expect_err("only lowercase .rdc files are valid capture evidence");
        assert!(error.contains("lowercase .rdc artifact"));
    }

    #[test]
    fn renderdoc_capture_report_rejects_empty_capture_artifacts() {
        let capture_path = temporary_capture_path("rdc");
        std::fs::write(&capture_path, b"").expect("write empty capture fixture");
        let report = RenderDocCaptureReport {
            capture_count: 1,
            latest_capture_path: Some(capture_path.clone()),
        };

        let result = report.capture_path_for_evidence();
        std::fs::remove_file(&capture_path).expect("remove capture fixture");

        let error =
            result.expect_err("an empty RenderDoc capture must not be accepted as evidence");
        assert!(error.contains("artifact is empty"));
    }

    #[test]
    fn renderdoc_capture_report_preserves_an_existing_nonempty_rdc_evidence_path() {
        let capture_path = temporary_capture_path("rdc");
        std::fs::write(&capture_path, b"RenderDoc capture").expect("write capture fixture");
        let report = RenderDocCaptureReport {
            capture_count: 1,
            latest_capture_path: Some(capture_path.clone()),
        };

        let result = report
            .capture_path_for_evidence()
            .map(std::path::Path::to_path_buf);
        std::fs::remove_file(&capture_path).expect("remove capture fixture");

        assert_eq!(
            result.expect("a completed bridge capture should expose its artifact path"),
            capture_path
        );
    }

    #[test]
    fn renderdoc_api_prefix_covers_template_and_capture_report_entries() {
        assert_eq!(
            std::mem::size_of::<super::RenderDocApi141>(),
            std::mem::size_of::<usize>() * 15,
            "RenderDoc API 1.4.1 needs 11 leading entries plus template, template query, count, and capture pointers"
        );
    }

    #[test]
    fn renderdoc_capture_template_identity_strips_only_the_lowercase_rdc_extension() {
        assert_eq!(
            super::capture_template_identity(std::path::Path::new("E:/evidence/pbr-frame.rdc")),
            std::path::PathBuf::from("E:/evidence/pbr-frame")
        );
        assert_eq!(
            super::capture_template_identity(std::path::Path::new("E:/evidence/pbr-frame.RDC")),
            std::path::PathBuf::from("E:/evidence/pbr-frame.RDC")
        );
        assert_eq!(
            super::capture_template_identity(std::path::Path::new("E:/evidence/pbr-frame.log")),
            std::path::PathBuf::from("E:/evidence/pbr-frame.log")
        );
    }

    #[test]
    fn renderdoc_capture_template_identity_requires_the_applied_template() {
        let expected = std::path::Path::new("E:/evidence/pbr-frame");

        assert!(super::capture_template_matches(
            expected,
            std::path::Path::new("E:/evidence/pbr-frame")
        ));
        assert!(super::capture_template_matches(
            expected,
            std::path::Path::new("e:\\Evidence\\PBR-FRAME")
        ));
        assert!(!super::capture_template_matches(
            expected,
            std::path::Path::new("E:/other/pbr-frame")
        ));
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

    fn capture_file_path_template(&self) -> Result<PathBuf, String> {
        let api = self.api()?;
        let get_capture_path = api
            .get_capture_file_path_template
            .ok_or("RenderDoc API lacks GetCaptureFilePathTemplate")?;
        let capture_path = unsafe { get_capture_path() };
        if capture_path.is_null() {
            return Err("RenderDoc returned a null capture file path template".to_owned());
        }
        let capture_path = unsafe { CStr::from_ptr(capture_path) };
        Ok(PathBuf::from(capture_path.to_string_lossy().as_ref()))
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

    let capture_template = CString::new(capture_path.to_string_lossy().as_bytes())?;
    let api = bridge
        .api()
        .map_err(|error| -> Box<dyn Error> { error.into() })?;
    let set_capture_path = api
        .set_capture_file_path_template
        .ok_or("RenderDoc API lacks SetCaptureFilePathTemplate")?;
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
