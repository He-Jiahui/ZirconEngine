use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use crate::asset::safe_project_path::is_link_or_reparse;

pub(crate) fn open_admitted_file(path: &Path, root: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
        options.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
    }
    let file = options.open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || is_link_or_reparse(&metadata) {
        return Err(io::Error::other(
            "opened auxiliary source is not a non-reparse regular file",
        ));
    }
    // 从已打开句柄反查物理路径，再复核它仍等于准入路径且位于资源根内。
    let opened_path = final_path(&file)?;
    if !path_is_within(&opened_path, root) || !paths_equal(&opened_path, path) {
        return Err(io::Error::other(format!(
            "opened auxiliary source {} escapes or differs from admitted path {}",
            opened_path.display(),
            path.display()
        )));
    }
    // Both identities retain open handles. A later pathname replacement cannot retarget either.
    let opened_identity = same_file::Handle::from_file(file.try_clone()?)?;
    let named_identity = same_file::Handle::from_path(path)?;
    if opened_identity != named_identity {
        return Err(io::Error::other(
            "auxiliary source identity changed during open",
        ));
    }
    Ok(file)
}

fn paths_equal(left: &Path, right: &Path) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        #[link(name = "kernel32")]
        extern "system" {
            fn CompareStringOrdinal(
                left: *const u16,
                left_len: i32,
                right: *const u16,
                right_len: i32,
                ignore_case: i32,
            ) -> i32;
        }
        let normalize = |path: &Path| {
            crate::asset::project::ProjectPaths::display_path(path)
                .as_os_str()
                .encode_wide()
                .collect::<Vec<_>>()
        };
        let left = normalize(left);
        let right = normalize(right);
        let (Ok(left_len), Ok(right_len)) = (i32::try_from(left.len()), i32::try_from(right.len()))
        else {
            return false;
        };
        // Ordinal comparison matches Windows path case semantics without reopening either path.
        // SAFETY: 两个 Vec<u16> 在调用期间存活；长度已转成 i32 并界定可读范围，API 同步读取这两段。
        return unsafe {
            CompareStringOrdinal(left.as_ptr(), left_len, right.as_ptr(), right_len, 1) == 2
        };
    }
    #[cfg(not(windows))]
    return left == right;
}

pub(super) fn path_is_within(path: &Path, root: &Path) -> bool {
    path.ancestors().any(|ancestor| paths_equal(ancestor, root))
}

#[cfg(windows)]
fn final_path(file: &File) -> io::Result<PathBuf> {
    use std::ffi::{c_void, OsString};
    use std::os::windows::ffi::OsStringExt;
    use std::os::windows::io::AsRawHandle;

    #[link(name = "kernel32")]
    extern "system" {
        fn GetFinalPathNameByHandleW(
            file: *mut c_void,
            path: *mut u16,
            capacity: u32,
            flags: u32,
        ) -> u32;
    }
    // The API returns an absolute DOS path with the same verbatim prefix as ProjectPaths.
    // TODO: [CR-ASSET-IMPORT-FFI-0001] 确认零容量探测允许空输出指针；SDK 未标注参数可空，需平台契约或独立 FFI 实测补证。
    let required =
        unsafe { GetFinalPathNameByHandleW(file.as_raw_handle(), std::ptr::null_mut(), 0, 0) };
    if required == 0 {
        return Err(io::Error::last_os_error());
    }
    let mut path = vec![0_u16; required as usize];
    // SAFETY: file 仍有效，path 提供 required 个可写 UTF-16 单元，与传入容量一致。
    let written =
        unsafe { GetFinalPathNameByHandleW(file.as_raw_handle(), path.as_mut_ptr(), required, 0) };
    if written == 0 {
        return Err(io::Error::last_os_error());
    }
    if written >= required {
        return Err(io::Error::other(
            "opened auxiliary path changed while queried",
        ));
    }
    path.truncate(written as usize);
    Ok(PathBuf::from(OsString::from_wide(&path)))
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn final_path(file: &File) -> io::Result<PathBuf> {
    use std::os::fd::AsRawFd;
    std::fs::read_link(format!("/proc/self/fd/{}", file.as_raw_fd()))
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
fn final_path(file: &File) -> io::Result<PathBuf> {
    use std::os::fd::AsRawFd;
    use std::os::unix::ffi::OsStringExt;
    extern "C" {
        fn fcntl(fd: i32, command: i32, ...) -> i32;
    }
    const F_GETPATH: i32 = 50;
    const MAXPATHLEN: usize = 1024;
    let mut bytes = [0_u8; MAXPATHLEN];
    // SAFETY: fd 来自仍存活的 File；bytes 是 F_GETPATH 要求的 MAXPATHLEN 可写缓冲区。
    let result = unsafe { fcntl(file.as_raw_fd(), F_GETPATH, bytes.as_mut_ptr()) };
    if result != 0 {
        return Err(io::Error::last_os_error());
    }
    let end = bytes
        .iter()
        .position(|value| *value == 0)
        .ok_or_else(|| io::Error::other("opened auxiliary path was not terminated"))?;
    Ok(PathBuf::from(std::ffi::OsString::from_vec(
        bytes[..end].to_vec(),
    )))
}

#[cfg(not(any(
    windows,
    target_os = "linux",
    target_os = "android",
    target_os = "macos",
    target_os = "ios"
)))]
fn final_path(_file: &File) -> io::Result<PathBuf> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "opened auxiliary path validation is unavailable on this platform",
    ))
}
