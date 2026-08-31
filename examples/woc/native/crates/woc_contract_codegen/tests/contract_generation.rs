use std::fs;
use std::path::{Path, PathBuf};

use woc_contract_codegen::{
    generate_projections, load_contract_manifest, verify_projection, write_projection,
    ContractError, ProjectionError, ProjectionWriteOutcome,
};

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

#[test]
fn projections_are_reproducible_and_current() {
    let manifest = load_contract_manifest(project_root().join("contracts/woc.contracts.json"))
        .expect("canonical WOC contracts must load");
    let first = generate_projections(&manifest).expect("first projection must generate");
    let second = generate_projections(&manifest).expect("second projection must generate");

    assert_eq!(first, second);
    assert_eq!(first.fingerprint_hex.len(), 64);
    verify_projection(
        project_root().join("native/crates/woc_protocol/src/generated.rs"),
        &first.rust,
    )
    .expect("Rust projection must be current");
    verify_projection(
        project_root().join("scripts/woc_game/src/generated/contracts.zr"),
        &first.zrvm,
    )
    .expect("ZrVM projection must be current");
}

#[test]
fn stale_projection_is_rejected() {
    let path = temporary_file("stale.rs");
    fs::write(&path, "stale\n").expect("stale fixture must be written");
    let error = verify_projection(&path, "current\n").expect_err("stale bytes must fail");
    assert!(matches!(error, ProjectionError::Drift { .. }));
    fs::remove_file(path).expect("stale fixture must be removed");
}

#[test]
fn unchanged_projection_is_not_rewritten() {
    let path = temporary_file("write-if-changed.rs");
    let _ = fs::remove_file(&path);

    assert_eq!(
        write_projection(&path, "first\n").expect("first projection must be written"),
        ProjectionWriteOutcome::Written
    );
    let first_modified = fs::metadata(&path)
        .expect("first projection metadata must be readable")
        .modified()
        .expect("first projection modification time must be readable");
    assert_eq!(
        write_projection(&path, "first\n").expect("unchanged projection must be accepted"),
        ProjectionWriteOutcome::Unchanged
    );
    assert_eq!(
        fs::metadata(&path)
            .expect("unchanged projection metadata must be readable")
            .modified()
            .expect("unchanged projection modification time must be readable"),
        first_modified
    );
    assert_eq!(
        write_projection(&path, "second\n").expect("changed projection must be written"),
        ProjectionWriteOutcome::Written
    );
    assert_eq!(
        fs::read_to_string(&path).expect("changed projection must be readable"),
        "second\n"
    );

    fs::remove_file(path).expect("projection fixture must be removed");
}

#[test]
fn reserved_contract_id_is_rejected() {
    let source = fs::read_to_string(project_root().join("contracts/woc.contracts.json"))
        .expect("contract fixture must be readable");
    let mut value: serde_json::Value = serde_json::from_str(&source).expect("fixture must parse");
    value["contracts"][0]["id"] = serde_json::Value::from(100);
    let bytes = serde_json::to_vec(&value).expect("mutated fixture must serialize");
    let error = woc_contract_codegen::ContractManifest::from_slice(&bytes)
        .expect_err("reserved id must fail");
    assert!(matches!(error, ContractError::ReservedId { .. }));
}

#[test]
fn unbounded_variable_field_is_rejected() {
    let source = fs::read_to_string(project_root().join("contracts/woc.contracts.json"))
        .expect("contract fixture must be readable");
    let mut value: serde_json::Value = serde_json::from_str(&source).expect("fixture must parse");
    value["contracts"][2]["fields"][3]
        .as_object_mut()
        .expect("payload field must be an object")
        .remove("max_length");
    let bytes = serde_json::to_vec(&value).expect("mutated fixture must serialize");
    let error = woc_contract_codegen::ContractManifest::from_slice(&bytes)
        .expect_err("unbounded bytes must fail");
    assert!(matches!(error, ContractError::MissingBound { .. }));
}

#[test]
fn positive_protocol_revisions_are_accepted_without_changing_manifest_schema_rules() {
    let source = fs::read_to_string(project_root().join("contracts/woc.contracts.json"))
        .expect("contract fixture must be readable");
    let mut value: serde_json::Value = serde_json::from_str(&source).expect("fixture must parse");
    value["protocol_version"] = serde_json::Value::from(2);
    let bytes = serde_json::to_vec(&value).expect("mutated fixture must serialize");
    let manifest = woc_contract_codegen::ContractManifest::from_slice(&bytes)
        .expect("a positive protocol revision must remain generatable");
    assert_eq!(manifest.schema_version, 1);
    assert_eq!(manifest.protocol_version, 2);
}

fn temporary_file(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("zircon-woc-codegen-{}-{name}", std::process::id()))
}
