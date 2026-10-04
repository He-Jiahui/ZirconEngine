use std::fs;
use std::path::Path;

use zircon_runtime::core::framework::script::ScriptHostValue;
use zircon_runtime::script::{VmBackend, VmPluginInstance, VmStateSchema};

use crate::{register_zr_vm_backend, ZrVmBackend};

use super::support::{build_real_backend_host, ZrVmProjectFixture};

#[test]
fn real_backend_array_uint_roundtrip_preserves_binary_bytes() {
    let fixture = ZrVmProjectFixture::new("array_uint_roundtrip", "0.1.0");
    fs::write(
        fixture.project_path.parent().unwrap().join("src/main.zr"),
        r#"var container = %import("zr.container");
var callCount = 0;
pub arrayRoundtrip(input: container.Array<uint>): container.Array<uint> {
    callCount = callCount + 1;
    var output = new container.Array<uint>();
    for (var byte in input) { output.add(byte); }
    return output;
}
pub calls(): int { return callCount; }
"#,
    )
    .unwrap();
    let manager = zircon_runtime::script::VmPluginManager::mock();
    let packages = manager.discover_packages(&fixture.root).unwrap();
    let host = build_real_backend_host(&manager, &packages[0]);
    let mut instance = ZrVmBackend::default()
        .load_package(&packages[0].package, &host)
        .expect("load a real ZrVM Array<uint> fixture");

    for (index, bytes) in [
        Vec::new(),
        vec![0, 1, 127, 128, 255],
        (0..4096).map(|n| n as u8).collect(),
    ]
    .into_iter()
    .enumerate()
    {
        let result = instance
            .call_export(
                "main",
                "arrayRoundtrip",
                &[ScriptHostValue::Bytes(bytes.clone())],
            )
            .expect("lift and lower Array<uint> through the real binding");
        assert_eq!(result, Some(ScriptHostValue::Bytes(bytes)));
        assert_eq!(
            instance.call_export("main", "calls", &[]).unwrap(),
            Some(ScriptHostValue::Int((index + 1) as i64))
        );
    }
}

#[test]
fn real_backend_loads_authored_woc_package_and_calls_its_scalar_math() {
    let fixture = woc_project_fixture("main");
    let manager = zircon_runtime::script::VmPluginManager::mock();
    register_zr_vm_backend(&manager);
    let packages = manager.discover_packages(&fixture.root).unwrap();
    assert_eq!(packages.len(), 1);
    assert!(packages[0]
        .package
        .manifest
        .capabilities
        .contains("math.scalar"));
    let slot = manager
        .load_discovered_package(&packages[0])
        .expect("load the authored WOC manifest, query its schema, and activate it");
    let schema = manager
        .call_slot_export(slot, "main", "stateSchema", &[])
        .unwrap()
        .unwrap();
    let ScriptHostValue::String(schema) = schema else {
        panic!("WOC stateSchema must return lifecycle JSON")
    };
    let schema = VmStateSchema::from_json(&schema).unwrap();
    assert_eq!(
        schema.schema_version,
        zircon_runtime::script::VM_STATE_SCHEMA_VERSION_V3
    );
    assert!(schema.types.is_empty());

    for (value, expected) in [(2.5, 3), (-2.5, -2)] {
        assert_eq!(
            manager
                .call_slot_export(
                    slot,
                    "progression/stat_core_rules",
                    "roundJs",
                    &[ScriptHostValue::Float(value)],
                )
                .expect("authored WOC math uses the package's declared capability"),
            Some(ScriptHostValue::Int(expected))
        );
    }
    for (input, expected) in [(-1.5_f64, -1.0_f64), (-0.5, -0.0), (0.5, 1.0)] {
        let value = manager
            .call_slot_export(
                slot,
                "kernel/math_runtime_test_main",
                "roundValue",
                &[ScriptHostValue::Float(input)],
            )
            .expect("round dispatch through the real retained WOC package")
            .unwrap();
        let ScriptHostValue::Float(actual) = value else {
            panic!("math.round must retain its float result")
        };
        assert_eq!(actual.to_bits(), expected.to_bits(), "round({input:?})");
    }
    manager.unload_slot(slot).unwrap();
    assert!(manager.list_slots().is_empty());
}

#[test]
fn real_backend_woc_wos118_codec_migrates_and_rejects_invalid_bytes() {
    let (_fixture, mut instance) = load_woc_codec();
    let identity = instance
        .call_export(
            "world/deathless_will_runtime_test_main",
            "worldStateIdentity",
            &[],
        )
        .unwrap()
        .unwrap();
    let ScriptHostValue::String(identity) = identity else {
        panic!("WOC identity export must return JSON")
    };
    let identity: serde_json::Value = serde_json::from_str(&identity).unwrap();
    assert_eq!(identity["world_state"], "WOS118");

    let encoded = instance
        .call_export(
            "world/state",
            "worldStateCodecEncode",
            &[ScriptHostValue::Int(118)],
        )
        .unwrap()
        .unwrap();
    let ScriptHostValue::Bytes(bytes) = encoded.clone() else {
        panic!("WOS export must return bytes")
    };
    assert_eq!(&bytes[..6], b"WOS2\x76\x00");
    assert_eq!(
        instance
            .call_export(
                "world/state",
                "worldStateCodecRoundtrip",
                &[encoded.clone()]
            )
            .unwrap(),
        Some(encoded.clone())
    );

    let legacy = instance
        .call_export(
            "world/state",
            "worldStateCodecEncode",
            &[ScriptHostValue::Int(117)],
        )
        .unwrap()
        .unwrap();
    let ScriptHostValue::Bytes(legacy_bytes) = &legacy else {
        panic!("legacy WOS export must return bytes")
    };
    assert_eq!(bytes.len(), legacy_bytes.len() + 2);
    assert_eq!(&bytes[6..bytes.len() - 2], &legacy_bytes[6..]);
    assert_eq!(&bytes[bytes.len() - 2..], &[0, 0]);
    assert_eq!(
        instance
            .call_export("world/state", "worldStateCodecRoundtrip", &[legacy])
            .unwrap(),
        Some(encoded.clone())
    );

    let mut truncated = bytes.clone();
    truncated.pop();
    let mut unknown = bytes.clone();
    unknown[4] = 119;
    for (invalid, expected) in [(truncated, "truncated"), (unknown, "schema is unsupported")] {
        let error = instance
            .call_export(
                "world/state",
                "worldStateCodecRoundtrip",
                &[ScriptHostValue::Bytes(invalid)],
            )
            .expect_err("malformed WOS bytes must fail in the authored decoder");
        assert!(error.to_string().contains(expected), "{error}");
    }
    assert_eq!(
        instance
            .call_export("world/state", "worldStateCodecRoundtrip", &[encoded])
            .unwrap(),
        Some(ScriptHostValue::Bytes(bytes))
    );
    assert_eq!(
        instance
            .call_export("world/deathless_will_runtime_test_main", "main", &[])
            .unwrap(),
        Some(ScriptHostValue::Int(1))
    );
}

fn load_woc_codec() -> (ZrVmProjectFixture, Box<dyn VmPluginInstance>) {
    let fixture = woc_project_fixture("world/deathless_will_runtime_test_main");
    let manager = zircon_runtime::script::VmPluginManager::mock();
    let packages = manager.discover_packages(&fixture.root).unwrap();
    let host = build_real_backend_host(&manager, &packages[0]);
    let instance = ZrVmBackend::default()
        .load_package(&packages[0].package, &host)
        .expect("compile and load the authored WOC world codec in real ZrVM");
    (fixture, instance)
}

pub(super) fn woc_project_fixture(entry: &str) -> ZrVmProjectFixture {
    let mut fixture = ZrVmProjectFixture::new("woc_wos118_codec", "0.1.0");
    let authored_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples/woc/scripts/woc_game")
        .canonicalize()
        .expect("managed fixture includes the authored WOC source closure");
    let package_root = fixture.root.join("woc_wos118_codec");
    fs::copy(
        authored_root.join("plugin.toml"),
        package_root.join("plugin.toml"),
    )
    .unwrap();
    let mut project: serde_json::Value =
        serde_json::from_reader(fs::File::open(authored_root.join("woc_game.zrp")).unwrap())
            .unwrap();
    project["source"] = serde_json::to_value(authored_root.join("src")).unwrap();
    project["binary"] = serde_json::to_value(package_root.join("bin")).unwrap();
    project["entry"] = serde_json::json!(entry);
    fs::create_dir_all(package_root.join("bin")).unwrap();
    fixture.project_path = package_root.join("woc_game.zrp");
    fs::write(
        &fixture.project_path,
        serde_json::to_vec_pretty(&project).unwrap(),
    )
    .unwrap();
    fixture
}
