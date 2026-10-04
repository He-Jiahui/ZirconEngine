//! 原子写入的同目录产物命名与识别；命名隔离角色，实际排他创建由暂存写入端承担。

use std::io;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};

use super::{path_entry, PathEntry};
use crate::io::artifact_identity::ArtifactSequence;

static ATOMIC_FILE_SEQUENCE: ArtifactSequence = ArtifactSequence::new();

pub(super) fn unique_sibling_path(
    directory: &Path,
    target: &Path,
    role: &str,
) -> io::Result<PathBuf> {
    unique_sibling_path_with_sequence(directory, target, role, &ATOMIC_FILE_SEQUENCE)
}

fn unique_sibling_path_with_sequence(
    directory: &Path,
    target: &Path,
    role: &str,
    sequence: &ArtifactSequence,
) -> io::Result<PathBuf> {
    let file_name = target_file_name(target);
    loop {
        let id = sequence.next().map_err(io::Error::other)?.get();
        let candidate = directory.join(format!(
            ".{file_name}.zr-{role}-{}-{id}",
            std::process::id()
        ));
        if path_entry(&candidate)? == PathEntry::Missing {
            return Ok(candidate);
        }
    }
}

pub(super) fn target_file_name(target: &Path) -> &str {
    target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("zircon.data")
}

/// 仅识别原子写入产物的名称形状；不证明文件所属事务，也不能单凭此结果授权删除或恢复。
pub fn is_atomic_write_transaction_path(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if !file_name.starts_with('.') {
        return false;
    }
    [".zr-staging-", ".zr-backup-"].into_iter().any(|marker| {
        let Some((_, suffix)) = file_name.rsplit_once(marker) else {
            return false;
        };
        let Some((process_id, sequence)) = suffix.split_once('-') else {
            return false;
        };
        !process_id.is_empty()
            && !sequence.is_empty()
            && process_id.bytes().all(|byte| byte.is_ascii_digit())
            && sequence.bytes().all(|byte| byte.is_ascii_digit())
            && sequence.parse::<NonZeroU64>().is_ok()
    })
}

#[cfg(test)]
#[path = "tests/pathing.rs"]
mod tests;
