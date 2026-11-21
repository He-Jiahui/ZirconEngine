//! Freeze the v7 wire contract from e89349ca / 22137934, independently of today's writer.

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

use serde::Deserialize;

use crate::io::source_revision::SourceRevision;
use crate::io::transaction::pathing::{journal_path, transaction_id_for_test, transaction_sibling};
use crate::io::transaction::{
    commit_prepared_files, recover_pending_transactions, DurableCommitReport,
    DurableTransactionError, JournalDocument, PreparedFileWrite, RecoveryPolicy, TransactionFault,
    TransactionPhase,
};

const TAG: &str = "project";
const OLD: &[u8] = b"title = \"previous generation\"\n";
const NEW: &[u8] = b"title = \"uncommitted generation\"\n";

// v7's schema.rs, journal/intent.rs and journal/append.rs define these five frames. Only paths,
// the transaction identity bound to the scratch directory, and digests of OLD/NEW are substituted.
fn v7_payloads(fixture: &Fixture, id: &str) -> [String; 5] {
    let quote = |p: &Path| toml::Value::String(p.to_str().unwrap().to_owned()).to_string();
    let target = quote(&fixture.target);
    let staging = quote(&fixture.sibling("stage", id));
    let backup = quote(&fixture.sibling("backup", id));
    let rollback = quote(&fixture.sibling("rollback-stage", id));
    let original_digest = blake3::hash(OLD).to_hex();
    let new_digest = blake3::hash(NEW).to_hex();
    [
        format!(
            "version = 7\ntag = \"project\"\ntransaction_id = \"{id}\"\n\n[[documents]]\n\
             target = {target}\nstaging = {staging}\nbackup = {backup}\nrollback_staging = {rollback}\n"
        ),
        format!(
            "[[transitions]]\nphase = \"intent\"\ndocument_index = 0\nstate = \"prepared\"\n\
             target_existed = true\noriginal_digest = \"{original_digest}\"\nnew_digest = \"{new_digest}\"\n"
        ),
        "[[transitions]]\nphase = \"active\"\n".into(),
        "[[transitions]]\nphase = \"active\"\ndocument_index = 0\nstate = \"committing\"\n".into(),
        "[[transitions]]\nphase = \"active\"\ndocument_index = 0\nstate = \"committed\"\n".into(),
    ]
}

fn frame_payloads(mut bytes: &[u8]) -> Vec<&str> {
    let mut result = Vec::new();
    while !bytes.is_empty() {
        let body_len = u64::from_le_bytes(bytes[..8].try_into().unwrap()) as usize;
        let (frame, remaining) = bytes.split_at(40 + body_len);
        let payload = &frame[40..];
        assert_eq!(&frame[8..40], blake3::hash(payload).as_bytes());
        result.push(std::str::from_utf8(payload).unwrap());
        bytes = remaining;
    }
    result
}

#[test]
fn legacy_wire_v7_recovers_the_frozen_active_generation() {
    let fixture = Fixture::new("read-v7");
    let id = transaction_id_for_test(&fixture.journal, 1);
    fs::write(&fixture.target, NEW).unwrap();
    let backup = fixture.sibling("backup", &id);
    fs::write(&backup, OLD).unwrap();
    let journal = journal_path(&fixture.journal, &fixture.target, TAG, &id);
    let mut bytes = Vec::new();
    // Fixed v7 frame length and checksum encoding, not TransactionJournal serialization.
    for payload in v7_payloads(&fixture, &id) {
        bytes.extend_from_slice(&(payload.len() as u64).to_le_bytes());
        bytes.extend_from_slice(blake3::hash(payload.as_bytes()).as_bytes());
        bytes.extend_from_slice(payload.as_bytes());
    }
    fs::write(&journal, &bytes).unwrap();
    fixture.recover(false);
    assert!(!journal.exists());
    assert!(!backup.exists());
}

#[test]
fn legacy_wire_v7_is_still_emitted_by_ordinary_writes() {
    let fixture = Fixture::new("write-v7");
    fs::write(&fixture.target, OLD).unwrap();
    let bytes = fixture.interrupt(PreparedFileWrite::new(&fixture.target, NEW.to_vec()));
    let actual = frame_payloads(&bytes);
    let legacy: LegacyIntent = toml::from_str(actual[0]).expect("the old v7 reader shape");
    assert_eq!(legacy.version, 7);
    assert!(legacy.documents.len() == 1);
    let expected = v7_payloads(&fixture, &legacy.transaction_id);
    assert_eq!(actual.len(), expected.len());
    for (frame, (actual, reference)) in actual.iter().zip(&expected).enumerate() {
        assert_eq!(
            toml::from_str::<toml::Table>(actual).unwrap(),
            toml::from_str::<toml::Table>(reference).unwrap(),
            "legacy v7 wire fields in frame {frame}"
        );
    }
    fixture.recover(false);
}

#[test]
fn legacy_wire_v7_reader_rejects_real_v8_stamp_but_current_replay_preserves_it() {
    let fixture = Fixture::new("write-v8");
    fs::write(&fixture.target, OLD).unwrap();
    let stamp = UNIX_EPOCH + Duration::from_secs(950_000_000);
    let file = File::options().write(true).open(&fixture.target).unwrap();
    file.set_modified(stamp).unwrap();
    file.sync_all().unwrap();
    drop(file);
    assert_eq!(
        fs::metadata(&fixture.target).unwrap().modified().unwrap(),
        stamp
    );
    let expected = SourceRevision::observe(&fixture.target).unwrap();
    let bytes = fixture.interrupt(
        PreparedFileWrite::new(&fixture.target, NEW.to_vec())
            .preserving_observed_mtime(expected, stamp),
    );
    let frames = frame_payloads(&bytes);
    let intent: toml::Table = toml::from_str(frames[0]).unwrap();
    assert_eq!(intent["version"].as_integer(), Some(8));
    let err = toml::from_str::<LegacyIntent>(frames[0]).unwrap_err();
    assert!(
        err.to_string().contains("unknown field `retained_mtime`"),
        "{err}"
    );
    assert_eq!(
        fs::metadata(&fixture.target).unwrap().modified().unwrap(),
        stamp
    );
    fixture.recover(true);
    assert_eq!(
        fs::metadata(&fixture.target).unwrap().modified().unwrap(),
        stamp
    );
}

// The old reader's `deny_unknown_fields` mattered at both levels. Keep the independent old
// document shape rather than deserializing with the production JournalIntent implementation.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyIntent {
    version: u32,
    #[serde(rename = "tag")]
    _tag: String,
    transaction_id: String,
    documents: Vec<LegacyDocument>,
    #[serde(default, rename = "transitions")]
    _transitions: Vec<toml::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyDocument {
    #[serde(rename = "target")]
    _target: PathBuf,
    #[serde(rename = "staging")]
    _staging: PathBuf,
    #[serde(rename = "backup")]
    _backup: PathBuf,
    #[serde(rename = "rollback_staging")]
    _rollback_staging: PathBuf,
    #[serde(default, rename = "retirements")]
    _retirements: Vec<toml::Value>,
}

struct Fixture {
    root: PathBuf,
    journal: PathBuf,
    target: PathBuf,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let root = super::test_directory(label);
        fs::create_dir_all(root.parent().unwrap()).unwrap();
        fs::create_dir(&root).expect("claim an unused test root");
        let root = fs::canonicalize(root).unwrap();
        let assets = root.join("assets");
        let journal = root.join("journal");
        fs::create_dir(&assets).unwrap();
        fs::create_dir(&journal).unwrap();
        let target = assets.join("generation.zmeta");
        Self {
            root,
            journal,
            target,
        }
    }

    fn sibling(&self, role: &str, id: &str) -> PathBuf {
        transaction_sibling(&self.target, TAG, role, id)
    }

    fn interrupt(&self, write: PreparedFileWrite) -> Vec<u8> {
        let err = commit_prepared_files(
            &self.journal,
            TAG,
            vec![write],
            TransactionFault::CrashAfterCommit(0),
            &mut DurableCommitReport::default(),
        )
        .unwrap_err();
        assert!(
            matches!(err, DurableTransactionError::Operation { phase: TransactionPhase::Commit, source, .. }
                 if source.kind() == std::io::ErrorKind::Interrupted)
        );
        assert_eq!(fs::read(&self.target).unwrap(), NEW);
        let entries = fs::read_dir(&self.journal)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(entries.len(), 1);
        let path = entries[0].path();
        assert_eq!(path.extension().unwrap(), "zrjournal");
        fs::read(path).unwrap()
    }

    fn recover(&self, permits_clock: bool) {
        let report = recover_pending_transactions(
            &self.journal,
            TAG,
            &mut OneTarget {
                path: &self.target,
                permits_clock,
            },
        )
        .unwrap();
        assert_eq!(report.rollback_count(), 1);
        assert_eq!(fs::read(&self.target).unwrap(), OLD);
        assert_eq!(fs::read_dir(&self.journal).unwrap().count(), 0);
        assert_eq!(
            fs::read_dir(self.target.parent().unwrap()).unwrap().count(),
            1
        );
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if std::thread::panicking() {
            eprintln!("legacy WAL fixture retained: {}", self.root.display());
        } else if fs::canonicalize(&self.root).ok().as_deref() == Some(self.root.as_path()) {
            fs::remove_dir_all(&self.root).unwrap();
        }
    }
}

struct OneTarget<'a> {
    path: &'a Path,
    permits_clock: bool,
}

impl RecoveryPolicy for OneTarget<'_> {
    fn validate_document(&self, _: &Path, doc: &JournalDocument) -> Result<(), String> {
        if doc.target() == self.path && doc.retired_paths().next().is_none() {
            Ok(())
        } else {
            Err("not the fixture's exact owned target".into())
        }
    }

    fn allows_retained_source_mtime(&self, _: &JournalDocument) -> bool {
        self.permits_clock
    }
}
