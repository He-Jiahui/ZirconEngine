use std::cmp::Ordering;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};

pub(super) fn is_windows_drive_relative(path: &Path) -> bool {
    matches!(
        path.components().next(),
        Some(std::path::Component::Prefix(prefix))
            if matches!(
                prefix.kind(),
                std::path::Prefix::Disk(_) | std::path::Prefix::VerbatimDisk(_)
            )
    ) && !path.has_root()
}

pub(super) fn normalize_windows_final_path(path: PathBuf) -> PathBuf {
    const VERBATIM_PREFIX: &[u16] = &[b'\\' as u16, b'\\' as u16, b'?' as u16, b'\\' as u16];
    const VERBATIM_UNC_PREFIX: &[u16] = &[
        b'\\' as u16,
        b'\\' as u16,
        b'?' as u16,
        b'\\' as u16,
        b'U' as u16,
        b'N' as u16,
        b'C' as u16,
        b'\\' as u16,
    ];

    let wide = path.as_os_str().encode_wide().collect::<Vec<_>>();
    if wide_starts_with_ascii_case_insensitive(&wide, VERBATIM_UNC_PREFIX) {
        let mut normalized = vec![b'\\' as u16, b'\\' as u16];
        normalized.extend_from_slice(&wide[VERBATIM_UNC_PREFIX.len()..]);
        return PathBuf::from(std::ffi::OsString::from_wide(&normalized));
    }
    if wide.starts_with(VERBATIM_PREFIX) {
        let suffix = &wide[VERBATIM_PREFIX.len()..];
        if suffix.len() >= 2
            && ((b'A' as u16..=b'Z' as u16).contains(&suffix[0])
                || (b'a' as u16..=b'z' as u16).contains(&suffix[0]))
            && suffix[1] == b':' as u16
        {
            return PathBuf::from(std::ffi::OsString::from_wide(suffix));
        }
    }
    path
}

pub(super) fn wide_ascii_lowercase(value: u16) -> Option<u16> {
    const ASCII_UPPER_A: u16 = b'A' as u16;
    const ASCII_UPPER_Z: u16 = b'Z' as u16;
    const ASCII_LOWER_A: u16 = b'a' as u16;
    const ASCII_LOWER_Z: u16 = b'z' as u16;
    const ASCII_CASE_DELTA: u16 = ASCII_LOWER_A - ASCII_UPPER_A;

    if (ASCII_UPPER_A..=ASCII_UPPER_Z).contains(&value) {
        return Some(value + ASCII_CASE_DELTA);
    }
    if (ASCII_LOWER_A..=ASCII_LOWER_Z).contains(&value) {
        return Some(value);
    }
    None
}

pub(super) fn windows_os_str_equals_ascii_case_insensitive(
    value: &std::ffi::OsStr,
    expected: &str,
) -> bool {
    let mut value = value.encode_wide();
    let mut expected = expected.encode_utf16();
    loop {
        match (value.next(), expected.next()) {
            (None, None) => return true,
            (Some(actual), Some(expected))
                if actual == expected
                    || matches!(
                        (wide_ascii_lowercase(actual), wide_ascii_lowercase(expected)),
                        (Some(actual), Some(expected)) if actual == expected
                    ) => {}
            _ => return false,
        }
    }
}

pub(super) fn wide_starts_with_ascii_case_insensitive(path: &[u16], prefix: &[u16]) -> bool {
    path.get(..prefix.len()).is_some_and(|head| {
        head.iter().zip(prefix).all(|(actual, expected)| {
            actual == expected
                || matches!(
                    (wide_ascii_lowercase(*actual), wide_ascii_lowercase(*expected)),
                    (Some(actual), Some(expected)) if actual == expected
                )
        })
    })
}

fn compare_os_strings_ignore_case(left: &std::ffi::OsStr, right: &std::ffi::OsStr) -> Ordering {
    const CSTR_LESS_THAN: i32 = 1;
    const CSTR_EQUAL: i32 = 2;
    const CSTR_GREATER_THAN: i32 = 3;

    let left_wide = left.encode_wide().collect::<Vec<_>>();
    let right_wide = right.encode_wide().collect::<Vec<_>>();
    let (Ok(left_length), Ok(right_length)) = (
        i32::try_from(left_wide.len()),
        i32::try_from(right_wide.len()),
    ) else {
        return left.cmp(right);
    };

    // SAFETY: both pointers reference live UTF-16 buffers for the explicit converted lengths;
    // CompareStringOrdinal does not retain them and the final flag requests ordinal case folding.
    match unsafe {
        CompareStringOrdinal(
            left_wide.as_ptr(),
            left_length,
            right_wide.as_ptr(),
            right_length,
            1,
        )
    } {
        CSTR_LESS_THAN => Ordering::Less,
        CSTR_EQUAL => Ordering::Equal,
        CSTR_GREATER_THAN => Ordering::Greater,
        _ => left.cmp(right),
    }
}

pub(super) fn compare_paths_ignore_case(left: &Path, right: &Path) -> Ordering {
    compare_os_strings_ignore_case(left.as_os_str(), right.as_os_str())
}

pub(super) fn strip_path_prefix_ignore_case(path: &Path, root: &Path) -> Option<PathBuf> {
    let mut path_components = path.components();
    for root_component in root.components() {
        let path_component = path_components.next()?;
        if compare_os_strings_ignore_case(path_component.as_os_str(), root_component.as_os_str())
            != Ordering::Equal
        {
            return None;
        }
    }
    Some(path_components.collect())
}

pub(super) fn windows_paths_equal_ignore_case(left: &Path, right: &Path) -> bool {
    compare_paths_ignore_case(left, right) == Ordering::Equal
}

#[link(name = "kernel32")]
extern "system" {
    fn CompareStringOrdinal(
        left: *const u16,
        left_length: i32,
        right: *const u16,
        right_length: i32,
        ignore_case: i32,
    ) -> i32;
}
