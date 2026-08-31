use std::path::PathBuf;

pub(super) fn push_svg_variants(candidates: &mut Vec<PathBuf>, path: PathBuf) {
    if path.as_os_str().is_empty() {
        return;
    }
    if path.extension().is_some() {
        push_candidate(candidates, path);
        return;
    }
    push_candidate(candidates, path.with_extension("svg"));
    push_candidate(candidates, path);
}

pub(super) fn push_candidate(candidates: &mut Vec<PathBuf>, path: PathBuf) {
    if candidates.last() == Some(&path) {
        return;
    }
    if !candidates.iter().any(|candidate| candidate == &path) {
        candidates.push(path);
    }
}

#[cfg(test)]
#[path = "variants/last_candidate_tests.rs"]
mod last_candidate_tests;
