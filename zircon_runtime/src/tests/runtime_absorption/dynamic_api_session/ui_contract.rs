use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const RUNTIME_UI_ROOT: &str = "zircon_runtime/src/ui";
const INTERFACE_UI_ROOT: &str = "zircon_runtime_interface/src/ui";
const RUNTIME_10_UI_SINGLE_SOURCE_STATUS: &str =
    "runtime_10_m2_1_ui_contract_duplicate_public_types_removed_static_passed_cargo_pending";

#[test]
fn runtime_10_ui_contract_types_have_single_definition_across_interface_and_runtime() {
    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = runtime_root
        .parent()
        .expect("runtime crate should live under the repository root");

    let duplicates = duplicate_public_ui_types(repo_root);
    assert!(
        duplicates.is_empty(),
        "Runtime/interface UI public struct or enum names should have a single owner; duplicates: {}",
        duplicates
            .iter()
            .map(|duplicate| duplicate.describe(repo_root))
            .collect::<Vec<_>>()
            .join("; ")
    );

    assert_no_runtime_duplicate_source(repo_root, "event_ui/codec.rs", "UiBindingCodec");
    assert_no_runtime_duplicate_source(
        repo_root,
        "template/asset/schema/policy.rs",
        "UiAssetSchemaVersionPolicy",
    );

    for (relative_file, required_anchor) in [
        (
            "zircon_runtime_interface/src/ui/event_ui/codec.rs",
            "pub struct UiBindingCodec",
        ),
        (
            "zircon_runtime_interface/src/ui/template/asset/schema/policy.rs",
            "pub struct UiAssetSchemaVersionPolicy",
        ),
    ] {
        let source = fs::read_to_string(repo_root.join(relative_file))
            .unwrap_or_else(|error| panic!("`{relative_file}` should be readable: {error}"));
        assert!(
            source.contains(required_anchor),
            "`{relative_file}` should keep interface-owned UI contract anchor `{required_anchor}`"
        );
    }

    for relative_doc in [
        "docs/plans/zircon_runtime/runtime/10/2026-07-09-dynamic-api-and-interface-convergence-output-records.md",
        "docs/plans/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
        "docs/engine-architecture/runtime-interface-convergence.md",
        "docs/engine-architecture/runtime-architecture-review-m0.md",
    ] {
        let doc = fs::read_to_string(repo_root.join(relative_doc))
            .unwrap_or_else(|error| panic!("`{relative_doc}` should be readable: {error}"));
        assert!(
            doc.contains(RUNTIME_10_UI_SINGLE_SOURCE_STATUS),
            "`{relative_doc}` should record Runtime 10 UI single-source contract status `{RUNTIME_10_UI_SINGLE_SOURCE_STATUS}`"
        );
        assert!(
            doc.contains("ui_contract_duplicate_public_types = 0"),
            "`{relative_doc}` should record the current UI contract duplicate count"
        );
    }
}

fn assert_no_runtime_duplicate_source(repo_root: &Path, ui_relative_path: &str, type_name: &str) {
    let path = repo_root.join(RUNTIME_UI_ROOT).join(ui_relative_path);
    assert!(
        !path.exists(),
        "runtime-local duplicate `{type_name}` should stay removed from `{}`",
        path.display()
    );
}

fn duplicate_public_ui_types(repo_root: &Path) -> Vec<PublicTypeDuplicate> {
    let runtime_types = collect_public_types(&repo_root.join(RUNTIME_UI_ROOT));
    let interface_types = collect_public_types(&repo_root.join(INTERFACE_UI_ROOT));
    let mut duplicates = Vec::new();

    for (key, runtime_locations) in runtime_types {
        if let Some(interface_locations) = interface_types.get(&key) {
            for runtime in &runtime_locations {
                for interface in interface_locations {
                    duplicates.push(PublicTypeDuplicate {
                        kind: key.kind.clone(),
                        name: key.name.clone(),
                        runtime: runtime.clone(),
                        interface: interface.clone(),
                    });
                }
            }
        }
    }

    duplicates
}

fn collect_public_types(root: &Path) -> BTreeMap<PublicTypeKey, Vec<PathBuf>> {
    let mut types = BTreeMap::new();
    collect_public_types_inner(root, root, &mut types);
    types
}

fn collect_public_types_inner(
    root: &Path,
    current: &Path,
    types: &mut BTreeMap<PublicTypeKey, Vec<PathBuf>>,
) {
    let Ok(entries) = fs::read_dir(current) else {
        return;
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            collect_public_types_inner(root, &path, types);
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }

        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("`{}` should be readable: {error}", path.display()));
        for key in public_type_keys(&source) {
            types
                .entry(key)
                .or_insert_with(Vec::new)
                .push(path.strip_prefix(root).unwrap_or(&path).to_path_buf());
        }
    }
}

fn public_type_keys(source: &str) -> Vec<PublicTypeKey> {
    source
        .lines()
        .filter_map(|line| {
            let line = line.trim_start();
            let rest = line.strip_prefix("pub ")?;
            let (kind, name_tail) = if let Some(tail) = rest.strip_prefix("struct ") {
                ("struct", tail)
            } else if let Some(tail) = rest.strip_prefix("enum ") {
                ("enum", tail)
            } else {
                return None;
            };
            let name = name_tail
                .split(|character: char| !(character == '_' || character.is_ascii_alphanumeric()))
                .next()
                .unwrap_or_default();
            (!name.is_empty()).then(|| PublicTypeKey {
                kind: kind.to_string(),
                name: name.to_string(),
            })
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct PublicTypeKey {
    kind: String,
    name: String,
}

struct PublicTypeDuplicate {
    kind: String,
    name: String,
    runtime: PathBuf,
    interface: PathBuf,
}

impl PublicTypeDuplicate {
    fn describe(&self, repo_root: &Path) -> String {
        format!(
            "{} {} runtime={} interface={}",
            self.kind,
            self.name,
            repo_root
                .join(RUNTIME_UI_ROOT)
                .join(&self.runtime)
                .display(),
            repo_root
                .join(INTERFACE_UI_ROOT)
                .join(&self.interface)
                .display()
        )
    }
}
