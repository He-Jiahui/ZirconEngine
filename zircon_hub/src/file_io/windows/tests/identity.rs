use super::*;
use std::{fs, time::SystemTime};

#[test]
fn open_handle_identity_distinguishes_equal_content_files_and_hard_links() {
    let root = std::env::temp_dir().join(format!(
        "hub-file-identity-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    let first_path = root.join("first");
    let second_path = root.join("second");
    let link_path = root.join("link");
    fs::write(&first_path, b"same bytes").unwrap();
    fs::write(&second_path, b"same bytes").unwrap();
    fs::hard_link(&first_path, &link_path).unwrap();
    let first = File::open(&first_path).unwrap();
    let second = File::open(&second_path).unwrap();
    let link = File::open(&link_path).unwrap();
    assert!(same_file(&first, &first.try_clone().unwrap()).unwrap());
    assert!(same_file(&first, &link).unwrap());
    assert!(!same_file(&first, &second).unwrap());
    drop((first, second, link));
    fs::remove_dir_all(root).unwrap();
}
