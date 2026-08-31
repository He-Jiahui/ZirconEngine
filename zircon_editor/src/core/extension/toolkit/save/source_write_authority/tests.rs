use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::*;

static NEXT_TEST_ROOT: AtomicU64 = AtomicU64::new(1);

#[test]
fn aliases_for_one_source_share_one_write_lease() {
    let fixture = Fixture::new("alias-lease");
    let authority = DocumentSourceWriteAuthority::default();
    let source = fixture.source("panel.zui", b"baseline");
    let alias = fixture
        .root
        .join("assets")
        .join("..")
        .join("assets/panel.zui");

    let lease = authority.acquire(&fixture.root, &source).unwrap();
    assert!(authority
        .try_acquire(&fixture.root, &alias)
        .unwrap()
        .is_none());
    drop(lease);
    assert!(authority
        .try_acquire(&fixture.root, &alias)
        .unwrap()
        .is_some());
}

#[cfg(windows)]
#[test]
fn uncreated_windows_case_aliases_share_one_write_lease() {
    let fixture = Fixture::new("uncreated-case-alias-lease");
    let authority = DocumentSourceWriteAuthority::default();
    let source = fixture.root.join("assets/Panel.zui");
    let alias = fixture.root.join("ASSETS/panel.zui");

    let lease = authority.acquire(&fixture.root, &source).unwrap();
    assert!(authority
        .try_acquire(&fixture.root, &alias)
        .unwrap()
        .is_none());
    drop(lease);
    assert!(authority
        .try_acquire(&fixture.root, &alias)
        .unwrap()
        .is_some());
}

#[test]
fn same_source_waits_until_the_current_write_lease_is_released() {
    let fixture = Fixture::new("same-source-wait");
    let authority = Arc::new(DocumentSourceWriteAuthority::default());
    let source = fixture.source("panel.zui", b"baseline");
    let waiting_authority = Arc::clone(&authority);
    let waiting_root = fixture.root.clone();
    let waiting_source = source.clone();
    let (waiting_send, waiting_receive) = mpsc::sync_channel(1);
    let (acquired_send, acquired_receive) = mpsc::channel();
    let active = authority.acquire(&fixture.root, &source).unwrap();
    let waiting = std::thread::spawn(move || {
        let lease = waiting_authority
            .acquire_with_wait_hook(&waiting_root, &waiting_source, || {
                waiting_send.send(()).unwrap();
            })
            .unwrap();
        acquired_send.send(()).unwrap();
        drop(lease);
    });

    waiting_receive
        .recv_timeout(Duration::from_secs(5))
        .expect("the competing source writer must enter the Condvar wait path");
    assert!(
        acquired_receive.try_recv().is_err(),
        "the competing source writer must wait while the first lease is active"
    );
    drop(active);
    acquired_receive
        .recv_timeout(Duration::from_secs(5))
        .expect("the competing source writer must acquire after release");
    waiting.join().unwrap();
}

#[test]
fn stale_save_after_a_cooperating_external_effect_reports_source_changed() {
    let fixture = Fixture::new("save-versus-external-effect");
    let authority = Arc::new(DocumentSourceWriteAuthority::default());
    let source = fixture.source("panel.zui", b"opened baseline");
    let external_effect = authority.acquire(&fixture.root, &source).unwrap();
    let waiting_authority = Arc::clone(&authority);
    let waiting_root = fixture.root.clone();
    let waiting_source = source.clone();
    let (waiting_send, waiting_receive) = mpsc::sync_channel(1);
    let (result_send, result_receive) = mpsc::channel();
    let stale_save = std::thread::spawn(move || {
        let lease = waiting_authority
            .acquire_with_wait_hook(&waiting_root, &waiting_source, || {
                waiting_send.send(()).unwrap();
            })
            .unwrap();
        result_send
            .send(lease.commit_if_matches(b"opened baseline", b"stale save"))
            .unwrap();
    });

    waiting_receive
        .recv_timeout(Duration::from_secs(5))
        .expect("the stale save must wait behind the cooperating external effect");
    assert!(matches!(
        external_effect.replace(b"undo replacement"),
        DocumentSourceWriteOutcome::DurableBestEffort
    ));
    drop(external_effect);

    assert!(matches!(
        result_receive
            .recv_timeout(Duration::from_secs(5))
            .expect("the stale save must finish after the external effect releases its lease"),
        DocumentSourceWriteOutcome::SourceChanged
    ));
    stale_save.join().unwrap();
    assert_eq!(fs::read(source).unwrap(), b"undo replacement");
}

#[test]
fn distinct_sources_can_hold_write_leases_together() {
    let fixture = Fixture::new("distinct-leases");
    let authority = DocumentSourceWriteAuthority::default();
    let first = fixture.source("first.zui", b"first");
    let second = fixture.source("second.zui", b"second");

    let first_lease = authority.acquire(&fixture.root, &first).unwrap();
    let second_lease = authority
        .try_acquire(&fixture.root, &second)
        .unwrap()
        .expect("a distinct source must not wait for another source");

    drop((first_lease, second_lease));
}

#[test]
fn stale_baseline_cannot_overwrite_the_first_committed_save() {
    let fixture = Fixture::new("stale-baseline");
    let authority = DocumentSourceWriteAuthority::default();
    let source = fixture.source("panel.zui", b"opened baseline");

    let first = authority.acquire(&fixture.root, &source).unwrap();
    assert!(matches!(
        first.commit_if_matches(b"opened baseline", b"first save"),
        DocumentSourceWriteOutcome::DurableBestEffort
    ));
    drop(first);

    let stale = authority.acquire(&fixture.root, &source).unwrap();
    assert!(matches!(
        stale.commit_if_matches(b"opened baseline", b"stale save"),
        DocumentSourceWriteOutcome::SourceChanged
    ));
    assert_eq!(fs::read(&source).unwrap(), b"first save");
}

#[test]
fn read_only_source_is_rejected_before_publication() {
    let fixture = Fixture::new("read-only");
    let authority = DocumentSourceWriteAuthority::default();
    let source = fixture.source("panel.zui", b"baseline");
    let mut permissions = fs::metadata(&source).unwrap().permissions();
    permissions.set_readonly(true);
    fs::set_permissions(&source, permissions).unwrap();

    let lease = authority.acquire(&fixture.root, &source).unwrap();
    let outcome = lease.commit_if_matches(b"baseline", b"replacement");

    assert!(matches!(
        outcome,
        DocumentSourceWriteOutcome::NotPublished(error)
            if error.kind() == io::ErrorKind::PermissionDenied
    ));
    assert_eq!(fs::read(&source).unwrap(), b"baseline");
    let mut permissions = fs::metadata(&source).unwrap().permissions();
    permissions.set_readonly(false);
    fs::set_permissions(&source, permissions).unwrap();
}

#[test]
fn nonparticipating_writer_is_reported_only_as_best_effort() {
    let fixture = Fixture::new("external-race");
    let authority = DocumentSourceWriteAuthority::default();
    let source = fixture.source("panel.zui", b"baseline");
    let lease = authority.acquire(&fixture.root, &source).unwrap();

    let outcome =
        lease.commit_if_matches_with_publisher(b"baseline", b"local save", |path, replacement| {
            fs::write(path, b"nonparticipating write")?;
            atomic_write(path, replacement)
        });

    assert!(matches!(
        outcome,
        DocumentSourceWriteOutcome::DurableBestEffort
    ));
    assert_eq!(fs::read(&source).unwrap(), b"local save");
}

#[test]
fn publication_error_after_visible_replace_is_reported_as_not_durable() {
    let fixture = Fixture::new("published-not-durable");
    let authority = DocumentSourceWriteAuthority::default();
    let source = fixture.source("panel.zui", b"baseline");
    let lease = authority.acquire(&fixture.root, &source).unwrap();

    let outcome = lease.commit_if_matches_with_publisher(
        b"baseline",
        b"published bytes",
        |path, replacement| {
            fs::write(path, replacement)?;
            Err(io::Error::new(
                io::ErrorKind::Other,
                "injected durability barrier failure",
            ))
        },
    );

    assert!(matches!(
        outcome,
        DocumentSourceWriteOutcome::PublishedNotDurable(_)
    ));
    assert_eq!(fs::read(source).unwrap(), b"published bytes");
}

#[test]
fn matching_bytes_do_not_prove_that_a_failed_publisher_replaced_the_source() {
    let fixture = Fixture::new("matching-bytes-not-published");
    let authority = DocumentSourceWriteAuthority::default();
    let source = fixture.source("panel.zui", b"unchanged bytes");
    let lease = authority.acquire(&fixture.root, &source).unwrap();

    let outcome = lease.commit_if_matches_with_publisher(
        b"unchanged bytes",
        b"unchanged bytes",
        |_path, _replacement| {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "injected failure before publication",
            ))
        },
    );

    assert!(matches!(
        outcome,
        DocumentSourceWriteOutcome::NotPublished(_)
    ));
    assert_eq!(fs::read(source).unwrap(), b"unchanged bytes");
}

#[test]
fn unknown_prepublication_observation_does_not_block_a_successful_replace() {
    let fixture = Fixture::new("unknown-prepublication-observation");
    let authority = DocumentSourceWriteAuthority::default();
    let source = fixture.source("panel.zui", b"baseline");
    let lease = authority.acquire(&fixture.root, &source).unwrap();
    let mut publisher_called = false;

    let outcome = lease.replace_with_publisher_and_observer(
        b"replacement",
        |path, replacement| {
            publisher_called = true;
            fs::write(path, replacement)
        },
        |_path| SourceBeforePublication::Unknown,
    );

    assert!(publisher_called);
    assert!(matches!(
        outcome,
        DocumentSourceWriteOutcome::DurableBestEffort
    ));
    assert_eq!(fs::read(source).unwrap(), b"replacement");
}

#[test]
fn unconditional_replace_creates_a_missing_source_inside_the_project() {
    let fixture = Fixture::new("create-missing-source");
    let authority = DocumentSourceWriteAuthority::default();
    let source = fixture.root.join("assets/created.zui");
    let lease = authority.acquire(&fixture.root, &source).unwrap();

    assert!(matches!(
        lease.replace(b"created bytes"),
        DocumentSourceWriteOutcome::DurableBestEffort
    ));
    assert_eq!(fs::read(source).unwrap(), b"created bytes");
}

#[test]
fn conditional_save_treats_a_missing_source_as_changed() {
    let fixture = Fixture::new("conditional-missing-source");
    let authority = DocumentSourceWriteAuthority::default();
    let source = fixture.root.join("assets/missing.zui");
    let lease = authority.acquire(&fixture.root, &source).unwrap();

    assert!(matches!(
        lease.commit_if_matches(b"opened bytes", b"saved bytes"),
        DocumentSourceWriteOutcome::SourceChanged
    ));
    assert!(!source.exists());
}

#[test]
fn removing_a_missing_source_is_an_idempotent_no_op() {
    let fixture = Fixture::new("remove-missing-source");
    let authority = DocumentSourceWriteAuthority::default();
    let source = fixture.root.join("assets/missing.zui");
    let lease = authority.acquire(&fixture.root, &source).unwrap();

    assert!(!lease.remove_if_exists().unwrap());
    assert!(!source.exists());
}

#[test]
fn missing_source_outside_project_root_is_rejected() {
    let fixture = Fixture::new("outside-missing-root");
    let outside = Fixture::new("outside-missing-source");
    let source = outside.root.join("assets/missing.zui");

    let error = DocumentSourceWriteAuthority::default()
        .acquire(&fixture.root, &source)
        .unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
}

#[test]
fn source_outside_project_root_is_rejected() {
    let fixture = Fixture::new("outside-root");
    let outside = Fixture::new("outside-source");
    let source = outside.source("panel.zui", b"baseline");

    let error = DocumentSourceWriteAuthority::default()
        .acquire(&fixture.root, &source)
        .unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
}

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let root = test_output_root().join(format!(
            "zircon_editor_source_write_{label}_{}_{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time after Unix epoch")
                .as_nanos(),
            NEXT_TEST_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("assets")).unwrap();
        Self { root }
    }

    fn source(&self, file_name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.root.join("assets").join(file_name);
        fs::write(&path, bytes).unwrap();
        path
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn test_output_root() -> PathBuf {
    std::env::var_os("ZIRCON_TEST_OUTPUT_ROOT")
        .or_else(|| std::env::var_os("CARGO_TARGET_DIR"))
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::current_dir()
                .expect("resolve current workspace for source-write test output")
                .join("target")
        })
        .join("zircon-test-output")
}
