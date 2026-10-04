use super::*;
use crate::{
    asset::pack::{ZrPackInputAsset, ZrPackWriter},
    core::framework::{platform::RuntimeTargetMode, project::ExportTargetPlatform},
    plugin::native::NativePluginArtifactTarget,
};

fn request() -> InstallRequest {
    InstallRequest {
        schema_version: INSTALL_REQUEST_SCHEMA_V2,
        operation_id: uuid::Uuid::new_v4().to_string(),
        identity_digest: "a".repeat(64),
        package_id: "3298de17-8f1a-4e04-a28e-55c1ff43f2d8".into(),
        version: "1.0.0".into(),
        release_revision: "1".into(),
        artifact_digest: "b".repeat(64),
        artifact_size: 3,
        expected_inventory_revision: "0".into(),
        target: NativePluginArtifactTarget::new(
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Windows,
        ),
    }
}

fn policy() -> PackageHostPolicy {
    PackageHostPolicy {
        root: PathBuf::from("unused"),
        trust_registry: serde_json::json!({}),
        key_policies: vec![],
        trust_valid_until: Utc::now() + chrono::Duration::hours(1),
        target: NativePluginArtifactTarget::new(
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Windows,
        ),
        target_triple: "x86_64-pc-windows-msvc".into(),
        sdk_api_version: "0.1.0".into(),
        build_set_id: "a".repeat(64),
        allowed_capabilities: vec![],
        max_receipt_age_seconds: 3600,
    }
}

#[test]
fn package_service_request_rejects_aliases_oversize_and_unknown_fields() {
    let original = request();
    original.validate().unwrap();
    let wire = serde_json::to_value(&original).unwrap();
    assert_eq!(wire["schemaVersion"], INSTALL_REQUEST_SCHEMA_V2);
    assert_eq!(
        wire["target"],
        serde_json::json!({"runtime_mode":"client_runtime","platform":"windows"})
    );
    let mut invalid = original.clone();
    invalid.expected_inventory_revision = "00".into();
    assert!(matches!(invalid.validate(), Err(PackageError::Invalid)));
    invalid = original.clone();
    invalid.release_revision = "0".into();
    assert!(matches!(invalid.validate(), Err(PackageError::Invalid)));
    invalid = original.clone();
    invalid.operation_id.make_ascii_uppercase();
    assert!(matches!(invalid.validate(), Err(PackageError::Invalid)));
    invalid = original.clone();
    invalid.artifact_size = MAX_PACKAGE_BYTES as u64 + 1;
    assert!(matches!(invalid.validate(), Err(PackageError::Capacity)));
    let mut wire = serde_json::to_value(original).unwrap();
    assert!(wire.get("operationId").is_some());
    wire["token"] = serde_json::json!("forbidden");
    assert!(serde_json::from_value::<InstallRequest>(wire).is_err());
}

#[test]
fn package_service_request_requires_the_selected_host_target_and_v2_schema() {
    let mut request = request();
    request.target.runtime_mode = RuntimeTargetMode::EditorHost;
    let bytes = b"bad".to_vec();
    request.artifact_digest = digest(&bytes);
    assert!(matches!(
        PreparedPackage::verify(request.clone(), bytes.clone(), &policy()),
        Err(PackageError::Trust)
    ));

    request.target.runtime_mode = RuntimeTargetMode::ClientRuntime;
    request.schema_version = 1;
    assert!(matches!(
        PreparedPackage::verify(request, bytes, &policy()),
        Err(PackageError::Invalid)
    ));
}

#[test]
fn package_service_member_paths_reject_case_prefix_and_windows_aliases() {
    assert!(distinct_members(["plugin.toml", "bin/native.dll"]));
    for bad in [
        "../outside",
        "/root",
        "A/../B",
        "a\\b",
        "a:stream",
        "NUL.txt",
        "COM1",
        "dir./file",
    ] {
        assert!(!valid_member(bad), "{bad}");
    }
    assert!(!distinct_members(["bin/native.dll", "BIN/native.dll"]));
    assert!(!distinct_members(["bin", "bin/native.dll"]));
}

#[test]
fn package_service_verifies_whole_archive_before_parsing() {
    let mut input = request();
    assert!(matches!(
        PreparedPackage::verify(input.clone(), b"bad".to_vec(), &policy()),
        Err(PackageError::Trust)
    ));
    input.artifact_digest = digest(b"bad");
    assert!(matches!(
        PreparedPackage::verify(input, b"bad".to_vec(), &policy()),
        Err(PackageError::Invalid)
    ));
}

#[test]
fn package_service_rejects_expanded_deduplicated_budget_and_member_collisions() {
    let duplicate = vec![0; MAX_PACKAGE_BYTES / 2 + 1];
    let bytes = ZrPackWriter::write([
        ZrPackInputAsset::new("first.bin", duplicate.clone()),
        ZrPackInputAsset::new("second.bin", duplicate),
    ])
    .unwrap()
    .bytes;
    assert!(bytes.len() < MAX_PACKAGE_BYTES);
    let mut input = request();
    input.artifact_size = bytes.len() as u64;
    input.artifact_digest = digest(&bytes);
    assert!(matches!(
        PreparedPackage::verify(input, bytes, &policy()),
        Err(PackageError::Capacity)
    ));
    for names in [["bin", "bin/file"], ["bin/a", "BIN/A"]] {
        let bytes = ZrPackWriter::write(names.map(|name| ZrPackInputAsset::new(name, b"data")))
            .unwrap()
            .bytes;
        let mut input = request();
        input.artifact_size = bytes.len() as u64;
        input.artifact_digest = digest(&bytes);
        assert!(matches!(
            PreparedPackage::verify(input, bytes, &policy()),
            Err(PackageError::Invalid)
        ));
    }
}

#[test]
fn package_service_receipt_fingerprint_binds_account_and_inventory_request() {
    let input = request();
    let package = InstalledPackage {
        operation_id: input.operation_id.clone(),
        package_id: input.package_id.clone(),
        version: input.version.clone(),
        release_revision: input.release_revision.clone(),
        artifact_digest: input.artifact_digest.clone(),
        slot: "unused".into(),
        files: BTreeMap::new(),
    };
    let receipt = InstallReceipt {
        schema_version: 1,
        operation_id: input.operation_id.clone(),
        request_digest: input.fingerprint().unwrap(),
        inventory_revision: "1".into(),
        package,
        plugin_id: None,
        target: None,
    };
    assert!(receipt.matches_request(&input).unwrap());
    let mut changed = input.clone();
    changed.identity_digest = "c".repeat(64);
    assert!(!receipt.matches_request(&changed).unwrap());
    changed = input;
    changed.expected_inventory_revision = "1".into();
    assert!(!receipt.matches_request(&changed).unwrap());
}

#[cfg(windows)]
mod storage {
    use super::*;
    use crate::core::resource::io::transaction::TransactionFault;
    use std::{fs, path::Path, process::Command};

    struct PrivateRoot {
        path: PathBuf,
    }

    impl PrivateRoot {
        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for PrivateRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn private_root() -> PrivateRoot {
        let path = std::env::temp_dir().join(format!(
            "zircon-package-service-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        fs::create_dir(&path).unwrap();
        let dir = PrivateRoot { path };
        let username = format!(
            "{}\\{}",
            std::env::var("USERDOMAIN").unwrap(),
            std::env::var("USERNAME").unwrap()
        );
        let output = Command::new("icacls.exe")
            .arg(dir.path())
            .args([
                "/inheritance:r",
                "/grant:r",
                &format!("{username}:(OI)(CI)F"),
                "*S-1-5-18:(OI)(CI)F",
            ])
            .output()
            .unwrap();
        assert!(output.status.success(), "private fixture ACL failed");
        dir
    }

    #[test]
    fn package_store_enumerates_private_identity_partitions_in_stable_order() {
        let dir = private_root();
        drop(PackageStore::open(dir.path(), &"b".repeat(64)).unwrap());
        drop(PackageStore::open(dir.path(), &"a".repeat(64)).unwrap());

        let ids = PackageStore::open_all_existing(dir.path())
            .unwrap()
            .iter()
            .map(|store| store.identity_digest().to_owned())
            .collect::<Vec<_>>();

        assert_eq!(ids, vec!["a".repeat(64), "b".repeat(64)]);
    }

    #[test]
    fn package_store_refuses_unknown_entries_in_the_host_install_root() {
        let dir = private_root();
        fs::write(
            dir.path().join("unexpected.txt"),
            b"not an identity partition",
        )
        .unwrap();

        assert!(matches!(
            PackageStore::open_all_existing(dir.path()),
            Err(PackageError::Storage)
        ));
    }

    // Storage tests inject already-admitted bytes; signature admission has separate tests.
    fn prepared(input: InstallRequest) -> PreparedPackage {
        PreparedPackage {
            request: input,
            plugin_id: "fixture.plugin".into(),
            files: BTreeMap::from([
                ("plugin.toml".into(), b"manifest".to_vec()),
                ("bin/test.dll".into(), b"dll".to_vec()),
            ]),
            valid_until: Utc::now() + chrono::Duration::hours(1),
        }
    }

    #[test]
    fn package_service_query_absence_does_not_create_namespaces_and_commit_binds_owner() {
        let dir = private_root();
        let input = request();
        assert!(
            PackageStore::open_existing(dir.path(), &input.identity_digest)
                .unwrap()
                .is_none()
        );
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
        let store = PackageStore::open(dir.path(), &input.identity_digest).unwrap();
        let mut other = input.clone();
        other.identity_digest = "f".repeat(64);
        assert!(matches!(
            store.commit(prepared(other)),
            Err(PackageError::Invalid)
        ));
        let namespace = dir.path().join(&input.identity_digest);
        assert_eq!(fs::read_dir(namespace).unwrap().count(), 1);
        assert_eq!(store.inventory().unwrap().revision, "0");
    }

    #[test]
    fn package_service_real_junction_attack_is_denied_during_directory_pin() {
        use std::os::windows::{ffi::OsStrExt, fs::OpenOptionsExt, io::AsRawHandle};
        let dir = private_root();
        let parent = dir.path().join("parent");
        let target = dir.path().join("outside");
        fs::create_dir(&parent).unwrap();
        fs::create_dir(&target).unwrap();
        let substitute: Vec<u8> = format!("\\??\\{}", target.display())
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect();
        let display: Vec<u8> = target
            .as_os_str()
            .encode_wide()
            .flat_map(u16::to_le_bytes)
            .collect();
        let mut buffer = vec![0u8; 20 + substitute.len() + display.len()];
        buffer[..4].copy_from_slice(&0xA000_0003u32.to_le_bytes());
        let length = (buffer.len() - 8) as u16;
        buffer[4..6].copy_from_slice(&length.to_le_bytes());
        buffer[10..12].copy_from_slice(&(substitute.len() as u16).to_le_bytes());
        buffer[12..14].copy_from_slice(&((substitute.len() + 2) as u16).to_le_bytes());
        buffer[14..16].copy_from_slice(&(display.len() as u16).to_le_bytes());
        buffer[16..16 + substitute.len()].copy_from_slice(&substitute);
        let display_start = 18 + substitute.len();
        buffer[display_start..display_start + display.len()].copy_from_slice(&display);
        let attack = || -> std::io::Result<()> {
            let handle = fs::OpenOptions::new()
                .write(true)
                .share_mode(7)
                .custom_flags(0x0220_0000)
                .open(&parent)?;
            let mut returned = 0;
            let result = unsafe {
                DeviceIoControl(
                    handle.as_raw_handle(),
                    0x0009_00A4,
                    buffer.as_ptr().cast(),
                    buffer.len() as u32,
                    std::ptr::null_mut(),
                    0,
                    &mut returned,
                    std::ptr::null_mut(),
                )
            };
            if result == 0 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(())
            }
        };
        let writable = fs::OpenOptions::new()
            .write(true)
            .share_mode(7)
            .custom_flags(0x0220_0000)
            .open(&parent)
            .unwrap();
        assert!(super::super::windows::DirectoryPins::open(&parent, false).is_err());
        drop(writable);
        let pins = super::super::windows::DirectoryPins::open(&parent, false).unwrap();
        assert!(attack().is_err());
        drop(pins);
        attack().unwrap();
        assert!(
            super::super::windows::DirectoryPins::open(&parent.join("redirected"), true).is_err()
        );
        assert!(!target.join("redirected").exists());
        fs::write(parent.join("control"), b"redirected").unwrap();
        assert_eq!(fs::read(target.join("control")).unwrap(), b"redirected");
        fs::remove_dir(parent).unwrap();
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn DeviceIoControl(
            handle: *mut std::ffi::c_void,
            code: u32,
            input: *const std::ffi::c_void,
            input_size: u32,
            output: *mut std::ffi::c_void,
            output_size: u32,
            returned: *mut u32,
            overlapped: *mut std::ffi::c_void,
        ) -> i32;
    }

    #[test]
    fn package_service_store_commits_receipt_once_and_detects_missing_or_changed_files() {
        let dir = private_root();
        let input = request();
        let store = PackageStore::open(dir.path(), &input.identity_digest).unwrap();
        let receipt = store.commit(prepared(input.clone())).unwrap();
        assert_eq!(receipt.inventory_revision, "1");
        assert_eq!(receipt.schema_version, INSTALL_RECEIPT_SCHEMA_V2);
        assert_eq!(receipt.plugin_id.as_deref(), Some("fixture.plugin"));
        assert_eq!(receipt.target.as_ref(), Some(&input.target));
        assert_eq!(
            store
                .commit(prepared(input.clone()))
                .unwrap()
                .inventory_revision,
            "1"
        );
        assert_eq!(store.inventory().unwrap().packages.len(), 1);
        let file = dir
            .path()
            .join(&input.identity_digest)
            .join(&receipt.package.slot)
            .join("bin/test.dll");
        fs::write(&file, b"changed").unwrap();
        assert!(store.receipt(&input.operation_id).is_err());
        assert!(store.inventory().is_err());
    }

    #[test]
    fn package_service_store_conflicts_do_not_publish_or_cross_identities() {
        let dir = private_root();
        let input = request();
        let store = PackageStore::open(dir.path(), &input.identity_digest).unwrap();
        store.commit(prepared(input.clone())).unwrap();
        let mut changed = input.clone();
        changed.artifact_digest = "c".repeat(64);
        assert!(matches!(
            store.commit(prepared(changed)),
            Err(PackageError::Conflict)
        ));
        let mut next = input.clone();
        next.operation_id = uuid::Uuid::new_v4().to_string();
        next.release_revision = "2".into();
        next.artifact_digest = "d".repeat(64);
        assert!(matches!(
            store.commit(prepared(next.clone())),
            Err(PackageError::Conflict)
        ));
        assert!(store.receipt(&next.operation_id).unwrap().is_none());
        let other = PackageStore::open(dir.path(), &"e".repeat(64)).unwrap();
        assert_eq!(other.inventory().unwrap().revision, "0");
        assert!(other.receipt(&input.operation_id).unwrap().is_none());
    }

    #[test]
    fn package_service_restart_rolls_back_partial_commit_and_retries_same_operation() {
        let dir = private_root();
        let input = request();
        let store = PackageStore::open(dir.path(), &input.identity_digest).unwrap();
        assert!(matches!(
            store.commit_with_fault(
                prepared(input.clone()),
                TransactionFault::CrashAfterCommit(0)
            ),
            Err(PackageError::OutcomeUnknown)
        ));
        drop(store);
        let store = PackageStore::open(dir.path(), &input.identity_digest).unwrap();
        assert!(store.receipt(&input.operation_id).unwrap().is_none());
        assert_eq!(store.inventory().unwrap().revision, "0");
        assert_eq!(
            store.commit(prepared(input)).unwrap().inventory_revision,
            "1"
        );
    }

    #[test]
    fn package_service_restart_preserves_committed_unknown_receipt_without_second_install() {
        let dir = private_root();
        let input = request();
        let store = PackageStore::open(dir.path(), &input.identity_digest).unwrap();
        assert!(matches!(
            store.commit_with_fault(
                prepared(input.clone()),
                TransactionFault::CrashAfterAllCommitted
            ),
            Err(PackageError::OutcomeUnknown)
        ));
        drop(store);
        let store = PackageStore::open(dir.path(), &input.identity_digest).unwrap();
        assert_eq!(
            store
                .receipt(&input.operation_id)
                .unwrap()
                .unwrap()
                .inventory_revision,
            "1"
        );
        assert_eq!(
            store.commit(prepared(input)).unwrap().inventory_revision,
            "1"
        );
    }

    #[test]
    fn package_service_owner_lock_and_parent_cannot_be_replaced() {
        let dir = private_root();
        let input = request();
        let store = PackageStore::open(dir.path(), &input.identity_digest).unwrap();
        let owner = dir.path().join(&input.identity_digest);
        assert!(matches!(
            PackageStore::open(dir.path(), &input.identity_digest),
            Err(PackageError::Busy)
        ));
        assert!(fs::remove_file(owner.join(".package-owner")).is_err());
        assert!(fs::rename(&owner, dir.path().join("moved")).is_err());
        drop(store);
        fs::rename(owner, dir.path().join("moved")).unwrap();
    }

    #[test]
    fn package_service_reads_reject_hardlinks_devices_and_oversize() {
        let dir = private_root();
        let path = dir.path().join("input");
        fs::write(&path, b"test").unwrap();
        assert!(matches!(
            read_regular(&path, 3),
            Err(PackageError::Capacity)
        ));
        assert_eq!(read_regular(&path, 4).unwrap(), b"test");
        fs::hard_link(&path, dir.path().join("alias")).unwrap();
        assert!(read_regular(&path, 4).is_err());
        assert!(read_regular(dir.path(), 4).is_err());
        assert!(read_regular(Path::new(r"\\.\NUL"), 4).is_err());
    }
}
