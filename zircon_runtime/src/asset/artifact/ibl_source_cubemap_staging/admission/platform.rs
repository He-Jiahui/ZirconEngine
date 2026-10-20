use std::fs::{File, Metadata, OpenOptions};
use std::io;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Stamp {
    pub(super) len: u64,
    #[cfg(windows)]
    volume: u64,
    #[cfg(windows)]
    file: [u8; 16],
    #[cfg(windows)]
    change: i64,
    #[cfg(windows)]
    written: i64,
    #[cfg(unix)]
    device: u64,
    #[cfg(unix)]
    inode: u64,
    #[cfg(unix)]
    change: (i64, i64),
    #[cfg(unix)]
    written: (i64, i64),
}

fn regular(metadata: &Metadata) -> io::Result<()> {
    #[cfg(windows)]
    let reparse = {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    };
    #[cfg(not(windows))]
    let reparse = metadata.file_type().is_symlink();
    if reparse || !metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "IBL payload is not a regular file",
        ));
    }
    Ok(())
}

fn check_path(path: &Path) -> io::Result<()> {
    for parent in path.ancestors().skip(1) {
        if parent.as_os_str().is_empty() {
            continue;
        }
        let metadata = std::fs::symlink_metadata(parent)?;
        #[cfg(windows)]
        let reparse = {
            use std::os::windows::fs::MetadataExt;
            metadata.file_attributes() & 0x400 != 0
        };
        #[cfg(not(windows))]
        let reparse = metadata.file_type().is_symlink();
        if reparse || !metadata.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "IBL parent is not a physical directory",
            ));
        }
    }
    regular(&std::fs::symlink_metadata(path)?)
}

pub(super) fn open(path: &Path, payload: bool) -> io::Result<File> {
    check_path(path)?;
    let mut options = OpenOptions::new();
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Foundation::GENERIC_READ;
        use windows_sys::Win32::Storage::FileSystem::{
            FILE_FLAG_OPEN_REPARSE_POINT, FILE_READ_ATTRIBUTES,
        };
        // Metadata-only pins do not retain a payload read handle across ReplaceFileW.
        options
            .access_mode(if payload {
                GENERIC_READ
            } else {
                FILE_READ_ATTRIBUTES
            })
            .share_mode(0x7);
        options.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
    }
    #[cfg(not(windows))]
    {
        let _ = payload;
        options.read(true);
    }
    let file = options.open(path)?;
    regular(&file.metadata()?)?;
    check_path(path)?;
    Ok(file)
}

#[cfg(windows)]
pub(super) fn stamp(file: &File) -> io::Result<Stamp> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::{
        FileBasicInfo, FileIdInfo, GetFileInformationByHandleEx, FILE_BASIC_INFO, FILE_ID_INFO,
    };
    let metadata = file.metadata()?;
    regular(&metadata)?;
    let mut id = FILE_ID_INFO::default();
    let mut basic = FILE_BASIC_INFO::default();
    // SAFETY: file owns a live handle; both repr(C) output buffers have the exact class-specific
    // size and remain writable through these synchronous calls. No foreign pointer is retained.
    let id_ok = unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle().cast(),
            FileIdInfo,
            (&mut id as *mut FILE_ID_INFO).cast(),
            std::mem::size_of::<FILE_ID_INFO>() as u32,
        )
    };
    if id_ok == 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: the same live handle and writable FILE_BASIC_INFO buffer satisfy FileBasicInfo.
    let basic_ok = unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle().cast(),
            FileBasicInfo,
            (&mut basic as *mut FILE_BASIC_INFO).cast(),
            std::mem::size_of::<FILE_BASIC_INFO>() as u32,
        )
    };
    if basic_ok == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(Stamp {
        len: metadata.len(),
        volume: id.VolumeSerialNumber,
        file: id.FileId.Identifier,
        change: basic.ChangeTime,
        written: basic.LastWriteTime,
    })
}

#[cfg(unix)]
pub(super) fn stamp(file: &File) -> io::Result<Stamp> {
    use std::os::unix::fs::MetadataExt;
    let metadata = file.metadata()?;
    regular(&metadata)?;
    Ok(Stamp {
        len: metadata.len(),
        device: metadata.dev(),
        inode: metadata.ino(),
        change: (metadata.ctime(), metadata.ctime_nsec()),
        written: (metadata.mtime(), metadata.mtime_nsec()),
    })
}

#[cfg(not(any(windows, unix)))]
pub(super) fn stamp(_file: &File) -> io::Result<Stamp> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "IBL file change identity is unsupported",
    ))
}
