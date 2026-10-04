use super::*;

struct OriginalDacl {
    directory: File,
    sddl: String,
}

impl Drop for OriginalDacl {
    fn drop(&mut self) {
        journal::restore_sddl(&self.directory, &self.sddl).expect("test DACL should restore");
    }
}

#[test]
fn null_dacl_is_rejected_without_changing_directory_permissions() {
    assert_null_dacl_rejected(false);
    assert_null_dacl_rejected(true);
}

fn assert_null_dacl_rejected(nested: bool) {
    let root = std::env::temp_dir().join(format!(
        "cargo-zircon-null-dacl-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    let path = root.join("source");
    fs::create_dir(&path).unwrap();
    let original = OriginalDacl {
        sddl: directory_sddl(&path).unwrap(),
        directory: OpenOptions::new()
            .read(true)
            .access_mode(GENERIC_READ | WRITE_DAC)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(&path)
            .unwrap(),
    };
    status(unsafe {
        SetSecurityInfo(
            original.directory.as_raw_handle(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION
                | if nested {
                    UNPROTECTED_DACL_SECURITY_INFORMATION
                } else {
                    PROTECTED_DACL_SECURITY_INFORMATION
                },
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
        )
    })
    .unwrap();
    let null_sddl = directory_sddl(&path).unwrap();
    if nested {
        let error = super::super::visit_snapshot_files(&root, true, |_| Ok(()))
            .err()
            .expect("a null DACL child must be rejected before its permissions change");
        assert!(error.to_string().contains("null DACL"));
    } else {
        let journal = Rc::new(NamespaceJournal::acquire(&path).unwrap());
        let error = DirectoryLease::open(&path, Rc::clone(&journal))
            .err()
            .unwrap();
        assert_eq!(error.kind(), io::ErrorKind::Unsupported);
        assert!(error.to_string().contains("null DACL"));
    }
    assert_eq!(directory_sddl(&path).unwrap(), null_sddl);
    fs::write(path.join("still-writable"), b"unchanged permissions").unwrap();
    drop(original);
    fs::remove_dir_all(root).unwrap();
}
