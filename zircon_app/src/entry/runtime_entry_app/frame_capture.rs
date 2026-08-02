use std::ffi::OsString;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use image::ImageEncoder;

static NEXT_CAPTURE_STAGING_ID: AtomicU64 = AtomicU64::new(1);

trait FrameCaptureSync {
    fn sync_frame_capture(&self) -> std::io::Result<()>;
}

impl FrameCaptureSync for std::fs::File {
    fn sync_frame_capture(&self) -> std::io::Result<()> {
        self.sync_all()
    }
}

pub(super) fn write_runtime_frame_png(
    path: &Path,
    width: u32,
    height: u32,
    rgba: &[u8],
) -> Result<(), String> {
    let expected_len = width
        .checked_mul(height)
        .and_then(|pixel_count| pixel_count.checked_mul(4))
        .map(usize::try_from)
        .transpose()
        .map_err(|error| format!("frame dimensions do not fit usize: {error}"))?
        .ok_or_else(|| "frame dimensions overflow".to_owned())?;
    if rgba.len() != expected_len {
        return Err(format!(
            "frame RGBA length {} does not match {width}x{height} output",
            rgba.len()
        ));
    }
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|error| {
            format!(
                "create frame capture directory {}: {error}",
                parent.display()
            )
        })?;
    }
    let (staging_path, staging_file) = reserve_frame_capture_staging_file(path)?;
    if let Err(error) = encode_frame_capture_staging_file(staging_file, path, width, height, rgba) {
        return Err(remove_staging_after_failure(&staging_path, error));
    }
    if let Err(error) = commit_frame_capture_staging_file(&staging_path, path) {
        return Err(remove_staging_after_failure(
            &staging_path,
            format!(
                "commit frame capture {} from {}: {error}",
                path.display(),
                staging_path.display()
            ),
        ));
    }
    Ok(())
}

fn commit_frame_capture_staging_file(
    staging_path: &Path,
    final_path: &Path,
) -> std::io::Result<()> {
    #[cfg(windows)]
    if final_path.exists() {
        return replace_existing_frame_capture_file(staging_path, final_path);
    }

    std::fs::rename(staging_path, final_path)
}

#[cfg(windows)]
fn replace_existing_frame_capture_file(
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

fn reserve_frame_capture_staging_file(path: &Path) -> Result<(PathBuf, std::fs::File), String> {
    const MAX_STAGING_ATTEMPTS: usize = 64;

    let file_name = path
        .file_name()
        .ok_or_else(|| format!("frame capture path {} has no file name", path.display()))?;
    for _ in 0..MAX_STAGING_ATTEMPTS {
        let id = NEXT_CAPTURE_STAGING_ID.fetch_add(1, Ordering::Relaxed);
        let mut staging_name = OsString::from(file_name);
        staging_name.push(format!(".partial-{}-{id}", std::process::id()));
        let staging_path = path.with_file_name(staging_name);
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staging_path)
        {
            Ok(file) => return Ok((staging_path, file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "create frame capture staging file {}: {error}",
                    staging_path.display()
                ));
            }
        }
    }
    Err(format!(
        "could not reserve a frame capture staging file beside {} after {MAX_STAGING_ATTEMPTS} attempts",
        path.display()
    ))
}

fn encode_frame_capture_staging_file(
    staging_file: std::fs::File,
    final_path: &Path,
    width: u32,
    height: u32,
    rgba: &[u8],
) -> Result<(), String> {
    let mut writer = BufWriter::new(staging_file);
    image::codecs::png::PngEncoder::new(&mut writer)
        .write_image(rgba, width, height, image::ExtendedColorType::Rgba8)
        .map_err(|error| format!("encode frame capture {}: {error}", final_path.display()))?;
    // Buffered encoder success is not durable until both userspace and filesystem writes finish.
    flush_frame_capture_writer(&mut writer, final_path)?;
    sync_frame_capture_writer(writer.get_ref(), final_path)?;
    Ok(())
}

fn flush_frame_capture_writer(writer: &mut impl Write, final_path: &Path) -> Result<(), String> {
    writer
        .flush()
        .map_err(|error| format!("flush frame capture {}: {error}", final_path.display()))
}

fn sync_frame_capture_writer(
    writer: &impl FrameCaptureSync,
    final_path: &Path,
) -> Result<(), String> {
    writer
        .sync_frame_capture()
        .map_err(|error| format!("sync frame capture {}: {error}", final_path.display()))
}

fn remove_staging_after_failure(staging_path: &Path, failure: String) -> String {
    match std::fs::remove_file(staging_path) {
        Ok(()) => failure,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => failure,
        Err(error) => format!(
            "{failure}; cleanup frame capture staging file {} failed: {error}",
            staging_path.display()
        ),
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::{
        FrameCaptureSync, flush_frame_capture_writer, sync_frame_capture_writer,
        write_runtime_frame_png,
    };

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

    impl FrameCaptureSync for SyncFailureWriter {
        fn sync_frame_capture(&self) -> std::io::Result<()> {
            Err(std::io::Error::other("sync unavailable"))
        }
    }

    fn capture_test_root(case_name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "zircon-runtime-frame-capture-{}-{case_name}",
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
    fn runtime_frame_png_encoder_roundtrips_rgba_pixels() {
        let path = std::env::temp_dir().join(format!(
            "zircon_runtime_frame_capture_{}_{}.png",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time after Unix epoch")
                .as_nanos()
        ));
        let rgba = [
            255, 0, 0, 255, // red
            0, 255, 0, 128, // green with alpha
        ];

        write_runtime_frame_png(&path, 2, 1, &rgba).expect("frame capture PNG should encode");
        let decoded = image::open(&path)
            .expect("written frame capture PNG should decode")
            .to_rgba8();
        let _ = std::fs::remove_file(&path);

        assert_eq!(decoded.dimensions(), (2, 1));
        assert_eq!(decoded.as_raw(), &rgba);
    }

    #[test]
    fn runtime_frame_png_encoder_rejects_mismatched_rgba_without_writing_evidence() {
        let path = std::env::temp_dir().join(format!(
            "zircon_runtime_frame_capture_invalid_{}_{}.png",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time after Unix epoch")
                .as_nanos()
        ));

        let error = write_runtime_frame_png(&path, 2, 1, &[255, 0, 0, 255])
            .expect_err("truncated RGBA frame must not produce PNG evidence");

        assert!(error.contains("does not match 2x1 output"));
        assert!(!path.exists());
    }

    #[test]
    fn runtime_frame_png_encoder_cleans_staging_file_when_commit_fails() {
        let root = capture_test_root("commit-failure");
        let path = root.join("runtime-first-frame.png");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&path).unwrap();

        let error = write_runtime_frame_png(&path, 1, 1, &[255, 0, 0, 255])
            .expect_err("a directory cannot be committed as PNG evidence");

        assert!(error.contains("commit frame capture"), "{error}");
        assert!(path.is_dir(), "failed commit must preserve the destination");
        assert_eq!(partial_capture_files(&root), Vec::new());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn runtime_frame_png_encoder_cleans_staging_file_when_encoding_fails() {
        let root = capture_test_root("encode-failure");
        let path = root.join("runtime-first-frame.png");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let error = write_runtime_frame_png(&path, 0, 1, &[])
            .expect_err("zero-width PNG evidence must fail during encoding");

        assert!(error.contains("encode frame capture"), "{error}");
        assert!(!path.exists());
        assert_eq!(partial_capture_files(&root), Vec::new());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn runtime_frame_png_encoder_replaces_existing_evidence_only_after_success() {
        let root = capture_test_root("replace-existing");
        let path = root.join("runtime-first-frame.png");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(&path, b"stale evidence").unwrap();

        write_runtime_frame_png(&path, 1, 1, &[1, 2, 3, 255])
            .expect("complete PNG should atomically replace stale evidence");
        let decoded = image::open(&path).unwrap().to_rgba8();

        assert_eq!(decoded.dimensions(), (1, 1));
        assert_eq!(decoded.as_raw(), &[1, 2, 3, 255]);
        assert_eq!(partial_capture_files(&root), Vec::new());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn frame_capture_flush_and_sync_failures_are_not_reported_as_success() {
        let path = std::path::Path::new("runtime-first-frame.png");
        let flush_error = flush_frame_capture_writer(&mut FlushFailureWriter, path)
            .expect_err("flush failure must block frame capture commit");
        let sync_error = sync_frame_capture_writer(&SyncFailureWriter, path)
            .expect_err("sync failure must block frame capture commit");

        assert_eq!(
            flush_error,
            "flush frame capture runtime-first-frame.png: flush unavailable"
        );
        assert_eq!(
            sync_error,
            "sync frame capture runtime-first-frame.png: sync unavailable"
        );
    }
}
