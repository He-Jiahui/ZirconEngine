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
#[path = "tests/capture.rs"]
mod tests;
