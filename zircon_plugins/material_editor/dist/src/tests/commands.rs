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
    assert_eq!(manifest.commands.len(), 3);
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
        (
            2,
            "material_editor.graph.compile_shader_contract",
            "material_editor.compile_graph_shader.v1",
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
        3,
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
        .command_with_execution_contract(
            "material_editor.graph.compile_shader_contract",
            "command.material_editor.graph.compile_shader_contract.label",
            "command.material_editor.graph.compile_shader_contract.description",
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
                        (
                            "command.material_editor.graph.compile_shader_contract.label",
                            "Check Compiled Material Shader Contract",
                        ),
                        (
                            "command.material_editor.graph.compile_shader_contract.description",
                            "Compile the submitted graph and check the supplied shader contract",
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
                        (
                            "command.material_editor.graph.compile_shader_contract.label",
                            "校验编译材质的着色器契约",
                        ),
                        (
                            "command.material_editor.graph.compile_shader_contract.description",
                            "编译本次材质图并校验传入的着色器契约",
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
    assert_eq!(actual.contributions().len(), 4);
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

#[test]
fn material_editor_dist_native_shader_contract_returns_actual_material_without_mutating_sources() {
    for value in [0.125, 0.875] {
        let graph = graph(value);
        let shader = shader(&graph);
        let reference = graph.shader.as_ref().unwrap();
        let expected = compile_material_graph_with_shader_contract(&graph, &shader).unwrap();
        let payload = shader_request(&graph, reference, &shader);
        let original = payload.clone();
        let (status, capture) = invoke(
            2,
            &payload,
            MAX_OUTPUT_BYTES,
            ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        );
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
        assert_eq!(capture.calls, 1);
        let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
        assert_eq!(result["ok"], true);
        assert_eq!(
            result["graph_uri"],
            serde_json::to_value(&graph.uri).unwrap()
        );
        assert_eq!(
            result["shader_reference"],
            serde_json::to_value(reference).unwrap()
        );
        assert_eq!(result["diagnostics"], json!([]));
        assert!(result["shader_error"].is_null());
        assert_eq!(result["material"], serde_json::to_value(expected).unwrap());
        assert_eq!(result["material"]["base_color"][0], json!(value));
        assert_eq!(payload, original);
    }
}

#[test]
fn material_editor_dist_native_shader_contract_rejects_both_reference_mismatches_before_compile() {
    let mut graph = graph(0.25);
    let expected = graph.shader.clone().unwrap();
    let mut wrong_uuid = reference("builtin://shader/another-identity");
    wrong_uuid.locator = expected.locator.clone();
    let mut wrong_locator = expected.clone();
    wrong_locator.locator = AssetUri::parse("builtin://shader/renamed").unwrap();
    // Graph and shader would also fail. Full source identity must reject first.
    graph.links.clear();
    let mut shader = shader(&graph);
    shader
        .validation_diagnostics
        .push("真实着色器导入失败".into());
    for actual in [wrong_uuid, wrong_locator] {
        let payload = shader_request(&graph, &actual, &shader);
        let (status, capture) = invoke(
            2,
            &payload,
            MAX_OUTPUT_BYTES,
            ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        );
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
        let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
        assert_eq!(result["ok"], false);
        assert!(result["material"].is_null());
        assert_eq!(result["diagnostics"], json!([]));
        assert_eq!(
            result["shader_error"],
            json!({
                "kind":"reference_mismatch", "expected":expected, "actual":actual
            })
        );
    }
}

#[test]
fn material_editor_dist_native_shader_contract_preserves_wrong_shader_uri_without_node_target() {
    let graph = graph(0.25);
    let reference = graph.shader.as_ref().unwrap();
    let mut shader = shader(&graph);
    shader.uri = AssetUri::parse("res://shaders/另一份很长的着色器.zshader").unwrap();
    let expected_error =
        json!({"kind":"locator_mismatch", "expected":reference, "actual":shader.uri});
    let (status, capture) = invoke(
        2,
        &shader_request(&graph, reference, &shader),
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
    assert_eq!(result["ok"], false);
    assert_eq!(result["diagnostics"], json!([]));
    assert_eq!(result["shader_error"], expected_error);
    assert!(result["material"].is_null());
}

#[test]
fn material_editor_dist_native_shader_contract_keeps_import_diagnostics_in_original_order() {
    let graph = graph(0.25);
    let reference = graph.shader.as_ref().unwrap();
    let mut shader = shader(&graph);
    shader.validation_diagnostics = vec![
        "properties.颜色 的真实导入失败".into(),
        "texture detail_mask: incompatible capture".into(),
    ];
    let expected_error =
        json!({"kind":"import_diagnostics", "diagnostics":shader.validation_diagnostics});
    let (status, capture) = invoke(
        2,
        &shader_request(&graph, reference, &shader),
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
    assert_eq!(result["ok"], false);
    assert_eq!(result["diagnostics"], json!([]));
    assert_eq!(result["shader_error"], expected_error);
    assert!(result["material"].is_null());
}

#[test]
fn material_editor_dist_native_shader_contract_retains_runtime_error_variants_sources_and_paths() {
    use zircon_runtime::asset::{ShaderMaterialPropertyAsset, ShaderTextureSlotAsset};
    use zircon_runtime::core::framework::render::MaterialPropertyKind;
    let graph = graph(0.25);
    let reference = graph.shader.as_ref().unwrap();
    let mut shader = shader(&graph);
    shader.property_schema.push(ShaderMaterialPropertyAsset {
        name: "detail_strength".into(),
        kind: MaterialPropertyKind::Float,
        required: true,
        default: None,
        editor: BTreeMap::new(),
    });
    shader.texture_slots.push(ShaderTextureSlotAsset {
        name: "detail_mask".into(),
        kind: "texture2d".into(),
        required: true,
        default: None,
        sampler: None,
        group: None,
        label: None,
        option: None,
        st: false,
        editor: BTreeMap::new(),
    });
    shader.regenerate_material_artifact();
    let Err(MaterialGraphShaderCompileError::ShaderContract(errors)) =
        compile_material_graph_with_shader_contract(&graph, &shader)
    else {
        panic!("required fields must fail the actual Runtime contract");
    };
    assert_eq!(errors.len(), 2);
    let expected = serde_json::to_value(&errors).unwrap();
    let (status, capture) = invoke(
        2,
        &shader_request(&graph, reference, &shader),
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
    assert_eq!(result["ok"], false);
    assert_eq!(result["diagnostics"], json!([]));
    assert_eq!(result["shader_error"]["kind"], "contract");
    assert_eq!(result["shader_error"]["diagnostics"], expected);
    assert!(expected
        .as_array()
        .unwrap()
        .iter()
        .any(|d| d["source"] == "shader_schema" && d["path"] == "overrides.detail_strength"));
    assert!(expected
        .as_array()
        .unwrap()
        .iter()
        .any(|d| d["source"] == "shader_schema" && d["path"] == "textures.detail_mask"));
    assert!(result["material"].is_null());
}

#[test]
fn material_editor_dist_native_shader_contract_keeps_graph_errors_in_graph_coordinates() {
    let mut graph = graph(0.25);
    graph.links[0].from_pin = "missing / 输入".into();
    let reference = graph.shader.as_ref().unwrap();
    let mut shader = shader(&graph);
    shader
        .validation_diagnostics
        .push("must not replace graph error".into());
    let expected: Vec<_> = compile_material_graph_with_diagnostics(&graph)
        .unwrap_err()
        .iter()
        .map(diagnostic_json)
        .collect();
    let (status, capture) = invoke(
        2,
        &shader_request(&graph, reference, &shader),
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
    assert_eq!(result["ok"], false);
    assert_eq!(result["diagnostics"], json!(expected));
    assert!(result["shader_error"].is_null());
    assert!(result["material"].is_null());
}

#[test]
fn material_editor_dist_native_shader_contract_rejects_missing_extra_and_duplicate_arguments() {
    let graph = graph(0.25);
    let reference = graph.shader.as_ref().unwrap();
    let shader = shader(&graph);
    let valid = json!({"graph":graph, "shader_reference":reference, "shader":shader});
    let mut extra = valid.clone();
    extra["unrecognized"] = json!(true);
    let mut missing = valid.clone();
    missing.as_object_mut().unwrap().remove("shader_reference");
    let mut invalid = valid.clone();
    invalid["shader"] = Value::Null;
    let mut invalid_requests = vec![
        request(&graph),
        serde_json::to_vec(&extra).unwrap(),
        serde_json::to_vec(&missing).unwrap(),
        serde_json::to_vec(&invalid).unwrap(),
    ];
    for key in ["graph", "shader_reference", "shader"] {
        invalid_requests.push(
            format!(
                r#"{{"graph":{},"shader_reference":{},"shader":{},"{key}":{}}}"#,
                valid["graph"], valid["shader_reference"], valid["shader"], valid[key]
            )
            .into_bytes(),
        );
    }
    for bytes in invalid_requests {
        let (status, capture) = invoke(2, &bytes, MAX_OUTPUT_BYTES, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
        assert_eq!(capture.calls, 0);
        assert!(capture.bytes.is_empty());
    }
}

#[test]
fn material_editor_dist_native_shader_contract_budget_failure_never_publishes_a_prefix() {
    let graph = graph(0.25);
    let shader = shader(&graph);
    let payload = shader_request(&graph, graph.shader.as_ref().unwrap(), &shader);
    for budget in [0, 1, 24] {
        let (status, capture) = invoke(2, &payload, budget, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
        assert_eq!(capture.calls, 0);
        assert!(capture.bytes.is_empty());
    }
}

#[test]
fn material_editor_dist_native_rejects_complete_positional_arrays_without_output() {
    let graph = graph(0.25);
    let shader = shader(&graph);
    let graph_array = serde_json::to_vec(&json!([&graph])).unwrap();
    let shader_array =
        serde_json::to_vec(&json!([&graph, graph.shader.as_ref().unwrap(), &shader])).unwrap();
    for (slot, array) in [
        (0, graph_array.as_slice()),
        (1, graph_array.as_slice()),
        (2, shader_array.as_slice()),
    ] {
        for whitespace in ["", " \t\r\n"] {
            let mut payload = whitespace.as_bytes().to_vec();
            payload.extend_from_slice(array);
            payload.extend_from_slice(whitespace.as_bytes());
            let (status, capture) = invoke(
                slot,
                &payload,
                MAX_OUTPUT_BYTES,
                ZIRCON_NATIVE_PLUGIN_STATUS_OK,
            );
            assert_eq!(
                status.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
                "slot {slot}"
            );
            assert_eq!(capture.calls, 0, "slot {slot}");
            assert!(capture.bytes.is_empty(), "slot {slot}");
        }
    }
}

#[test]
fn material_editor_dist_native_keeps_json_whitespace_and_nested_arrays_in_named_requests() {
    let graph = graph(0.25);
    let shader = shader(&graph);
    let requests = [
        request(&graph),
        request(&graph),
        shader_request(&graph, graph.shader.as_ref().unwrap(), &shader),
    ];
    for (slot, request) in requests.iter().enumerate() {
        let slot = slot as u32;
        let (status, expected) = invoke(
            slot,
            request,
            MAX_OUTPUT_BYTES,
            ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        );
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK, "slot {slot}");
        assert_eq!(expected.calls, 1, "slot {slot}");
        for whitespace in [" ", "\t", "\r", "\n", " \t\r\n"] {
            let mut payload = whitespace.as_bytes().to_vec();
            payload.extend_from_slice(request);
            payload.extend_from_slice(whitespace.as_bytes());
            let (status, capture) = invoke(
                slot,
                &payload,
                MAX_OUTPUT_BYTES,
                ZIRCON_NATIVE_PLUGIN_STATUS_OK,
            );
            assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK, "slot {slot}");
            assert_eq!(capture.calls, 1, "slot {slot}");
            assert_eq!(capture.bytes, expected.bytes, "slot {slot}");
        }
    }
}

// Same Runtime ShaderAsset DTO and schema defaults as the source47 contract tests.
// This descriptor fixture does not establish WGSL/GPU readiness or a preview producer.
fn shader(graph: &MaterialGraphAsset) -> ShaderAsset {
    use zircon_runtime::asset::ShaderSourceLanguage;
    use zircon_runtime::core::framework::render::ShaderAssetKind;
    serde_json::from_value(json!({
        "uri":graph.shader.as_ref().unwrap().locator,
        "kind":ShaderAssetKind::Surface,
        "source_language":ShaderSourceLanguage::Wgsl,
        "source":"",
    }))
    .unwrap()
}

fn shader_request(
    graph: &MaterialGraphAsset,
    reference: &AssetReference,
    shader: &ShaderAsset,
) -> Vec<u8> {
    serde_json::to_vec(&json!({"graph":graph,"shader_reference":reference,"shader":shader}))
        .unwrap()
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
