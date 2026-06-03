use std::collections::BTreeMap;
use std::path::Path;

pub(super) fn assert_unique_row(
    relative_path: &Path,
    rows: &mut BTreeMap<String, String>,
    identity: &str,
    context: String,
) {
    if let Some(previous_context) = rows.insert(identity.to_string(), context.clone()) {
        panic!(
            "plugin manifest {relative_path:?} {context} identity `{identity}` should be unique; first declared by {previous_context}"
        );
    }
}
