use std::fs;

use zircon_runtime::script::{VmBackend, VmPluginInstance, VmStateBlob};

use crate::ZrVmBackend;

use super::support::{build_real_backend_host, ZrVmProjectFixture};

fn load_source(source: &str) -> (ZrVmProjectFixture, Box<dyn VmPluginInstance>) {
    let fixture = ZrVmProjectFixture::new("explicit_state_encoding", "0.1.0");
    fs::write(
        fixture.project_path.parent().unwrap().join("src/main.zr"),
        source,
    )
    .unwrap();
    let manager = zircon_runtime::script::VmPluginManager::mock();
    let packages = manager.discover_packages(&fixture.root).unwrap();
    let host = build_real_backend_host(&manager, &packages[0]);
    let instance = ZrVmBackend::default()
        .load_package(&packages[0].package, &host)
        .unwrap();
    (fixture, instance)
}

#[test]
fn real_backend_explicit_byte_state_preserves_opaque_payload_and_schema() {
    let (_fixture, mut instance) = load_source(
        r#"
var container = %import("zr.container");
var retained = new container.Array<uint>();
pub stateEncoding(): string { return "vm-state-bytes-v1"; }
pub stateSchema(): string { return "{\"schema_version\":3,\"types\":[]}"; }
pub saveState(): container.Array<uint> { return retained; }
pub restoreState(state: container.Array<uint>): int { retained = state; return 0; }
"#,
    );
    assert!(instance.save_state().unwrap().payload.is_empty());
    assert!(instance.state_schema().unwrap().unwrap().types.is_empty());
    let state = VmStateBlob::from_payload(vec![0, 1, 127, 128, 255]);
    instance.restore_state(&state).unwrap();
    assert_eq!(instance.save_state().unwrap(), state);
    let mut invalid = state.clone();
    invalid.schema_version = 4;
    assert!(instance.restore_state(&invalid).is_err());
    assert_eq!(instance.save_state().unwrap(), state);
    invalid = super::support::fixture_state_blob("reflected");
    assert!(!invalid.types.is_empty());
    assert!(instance.restore_state(&invalid).is_err());
    assert_eq!(instance.save_state().unwrap(), state);
}

#[test]
fn real_backend_byte_state_requires_explicit_encoding_and_valid_byte_elements() {
    let source = r#"
var container = %import("zr.container");
pub saveState(): container.Array<uint> {
    var state = new container.Array<uint>();
    state.add(<uint>256);
    return state;
}
"#;
    let (_fixture, mut instance) = load_source(source);
    assert!(instance
        .save_state()
        .unwrap_err()
        .to_string()
        .contains("JSON saveState"));
    drop(instance);
    let (_fixture, mut instance) = load_source(&format!(
        "{source}\npub stateEncoding(): string {{ return \"vm-state-bytes-v1\"; }}\n"
    ));
    assert!(instance
        .save_state()
        .unwrap_err()
        .to_string()
        .contains("outside 0..=255"));
}

#[test]
fn real_backend_byte_state_rejects_missing_or_failed_lifecycle_exports() {
    for export in ["", "pub saveState(): string { return \"wrong\"; }"] {
        let (_fixture, mut instance) = load_source(&format!(
            "pub stateEncoding(): string {{ return \"vm-state-bytes-v1\"; }}\n{export}"
        ));
        assert!(instance.save_state().is_err());
        assert!(instance.restore_state(&VmStateBlob::default()).is_err());
    }
    let (_fixture, mut instance) = load_source(
        r#"
var container = %import("zr.container");
pub stateEncoding(): string { return "vm-state-bytes-v1"; }
pub restoreState(state: container.Array<uint>): int { return -1; }
pub stateSchema(): string { return "{\"schema_version\":4,\"types\":[]}"; }
"#,
    );
    assert!(instance.restore_state(&VmStateBlob::default()).is_err());
    assert!(instance.state_schema().is_err());
}

#[test]
fn real_backend_rejects_unknown_encoding_at_package_load() {
    let fixture = ZrVmProjectFixture::new("unknown_state_encoding", "0.1.0");
    fs::write(
        fixture.project_path.parent().unwrap().join("src/main.zr"),
        "pub stateEncoding(): string { return \"vm-state-bytes-v999\"; }",
    )
    .unwrap();
    let manager = zircon_runtime::script::VmPluginManager::mock();
    let packages = manager.discover_packages(&fixture.root).unwrap();
    let host = build_real_backend_host(&manager, &packages[0]);
    let result = ZrVmBackend::default().load_package(&packages[0].package, &host);
    let Err(error) = result else {
        panic!("unknown encoding must prevent loading")
    };
    assert!(error
        .to_string()
        .contains("unsupported zr_vm stateEncoding"));
}
