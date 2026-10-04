use std::fs;

use super::super::{directory_permissions, directory_sddl};
use super::*;

#[test]
fn recovery_rejects_null_dacl_without_changing_permissions_or_retiring_evidence() {
    let root = std::env::temp_dir().join(format!(
        "cargo-zircon-null-dacl-recovery-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let source = root.join("source");
    fs::create_dir_all(&source).unwrap();
    let nested = source.join("nested");
    fs::create_dir(&nested).unwrap();
    let original_sddl = directory_sddl(&source).unwrap();
    let original_permissions = directory_permissions(&source).unwrap();
    let nested_sddl = directory_sddl(&nested).unwrap();
    let nested_permissions = directory_permissions(&nested).unwrap();
    let journal = NamespaceJournal::acquire(&source).unwrap();
    let record = RecoveryRecord {
        version: JOURNAL_VERSION,
        relative_path: String::new(),
        identity: directory_identity(&journal._root_lease).unwrap(),
        original_sddl: "D:NO_ACCESS_CONTROL".to_string(),
    };
    let mut bytes = serde_json::to_vec(&record).unwrap();
    bytes.push(b'\n');
    let directory = open_directory(&nested, GENERIC_READ, FILE_SHARE_READ).unwrap();
    let nested_record = RecoveryRecord {
        version: JOURNAL_VERSION,
        relative_path: "nested".to_string(),
        identity: directory_identity(&directory).unwrap(),
        original_sddl: "D:P(A;;FA;;;WD)".to_string(),
    };
    drop(directory);
    bytes.extend(serde_json::to_vec(&nested_record).unwrap());
    bytes.push(b'\n');
    journal.file.borrow_mut().write_all(&bytes).unwrap();
    journal.file.borrow().sync_all().unwrap();
    journal.retain_for_recovery();
    drop(journal);
    let journal_path = fs::read_dir(&root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| path.extension().is_some_and(|value| value == "jsonl"))
        .unwrap();

    let error = NamespaceJournal::acquire(&source).err();
    let recovered_permissions = directory_permissions(&source);
    let recovered_nested_permissions = directory_permissions(&nested);
    let retained = fs::read(&journal_path).ok();

    // Restore the fixture even when exercising the pre-fix null-DACL path.
    let directory = open_directory(&source, GENERIC_READ | WRITE_DAC, FILE_SHARE_READ).unwrap();
    restore_sddl(&directory, &original_sddl).unwrap();
    drop(directory);
    let directory = open_directory(&nested, GENERIC_READ | WRITE_DAC, FILE_SHARE_READ).unwrap();
    restore_sddl(&directory, &nested_sddl).unwrap();
    drop(directory);
    fs::remove_dir_all(&root).unwrap();

    assert!(error.unwrap().to_string().contains("null DACL"));
    assert_eq!(recovered_permissions.unwrap(), original_permissions);
    assert_eq!(recovered_nested_permissions.unwrap(), nested_permissions);
    assert_eq!(retained.as_deref(), Some(bytes.as_slice()));
}
