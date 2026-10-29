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
    assert_eq!(manifest.commands.len(), 11);
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
        (
            6,
            "material_editor.graph.plan_parameter_edit",
            "material_editor.plan_graph_parameter_edit.v1",
        ),
        (
            7,
            "material_editor.graph.search_nodes",
            "material_editor.search_graph_nodes.v1",
        ),
        (
            8,
            "material_editor.graph.plan_node_creation",
            "material_editor.plan_graph_node_creation.v1",
        ),
        (
            9,
            "material_editor.graph.plan_node_deletion",
            "material_editor.plan_graph_node_deletion.v1",
        ),
        (
            10,
            "material_editor.graph.plan_connection",
            "material_editor.plan_graph_connection.v1",
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
        .command_with_execution_contract(
            "material_editor.graph.plan_parameter_edit",
            "command.material_editor.graph.plan_parameter_edit.label",
            "command.material_editor.graph.plan_parameter_edit.description",
            contract(),
        )
        .command_with_execution_contract(
            "material_editor.graph.search_nodes",
            "command.material_editor.graph.search_nodes.label",
            "command.material_editor.graph.search_nodes.description",
            contract(),
        )
        .command_with_execution_contract(
            "material_editor.graph.plan_node_creation",
            "command.material_editor.graph.plan_node_creation.label",
            "command.material_editor.graph.plan_node_creation.description",
            contract(),
        )
        .command_with_execution_contract(
            "material_editor.graph.plan_node_deletion",
            "command.material_editor.graph.plan_node_deletion.label",
            "command.material_editor.graph.plan_node_deletion.description",
            contract(),
        )
        .command_with_execution_contract(
            "material_editor.graph.plan_connection",
            "command.material_editor.graph.plan_connection.label",
            "command.material_editor.graph.plan_connection.description",
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
                        (
                            "command.material_editor.graph.plan_parameter_edit.label",
                            "Plan Material Graph Parameter Edit",
                        ),
                        (
                            "command.material_editor.graph.plan_parameter_edit.description",
                            "Plan one presence-sensitive parameter override edit without applying it",
                        ),
                        (
                            "command.material_editor.graph.search_nodes.label",
                            "Search Material Nodes",
                        ),
                        (
                            "command.material_editor.graph.search_nodes.description",
                            "Read exact registered material actions and pin keys for the submitted query",
                        ),
                        (
                            "command.material_editor.graph.plan_node_creation.label",
                            "Plan Material Node Creation",
                        ),
                        (
                            "command.material_editor.graph.plan_node_creation.description",
                            "Preflight one typed node and optional autowire without changing the authoring graph",
                        ),
                        (
                            "command.material_editor.graph.plan_node_deletion.label",
                            "Plan Material Node Deletion",
                        ),
                        (
                            "command.material_editor.graph.plan_node_deletion.description",
                            "Preflight original nodes and incident link occurrences without applying a deletion",
                        ),
                        (
                            "command.material_editor.graph.plan_connection.label",
                            "Plan Material Connection",
                        ),
                        (
                            "command.material_editor.graph.plan_connection.description",
                            "Preflight exact pin keys and one input replacement without changing the authoring graph",
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
                        (
                            "command.material_editor.graph.plan_parameter_edit.label",
                            "规划材质图参数编辑",
                        ),
                        (
                            "command.material_editor.graph.plan_parameter_edit.description",
                            "规划一个保留覆盖存在性的参数编辑候选，不提交作者事务",
                        ),
                        (
                            "command.material_editor.graph.search_nodes.label",
                            "搜索材质节点",
                        ),
                        (
                            "command.material_editor.graph.search_nodes.description",
                            "按原始查询读取已注册材质动作与精确端口键",
                        ),
                        (
                            "command.material_editor.graph.plan_node_creation.label",
                            "规划材质节点创建",
                        ),
                        (
                            "command.material_editor.graph.plan_node_creation.description",
                            "预检一个类型化节点及可选自动连线，不提交作者材质图",
                        ),
                        (
                            "command.material_editor.graph.plan_node_deletion.label",
                            "规划材质节点删除",
                        ),
                        (
                            "command.material_editor.graph.plan_node_deletion.description",
                            "预检原始节点及关联连线，不提交删除事务",
                        ),
                        (
                            "command.material_editor.graph.plan_connection.label",
                            "规划材质连线",
                        ),
                        (
                            "command.material_editor.graph.plan_connection.description",
                            "预检精确端口键及一个输入替换，不提交作者材质图",
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
    assert_eq!(actual.contributions().len(), 12);
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

#[test]
fn material_editor_dist_native_parameter_edit_preserves_presence_and_signed_zero() {
    for previous in [
        None,
        Some(Parameter::Scalar(0.0)),
        Some(Parameter::Scalar(-0.0)),
        Some(Parameter::Scalar(0.25)),
        Some(Parameter::Vector([0.0, 0.5, 0.75, 1.0])),
    ] {
        let mut source = parameter_edit_graph();
        if let Some(value) = &previous {
            source
                .parameters
                .insert(" Gain 原始 ".into(), value.clone());
        }
        for next in [0.0_f32, -0.0, 0.25] {
            let result =
                parameter_edit_result(&source, "gain", json!({"kind":"set_scalar","value":next}));
            let plan = &result["plan"];
            assert_eq!(plan["key"], " Gain 原始 ");
            assert_eq!(plan["node_id"], "gain");
            assert_eq!(plan["affected_node_ids"], json!(["gain"]));
            assert_eq!(plan["previous_present"], previous.is_some());
            assert_eq!(
                plan["previous_value"],
                serde_json::to_value(&previous).unwrap()
            );
            if let Some(Parameter::Scalar(before)) = &previous {
                assert_eq!(
                    (plan["previous_value"]["value"].as_f64().unwrap() as f32).to_bits(),
                    before.to_bits(),
                );
            }
            assert_eq!(plan["proposed_present"], true);
            assert_eq!(
                (plan["proposed_value"]["value"].as_f64().unwrap() as f32).to_bits(),
                next.to_bits(),
            );
            let changed = match &previous {
                Some(Parameter::Scalar(before)) => before.to_bits() != next.to_bits(),
                _ => true,
            };
            assert_eq!(plan["changes_source"], changed);
        }
        let reset = parameter_edit_result(&source, "gain", json!({"kind":"reset"}));
        assert_eq!(reset["plan"]["previous_present"], previous.is_some());
        assert_eq!(reset["plan"]["proposed_present"], false);
        assert!(reset["plan"]["proposed_value"].is_null());
        assert_eq!(reset["plan"]["changes_source"], previous.is_some());
    }
}

#[test]
fn material_editor_dist_native_parameter_edit_qualifies_all_declarations_in_source_order() {
    let mut source = parameter_edit_graph();
    let mut second = source
        .nodes
        .iter()
        .find(|node| node.id == "gain")
        .unwrap()
        .clone();
    second.id = "same-key-first".into();
    source.nodes.insert(0, second);
    let result = parameter_edit_result(&source, "gain", json!({"kind":"set_scalar","value":0.75}));
    assert_eq!(
        result["plan"]["affected_node_ids"],
        json!(["same-key-first", "gain"])
    );
    assert_eq!(result["plan"]["key"], " Gain 原始 ");
    source.nodes[0].kind = Node::VectorParameter {
        name: " Gain 原始 ".into(),
        default: [1.0; 4],
    };
    let rejected =
        parameter_edit_result(&source, "gain", json!({"kind":"set_scalar","value":0.75}));
    assert!(rejected["plan"].is_null());
    assert_eq!(
        rejected["edit_error"]["diagnostics"][0]["target"]["node_id"],
        "same-key-first"
    );
    for target in ["missing", "output"] {
        let rejected = parameter_edit_result(&source, target, json!({"kind":"reset"}));
        assert!(rejected["plan"].is_null());
        assert!(!rejected["edit_error"]["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty());
    }
    source.nodes[0].id = "gain".into();
    let duplicate = parameter_edit_result(&source, "gain", json!({"kind":"reset"}));
    assert!(duplicate["plan"].is_null());
}

#[test]
fn material_editor_dist_native_parameter_vector_components_keep_untouched_source_bits() {
    for local in [None, Some([-0.0_f32, -5.0, f32::MAX, 0.5])] {
        let mut source = parameter_edit_graph();
        if let Some(values) = local {
            source
                .parameters
                .insert("tint".into(), Parameter::Vector(values));
        }
        let result = parameter_edit_result(
            &source,
            "tint",
            json!({"kind":"set_vector_components","components":[null,0.875,null,null]}),
        );
        let values = local.unwrap_or([0.25, 0.5, 0.75, 1.0]);
        let plan = &result["plan"];
        assert_eq!(plan["previous_present"], local.is_some());
        assert_eq!(plan["proposed_present"], true);
        for (channel, before) in values.iter().enumerate() {
            let expected = if channel == 1 { 0.875 } else { *before };
            assert_eq!(
                (plan["proposed_value"]["value"][channel].as_f64().unwrap() as f32).to_bits(),
                expected.to_bits(),
            );
        }
    }
    for (target, components) in [
        ("tint", json!([null, null, null, null])),
        ("gain", json!([1.0, null, null, null])),
    ] {
        let result = parameter_edit_result(
            &parameter_edit_graph(),
            target,
            json!({"kind":"set_vector_components","components":components}),
        );
        assert!(result["plan"].is_null());
    }
}

#[test]
fn material_editor_dist_native_parameter_reset_keeps_wrong_type_and_empty_key_for_repair() {
    for key in [" Gain 原始 ", ""] {
        let mut source = parameter_edit_graph();
        source
            .nodes
            .iter_mut()
            .find(|node| node.id == "gain")
            .unwrap()
            .kind = Node::ScalarParameter {
            name: key.into(),
            default: 0.25,
        };
        let previous = Parameter::Vector([-0.0, 0.5, 0.75, 1.0]);
        source.parameters.insert(key.into(), previous.clone());
        let result = parameter_edit_result(&source, "gain", json!({"kind":"reset"}));
        assert_eq!(result["plan"]["key"], key);
        assert_eq!(
            result["plan"]["previous_value"],
            serde_json::to_value(previous).unwrap()
        );
        assert_eq!(result["plan"]["proposed_present"], false);
        assert_eq!(result["plan"]["changes_source"], true);
        assert!(result["edit_error"].is_null());
        let expected: Vec<_> = validate_material_graph_with_diagnostics(&source)
            .iter()
            .map(diagnostic_json)
            .collect();
        assert_eq!(result["diagnostics"], json!(expected));
        assert!(!result["diagnostics"].as_array().unwrap().is_empty());
        assert_eq!(result["ok"], false);
        if key.is_empty() {
            let rejected =
                parameter_edit_result(&source, "gain", json!({"kind":"set_scalar","value":0.25}));
            assert!(rejected["plan"].is_null());
        }
    }
}

#[test]
fn material_editor_dist_native_parameter_edit_rejects_overflow_before_any_candidate() {
    let source = parameter_edit_graph();
    for edit in [
        json!({"kind":"set_scalar","value":1.0e100_f64}),
        json!({"kind":"set_vector","value":[0.0,1.0e100_f64,0.0,1.0]}),
        json!({"kind":"set_vector_components","components":[null,-1.0e100_f64,null,null]}),
    ] {
        let target = if edit["kind"] == "set_scalar" {
            "gain"
        } else {
            "tint"
        };
        let result = parameter_edit_result(&source, target, edit);
        assert!(result["plan"].is_null());
        assert!(!result["edit_error"]["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty());
    }
    for location in 0..3 {
        let mut graph_value = serde_json::to_value(&source).unwrap();
        match location {
            0 => graph_value["nodes"][0]["position"][1] = json!(1.0e100_f64),
            1 => graph_value["nodes"][0]["kind"]["default"][2] = json!(-1.0e100_f64),
            _ => graph_value["parameters"]["orphan"] = json!({"kind":"scalar","value":1.0e100_f64}),
        }
        let payload = serde_json::to_vec(&json!({
            "graph":graph_value, "node_id":"gain", "edit":{"kind":"reset"},
        }))
        .unwrap();
        let (status, capture) = invoke(
            6,
            &payload,
            MAX_OUTPUT_BYTES,
            ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        );
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
        assert_eq!(capture.calls, 0);
        assert!(capture.bytes.is_empty());
    }
    let hdr = parameter_edit_result(
        &source,
        "gain",
        json!({"kind":"set_scalar","value":f32::MAX}),
    );
    assert_eq!(
        (hdr["plan"]["proposed_value"]["value"].as_f64().unwrap() as f32).to_bits(),
        f32::MAX.to_bits()
    );
}

#[test]
fn material_editor_dist_native_parameter_edit_requires_strict_tagged_named_arguments() {
    let source = parameter_edit_graph();
    let graph_json = serde_json::to_string(&source).unwrap();
    let mut invalid = vec![
        b"[]".to_vec(), b"{}".to_vec(),
        serde_json::to_vec(&json!({"graph":source,"edit":{"kind":"reset"}})).unwrap(),
        serde_json::to_vec(&json!({"graph":source,"node_id":"gain","edit":{"kind":"reset"},"extra":true})).unwrap(),
        format!(r#"{{"graph":{graph_json},"graph":{graph_json},"node_id":"gain","edit":{{"kind":"reset"}}}}"#).into_bytes(),
        format!(r#"{{"graph":{graph_json},"node_id":"gain","node_id":"tint","edit":{{"kind":"reset"}}}}"#).into_bytes(),
        format!(r#"{{"graph":{graph_json},"node_id":"gain","edit":{{"kind":"reset","kind":"reset"}}}}"#).into_bytes(),
        format!(r#"{{"graph":{graph_json},"node_id":"gain","edit":{{"kind":"set_scalar","value":1.0,"value":2.0}}}}"#).into_bytes(),
    ];
    for edit in [
        json!(null),
        json!([]),
        json!({}),
        json!({"kind":"unknown"}),
        json!({"kind":"reset","value":0.0}),
        json!({"kind":"set_scalar","value":null}),
        json!({"kind":"set_scalar","value":0.0,"components":[null,null,null,null]}),
        json!({"kind":"set_vector","value":[0.0,1.0,2.0]}),
        json!({"kind":"set_vector_components","components":[null,null,null]}),
    ] {
        invalid.push(parameter_edit_request(&source, "gain", edit));
    }
    for payload in invalid {
        let (status, capture) = invoke(
            6,
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
fn material_editor_dist_native_parameter_edit_keeps_bounded_single_publish_and_host_rejection() {
    let payload = parameter_edit_request(&parameter_edit_graph(), "gain", json!({"kind":"reset"}));
    for budget in [0, 1, 24] {
        let (status, capture) = invoke(6, &payload, budget, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
        assert_eq!(capture.calls, 0);
        assert!(capture.bytes.is_empty());
    }
    let (status, capture) = invoke(
        6,
        &payload,
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_DENIED,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_DENIED);
    assert_eq!(capture.calls, 1);
}

#[test]
fn material_editor_dist_native_parameter_candidate_recompiles_and_raw_inverse_restores_source() {
    let source = graph(0.25);
    let original = serde_json::to_value(&source).unwrap();
    let result = parameter_edit_result(
        &source,
        "tint",
        json!({"kind":"set_vector_components","components":[0.875,null,null,null]}),
    );
    let plan = &result["plan"];
    assert_eq!(plan["changes_source"], true);
    let key = plan["key"].as_str().unwrap().to_owned();
    let previous: Option<Parameter> =
        serde_json::from_value(plan["previous_value"].clone()).unwrap();
    let proposed: Parameter = serde_json::from_value(plan["proposed_value"].clone()).unwrap();
    // This test applies a candidate to its local fixture only; the production callback
    // never mutates an author document or creates a host history/CAS permit.
    let mut edited = source.clone();
    edited.parameters.insert(key.clone(), proposed);
    let (status, capture) = invoke(
        1,
        &request(&edited),
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    let compiled: Value = serde_json::from_slice(&capture.bytes).unwrap();
    assert_eq!(compiled["ok"], true);
    assert_eq!(compiled["material"]["base_color"][0].as_f64(), Some(0.875));
    match previous {
        Some(value) => {
            edited.parameters.insert(key, value);
        }
        None => {
            edited.parameters.remove(&key);
        }
    }
    assert_eq!(serde_json::to_value(&edited).unwrap(), original);
    let (status, capture) = invoke(
        1,
        &request(&edited),
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    let compiled: Value = serde_json::from_slice(&capture.bytes).unwrap();
    assert_eq!(compiled["material"]["base_color"][0].as_f64(), Some(0.25));
}

fn parameter_edit_graph() -> MaterialGraphAsset {
    let mut source = graph(0.25);
    source.nodes.push(MaterialGraphNodeAsset {
        id: "gain".into(),
        position: [0.0, 120.0],
        kind: Node::ScalarParameter {
            name: " Gain 原始 ".into(),
            default: 0.25,
        },
    });
    source
}

fn parameter_edit_request(graph: &MaterialGraphAsset, node_id: &str, edit: Value) -> Vec<u8> {
    serde_json::to_vec(&json!({"graph":graph,"node_id":node_id,"edit":edit})).unwrap()
}

fn parameter_edit_result(graph: &MaterialGraphAsset, node_id: &str, edit: Value) -> Value {
    let payload = parameter_edit_request(graph, node_id, edit);
    let before = payload.clone();
    let (status, capture) = invoke(
        6,
        &payload,
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    assert_eq!(capture.calls, 1);
    assert_eq!(payload, before);
    serde_json::from_slice(&capture.bytes).unwrap()
}

#[test]
fn material_editor_dist_native_node_search_keeps_raw_query_registered_keys_and_pin_types() {
    for (query, ids) in [
        (
            "",
            vec![
                "output",
                "texture_sample",
                "scalar_parameter",
                "vector_parameter",
                "add",
                "multiply",
            ],
        ),
        (
            " \t\u{3000}",
            vec![
                "output",
                "texture_sample",
                "scalar_parameter",
                "vector_parameter",
                "add",
                "multiply",
            ],
        ),
        (" math ", vec!["add", "multiply"]),
        ("MATH multiply", vec!["multiply"]),
        ("parameter scalar", vec!["scalar_parameter"]),
        ("不存在的材质节点", vec![]),
    ] {
        let payload = serde_json::to_vec(&json!({"query":query})).unwrap();
        let before = payload.clone();
        let (status, capture) = invoke(
            7,
            &payload,
            MAX_OUTPUT_BYTES,
            ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        );
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
        assert_eq!(capture.calls, 1);
        assert_eq!(payload, before);
        let result: Value = serde_json::from_slice(&capture.bytes).unwrap();
        assert_eq!(result["query"], query);
        assert_eq!(
            result["actions"]
                .as_array()
                .unwrap()
                .iter()
                .map(|a| a["id"].as_str().unwrap())
                .collect::<Vec<_>>(),
            ids
        );
        assert_eq!(
            result["initial_action_id"],
            serde_json::to_value(ids.first()).unwrap()
        );
        for action in result["actions"].as_array().unwrap() {
            match action["id"].as_str().unwrap() {
                "output" => assert!(action["inputs"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|pin| pin["name"] == "base_color" && pin["required"] == true)),
                "add" | "multiply" => {
                    assert_eq!(action["category"], "Math");
                    assert_eq!(
                        action["inputs"][0],
                        json!({"name":"a","value_type":"float","required":true})
                    );
                    assert_eq!(action["outputs"][0]["name"], "value");
                }
                "texture_sample" => assert_eq!(action["outputs"][0]["value_type"], "vec4"),
                _ => {}
            }
        }
    }
}

#[test]
fn material_editor_dist_native_node_commands_reject_non_named_duplicate_or_unknown_arguments() {
    let node = creation_node("new product", Node::Multiply);
    let creation = creation_payload(&graph(0.25), "math", "multiply", &node, None, None);
    for (slot, value) in [(7, json!({"query":"math"})), (8, creation)] {
        let valid = serde_json::to_string(&value).unwrap();
        let mut extra = value.clone();
        extra["unrelated"] = json!(true);
        let mut wrong_query = value.clone();
        wrong_query["query"] = json!(7);
        let duplicate = valid.replacen("\"query\":", "\"query\":\"stale\",\"query\":", 1);
        for bad in [
            "{}".to_owned(),
            "null".to_owned(),
            "[]".to_owned(),
            format!("[{valid}]"),
            duplicate,
            serde_json::to_string(&extra).unwrap(),
            serde_json::to_string(&wrong_query).unwrap(),
        ] {
            let (status, capture) = invoke(
                slot,
                bad.as_bytes(),
                MAX_OUTPUT_BYTES,
                ZIRCON_NATIVE_PLUGIN_STATUS_OK,
            );
            assert_eq!(
                status.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
                "slot {slot}: {bad}"
            );
            assert_eq!(capture.calls, 0);
            assert!(capture.bytes.is_empty());
        }
    }
}

#[test]
fn material_editor_dist_native_node_creation_uses_clicked_key_and_graph_local_source_position() {
    let source = graph(0.25);
    let node = creation_node(" product / 原始 ", Node::Multiply);
    let result = node_creation_result(&creation_payload(
        &source, "math", "multiply", &node, None, None,
    ));
    assert_eq!(result["ok"], true);
    assert!(result["creation_error"].is_null());
    assert_eq!(result["plan"]["action_id"], "multiply");
    assert_eq!(result["plan"]["node"], serde_json::to_value(&node).unwrap());
    assert_eq!(
        (result["plan"]["node"]["position"][0].as_f64().unwrap() as f32).to_bits(),
        (-0.0_f32).to_bits()
    );
    assert!(result["plan"]["autowire"].is_null());
    assert!(result["plan"]["connection"].is_null());

    // A valid incomplete new node is authorable, not a compiled material result.
    // The source diagnostics stay separate from this request's candidate errors.
    let mut broken = source;
    broken.links.clear();
    let repair = node_creation_result(&creation_payload(
        &broken, "math", "multiply", &node, None, None,
    ));
    assert_eq!(repair["ok"], false);
    assert!(repair["plan"].is_object());
    assert!(repair["creation_error"].is_null());
    assert!(
        repair["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["target"]
                == json!({"kind":"input","node_id":"output","pin_name":"base_color"}))
    );
}

#[test]
fn material_editor_dist_native_node_creation_replaces_one_edge_and_compiles_test_local_inverse() {
    let mut source = parameter_edit_graph();
    let previous = MaterialGraphLinkAsset {
        from_node: "gain".into(),
        from_pin: "value".into(),
        to_node: "output".into(),
        to_pin: "roughness".into(),
    };
    source.links.push(previous.clone());
    let original = serde_json::to_value(&source).unwrap();
    let node = creation_node(
        "粗糙度参数 / new",
        Node::ScalarParameter {
            name: "粗糙度 key ".into(),
            default: 0.875,
        },
    );
    let wire = MaterialGraphLinkAsset {
        from_node: node.id.clone(),
        from_pin: "value".into(),
        to_node: "output".into(),
        to_pin: "roughness".into(),
    };
    let result = node_creation_result(&creation_payload(
        &source,
        "scalar parameter",
        "scalar_parameter",
        &node,
        Some(&wire),
        None,
    ));
    let plan = &result["plan"];
    assert_eq!(result["ok"], true);
    assert_eq!(
        plan["connection"]["action"],
        json!({"kind":"replace","previous_link":previous})
    );
    assert_eq!(plan["connection"]["source_value_type"], "scalar");
    assert_eq!(plan["connection"]["has_undetermined_types"], false);
    assert_eq!(plan["autowire"], serde_json::to_value(&wire).unwrap());

    // Only this unit test applies the plan. Production application belongs to
    // the original host document's one accounted delta/inverse/permit/history.
    let mut edited = source.clone();
    edited
        .nodes
        .push(serde_json::from_value(plan["node"].clone()).unwrap());
    edited.links[1] = serde_json::from_value(plan["autowire"].clone()).unwrap();
    let encoded = edited.to_toml_string().unwrap();
    let reopened = MaterialGraphAsset::from_toml_str(&encoded).unwrap();
    let (status, capture) = invoke(
        1,
        &request(&reopened),
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    let compiled: Value = serde_json::from_slice(&capture.bytes).unwrap();
    assert_eq!(compiled["ok"], true);
    assert_eq!(compiled["material"]["roughness"].as_f64(), Some(0.875));
    edited.nodes.pop();
    edited.links[1] =
        serde_json::from_value(plan["connection"]["action"]["previous_link"].clone()).unwrap();
    assert_eq!(serde_json::to_value(&edited).unwrap(), original);
    let (status, capture) = invoke(
        1,
        &request(&edited),
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    let restored: Value = serde_json::from_slice(&capture.bytes).unwrap();
    assert_eq!(restored["material"]["roughness"].as_f64(), Some(0.25));
    assert_eq!(serde_json::to_value(&source).unwrap(), original);
}

#[test]
fn material_editor_dist_native_node_creation_rejects_stale_keys_ids_and_invalid_wires_without_prefix(
) {
    let source = graph(0.25);
    let node = creation_node("new", Node::Multiply);
    let valid = creation_payload(&source, "math", "multiply", &node, None, None);
    let mut invalid = Vec::new();
    for key in ["add", "Multiply", "unknown"] {
        let mut value = valid.clone();
        value["action_id"] = json!(key);
        invalid.push(value);
    }
    for id in ["tint", "  "] {
        let mut value = valid.clone();
        value["node"]["id"] = json!(id);
        invalid.push(value);
    }
    let mut hidden = valid.clone();
    hidden["query"] = json!("scalar parameter");
    invalid.push(hidden);
    let mut ambiguous = valid.clone();
    ambiguous["graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::to_value(&source.nodes[0]).unwrap());
    invalid.push(ambiguous);
    let mut reserved = valid.clone();
    reserved["graph"]["links"].as_array_mut().unwrap().push(
        json!({"from_node":"new","from_pin":"value","to_node":"output","to_pin":"roughness"}),
    );
    invalid.push(reserved);
    for wire in [
        json!({"from_node":"tint","from_pin":"value","to_node":"output","to_pin":"roughness"}),
        json!({"from_node":"new","from_pin":"value","to_node":"new","to_pin":"a"}),
        json!({"from_node":"new","from_pin":"Value","to_node":"output","to_pin":"roughness"}),
    ] {
        let mut value = valid.clone();
        value["autowire"] = wire;
        invalid.push(value);
    }
    let duplicate_output = creation_node("another output", Node::Output);
    invalid.push(creation_payload(
        &source,
        "output",
        "output",
        &duplicate_output,
        None,
        None,
    ));
    for payload in invalid {
        let result = node_creation_result(&payload);
        assert_eq!(result["ok"], false);
        assert!(result["plan"].is_null());
        assert!(!result["creation_error"]["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty());
    }
}

#[test]
fn material_editor_dist_native_node_texture_creation_requires_full_host_reference_and_actual_kind()
{
    let source = graph(0.25);
    let texture = reference("res://textures/原始材质引用.no-extension-assumption");
    let node = creation_node(
        "纹理节点",
        Node::TextureSample {
            texture: texture.clone(),
        },
    );
    let wire = MaterialGraphLinkAsset {
        from_node: node.id.clone(),
        from_pin: "color".into(),
        to_node: "output".into(),
        to_pin: "base_color".into(),
    };
    let mut wrong_uuid = texture.clone();
    wrong_uuid.uuid = reference("res://textures/another-identity.ztex").uuid;
    let mut wrong_locator = texture.clone();
    wrong_locator.locator = AssetUri::parse("res://textures/renamed.ztex").unwrap();
    for observation in [
        None,
        Some(json!({"reference":wrong_uuid,"kind":AssetKind::Texture})),
        Some(json!({"reference":wrong_locator,"kind":AssetKind::Texture})),
        Some(json!({"reference":texture,"kind":AssetKind::Material})),
    ] {
        let result = node_creation_result(&creation_payload(
            &source,
            "texture",
            "texture_sample",
            &node,
            Some(&wire),
            observation,
        ));
        assert_eq!(result["ok"], false);
        assert!(result["plan"].is_null());
        assert!(
            result["creation_error"]["diagnostics"][0]["message"]
                .as_str()
                .unwrap()
                .contains("could not resolve")
                || result["creation_error"]["diagnostics"][0]["message"]
                    .as_str()
                    .unwrap()
                    .contains("expected Texture")
        );
    }
    let result = node_creation_result(&creation_payload(
        &source,
        "texture",
        "texture_sample",
        &node,
        Some(&wire),
        Some(json!({"reference":texture,"kind":AssetKind::Texture})),
    ));
    assert_eq!(result["ok"], true);
    assert_eq!(
        result["plan"]["node"]["kind"]["texture"],
        serde_json::to_value(&texture).unwrap()
    );
    assert_eq!(result["plan"]["connection"]["source_value_type"], "texture");
    assert_eq!(
        result["plan"]["connection"]["action"]["previous_link"],
        serde_json::to_value(&source.links[0]).unwrap()
    );
}

#[test]
fn material_editor_dist_native_node_creation_rejects_overflow_and_unrelated_resource_observation() {
    let source = graph(0.25);
    let scalar = creation_node(
        "scalar",
        Node::ScalarParameter {
            name: "scalar".into(),
            default: 0.25,
        },
    );
    let vector = creation_node(
        "vector",
        Node::VectorParameter {
            name: "vector".into(),
            default: [0.25, 0.5, 0.75, 1.0],
        },
    );
    let mut invalid = Vec::new();
    for (action, node) in [("scalar_parameter", scalar), ("vector_parameter", vector)] {
        let original = creation_payload(&source, "parameter", action, &node, None, None);
        for channel in 0..2 {
            let mut value = original.clone();
            value["node"]["position"][channel] = json!(1.0e100_f64);
            invalid.push(value);
        }
        if action == "scalar_parameter" {
            let mut value = original.clone();
            value["node"]["kind"]["default"] = json!(-1.0e100_f64);
            invalid.push(value);
        } else {
            for channel in 0..4 {
                let mut value = original.clone();
                value["node"]["kind"]["default"][channel] = json!(1.0e100_f64);
                invalid.push(value);
            }
        }
        let mut source_overflow = original.clone();
        source_overflow["graph"]["nodes"][0]["kind"]["default"][0] = json!(1.0e100_f64);
        invalid.push(source_overflow);
        let mut unrelated = original;
        unrelated["resource_observation"] =
            json!({"reference":reference("res://textures/unused.ztex"),"kind":AssetKind::Texture});
        invalid.push(unrelated);
    }
    for value in invalid {
        let payload = serde_json::to_vec(&value).unwrap();
        let (status, capture) = invoke(
            8,
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
fn material_editor_dist_native_node_commands_publish_once_only_with_full_output_budget() {
    let node = creation_node("new", Node::Multiply);
    for (slot, value) in [
        (7, json!({"query":""})),
        (
            8,
            creation_payload(&graph(0.25), "math", "multiply", &node, None, None),
        ),
    ] {
        let payload = serde_json::to_vec(&value).unwrap();
        let (status, full) = invoke(
            slot,
            &payload,
            MAX_OUTPUT_BYTES,
            ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        );
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
        assert_eq!(full.calls, 1);
        assert!(!full.bytes.is_empty());
        for budget in [0, full.bytes.len() - 1] {
            let (status, capture) = invoke(slot, &payload, budget, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
            assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
            assert_eq!(capture.calls, 0);
            assert!(capture.bytes.is_empty());
        }
        let (status, rejected) = invoke(
            slot,
            &payload,
            MAX_OUTPUT_BYTES,
            ZIRCON_NATIVE_PLUGIN_STATUS_DENIED,
        );
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_DENIED);
        assert_eq!(rejected.calls, 1);
        assert!(rejected.bytes.is_empty());
    }
}

fn creation_node(id: &str, kind: Node) -> MaterialGraphNodeAsset {
    MaterialGraphNodeAsset {
        id: id.into(),
        position: [-0.0, 128.5],
        kind,
    }
}

#[test]
fn material_editor_dist_native_node_creation_keeps_hdr_vector_bits_and_exact_parameter_type() {
    let source = parameter_edit_graph();
    let channels = [0.0_f32, 256.0, f32::from_bits(1), -0.0];
    let node = creation_node(
        "向量 / 新声明",
        Node::VectorParameter {
            name: "tint".into(),
            default: channels,
        },
    );
    let result = node_creation_result(&creation_payload(
        &source,
        "vector parameter",
        "vector_parameter",
        &node,
        None,
        None,
    ));
    assert_eq!(result["ok"], true);
    for (channel, expected) in channels.iter().enumerate() {
        assert_eq!(
            (result["plan"]["node"]["kind"]["default"][channel]
                .as_f64()
                .unwrap() as f32)
                .to_bits(),
            expected.to_bits()
        );
    }
    let wrong_kind = creation_node(
        "wrong type",
        Node::VectorParameter {
            name: " Gain 原始 ".into(),
            default: channels,
        },
    );
    let result = node_creation_result(&creation_payload(
        &source,
        "vector parameter",
        "vector_parameter",
        &wrong_kind,
        None,
        None,
    ));
    assert!(result["plan"].is_null());
    assert!(result["creation_error"]["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|d| d["target"] == json!({"kind":"node","node_id":"gain"})));
    let mut wrong_override = source;
    wrong_override
        .parameters
        .insert("tint".into(), Parameter::Scalar(0.25));
    let result = node_creation_result(&creation_payload(
        &wrong_override,
        "vector parameter",
        "vector_parameter",
        &node,
        None,
        None,
    ));
    assert!(result["plan"].is_null());
    assert!(result["creation_error"]["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|d| d["target"] == json!({"kind":"parameter","name":"tint"})));
}

fn creation_payload(
    graph: &MaterialGraphAsset,
    query: &str,
    action_id: &str,
    node: &MaterialGraphNodeAsset,
    autowire: Option<&MaterialGraphLinkAsset>,
    resource_observation: Option<Value>,
) -> Value {
    json!({"graph":graph,"query":query,"action_id":action_id,"node":node,"autowire":autowire,"resource_observation":resource_observation})
}

fn node_creation_result(value: &Value) -> Value {
    let payload = serde_json::to_vec(value).unwrap();
    let before = payload.clone();
    let (status, capture) = invoke(
        8,
        &payload,
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    assert_eq!(capture.calls, 1);
    assert_eq!(payload, before);
    serde_json::from_slice(&capture.bytes).unwrap()
}

#[test]
fn material_editor_dist_native_parameter_full_vector_set_keeps_hdr_bits_and_type_repair() {
    let values = [f32::MAX, f32::MIN, f32::from_bits(1), -0.0];
    for previous in [
        None,
        Some(Parameter::Vector(values)),
        Some(Parameter::Scalar(-0.0)),
    ] {
        let mut source = parameter_edit_graph();
        if let Some(value) = &previous {
            source.parameters.insert("tint".into(), value.clone());
        }
        let result =
            parameter_edit_result(&source, "tint", json!({"kind":"set_vector","value":values}));
        let plan = &result["plan"];
        assert_eq!(plan["previous_present"], previous.is_some());
        assert_eq!(plan["proposed_present"], true);
        assert_eq!(plan["proposed_value"]["kind"], "vector");
        assert_eq!(
            plan["changes_source"],
            !matches!(&previous, Some(Parameter::Vector(_)))
        );
        for (channel, expected) in values.iter().enumerate() {
            assert_eq!(
                (plan["proposed_value"]["value"][channel].as_f64().unwrap() as f32).to_bits(),
                expected.to_bits(),
            );
        }
        if let Some(Parameter::Scalar(before)) = previous {
            assert_eq!(plan["previous_value"]["kind"], "scalar");
            assert_eq!(
                (plan["previous_value"]["value"].as_f64().unwrap() as f32).to_bits(),
                before.to_bits(),
            );
        }
    }
}

#[test]
fn material_editor_dist_native_node_deletion_empty_root_and_mixed_selection_retains_output() {
    let source = graph(0.25);
    for selected in [vec![], vec!["output"]] {
        let result = node_deletion_result(&source, &selected);
        assert_eq!(result["ok"], true);
        assert_eq!(result["selected_node_ids"], json!(selected));
        assert!(result["deletion_error"].is_null());
        assert_eq!(result["plan"]["can_delete"], false);
        assert_eq!(result["plan"]["removed_nodes"], json!([]));
        assert_eq!(result["plan"]["removed_links"], json!([]));
        let protected = if selected.is_empty() {
            json!([])
        } else {
            json!([source.nodes[1]])
        };
        assert_eq!(result["plan"]["protected_nodes"], protected);
    }
    let result = node_deletion_result(&source, &["output", "tint"]);
    assert_eq!(result["plan"]["can_delete"], true);
    assert_eq!(
        result["plan"]["removed_nodes"],
        json!([{"source_index":0,"node":source.nodes[0]}])
    );
    assert_eq!(
        result["plan"]["removed_links"],
        json!([{"source_index":0,"link":source.links[0]}])
    );
    assert_eq!(result["plan"]["protected_nodes"], json!([source.nodes[1]]));
    assert!(result["plan"].get("parameters").is_none());
}

#[test]
fn material_editor_dist_native_node_deletion_preserves_source_order_and_incident_link_occurrences()
{
    let mut source = parameter_edit_graph();
    source.nodes[2].id = " 参数/节点 ".into();
    source.nodes.push(creation_node("数学 / 尾节点", Node::Add));
    source.links.extend([
        MaterialGraphLinkAsset {
            from_node: " 参数/节点 ".into(),
            from_pin: "value".into(),
            to_node: "数学 / 尾节点".into(),
            to_pin: "a".into(),
        },
        MaterialGraphLinkAsset {
            from_node: "数学 / 尾节点".into(),
            from_pin: "value".into(),
            to_node: "output".into(),
            to_pin: "roughness".into(),
        },
        MaterialGraphLinkAsset {
            from_node: "missing".into(),
            from_pin: "value".into(),
            to_node: "数学 / 尾节点".into(),
            to_pin: "b".into(),
        },
        source.links[0].clone(),
        MaterialGraphLinkAsset {
            from_node: "unrelated missing".into(),
            from_pin: "value".into(),
            to_node: "another missing".into(),
            to_pin: "a".into(),
        },
    ]);
    let selected = ["数学 / 尾节点", "output", " 参数/节点 ", "tint"];
    let result = node_deletion_result(&source, &selected);
    assert_eq!(result["selected_node_ids"], json!(selected));
    assert_eq!(result["ok"], false);
    assert!(!result["diagnostics"].as_array().unwrap().is_empty());
    assert!(result["deletion_error"].is_null());
    let plan = &result["plan"];
    assert_eq!(plan["can_delete"], true);
    assert_eq!(
        plan["removed_nodes"],
        json!([
            {"source_index":0,"node":source.nodes[0]},
            {"source_index":2,"node":source.nodes[2]},
            {"source_index":3,"node":source.nodes[3]}
        ])
    );
    assert_eq!(plan["protected_nodes"], json!([source.nodes[1]]));
    let links = plan["removed_links"].as_array().unwrap();
    assert_eq!(links.len(), 5);
    for (index, entry) in links.iter().enumerate() {
        assert_eq!(entry["source_index"].as_u64(), Some(index as u64));
        assert_eq!(
            entry["link"],
            serde_json::to_value(&source.links[index]).unwrap()
        );
    }
    assert_eq!(links[0]["link"], links[4]["link"]);
    assert_ne!(links[0]["source_index"], links[4]["source_index"]);
}

#[test]
fn material_editor_dist_native_node_deletion_rejects_whole_stale_duplicate_or_ambiguous_selection()
{
    let source = parameter_edit_graph();
    for selected in [
        vec!["Tint"],
        vec!["tint "],
        vec!["missing"],
        vec!["tint", "tint"],
        vec!["gain", "missing"],
        vec![""],
        vec!["\u{3000}"],
    ] {
        let result = node_deletion_result(&source, &selected);
        assert_eq!(result["ok"], false);
        assert!(result["plan"].is_null(), "selection {selected:?}");
        assert!(!result["deletion_error"]["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty());
    }
    let mut duplicate = source.clone();
    duplicate.nodes.push(source.nodes[0].clone());
    let mut blank = source;
    blank.nodes[0].id = "  ".into();
    for ambiguous in [duplicate, blank] {
        let result = node_deletion_result(&ambiguous, &["gain"]);
        assert!(result["plan"].is_null());
        assert!(!result["deletion_error"]["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty());
    }
}

#[test]
fn material_editor_dist_native_node_deletion_test_local_codec_and_raw_inverse_restore_compile() {
    let mut source = parameter_edit_graph();
    let values = [-0.0_f32, 256.0, f32::from_bits(1), -0.0];
    source.nodes.push(creation_node(
        "unconnected",
        Node::VectorParameter {
            name: "Unused / 未连接".into(),
            default: values,
        },
    ));
    source.nodes.push(creation_node(
        "shared declaration survivor",
        Node::ScalarParameter {
            name: " Gain 原始 ".into(),
            default: 0.25,
        },
    ));
    source.links.push(MaterialGraphLinkAsset {
        from_node: "gain".into(),
        from_pin: "value".into(),
        to_node: "output".into(),
        to_pin: "roughness".into(),
    });
    source
        .parameters
        .insert(" Gain 原始 ".into(), Parameter::Scalar(0.125));
    source
        .parameters
        .insert("Orphan / 原作者值".into(), Parameter::Vector(values));
    source
        .parameters
        .insert("tint".into(), Parameter::Vector([0.3, 0.4, 0.5, 0.8]));
    let saved = source.to_toml_string().unwrap();
    let reopened = MaterialGraphAsset::from_toml_str(&saved).unwrap();
    let before = zircon_plugin_material_editor_editor::compile_material_graph(&reopened).unwrap();
    assert_eq!(before.roughness, 0.125);
    let result = node_deletion_result(&reopened, &["unconnected", "output", "gain"]);
    assert_eq!(result["ok"], true);
    let plan = &result["plan"];
    assert_eq!(plan["protected_nodes"], json!([reopened.nodes[1]]));
    assert_eq!(plan["removed_nodes"].as_array().unwrap().len(), 2);
    assert_eq!(plan["removed_links"].as_array().unwrap().len(), 1);
    assert!(plan.get("parameters").is_none());

    // Only this test applies the returned occurrence data to a local clone.
    // Production must use the retained document's single accounted transaction.
    let mut edited = reopened.clone();
    for entry in plan["removed_links"].as_array().unwrap().iter().rev() {
        edited
            .links
            .remove(entry["source_index"].as_u64().unwrap() as usize);
    }
    for entry in plan["removed_nodes"].as_array().unwrap().iter().rev() {
        edited
            .nodes
            .remove(entry["source_index"].as_u64().unwrap() as usize);
    }
    let mut edited = MaterialGraphAsset::from_toml_str(&edited.to_toml_string().unwrap()).unwrap();
    assert_eq!(edited.parameters, reopened.parameters);
    assert!(edited
        .nodes
        .iter()
        .any(|node| node.id == "shared declaration survivor"));
    assert_eq!(
        edited.parameters.get(" Gain 原始 "),
        Some(&Parameter::Scalar(0.125))
    );
    let Some(Parameter::Vector(orphan)) = edited.parameters.get("Orphan / 原作者值") else {
        panic!("the original orphan override must remain authored");
    };
    for (actual, original) in orphan.iter().zip(values) {
        assert_eq!(actual.to_bits(), original.to_bits());
    }
    let after = zircon_plugin_material_editor_editor::compile_material_graph(&edited).unwrap();
    assert_ne!(after.roughness, before.roughness);
    assert_eq!(after.base_color, before.base_color);
    for entry in plan["removed_nodes"].as_array().unwrap() {
        edited.nodes.insert(
            entry["source_index"].as_u64().unwrap() as usize,
            serde_json::from_value(entry["node"].clone()).unwrap(),
        );
    }
    for entry in plan["removed_links"].as_array().unwrap() {
        edited.links.insert(
            entry["source_index"].as_u64().unwrap() as usize,
            serde_json::from_value(entry["link"].clone()).unwrap(),
        );
    }
    assert_eq!(edited.to_toml_string().unwrap(), saved);
    let restored = zircon_plugin_material_editor_editor::compile_material_graph(&edited).unwrap();
    assert_eq!(restored.roughness, before.roughness);
    assert_eq!(restored.base_color, before.base_color);
    let (status, capture) = invoke(
        1,
        &request(&edited),
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    let compiled: Value = serde_json::from_slice(&capture.bytes).unwrap();
    assert_eq!(compiled["ok"], true);
    assert_eq!(compiled["material"]["roughness"].as_f64(), Some(0.125));
}

#[test]
fn material_editor_dist_native_node_deletion_keeps_repair_plan_and_existing_diagnostics_separate() {
    let mut source = graph(0.25);
    source.nodes.push(creation_node(
        "repair",
        Node::ScalarParameter {
            name: "Repair".into(),
            default: 0.125,
        },
    ));
    source.links.extend([
        MaterialGraphLinkAsset {
            from_node: "repair".into(),
            from_pin: "Value".into(),
            to_node: "output".into(),
            to_pin: "roughness".into(),
        },
        MaterialGraphLinkAsset {
            from_node: "unrelated missing".into(),
            from_pin: "value".into(),
            to_node: "output".into(),
            to_pin: "unknown".into(),
        },
    ]);
    let result = node_deletion_result(&source, &["repair"]);
    assert_eq!(result["ok"], false);
    assert!(result["deletion_error"].is_null());
    assert!(!result["diagnostics"].as_array().unwrap().is_empty());
    assert_eq!(result["plan"]["can_delete"], true);
    assert_eq!(
        result["plan"]["removed_nodes"],
        json!([{"source_index":2,"node":source.nodes[2]}])
    );
    assert_eq!(
        result["plan"]["removed_links"],
        json!([{"source_index":1,"link":source.links[1]}])
    );
    let root_only = node_deletion_result(&source, &["output"]);
    assert_eq!(root_only["plan"]["can_delete"], false);
    assert_eq!(root_only["diagnostics"], result["diagnostics"]);
}

#[test]
fn material_editor_dist_native_node_deletion_rejects_non_named_duplicate_overflow_or_unknown_arguments(
) {
    let source = graph(0.25);
    let valid = json!({"graph":source,"selected_node_ids":["tint"]});
    let wire = serde_json::to_string(&valid).unwrap();
    let mut extra = valid.clone();
    extra["unrelated"] = json!(true);
    let mut null_selection = valid.clone();
    null_selection["selected_node_ids"] = Value::Null;
    let mut numeric_selection = valid.clone();
    numeric_selection["selected_node_ids"] = json!([1]);
    let mut overflow = valid.clone();
    overflow["graph"]["nodes"][0]["kind"]["default"][0] = json!(1.0e100_f64);
    for bad in [
        "{}".to_owned(),
        "null".to_owned(),
        "[]".to_owned(),
        format!("[{wire}]"),
        wire.replacen(
            "\"selected_node_ids\":",
            "\"selected_node_ids\":[],\"selected_node_ids\":",
            1,
        ),
        serde_json::to_string(&extra).unwrap(),
        serde_json::to_string(&null_selection).unwrap(),
        serde_json::to_string(&numeric_selection).unwrap(),
        serde_json::to_string(&overflow).unwrap(),
    ] {
        let (status, capture) = invoke(
            9,
            bad.as_bytes(),
            MAX_OUTPUT_BYTES,
            ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        );
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, "{bad}");
        assert_eq!(capture.calls, 0);
        assert!(capture.bytes.is_empty());
    }
}

#[test]
fn material_editor_dist_native_node_deletion_publishes_once_with_exact_budget_and_host_refusal() {
    let payload =
        serde_json::to_vec(&json!({"graph":graph(0.25),"selected_node_ids":["tint","output"]}))
            .unwrap();
    let (status, full) = invoke(
        9,
        &payload,
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    assert_eq!(full.calls, 1);
    assert!(!full.bytes.is_empty());
    for limit in [0, full.bytes.len() - 1] {
        let (status, capture) = invoke(9, &payload, limit, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
        assert_eq!(capture.calls, 0);
        assert!(capture.bytes.is_empty());
    }
    let (status, exact) = invoke(
        9,
        &payload,
        full.bytes.len(),
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    assert_eq!(exact.calls, 1);
    assert_eq!(exact.bytes, full.bytes);
    let (status, denied) = invoke(
        9,
        &payload,
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_DENIED,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_DENIED);
    assert_eq!(denied.calls, 1);
    assert!(denied.bytes.is_empty());
}

fn node_deletion_result(graph: &MaterialGraphAsset, selected: &[&str]) -> Value {
    let payload = serde_json::to_vec(&json!({"graph":graph,"selected_node_ids":selected})).unwrap();
    let before = payload.clone();
    let (status, capture) = invoke(
        9,
        &payload,
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    assert_eq!(capture.calls, 1);
    assert_eq!(payload, before);
    serde_json::from_slice(&capture.bytes).unwrap()
}

#[test]
fn material_editor_dist_native_connection_connect_keeps_source_and_changes_codec_compile() {
    let mut source = parameter_edit_graph();
    source
        .parameters
        .insert(" Gain 原始 ".into(), Parameter::Scalar(0.875));
    source.nodes[2].id = " 增益 / 原始 ".into();
    let link = wire_link(" 增益 / 原始 ", "value", "output", "roughness");
    let before = source.to_toml_string().unwrap();
    let result = connection_result(&source, &link);
    assert_eq!(source.to_toml_string().unwrap(), before);
    assert_eq!(result["ok"], true);
    assert!(result["connection_error"].is_null());
    assert_eq!(result["plan"]["action"], json!({"kind":"connect"}));
    assert_eq!(result["plan"]["source_value_type"], "scalar");
    assert_eq!(result["plan"]["has_undetermined_types"], false);
    assert_eq!(result["plan"]["changes_source"], true);
    assert_eq!(result["plan"]["link"], serde_json::to_value(&link).unwrap());

    // Test-local application exercises the real codec/compiler, not a product
    // history, native renderer or authority to apply an unqualified response.
    let mut local = source.clone();
    local
        .links
        .push(serde_json::from_value(result["plan"]["link"].clone()).unwrap());
    let local = MaterialGraphAsset::from_toml_str(&local.to_toml_string().unwrap()).unwrap();
    let original = zircon_plugin_material_editor_editor::compile_material_graph(&source).unwrap();
    let compiled = zircon_plugin_material_editor_editor::compile_material_graph(&local).unwrap();
    assert_ne!(compiled.roughness, original.roughness);
    assert_eq!(compiled.roughness, 0.875);
    assert_eq!(compiled.base_color, original.base_color);
    assert_eq!(local.parameters, source.parameters);
    assert_eq!(&local.links[..source.links.len()], source.links.as_slice());
}

#[test]
fn material_editor_dist_native_connection_replace_returns_original_occurrence_and_raw_inverse() {
    let mut source = parameter_edit_graph();
    source.nodes.push(creation_node(
        "新增益",
        Node::ScalarParameter {
            name: "new gain".into(),
            default: 0.875,
        },
    ));
    source
        .links
        .push(wire_link("gain", "value", "output", "roughness"));
    source
        .links
        .push(wire_link("gain", "value", "output", "metallic"));
    let bits = [-0.0_f32, 256.0, f32::from_bits(1), -0.0];
    source
        .parameters
        .insert("Orphan / 原始 ".into(), Parameter::Vector(bits));
    let saved = source.to_toml_string().unwrap();
    let before = zircon_plugin_material_editor_editor::compile_material_graph(&source).unwrap();
    let result = connection_result(
        &source,
        &wire_link("新增益", "value", "output", "roughness"),
    );
    let plan = &result["plan"];
    assert_eq!(result["ok"], true);
    assert_eq!(plan["action"]["kind"], "replace");
    assert_eq!(plan["action"]["previous_source_index"], 1);
    assert_eq!(
        plan["action"]["previous_link"],
        serde_json::to_value(&source.links[1]).unwrap()
    );
    assert_eq!(source.to_toml_string().unwrap(), saved);

    let mut local = source.clone();
    let index = plan["action"]["previous_source_index"].as_u64().unwrap() as usize;
    local.links[index] = serde_json::from_value(plan["link"].clone()).unwrap();
    let mut local = MaterialGraphAsset::from_toml_str(&local.to_toml_string().unwrap()).unwrap();
    let compiled = zircon_plugin_material_editor_editor::compile_material_graph(&local).unwrap();
    assert_eq!(compiled.roughness, 0.875);
    assert_eq!(compiled.metallic, before.metallic);
    assert_eq!(local.links[0], source.links[0]);
    assert_eq!(local.links[2], source.links[2]);
    let Some(Parameter::Vector(actual)) = local.parameters.get("Orphan / 原始 ") else {
        panic!("wire edits retain authored orphan overrides");
    };
    for (actual, expected) in actual.iter().zip(bits) {
        assert_eq!(actual.to_bits(), expected.to_bits());
    }
    // Restore the full original edge at its exact occurrence, not a newly
    // inferred valid connection, and retain unrelated link order and raw values.
    local.links[index] = serde_json::from_value(plan["action"]["previous_link"].clone()).unwrap();
    assert_eq!(local.to_toml_string().unwrap(), saved);
    assert_eq!(
        zircon_plugin_material_editor_editor::compile_material_graph(&local)
            .unwrap()
            .roughness,
        before.roughness
    );
}

#[test]
fn material_editor_dist_native_connection_exact_existing_link_is_an_unchanged_candidate() {
    let source = graph(0.25);
    let before = source.to_toml_string().unwrap();
    let result = connection_result(&source, &source.links[0]);
    assert_eq!(result["ok"], true);
    assert_eq!(result["diagnostics"], json!([]));
    assert_eq!(result["plan"]["action"], json!({"kind":"unchanged"}));
    assert_eq!(result["plan"]["changes_source"], false);
    assert_eq!(result["plan"]["source_value_type"], "vector");
    assert!(result["plan"]["action"].get("previous_link").is_none());
    assert_eq!(
        result["graph_uri"],
        serde_json::to_value(&source.uri).unwrap()
    );
    assert_eq!(source.to_toml_string().unwrap(), before);
}

#[test]
fn material_editor_dist_native_connection_rejects_stale_direction_type_cycle_and_duplicate_input() {
    let source = parameter_edit_graph();
    let mut cases = vec![
        (
            source.clone(),
            wire_link("missing", "value", "output", "roughness"),
            "missing source node",
        ),
        (
            source.clone(),
            wire_link("gain", "value", "missing", "roughness"),
            "missing target node",
        ),
        (
            source.clone(),
            wire_link("gain", "Value", "output", "roughness"),
            "no output pin",
        ),
        (
            source.clone(),
            wire_link("output", "value", "gain", "value"),
            "no output pin",
        ),
        (
            source.clone(),
            wire_link("tint", "value", "output", "roughness"),
            "does not produce a scalar",
        ),
        (
            source.clone(),
            wire_link("gain", "value", "output", "normal"),
            "requires a texture sample",
        ),
    ];
    let mut duplicate = source.clone();
    duplicate
        .links
        .push(wire_link("gain", "value", "output", "roughness"));
    duplicate.links.push(duplicate.links[1].clone());
    cases.push((
        duplicate,
        wire_link("gain", "value", "output", "roughness"),
        "multiple links target",
    ));
    let mut ambiguous = source.clone();
    ambiguous.nodes.push(ambiguous.nodes[2].clone());
    cases.push((
        ambiguous,
        wire_link("gain", "value", "output", "roughness"),
        "duplicate node",
    ));
    let mut cycle = source;
    cycle.nodes.push(creation_node("a", Node::Add));
    cycle.nodes.push(creation_node("b", Node::Multiply));
    cycle.links.push(wire_link("b", "value", "a", "a"));
    cases.push((cycle, wire_link("a", "value", "b", "a"), "contains a cycle"));
    for (source, link, reason) in cases {
        let original = serde_json::to_value(&source).unwrap();
        let result = connection_result(&source, &link);
        assert_eq!(result["ok"], false, "{link:?}");
        assert!(result["plan"].is_null(), "{link:?}");
        assert!(
            result["connection_error"]
                .as_str()
                .unwrap()
                .contains(reason),
            "{result}"
        );
        assert_eq!(serde_json::to_value(&source).unwrap(), original);
    }
}

#[test]
fn material_editor_dist_native_connection_preserves_repair_diagnostics_and_incomplete_types() {
    let mut source = parameter_edit_graph();
    source.links.push(wire_link(
        "missing / 原坏边",
        "value",
        "output",
        "roughness",
    ));
    let old = source.links[1].clone();
    let result = connection_result(&source, &wire_link("gain", "value", "output", "roughness"));
    assert_eq!(result["ok"], false);
    assert!(!result["diagnostics"].as_array().unwrap().is_empty());
    assert!(result["connection_error"].is_null());
    assert_eq!(result["plan"]["action"]["kind"], "replace");
    assert_eq!(result["plan"]["action"]["previous_source_index"], 1);
    assert_eq!(
        result["plan"]["action"]["previous_link"],
        serde_json::to_value(&old).unwrap()
    );
    assert_eq!(source.links[1], old);

    let mut incomplete = graph(0.25);
    incomplete.links.clear();
    incomplete.nodes.push(creation_node("math", Node::Add));
    let result = connection_result(
        &incomplete,
        &wire_link("math", "value", "output", "base_color"),
    );
    assert_eq!(result["ok"], false);
    assert!(!result["diagnostics"].as_array().unwrap().is_empty());
    assert!(result["connection_error"].is_null());
    assert_eq!(result["plan"]["source_value_type"], "undetermined");
    assert_eq!(result["plan"]["has_undetermined_types"], true);

    // A different broken source edge cannot be hidden by a valid local pin pair.
    source
        .links
        .push(wire_link("another missing", "value", "output", "metallic"));
    let rejected = connection_result(&source, &wire_link("gain", "value", "output", "roughness"));
    assert!(rejected["plan"].is_null());
    assert!(!rejected["diagnostics"].as_array().unwrap().is_empty());
}

#[test]
fn material_editor_dist_native_connection_rejects_non_named_duplicate_unknown_and_overflow_arguments(
) {
    let source = parameter_edit_graph();
    let link = wire_link("gain", "value", "output", "roughness");
    let valid: Value = serde_json::from_slice(&connection_request(&source, &link)).unwrap();
    let wire = serde_json::to_string(&valid).unwrap();
    let mut extra = valid.clone();
    extra["revision_permit"] = json!(true);
    let mut null_pin = valid.clone();
    null_pin["to_pin"] = Value::Null;
    let mut overflow = valid.clone();
    overflow["graph"]["nodes"][0]["kind"]["default"][0] = json!(1.0e100_f64);
    let mut missing = valid.clone();
    missing.as_object_mut().unwrap().remove("from_node");
    for bad in [
        "{}".to_owned(),
        "null".to_owned(),
        "[]".to_owned(),
        format!("[{wire}]"),
        wire.replacen("\"from_pin\":", "\"from_pin\":\"value\",\"from_pin\":", 1),
        wire.replacen(
            "\"graph\":",
            &format!(
                "\"graph\":{},\"graph\":",
                serde_json::to_string(&source).unwrap()
            ),
            1,
        ),
        serde_json::to_string(&extra).unwrap(),
        serde_json::to_string(&null_pin).unwrap(),
        serde_json::to_string(&overflow).unwrap(),
        serde_json::to_string(&missing).unwrap(),
    ] {
        let (status, capture) = invoke(
            10,
            bad.as_bytes(),
            MAX_OUTPUT_BYTES,
            ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        );
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, "{bad}");
        assert_eq!(capture.calls, 0);
        assert!(capture.bytes.is_empty());
    }
}

#[test]
fn material_editor_dist_native_connection_publishes_once_at_exact_budget_and_preserves_host_refusal(
) {
    let source = parameter_edit_graph();
    let payload = connection_request(&source, &wire_link("gain", "value", "output", "roughness"));
    let (status, full) = invoke(
        10,
        &payload,
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    assert_eq!(full.calls, 1);
    for limit in [0, full.bytes.len() - 1] {
        let (status, capture) = invoke(10, &payload, limit, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
        assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
        assert_eq!(capture.calls, 0);
        assert!(capture.bytes.is_empty());
    }
    let (status, exact) = invoke(
        10,
        &payload,
        full.bytes.len(),
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    assert_eq!(exact.calls, 1);
    assert_eq!(exact.bytes, full.bytes);
    let (status, refused) = invoke(
        10,
        &payload,
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_DENIED,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_DENIED);
    assert_eq!(refused.calls, 1);
    assert!(refused.bytes.is_empty());
}

fn wire_link(
    from_node: &str,
    from_pin: &str,
    to_node: &str,
    to_pin: &str,
) -> MaterialGraphLinkAsset {
    MaterialGraphLinkAsset {
        from_node: from_node.into(),
        from_pin: from_pin.into(),
        to_node: to_node.into(),
        to_pin: to_pin.into(),
    }
}

fn connection_request(graph: &MaterialGraphAsset, link: &MaterialGraphLinkAsset) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "graph":graph, "from_node":link.from_node, "from_pin":link.from_pin,
        "to_node":link.to_node, "to_pin":link.to_pin,
    }))
    .unwrap()
}

fn connection_result(graph: &MaterialGraphAsset, link: &MaterialGraphLinkAsset) -> Value {
    let payload = connection_request(graph, link);
    let before = payload.clone();
    let (status, capture) = invoke(
        10,
        &payload,
        MAX_OUTPUT_BYTES,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    );
    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    assert_eq!(capture.calls, 1);
    assert_eq!(payload, before);
    serde_json::from_slice(&capture.bytes).unwrap()
}
