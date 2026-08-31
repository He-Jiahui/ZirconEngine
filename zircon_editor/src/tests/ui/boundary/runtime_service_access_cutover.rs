use std::collections::BTreeSet;
use std::path::Path;

use super::support::collect_rust_files;

const CORE_HANDLE_OWNERS: &[&str] = &[
    "host/editor_asset_manager/handle.rs",
    "host/editor_manager.rs",
    "host/runtime_services.rs",
    "retained_host/app.rs",
    "retained_host/app/asset_runtime_access.rs",
    "retained_host/app/automation.rs",
    "retained_host/app/host_lifecycle/startup/constructors.rs",
    "retained_host/app/runtime_lease.rs",
    "retained_host/viewport/render_framework_access.rs",
];

const CORE_WEAK_OWNERS: &[&str] = &[
    "host/runtime_services.rs",
    "retained_host/app/asset_runtime_access.rs",
    "retained_host/app/runtime_lease.rs",
    "retained_host/viewport/render_framework_access.rs",
];

#[test]
fn ui_runtime_access_is_confined_to_composition_and_typed_service_owners() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ui_root = manifest_root.join("src").join("ui");
    let core_handle_owners = owner_set(CORE_HANDLE_OWNERS);
    let core_weak_owners = owner_set(CORE_WEAK_OWNERS);
    let mut violations = Vec::new();

    for path in collect_rust_files(&ui_root) {
        if is_test_owner(&path) {
            continue;
        }
        let relative = normalized_relative_path(&ui_root, &path);
        let source = std::fs::read_to_string(&path).expect("UI Rust source");
        let production = source.split("#[cfg(test)]").next().expect("source prefix");

        reject_unowned_token(
            &relative,
            production,
            "CoreHandle",
            &core_handle_owners,
            &mut violations,
        );
        reject_unowned_token(
            &relative,
            production,
            "CoreWeak",
            &core_weak_owners,
            &mut violations,
        );
        reject_token(&relative, production, "LevelSystem", &mut violations);
        reject_token(&relative, production, "ManagerResolver", &mut violations);

        let compact_source = production.split_whitespace().collect::<String>();
        for import in compact_source.split(';') {
            if import.contains("usezircon_runtime::") && import.contains("::*") {
                violations.push(format!(
                    "{relative}: wildcard runtime import can bypass the typed owner inventory: {import}"
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "UI runtime-service boundary violations:\n{}",
        violations.join("\n")
    );
}

fn reject_unowned_token(
    relative: &str,
    source: &str,
    token: &str,
    owners: &BTreeSet<&str>,
    violations: &mut Vec<String>,
) {
    if contains_word(source, token) && !owners.contains(relative) {
        violations.push(format!(
            "{relative}: `{token}` is only allowed in a composition or typed service-access owner"
        ));
    }
}

fn reject_token(relative: &str, source: &str, token: &str, violations: &mut Vec<String>) {
    if contains_word(source, token) {
        violations.push(format!(
            "{relative}: production UI must not depend on raw `{token}`"
        ));
    }
}

fn contains_word(source: &str, token: &str) -> bool {
    source.match_indices(token).any(|(index, _)| {
        let before = source[..index].chars().next_back();
        let after = source[index + token.len()..].chars().next();
        !before.is_some_and(is_identifier_char) && !after.is_some_and(is_identifier_char)
    })
}

fn is_identifier_char(character: char) -> bool {
    character == '_' || character.is_ascii_alphanumeric()
}

fn owner_set(owners: &'static [&'static str]) -> BTreeSet<&'static str> {
    owners.iter().copied().collect()
}

fn normalized_relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .expect("UI source path")
        .to_string_lossy()
        .replace('\\', "/")
}

fn is_test_owner(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == "tests.rs" || name.ends_with("_tests.rs"))
}
