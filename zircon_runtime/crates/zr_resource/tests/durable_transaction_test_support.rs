//! Integration regression for the ordinary Resource library with test-support enabled.
//! Inputs come only from the real transaction API; this test does not fabricate journal frames.

#[cfg(not(all(feature = "test-support", feature = "profiling")))]
compile_error!("this regression target requires test-support and profiling features");

use std::fs;
#[cfg(windows)]
use std::path::{Component, Prefix};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use zr_resource::assembly::io::transaction::{
    commit_prepared_files, recover_pending_transactions, DurableCommitReport,
    DurableTransactionError, JournalDocument, PreparedFileWrite, RecoveryPolicy, TransactionFault,
    TransactionPhase,
};

#[cfg(windows)]
fn validate_managed_root(configured: &Path, target_root: &Path) {
    let normalize_encoding = |path: &Path| {
        path.to_string_lossy()
            .trim_start_matches(r"\\?\")
            .replace('/', "\\")
            .to_ascii_lowercase()
    };
    assert_eq!(
        normalize_encoding(&configured),
        normalize_encoding(&target_root),
        "managed target path aliases are forbidden"
    );
    let mut components = target_root.components();
    let drive = match components.next() {
        Some(Component::Prefix(prefix)) => match prefix.kind() {
            Prefix::Disk(drive) | Prefix::VerbatimDisk(drive) => drive.to_ascii_uppercase(),
            other => panic!("unsupported physical target prefix: {other:?}"),
        },
        other => panic!("target must resolve to an absolute Windows drive: {other:?}"),
    };
    assert!(matches!(drive, b'D' | b'E' | b'F'));
    assert_eq!(components.next(), Some(Component::RootDir));
    match components.next() {
        Some(Component::Normal(root)) => {
            assert!(root.to_string_lossy().eq_ignore_ascii_case("cargo-targets"));
        }
        other => panic!("physical target must be under drive-root cargo-targets: {other:?}"),
    }
    assert!(
        components.next().is_some(),
        "require a private managed target slot"
    );
}

#[cfg(unix)]
fn validate_managed_root(configured: &Path, target_root: &Path) {
    assert_eq!(
        configured.as_os_str(),
        target_root.as_os_str(),
        "managed Unix target must exactly match its canonical physical path"
    );
    let approved_roots = [
        Path::new("/mnt/d/cargo-targets"),
        Path::new("/mnt/e/cargo-targets"),
        Path::new("/mnt/f/cargo-targets"),
    ];
    let approved = approved_roots.iter().any(|root| {
        target_root
            .strip_prefix(root)
            .is_ok_and(|relative| !relative.as_os_str().is_empty())
    });
    assert!(
        approved,
        "require a private slot under a mounted D/E/F cargo-targets root"
    );
}

#[cfg(not(any(windows, unix)))]
fn validate_managed_root(_configured: &Path, _target_root: &Path) {
    panic!("no approved managed storage root for this platform");
}

struct PrivateFixture {
    target_root: PathBuf,
    root: PathBuf,
}

impl PrivateFixture {
    fn new() -> Self {
        let configured = PathBuf::from(
            std::env::var_os("CARGO_TARGET_DIR")
                .expect("managed regression requires CARGO_TARGET_DIR"),
        );
        assert!(configured.is_absolute(), "managed target must be absolute");
        let target_root = fs::canonicalize(&configured)
            .expect("resolve the existing coordinator-assigned target root");
        validate_managed_root(&configured, &target_root);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after epoch")
            .as_nanos();
        for attempt in 0..64 {
            let root = target_root.join(format!(
                "resource-test-support-regression-{}-{timestamp}-{attempt}",
                std::process::id()
            ));
            match fs::create_dir(&root) {
                Ok(()) => return Self { target_root, root },
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create exclusive fixture: {error}"),
            }
        }
        panic!("private fixture allocation exhausted");
    }
}

impl Drop for PrivateFixture {
    fn drop(&mut self) {
        // Ownership begins only after exclusive create_dir succeeded. A failed assertion still
        // retires this fixture, and a changed/aliased path never broadens recursive cleanup.
        if let Ok(physical) = fs::canonicalize(&self.root) {
            if physical == self.root && physical.parent() == Some(self.target_root.as_path()) {
                let _ = fs::remove_dir_all(&self.root);
            }
        }
    }
}

fn actual_wal_frames(path: &Path) -> Vec<toml::Value> {
    let bytes = fs::read(path).expect("read the actual retained production WAL");
    let mut offset = 0_usize;
    let mut frames = Vec::new();
    while offset < bytes.len() {
        let remaining = &bytes[offset..];
        let header_bytes = 8 + blake3::OUT_LEN;
        assert!(remaining.len() >= header_bytes, "complete WAL header");
        let payload_len = u64::from_le_bytes(remaining[..8].try_into().unwrap());
        let payload_len = usize::try_from(payload_len).expect("bounded payload length");
        let end = header_bytes.checked_add(payload_len).expect("frame length");
        assert!(remaining.len() >= end, "complete WAL payload");
        let payload = &remaining[header_bytes..end];
        assert_eq!(
            blake3::hash(payload).as_bytes().as_slice(),
            &remaining[8..header_bytes],
            "production frame checksum"
        );
        let text = std::str::from_utf8(payload).expect("actual WAL payload is UTF-8");
        frames.push(toml::from_str(text).expect("decode actual WAL TOML"));
        offset += end;
    }
    frames
}

struct ExactTargets {
    first: PathBuf,
    second: PathBuf,
}

impl RecoveryPolicy for ExactTargets {
    fn validate_document(&self, _journal: &Path, document: &JournalDocument) -> Result<(), String> {
        if (document.target() == self.first || document.target() == self.second)
            && document.retired_paths().next().is_none()
        {
            Ok(())
        } else {
            Err("document is outside this test's two exact targets".to_owned())
        }
    }
}

#[test]
fn retained_wal_records_one_rollback_transition_in_test_support_dependency() {
    let fixture = PrivateFixture::new();
    let journal = fixture.root.join("journal");
    let first = fixture.root.join("first.zmeta");
    let second = fixture.root.join("second.zmeta");
    fs::write(&first, b"old-first").unwrap();
    fs::write(&second, b"old-second").unwrap();
    let mut report = DurableCommitReport::default();

    let error = commit_prepared_files(
        &journal,
        "project",
        vec![
            PreparedFileWrite::new(first.clone(), b"new-first".to_vec()),
            PreparedFileWrite::new(second.clone(), b"new-second".to_vec()),
        ],
        TransactionFault::RestoreFailure {
            commit_index: 1,
            restore_index: 0,
        },
        &mut report,
    )
    .expect_err("failed restoration must retain a real active journal");

    match &error {
        DurableTransactionError::Operation { phase, source, .. } => {
            assert_eq!(*phase, TransactionPhase::Rollback);
            assert_eq!(source.kind(), std::io::ErrorKind::Other);
            assert!(source.to_string().contains("injected restore failure"));
        }
        other => panic!("unexpected transaction error: {other:?}"),
    }
    assert_eq!(fs::read(&first).unwrap(), b"new-first");
    assert_eq!(fs::read(&second).unwrap(), b"old-second");
    // profiling exposes the real library's getters; it does not change the fault branch.
    assert_eq!(report.rollback_restore_attempt_count(), 1);
    assert_eq!(report.rollback_restore_success_count(), 0);
    assert_eq!(report.deferred_commit_recovery_count(), 0);
    assert_eq!(report.deferred_cleanup_count(), 0);

    let entries = fs::read_dir(&journal)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    assert_eq!(entries.len(), 1, "one WAL, not the sibling owner lock");
    let wal = &entries[0];
    assert_eq!(
        wal.extension().and_then(|value| value.to_str()),
        Some("zrjournal")
    );
    assert!(fs::symlink_metadata(wal).unwrap().is_file());
    let frames = actual_wal_frames(wal);
    let intent = frames.first().expect("actual immutable intent");
    assert_eq!(intent["version"].as_integer(), Some(7));
    assert_eq!(intent["tag"].as_str(), Some("project"));
    let documents = intent["documents"]
        .as_array()
        .expect("actual intent documents");
    assert_eq!(documents.len(), 2);
    assert_eq!(Path::new(documents[0]["target"].as_str().unwrap()), first);
    assert_eq!(Path::new(documents[1]["target"].as_str().unwrap()), second);

    let transitions = frames
        .iter()
        .skip(1)
        .flat_map(|frame| frame["transitions"].as_array().unwrap());
    let mut rollback_count = 0;
    let mut last_phase = "intent";
    for transition in transitions {
        last_phase = transition["phase"].as_str().expect("transition phase");
        if transition.get("state").and_then(toml::Value::as_str) == Some("rolling_back") {
            assert_eq!(last_phase, "active");
            assert_eq!(transition["document_index"].as_integer(), Some(0));
            rollback_count += 1;
        }
    }
    assert_eq!(
        last_phase, "active",
        "failed restoration is not terminal cleanup"
    );
    assert_eq!(
        rollback_count, 1,
        "one real rollback attempt must append one RollingBack transition; duplicate cfg branches append two"
    );

    let recovered = recover_pending_transactions(
        &journal,
        "project",
        &mut ExactTargets {
            first: first.clone(),
            second: second.clone(),
        },
    )
    .expect("recover actual retained production evidence");
    assert_eq!(recovered.rollback_count(), 1);
    assert_eq!(recovered.cleanup_count(), 1);
    assert_eq!(recovered.intent_orphan_cleanup_count(), 0);
    assert_eq!(fs::read(&first).unwrap(), b"old-first");
    assert_eq!(fs::read(&second).unwrap(), b"old-second");
    assert_eq!(fs::read_dir(&journal).unwrap().count(), 0);
}
