use std::collections::BTreeMap;
use std::path::Path;

pub(super) fn assert_unique_provider_row(
    relative_path: &Path,
    rows: &mut BTreeMap<String, String>,
    feature_id: &str,
    provider_package_id: &str,
    context: &str,
) {
    let key = format!("{feature_id}:{provider_package_id}");
    if let Some(previous_context) = rows.insert(key.clone(), context.to_string()) {
        panic!(
            "plugin manifest {relative_path:?} feature-extension provider `{key}` should be unique; first declared by {previous_context}, repeated by {context}"
        );
    }
}

pub(super) fn assert_unique_identity(
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
