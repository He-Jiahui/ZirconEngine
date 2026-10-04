use std::path::Path;

/// Gives Windows test fixtures the same protected ACL required by the service.
#[cfg(windows)]
pub(crate) fn make_private_directory(path: &Path) {
    let sid_output = std::process::Command::new("whoami.exe")
        .args(["/user", "/fo", "csv", "/nh"])
        .output()
        .expect("whoami must be available for the Windows ACL fixture");
    assert!(
        sid_output.status.success(),
        "whoami must identify the test user"
    );
    let sid_line = String::from_utf8_lossy(&sid_output.stdout);
    let sid = sid_line
        .split('"')
        .nth(3)
        .expect("whoami output must contain the user SID");

    // `Set-Acl` is absent in some PowerShell Core installations. `icacls` is
    // built into Windows and expresses the same protected owner/SYSTEM ACL.
    let inheritance = std::process::Command::new("icacls.exe")
        .args([path.as_os_str(), std::ffi::OsStr::new("/inheritance:r")])
        .output()
        .expect("icacls must be available for the Windows ACL fixture");
    assert!(
        inheritance.status.success(),
        "private ACL inheritance must be disabled: {}",
        String::from_utf8_lossy(&inheritance.stderr)
    );

    let owner = format!("*{sid}:(OI)(CI)(F)");
    let user_grant = std::process::Command::new("icacls.exe")
        .args([
            path.as_os_str(),
            std::ffi::OsStr::new("/grant:r"),
            owner.as_ref(),
        ])
        .output()
        .expect("icacls must grant the test user access");
    assert!(
        user_grant.status.success(),
        "private ACL owner grant must succeed: {}",
        String::from_utf8_lossy(&user_grant.stderr)
    );

    let system_grant = std::process::Command::new("icacls.exe")
        .args([
            path.as_os_str(),
            std::ffi::OsStr::new("/grant:r"),
            std::ffi::OsStr::new("*S-1-5-18:(OI)(CI)(F)"),
        ])
        .output()
        .expect("icacls must grant SYSTEM access");
    assert!(
        system_grant.status.success(),
        "private ACL SYSTEM grant must succeed: {}",
        String::from_utf8_lossy(&system_grant.stderr)
    );

    let verify = std::process::Command::new("icacls.exe")
        .arg(path.as_os_str())
        .output()
        .expect("icacls must verify the Windows ACL fixture");
    assert!(
        verify.status.success(),
        "private ACL fixture must be readable: {}",
        String::from_utf8_lossy(&verify.stderr)
    );
}

#[cfg(not(windows))]
pub(crate) fn make_private_directory(_path: &Path) {}
