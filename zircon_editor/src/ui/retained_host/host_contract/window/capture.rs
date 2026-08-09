use std::ffi::OsString;
use std::fs;
use std::io::{BufWriter, Write};
#[cfg(windows)]
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use super::UiHostWindow;
use crate::ui::retained_host::host_contract::diagnostics::HostWindowDiagnosticSeverity;
use crate::ui::retained_host::primitives::PlatformError;
use image::ImageEncoder;
use zircon_runtime::asset::project::ResolvedProjectPath;

static NEXT_EDITOR_CAPTURE_STAGING_ID: AtomicU64 = AtomicU64::new(1);

trait EditorCaptureSync {
    fn sync_editor_capture(&self) -> std::io::Result<()>;
}

impl EditorCaptureSync for fs::File {
    fn sync_editor_capture(&self) -> std::io::Result<()> {
        self.sync_all()
    }
}

impl UiHostWindow {
    /// Saves the host presentation only after a native presenter reports success.
    pub(in crate::ui::retained_host::host_contract) fn capture_first_presented_frame(
        &self,
    ) -> Result<Option<ResolvedProjectPath>, PlatformError> {
        let Some(path) = self
            .state
            .borrow_mut()
            .first_presented_frame_capture_path
            .take()
        else {
            return Ok(None);
        };
        let snapshot = self.window().take_snapshot()?;
        write_editor_frame_png(
            &path,
            snapshot.width(),
            snapshot.height(),
            snapshot.as_bytes(),
        )?;
        self.record_host_diagnostic(
            HostWindowDiagnosticSeverity::Info,
            format!(
                "editor_product_frame_capture_written path={}",
                path.display_path().display()
            ),
        );
        Ok(Some(path))
    }
}

fn write_editor_frame_png(
    path: &ResolvedProjectPath,
    width: u32,
    height: u32,
    rgba: &[u8],
) -> Result<(), PlatformError> {
    let expected_len = width
        .checked_mul(height)
        .and_then(|pixel_count| pixel_count.checked_mul(4))
        .map(usize::try_from)
        .transpose()
        .map_err(|error| {
            editor_capture_error(format!("editor frame dimensions do not fit usize: {error}"))
        })?
        .ok_or_else(|| editor_capture_error("editor frame dimensions overflow"))?;
    if rgba.len() != expected_len {
        return Err(editor_capture_error(format!(
            "editor frame RGBA length {} does not match {width}x{height} output",
            rgba.len()
        )));
    }
    if let Some(parent) = path
        .operation_path()
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        let display_parent = path
            .display_path()
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or(path.display_path());
        fs::create_dir_all(parent).map_err(|error| {
            editor_capture_error(format!(
                "failed to create editor first-frame capture directory '{}': {error}",
                display_parent.display()
            ))
        })?;
    }
    let (staging_path, staging_file) = reserve_editor_capture_staging_file(path)?;
    if let Err(error) = encode_editor_capture_staging_file(staging_file, path, width, height, rgba)
    {
        return Err(remove_editor_staging_after_failure(&staging_path, error));
    }
    if let Err(error) = commit_editor_capture_staging_file(&staging_path, path) {
        return Err(remove_editor_staging_after_failure(
            &staging_path,
            editor_capture_error(format!(
                "failed to commit editor first-frame capture '{}' from '{}': {error}",
                path.display_path().display(),
                staging_path.display_path().display()
            )),
        ));
    }
    Ok(())
}

fn commit_editor_capture_staging_file(
    staging_path: &ResolvedProjectPath,
    final_path: &ResolvedProjectPath,
) -> std::io::Result<()> {
    #[cfg(windows)]
    if final_path.operation_path().exists() {
        return replace_existing_editor_capture_file(
            staging_path.operation_path(),
            final_path.operation_path(),
        );
    }

    fs::rename(staging_path.operation_path(), final_path.operation_path())
}

#[cfg(windows)]
fn replace_existing_editor_capture_file(
    staging_path: &Path,
    final_path: &Path,
) -> std::io::Result<()> {
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "Kernel32")]
    unsafe extern "system" {
        fn ReplaceFileW(
            replaced_file_name: *const u16,
            replacement_file_name: *const u16,
            backup_file_name: *const u16,
            replace_flags: u32,
            exclude: *mut c_void,
            reserved: *mut c_void,
        ) -> i32;
    }

    let final_path = final_path
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let staging_path = staging_path
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let replaced = unsafe {
        ReplaceFileW(
            final_path.as_ptr(),
            staging_path.as_ptr(),
            std::ptr::null(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if replaced == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn reserve_editor_capture_staging_file(
    path: &ResolvedProjectPath,
) -> Result<(ResolvedProjectPath, fs::File), PlatformError> {
    const MAX_STAGING_ATTEMPTS: usize = 64;

    let file_name = path.operation_path().file_name().ok_or_else(|| {
        editor_capture_error(format!(
            "editor first-frame capture path '{}' has no file name",
            path.display_path().display()
        ))
    })?;
    for _ in 0..MAX_STAGING_ATTEMPTS {
        let id = NEXT_EDITOR_CAPTURE_STAGING_ID.fetch_add(1, Ordering::Relaxed);
        let mut staging_name = OsString::from(file_name);
        staging_name.push(format!(".partial-{}-{id}", std::process::id()));
        let staging_path = path.with_file_name(staging_name);
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(staging_path.operation_path())
        {
            Ok(file) => return Ok((staging_path, file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(editor_capture_error(format!(
                    "failed to create editor first-frame capture staging file '{}': {error}",
                    staging_path.display_path().display()
                )));
            }
        }
    }
    Err(editor_capture_error(format!(
        "could not reserve an editor first-frame capture staging file beside '{}' after {MAX_STAGING_ATTEMPTS} attempts",
        path.display_path().display()
    )))
}

fn encode_editor_capture_staging_file(
    staging_file: fs::File,
    final_path: &ResolvedProjectPath,
    width: u32,
    height: u32,
    rgba: &[u8],
) -> Result<(), PlatformError> {
    let mut writer = BufWriter::new(staging_file);
    image::codecs::png::PngEncoder::new(&mut writer)
        .write_image(rgba, width, height, image::ExtendedColorType::Rgba8)
        .map_err(|error| {
            editor_capture_error(format!(
                "failed to encode editor first-frame capture '{}': {error}",
                final_path.display_path().display()
            ))
        })?;
    // The evidence is publishable only after buffered and filesystem writes both succeed.
    flush_editor_capture_writer(&mut writer, final_path)?;
    sync_editor_capture_writer(writer.get_ref(), final_path)?;
    Ok(())
}

fn flush_editor_capture_writer(
    writer: &mut impl Write,
    final_path: &ResolvedProjectPath,
) -> Result<(), PlatformError> {
    writer.flush().map_err(|error| {
        editor_capture_error(format!(
            "failed to flush editor first-frame capture '{}': {error}",
            final_path.display_path().display()
        ))
    })
}

fn sync_editor_capture_writer(
    writer: &impl EditorCaptureSync,
    final_path: &ResolvedProjectPath,
) -> Result<(), PlatformError> {
    writer.sync_editor_capture().map_err(|error| {
        editor_capture_error(format!(
            "failed to sync editor first-frame capture '{}': {error}",
            final_path.display_path().display()
        ))
    })
}

fn remove_editor_staging_after_failure(
    staging_path: &ResolvedProjectPath,
    failure: PlatformError,
) -> PlatformError {
    match fs::remove_file(staging_path.operation_path()) {
        Ok(()) => failure,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => failure,
        Err(error) => editor_capture_error(format!(
            "{failure}; cleanup editor first-frame capture staging file '{}' failed: {error}",
            staging_path.display_path().display()
        )),
    }
}

fn editor_capture_error(message: impl Into<String>) -> PlatformError {
    PlatformError::Other(message.into())
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::path::Path;

    use super::{
        flush_editor_capture_writer, sync_editor_capture_writer, write_editor_frame_png,
        EditorCaptureSync,
    };
    use zircon_runtime::asset::project::{ProjectPaths, ResolvedProjectPath};

    struct FlushFailureWriter;

    impl Write for FlushFailureWriter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Err(std::io::Error::other("flush unavailable"))
        }
    }

    struct SyncFailureWriter;

    impl EditorCaptureSync for SyncFailureWriter {
        fn sync_editor_capture(&self) -> std::io::Result<()> {
            Err(std::io::Error::other("sync unavailable"))
        }
    }

    fn resolve_capture_path(path: &Path) -> ResolvedProjectPath {
        ProjectPaths::resolve_path(path).expect("capture path should resolve")
    }

    fn capture_test_root(case_name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "zircon-editor-frame-capture-{}-{case_name}",
            std::process::id()
        ))
    }

    fn partial_capture_files(root: &std::path::Path) -> Vec<std::path::PathBuf> {
        std::fs::read_dir(root)
            .expect("capture test root should remain readable")
            .map(|entry| {
                entry
                    .expect("capture test entry should remain readable")
                    .path()
            })
            .filter(|path| {
                path.file_name()
                    .is_some_and(|name| name.to_string_lossy().contains(".partial-"))
            })
            .collect()
    }

    #[test]
    fn editor_frame_png_encoder_roundtrips_rgba_pixels() {
        let root = capture_test_root("roundtrip");
        let path = root.join("editor-first-frame.png");
        let _ = std::fs::remove_dir_all(&root);
        let rgba = [255, 0, 0, 255, 0, 255, 0, 128];
        let resolved_path = resolve_capture_path(&path);

        write_editor_frame_png(&resolved_path, 2, 1, &rgba)
            .expect("frame capture PNG should encode");
        let decoded = image::open(&path)
            .expect("written editor capture PNG should decode")
            .to_rgba8();

        assert_eq!(decoded.dimensions(), (2, 1));
        assert_eq!(decoded.as_raw(), &rgba);
        assert_eq!(partial_capture_files(&root), Vec::new());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn editor_frame_png_encoder_rejects_mismatched_rgba_without_writing_evidence() {
        let root = capture_test_root("mismatched-rgba");
        let path = root.join("editor-first-frame.png");
        let _ = std::fs::remove_dir_all(&root);
        let resolved_path = resolve_capture_path(&path);

        let error = write_editor_frame_png(&resolved_path, 2, 1, &[255, 0, 0, 255])
            .expect_err("truncated RGBA frame must not produce PNG evidence");

        assert!(error.to_string().contains("does not match 2x1 output"));
        assert!(!path.exists());
    }

    #[test]
    fn editor_frame_png_encoder_cleans_staging_file_when_encoding_fails() {
        let root = capture_test_root("encode-failure");
        let path = root.join("editor-first-frame.png");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let resolved_path = resolve_capture_path(&path);

        let error = write_editor_frame_png(&resolved_path, 0, 1, &[])
            .expect_err("zero-width PNG evidence must fail during encoding");

        assert!(error
            .to_string()
            .contains("failed to encode editor first-frame capture"));
        assert!(!path.exists());
        assert_eq!(partial_capture_files(&root), Vec::new());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn editor_frame_png_encoder_cleans_staging_file_when_commit_fails() {
        let root = capture_test_root("commit-failure");
        let path = root.join("editor-first-frame.png");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&path).unwrap();
        let resolved_path = resolve_capture_path(&path);

        let error = write_editor_frame_png(&resolved_path, 1, 1, &[255, 0, 0, 255])
            .expect_err("a directory cannot be committed as PNG evidence");

        assert!(error
            .to_string()
            .contains("failed to commit editor first-frame capture"));
        assert!(path.is_dir(), "failed commit must preserve the destination");
        assert_eq!(partial_capture_files(&root), Vec::new());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn editor_frame_png_encoder_replaces_existing_evidence_only_after_success() {
        let root = capture_test_root("replace-existing");
        let path = root.join("editor-first-frame.png");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(&path, b"stale evidence").unwrap();
        let resolved_path = resolve_capture_path(&path);

        write_editor_frame_png(&resolved_path, 1, 1, &[1, 2, 3, 255])
            .expect("complete PNG should replace stale evidence");
        let decoded = image::open(&path).unwrap().to_rgba8();

        assert_eq!(decoded.dimensions(), (1, 1));
        assert_eq!(decoded.as_raw(), &[1, 2, 3, 255]);
        assert_eq!(partial_capture_files(&root), Vec::new());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn editor_frame_capture_flush_and_sync_failures_are_not_reported_as_success() {
        let path = resolve_capture_path(Path::new("editor-first-frame.png"));
        let flush_error = flush_editor_capture_writer(&mut FlushFailureWriter, &path)
            .expect_err("flush failure must block frame capture commit");
        let sync_error = sync_editor_capture_writer(&SyncFailureWriter, &path)
            .expect_err("sync failure must block frame capture commit");

        assert!(flush_error
            .to_string()
            .contains("flush editor first-frame capture"));
        assert!(flush_error.to_string().contains("flush unavailable"));
        assert!(sync_error
            .to_string()
            .contains("sync editor first-frame capture"));
        assert!(sync_error.to_string().contains("sync unavailable"));
    }
}
