use std::cmp::Ordering as CmpOrdering;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};

use crate::io::artifact_identity::{ArtifactIdentityExhausted, ArtifactSequence};

static TRANSACTION_SEQUENCE: ArtifactSequence = ArtifactSequence::new();

/// A physical operation path with platform-native ordering semantics.
///
/// Durable journals retain this path, not a caller-provided alias. The final directory entry is
/// preserved when it is a symlink so the transaction's existing non-link checks cannot be bypassed.
#[derive(Clone, Debug)]
pub(super) struct PathIdentity {
    operation_path: PathBuf,
    #[cfg(windows)]
    comparison_key: Vec<u16>,
}

impl PathIdentity {
    pub(super) fn resolve(path: &Path) -> io::Result<Self> {
        let operation_path = resolve_operation_path(path)?;
        Self::from_operation_path(operation_path)
    }

    fn from_operation_path(operation_path: PathBuf) -> io::Result<Self> {
        #[cfg(windows)]
        let comparison_key = {
            use std::os::windows::ffi::OsStrExt;

            let key = operation_path.as_os_str().encode_wide().collect::<Vec<_>>();
            i32::try_from(key.len()).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "transaction path exceeds the Windows comparison limit",
                )
            })?;
            key
        };
        Ok(Self {
            operation_path,
            #[cfg(windows)]
            comparison_key,
        })
    }

    pub(super) fn strict_ancestor_identities(&self) -> io::Result<Vec<Self>> {
        self.operation_path
            .ancestors()
            .skip(1)
            .map(|path| Self::from_operation_path(path.to_path_buf()))
            .collect()
    }

    pub(super) fn compare_namespace_paths(left: &Self, right: &Self) -> CmpOrdering {
        #[cfg(windows)]
        {
            return compare_windows_namespace_paths(&left.comparison_key, &right.comparison_key)
                .then_with(|| compare_windows_paths(&left.comparison_key, &right.comparison_key));
        }
        #[cfg(not(windows))]
        {
            let mut left_components = left.operation_path.components();
            let mut right_components = right.operation_path.components();
            loop {
                match (left_components.next(), right_components.next()) {
                    (Some(left), Some(right)) => {
                        let ordering = left.as_os_str().cmp(right.as_os_str());
                        if ordering != CmpOrdering::Equal {
                            return ordering;
                        }
                    }
                    (None, None) => return left.operation_path.cmp(&right.operation_path),
                    (None, Some(_)) => return CmpOrdering::Less,
                    (Some(_), None) => return CmpOrdering::Greater,
                }
            }
        }
    }

    pub(super) fn operation_path(&self) -> &Path {
        &self.operation_path
    }

    pub(super) fn has_exact_operation_path_encoding(&self, path: &Path) -> bool {
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;

            return self
                .operation_path
                .as_os_str()
                .encode_wide()
                .eq(path.as_os_str().encode_wide());
        }
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;

            return self.operation_path.as_os_str().as_bytes() == path.as_os_str().as_bytes();
        }
        #[cfg(not(any(unix, windows)))]
        {
            self.operation_path.as_os_str() == path.as_os_str()
        }
    }

    pub(super) fn into_operation_path(self) -> PathBuf {
        self.operation_path
    }

    pub(super) fn is_same_or_descendant_of(&self, ancestor: &Self) -> bool {
        #[cfg(windows)]
        {
            return windows_path_is_same_or_descendant(
                &self.comparison_key,
                &ancestor.comparison_key,
            );
        }
        #[cfg(not(windows))]
        self.operation_path.starts_with(&ancestor.operation_path)
    }
}

impl PartialEq for PathIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == CmpOrdering::Equal
    }
}

impl Eq for PathIdentity {}

impl PartialOrd for PathIdentity {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathIdentity {
    fn cmp(&self, other: &Self) -> CmpOrdering {
        #[cfg(windows)]
        {
            return compare_windows_paths(&self.comparison_key, &other.comparison_key);
        }
        #[cfg(not(windows))]
        self.operation_path.cmp(&other.operation_path)
    }
}

pub(super) fn next_transaction_id(
    journal_directory: &Path,
) -> Result<String, ArtifactIdentityExhausted> {
    next_transaction_id_with_sequence(journal_directory, &TRANSACTION_SEQUENCE)
}

fn next_transaction_id_with_sequence(
    journal_directory: &Path,
    sequence: &ArtifactSequence,
) -> Result<String, ArtifactIdentityExhausted> {
    let sequence = sequence.next()?.get();
    Ok(format!(
        "{}-{}-{sequence}",
        journal_owner_token(journal_directory),
        std::process::id()
    ))
}

pub(super) fn valid_transaction_id(value: &str, journal_directory: &PathIdentity) -> bool {
    let mut parts = value.split('-');
    parts.next().is_some_and(|part| {
        part.len() == 64
            && part
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            && part == journal_owner_token(journal_directory.operation_path())
    }) && parts.next().is_some_and(|part| {
        !part.is_empty()
            && part.bytes().all(|byte| byte.is_ascii_digit())
            && part.parse::<u32>().is_ok()
    }) && parts.next().is_some_and(|part| {
        !part.is_empty()
            && part.bytes().all(|byte| byte.is_ascii_digit())
            && part.parse::<NonZeroU64>().is_ok()
    }) && parts.next().is_none()
}

pub(super) fn journal_owner_token(journal_directory: &Path) -> String {
    path_encoding_token(journal_directory)
}

#[cfg(test)]
pub(super) fn transaction_id_for_test(journal_directory: &Path, sequence: u64) -> String {
    let sequence = NonZeroU64::new(sequence).expect("test transaction sequence must be nonzero");
    let journal_directory =
        PathIdentity::resolve(journal_directory).expect("resolve test journal owner identity");
    format!(
        "{}-{}-{}",
        journal_owner_token(journal_directory.operation_path()),
        std::process::id(),
        sequence
    )
}

pub(super) fn valid_tag(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 32
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

pub(super) fn transaction_sibling(
    owner: &Path,
    tag: &str,
    role: &str,
    transaction_id: &str,
) -> PathBuf {
    let parent = owner.parent().unwrap_or_else(|| Path::new("."));
    let token = basename_token(owner);
    parent.join(format!(".{token}.zr-{tag}-{role}-{transaction_id}"))
}

pub(super) fn journal_path(
    directory: &Path,
    first_target: &Path,
    tag: &str,
    transaction_id: &str,
) -> PathBuf {
    let token = basename_token(first_target);
    directory.join(format!(
        ".{token}.zr-{tag}-journal-{transaction_id}.zrjournal"
    ))
}

fn basename_token(owner: &Path) -> String {
    let name = owner
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new(""));
    path_encoding_token(Path::new(name))
}

fn path_encoding_token(path: &Path) -> String {
    let mut hasher = blake3::Hasher::new();
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        hasher.update(path.as_os_str().as_bytes());
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        for unit in path.as_os_str().encode_wide() {
            hasher.update(&unit.to_le_bytes());
        }
    }
    #[cfg(not(any(unix, windows)))]
    hasher.update(path.to_string_lossy().as_bytes());
    hasher.finalize().to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const OPAQUE_TRANSACTION_ID: &str =
        "0000000000000000000000000000000000000000000000000000000000000000-1-1";

    fn test_directory(label: &str) -> PathBuf {
        let output_root = std::env::var_os("ZIRCON_TEST_OUTPUT_ROOT")
            .or_else(|| std::env::var_os("CARGO_TARGET_DIR"))
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap().join("target"));
        output_root.join("zircon-test-output").join(format!(
            "transaction-identity-{label}-{}-{}",
            std::process::id(),
            crate::io::next_test_output_id()
        ))
    }

    #[test]
    fn transaction_identity_is_partitioned_by_canonical_journal_owner() {
        let root = test_directory("owner-partition");
        let first_owner = root.join("first-owner");
        let second_owner = root.join("second-owner");
        fs::create_dir_all(&first_owner).unwrap();
        fs::create_dir_all(&second_owner).unwrap();
        let first_sequence = ArtifactSequence::starting_at(7);
        let second_sequence = ArtifactSequence::starting_at(7);
        let first_identity = PathIdentity::resolve(&first_owner).unwrap();
        let second_identity = PathIdentity::resolve(&second_owner).unwrap();
        let first =
            next_transaction_id_with_sequence(first_identity.operation_path(), &first_sequence)
                .unwrap();
        let second =
            next_transaction_id_with_sequence(second_identity.operation_path(), &second_sequence)
                .unwrap();

        assert_ne!(first, second);
        assert!(valid_transaction_id(&first, &first_identity));
        assert!(valid_transaction_id(&second, &second_identity));
        assert!(!valid_transaction_id(&first, &second_identity));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn transaction_identity_parser_rejects_legacy_and_malformed_wires() {
        let owner = test_directory("wire-validation");
        fs::create_dir_all(&owner).unwrap();
        let identity = PathIdentity::resolve(&owner).unwrap();
        let token = journal_owner_token(identity.operation_path());

        assert!(!valid_transaction_id("42-1", &identity));
        assert!(!valid_transaction_id(&format!("{token}-42-0"), &identity));
        assert!(!valid_transaction_id(
            &format!("{}-42-1", token.to_uppercase()),
            &identity
        ));
        assert!(!valid_transaction_id(
            &format!("{}-42-1", &token[..32]),
            &identity
        ));
        assert!(!valid_transaction_id(
            &format!("{token}-42-1-extra"),
            &identity
        ));
        assert!(valid_transaction_id(&format!("{token}-42-1"), &identity));
        fs::remove_dir_all(owner).unwrap();
    }

    #[test]
    fn transaction_identity_exhaustion_is_terminal() {
        let owner = Path::new("journal-owner");
        let sequence = ArtifactSequence::starting_at(u64::MAX);

        let final_identity = next_transaction_id_with_sequence(owner, &sequence).unwrap();

        assert!(final_identity.ends_with(&format!("-{}-{}", std::process::id(), u64::MAX)));
        assert_eq!(
            next_transaction_id_with_sequence(owner, &sequence),
            Err(ArtifactIdentityExhausted)
        );
    }

    #[cfg(unix)]
    #[test]
    fn basename_tokens_distinguish_non_unicode_basenames() {
        use std::os::unix::ffi::OsStringExt;

        let parent = Path::new("/tmp");
        let first = parent.join(OsString::from_vec(vec![b'a', 0x80]));
        let second = parent.join(OsString::from_vec(vec![b'a', 0x81]));
        let literal = parent.join("zircon.data");

        let first_artifact = transaction_sibling(&first, "project", "stage", OPAQUE_TRANSACTION_ID);
        let second_artifact =
            transaction_sibling(&second, "project", "stage", OPAQUE_TRANSACTION_ID);
        let literal_artifact =
            transaction_sibling(&literal, "project", "stage", OPAQUE_TRANSACTION_ID);

        assert_ne!(first_artifact, second_artifact);
        assert_ne!(first_artifact, literal_artifact);
        assert_ne!(second_artifact, literal_artifact);
    }

    #[cfg(windows)]
    #[test]
    fn basename_tokens_distinguish_non_unicode_basenames() {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;

        let parent = Path::new(r"C:\zircon");
        let first = parent.join(OsString::from_wide(&[0xd800]));
        let second = parent.join(OsString::from_wide(&[0xd801]));
        let literal = parent.join("zircon.data");

        let first_artifact = transaction_sibling(&first, "project", "stage", OPAQUE_TRANSACTION_ID);
        let second_artifact =
            transaction_sibling(&second, "project", "stage", OPAQUE_TRANSACTION_ID);
        let literal_artifact =
            transaction_sibling(&literal, "project", "stage", OPAQUE_TRANSACTION_ID);

        assert_ne!(first_artifact, second_artifact);
        assert_ne!(first_artifact, literal_artifact);
        assert_ne!(second_artifact, literal_artifact);
    }

    #[test]
    fn split_at_deepest_existing_ancestor_scans_from_leaf() {
        let output_root = std::env::var_os("ZIRCON_TEST_OUTPUT_ROOT")
            .or_else(|| std::env::var_os("CARGO_TARGET_DIR"))
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap().join("target"));
        let root = output_root.join("zircon-test-output").join(format!(
            "zircon-resource-pathing-{}-{}",
            std::process::id(),
            crate::io::next_test_output_id()
        ));
        let existing = root.join("existing").join("branch");
        fs::create_dir_all(&existing).unwrap();
        let missing = existing.join("new").join("nested").join("asset.zmeta");

        let (ancestor, tail) = split_at_deepest_existing_ancestor(&missing).unwrap();

        assert_eq!(ancestor, existing);
        assert_eq!(
            tail,
            vec![
                OsString::from("new"),
                OsString::from("nested"),
                OsString::from("asset.zmeta"),
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn split_at_deepest_existing_ancestor_preserves_parent_components() {
        let output_root = std::env::var_os("ZIRCON_TEST_OUTPUT_ROOT")
            .or_else(|| std::env::var_os("CARGO_TARGET_DIR"))
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap().join("target"));
        let root = output_root.join("zircon-test-output").join(format!(
            "zircon-resource-parent-pathing-{}-{}",
            std::process::id(),
            crate::io::next_test_output_id()
        ));
        let existing = root.join("existing");
        fs::create_dir_all(&existing).unwrap();
        let mut missing = existing.as_os_str().to_os_string();
        missing.push(std::path::MAIN_SEPARATOR_STR);
        missing.push("missing");
        missing.push(std::path::MAIN_SEPARATOR_STR);
        missing.push("..");
        missing.push(std::path::MAIN_SEPARATOR_STR);
        missing.push("asset.zmeta");
        let missing = PathBuf::from(missing);

        let (ancestor, tail) = split_at_deepest_existing_ancestor(&missing).unwrap();

        assert_eq!(ancestor, existing);
        assert_eq!(
            tail,
            vec![
                OsString::from("missing"),
                OsString::from(".."),
                OsString::from("asset.zmeta"),
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }
}

fn resolve_operation_path(path: &Path) -> io::Result<PathBuf> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            let parent = path
                .parent()
                .ok_or_else(|| invalid_path("path has no parent"))?;
            let name = path
                .file_name()
                .ok_or_else(|| invalid_path("path has no final directory entry"))?;
            fs::canonicalize(parent).map(|parent| parent.join(name))
        }
        Ok(_) => fs::canonicalize(path),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            resolve_path_below_existing_ancestor(path)
        }
        Err(error) => Err(error),
    }
}

fn resolve_path_below_existing_ancestor(path: &Path) -> io::Result<PathBuf> {
    let (existing, tail) = split_at_deepest_existing_ancestor(path)?;
    let mut resolved = fs::canonicalize(existing)?;
    for component in tail {
        append_uncreated_component(&mut resolved, component);
    }
    Ok(resolved)
}

fn split_at_deepest_existing_ancestor(path: &Path) -> io::Result<(PathBuf, Vec<OsString>)> {
    // Win32 may resolve `missing\..` before metadata runs, so retain the raw lexical tail.
    let mut candidate = path.to_path_buf();
    let mut unresolved_tail = Vec::new();

    loop {
        let (candidate_parent, trailing_normal_component) = match split_lexical_tail(&candidate) {
            Some((parent, component)) if component == ".." => {
                unresolved_tail.push(OsString::from(".."));
                candidate = parent;
                continue;
            }
            Some((parent, component)) if component == "." => {
                unresolved_tail.push(OsString::from("."));
                candidate = parent;
                continue;
            }
            Some((parent, component)) => (Some(parent), Some(component)),
            None => (None, None),
        };
        match fs::metadata(&candidate) {
            Ok(_) => {
                unresolved_tail.reverse();
                return Ok((candidate, unresolved_tail));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let component = trailing_normal_component
                    .ok_or_else(|| invalid_path("path has no existing physical ancestor"))?;
                unresolved_tail.push(component);
                candidate = candidate_parent
                    .ok_or_else(|| invalid_path("path has no existing physical ancestor"))?;
            }
            Err(error) => return Err(error),
        }
    }
}

#[cfg(windows)]
fn split_lexical_tail(path: &Path) -> Option<(PathBuf, OsString)> {
    use std::os::windows::ffi::{OsStrExt, OsStringExt};

    let encoded = path.as_os_str().encode_wide().collect::<Vec<_>>();
    let mut end = encoded.len();
    while end > 0 && (encoded[end - 1] == u16::from(b'\\') || encoded[end - 1] == u16::from(b'/')) {
        end -= 1;
    }
    let mut start = end;
    while start > 0
        && encoded[start - 1] != u16::from(b'\\')
        && encoded[start - 1] != u16::from(b'/')
    {
        start -= 1;
    }
    if start == end || (start == 0 && path.has_root()) {
        return None;
    }
    Some((
        PathBuf::from(OsString::from_wide(&encoded[..start])),
        OsString::from_wide(&encoded[start..end]),
    ))
}

#[cfg(unix)]
fn split_lexical_tail(path: &Path) -> Option<(PathBuf, OsString)> {
    use std::os::unix::ffi::{OsStrExt, OsStringExt};

    let encoded = path.as_os_str().as_bytes();
    let mut end = encoded.len();
    while end > 0 && encoded[end - 1] == b'/' {
        end -= 1;
    }
    let mut start = end;
    while start > 0 && encoded[start - 1] != b'/' {
        start -= 1;
    }
    if start == end || (start == 0 && path.has_root()) {
        return None;
    }
    Some((
        PathBuf::from(OsString::from_vec(encoded[..start].to_vec())),
        OsString::from_vec(encoded[start..end].to_vec()),
    ))
}

#[cfg(not(any(unix, windows)))]
fn split_lexical_tail(path: &Path) -> Option<(PathBuf, OsString)> {
    Some((
        path.parent()?.to_path_buf(),
        path.file_name()?.to_os_string(),
    ))
}

fn append_uncreated_component(path: &mut PathBuf, component: OsString) {
    if component == "." {
        return;
    }
    if component == ".." {
        let _ = path.pop();
        return;
    }
    path.push(component);
}

fn invalid_path(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

#[cfg(windows)]
fn compare_windows_paths(left: &[u16], right: &[u16]) -> CmpOrdering {
    const CSTR_LESS_THAN: i32 = 1;
    const CSTR_EQUAL: i32 = 2;
    const CSTR_GREATER_THAN: i32 = 3;

    let left_length = i32::try_from(left.len()).expect("validated Windows path length");
    let right_length = i32::try_from(right.len()).expect("validated Windows path length");
    let comparison = unsafe {
        CompareStringOrdinal(left.as_ptr(), left_length, right.as_ptr(), right_length, 1)
    };
    match comparison {
        CSTR_LESS_THAN => CmpOrdering::Less,
        CSTR_EQUAL => CmpOrdering::Equal,
        CSTR_GREATER_THAN => CmpOrdering::Greater,
        _ => left.cmp(right),
    }
}

#[cfg(windows)]
fn windows_path_is_same_or_descendant(path: &[u16], ancestor: &[u16]) -> bool {
    if path.len() < ancestor.len()
        || compare_windows_paths(&path[..ancestor.len()], ancestor) != CmpOrdering::Equal
    {
        return false;
    }
    path.len() == ancestor.len()
        || ancestor
            .last()
            .is_some_and(|unit| windows_path_separator(*unit))
        || windows_path_separator(path[ancestor.len()])
}

#[cfg(windows)]
fn compare_windows_namespace_paths(left: &[u16], right: &[u16]) -> CmpOrdering {
    let mut left_start = 0;
    let mut right_start = 0;
    loop {
        while left_start < left.len() && windows_path_separator(left[left_start]) {
            left_start += 1;
        }
        while right_start < right.len() && windows_path_separator(right[right_start]) {
            right_start += 1;
        }
        let left_end = left[left_start..]
            .iter()
            .position(|unit| windows_path_separator(*unit))
            .map_or(left.len(), |offset| left_start + offset);
        let right_end = right[right_start..]
            .iter()
            .position(|unit| windows_path_separator(*unit))
            .map_or(right.len(), |offset| right_start + offset);
        let ordering =
            compare_windows_paths(&left[left_start..left_end], &right[right_start..right_end]);
        if ordering != CmpOrdering::Equal {
            return ordering;
        }
        let left_done = left_end == left.len();
        let right_done = right_end == right.len();
        match (left_done, right_done) {
            (true, true) => return CmpOrdering::Equal,
            (true, false) => return CmpOrdering::Less,
            (false, true) => return CmpOrdering::Greater,
            (false, false) => {
                left_start = left_end + 1;
                right_start = right_end + 1;
            }
        }
    }
}

#[cfg(windows)]
fn windows_path_separator(unit: u16) -> bool {
    unit == u16::from(b'\\') || unit == u16::from(b'/')
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn CompareStringOrdinal(
        left: *const u16,
        left_length: i32,
        right: *const u16,
        right_length: i32,
        ignore_case: i32,
    ) -> i32;
}
