use crate::core::runtime::diagnostics::profiling;
use std::sync::MutexGuard;

pub(in crate::graphics::scene::scene_renderer) struct ManualCpuProfileCapture {
    _lock: MutexGuard<'static, ()>,
    finished: bool,
}

impl ManualCpuProfileCapture {
    pub(in crate::graphics::scene::scene_renderer) fn finish_and_export(mut self) {
        profiling::stop_capture();
        self.finished = true;
        let report = profiling::export_report()
            .expect("manual realtime IBL CPU profile must export its capture report");
        eprintln!(
            "realtime_ibl_cpu_profile_export session_id={} export_dir={} files={}",
            report.snapshot.session_id,
            report.export_dir,
            report.files.join(","),
        );
    }
}

impl Drop for ManualCpuProfileCapture {
    fn drop(&mut self) {
        if !self.finished {
            profiling::stop_capture();
        }
    }
}

pub(in crate::graphics::scene::scene_renderer) fn start_manual_cpu_profile_capture(
    session_id: &str,
) -> ManualCpuProfileCapture {
    let lock = profiling::test_capture_lock();
    profiling::reset_capture();
    let mut config = profiling::ProfileCaptureConfig::default();
    config.session_id = session_id.to_string();
    config.output_root = required_manual_profile_output_root();
    let status = profiling::start_capture(config);
    assert!(
        status.active,
        "manual realtime IBL CPU profiles require the `profiling` Cargo feature"
    );
    ManualCpuProfileCapture {
        _lock: lock,
        finished: false,
    }
}

fn required_manual_profile_output_root() -> String {
    let output_root = std::env::var("ZIRCON_PROFILE_OUTPUT_ROOT").expect(
        "manual realtime IBL CPU profiles require an absolute non-C `ZIRCON_PROFILE_OUTPUT_ROOT`",
    );
    let output_root = output_root.trim().to_string();
    assert!(
        is_absolute_non_c_output_root(&output_root),
        "manual realtime IBL CPU profile output must use an absolute non-C root, got `{output_root}`"
    );
    output_root
}

fn is_absolute_non_c_output_root(output_root: &str) -> bool {
    let output_root = output_root.trim();
    if output_root.starts_with(r"\\?\")
        || output_root.starts_with(r"\\.\")
        || output_root.starts_with(r"\\")
    {
        return false;
    }
    let bytes = output_root.as_bytes();
    let Some(&drive_letter) = bytes.first().filter(u8::is_ascii_alphabetic) else {
        return std::path::Path::new(output_root).is_absolute();
    };
    bytes.len() >= 3
        && bytes[1] == b':'
        && matches!(bytes[2], b'\\' | b'/')
        && !drive_letter.eq_ignore_ascii_case(&b'C')
}

#[cfg(test)]
mod tests {
    use super::is_absolute_non_c_output_root;

    #[test]
    fn manual_profile_root_requires_an_absolute_non_c_location() {
        assert!(is_absolute_non_c_output_root(
            "E:\\Git\\ZirconEngine\\docs\\tests"
        ));
        assert!(is_absolute_non_c_output_root(
            "E:/Git/ZirconEngine/docs/tests"
        ));
        assert!(!is_absolute_non_c_output_root("C:\\Users\\profile"));
        assert!(!is_absolute_non_c_output_root("\\\\?\\C:\\profiles"));
        assert!(!is_absolute_non_c_output_root("\\\\.\\C:\\profiles"));
        assert!(!is_absolute_non_c_output_root(
            "\\\\server\\share\\profiles"
        ));
        assert!(!is_absolute_non_c_output_root("target/zircon-profiles"));
    }
}
