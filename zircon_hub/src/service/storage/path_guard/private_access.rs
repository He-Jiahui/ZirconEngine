use std::{
    ffi::c_void,
    fs::File,
    io,
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
};

pub(super) const READ_CONTROL: u32 = 0x0002_0000;
const SE_DACL_PROTECTED: u16 = 0x1000;

#[repr(C)]
struct TokenUser {
    sid: *mut c_void,
    attributes: u32,
}

#[repr(C)]
struct Acl {
    revision: u8,
    reserved: u8,
    size: u16,
    count: u16,
    reserved2: u16,
}

#[repr(C)]
struct AceHeader {
    kind: u8,
    flags: u8,
    size: u16,
}

struct SecurityDescriptor(*mut c_void);

impl Drop for SecurityDescriptor {
    fn drop(&mut self) {
        unsafe { LocalFree(self.0) };
    }
}

pub(super) struct Identity {
    token_user: Vec<usize>,
}

impl Identity {
    pub(super) fn current() -> io::Result<Self> {
        let mut handle = std::ptr::null_mut();
        if unsafe { OpenProcessToken(GetCurrentProcess(), 8, &mut handle) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let token = unsafe { OwnedHandle::from_raw_handle(handle) };
        // TOKEN_USER contains one SID with at most 15 subauthorities. Word alignment is required.
        let mut token_user = vec![0usize; 32];
        let mut length = 0;
        if unsafe {
            GetTokenInformation(
                token.as_raw_handle(),
                1,
                token_user.as_mut_ptr().cast(),
                (token_user.len() * std::mem::size_of::<usize>()) as u32,
                &mut length,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        if (length as usize) < std::mem::size_of::<TokenUser>() {
            return Err(super::invalid_path());
        }
        Ok(Self { token_user })
    }

    /// Checks the owner and all grants, including inherit-only grants for future SQLite sidecars.
    /// Returns whether inheritance stops at this object.
    pub(super) fn require_private(&self, file: &File) -> io::Result<bool> {
        let mut owner = std::ptr::null_mut();
        let mut acl: *mut Acl = std::ptr::null_mut();
        let mut descriptor = std::ptr::null_mut();
        let result = unsafe {
            GetSecurityInfo(
                file.as_raw_handle(),
                1,
                1 | 4,
                &mut owner,
                std::ptr::null_mut(),
                &mut acl,
                std::ptr::null_mut(),
                &mut descriptor,
            )
        };
        if result != 0 {
            return Err(io::Error::from_raw_os_error(result as i32));
        }
        let descriptor = SecurityDescriptor(descriptor);
        if descriptor.0.is_null() || owner.is_null() || acl.is_null() {
            return Err(super::invalid_path());
        }
        // All pointers below belong to the OS-allocated descriptor, retained for this scope.
        unsafe {
            if !self.trusted_sid(owner) || IsValidAcl(acl) == 0 {
                return Err(super::invalid_path());
            }
            for index in 0..u32::from((*acl).count) {
                let mut entry = std::ptr::null_mut();
                if GetAce(acl, index, &mut entry) == 0 {
                    return Err(io::Error::last_os_error());
                }
                let header = &*entry.cast::<AceHeader>();
                match header.kind {
                    0 if header.size >= 16 => {
                        let sid = entry.cast::<u8>().add(8).cast();
                        if !self.trusted_sid(sid) {
                            return Err(super::invalid_path());
                        }
                    }
                    1 => {} // Denials cannot grant access.
                    _ => return Err(super::invalid_path()),
                }
            }
            let mut control = 0;
            let mut revision = 0;
            if GetSecurityDescriptorControl(descriptor.0, &mut control, &mut revision) == 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(control & SE_DACL_PROTECTED != 0)
        }
    }

    unsafe fn trusted_sid(&self, sid: *mut c_void) -> bool {
        let user = unsafe { &*self.token_user.as_ptr().cast::<TokenUser>() };
        unsafe {
            EqualSid(sid, user.sid) != 0
                || IsWellKnownSid(sid, 22) != 0 // LocalSystem
                || IsWellKnownSid(sid, 26) != 0 // Administrators are within the OS trust boundary.
        }
    }
}

#[link(name = "advapi32")]
unsafe extern "system" {
    fn OpenProcessToken(process: *mut c_void, access: u32, token: *mut *mut c_void) -> i32;
    fn GetTokenInformation(
        token: *mut c_void,
        class: u32,
        buffer: *mut c_void,
        length: u32,
        returned: *mut u32,
    ) -> i32;
    fn GetSecurityInfo(
        handle: *mut c_void,
        object_type: u32,
        information: u32,
        owner: *mut *mut c_void,
        group: *mut *mut c_void,
        dacl: *mut *mut Acl,
        sacl: *mut *mut Acl,
        descriptor: *mut *mut c_void,
    ) -> u32;
    fn GetAce(acl: *mut Acl, index: u32, ace: *mut *mut c_void) -> i32;
    fn IsValidAcl(acl: *mut Acl) -> i32;
    fn EqualSid(left: *mut c_void, right: *mut c_void) -> i32;
    fn IsWellKnownSid(sid: *mut c_void, kind: u32) -> i32;
    fn GetSecurityDescriptorControl(
        descriptor: *mut c_void,
        control: *mut u16,
        revision: *mut u32,
    ) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentProcess() -> *mut c_void;
    fn LocalFree(memory: *mut c_void) -> *mut c_void;
}
