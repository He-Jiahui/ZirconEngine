use std::collections::HashMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use super::{
    canonical_package_id, canonicalize_package_id_reference_in_place, resolve_build,
    select_product_artifact,
};
use crate::build::product_build::CargoRuntimeDependencyDeclaration;

#[test]
fn rejects_a_runtime_dependency_outside_the_product_resolve_graph() {
    let product_id = "path+file:///snapshot#zircon_app@0.1.0";
    let unrelated_id = "path+file:///snapshot#unrelated_runtime@0.1.0";
    let metadata = serde_json::to_vec(&serde_json::json!({
        "packages": [
            {
                "name": "zircon_app",
                "id": product_id,
                "targets": [{"name": "zircon_runtime", "kind": ["bin"]}]
            },
            {
                "name": "unrelated_runtime",
                "id": unrelated_id,
                "targets": [{"name": "unrelated_runtime", "kind": ["lib"]}]
            }
        ],
        "resolve": {"nodes": [
            {"id": product_id, "dependencies": []},
            {"id": unrelated_id, "dependencies": []}
        ]}
    }))
    .unwrap();

    let error = resolve_build(
        &metadata,
        Path::new("/snapshot"),
        "zircon_app",
        "zircon_runtime",
        vec![CargoRuntimeDependencyDeclaration {
            logical_name: "unrelated-runtime".to_string(),
            relative_path: "bin/unrelated_runtime.dll".to_string(),
            package: "unrelated_runtime".to_string(),
            target: "unrelated_runtime".to_string(),
            artifact_file_name: "unrelated_runtime.dll".to_string(),
        }],
    )
    .err()
    .unwrap();

    assert!(error.to_string().contains("not reachable from product"));
}

#[test]
fn skips_unselected_artifact_payloads_before_path_materialization() {
    let messages = concat!(
        "{\"reason\":\"compiler-artifact\",\"package_id\":\"ignored\",\"target\":{\"name\":\"ignored\",\"kind\":[\"lib\"]},\"filenames\":{\"not\":\"an-array\"}}\n",
        "{\"reason\":\"compiler-artifact\",\"package_id\":\"selected\",\"target\":{\"name\":\"zircon_runtime\",\"kind\":[\"bin\"]},\"filenames\":[\"/target/zircon_runtime\",\"/target/zircon_runtime.pdb\"],\"executable\":\"/target/zircon_runtime\"}\n",
        "{\"reason\":\"build-finished\",\"success\":true}\n"
    );

    let artifact = select_product_artifact(
        Cursor::new(messages.as_bytes()),
        "selected",
        "zircon_runtime",
    )
    .unwrap();

    assert_eq!(artifact.executable, PathBuf::from("/target/zircon_runtime"));
    assert_eq!(
        artifact.symbol_files,
        vec![PathBuf::from("/target/zircon_runtime.pdb")]
    );
}

#[test]
fn selected_artifact_fuses_executable_check_and_symbol_collection() {
    let messages = concat!(
        "{\"reason\":\"compiler-artifact\",\"package_id\":\"selected\",\"target\":{\"name\":\"zircon_runtime\",\"kind\":[\"bin\"]},\"filenames\":[\"/target/z-last.pdb\",\"/target/zircon_runtime\",\"/target/z-first.PDB\",\"/target/z-last.pdb\"],\"executable\":\"/target/zircon_runtime\"}\n",
        "{\"reason\":\"build-finished\",\"success\":true}\n"
    );

    let artifact = select_product_artifact(
        Cursor::new(messages.as_bytes()),
        "selected",
        "zircon_runtime",
    )
    .unwrap();

    assert_eq!(artifact.executable, PathBuf::from("/target/zircon_runtime"));
    assert_eq!(
        artifact.symbol_files,
        vec![
            PathBuf::from("/target/z-first.PDB"),
            PathBuf::from("/target/z-last.pdb"),
        ]
    );
}

#[test]
fn cargo_graph_digest_is_stable_across_build_set_locations() {
    let first_root = if cfg!(windows) {
        Path::new("C:/buildsets/first/source")
    } else {
        Path::new("/buildsets/first/source")
    };
    let second_root = if cfg!(windows) {
        Path::new("D:/buildsets/second/source")
    } else {
        Path::new("/buildsets/second/source")
    };
    let first = resolve_build(
        &graph_metadata(first_root, &[]),
        first_root,
        "zircon_app",
        "zircon_app",
        Vec::new(),
    )
    .unwrap();
    let second = resolve_build(
        &graph_metadata(second_root, &[]),
        second_root,
        "zircon_app",
        "zircon_app",
        Vec::new(),
    )
    .unwrap();
    let changed = resolve_build(
        &graph_metadata(second_root, &["changed-feature"]),
        second_root,
        "zircon_app",
        "zircon_app",
        Vec::new(),
    )
    .unwrap();

    assert_ne!(first.product_package_id, second.product_package_id);
    assert_eq!(first.cargo_graph_digest, second.cargo_graph_digest);
    assert_ne!(second.cargo_graph_digest, changed.cargo_graph_digest);
}

#[test]
fn cargo_graph_digest_preserves_external_package_identity() {
    let root = if cfg!(windows) {
        Path::new("C:/buildsets/source")
    } else {
        Path::new("/buildsets/source")
    };
    let local_id = format!("path+file://{}#zircon_app@0.1.0", root.display());
    let registry_id = "registry+https://example.invalid/index#dependency@1.2.3";
    let metadata = serde_json::to_vec(&serde_json::json!({
        "packages": [
            {
                "name": "zircon_app",
                "version": "0.1.0",
                "id": local_id,
                "source": null,
                "manifest_path": root.join("crates/zircon_app/Cargo.toml"),
                "targets": [{
                    "name": "zircon_app",
                    "kind": ["bin"],
                    "src_path": root.join("crates/zircon_app/src/main.rs")
                }]
            },
            {
                "name": "dependency",
                "version": "1.2.3",
                "id": registry_id,
                "source": "registry+https://example.invalid/index",
                "manifest_path": "C:/cargo/registry/dependency/Cargo.toml",
                "targets": [{
                    "name": "dependency",
                    "kind": ["lib"],
                    "src_path": "C:/cargo/registry/dependency/src/lib.rs"
                }]
            }
        ],
        "workspace_members": [local_id],
        "workspace_default_members": [local_id],
        "resolve": {"nodes": [
            {
                "id": local_id,
                "dependencies": [registry_id],
                "deps": [{"name": "dependency", "pkg": registry_id}],
                "features": []
            },
            {
                "id": registry_id,
                "dependencies": [],
                "deps": [],
                "features": []
            }
        ]}
    }))
    .unwrap();

    let resolution =
        resolve_build(&metadata, root, "zircon_app", "zircon_app", Vec::new()).unwrap();

    assert_eq!(resolution.product_package_id, local_id);
    assert_eq!(resolution.cargo_graph_digest.len(), 64);
}

#[test]
fn cargo_graph_digest_is_stable_across_resolve_feature_order() {
    let root = if cfg!(windows) {
        Path::new("C:/buildsets/source")
    } else {
        Path::new("/buildsets/source")
    };
    let first = resolve_build(
        &graph_metadata(root, &["zeta-feature", "alpha-feature"]),
        root,
        "zircon_app",
        "zircon_app",
        Vec::new(),
    )
    .unwrap();
    let reordered = resolve_build(
        &graph_metadata(root, &["alpha-feature", "zeta-feature"]),
        root,
        "zircon_app",
        "zircon_app",
        Vec::new(),
    )
    .unwrap();

    assert_eq!(first.cargo_graph_digest, reordered.cargo_graph_digest);
}

#[test]
fn cargo_graph_digest_is_stable_across_target_order() {
    let root = if cfg!(windows) {
        Path::new("C:/buildsets/source")
    } else {
        Path::new("/buildsets/source")
    };
    let mut reordered_value: serde_json::Value =
        serde_json::from_slice(&graph_metadata(root, &[])).unwrap();
    let targets = reordered_value["packages"][0]["targets"]
        .as_array_mut()
        .unwrap();
    targets.push(serde_json::json!({
        "name": "zircon_helper",
        "kind": ["lib"],
        "crate_types": ["lib"],
        "required-features": [],
        "edition": "2024",
        "src_path": root.join("crates/zircon_app/src/helper.rs"),
    }));
    let first_metadata = serde_json::to_vec(&reordered_value).unwrap();
    let first = resolve_build(
        &first_metadata,
        root,
        "zircon_app",
        "zircon_app",
        Vec::new(),
    )
    .unwrap();
    targets.reverse();
    let reordered_metadata = serde_json::to_vec(&reordered_value).unwrap();
    let reordered = resolve_build(
        &reordered_metadata,
        root,
        "zircon_app",
        "zircon_app",
        Vec::new(),
    )
    .unwrap();

    assert_eq!(first.cargo_graph_digest, reordered.cargo_graph_digest);
}

#[test]
fn canonical_local_package_identity_carries_the_reused_manifest_path() {
    let root = if cfg!(windows) {
        Path::new("C:/buildsets/source")
    } else {
        Path::new("/buildsets/source")
    };
    let metadata: super::CargoMetadata =
        serde_json::from_slice(&graph_metadata(root, &[])).unwrap();
    let package = &metadata.packages[0];

    let (canonical_id, canonical_manifest_path) = canonical_package_id(package, root).unwrap();

    assert_eq!(
        canonical_id.as_deref(),
        Some("path+build-set:///crates/zircon_app/Cargo.toml#zircon_app@0.1.0")
    );
    assert_eq!(
        canonical_manifest_path,
        Path::new("crates/zircon_app/Cargo.toml")
    );
}

#[test]
fn canonical_package_reference_reuses_unchanged_identity_allocation() {
    let registry_id = "registry+https://example.invalid/index#dependency@1.2.3".to_string();
    let local_id = "path+file:///snapshot#zircon_app@0.1.0".to_string();
    let canonical_local_id = "path+build-set:///crates/zircon_app/Cargo.toml#zircon_app@0.1.0";
    let package_ids = HashMap::from([
        (registry_id.clone(), None),
        (local_id.clone(), Some(canonical_local_id.to_string())),
    ]);
    let mut unchanged = registry_id;
    let unchanged_allocation = unchanged.as_ptr();

    canonicalize_package_id_reference_in_place(&mut unchanged, &package_ids).unwrap();

    assert_eq!(unchanged_allocation, unchanged.as_ptr());
    let mut changed = local_id;
    canonicalize_package_id_reference_in_place(&mut changed, &package_ids).unwrap();
    assert_eq!(changed, canonical_local_id);
    let mut unknown = "registry+unknown#missing@1.0.0".to_string();
    let error = canonicalize_package_id_reference_in_place(&mut unknown, &package_ids).unwrap_err();
    assert!(error.to_string().contains("references unknown package"));
}

fn graph_metadata(snapshot_root: &Path, resolve_features: &[&str]) -> Vec<u8> {
    let manifest_path = snapshot_root.join("crates/zircon_app/Cargo.toml");
    let source_path = snapshot_root.join("crates/zircon_app/src/main.rs");
    let package_id = format!("path+file://{}#zircon_app@0.1.0", snapshot_root.display());
    serde_json::to_vec(&serde_json::json!({
        "packages": [{
            "name": "zircon_app",
            "version": "0.1.0",
            "id": package_id,
            "source": null,
            "checksum": null,
            "manifest_path": manifest_path,
            "features": {"target-client": []},
            "targets": [{
                "name": "zircon_app",
                "kind": ["bin"],
                "crate_types": ["bin"],
                "required-features": [],
                "edition": "2024",
                "src_path": source_path
            }]
        }],
        "workspace_members": [package_id],
        "workspace_default_members": [package_id],
        "resolve": {"nodes": [{
            "id": package_id,
            "dependencies": [],
            "deps": [],
            "features": resolve_features
        }]}
    }))
    .unwrap()
}
