use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use woc_runtime::{inspect_project, WocHostRole, WocProjectIdentityError};

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

#[test]
fn every_host_role_reports_the_same_authoritative_project_identity() {
    let roles = [
        WocHostRole::Client,
        WocHostRole::Server,
        WocHostRole::Bot,
        WocHostRole::Headless,
    ];
    let identities = roles
        .into_iter()
        .map(|role| inspect_project(project_root(), role).expect("project identity must be valid"))
        .collect::<Vec<_>>();

    for identity in &identities {
        assert_eq!(identity.project_name, "World of Claudecraft");
        assert_eq!(identity.script_package, "woc_game");
        assert_eq!(identity.backend, "zr_vm:project");
        assert_eq!(identity.zr_vm_project, "woc_game.zrp");
        assert_eq!(identity.zr_vm_entry_module, "main");
        assert_eq!(identity.zr_vm_execution_mode, "interp");
        assert_eq!(
            identity.contract_schema_fingerprint,
            woc_protocol::SCHEMA_FINGERPRINT_HEX
        );
        assert_eq!(
            identity.command_catalog_sha256,
            woc_protocol::COMMAND_CATALOG_SHA256
        );
        assert_eq!(
            identity.command_payload_schema_sha256,
            woc_protocol::COMMAND_PAYLOAD_SCHEMA_SHA256
        );
        assert_eq!(identity.world_state_format, "WOS64");
        assert_eq!(identity.world_state_schema_version, 64);
        assert_eq!(identity.simulation_hz, 20);
        assert_eq!(identity.presentation_hz, 60);
    }
    assert!(identities
        .windows(2)
        .all(|pair| pair[0].source_commit == pair[1].source_commit));
}

#[test]
fn server_identity_requires_zr_vm_for_the_server_target_mode() {
    let fixture = fixture_project(
        "missing-server-zr-vm",
        r#"
name = "fixture"

[scripts]
startup_packages = ["woc_game"]

[plugins]
[[plugins.selections]]
id = "zr_vm_language"
enabled = true
required = true
target_modes = ["client_runtime"]

[[plugins.selections]]
id = "woc_runtime"
enabled = true
required = true
target_modes = ["client_runtime", "server_runtime"]
"#,
    );

    let error = inspect_project(&fixture, WocHostRole::Server)
        .expect_err("server identity must reject a missing server ZrVM selection");
    assert_invalid_contains(error, "zr_vm_language is not enabled for server_runtime");
    fs::remove_dir_all(fixture).expect("temporary fixture must be removable");
}

#[test]
fn client_identity_requires_an_enabled_woc_runtime_selection() {
    let fixture = fixture_project(
        "disabled-client-woc-runtime",
        r#"
name = "fixture"

[scripts]
startup_packages = ["woc_game"]

[plugins]
[[plugins.selections]]
id = "zr_vm_language"
enabled = true
required = true
target_modes = ["client_runtime", "server_runtime"]

[[plugins.selections]]
id = "woc_runtime"
enabled = false
required = true
target_modes = ["client_runtime", "server_runtime"]
"#,
    );

    let error = inspect_project(&fixture, WocHostRole::Client)
        .expect_err("client identity must reject a disabled WOC runtime selection");
    assert_invalid_contains(error, "woc_runtime is not enabled for client_runtime");
    fs::remove_dir_all(fixture).expect("temporary fixture must be removable");
}

#[test]
fn bot_and_headless_identities_require_the_server_target_mode() {
    let fixture = fixture_project(
        "server-backed-roles",
        r#"
name = "fixture"

[scripts]
startup_packages = ["woc_game"]

[plugins]
[[plugins.selections]]
id = "zr_vm_language"
enabled = true
required = true
target_modes = ["server_runtime"]

[[plugins.selections]]
id = "woc_runtime"
enabled = true
required = true
target_modes = ["server_runtime"]
"#,
    );

    inspect_project(&fixture, WocHostRole::Bot).expect("bot identity must map to server_runtime");
    inspect_project(&fixture, WocHostRole::Headless)
        .expect("headless identity must map to server_runtime");
    fs::remove_dir_all(fixture).expect("temporary fixture must be removable");
}

#[test]
fn identity_requires_every_critical_plugin_selection_to_be_required() {
    for plugin_id in ["zr_vm_language", "woc_runtime"] {
        let fixture = fixture_project(
            plugin_id,
            &format!(
                r#"
name = "fixture"

[scripts]
startup_packages = ["woc_game"]

[plugins]
[[plugins.selections]]
id = "zr_vm_language"
enabled = true
required = {zr_vm_required}
target_modes = ["client_runtime", "server_runtime"]

[[plugins.selections]]
id = "woc_runtime"
enabled = true
required = {woc_runtime_required}
target_modes = ["client_runtime", "server_runtime"]
"#,
                zr_vm_required = plugin_id != "zr_vm_language",
                woc_runtime_required = plugin_id != "woc_runtime",
            ),
        );

        let error = inspect_project(&fixture, WocHostRole::Client)
            .expect_err("identity must reject a non-required critical plugin");
        assert_invalid_contains(
            error,
            &format!("{plugin_id} is not required for client_runtime"),
        );
        fs::remove_dir_all(fixture).expect("temporary fixture must be removable");
    }
}

#[test]
fn identity_requires_the_canonical_zr_vm_project_binding() {
    let fixture = fixture_project(
        "wrong-zr-vm-project",
        r#"
name = "fixture"

[scripts]
startup_packages = ["woc_game"]

[plugins]
[[plugins.selections]]
id = "zr_vm_language"
enabled = true
required = true
target_modes = ["client_runtime", "server_runtime"]

[[plugins.selections]]
id = "woc_runtime"
enabled = true
required = true
target_modes = ["client_runtime", "server_runtime"]
"#,
    );
    fs::write(
        fixture.join("scripts/woc_game/plugin.toml"),
        r#"
name = "woc_game"
entry = "main"
backend = "zr_vm:project"

[zr_vm]
project = "alternate.zrp"
entry_module = "main"
execution_mode = "interp"
"#,
    )
    .expect("fixture package manifest");

    let error = inspect_project(&fixture, WocHostRole::Client)
        .expect_err("identity must reject a substituted ZrVM project");
    assert_invalid_contains(error, "zr_vm.project must be woc_game.zrp");
    fs::remove_dir_all(fixture).expect("temporary fixture must be removable");
}

fn fixture_project(name: &str, project_toml: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time must be after the epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon-woc-runtime-identity-{}-{}-{nanos}",
        std::process::id(),
        name
    ));
    fs::create_dir_all(root.join("scripts/woc_game")).expect("fixture package directory");
    fs::create_dir_all(root.join("reference/current-head")).expect("fixture reference directory");
    fs::write(root.join("zircon-project.toml"), project_toml).expect("fixture project manifest");
    fs::write(
        root.join("scripts/woc_game/plugin.toml"),
        r#"name = "woc_game"
entry = "main"
backend = "zr_vm:project"

[zr_vm]
project = "woc_game.zrp"
entry_module = "main"
execution_mode = "interp"
"#,
    )
    .expect("fixture package manifest");
    fs::write(
        root.join("scripts/woc_game/woc_game.zrp"),
        r#"{
  "name": "woc_game",
  "source": "src",
  "binary": "bin",
  "entry": "main"
}
"#,
    )
    .expect("fixture ZrVM project manifest");
    fs::write(
        root.join("reference/current-head/source_manifest.json"),
        format!(
            "{{\"source_commit\":\"{}\"}}",
            woc_protocol::REFERENCE_COMMIT
        ),
    )
    .expect("fixture source manifest");
    root
}

fn assert_invalid_contains(error: WocProjectIdentityError, expected: &str) {
    match error {
        WocProjectIdentityError::Invalid(message) => assert!(
            message.contains(expected),
            "expected `{expected}` in `{message}`"
        ),
        other => panic!("expected an invalid identity error, got {other:?}"),
    }
}
