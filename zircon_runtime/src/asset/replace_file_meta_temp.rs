use std::path::Path;

/// Recognizes the short-lived Windows ReplaceFileW sibling names surfaced by asset watching.
/// Keep this to the observed `.zmeta~RF` + seven hexadecimal digits + `.TMP` form so ordinary
/// user files with similar suffixes remain visible.
pub(crate) fn is_replace_file_meta_temporary(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let Some(file_name) = file_name.strip_suffix(".TMP") else {
        return false;
    };
    let Some((_, random_suffix)) = file_name.rsplit_once(".zmeta~RF") else {
        return false;
    };
    random_suffix.len() == 7 && random_suffix.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
#[path = "tests/replace_file_meta_temp.rs"]
mod tests;
