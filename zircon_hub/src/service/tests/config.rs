use super::*;

#[test]
fn bounded_read_rejects_an_oversized_file() {
    let path = std::env::temp_dir().join(format!("hub-config-{}.test", uuid::Uuid::new_v4()));
    std::fs::write(&path, [b'x'; 4097]).unwrap();
    assert!(matches!(
        read_bounded(&path, 4096),
        Err(ServiceError::Configuration)
    ));
    assert_eq!(read_bounded(&path, 4097).unwrap().len(), 4097);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn secure_bounded_read_requires_a_regular_file() {
    let directory = std::env::temp_dir().join(format!("hub-config-{}.dir", uuid::Uuid::new_v4()));
    std::fs::create_dir(&directory).unwrap();
    assert!(matches!(
        read_bounded_regular(&directory, 4096),
        Err(ServiceError::Configuration)
    ));
    std::fs::remove_dir(directory).unwrap();
}

#[cfg(unix)]
#[test]
fn secure_bounded_read_rejects_a_symlink_target() {
    let target = std::env::temp_dir().join(format!("hub-secret-{}.txt", uuid::Uuid::new_v4()));
    let link = std::env::temp_dir().join(format!("hub-secret-{}.link", uuid::Uuid::new_v4()));
    std::fs::write(&target, b"secret").unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();
    assert!(matches!(
        read_bounded_regular(&link, 4096),
        Err(ServiceError::Configuration)
    ));
    std::fs::remove_file(link).unwrap();
    std::fs::remove_file(target).unwrap();
}
