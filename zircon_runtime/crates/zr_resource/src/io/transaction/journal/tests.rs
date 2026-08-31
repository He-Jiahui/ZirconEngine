use std::fs;
use std::path::{Path, PathBuf};

use super::super::PreparedFileWrite;
use super::super::error::TransactionPhase;
use super::super::schema::{
    JOURNAL_VERSION, JournalIntent, JournalPhase, JournalState, JournalTransition,
    TransactionJournal,
};
use super::frame_codec::{FRAME_HEADER_BYTES, TransitionAppend, encode_frame, transition_frame};
use super::*;

const TEST_TRANSACTION_ID: &str =
    "0000000000000000000000000000000000000000000000000000000000000000-1-1";

#[test]
fn immutable_intent_does_not_replace_an_existing_journal() {
    let root = test_directory("intent-create-new");
    let journal_directory = root.join("journal");
    let target = root.join("asset.zmeta");
    fs::create_dir_all(&journal_directory).unwrap();
    let transaction_id = super::super::pathing::transaction_id_for_test(&journal_directory, 1);
    let journal = super::super::pathing::journal_path(
        &journal_directory,
        &target,
        "project",
        &transaction_id,
    );
    let existing_evidence = b"existing recovery evidence";
    fs::write(&journal, existing_evidence).unwrap();

    let error = create_intent(
        &journal_directory,
        "project",
        &transaction_id,
        &[PreparedFileWrite::new(target, b"new generation".to_vec())],
    )
    .expect_err("immutable intent publication must never replace recovery evidence");

    assert!(matches!(
        error,
        super::super::error::DurableTransactionError::Operation { source, .. }
            if source.kind() == std::io::ErrorKind::AlreadyExists
    ));
    assert_eq!(fs::read(&journal).unwrap(), existing_evidence);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn every_torn_transition_tail_folds_to_the_previous_durable_state() {
    let intent = intent_frame();
    let transition = encode_frame(
        toml::to_string_pretty(&prepared_transition())
            .unwrap()
            .as_bytes(),
    )
    .unwrap();

    for cut in 0..transition.len() {
        let mut bytes = intent.clone();
        bytes.extend_from_slice(&transition[..cut]);
        let journal = decode_journal(Path::new("journal.zrjournal"), &bytes).unwrap();
        assert!(
            journal.transitions.is_empty(),
            "a {cut}-byte torn frame must not publish its transition"
        );
    }

    let mut complete = intent;
    complete.extend_from_slice(&transition);
    let journal = decode_journal(Path::new("journal.zrjournal"), &complete).unwrap();
    assert_eq!(journal.transitions.len(), 1);
}

#[test]
fn final_checksum_failure_is_a_torn_transition_but_intent_corruption_is_fatal() {
    let intent = intent_frame();
    let mut transition = encode_frame(
        toml::to_string_pretty(&prepared_transition())
            .unwrap()
            .as_bytes(),
    )
    .unwrap();
    *transition.last_mut().unwrap() ^= 0xff;
    let mut bytes = intent.clone();
    bytes.extend_from_slice(&transition);

    let journal = decode_journal(Path::new("journal.zrjournal"), &bytes).unwrap();
    assert!(journal.transitions.is_empty());

    let mut corrupt_intent = intent;
    *corrupt_intent.last_mut().unwrap() ^= 0xff;
    assert!(decode_journal(Path::new("journal.zrjournal"), &corrupt_intent).is_err());
}

#[test]
fn torn_transition_tail_is_truncated_before_recovery_appends() {
    let root = test_directory("torn-transition-tail");
    let path = root.join("project.zrjournal");
    fs::create_dir_all(&root).unwrap();
    let mut bytes = intent_frame();
    let prepared = encode_frame(
        toml::to_string_pretty(&prepared_transition())
            .unwrap()
            .as_bytes(),
    )
    .unwrap();
    bytes.extend_from_slice(&prepared);
    let durable_len = bytes.len();
    let active = transition_frame(
        &path,
        JournalTransition {
            phase: JournalPhase::Active,
            document_index: None,
            state: None,
            target_existed: None,
            original_digest: None,
            new_digest: None,
            retired_digests: Vec::new(),
        },
        TransactionPhase::Recovery,
    )
    .unwrap();
    bytes.extend_from_slice(&active[..FRAME_HEADER_BYTES - 1]);
    fs::write(&path, &bytes).unwrap();

    let (_, decoded_len) = decode_journal_with_valid_len(&path, &bytes).unwrap();
    assert_eq!(decoded_len, durable_len);
    truncate_torn_tail(&path, decoded_len).unwrap();
    record_phase(&path, JournalPhase::Active).unwrap();

    let decoded = decode_journal(&path, &fs::read(&path).unwrap()).unwrap();
    assert_eq!(decoded.transitions.len(), 2);
    assert_eq!(decoded.fold().unwrap().phase, JournalPhase::Active);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn transition_and_commit_point_appends_cannot_exceed_the_journal_bound() {
    let root = test_directory("append-bound");
    let path = root.join("project.zrjournal");
    fs::create_dir_all(&root).unwrap();
    let file = fs::File::create(&path).unwrap();
    file.set_len(MAX_JOURNAL_BYTES as u64).unwrap();
    drop(file);

    let transition_error = record_phase(&path, JournalPhase::Active)
        .expect_err("a transition must not grow an already-full journal");
    let commit_point_error = record_commit_point(&path)
        .expect_err("the commit point must not grow an already-full journal");

    assert!(transition_error.to_string().contains("bounded size"));
    assert!(commit_point_error.to_string().contains("bounded size"));
    assert_eq!(fs::metadata(&path).unwrap().len(), MAX_JOURNAL_BYTES as u64);
    fs::remove_dir_all(root).unwrap();
}

fn intent_frame() -> Vec<u8> {
    let journal = TransactionJournal {
        version: JOURNAL_VERSION,
        tag: "project".to_owned(),
        transaction_id: TEST_TRANSACTION_ID.to_owned(),
        documents: vec![JournalIntent {
            target: PathBuf::from("C:/project/.zircon/registry/asset-registry.json"),
            staging: PathBuf::from("C:/project/.zircon/registry/.registry.zr-project-stage-1-1"),
            backup: PathBuf::from("C:/project/.zircon/registry/.registry.zr-project-backup-1-1"),
            rollback_staging: PathBuf::from(
                "C:/project/.zircon/registry/.registry.zr-project-rollback-stage-1-1",
            ),
            retirements: Vec::new(),
        }],
        transitions: Vec::new(),
    };
    encode_frame(toml::to_string_pretty(&journal).unwrap().as_bytes()).unwrap()
}

fn prepared_transition() -> TransitionAppend {
    TransitionAppend {
        transitions: vec![JournalTransition {
            phase: JournalPhase::Intent,
            document_index: Some(0),
            state: Some(JournalState::Prepared),
            target_existed: Some(true),
            original_digest: Some("old".to_owned()),
            new_digest: Some("new".to_owned()),
            retired_digests: Vec::new(),
        }],
    }
}

fn test_directory(name: &str) -> PathBuf {
    let output_root = std::env::var_os("ZIRCON_TEST_OUTPUT_ROOT")
        .or_else(|| std::env::var_os("CARGO_TARGET_DIR"))
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::current_dir()
                .expect("resolve current workspace for durable journal test output")
                .join("target")
        });
    output_root.join("zircon-test-output").join(format!(
        "zircon-durable-journal-{name}-{}-{}",
        std::process::id(),
        crate::io::next_test_output_id()
    ))
}
