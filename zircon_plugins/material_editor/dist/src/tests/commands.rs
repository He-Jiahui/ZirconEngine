use std::collections::BTreeMap;
use std::ffi::{c_void, CStr};

use super::*;
use zircon_plugin_sdk::native::{
    NativePluginBehaviorV4, NativePluginHostFunctionTableV3, ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
};
use zircon_runtime::asset::{
    AssetReference, AssetUri, MaterialGraphLinkAsset, MaterialGraphNodeAsset,
    MaterialGraphNodeKindAsset as Node, MaterialGraphParameterAsset as Parameter,
    MaterialTextureSlotValue,
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
    assert_eq!(manifest.commands.len(), 6);
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
        (
            3,
            "material_editor.graph.parameters",
            "material_editor.graph_parameters.v1",
        ),
        (
            4,
            "material_editor.material.inspect",
            "material_editor.inspect_material.v1",
        ),
        (
            5,
            "material_editor.graph.details",
            "material_editor.graph_details.v1",
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
        u32::MAX,
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
        .command_with_execution_contract(
            "material_editor.graph.parameters",
            "command.material_editor.graph.parameters.label",
            "command.material_editor.graph.parameters.description",
            contract(),
        )
        .command_with_execution_contract(
            "material_editor.material.inspect",
            "command.material_editor.material.inspect.label",
            "command.material_editor.material.inspect.description",
            contract(),
        )
        .command_with_execution_contract(
            "material_editor.graph.details",
            "command.material_editor.graph.details.label",
            "command.material_editor.graph.details.description",
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
                        (
                            "command.material_editor.graph.parameters.label",
                            "Inspect Material Graph Parameters",
                        ),
                        (
                            "command.material_editor.graph.parameters.description",
                            "Read exact declarations and overrides from the submitted authoring graph",
                        ),
                        (
                            "command.material_editor.material.inspect.label",
                            "Inspect Authored Material",
                        ),
                        (
                            "command.material_editor.material.inspect.description",
                            "Read local material overrides and optional shader schema without flattening inheritance",
                        ),
                        (
                            "command.material_editor.graph.details.label",
                            "Inspect Material Graph Selection",
                        ),
                        (
                            "command.material_editor.graph.details.description",
                            "Read original Details targets and common or mixed values for the submitted selection",
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
                        (
                            "command.material_editor.graph.parameters.label",
                            "查看材质图参数",
                        ),
                        (
                            "command.material_editor.graph.parameters.description",
                            "读取本次作者材质图的精确参数声明与覆盖值",
                        ),
                        (
                            "command.material_editor.material.inspect.label",
                            "查看作者材质",
                        ),
                        (
                            "command.material_editor.material.inspect.description",
                            "读取材质局部覆盖与可选着色器结构，保留原继承关系",
                        ),
                        (
                            "command.material_editor.graph.details.label",
                            "查看材质图选择详情",
                        ),
                        (
                            "command.material_editor.graph.details.description",
                            "读取本次选择的原始详情对象及共同值或混合值",
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
    assert_eq!(actual.contributions().len(), 7);
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
        (3, graph_array.as_slice()),
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
        request(&graph),
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

#[test]
fn material_editor_dist_native_parameters_keep_source_order_exact_keys_and_distinct_declarations() {
    let mut graph = graph(0.25);
    if let Node::VectorParameter { name, .. } = &mut graph.nodes[0].kind {
        *name = " 颜色 / Tint ".into();
    }
    graph.nodes.insert(
        0,
        MaterialGraphNodeAsset {
            id: "z / scalar".into(),
            position: [0.0, 0.0],
            kind: Node::ScalarParameter {
                name: "Roughness".into(),
                default: 2.0,
            },
        },
    );
    graph.nodes.push(MaterialGraphNodeAsset {
        id: "a / 同名声明".into(),
        position: [0.0, 0.0],
        kind: Node::VectorParameter {
            name: " 颜色 / Tint ".into(),
            default: [0.9; 4],
        },
    });
    let value = Parameter::Vector([1.5, -0.25, 0.0, 1.0]);
    graph
        .parameters
        .insert(" 颜色 / Tint ".into(), value.clone());
    graph
        .parameters
        .insert("orphan".into(), Parameter::Scalar(7.0));
    let payload = request(&graph);
    let original = payload.clone();
    let (status, capture) = invoke(
        3,
        &payload,
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    assert_eq!(capture.calls, 1);
    let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
    assert_eq!(
        result["graph_uri"],
        serde_json::to_value(&graph.uri).unwrap()
    );
    assert_eq!(
        result["parameters"],
        json!([
            {"node_id":"z / scalar", "key":"Roughness", "default":Parameter::Scalar(2.0),
             "override_present":false, "override_value":null, "effective_value":Parameter::Scalar(2.0)},
            {"node_id":"tint", "key":" 颜色 / Tint ", "default":Parameter::Vector([0.25,0.5,0.75,1.0]),
             "override_present":true, "override_value":value, "effective_value":value},
            {"node_id":"a / 同名声明", "key":" 颜色 / Tint ", "default":Parameter::Vector([0.9;4]),
             "override_present":true, "override_value":value, "effective_value":value}
        ])
    );
    assert_eq!(payload, original);
    assert!(result.get("material").is_none());
}

#[test]
fn material_editor_dist_native_parameters_distinguish_absent_default_and_signed_zero_overrides() {
    let default = Parameter::Vector([0.0, 0.5, 0.75, 1.0]);
    for value in [
        None,
        Some(default.clone()),
        Some(Parameter::Vector([-0.0, 0.5, 0.75, 1.0])),
    ] {
        let mut graph = graph(0.0);
        if let Some(value) = &value {
            graph.parameters.insert("tint".into(), value.clone());
        }
        let (status, capture) = invoke(
            3,
            &request(&graph),
            MAX_OUTPUT_BYTES,
            ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        );
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
        let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
        let row = &result["parameters"][0];
        assert_eq!(row["override_present"], value.is_some());
        assert_eq!(row["override_value"], serde_json::to_value(&value).unwrap());
        assert_eq!(row["default"], serde_json::to_value(&default).unwrap());
        let effective = value.as_ref().unwrap_or(&default);
        assert_eq!(
            row["effective_value"],
            serde_json::to_value(effective).unwrap()
        );
        let Parameter::Vector(expected) = effective else {
            unreachable!()
        };
        assert_eq!(
            row["effective_value"]["value"][0]
                .as_f64()
                .unwrap()
                .is_sign_negative(),
            expected[0].is_sign_negative()
        );
    }
}

#[test]
fn material_editor_dist_native_parameters_preserve_wrong_type_override_and_its_diagnostic() {
    let mut graph = graph(0.25);
    graph
        .parameters
        .insert("tint".into(), Parameter::Scalar(2.0));
    let (status, capture) = invoke(
        3,
        &request(&graph),
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
    let row = &result["parameters"][0];
    assert_eq!(result["ok"], false);
    assert_eq!(row["override_present"], true);
    assert_eq!(
        row["override_value"],
        serde_json::to_value(Parameter::Scalar(2.0)).unwrap()
    );
    assert!(row["effective_value"].is_null());
    assert_eq!(
        row["default"],
        serde_json::to_value(Parameter::Vector([0.25, 0.5, 0.75, 1.0])).unwrap()
    );
    assert!(result["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(
            |d| d["target"] == json!({"kind":"parameter", "name":"tint"})
                && d["message"]
                    .as_str()
                    .unwrap()
                    .contains("incompatible override type")
        ));
}

#[test]
fn material_editor_dist_native_parameters_reject_ambiguous_node_identity_as_one_batch() {
    for id in ["tint", "", " \t"] {
        let mut graph = graph(0.25);
        let mut other = graph.nodes[0].clone();
        other.id = id.into();
        graph.nodes.push(other);
        let (status, capture) = invoke(
            3,
            &request(&graph),
            MAX_OUTPUT_BYTES,
            ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        );
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
        assert_eq!(capture.calls, 1);
        let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
        assert_eq!(result["ok"], false);
        assert_eq!(result["parameters"], json!([]));
        assert_eq!(result["diagnostics"].as_array().unwrap().len(), 1);
        assert_eq!(result["diagnostics"][0]["target"], json!({"kind":"graph"}));
        assert!(result["diagnostics"][0]["message"]
            .as_str()
            .unwrap()
            .contains("unambiguous"));
    }
}

#[test]
fn material_editor_dist_native_parameters_keep_empty_source_and_required_diagnostics() {
    let mut graph = graph(0.25);
    graph.nodes.clear();
    graph.links.clear();
    graph
        .parameters
        .insert("orphan".into(), Parameter::Scalar(1.0));
    let (status, capture) = invoke(
        3,
        &request(&graph),
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
    assert_eq!(result["ok"], false);
    assert_eq!(result["parameters"], json!([]));
    assert!(!result["diagnostics"].as_array().unwrap().is_empty());
    assert!(graph.nodes.is_empty());
    assert_eq!(graph.parameters.len(), 1);
}

#[test]
fn material_editor_dist_native_parameters_use_existing_strict_transport_and_single_publish() {
    let graph = graph(0.25);
    let graph_json = serde_json::to_value(&graph).unwrap();
    for payload in [
        request(&graph)
            .into_iter()
            .chain(b" true".iter().copied())
            .collect(),
        serde_json::to_vec(&json!({"graph":graph_json,"node_id":"tint"})).unwrap(),
        format!(r#"{{"graph":{graph_json},"graph":{graph_json}}}"#).into_bytes(),
    ] {
        let (status, capture) = invoke(
            3,
            &payload,
            MAX_OUTPUT_BYTES,
            ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        );
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
        assert_eq!(capture.calls, 0);
        assert!(capture.bytes.is_empty());
    }
    let payload = request(&graph);
    for budget in [0, 1, 24] {
        let (status, capture) = invoke(3, &payload, budget, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
        assert_eq!(capture.calls, 0);
        assert!(capture.bytes.is_empty());
    }
    let (status, capture) = invoke(
        3,
        &payload,
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_DENIED,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_DENIED);
    assert_eq!(capture.calls, 1);
    assert!(capture.bytes.is_empty());
}

#[test]
fn material_editor_dist_native_material_inspect_keeps_schema_order_groups_and_local_presence() {
    let document = inspect_document();
    let schema = inspect_shader(&document);
    let payload = inspect_request(&document, Some(&schema));
    let before = payload.clone();
    let (status, capture) = invoke(
        4,
        &payload,
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    assert_eq!(capture.calls, 1);
    let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
    assert_eq!(result["ok"], true);
    assert_eq!(result["schema_available"], true);
    assert!(result["projection_error"].is_null());
    assert_eq!(
        result["parent_reference"],
        serde_json::to_value(&document.parent).unwrap()
    );
    assert_eq!(
        result["shader_reference"],
        serde_json::to_value(&document.shader).unwrap()
    );
    let rows = result["projection"]["properties"].as_array().unwrap();
    assert_eq!(
        rows.iter()
            .map(|row| row["name"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["z", "roughness", " roughness ", "未知 参数"]
    );
    assert_eq!(rows[0]["is_overridden"], false);
    assert!(rows[0]["override_value"].is_null());
    assert_eq!(rows[1]["group"], "表面参数 / Surface");
    assert_eq!(rows[1]["label"], "粗糙度：保留显示名");
    assert_eq!(rows[1]["default_value"], 0.5);
    assert_eq!(rows[1]["override_value"], 0.5);
    assert_eq!(rows[1]["is_overridden"], true);
    assert_eq!(rows[2]["override_value"], 0.3);
    assert!(rows[2]["kind"].is_null());
    let textures = result["projection"]["texture_slots"].as_array().unwrap();
    assert_eq!(textures[0]["name"], "base_color");
    assert_eq!(textures[0]["is_overridden"], true);
    assert_eq!(textures[0]["local_value"], json!({}));
    assert!(textures[0]["reference"].is_null());
    assert_eq!(textures[0]["default_fallback"], "white");
    assert_eq!(textures[1]["is_overridden"], false);
    assert!(textures[1]["local_value"].is_null());
    assert_eq!(
        result["options"],
        serde_json::to_value(&document.options).unwrap()
    );
    assert_eq!(
        result["queue"],
        serde_json::to_value(&document.queue).unwrap()
    );
    assert!(result.get("material").is_none());
    assert_eq!(payload, before);
}

#[test]
fn material_editor_dist_native_material_inspect_without_schema_keeps_exact_values_and_texture_identity(
) {
    let document = inspect_document();
    let payload = inspect_request(&document, None);
    let before = payload.clone();
    let (status, capture) = invoke(
        4,
        &payload,
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
    assert_eq!(result["ok"], false);
    assert_eq!(result["schema_available"], false);
    assert!(result["projection_error"].is_null());
    let rows = result["projection"]["properties"].as_array().unwrap();
    assert_eq!(
        rows.iter()
            .map(|row| row["name"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [" roughness ", "roughness", "未知 参数"]
    );
    assert!(rows
        .iter()
        .all(|row| row["kind"].is_null() && row["default_value"].is_null()));
    assert_eq!(
        rows[2]["override_value"].as_i64(),
        Some(9_007_199_254_740_993)
    );
    let detail = result["projection"]["texture_slots"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["name"] == "detail")
        .unwrap();
    assert_eq!(
        detail["reference"],
        serde_json::to_value(&document.textures["detail"].reference).unwrap()
    );
    assert_eq!(
        detail["local_value"],
        serde_json::to_value(&document.textures["detail"]).unwrap()
    );
    assert_eq!(detail["local_value"]["uv_channel"], 3);
    assert!(result["projection"]["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["path"] == "shader" && row["source"] == "dependency_resolution"));
    assert_eq!(payload, before);
}

#[test]
fn material_editor_dist_native_material_inspect_rejects_uuid_or_locator_mismatch_without_rows() {
    let document = inspect_document();
    let schema = inspect_shader(&document);
    let mut wrong_uuid = document.shader.clone();
    wrong_uuid.uuid = reference("builtin://shader/other-identity").uuid;
    assert_ne!(wrong_uuid.uuid, document.shader.uuid);
    for wrong in [wrong_uuid, reference("builtin://shader/other-locator")] {
        let payload = serde_json::to_vec(&json!({
            "document":document, "shader_reference":wrong, "shader":schema,
        }))
        .unwrap();
        let (status, capture) = invoke(
            4,
            &payload,
            MAX_OUTPUT_BYTES,
            ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        );
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
        let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
        assert_eq!(
            result["projection_error"],
            json!({
                "kind":"reference_mismatch", "expected":document.shader, "actual":wrong,
            })
        );
        assert!(result["projection"].is_null());
        assert_eq!(result["ok"], false);
        assert_eq!(result["schema_available"], false);
    }
    let mut wrong = schema;
    wrong.uri = AssetUri::parse("builtin://shader/unrelated-payload").unwrap();
    let (status, capture) = invoke(
        4,
        &inspect_request(&document, Some(&wrong)),
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
    assert_eq!(result["projection_error"]["kind"], "locator_mismatch");
    assert!(result["projection"].is_null());
}

#[test]
fn material_editor_dist_native_material_inspect_rejects_ambiguous_schema_as_one_projection() {
    let document = inspect_document();
    for (texture, kind, name) in [
        (false, "duplicate_property", "roughness"),
        (true, "duplicate_texture_slot", "base_color"),
    ] {
        let mut schema = inspect_shader(&document);
        if texture {
            schema.texture_slots.push(schema.texture_slots[0].clone());
        } else {
            schema
                .property_schema
                .push(schema.property_schema[1].clone());
        }
        let (status, capture) = invoke(
            4,
            &inspect_request(&document, Some(&schema)),
            MAX_OUTPUT_BYTES,
            ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        );
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
        assert_eq!(capture.calls, 1);
        let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
        assert_eq!(result["projection_error"], json!({"kind":kind,"name":name}));
        assert!(result["projection"].is_null());
        assert_eq!(result["ok"], false);
    }
}

#[test]
fn material_editor_dist_native_material_inspect_rejects_unsupported_document_versions_and_keeps_diagnostics(
) {
    for version in [0, 1, 3, u32::MAX] {
        let mut document = inspect_document();
        document.version = version;
        document
            .validation_diagnostics
            .push("作者原诊断必须保留".into());
        let (status, capture) = invoke(
            4,
            &inspect_request(&document, None),
            MAX_OUTPUT_BYTES,
            ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        );
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
        let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
        assert_eq!(
            result["projection_error"],
            json!({
                "kind":"unsupported_document_version","expected":2,"actual":version,
            })
        );
        assert!(result["projection"].is_null());
        assert_eq!(
            result["validation_diagnostics"],
            json!(["作者原诊断必须保留"])
        );
        assert_eq!(result["ok"], false);
    }
}

#[test]
fn material_editor_dist_native_material_inspect_preserves_source_diagnostics_in_original_order() {
    let mut document = inspect_document();
    document.validation_diagnostics = vec!["原作者诊断1".into(), "原作者诊断2".into()];
    let mut schema = inspect_shader(&document);
    schema.validation_diagnostics = vec![
        "wgsl_capture original capture error".into(),
        "原shader诊断".into(),
    ];
    let (status, capture) = invoke(
        4,
        &inspect_request(&document, Some(&schema)),
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
    assert_eq!(result["ok"], false);
    assert_eq!(result["schema_available"], true);
    let rows = result["projection"]["diagnostics"].as_array().unwrap();
    assert_eq!(
        rows.iter()
            .map(|row| row["message"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "原作者诊断1",
            "原作者诊断2",
            "wgsl_capture original capture error",
            "原shader诊断"
        ]
    );
    assert_eq!(rows[0]["path"], "material.validation_diagnostics");
    assert_eq!(rows[2]["path"], "shader.validation_diagnostics");
    assert_eq!(rows[2]["source"], "wgsl_capture");
    assert!(rows[0]["source"].is_null());
}

#[test]
fn material_editor_dist_native_material_inspect_uses_strict_bounded_single_publish_transport() {
    let document = inspect_document();
    let schema = inspect_shader(&document);
    let encoded = serde_json::to_string(&document).unwrap();
    let invalid = vec![
        serde_json::to_vec(&json!([document])).unwrap(),
        serde_json::to_vec(&json!({"document":document,"unexpected":true})).unwrap(),
        serde_json::to_vec(&json!({"document":document,"shader_reference":document.shader}))
            .unwrap(),
        serde_json::to_vec(&json!({"document":document,"shader":schema})).unwrap(),
        format!("{{\"document\":{encoded},\"document\":{encoded}}}").into_bytes(),
    ];
    for payload in invalid {
        let (status, capture) = invoke(
            4,
            &payload,
            MAX_OUTPUT_BYTES,
            ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        );
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
        assert_eq!(capture.calls, 0);
        assert!(capture.bytes.is_empty());
    }
    let payload = inspect_request(&document, Some(&schema));
    for budget in [0, 1, 24] {
        let (status, capture) = invoke(4, &payload, budget, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
        assert_eq!(capture.calls, 0);
        assert!(capture.bytes.is_empty());
    }
    let (status, capture) = invoke(
        4,
        &payload,
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_DENIED,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_DENIED);
    assert_eq!(capture.calls, 1);
    assert!(capture.bytes.is_empty());
}

fn inspect_document() -> ZMaterialDocument {
    let mut document: ZMaterialDocument = serde_json::from_value(json!({
        "version":2,"name":"真实输入 / Material Instance",
        "shader":reference("builtin://shader/pbr"),
        "parent":reference("res://materials/父材质.zmaterial"),
        "overrides":{"roughness":0.5," roughness ":0.3,"未知 参数":9_007_199_254_740_993_i64},
        "textures":{"base_color":{}},"options":{"USE_DETAIL":false},"queue":{"offset":0},
        "editor":{"unknown_author_metadata":"保留，不从投影重建保存"},
    }))
    .unwrap();
    let mut texture = MaterialTextureSlotValue::new(reference("res://textures/原纹理.ztex#color"));
    texture.uv_channel = 3;
    document.textures.insert("detail".into(), texture);
    document
}

fn inspect_shader(document: &ZMaterialDocument) -> ShaderAsset {
    use zircon_runtime::core::framework::render::MaterialPropertyKind;
    let mut schema = shader(&graph(0.25));
    schema.uri = document.shader.locator.clone();
    schema.property_schema = serde_json::from_value(json!([
        {"name":"z","kind":MaterialPropertyKind::Float,"default":0.25},
        {"name":"roughness","kind":MaterialPropertyKind::Float,"default":0.5,
         "editor":{"group":"表面参数 / Surface","label":"粗糙度：保留显示名"}},
    ]))
    .unwrap();
    schema.texture_slots = serde_json::from_value(json!([
        {"name":"base_color","kind":"texture2d","default":"white"},
        {"name":"normal","kind":"texture2d","default":"normal"},
    ]))
    .unwrap();
    schema
}

fn inspect_request(document: &ZMaterialDocument, schema: Option<&ShaderAsset>) -> Vec<u8> {
    let mut value = json!({"document":document});
    if let Some(schema) = schema {
        value["shader_reference"] = serde_json::to_value(&document.shader).unwrap();
        value["shader"] = serde_json::to_value(schema).unwrap();
    }
    serde_json::to_vec(&value).unwrap()
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

#[test]
fn material_editor_dist_native_details_empty_and_output_selection_keep_original_graph_fields() {
    let mut graph = graph(0.25);
    graph
        .parameters
        .insert("精确 key ".into(), Parameter::Scalar(-0.0));
    let mut second_root = graph.nodes[1].clone();
    second_root.id = "第二个输出".into();
    graph.nodes.push(second_root);
    for selected in [vec![], vec!["第二个输出", "output"]] {
        let payload = details_request(&graph, &selected);
        let before = payload.clone();
        let (status, capture) = invoke(
            5,
            &payload,
            MAX_OUTPUT_BYTES,
            ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        );
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
        assert_eq!(capture.calls, 1);
        assert_eq!(payload, before);
        let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
        assert!(result["selection_error"].is_null());
        let projection = &result["projection"];
        assert_eq!(projection["targets"].as_array().unwrap().len(), 1);
        assert_eq!(projection["targets"][0]["kind"], "graph");
        assert_eq!(
            projection["targets"][0]["fields"],
            json!({"name":graph.name,"shader_reference":graph.shader,"parameters":graph.parameters})
        );
        assert!(projection["expression_values"].is_null());
        assert_eq!(
            projection["selected_node_ids"],
            if selected.is_empty() {
                json!([])
            } else {
                json!(["output", "第二个输出"])
            }
        );
    }
}

#[test]
fn material_editor_dist_native_details_keep_source_order_and_exact_expression_identity() {
    let mut graph = graph(0.25);
    graph.nodes[0].id = " 节点 / 一 ".into();
    graph.links[0].from_node = graph.nodes[0].id.clone();
    let second = MaterialGraphNodeAsset {
        id: "Node".into(),
        position: [-13.5, 620.0],
        kind: Node::Multiply,
    };
    graph.nodes.insert(0, second);
    let payload = details_request(&graph, &[" 节点 / 一 ", "output", "Node"]);
    let before = payload.clone();
    let (status, capture) = invoke(
        5,
        &payload,
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
    assert_eq!(
        result["graph_uri"],
        serde_json::to_value(&graph.uri).unwrap()
    );
    assert_eq!(
        result["projection"]["selected_node_ids"],
        json!(["Node", " 节点 / 一 ", "output"])
    );
    let targets = result["projection"]["targets"].as_array().unwrap();
    assert_eq!(targets.len(), 3);
    for (target, node) in targets.iter().take(2).zip(graph.nodes.iter()) {
        assert_eq!(target["kind"], "expression");
        assert_eq!(target["node"], serde_json::to_value(node).unwrap());
    }
    assert_eq!(targets[2]["kind"], "graph");
    assert!(result["projection"]["expression_values"].is_null());
    assert_eq!(payload, before);
}

#[test]
fn material_editor_dist_native_details_scalar_common_mixed_and_signed_zero_are_distinct() {
    for (first, second, mixed) in [
        (0.5, 0.5, false),
        (0.5, 0.75, true),
        (0.0, -0.0, true),
        (-0.0, -0.0, false),
    ] {
        let mut graph = graph(0.25);
        graph.nodes[0].kind = Node::ScalarParameter {
            name: "roughness".into(),
            default: first,
        };
        graph.nodes.insert(
            1,
            MaterialGraphNodeAsset {
                id: "second".into(),
                position: [10.0, 20.0],
                kind: Node::ScalarParameter {
                    name: "roughness".into(),
                    default: second,
                },
            },
        );
        graph
            .parameters
            .insert("roughness".into(), Parameter::Scalar(1.0));
        let (status, capture) = invoke(
            5,
            &details_request(&graph, &["second", "tint"]),
            MAX_OUTPUT_BYTES,
            ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        );
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
        let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
        let field = &result["projection"]["expression_values"];
        assert_eq!(field["kind"], "scalar_default");
        if mixed {
            assert_eq!(field["value"], json!({"kind":"mixed"}));
        } else {
            assert_eq!(field["value"], json!({"kind":"common","value":first}));
            if first == 0.0 {
                assert!(field["value"]["value"].as_f64().unwrap().is_sign_negative());
            }
        }
    }
}

#[test]
fn material_editor_dist_native_details_vector_mixed_channels_never_use_graph_overrides() {
    let mut graph = graph(0.25);
    graph.nodes[0].kind = Node::VectorParameter {
        name: "tint".into(),
        default: [0.25, 0.5, 0.75, 0.0],
    };
    graph.nodes.insert(
        1,
        MaterialGraphNodeAsset {
            id: "second".into(),
            position: [20.0, 50.0],
            kind: Node::VectorParameter {
                name: "tint".into(),
                default: [0.25, 0.75, 0.75, -0.0],
            },
        },
    );
    graph
        .parameters
        .insert("tint".into(), Parameter::Vector([1.0; 4]));
    let (status, capture) = invoke(
        5,
        &details_request(&graph, &["second", "tint"]),
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
    assert_eq!(
        result["projection"]["expression_values"],
        json!({"kind":"vector_default","channels":[
            {"kind":"common","value":0.25}, {"kind":"mixed"},
            {"kind":"common","value":0.75}, {"kind":"mixed"}
        ]})
    );
}

#[test]
fn material_editor_dist_native_details_texture_common_value_requires_full_uuid_and_locator() {
    let first = reference("res://textures/岩石.ztex#base-color");
    let mut changed_locator = first.clone();
    changed_locator.locator = AssetUri::parse("res://textures/other.ztex#base-color").unwrap();
    let mut changed_uuid = first.clone();
    changed_uuid.uuid = reference("res://textures/different-identity.ztex").uuid;
    for (second, common) in [
        (first.clone(), true),
        (changed_locator, false),
        (changed_uuid, false),
    ] {
        let mut graph = graph(0.25);
        graph.nodes[0].kind = Node::TextureSample {
            texture: first.clone(),
        };
        graph.nodes.insert(
            1,
            MaterialGraphNodeAsset {
                id: "second".into(),
                position: [20.0, 60.0],
                kind: Node::TextureSample { texture: second },
            },
        );
        let (status, capture) = invoke(
            5,
            &details_request(&graph, &["second", "tint"]),
            MAX_OUTPUT_BYTES,
            ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        );
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
        let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
        assert_eq!(result["projection"]["expression_values"]["kind"], "texture");
        assert_eq!(
            result["projection"]["expression_values"]["value"],
            if common {
                json!({"kind":"common","value":first})
            } else {
                json!({"kind":"mixed"})
            }
        );
    }
}

#[test]
fn material_editor_dist_native_details_stale_or_ambiguous_identity_rejects_the_entire_projection() {
    let original = graph(0.25);
    let mut duplicate = original.clone();
    duplicate.nodes.push(duplicate.nodes[0].clone());
    let mut blank = original.clone();
    blank.nodes[0].id = "   ".into();
    for (graph, selected) in [
        (&original, vec!["tint", "deleted"]),
        (&original, vec!["tint", "tint"]),
        (&original, vec!["tint", "Tint"]),
        (&original, vec![" "]),
        (&duplicate, vec!["output"]),
        (&blank, vec!["output"]),
    ] {
        let (status, capture) = invoke(
            5,
            &details_request(graph, &selected),
            MAX_OUTPUT_BYTES,
            ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        );
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
        assert_eq!(capture.calls, 1);
        let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
        assert_eq!(result["ok"], false);
        assert!(result["projection"].is_null());
        assert!(!result["selection_error"]["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty());
    }
}

#[test]
fn material_editor_dist_native_details_broken_graph_stays_inspectable_with_original_diagnostics() {
    let mut graph = graph(0.25);
    graph.shader = None;
    graph.links.clear();
    let expected: Vec<_> = validate_material_graph_with_diagnostics(&graph)
        .iter()
        .map(diagnostic_json)
        .collect();
    assert!(!expected.is_empty());
    for selected in [vec![], vec!["tint"], vec!["output"]] {
        let (status, capture) = invoke(
            5,
            &details_request(&graph, &selected),
            MAX_OUTPUT_BYTES,
            ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        );
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
        let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
        assert_eq!(result["ok"], false);
        assert!(result["selection_error"].is_null());
        assert!(!result["projection"].is_null());
        assert_eq!(result["diagnostics"], json!(expected));
    }
}

#[test]
fn material_editor_dist_native_details_requires_explicit_strict_selection_arguments() {
    let graph = serde_json::to_value(graph(0.25)).unwrap();
    for payload in [
        serde_json::to_vec(&json!({"graph":graph})).unwrap(),
        serde_json::to_vec(&json!({"graph":graph,"selected_node_ids":null})).unwrap(),
        serde_json::to_vec(&json!({"graph":graph,"selected_node_ids":[4]})).unwrap(),
        serde_json::to_vec(&json!({"graph":graph,"selected_node_ids":[],"other":true})).unwrap(),
        serde_json::to_vec(&json!([graph, []])).unwrap(),
        format!(r#"{{"graph":{graph},"selected_node_ids":[],"selected_node_ids":[]}}"#)
            .into_bytes(),
        format!(r#"{{"graph":{graph},"graph":{graph},"selected_node_ids":[]}}"#).into_bytes(),
    ] {
        let (status, capture) = invoke(
            5,
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
fn material_editor_dist_native_details_uses_original_single_publish_and_output_budget() {
    let payload = details_request(&graph(0.25), &["tint"]);
    for budget in [0, 1, 24] {
        let (status, capture) = invoke(5, &payload, budget, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
        assert_eq!(capture.calls, 0);
        assert!(capture.bytes.is_empty());
    }
    let (status, capture) = invoke(
        5,
        &payload,
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_DENIED,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_DENIED);
    assert_eq!(capture.calls, 1);
    assert!(capture.bytes.is_empty());
}

fn details_request(graph: &MaterialGraphAsset, selected_node_ids: &[&str]) -> Vec<u8> {
    serde_json::to_vec(&json!({"graph":graph,"selected_node_ids":selected_node_ids})).unwrap()
}

#[test]
fn material_editor_dist_native_numeric_overflow_in_defaults_never_becomes_null() {
    for number in [1.0e100_f64, -1.0e100_f64] {
        let mut scalar = serde_json::to_value(graph(0.25)).unwrap();
        scalar["nodes"][0]["kind"] =
            json!({"kind":"scalar_parameter","name":"roughness","default":number});
        let mut variants = vec![scalar];
        for channel in 0..4 {
            let mut vector = serde_json::to_value(graph(0.25)).unwrap();
            vector["nodes"][0]["kind"]["default"][channel] = json!(number);
            variants.push(vector);
        }
        for graph in variants {
            for slot in [0, 1, 2, 3, 5] {
                let payload = numeric_graph_request(slot, graph.clone());
                let before = payload.clone();
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
                assert_eq!(capture.calls, 0);
                assert!(capture.bytes.is_empty());
                assert_eq!(payload, before);
            }
        }
    }
}

#[test]
fn material_editor_dist_native_numeric_overflow_in_unselected_positions_rejects_all_slots() {
    for number in [1.0e100_f64, -1.0e100_f64] {
        for coordinate in 0..2 {
            let mut graph = serde_json::to_value(graph(0.25)).unwrap();
            // Details selects tint: a bad unselected Output position is still source input.
            graph["nodes"][1]["position"][coordinate] = json!(number);
            for slot in [0, 1, 2, 3, 5] {
                let (status, capture) = invoke(
                    slot,
                    &numeric_graph_request(slot, graph.clone()),
                    MAX_OUTPUT_BYTES,
                    ZIRCON_NATIVE_PLUGIN_STATUS_OK,
                );
                assert_eq!(
                    status.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
                    "slot {slot}"
                );
                assert_eq!(capture.calls, 0);
                assert!(capture.bytes.is_empty());
            }
        }
    }
}

#[test]
fn material_editor_dist_native_numeric_overflow_in_orphan_overrides_publishes_no_prefix() {
    for number in [1.0e100_f64, -1.0e100_f64] {
        let mut values = vec![json!({"kind":"scalar","value":number})];
        for channel in 0..4 {
            let mut value = json!({"kind":"vector","value":[0.25,0.5,0.75,1.0]});
            value["value"][channel] = json!(number);
            values.push(value);
        }
        for value in values {
            let mut graph = serde_json::to_value(graph(0.25)).unwrap();
            graph["parameters"]["undeclared key"] = value;
            for slot in [0, 1, 2, 3, 5] {
                let (status, capture) = invoke(
                    slot,
                    &numeric_graph_request(slot, graph.clone()),
                    MAX_OUTPUT_BYTES,
                    ZIRCON_NATIVE_PLUGIN_STATUS_OK,
                );
                assert_eq!(
                    status.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
                    "slot {slot}"
                );
                assert_eq!(capture.calls, 0);
                assert!(capture.bytes.is_empty());
            }
        }
    }
}

#[test]
fn material_editor_dist_native_numeric_finite_hdr_subnormal_and_signed_zero_keep_their_bits() {
    let values = [f32::MAX, f32::MIN, f32::from_bits(1), -0.0];
    let mut graph = graph(0.25);
    graph.name = "1e100 is a name, not a numeric token".into();
    graph.nodes[0].position = [f32::MAX, f32::MIN];
    graph.nodes[0].kind = Node::VectorParameter {
        name: "tint".into(),
        default: values,
    };
    graph
        .parameters
        .insert("tint".into(), Parameter::Vector(values));
    for slot in [3, 5] {
        let payload = numeric_graph_request(slot, serde_json::to_value(&graph).unwrap());
        let before = payload.clone();
        let (status, capture) = invoke(
            slot,
            &payload,
            MAX_OUTPUT_BYTES,
            ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        );
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
        assert_eq!(capture.calls, 1);
        let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
        for (channel, expected) in values.iter().enumerate() {
            let actual = if slot == 3 {
                &result["parameters"][0]["default"]["value"][channel]
            } else {
                &result["projection"]["expression_values"]["channels"][channel]["value"]
            };
            assert_eq!(
                (actual.as_f64().unwrap() as f32).to_bits(),
                expected.to_bits()
            );
            if slot == 3 {
                let local = &result["parameters"][0]["override_value"]["value"][channel];
                assert_eq!(
                    (local.as_f64().unwrap() as f32).to_bits(),
                    expected.to_bits()
                );
            }
        }
        assert_eq!(payload, before);
    }
}

#[test]
fn material_editor_dist_native_numeric_graph_guard_keeps_f64_author_document_values() {
    // ZMaterialDocument uses TOML f64 values: this is not an f32 graph field.
    let mut value: Value =
        serde_json::from_slice(&inspect_request(&inspect_document(), None)).unwrap();
    value["document"]["overrides"]["finite f64"] = json!(1.0e100_f64);
    let payload = serde_json::to_vec(&value).unwrap();
    let (status, capture) = invoke(
        4,
        &payload,
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    assert_eq!(capture.calls, 1);
    let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
    let row = result["projection"]["properties"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["name"] == "finite f64")
        .unwrap();
    assert_eq!(row["override_value"].as_f64(), Some(1.0e100_f64));
    assert_eq!(serde_json::from_slice::<Value>(&payload).unwrap(), value);
}

fn numeric_graph_request(slot: u32, graph_value: Value) -> Vec<u8> {
    let source = graph(0.25);
    let mut value: Value = match slot {
        2 => serde_json::from_slice(&shader_request(
            &source,
            source.shader.as_ref().unwrap(),
            &shader(&source),
        ))
        .unwrap(),
        5 => json!({"selected_node_ids":["tint"]}),
        0 | 1 | 3 => json!({}),
        _ => panic!("unexpected numeric graph test slot"),
    };
    value["graph"] = graph_value;
    serde_json::to_vec(&value).unwrap()
}
