//! 显式首帧证据的 PNG 编码与完整文件发布。
//! 先在目标旁准备完整文件，再替换目标；操作路径与展示路径分别服务 I/O 和诊断。

use std::ffi::OsString;
use std::io::{BufWriter, Write};
#[cfg(windows)]
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use image::ImageEncoder;
use zircon_runtime::asset::project::ResolvedProjectPath;

static NEXT_CAPTURE_STAGING_ID: AtomicU64 = AtomicU64::new(1);

// 将文件同步抽成可注入边界，测试可证明 flush/sync 失败不会被当作捕获完成。
trait FrameCaptureSync {
    fn sync_frame_capture(&self) -> std::io::Result<()>;
}

impl FrameCaptureSync for std::fs::File {
    fn sync_frame_capture(&self) -> std::io::Result<()> {
        self.sync_all()
    }
}

/// 发布完整首帧 PNG；输入字节仅借用于本次调用。已创建的 staging 在失败时尝试清理，清理失败附在返回诊断中。
pub(super) fn write_runtime_frame_png(
    path: &ResolvedProjectPath,
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
        .operation_path()
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        let display_parent = path
            .display_path()
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or(path.display_path());
        std::fs::create_dir_all(parent).map_err(|error| {
            format!(
                "create frame capture directory {}: {error}",
                display_parent.display()
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
                path.display_path().display(),
                staging_path.display_path().display()
            ),
        ));
    }
    Ok(())
}

// 同目录临时文件使发布留在同一卷；Windows 的已有目标采用替换 API。
fn commit_frame_capture_staging_file(
    staging_path: &ResolvedProjectPath,
    final_path: &ResolvedProjectPath,
) -> std::io::Result<()> {
    #[cfg(windows)]
    if final_path.operation_path().exists() {
        return replace_existing_frame_capture_file(
            staging_path.operation_path(),
            final_path.operation_path(),
        );
    }

    std::fs::rename(staging_path.operation_path(), final_path.operation_path())
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
    // SAFETY: 两个 UTF-16 缓冲区含终止符且在调用返回前存活；备份和保留参数均为空，flags 为零。
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

/// 仅以 create_new 取得临时文件归属；撞名重试，不能覆盖其他捕获的 staging。
fn reserve_frame_capture_staging_file(
    path: &ResolvedProjectPath,
) -> Result<(ResolvedProjectPath, std::fs::File), String> {
    const MAX_STAGING_ATTEMPTS: usize = 64;

    let file_name = path.operation_path().file_name().ok_or_else(|| {
        format!(
            "frame capture path {} has no file name",
            path.display_path().display()
        )
    })?;
    for _ in 0..MAX_STAGING_ATTEMPTS {
        let id = NEXT_CAPTURE_STAGING_ID.fetch_add(1, Ordering::Relaxed);
        let mut staging_name = OsString::from(file_name);
        staging_name.push(format!(".partial-{}-{id}", std::process::id()));
        let staging_path = path.with_file_name(staging_name);
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(staging_path.operation_path())
        {
            Ok(file) => return Ok((staging_path, file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "create frame capture staging file {}: {error}",
                    staging_path.display_path().display()
                ));
            }
        }
    }
    Err(format!(
        "could not reserve a frame capture staging file beside {} after {MAX_STAGING_ATTEMPTS} attempts",
        path.display_path().display()
    ))
}

fn encode_frame_capture_staging_file(
    staging_file: std::fs::File,
    final_path: &ResolvedProjectPath,
    width: u32,
    height: u32,
    rgba: &[u8],
) -> Result<(), String> {
    let mut writer = BufWriter::new(staging_file);
    image::codecs::png::PngEncoder::new(&mut writer)
        .write_image(rgba, width, height, image::ExtendedColorType::Rgba8)
        .map_err(|error| {
            format!(
                "encode frame capture {}: {error}",
                final_path.display_path().display()
            )
        })?;
    // Buffered encoder success is not durable until both userspace and filesystem writes finish.
    flush_frame_capture_writer(&mut writer, final_path)?;
    sync_frame_capture_writer(writer.get_ref(), final_path)?;
    Ok(())
}

fn flush_frame_capture_writer(
    writer: &mut impl Write,
    final_path: &ResolvedProjectPath,
) -> Result<(), String> {
    writer.flush().map_err(|error| {
        format!(
            "flush frame capture {}: {error}",
            final_path.display_path().display()
        )
    })
}

fn sync_frame_capture_writer(
    writer: &impl FrameCaptureSync,
    final_path: &ResolvedProjectPath,
) -> Result<(), String> {
    writer.sync_frame_capture().map_err(|error| {
        format!(
            "sync frame capture {}: {error}",
            final_path.display_path().display()
        )
    })
}

fn remove_staging_after_failure(staging_path: &ResolvedProjectPath, failure: String) -> String {
    match std::fs::remove_file(staging_path.operation_path()) {
        Ok(()) => failure,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => failure,
        Err(error) => format!(
            "{failure}; cleanup frame capture staging file {} failed: {error}",
            staging_path.display_path().display()
        ),
    }
}

#[cfg(test)]
#[path = "tests/frame_capture.rs"]
mod tests;
