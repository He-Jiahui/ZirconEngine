use super::*;

pub(super) fn priority_guard_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in PRIORITY_GUARD_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}
