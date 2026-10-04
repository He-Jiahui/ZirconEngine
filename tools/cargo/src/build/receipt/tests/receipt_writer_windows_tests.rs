use std::fs::{self, OpenOptions};
use std::io::Write;
use std::sync::{Arc, Barrier};
use std::time::{SystemTime, UNIX_EPOCH};

use super::{open_locked_temporary_receipt, publish_locked_receipt, write_new_json_with};

#[test]
fn publication_file_denies_replacement_until_its_handle_is_released() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "cargo-zircon-receipt-publication-lock-{}-{nonce}.tmp",
        std::process::id()
    ));
    let published = path.with_extension("published.json");
    let mut file = open_locked_temporary_receipt(&path).unwrap();
    file.write_all(b"{\"receipt\":true}").unwrap();
    file.sync_all().unwrap();

    assert!(OpenOptions::new().write(true).open(&path).is_err());
    assert!(fs::remove_file(&path).is_err());
    publish_locked_receipt(&file, &path, &published).unwrap();
    assert_eq!(fs::read(&published).unwrap(), b"{\"receipt\":true}");
    assert!(fs::remove_file(&published).is_err());
    assert!(OpenOptions::new().write(true).open(&published).is_err());
    assert!(fs::rename(&published, &path).is_err());

    drop(file);
    fs::remove_file(published).unwrap();
    assert!(!path.exists());
}

#[test]
fn competing_receipt_publications_keep_one_complete_output_and_clean_temporaries() {
    let directory = temporary_directory("race");
    let output = directory.join("receipt.json");
    let barrier = Arc::new(Barrier::new(2));
    let workers = [b"{\"receipt\":1}".as_slice(), b"{\"receipt\":2}".as_slice()]
        .into_iter()
        .map(|bytes| {
            let barrier = barrier.clone();
            let output = output.clone();
            std::thread::spawn(move || {
                write_new_json_with(&output, |file| {
                    file.write_all(bytes)?;
                    file.sync_all()?;
                    barrier.wait();
                    Ok(bytes)
                })
            })
        })
        .collect::<Vec<_>>();
    let results = workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .collect::<Vec<_>>();
    let winners = results
        .iter()
        .filter_map(|result| result.as_ref().ok())
        .collect::<Vec<_>>();
    assert_eq!(winners.len(), 1);
    assert_eq!(fs::read(&output).unwrap(), *winners[0]);
    assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn serialization_failure_cleans_the_temporary_file_and_allows_a_later_publication() {
    let directory = temporary_directory("write-failure");
    let output = directory.join("receipt.json");
    let failed = write_new_json_with::<()>(&output, |file| {
        file.write_all(b"partial")?;
        Err(std::io::Error::other("intentional write failure").into())
    });
    assert!(failed
        .unwrap_err()
        .to_string()
        .contains("intentional write failure"));
    assert_eq!(fs::read_dir(&directory).unwrap().count(), 0);
    super::write_new_json(&serde_json::json!({"receipt": "recovered"}), &output).unwrap();
    let receipt: serde_json::Value = serde_json::from_slice(&fs::read(&output).unwrap()).unwrap();
    assert_eq!(receipt["receipt"], "recovered");
    assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn serialization_panic_cleans_the_temporary_file() {
    let directory = temporary_directory("write-panic");
    let output = directory.join("receipt.json");
    let result = std::panic::catch_unwind(|| {
        write_new_json_with::<()>(&output, |file| {
            file.write_all(b"partial")?;
            panic!("intentional serializer panic");
        })
    });
    let remaining = fs::read_dir(&directory).unwrap().count();
    fs::remove_dir_all(directory).unwrap();
    assert!(result.is_err());
    assert_eq!(remaining, 0);
}

#[test]
fn publication_recovers_a_temporary_receipt_after_process_exit() {
    const CRASH_FIXTURE: &str = "CARGO_ZIRCON_RECEIPT_CRASH_FIXTURE";
    if let Some(directory) = std::env::var_os(CRASH_FIXTURE) {
        let output = std::path::PathBuf::from(directory).join("receipt.json");
        let _ = write_new_json_with::<()>(&output, |file| {
            file.write_all(b"partial")?;
            file.sync_all()?;
            std::process::exit(73);
        });
        unreachable!("child exits before publication");
    }
    let directory = temporary_directory("process-exit");
    let child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "build::receipt::receipt_writer::windows_tests::publication_recovers_a_temporary_receipt_after_process_exit",
            "--nocapture",
        ])
        .env(CRASH_FIXTURE, &directory)
        .output()
        .unwrap();
    assert_eq!(child.status.code(), Some(73));
    assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);
    let output = directory.join("receipt.json");
    super::write_new_json(&serde_json::json!({"receipt": "recovered"}), &output).unwrap();
    let remaining = fs::read_dir(&directory).unwrap().count();
    let receipt: serde_json::Value = serde_json::from_slice(&fs::read(output).unwrap()).unwrap();
    fs::remove_dir_all(directory).unwrap();
    assert_eq!(receipt["receipt"], "recovered");
    assert_eq!(remaining, 1);
}

#[test]
fn failed_temporary_cleanup_is_reported_with_the_write_failure() {
    let directory = temporary_directory("cleanup-failure");
    let output = directory.join("receipt.json");
    let error = write_new_json_with::<()>(&output, |file| {
        file.write_all(b"partial")?;
        let mut permissions = file.metadata()?.permissions();
        permissions.set_readonly(true);
        file.set_permissions(permissions)?;
        Err(std::io::Error::other("intentional write failure").into())
    })
    .unwrap_err();
    let leftovers = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    for path in &leftovers {
        let mut permissions = fs::metadata(path).unwrap().permissions();
        permissions.set_readonly(false);
        fs::set_permissions(path, permissions).unwrap();
    }
    fs::remove_dir_all(directory).unwrap();
    assert!(error.to_string().contains("intentional write failure"));
    assert!(error
        .to_string()
        .contains("temporary receipt cleanup failed"));
    assert_eq!(leftovers.len(), 1);
}

#[test]
fn recovery_preserves_active_and_unrelated_temporary_paths() {
    let directory = temporary_directory("recovery-ownership");
    let output = directory.join("receipt.json");
    let active_path = directory.join(".receipt.json.receipt-1-18446744073709551615.tmp");
    let mut active = open_locked_temporary_receipt(&active_path).unwrap();
    active.write_all(b"active writer").unwrap();
    let unrelated = directory.join(".receipt.json.receipt-unrelated.tmp");
    fs::write(&unrelated, b"unrelated file").unwrap();

    super::write_new_json(&serde_json::json!({"receipt": "published"}), &output).unwrap();

    assert_eq!(fs::read(&active_path).unwrap(), b"active writer");
    assert_eq!(fs::read(&unrelated).unwrap(), b"unrelated file");
    assert!(output.is_file());
    assert_eq!(fs::read_dir(&directory).unwrap().count(), 3);
    drop(active);
    fs::remove_dir_all(directory).unwrap();
}

fn temporary_directory(label: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "cargo-zircon-receipt-{label}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir(&directory).unwrap();
    directory
}
