use super::*;
use std::fs;

fn fixture() -> PathBuf {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let directory = std::env::temp_dir().join(format!(
        "zircon-account-config-{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    fs::create_dir(&directory).unwrap();
    directory
}

fn write_valid(path: &Path) {
    fs::write(
        path,
        serde_json::json!({
            "issuer": "https://identity.example/realm",
            "client_id": "hub",
            "service_url": "https://service.example",
            "callback_port": 8480,
            "allow_loopback_http": false
        })
        .to_string(),
    )
    .unwrap();
}

#[test]
fn config_requires_a_regular_file_and_respects_the_byte_limit() {
    let directory = fixture();
    let path = directory.join("account.json");
    write_valid(&path);
    assert!(AccountConfig::load(&path).is_ok());
    fs::create_dir(directory.join("nested")).unwrap();
    assert!(AccountConfig::load(&directory.join("nested")).is_err());
    fs::write(&path, vec![b'x'; 65537]).unwrap();
    assert!(AccountConfig::load(&path).is_err());
    fs::remove_dir_all(directory).unwrap();
}

#[cfg(unix)]
#[test]
fn config_fifo_is_rejected_without_waiting_for_a_writer() {
    use std::{ffi::CString, os::unix::ffi::OsStrExt};
    let directory = fixture();
    let path = directory.join("account.pipe");
    let name = CString::new(path.as_os_str().as_bytes()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
    assert!(AccountConfig::load(&path).is_err());
    fs::remove_dir_all(directory).unwrap();
}
