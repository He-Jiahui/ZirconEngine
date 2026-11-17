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
    loop {
        let id = sequence.next().map_err(io::Error::other)?;
        let candidate = directory.join(sibling_file_name(target, role, std::process::id(), id)?);
        if path_entry(&candidate)? == PathEntry::Missing {
            return Ok(candidate);
        }
    }
}

fn sibling_file_name(
    target: &Path,
    role: &str,
    process_id: u32,
    id: NonZeroU64,
) -> io::Result<String> {
    let legacy = format!(".{}.zr-{role}-{process_id}-{id}", target_file_name(target));
    // Budget the actual candidate after allocating its checked identity, not a truncated target
    // or nonce. Names that fit retain their exact legacy spelling, including the old fallback.
    #[cfg(any(windows, unix))]
    if !component_within_budget(&legacy) {
        let owner = native_leaf_token(target).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "atomic target has no final filename",
            )
        })?;
        let class = if role == "staging"
            && target
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".zrjournal"))
        {
            ".zrjournal"
        } else {
            ""
        };
        return Ok(format!(".{owner}{class}.zr2-{role}-{process_id}-{id}"));
    }
    Ok(legacy)
}

#[cfg(any(windows, unix))]
fn component_within_budget(name: &str) -> bool {
    #[cfg(windows)]
    let length = name.encode_utf16().count();
    #[cfg(unix)]
    let length = name.len();
    length <= 255
}

// Hash only the complete native final component. This is an exact spelling token, not a
// filesystem alias identity, and must never use the legacy fallback or a lossy conversion.
#[cfg(any(windows, unix))]
pub(super) fn native_leaf_token(target: &Path) -> Option<String> {
    let name = target.file_name()?;
    let mut hasher = blake3::Hasher::new();
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        hasher.update(name.as_bytes());
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        for unit in name.encode_wide() {
            hasher.update(&unit.to_le_bytes());
        }
    }
    Some(hasher.finalize().to_hex().to_string())
}

// 回退名仅用于同目录临时产物；实际目标保留原 Path，不能用该名推断目标身份。
pub(super) fn target_file_name(target: &Path) -> &str {
    target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("zircon.data")
}

/// 仅识别原子写入产物的名称形状；不证明文件所属事务，也不能单凭此结果授权删除或恢复。
pub fn is_atomic_write_transaction_path(path: &Path) -> bool {
    is_atomic_write_transaction_path_legacy(path) || compact_frame(path).is_some()
}

// Keep the old predicate separate and unchanged: expanding the public shape query must not
// grant compact names the legacy recovery prefix's owner/role authority.
pub(super) fn is_atomic_write_transaction_path_legacy(path: &Path) -> bool {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CompactRole {
    Staging,
    Backup,
}

// The frame is (native owner token, exact role, journal class); numeric fields are validated
// in full here even when the caller needs only its shape or class.
fn compact_frame(path: &Path) -> Option<(&str, CompactRole, bool)> {
    let name = path.file_name()?.to_str()?.strip_prefix('.')?;
    let (owner, artifact) = name.rsplit_once(".zr2-")?;
    let (owner, journal) = match owner.strip_suffix(".zrjournal") {
        Some(owner) => (owner, true),
        None => (owner, false),
    };
    if owner.len() != 64
        || !owner
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return None;
    }
    let (role, nonce) = if let Some(nonce) = artifact.strip_prefix("staging-") {
        (CompactRole::Staging, nonce)
    } else {
        (CompactRole::Backup, artifact.strip_prefix("backup-")?)
    };
    if journal && role != CompactRole::Staging {
        return None;
    }
    let (process_id, sequence) = nonce.split_once('-')?;
    if process_id.parse::<u32>().ok()?.to_string() != process_id
        || sequence.parse::<NonZeroU64>().ok()?.to_string() != sequence
    {
        return None;
    }
    Some((owner, role, journal))
}

#[cfg(windows)]
pub(super) fn is_compact_backup_for_owner(path: &Path, owner: &str) -> bool {
    compact_frame(path).is_some_and(|(candidate_owner, role, journal)| {
        !journal && role == CompactRole::Backup && candidate_owner == owner
    })
}

// This preserves only the legacy journal class for strict compact staging. It is not proof
// of transaction ownership; durable recovery still validates the entire directory first.
pub(in crate::io) fn is_compact_journal_staging_path(path: &Path) -> bool {
    compact_frame(path).is_some_and(|(_, role, journal)| journal && role == CompactRole::Staging)
}

#[cfg(test)]
pub(in crate::io) fn sibling_file_name_for_test(
    target: &Path,
    role: &str,
    process_id: u32,
    id: NonZeroU64,
) -> io::Result<String> {
    sibling_file_name(target, role, process_id, id)
}

#[cfg(test)]
#[path = "tests/pathing.rs"]
mod tests;
