use std::fs::{File, Metadata};
use std::io;
use std::path::Path;

#[cfg(windows)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FileIdentity {
    volume: u64,
    file: [u8; 16],
}

#[cfg(windows)]
pub(super) fn identity(file: &File) -> io::Result<FileIdentity> {
    use std::ffi::c_void;
    use std::os::windows::io::AsRawHandle;

    #[repr(C)]
    struct FileIdInfo {
        volume: u64,
        file: [u8; 16],
    }
    #[link(name = "Kernel32")]
    extern "system" {
        fn GetFileInformationByHandleEx(
            handle: *mut c_void,
            class: u32,
            info: *mut c_void,
            size: u32,
        ) -> i32;
    }
    let mut info = FileIdInfo {
        volume: 0,
        file: [0; 16],
    };
    // SAFETY: the live File owns the handle; FileIdInfo matches FILE_ID_INFO, and its writable
    // buffer stays valid for the synchronous call. FileIdInfo is class 0x12 in the Windows API.
    let ok = unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle(),
            0x12,
            (&mut info as *mut FileIdInfo).cast(),
            std::mem::size_of::<FileIdInfo>() as u32,
        )
    };
    if ok == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(FileIdentity {
        volume: info.volume,
        file: info.file,
    })
}

#[cfg(windows)]
pub(super) fn pin(path: &Path, file: &File) -> io::Result<File> {
    use std::os::windows::fs::OpenOptionsExt;
    let object = std::fs::OpenOptions::new()
        .access_mode(0)
        .share_mode(0x7)
        .open(path)?;
    if identity(&object)? != identity(file)? {
        return Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "source path changed before identity pin",
        ));
    }
    Ok(object)
}

#[cfg(windows)]
pub(super) fn is_reparse(metadata: &Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    metadata.file_attributes() & 0x400 != 0
}

#[cfg(unix)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FileIdentity {
    device: u64,
    inode: u64,
}

#[cfg(unix)]
pub(super) fn identity(file: &File) -> io::Result<FileIdentity> {
    use std::os::unix::fs::MetadataExt;
    let metadata = file.metadata()?;
    Ok(FileIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
    })
}

#[cfg(unix)]
pub(super) fn pin(path: &Path, file: &File) -> io::Result<File> {
    let current = File::open(path)?;
    if identity(&current)? != identity(file)? {
        return Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "source path changed before identity pin",
        ));
    }
    file.try_clone()
}

#[cfg(unix)]
pub(super) fn is_reparse(metadata: &Metadata) -> bool {
    metadata.file_type().is_symlink()
}

#[cfg(not(any(unix, windows)))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FileIdentity;

#[cfg(not(any(unix, windows)))]
pub(super) fn identity(_file: &File) -> io::Result<FileIdentity> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "source object identity is unsupported on this platform",
    ))
}

#[cfg(not(any(unix, windows)))]
pub(super) fn pin(_path: &Path, _file: &File) -> io::Result<File> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "source object identity is unsupported on this platform",
    ))
}

#[cfg(not(any(unix, windows)))]
pub(super) fn is_reparse(metadata: &Metadata) -> bool {
    metadata.file_type().is_symlink()
}
