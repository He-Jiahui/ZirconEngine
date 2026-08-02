use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::core::resource::io::atomic_file::{atomic_write, stage_atomic_write};

pub(super) struct PreparedFileWrite {
    pub(super) path: PathBuf,
    pub(super) bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum TargetedTransactionFault {
    #[default]
    None,
    #[cfg(test)]
    BeforeCommit(usize),
}

pub(super) fn commit_prepared_files(
    writes: Vec<PreparedFileWrite>,
    fault: TargetedTransactionFault,
) -> io::Result<()> {
    reject_duplicate_targets(&writes)?;
    let originals = writes
        .iter()
        .map(|write| match fs::read(&write.path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        })
        .collect::<io::Result<Vec<_>>>()?;
    let staged = writes
        .iter()
        .map(|write| stage_atomic_write(&write.path, &write.bytes))
        .collect::<io::Result<Vec<_>>>()?;

    let mut committed = 0;
    for (index, pending) in staged.into_iter().enumerate() {
        #[cfg(test)]
        if fault == TargetedTransactionFault::BeforeCommit(index) {
            rollback(&writes, &originals, committed)?;
            return Err(io::Error::other(format!(
                "injected targeted transaction failure before file {index}"
            )));
        }
        #[cfg(not(test))]
        let _ = fault;
        if let Err(error) = pending.commit() {
            rollback(&writes, &originals, committed)?;
            return Err(error);
        }
        committed += 1;
    }
    Ok(())
}

fn reject_duplicate_targets(writes: &[PreparedFileWrite]) -> io::Result<()> {
    let mut targets = HashSet::with_capacity(writes.len());
    for write in writes {
        if !targets.insert(&write.path) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "targeted transaction contains duplicate target {}",
                    write.path.display()
                ),
            ));
        }
    }
    Ok(())
}

fn rollback(
    writes: &[PreparedFileWrite],
    originals: &[Option<Vec<u8>>],
    committed: usize,
) -> io::Result<()> {
    let mut first_error = None;
    for (write, original) in writes[..committed]
        .iter()
        .zip(&originals[..committed])
        .rev()
    {
        let result = match original {
            Some(bytes) => atomic_write(&write.path, bytes),
            None => match fs::remove_file(&write.path) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(error),
            },
        };
        if first_error.is_none() {
            first_error = result.err();
        }
    }
    first_error.map_or(Ok(()), Err)
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn later_commit_failure_restores_every_previously_published_target() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zircon_targeted_transaction_{}_{}",
            std::process::id(),
            unique
        ));
        fs::create_dir_all(&root).unwrap();
        let first = root.join("first.data");
        let second = root.join("second.data");
        fs::write(&first, b"first-original").unwrap();
        fs::write(&second, b"second-original").unwrap();

        commit_prepared_files(
            vec![
                PreparedFileWrite {
                    path: first.clone(),
                    bytes: b"first-replacement".to_vec(),
                },
                PreparedFileWrite {
                    path: second.clone(),
                    bytes: b"second-replacement".to_vec(),
                },
            ],
            TargetedTransactionFault::BeforeCommit(1),
        )
        .unwrap_err();

        assert_eq!(fs::read(&first).unwrap(), b"first-original");
        assert_eq!(fs::read(&second).unwrap(), b"second-original");
        assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
        fs::remove_dir_all(root).unwrap();
    }
}
