use super::*;
use std::{
    fs,
    io::{Read, Write},
    path::PathBuf,
};

struct Fixture {
    root: PathBuf,
    junction: PathBuf,
    held: Option<File>,
}

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "hub-no-reparse-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        Self {
            junction: root.join("parent"),
            root,
            held: None,
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.held.take();
        let _ = fs::remove_dir(&self.junction);
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn open(path: &Path, disposition: u32, directory: bool) -> io::Result<File> {
    unsafe {
        open_no_reparse(
            path,
            if directory { 0x80 } else { 0xC000_0000 },
            3,
            disposition,
            directory,
            std::ptr::null_mut(),
        )
    }
}

#[test]
fn native_path_conversion_keeps_drive_unc_and_volume_roots_anchored() {
    let cases = [
        (r"E:\project", r"\??\E:\project"),
        (r"\\?\E:\project", r"\??\E:\project"),
        (r"\\server\share\project", r"\??\UNC\server\share\project"),
        (
            r"\\?\UNC\server\share\project",
            r"\??\UNC\server\share\project",
        ),
        (
            r"\\?\Volume{01234567-89ab-cdef-0123-456789abcdef}\project",
            r"\??\Volume{01234567-89ab-cdef-0123-456789abcdef}\project",
        ),
    ];

    for (input, expected) in cases {
        let encoded = native_object_path(Path::new(input)).unwrap();
        assert_eq!(String::from_utf16(&encoded).unwrap(), expected, "{input}");
    }
    assert!(native_object_path(Path::new(r"\\.\PhysicalDrive0")).is_err());
}

#[test]
fn no_reparse_open_rejects_a_junction_installed_after_parent_admission() {
    let mut fixture = Fixture::new();
    let target = fixture.root.join("target");
    fs::create_dir(&target).unwrap();
    fixture.held = Some(open(&fixture.junction, FILE_OPEN_IF, true).unwrap());
    set_junction(&fixture.junction, &target).unwrap();
    assert!(open(
        &fixture.junction.join("redirected.dat"),
        FILE_OPEN_IF,
        false
    )
    .is_err());
    assert!(!target.join("redirected.dat").exists());
    assert!(open(
        &fixture.junction.join("redirected-directory"),
        FILE_OPEN_IF,
        true
    )
    .is_err());
    assert!(!target.join("redirected-directory").exists());
    // Confirm that the fixture redirects an ordinary path open.
    fs::write(fixture.junction.join("control"), b"redirected").unwrap();
    assert_eq!(fs::read(target.join("control")).unwrap(), b"redirected");
}

#[test]
fn no_reparse_open_preserves_existing_data_and_supports_normal_creation() {
    let fixture = Fixture::new();
    let path = fixture.root.join("journal");
    let mut file = open(&path, FILE_CREATE, false).unwrap();
    use std::io::Write;
    file.write_all(b"preserved").unwrap();
    file.sync_all().unwrap();
    drop(file);
    drop(open(&path, FILE_OPEN_IF, false).unwrap());
    assert_eq!(fs::read(&path).unwrap(), b"preserved");
    assert!(open(&path, FILE_CREATE, false).is_err());
    assert!(open(&path.with_extension("dat:stream"), FILE_OPEN_IF, false).is_err());
}

#[test]
fn private_relative_entries_use_protected_owner_and_system_acl() {
    let fixture = Fixture::new();
    let parent = unsafe {
        open_no_reparse(
            &fixture.root,
            0x80 | 0x2 | 0x4, // FILE_READ_ATTRIBUTES | FILE_ADD_FILE | FILE_ADD_SUBDIRECTORY
            3,
            FILE_OPEN,
            true,
            std::ptr::null_mut(),
        )
    }
    .unwrap();
    drop(create_relative_private_directory(&parent, OsStr::new("private")).unwrap());
    drop(create_relative_private_file(&parent, OsStr::new("secret")).unwrap());
    let directory = open_relative(
        &parent,
        OsStr::new("private"),
        0x0002_0000, // READ_CONTROL
        Some(true),
        3,
    )
    .unwrap();
    let file = open_relative(
        &parent,
        OsStr::new("secret"),
        0x0002_0000, // READ_CONTROL
        Some(false),
        3,
    )
    .unwrap();

    assert_eq!(
        protected_acl_summary(&directory).unwrap(),
        (true, vec!["S-1-3-4".to_owned(), "S-1-5-18".to_owned()])
    );
    assert_eq!(
        protected_acl_summary(&file).unwrap(),
        (true, vec!["S-1-3-4".to_owned(), "S-1-5-18".to_owned()])
    );

    drop(create_relative_directory(&parent, OsStr::new("legacy")).unwrap());
    let legacy = open_relative(
        &parent,
        OsStr::new("legacy"),
        0x80 | 0x0004_0000, // FILE_READ_ATTRIBUTES | WRITE_DAC
        Some(true),
        3,
    )
    .unwrap();
    protect_private_object(&legacy).unwrap();
    drop(legacy);
    let legacy = open_relative(
        &parent,
        OsStr::new("legacy"),
        0x0002_0000, // READ_CONTROL
        Some(true),
        3,
    )
    .unwrap();
    assert_eq!(
        protected_acl_summary(&legacy).unwrap(),
        (true, vec!["S-1-3-4".to_owned(), "S-1-5-18".to_owned()])
    );
}

#[test]
fn opening_a_legacy_private_file_replaces_its_broad_dacl_before_reading() {
    use crate::file_io::AnchoredDirectory;

    let fixture = Fixture::new();
    let parent = unsafe {
        open_no_reparse(
            &fixture.root,
            0x80 | 0x2 | 0x4,
            3,
            FILE_OPEN,
            true,
            std::ptr::null_mut(),
        )
    }
    .unwrap();
    let mut created = create_relative_file(&parent, OsStr::new("legacy-marker")).unwrap();
    created.write_all(b"legacy private marker").unwrap();
    drop(created);
    let legacy = open_relative(
        &parent,
        OsStr::new("legacy-marker"),
        0x1 | 0x80 | 0x0002_0000 | 0x0004_0000,
        Some(false),
        3,
    )
    .unwrap();
    set_dacl_sddl(&legacy, "D:P(A;;FA;;;WD)").unwrap();
    assert_eq!(
        protected_acl_summary(&legacy).unwrap(),
        (true, vec!["S-1-1-0".to_owned()])
    );
    drop(legacy);
    drop(parent);

    let anchor = AnchoredDirectory::open(&fixture.root).unwrap();
    let mut protected = anchor
        .open_existing_private_file(OsStr::new("legacy-marker"))
        .unwrap();
    let mut bytes = Vec::new();
    protected.read_to_end(&mut bytes).unwrap();
    let acl_check = unsafe {
        open_no_reparse(
            &fixture.root.join("legacy-marker"),
            0x0002_0000, // READ_CONTROL
            3,
            FILE_OPEN,
            false,
            std::ptr::null_mut(),
        )
    }
    .unwrap();

    assert_eq!(bytes, b"legacy private marker");
    assert_eq!(
        protected_acl_summary(&acl_check).unwrap(),
        (true, vec!["S-1-3-4".to_owned(), "S-1-5-18".to_owned()])
    );
}

fn set_dacl_sddl(file: &File, sddl: &str) -> io::Result<()> {
    let mut encoded: Vec<u16> = format!("{sddl}\0").encode_utf16().collect();
    let mut descriptor = std::ptr::null_mut();
    if unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            encoded.as_mut_ptr(),
            1,
            &mut descriptor,
            std::ptr::null_mut(),
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if descriptor.is_null() {
        return Err(io::Error::other("test security descriptor missing"));
    }
    let status = unsafe {
        SetKernelObjectSecurity(
            file.as_raw_handle(),
            4 | 0x8000_0000, // DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION
            descriptor,
        )
    };
    unsafe { LocalFree(descriptor) };
    if status == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn protected_acl_summary(file: &File) -> io::Result<(bool, Vec<String>)> {
    use std::ffi::c_void;
    let mut dacl = std::ptr::null_mut::<AclPlaceholder>();
    let mut descriptor = std::ptr::null_mut::<c_void>();
    let status = unsafe {
        GetSecurityInfo(
            file.as_raw_handle(),
            1,
            4, // DACL_SECURITY_INFORMATION
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut dacl,
            std::ptr::null_mut(),
            &mut descriptor,
        )
    };
    if status != 0 {
        return Err(io::Error::from_raw_os_error(status as i32));
    }
    if descriptor.is_null() || dacl.is_null() {
        if !descriptor.is_null() {
            unsafe { LocalFree(descriptor) };
        }
        return Err(io::Error::other("private DACL missing"));
    }
    let mut control = 0;
    let mut revision = 0;
    let result = unsafe { GetSecurityDescriptorControl(descriptor, &mut control, &mut revision) };
    if result == 0 {
        unsafe { LocalFree(descriptor) };
        return Err(io::Error::last_os_error());
    }
    let ace_count = unsafe { (*dacl).ace_count };
    let mut sids = Vec::with_capacity(ace_count as usize);
    for index in 0..u32::from(ace_count) {
        let mut ace = std::ptr::null_mut::<c_void>();
        if unsafe { GetAce(dacl, index, &mut ace) } == 0 || ace.is_null() {
            unsafe { LocalFree(descriptor) };
            return Err(io::Error::last_os_error());
        }
        if unsafe { *ace.cast::<u8>() } != 0 {
            unsafe { LocalFree(descriptor) };
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unexpected ACE type",
            ));
        }
        let sid = unsafe { ace.cast::<u8>().add(8).cast::<c_void>() };
        let mut sid_string = std::ptr::null_mut::<u16>();
        if unsafe { ConvertSidToStringSidW(sid, &mut sid_string) } == 0 || sid_string.is_null() {
            unsafe { LocalFree(descriptor) };
            return Err(io::Error::last_os_error());
        }
        let mut length = 0;
        while unsafe { *sid_string.add(length) } != 0 {
            length += 1;
        }
        let sid_text = unsafe { std::slice::from_raw_parts(sid_string, length) };
        sids.push(String::from_utf16_lossy(sid_text));
        unsafe { LocalFree(sid_string.cast::<c_void>()) };
    }
    unsafe { LocalFree(descriptor) };
    sids.sort();
    Ok((control & 0x1000 != 0, sids))
}

#[repr(C)]
struct AclPlaceholder {
    revision: u8,
    reserved: u8,
    size: u16,
    ace_count: u16,
    reserved2: u16,
}

#[link(name = "advapi32")]
unsafe extern "system" {
    fn ConvertSidToStringSidW(sid: *mut std::ffi::c_void, output: *mut *mut u16) -> i32;
    fn GetSecurityInfo(
        handle: *mut std::ffi::c_void,
        object_type: u32,
        information: u32,
        owner: *mut *mut std::ffi::c_void,
        group: *mut *mut std::ffi::c_void,
        dacl: *mut *mut AclPlaceholder,
        sacl: *mut *mut AclPlaceholder,
        descriptor: *mut *mut std::ffi::c_void,
    ) -> u32;
    fn GetSecurityDescriptorControl(
        descriptor: *mut std::ffi::c_void,
        control: *mut u16,
        revision: *mut u32,
    ) -> i32;
    fn GetAce(acl: *const AclPlaceholder, index: u32, ace: *mut *mut std::ffi::c_void) -> i32;
}

fn set_junction(path: &Path, target: &Path) -> io::Result<()> {
    let path = junction_tool_path(path);
    let target = junction_tool_path(target);
    let output = std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "$ErrorActionPreference='Stop'; New-Item -ItemType Junction -Path $env:ZIRCON_HUB_JUNCTION_PATH -Target $env:ZIRCON_HUB_JUNCTION_TARGET | Out-Null",
        ])
        .env("ZIRCON_HUB_JUNCTION_PATH", path)
        .env("ZIRCON_HUB_JUNCTION_TARGET", target)
        .output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(io::Error::other(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ))
    }
}

fn junction_tool_path(path: &Path) -> PathBuf {
    // A junction substitute name already receives the NT object prefix from
    // PowerShell; do not nest an extended `\\?\` prefix inside it.
    let value = path.to_string_lossy();
    if let Some(rest) = value.strip_prefix("\\\\?\\UNC\\") {
        PathBuf::from(format!("\\\\{rest}"))
    } else if let Some(rest) = value.strip_prefix("\\\\?\\") {
        PathBuf::from(rest)
    } else {
        path.to_path_buf()
    }
}
