use std::collections::BTreeMap;
use std::ffi::{c_void, CStr};

use super::*;
use zircon_plugin_sdk::native::{
    NativePluginBehaviorV4, NativePluginHostFunctionTableV3, ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
};
use zircon_runtime::asset::{
    AssetReference, AssetUri, MaterialGraphLinkAsset, MaterialGraphNodeAsset,
    MaterialGraphNodeKindAsset as Node,
};

#[test]
fn material_editor_dist_native_manifest_declares_real_slots_and_contribution_schema() {
    let behavior = behavior();
    // SAFETY: The entry exposes a static NUL-terminated editor contribution schema.
    let registration_schema =
        unsafe { CStr::from_ptr(behavior.schema_versions.registration_manifest_schema) };
    assert_eq!(registration_schema.to_bytes_with_nul(), REGISTRATION_SCHEMA);
    // SAFETY: The behavior command manifest points to the static NUL-terminated MANIFEST.
    let command_manifest = unsafe { CStr::from_ptr(behavior.command_manifest) };
    let manifest =
        native::command_manifest_v4_from_toml(command_manifest.to_str().unwrap()).unwrap();
    assert_eq!(manifest.commands.len(), 2);
    for (slot, name, schema) in [
        (
            0,
            "material_editor.graph.validate",
            "material_editor.validate_graph.v1",
        ),
        (
            1,
            "material_editor.graph.compile",
            "material_editor.compile_graph.v1",
        ),
    ] {
        let command = &manifest.commands[slot as usize];
        assert_eq!(command.slot, slot);
        assert_eq!(command.name, name);
        assert_eq!(command.payload_schema, schema);
        assert_eq!(command.max_output_bytes, MAX_OUTPUT_BYTES);
    }
    assert!(behavior.invoke_command.is_some());
    assert_eq!(behavior.is_stateless, 1);
}

#[test]
fn material_editor_dist_native_compile_changes_real_material_from_each_graph_input() {
    for value in [0.25, 0.875] {
        let graph = graph(value);
        let payload = request(&graph);
        let original = payload.clone();
        let (status, capture) = invoke(
            1,
            &payload,
            MAX_OUTPUT_BYTES,
            ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        );
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
        assert_eq!(capture.calls, 1);
        let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
        assert_eq!(result["ok"], true);
        assert_eq!(result["diagnostics"], json!([]));
        assert_eq!(
            result["graph_uri"],
            serde_json::to_value(&graph.uri).unwrap()
        );
        assert_eq!(
            result["material"]["base_color"],
            json!([value, 0.5, 0.75, 1.0])
        );
        assert_eq!(
            result["material"]["shader"],
            serde_json::to_value(&graph.shader).unwrap()
        );
        assert_eq!(payload, original);
    }
}

#[test]
fn material_editor_dist_native_validation_keeps_cjk_node_and_exact_input_target() {
    let mut graph = graph(0.25);
    graph.nodes[1].id = "输出 / Material".into();
    graph.links.clear();
    let (status, capture) = invoke(
        0,
        &request(&graph),
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
    assert_eq!(result["ok"], false);
    assert!(result["material"].is_null());
    assert!(result["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|d| d["target"]
            == json!({
                "kind":"input", "node_id":"输出 / Material", "pin_name":"base_color"
            })));
}

#[test]
fn material_editor_dist_native_compile_errors_never_return_a_material() {
    let mut graph = graph(0.25);
    graph.shader = None;
    let (status, capture) = invoke(
        1,
        &request(&graph),
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
    assert_eq!(result["ok"], false);
    assert!(result["material"].is_null());
    assert!(result["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|d| d["target"] == json!({"kind":"graph"})
            && d["message"].as_str().unwrap().contains("no shader target")));
}

#[test]
fn material_editor_dist_native_compile_keeps_texture_uuid_and_locator_together() {
    let mut graph = graph(0.25);
    let mut texture = reference("res://textures/岩石.ztex#albedo");
    // Preserve a real input identity even when its locator has changed.
    texture.uuid = reference("res://textures/original-name.ztex").uuid;
    graph.nodes[0].kind = Node::TextureSample {
        texture: texture.clone(),
    };
    graph.links[0].from_pin = "color".into();
    let (status, capture) = invoke(
        1,
        &request(&graph),
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
    assert_eq!(result["ok"], true);
    assert_eq!(
        result["material"]["base_color_texture"],
        serde_json::to_value(&texture).unwrap()
    );
}

#[test]
fn material_editor_dist_native_rejects_malformed_or_ambiguous_arguments_without_output() {
    let valid = serde_json::to_value(graph(0.25)).unwrap();
    for payload in [
        b"not-json".to_vec(),
        b"null".to_vec(),
        b"[]".to_vec(),
        b"{}".to_vec(),
        serde_json::to_vec(&json!({"graph":null})).unwrap(),
        serde_json::to_vec(&json!({"graph":valid,"other":true})).unwrap(),
        format!(r#"{{"graph":{valid},"graph":{valid}}}"#).into_bytes(),
    ] {
        let (status, capture) = invoke(
            1,
            &payload,
            MAX_OUTPUT_BYTES,
            ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        );
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
        assert_eq!(capture.calls, 0);
        assert!(capture.bytes.is_empty());
    }
}

#[test]
fn material_editor_dist_native_unknown_slot_is_denied_before_reading_payload() {
    let (status, capture) = invoke_slice(
        2,
        NativePluginByteSliceV3 {
            data: std::ptr::null(),
            len: usize::MAX,
        },
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        true,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_DENIED);
    assert_eq!(capture.calls, 0);
}

#[test]
fn material_editor_dist_native_rejects_invalid_or_oversized_slices_before_read() {
    for payload in [
        NativePluginByteSliceV3 {
            data: std::ptr::null(),
            len: 1,
        },
        NativePluginByteSliceV3 {
            data: b"x".as_ptr(),
            len: MAX_INPUT_BYTES + 1,
        },
        NativePluginByteSliceV3 {
            data: std::ptr::null(),
            len: 0,
        },
    ] {
        let (status, capture) = invoke_slice(
            1,
            payload,
            MAX_OUTPUT_BYTES,
            ZIRCON_NATIVE_PLUGIN_STATUS_OK,
            true,
        );
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
        assert_eq!(capture.calls, 0);
    }
}

#[test]
fn material_editor_dist_native_small_output_budget_publishes_no_partial_result() {
    for budget in [0, 1, 24] {
        let (status, capture) = invoke(
            1,
            &request(&graph(0.25)),
            budget,
            ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        );
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
        assert_eq!(capture.calls, 0);
        assert!(capture.bytes.is_empty());
    }
}

#[test]
fn material_editor_dist_native_propagates_host_writer_rejection() {
    let (status, capture) = invoke(
        1,
        &request(&graph(0.25)),
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_DENIED,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_DENIED);
    assert_eq!(capture.calls, 1);
    assert!(capture.bytes.is_empty());
}

#[test]
fn material_editor_dist_native_requires_a_live_output_writer() {
    let payload = request(&graph(0.25));
    let (status, capture) = invoke_slice(
        1,
        NativePluginByteSliceV3 {
            data: payload.as_ptr(),
            len: payload.len(),
        },
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        false,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
    assert_eq!(capture.calls, 0);
}

#[test]
fn material_editor_dist_native_contribution_batch_matches_existing_sdk_contract() {
    use zircon_plugin_sdk::editor_contribution::EditorContributionBuilder;

    // These two contracts are the host's existing typed execution DTOs, inferred
    // from the SDK builder's concrete parameter type, not a new transport model.
    let contract = || {
        serde_json::from_value(json!({
        "result_codec":"zircon.editor.command-result.v1",
        "resource_budget":{"max_input_bytes":1048576,"max_output_bytes":1048576,"max_execution_time_ms":5000}
    })).unwrap()
    };
    let expected = EditorContributionBuilder::new("material_editor")
        .command_with_execution_contract(
            "material_editor.graph.validate",
            "command.material_editor.graph.validate.label",
            "command.material_editor.graph.validate.description",
            contract(),
        )
        .command_with_execution_contract(
            "material_editor.graph.compile",
            "command.material_editor.graph.compile.label",
            "command.material_editor.graph.compile.description",
            contract(),
        )
        .localization_bundle(
            [
                (
                    "en",
                    [
                        (
                            "command.material_editor.graph.validate.label",
                            "Validate Material Graph",
                        ),
                        (
                            "command.material_editor.graph.validate.description",
                            "Validate the submitted material graph",
                        ),
                        (
                            "command.material_editor.graph.compile.label",
                            "Compile Material Graph",
                        ),
                        (
                            "command.material_editor.graph.compile.description",
                            "Compile the submitted material graph into a MaterialAsset",
                        ),
                    ],
                ),
                (
                    "zh-CN",
                    [
                        ("command.material_editor.graph.validate.label", "校验材质图"),
                        (
                            "command.material_editor.graph.validate.description",
                            "校验本次请求中的材质图",
                        ),
                        ("command.material_editor.graph.compile.label", "编译材质图"),
                        (
                            "command.material_editor.graph.compile.description",
                            "将本次请求中的材质图编译为 MaterialAsset",
                        ),
                    ],
                ),
            ]
            .into_iter()
            .map(|(locale, entries)| {
                (
                    locale.to_string(),
                    entries
                        .into_iter()
                        .map(|(key, value)| (key.to_string(), value.to_string()))
                        .collect(),
                )
            })
            .collect(),
        )
        .build()
        .unwrap();
    let actual = deserialize_like(registration_json(), &expected);
    assert_eq!(actual, expected);
    assert_eq!(actual.package_id(), "material_editor");
    assert_eq!(actual.contributions().len(), 3);
}

#[test]
fn material_editor_dist_native_contribution_commands_match_every_callback_slot_and_budget() {
    let batch: Value = serde_json::from_str(registration_json()).unwrap();
    let contributions = batch["contributions"].as_array().unwrap();
    let commands: Vec<_> = contributions
        .iter()
        .filter(|item| item["kind"] == "command")
        .collect();
    let manifest = native::command_manifest_v4_from_toml(
        // SAFETY: behavior() returns the immutable SDK static command manifest.
        unsafe { CStr::from_ptr(behavior().command_manifest) }
            .to_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(commands.len(), manifest.commands.len());
    for endpoint in manifest.commands {
        let matching: Vec<_> = commands
            .iter()
            .filter(|command| command["id"] == endpoint.name)
            .collect();
        assert_eq!(matching.len(), 1);
        let command = matching[0];
        assert_eq!(command["schema"], "zircon.editor.command/3");
        assert_eq!(
            command["execution_contract"]["result_codec"],
            "zircon.editor.command-result.v1"
        );
        let budget = &command["execution_contract"]["resource_budget"];
        assert_eq!(budget["max_input_bytes"], MAX_INPUT_BYTES);
        assert_eq!(budget["max_output_bytes"], endpoint.max_output_bytes);
        assert_eq!(budget["max_execution_time_ms"], 5000);
        let bundle = contributions
            .iter()
            .find(|item| item["kind"] == "localization_bundle")
            .unwrap();
        for locale in ["en", "zh-CN"] {
            for key in ["label_key", "description_key"] {
                let name = command[key].as_str().unwrap();
                assert!(!bundle["locales"][locale][name].as_str().unwrap().is_empty());
            }
        }
    }
}

#[test]
fn material_editor_dist_native_without_material_capability_exposes_no_behavior() {
    let host = NativePluginHostFunctionTableV3 {
        abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
        host_handle: 1,
        granted_capabilities: b"\0".as_ptr().cast(),
        host_abi_version: Some(host_abi_version),
        host_has_capability: None,
        host_log: None,
        host_diagnostic: None,
    };
    let report = crate::zircon_plugin_material_editor_editor_entry_v3(&host);
    assert!(!report.is_null());
    // SAFETY: The SDK entry returns a live static missing-capability report.
    let report = unsafe { &*report };
    assert!(report.behavior.is_null());
    assert!(report.bridge_methods.is_null());
}

fn registration_json() -> &'static str {
    // SAFETY: The granted entry points to the immutable static NUL-terminated batch.
    unsafe { CStr::from_ptr(behavior().registration_manifest) }
        .to_str()
        .unwrap()
}

fn deserialize_like<T: serde::de::DeserializeOwned>(json: &str, _expected: &T) -> T {
    serde_json::from_str(json).unwrap()
}

struct Capture {
    bytes: Vec<u8>,
    calls: usize,
    writer_status: u32,
}

fn invoke(
    slot: u32,
    payload: &[u8],
    budget: usize,
    writer_status: u32,
) -> (NativePluginCallbackStatusV3, Capture) {
    invoke_slice(
        slot,
        NativePluginByteSliceV3 {
            data: payload.as_ptr(),
            len: payload.len(),
        },
        budget,
        writer_status,
        true,
    )
}

fn invoke_slice(
    slot: u32,
    payload: NativePluginByteSliceV3,
    budget: usize,
    writer_status: u32,
    writer: bool,
) -> (NativePluginCallbackStatusV3, Capture) {
    let mut capture = Capture {
        bytes: Vec::new(),
        calls: 0,
        writer_status,
    };
    let sink = NativePluginOutputSinkV4 {
        context: (&mut capture as *mut Capture).cast(),
        max_output_bytes: budget,
        write: if writer { Some(write_output) } else { None },
    };
    // SAFETY: Valid test payloads live through the call; deliberately invalid shapes
    // exercise branches that must reject before reading their pointers.
    let status = unsafe { behavior().invoke_command.unwrap()(slot, payload, sink) };
    (status, capture)
}

unsafe extern "C" fn write_output(
    context: *mut c_void,
    bytes: NativePluginByteSliceV3,
) -> NativePluginCallbackStatusV3 {
    // SAFETY: invoke_slice supplies one live exclusive Capture for this synchronous call.
    let capture = unsafe { &mut *context.cast::<Capture>() };
    capture.calls += 1;
    if capture.writer_status == ZIRCON_NATIVE_PLUGIN_STATUS_OK {
        // SAFETY: The command writer borrows its completed encoded result for this call.
        capture
            .bytes
            .extend_from_slice(unsafe { native::bytes_from_slice(bytes) });
    }
    callback_status(capture.writer_status, b"test output writer\0")
}

fn behavior() -> &'static NativePluginBehaviorV4 {
    let host = NativePluginHostFunctionTableV3 {
        abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
        host_handle: 1,
        granted_capabilities: b"editor.extension.material_editor_authoring\0"
            .as_ptr()
            .cast(),
        host_abi_version: Some(host_abi_version),
        host_has_capability: None,
        host_log: None,
        host_diagnostic: None,
    };
    let report = crate::zircon_plugin_material_editor_editor_entry_v3(&host);
    assert!(!report.is_null());
    // SAFETY: The entry returns SDK static tables, not a borrow of the local host value.
    unsafe { &*(*report).behavior }
}

unsafe extern "C" fn host_abi_version() -> u32 {
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION
}

fn graph(value: f32) -> MaterialGraphAsset {
    MaterialGraphAsset {
        uri: AssetUri::parse("res://materials/native-test.mgraph").unwrap(),
        name: "真实输入 / Native command".into(),
        shader: Some(reference("builtin://shader/pbr")),
        nodes: vec![
            MaterialGraphNodeAsset {
                id: "tint".into(),
                position: [0.0, 0.0],
                kind: Node::VectorParameter {
                    name: "tint".into(),
                    default: [value, 0.5, 0.75, 1.0],
                },
            },
            MaterialGraphNodeAsset {
                id: "output".into(),
                position: [200.0, 0.0],
                kind: Node::Output,
            },
        ],
        links: vec![MaterialGraphLinkAsset {
            from_node: "tint".into(),
            from_pin: "value".into(),
            to_node: "output".into(),
            to_pin: "base_color".into(),
        }],
        parameters: BTreeMap::new(),
    }
}
fn reference(locator: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(locator).unwrap())
}
fn request(graph: &MaterialGraphAsset) -> Vec<u8> {
    serde_json::to_vec(&json!({"graph": graph})).unwrap()
}
