use std::{ffi::c_void, fs::File, io, os::windows::io::AsRawHandle};

// FILE_INFO_BY_HANDLE_CLASS::FileIdInfo returns the full 128-bit identity,
// including on filesystems whose IDs do not fit BY_HANDLE_FILE_INFORMATION.
const FILE_ID_INFO_CLASS: u32 = 18;

#[repr(C)]
#[derive(Default, PartialEq, Eq)]
struct FileIdentity {
    volume_serial_number: u64,
    file_id: [u8; 16],
}

fn identity(file: &File) -> io::Result<FileIdentity> {
    let mut information = FileIdentity::default();
    // Both the borrowed handle and ABI-compatible output buffer remain valid
    // for this synchronous call. Failure preserves the Windows error.
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle(),
            FILE_ID_INFO_CLASS,
            (&mut information as *mut FileIdentity).cast(),
            std::mem::size_of::<FileIdentity>() as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(information)
}

pub(crate) fn same_file(left: &File, right: &File) -> io::Result<bool> {
    Ok(identity(left)? == identity(right)?)
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetFileInformationByHandleEx(
        handle: *mut c_void,
        information_class: u32,
        information: *mut c_void,
        size: u32,
    ) -> i32;
}

#[cfg(test)]
#[path = "tests/identity.rs"]
mod tests;
