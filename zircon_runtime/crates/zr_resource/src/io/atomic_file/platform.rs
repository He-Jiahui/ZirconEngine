use std::io;
use std::path::Path;

#[cfg(not(windows))]
use std::fs;

#[cfg(unix)]
pub(super) fn sync_directory(path: &Path) -> io::Result<()> {
    std::fs::File::open(path)?.sync_all()
}

#[cfg(not(unix))]
pub(super) fn sync_directory(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(unix)]
pub(super) fn sync_parent_directory_entry(path: &Path) -> io::Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    std::fs::File::open(parent)?.sync_all()
}

#[cfg(not(unix))]
pub(super) fn sync_parent_directory_entry(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(not(windows))]
pub(super) fn rename_staging(staging_path: &Path, target: &Path) -> io::Result<()> {
    fs::rename(staging_path, target)
}

#[cfg(windows)]
pub(super) fn rename_staging(staging_path: &Path, target: &Path) -> io::Result<()> {
    const MOVEFILE_WRITE_THROUGH: u32 = 0x0000_0008;

    #[link(name = "Kernel32")]
    extern "system" {
        fn MoveFileExW(
            existing_file_name: *const u16,
            new_file_name: *const u16,
            flags: u32,
        ) -> i32;
    }

    let staging_wide = windows_api_path(staging_path)?;
    let target_wide = windows_api_path(target)?;
    let moved = unsafe {
        MoveFileExW(
            staging_wide.as_ptr(),
            target_wide.as_ptr(),
            MOVEFILE_WRITE_THROUGH,
        )
    };
    if moved == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(windows)]
pub(super) fn sync_committed_target(path: &Path) -> io::Result<()> {
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)?
        .sync_all()
}

#[cfg(not(windows))]
pub(super) fn sync_committed_target(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(not(windows))]
pub(super) fn replace_existing_staged_file(staging: &Path, target: &Path) -> io::Result<()> {
    fs::rename(staging, target)
}

#[cfg(windows)]
pub(super) fn replace_existing_staged_file(staging: &Path, target: &Path) -> io::Result<()> {
    use std::ffi::c_void;

    #[link(name = "Kernel32")]
    extern "system" {
        fn ReplaceFileW(
            replaced_file_name: *const u16,
            replacement_file_name: *const u16,
            backup_file_name: *const u16,
            replace_flags: u32,
            exclude: *mut c_void,
            reserved: *mut c_void,
        ) -> i32;
    }

    let target = windows_api_path(target)?;
    let staging = windows_api_path(staging)?;
    let replaced = unsafe {
        ReplaceFileW(
            target.as_ptr(),
            staging.as_ptr(),
            std::ptr::null(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if replaced == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(windows)]
fn windows_api_path(path: &Path) -> io::Result<Vec<u16>> {
    const LEGACY_MAX_PATH: usize = 260;
    const SEPARATOR: u16 = b'\\' as u16;
    const VERBATIM_PREFIX: &[u16] = &[SEPARATOR, SEPARATOR, b'?' as u16, SEPARATOR];
    const DEVICE_PREFIX: &[u16] = &[SEPARATOR, SEPARATOR, b'.' as u16, SEPARATOR];
    const UNC_PREFIX: &[u16] = &[SEPARATOR, SEPARATOR];
    const VERBATIM_UNC_PREFIX: &[u16] = &[
        SEPARATOR,
        SEPARATOR,
        b'?' as u16,
        SEPARATOR,
        b'U' as u16,
        b'N' as u16,
        b'C' as u16,
        SEPARATOR,
    ];

    let encoded = encode_nul_terminated_path(path);
    let path_units = &encoded[..encoded.len() - 1];
    if path_units.contains(&0) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Windows file path contains an embedded NUL",
        ));
    }
    if path_units.starts_with(VERBATIM_PREFIX)
        || path_units.starts_with(DEVICE_PREFIX)
        || (path.is_absolute() && path_units.len() < LEGACY_MAX_PATH)
    {
        return Ok(encoded);
    }

    let absolute = std::path::absolute(path)?;
    let absolute_encoded = encode_nul_terminated_path(&absolute);
    let absolute_units = &absolute_encoded[..absolute_encoded.len() - 1];
    if absolute_units.len() < LEGACY_MAX_PATH {
        return Ok(encoded);
    }

    let mut wide = Vec::with_capacity(absolute_units.len() + VERBATIM_UNC_PREFIX.len() + 1);
    if absolute_units.starts_with(UNC_PREFIX) {
        wide.extend_from_slice(VERBATIM_UNC_PREFIX);
        wide.extend_from_slice(&absolute_units[UNC_PREFIX.len()..]);
    } else {
        wide.extend_from_slice(VERBATIM_PREFIX);
        wide.extend_from_slice(absolute_units);
    }
    wide.push(0);
    Ok(wide)
}

#[cfg(windows)]
fn encode_nul_terminated_path(path: &Path) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;

    path.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

#[cfg(all(test, windows))]
#[path = "platform/windows_path_encoding_tests.rs"]
mod windows_path_encoding_tests;
