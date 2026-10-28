//! Native command transport for the existing material graph validator and compiler.
//! The host retains document/revision authority and decides whether a result may be applied.

use std::io::{self, Write};

use serde_json::{json, Value};
use zircon_plugin_material_editor_editor::{
    can_connect_material_graph, compile_material_graph_with_diagnostics,
    compile_material_graph_with_shader_contract, plan_material_document_property_edit,
    plan_material_document_property_reset, plan_material_graph_node_deletion,
    plan_material_graph_node_search_creation, plan_material_graph_parameter_edit,
    plan_material_graph_parameter_vector_edit, project_authored_material_document,
    project_material_graph_details_selection, validate_material_graph_with_diagnostics,
    MaterialDocumentProjectionError, MaterialDocumentPropertyEdit,
    MaterialDocumentPropertyEditError, MaterialDocumentPropertyResetError,
    MaterialGraphConnectionAction, MaterialGraphDetailsTarget, MaterialGraphDetailsValue,
    MaterialGraphDiagnostic, MaterialGraphDiagnosticTarget, MaterialGraphExpressionDetailsValues,
    MaterialGraphNodeSearch, MaterialGraphParameterEdit, MaterialGraphPresentation,
    MaterialGraphShaderCompileError, MaterialGraphValueType,
};
use zircon_plugin_sdk::native::{
    self, callback_status, NativePluginByteSliceV3, NativePluginCallbackStatusV3,
    NativePluginOutputSinkV4, ZIRCON_NATIVE_PLUGIN_STATUS_DENIED,
    ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, ZIRCON_NATIVE_PLUGIN_STATUS_OK,
};
use zircon_runtime::asset::{
    AssetKind, AssetReference, MaterialGraphAsset, MaterialGraphLinkAsset, MaterialGraphNodeAsset,
    MaterialGraphNodeKindAsset, MaterialGraphParameterAsset, ShaderAsset, ZMaterialDocument,
};
use zircon_runtime::core::framework::render::RenderMaterialPropertyValue;

pub(super) const MAX_INPUT_BYTES: usize = 1024 * 1024;
pub(super) const MAX_OUTPUT_BYTES: usize = 1024 * 1024;
pub(super) const MANIFEST: &[u8] = concat!(
    r#"schema = "zircon.native.command-manifest/4"
[[commands]]
name = "material_editor.graph.validate"
slot = 0
payload_schema = "material_editor.validate_graph.v1"
max_output_bytes = 1048576
[[commands]]
name = "material_editor.graph.compile"
slot = 1
payload_schema = "material_editor.compile_graph.v1"
max_output_bytes = 1048576
[[commands]]
name = "material_editor.graph.compile_shader_contract"
slot = 2
payload_schema = "material_editor.compile_graph_shader.v1"
max_output_bytes = 1048576
[[commands]]
name = "material_editor.graph.parameters"
slot = 3
payload_schema = "material_editor.graph_parameters.v1"
max_output_bytes = 1048576
[[commands]]
name = "material_editor.material.inspect"
slot = 4
payload_schema = "material_editor.inspect_material.v1"
max_output_bytes = 1048576
[[commands]]
name = "material_editor.graph.details"
slot = 5
payload_schema = "material_editor.graph_details.v1"
max_output_bytes = 1048576
[[commands]]
name = "material_editor.graph.plan_parameter_edit"
slot = 6
payload_schema = "material_editor.plan_graph_parameter_edit.v1"
max_output_bytes = 1048576
[[commands]]
name = "material_editor.graph.search_nodes"
slot = 7
payload_schema = "material_editor.search_graph_nodes.v1"
max_output_bytes = 1048576
[[commands]]
name = "material_editor.graph.plan_node_creation"
slot = 8
payload_schema = "material_editor.plan_graph_node_creation.v1"
max_output_bytes = 1048576
[[commands]]
name = "material_editor.graph.plan_node_deletion"
slot = 9
payload_schema = "material_editor.plan_graph_node_deletion.v1"
max_output_bytes = 1048576
[[commands]]
name = "material_editor.graph.plan_connection"
slot = 10
payload_schema = "material_editor.plan_graph_connection.v1"
max_output_bytes = 1048576
[[commands]]
name = "material_editor.material.plan_property_edit"
slot = 11
payload_schema = "material_editor.plan_material_property_edit.v1"
max_output_bytes = 1048576
[[commands]]
name = "material_editor.material.plan_property_reset"
slot = 12
payload_schema = "material_editor.plan_material_property_reset.v1"
max_output_bytes = 1048576
"#,
    "\0"
)
.as_bytes();
// The existing native editor loader recognizes this schema and materializes real
// generation-bound NativeEndpoint commands. This replaces the empty V3 editor
// registration projection; package modules/capabilities remain in the original declaration.
pub(super) const REGISTRATION_SCHEMA: &[u8] = b"zircon.editor.contribution-batch/1\0";
pub(super) const REGISTRATION_MANIFEST: &[u8] = concat!(
    r#"{
  "package_id": "material_editor",
  "contributions": [
    {
      "kind": "command",
      "id": "material_editor.graph.validate",
      "schema": "zircon.editor.command/3",
      "localization_bundle_id": "material_editor",
      "label_key": "command.material_editor.graph.validate.label",
      "description_key": "command.material_editor.graph.validate.description",
      "execution_contract": {
        "result_codec": "zircon.editor.command-result.v1",
        "resource_budget": {
          "max_input_bytes": 1048576,
          "max_output_bytes": 1048576,
          "max_execution_time_ms": 5000
        }
      }
    },
    {
      "kind": "command",
      "id": "material_editor.graph.compile",
      "schema": "zircon.editor.command/3",
      "localization_bundle_id": "material_editor",
      "label_key": "command.material_editor.graph.compile.label",
      "description_key": "command.material_editor.graph.compile.description",
      "execution_contract": {
        "result_codec": "zircon.editor.command-result.v1",
        "resource_budget": {
          "max_input_bytes": 1048576,
          "max_output_bytes": 1048576,
          "max_execution_time_ms": 5000
        }
      }
    },
    {
      "kind": "command",
      "id": "material_editor.graph.compile_shader_contract",
      "schema": "zircon.editor.command/3",
      "localization_bundle_id": "material_editor",
      "label_key": "command.material_editor.graph.compile_shader_contract.label",
      "description_key": "command.material_editor.graph.compile_shader_contract.description",
      "execution_contract": {
        "result_codec": "zircon.editor.command-result.v1",
        "resource_budget": {
          "max_input_bytes": 1048576,
          "max_output_bytes": 1048576,
          "max_execution_time_ms": 5000
        }
      }
    },
    {
      "kind": "command",
      "id": "material_editor.graph.parameters",
      "schema": "zircon.editor.command/3",
      "localization_bundle_id": "material_editor",
      "label_key": "command.material_editor.graph.parameters.label",
      "description_key": "command.material_editor.graph.parameters.description",
      "execution_contract": {
        "result_codec": "zircon.editor.command-result.v1",
        "resource_budget": {
          "max_input_bytes": 1048576,
          "max_output_bytes": 1048576,
          "max_execution_time_ms": 5000
        }
      }
    },
    {
      "kind": "command",
      "id": "material_editor.material.inspect",
      "schema": "zircon.editor.command/3",
      "localization_bundle_id": "material_editor",
      "label_key": "command.material_editor.material.inspect.label",
      "description_key": "command.material_editor.material.inspect.description",
      "execution_contract": {
        "result_codec": "zircon.editor.command-result.v1",
        "resource_budget": {
          "max_input_bytes": 1048576,
          "max_output_bytes": 1048576,
          "max_execution_time_ms": 5000
        }
      }
    },
    {
      "kind": "command",
      "id": "material_editor.graph.details",
      "schema": "zircon.editor.command/3",
      "localization_bundle_id": "material_editor",
      "label_key": "command.material_editor.graph.details.label",
      "description_key": "command.material_editor.graph.details.description",
      "execution_contract": {
        "result_codec": "zircon.editor.command-result.v1",
        "resource_budget": {
          "max_input_bytes": 1048576,
          "max_output_bytes": 1048576,
          "max_execution_time_ms": 5000
        }
      }
    },
    {
      "kind": "command",
      "id": "material_editor.graph.plan_parameter_edit",
      "schema": "zircon.editor.command/3",
      "localization_bundle_id": "material_editor",
      "label_key": "command.material_editor.graph.plan_parameter_edit.label",
      "description_key": "command.material_editor.graph.plan_parameter_edit.description",
      "execution_contract": {
        "result_codec": "zircon.editor.command-result.v1",
        "resource_budget": {
          "max_input_bytes": 1048576,
          "max_output_bytes": 1048576,
          "max_execution_time_ms": 5000
        }
      }
    },
    {
      "kind": "command",
      "id": "material_editor.graph.search_nodes",
      "schema": "zircon.editor.command/3",
      "localization_bundle_id": "material_editor",
      "label_key": "command.material_editor.graph.search_nodes.label",
      "description_key": "command.material_editor.graph.search_nodes.description",
      "execution_contract": {
        "result_codec": "zircon.editor.command-result.v1",
        "resource_budget": {
          "max_input_bytes": 1048576,
          "max_output_bytes": 1048576,
          "max_execution_time_ms": 5000
        }
      }
    },
    {
      "kind": "command",
      "id": "material_editor.graph.plan_node_creation",
      "schema": "zircon.editor.command/3",
      "localization_bundle_id": "material_editor",
      "label_key": "command.material_editor.graph.plan_node_creation.label",
      "description_key": "command.material_editor.graph.plan_node_creation.description",
      "execution_contract": {
        "result_codec": "zircon.editor.command-result.v1",
        "resource_budget": {
          "max_input_bytes": 1048576,
          "max_output_bytes": 1048576,
          "max_execution_time_ms": 5000
        }
      }
    },
    {
      "kind": "command",
      "id": "material_editor.graph.plan_node_deletion",
      "schema": "zircon.editor.command/3",
      "localization_bundle_id": "material_editor",
      "label_key": "command.material_editor.graph.plan_node_deletion.label",
      "description_key": "command.material_editor.graph.plan_node_deletion.description",
      "execution_contract": {
        "result_codec": "zircon.editor.command-result.v1",
        "resource_budget": {
          "max_input_bytes": 1048576,
          "max_output_bytes": 1048576,
          "max_execution_time_ms": 5000
        }
      }
    },
    {
      "kind": "command",
      "id": "material_editor.graph.plan_connection",
      "schema": "zircon.editor.command/3",
      "localization_bundle_id": "material_editor",
      "label_key": "command.material_editor.graph.plan_connection.label",
      "description_key": "command.material_editor.graph.plan_connection.description",
      "execution_contract": {
        "result_codec": "zircon.editor.command-result.v1",
        "resource_budget": {
          "max_input_bytes": 1048576,
          "max_output_bytes": 1048576,
          "max_execution_time_ms": 5000
        }
      }
    },
    {
      "kind": "command",
      "id": "material_editor.material.plan_property_edit",
      "schema": "zircon.editor.command/3",
      "localization_bundle_id": "material_editor",
      "label_key": "command.material_editor.material.plan_property_edit.label",
      "description_key": "command.material_editor.material.plan_property_edit.description",
      "execution_contract": {
        "result_codec": "zircon.editor.command-result.v1",
        "resource_budget": {
          "max_input_bytes": 1048576,
          "max_output_bytes": 1048576,
          "max_execution_time_ms": 5000
        }
      }
    },
    {
      "kind": "command",
      "id": "material_editor.material.plan_property_reset",
      "schema": "zircon.editor.command/3",
      "localization_bundle_id": "material_editor",
      "label_key": "command.material_editor.material.plan_property_reset.label",
      "description_key": "command.material_editor.material.plan_property_reset.description",
      "execution_contract": {
        "result_codec": "zircon.editor.command-result.v1",
        "resource_budget": {
          "max_input_bytes": 1048576,
          "max_output_bytes": 1048576,
          "max_execution_time_ms": 5000
        }
      }
    },
    {
      "kind": "localization_bundle",
      "id": "material_editor",
      "schema": "zircon.editor.localization-bundle/1",
      "locales": {
        "en": {
          "command.material_editor.graph.validate.label": "Validate Material Graph",
          "command.material_editor.graph.validate.description": "Validate the submitted material graph",
          "command.material_editor.graph.compile.label": "Compile Material Graph",
          "command.material_editor.graph.compile.description": "Compile the submitted material graph into a MaterialAsset",
          "command.material_editor.graph.compile_shader_contract.label": "Check Compiled Material Shader Contract",
          "command.material_editor.graph.compile_shader_contract.description": "Compile the submitted graph and check the supplied shader contract",
          "command.material_editor.graph.parameters.label": "Inspect Material Graph Parameters",
          "command.material_editor.graph.parameters.description": "Read exact declarations and overrides from the submitted authoring graph",
          "command.material_editor.material.inspect.label": "Inspect Authored Material",
          "command.material_editor.material.inspect.description": "Read local material overrides and optional shader schema without flattening inheritance",
          "command.material_editor.graph.details.label": "Inspect Material Graph Selection",
          "command.material_editor.graph.details.description": "Read original Details targets and common or mixed values for the submitted selection",
          "command.material_editor.graph.plan_parameter_edit.label": "Plan Material Graph Parameter Edit",
          "command.material_editor.graph.plan_parameter_edit.description": "Plan one presence-sensitive parameter override edit without applying it",
          "command.material_editor.graph.search_nodes.label": "Search Material Nodes",
          "command.material_editor.graph.search_nodes.description": "Read exact registered material actions and pin keys for the submitted query",
          "command.material_editor.graph.plan_node_creation.label": "Plan Material Node Creation",
          "command.material_editor.graph.plan_node_creation.description": "Preflight one typed node and optional autowire without changing the authoring graph",
          "command.material_editor.graph.plan_node_deletion.label": "Plan Material Node Deletion",
          "command.material_editor.graph.plan_node_deletion.description": "Preflight original nodes and incident link occurrences without applying a deletion",
          "command.material_editor.graph.plan_connection.label": "Plan Material Connection",
          "command.material_editor.graph.plan_connection.description": "Preflight exact pin keys and one input replacement without changing the authoring graph",
          "command.material_editor.material.plan_property_edit.label": "Plan Authored Material Property Edit",
          "command.material_editor.material.plan_property_edit.description": "Preflight one exact local property Set or ClearOverride without applying it",
          "command.material_editor.material.plan_property_reset.label": "Plan Authored Material Property Reset",
          "command.material_editor.material.plan_property_reset.description": "Plan a reset from the host's observed default while preserving override presence"
        },
        "zh-CN": {
          "command.material_editor.graph.validate.label": "校验材质图",
          "command.material_editor.graph.validate.description": "校验本次请求中的材质图",
          "command.material_editor.graph.compile.label": "编译材质图",
          "command.material_editor.graph.compile.description": "将本次请求中的材质图编译为 MaterialAsset",
          "command.material_editor.graph.compile_shader_contract.label": "校验编译材质的着色器契约",
          "command.material_editor.graph.compile_shader_contract.description": "编译本次材质图并校验传入的着色器契约",
          "command.material_editor.graph.parameters.label": "查看材质图参数",
          "command.material_editor.graph.parameters.description": "读取本次作者材质图的精确参数声明与覆盖值",
          "command.material_editor.material.inspect.label": "查看作者材质",
          "command.material_editor.material.inspect.description": "读取材质局部覆盖与可选着色器结构，保留原继承关系",
          "command.material_editor.graph.details.label": "查看材质图选择详情",
          "command.material_editor.graph.details.description": "读取本次选择的原始详情对象及共同值或混合值",
          "command.material_editor.graph.plan_parameter_edit.label": "规划材质图参数编辑",
          "command.material_editor.graph.plan_parameter_edit.description": "规划一个保留覆盖存在性的参数编辑候选，不提交作者事务",
          "command.material_editor.graph.search_nodes.label": "搜索材质节点",
          "command.material_editor.graph.search_nodes.description": "按原始查询读取已注册材质动作与精确端口键",
          "command.material_editor.graph.plan_node_creation.label": "规划材质节点创建",
          "command.material_editor.graph.plan_node_creation.description": "预检一个类型化节点及可选自动连线，不提交作者材质图",
          "command.material_editor.graph.plan_node_deletion.label": "规划材质节点删除",
          "command.material_editor.graph.plan_node_deletion.description": "预检原始节点及关联连线，不提交删除事务",
          "command.material_editor.graph.plan_connection.label": "规划材质连线",
          "command.material_editor.graph.plan_connection.description": "预检精确端口键及一个输入替换，不提交作者材质图",
          "command.material_editor.material.plan_property_edit.label": "规划作者材质属性编辑",
          "command.material_editor.material.plan_property_edit.description": "预检精确局部属性的设置或清除覆盖，不提交作者事务",
          "command.material_editor.material.plan_property_reset.label": "规划作者材质属性重置",
          "command.material_editor.material.plan_property_reset.description": "使用宿主实际观测的默认值规划重置，保留覆盖存在性"
        }
      }
    }
  ]
}"#,
    "\0"
).as_bytes();

const COMPLETED: &[u8] = b"material command completed; inspect the result status and diagnostics\0";
const UNKNOWN_SLOT: &[u8] = b"material graph command slot is unknown\0";
const INVALID_REQUEST: &[u8] =
    b"material command requires bounded named JSON arguments; graph numbers must fit finite f32\0";
const OUTPUT_FAILED: &[u8] = b"material graph command could not encode its bounded result\0";
const PANICKED: &[u8] = b"material graph command panicked\0";

/// # Safety
/// The payload and sink must satisfy the SDK callback lifetime and pointer contracts.
pub(super) unsafe extern "C" fn invoke_command(
    slot: u32,
    payload: NativePluginByteSliceV3,
    output: NativePluginOutputSinkV4,
) -> NativePluginCallbackStatusV3 {
    native::catch_native_callback_panic(PANICKED, || {
        if slot > 12 {
            return callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_DENIED, UNKNOWN_SLOT);
        }
        if payload.len == 0 || payload.len > MAX_INPUT_BYTES || payload.data.is_null() {
            return callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, INVALID_REQUEST);
        }
        if output.max_output_bytes == 0 || output.write.is_none() {
            return callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, OUTPUT_FAILED);
        }
        // SAFETY: The host supplies one immutable input allocation for this callback;
        // the bounded length and non-null shape were checked before reading it.
        let bytes = unsafe { native::bytes_from_slice(payload) };
        // Derived named structs also accept positional sequences. Keep the wire
        // contract object-only without a Value pass that would discard duplicate keys.
        if bytes
            .iter()
            .copied()
            .find(|byte| !matches!(*byte, b' ' | b'\t' | b'\n' | b'\r'))
            != Some(b'{')
        {
            return callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, INVALID_REQUEST);
        }
        let result = if slot == 12 {
            let Ok(arguments) = serde_json::from_slice::<PropertyResetArguments>(bytes) else {
                return callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, INVALID_REQUEST);
            };
            if arguments.shader.is_some() != arguments.shader_reference.is_some() {
                return callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, INVALID_REQUEST);
            }
            material_property_reset_result(&arguments)
        } else if slot == 11 {
            let Ok(arguments) = serde_json::from_slice::<PropertyEditArguments>(bytes) else {
                return callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, INVALID_REQUEST);
            };
            if arguments.shader.is_some() != arguments.shader_reference.is_some() {
                return callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, INVALID_REQUEST);
            }
            material_property_edit_result(&arguments)
        } else if slot == 10 {
            let Ok(arguments) = serde_json::from_slice::<ConnectionArguments>(bytes) else {
                return callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, INVALID_REQUEST);
            };
            graph_connection_result(&arguments)
        } else if slot == 9 {
            let Ok(arguments) = serde_json::from_slice::<DetailsArguments>(bytes) else {
                return callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, INVALID_REQUEST);
            };
            graph_node_deletion_result(&arguments)
        } else if slot == 8 {
            let Ok(arguments) = serde_json::from_slice::<NodeCreationArguments>(bytes) else {
                return callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, INVALID_REQUEST);
            };
            if arguments.resource_observation.is_some()
                && !matches!(
                    &arguments.node.kind,
                    MaterialGraphNodeKindAsset::TextureSample { .. }
                )
            {
                return callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, INVALID_REQUEST);
            }
            graph_node_creation_result(&arguments)
        } else if slot == 7 {
            let Ok(arguments) = serde_json::from_slice::<NodeSearchArguments>(bytes) else {
                return callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, INVALID_REQUEST);
            };
            graph_node_search_result(&arguments)
        } else if slot == 6 {
            let Ok(arguments) = serde_json::from_slice::<ParameterEditArguments>(bytes) else {
                return callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, INVALID_REQUEST);
            };
            graph_parameter_edit_result(&arguments)
        } else if slot == 5 {
            let Ok(arguments) = serde_json::from_slice::<DetailsArguments>(bytes) else {
                return callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, INVALID_REQUEST);
            };
            graph_details_result(&arguments)
        } else if slot == 4 {
            let Ok(arguments) = serde_json::from_slice::<MaterialArguments>(bytes) else {
                return callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, INVALID_REQUEST);
            };
            // A shader payload and its complete resolved reference form one input.
            // URI equality alone cannot qualify a different shader UUID.
            if arguments.shader.is_some() != arguments.shader_reference.is_some() {
                return callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, INVALID_REQUEST);
            }
            authored_material_result(&arguments)
        } else if slot == 2 {
            let Ok(arguments) = serde_json::from_slice::<ShaderArguments>(bytes) else {
                return callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, INVALID_REQUEST);
            };
            shader_result(
                &arguments.graph,
                &arguments.shader_reference,
                &arguments.shader,
            )
        } else {
            let Some(graph) = decode_graph(bytes) else {
                return callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, INVALID_REQUEST);
            };
            if slot == 3 {
                parameter_result(&graph)
            } else {
                graph_result(&graph, slot)
            }
        };
        let mut encoded = BoundedOutput {
            bytes: Vec::new(),
            limit: output.max_output_bytes.min(MAX_OUTPUT_BYTES),
        };
        if serde_json::to_writer(&mut encoded, &result).is_err() {
            return callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, OUTPUT_FAILED);
        }
        // Publish once, after full encoding; rejected or truncated encodings publish no prefix.
        // SAFETY: This is the unchanged host sink, used only during its active callback.
        let status = unsafe { output.write(&encoded.bytes) };
        if status.code != ZIRCON_NATIVE_PLUGIN_STATUS_OK {
            return status;
        }
        callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_OK, COMPLETED)
    })
}

/// Projects the original authored declarations, not compiled visibility or an instance ABI.
/// The host must retain this request's document/revision and native generation;
/// returned node IDs/keys do not grant selection, editing or history authority.
fn parameter_result(graph: &MaterialGraphAsset) -> Value {
    let (diagnostics, parameters) = match MaterialGraphPresentation::new(graph) {
        Err(diagnostics) => (diagnostics, Vec::new()),
        Ok(presentation) => (
            validate_material_graph_with_diagnostics(graph),
            presentation
                .parameter_declarations()
                .map(|parameter| {
                    json!({
                        "node_id": parameter.node().id,
                        "key": parameter.name(),
                        "default": parameter.default_value(),
                        "override_present": parameter.override_value().is_some(),
                        "override_value": parameter.override_value(),
                        "effective_value": parameter.effective_value(),
                    })
                })
                .collect(),
        ),
    };
    // Invalid graphs still expose unambiguous declarations for repair; the
    // original diagnostics remain visible and ok never implies renderer readiness.
    json!({
        "graph_uri": graph.uri,
        "ok": diagnostics.is_empty(),
        "diagnostics": diagnostics.iter().map(diagnostic_json).collect::<Vec<_>>(),
        "parameters": parameters,
    })
}

fn graph_result(graph: &MaterialGraphAsset, slot: u32) -> Value {
    let (diagnostics, material) = if slot == 0 {
        (validate_material_graph_with_diagnostics(graph), None)
    } else {
        match compile_material_graph_with_diagnostics(graph) {
            Ok(material) => (Vec::new(), Some(material)),
            Err(diagnostics) => (diagnostics, None),
        }
    };
    // Native OK denotes a completed request, including a graph with compile errors.
    // Only result.ok with an empty diagnostic list qualifies a successful compile.
    json!({
        "graph_uri": graph.uri,
        "ok": diagnostics.is_empty(),
        "diagnostics": diagnostics.iter().map(diagnostic_json).collect::<Vec<_>>(),
        "material": material,
    })
}

/// Checks the original full reference before using the supplied shader payload.
/// The host still owns ResourceManager resolution, source generation and result admission.
fn shader_result(
    graph: &MaterialGraphAsset,
    shader_reference: &AssetReference,
    shader: &ShaderAsset,
) -> Value {
    let (diagnostics, shader_error, material) = if graph.shader.as_ref() != Some(shader_reference) {
        (
            Vec::new(),
            json!({
                "kind": "reference_mismatch",
                "expected": graph.shader,
                "actual": shader_reference,
            }),
            None,
        )
    } else {
        match compile_material_graph_with_shader_contract(graph, shader) {
            Ok(material) => (Vec::new(), Value::Null, Some(material)),
            Err(MaterialGraphShaderCompileError::Graph(diagnostics)) => (
                diagnostics.iter().map(diagnostic_json).collect(),
                Value::Null,
                None,
            ),
            Err(MaterialGraphShaderCompileError::ShaderLocatorMismatch { expected, actual }) => (
                Vec::new(),
                json!({"kind":"locator_mismatch", "expected":expected, "actual":actual}),
                None,
            ),
            Err(MaterialGraphShaderCompileError::ShaderImportDiagnostics(diagnostics)) => (
                Vec::new(),
                json!({"kind":"import_diagnostics", "diagnostics":diagnostics}),
                None,
            ),
            Err(MaterialGraphShaderCompileError::ShaderContract(diagnostics)) => (
                Vec::new(),
                // Serialize the existing Runtime error DTOs, including their real source/path.
                json!({"kind":"contract", "diagnostics":diagnostics}),
                None,
            ),
        }
    };
    json!({
        "graph_uri": graph.uri,
        "shader_reference": shader_reference,
        "ok": material.is_some(),
        "diagnostics": diagnostics,
        "shader_error": shader_error,
        "material": material,
    })
}

/// Reads the original author document rather than a flattened runtime material.
/// Rows do not establish parent resolution, shader/GPU readiness or write authority.
fn authored_material_result(arguments: &MaterialArguments) -> Value {
    let document = &arguments.document;
    let mut error = Value::Null;
    let projection = if document.version != 2 {
        error = json!({
            "kind": "unsupported_document_version", "expected": 2, "actual": document.version,
        });
        None
    } else if arguments
        .shader_reference
        .as_ref()
        .is_some_and(|reference| reference != &document.shader)
    {
        error = json!({
            "kind": "reference_mismatch", "expected": document.shader,
            "actual": arguments.shader_reference,
        });
        None
    } else {
        match project_authored_material_document(document, arguments.shader.as_ref()) {
            Ok(projection) => Some(projection),
            Err(reason) => {
                error = match reason {
                    MaterialDocumentProjectionError::ShaderLocatorMismatch { expected, actual } => {
                        json!({"kind": "locator_mismatch", "expected": expected, "actual": actual})
                    }
                    MaterialDocumentProjectionError::DuplicatePropertyName(name) => {
                        json!({"kind": "duplicate_property", "name": name})
                    }
                    MaterialDocumentProjectionError::DuplicateTextureSlotName(name) => {
                        json!({"kind": "duplicate_texture_slot", "name": name})
                    }
                };
                None
            }
        }
    };
    // No prefix of an ambiguous schema is published. A null projection leaves
    // the host's original document/diagnostics intact; it is not an empty panel.
    let ok = projection
        .as_ref()
        .is_some_and(|projection| projection.diagnostics.is_empty());
    let schema_available = projection.is_some() && arguments.shader.is_some();
    let projection = projection.map(|projection| {
        json!({
            "properties": projection.properties.iter().map(|row| json!({
                "name": row.name, "kind": row.kind, "group": row.group, "label": row.label,
                "default_value": row.default_value, "override_value": row.override_value,
                "is_overridden": row.is_overridden,
            })).collect::<Vec<_>>(),
            "texture_slots": projection.texture_slots.iter().map(|row| json!({
                "name": row.name, "kind": row.kind, "group": row.group, "label": row.label,
                "default_fallback": row.default_fallback, "reference": row.reference,
                "fallback": row.fallback, "is_overridden": row.is_overridden,
                "local_value": document.textures.get(&row.name),
            })).collect::<Vec<_>>(),
            "diagnostics": projection.diagnostics.iter().map(|row| json!({
                "source": row.source, "path": row.path, "message": row.message,
            })).collect::<Vec<_>>(),
        })
    });
    json!({
        "document_version": document.version, "material_name": document.name,
        "shader_reference": document.shader, "parent_reference": document.parent,
        "options": document.options, "queue": document.queue,
        "validation_diagnostics": document.validation_diagnostics,
        "ok": ok, "schema_available": schema_available,
        "projection_error": error, "projection": projection,
    })
}

// These commands adapt the original borrow-only property planners. The host
// retains the source document, raw prior entry and coherent revision/handles;
// a serialized candidate supplies neither an inverse nor a write permit.
fn material_property_edit_result(arguments: &PropertyEditArguments) -> Value {
    let document = &arguments.document;
    let edit = match &arguments.edit {
        PropertyEditRequest::Set { value } => MaterialDocumentPropertyEdit::Set(value),
        PropertyEditRequest::ClearOverride {} => MaterialDocumentPropertyEdit::ClearOverride,
    };
    let candidate =
        match material_property_source_error(document, arguments.shader_reference.as_ref()) {
            Some(error) => Err(error),
            None => plan_material_document_property_edit(
                document,
                arguments.shader.as_ref(),
                &arguments.name,
                edit,
            )
            .map_err(|error| material_property_edit_error(&error)),
        };
    let (plan, error) = match candidate {
        Ok(plan) => (
            Some(json!({
                "name": plan.name(),
                "was_overridden": plan.was_overridden(),
                "will_be_overridden": plan.will_be_overridden(),
                "edit": material_property_edit_json(plan.edit()),
            })),
            Value::Null,
        ),
        Err(error) => (None, error),
    };
    material_property_response(document, arguments.shader.as_ref(), plan, error)
}

fn material_property_reset_result(arguments: &PropertyResetArguments) -> Value {
    let document = &arguments.document;
    let expected_lookup = document.parent.as_ref().unwrap_or(&document.shader);
    let candidate = match material_property_source_error(document, arguments.shader_reference.as_ref()) {
        Some(error) => Err(error),
        None => plan_material_document_property_reset(
            document,
            arguments.shader.as_ref(),
            &arguments.name,
            |declaration| {
                let Some(observation) = &arguments.default_observation else {
                    return Ok(None);
                };
                if observation.name != declaration.name
                    || &observation.lookup_reference != expected_lookup
                {
                    return Err("default observation does not match the exact property and full parent/shader lookup reference".to_owned());
                }
                Ok(Some(&observation.value))
            },
        )
        .map_err(|error| match error {
            MaterialDocumentPropertyResetError::Edit(error) => material_property_edit_error(&error),
            MaterialDocumentPropertyResetError::Resolve(message) => {
                json!({"kind": "default_observation_mismatch", "message": message})
            }
            MaterialDocumentPropertyResetError::DefaultUnavailable(name) => {
                json!({"kind": "default_unavailable", "name": name})
            }
        }),
    };
    let (plan, error) = match candidate {
        Ok(plan) => (
            Some(json!({
                "name": plan.name(),
                "was_overridden": plan.was_overridden(),
                "will_be_overridden": plan.will_be_overridden(),
                "resolved_default": plan.resolved_default(),
                "edit": plan.property_edit().map(|edit| material_property_edit_json(edit.edit())),
            })),
            Value::Null,
        ),
        Err(error) => (None, error),
    };
    material_property_response(document, arguments.shader.as_ref(), plan, error)
}

fn material_property_source_error(
    document: &ZMaterialDocument,
    shader_reference: Option<&AssetReference>,
) -> Option<Value> {
    if document.version != 2 {
        Some(
            json!({"kind": "unsupported_document_version", "expected": 2, "actual": document.version}),
        )
    } else if shader_reference.is_some_and(|reference| reference != &document.shader) {
        Some(
            json!({"kind": "reference_mismatch", "expected": document.shader, "actual": shader_reference}),
        )
    } else {
        None
    }
}

fn material_property_edit_error(error: &MaterialDocumentPropertyEditError) -> Value {
    match error {
        MaterialDocumentPropertyEditError::Source(error) => {
            json!({"kind": "source", "message": error.to_string()})
        }
        MaterialDocumentPropertyEditError::SchemaUnavailable => {
            json!({"kind": "schema_unavailable"})
        }
        MaterialDocumentPropertyEditError::EmptyPropertyName => {
            json!({"kind": "empty_property_name"})
        }
        MaterialDocumentPropertyEditError::UndeclaredProperty(name) => {
            json!({"kind": "undeclared_property", "name": name})
        }
        MaterialDocumentPropertyEditError::TypeMismatch {
            name,
            expected,
            actual,
        } => {
            json!({"kind": "type_mismatch", "name": name, "expected": expected, "actual": actual})
        }
        MaterialDocumentPropertyEditError::NonFiniteValue(name) => {
            json!({"kind": "non_finite_value", "name": name})
        }
    }
}

fn material_property_edit_json(edit: MaterialDocumentPropertyEdit<'_>) -> Value {
    match edit {
        MaterialDocumentPropertyEdit::Set(value) => json!({"kind": "set", "value": value}),
        MaterialDocumentPropertyEdit::ClearOverride => json!({"kind": "clear_override"}),
    }
}

fn material_property_response(
    document: &ZMaterialDocument,
    shader: Option<&ShaderAsset>,
    plan: Option<Value>,
    error: Value,
) -> Value {
    // A repair candidate may coexist with the original author/shader diagnostics.
    // Do not erase them or advertise a candidate as an applied/compiled material.
    let shader_diagnostics = shader
        .map(|shader| shader.validation_diagnostics.as_slice())
        .unwrap_or(&[]);
    json!({
        "document_version": document.version,
        "material_name": document.name,
        "shader_reference": document.shader,
        "parent_reference": document.parent,
        "validation_diagnostics": document.validation_diagnostics,
        "shader_validation_diagnostics": shader_diagnostics,
        "ok": plan.is_some() && document.validation_diagnostics.is_empty() && shader_diagnostics.is_empty(),
        "property_error": error,
        "plan": plan,
    })
}

/// Adapts the existing source-borrowed Details projection to the native reader.
/// Graph fields and selected nodes are inspection values, never a save document.
/// The host retains source/selection revision, view incarnation, focus and history;
/// no returned target is authority to apply an edit or admit a stale native result.
fn graph_details_result(arguments: &DetailsArguments) -> Value {
    let graph = &arguments.graph;
    let diagnostics = validate_material_graph_with_diagnostics(graph);
    let selected: Vec<_> = arguments
        .selected_node_ids
        .iter()
        .map(String::as_str)
        .collect();
    let (selection_error, projection) = match project_material_graph_details_selection(
        graph, &selected,
    ) {
        Err(errors) => (
            json!({"diagnostics": errors.iter().map(diagnostic_json).collect::<Vec<_>>()}),
            None,
        ),
        Ok(selection) => {
            let targets: Vec<_> = selection
                .targets()
                .iter()
                .map(|target| match target {
                    MaterialGraphDetailsTarget::Graph(source) => json!({
                        "kind": "graph",
                        "fields": {
                            "name": source.name, "shader_reference": source.shader,
                            "parameters": source.parameters,
                        },
                    }),
                    MaterialGraphDetailsTarget::Expression(node) => {
                        json!({"kind": "expression", "node": node})
                    }
                })
                .collect();
            let expression_values = match selection.expression_values() {
                Some(MaterialGraphExpressionDetailsValues::ScalarDefault(value)) => {
                    json!({"kind": "scalar_default", "value": details_value_json(value)})
                }
                Some(MaterialGraphExpressionDetailsValues::VectorDefault(values)) => {
                    json!({"kind": "vector_default", "channels": values.map(details_value_json)})
                }
                Some(MaterialGraphExpressionDetailsValues::Texture(value)) => {
                    json!({"kind": "texture", "value": details_value_json(value)})
                }
                None => Value::Null,
            };
            (
                Value::Null,
                Some(json!({
                    "selected_node_ids": selection.selected_nodes().iter()
                        .map(|node| node.id.as_str()).collect::<Vec<_>>(),
                    "targets": targets,
                    "expression_values": expression_values,
                })),
            )
        }
    };
    // Broken graphs remain inspectable for repair; their original diagnostics
    // are never cleared by a valid selection. A rejected selection publishes no prefix.
    json!({
        "graph_uri": graph.uri,
        "ok": projection.is_some() && diagnostics.is_empty(),
        "diagnostics": diagnostics.iter().map(diagnostic_json).collect::<Vec<_>>(),
        "selection_error": selection_error,
        "projection": projection,
    })
}

/// Transports one existing domain candidate; it never applies a map update.
/// The host must retain the exact submitted source/selection/native generation,
/// then account for the raw Option before/after in one delta/inverse and its
/// original prepare/permit/CAS/history. A Reset inverse is raw restoration, not
/// a newly validated Set; neither diagnostics nor a no-op grant mutation authority.
fn graph_parameter_edit_result(arguments: &ParameterEditArguments) -> Value {
    let graph = &arguments.graph;
    let diagnostics = validate_material_graph_with_diagnostics(graph);
    let candidate = match &arguments.edit {
        ParameterEditRequest::SetScalar { value } => plan_material_graph_parameter_edit(
            graph,
            &arguments.node_id,
            MaterialGraphParameterEdit::Set(MaterialGraphParameterAsset::Scalar(*value)),
        ),
        ParameterEditRequest::SetVector { value } => plan_material_graph_parameter_edit(
            graph,
            &arguments.node_id,
            MaterialGraphParameterEdit::Set(MaterialGraphParameterAsset::Vector(*value)),
        ),
        ParameterEditRequest::Reset {} => plan_material_graph_parameter_edit(
            graph,
            &arguments.node_id,
            MaterialGraphParameterEdit::Reset,
        ),
        ParameterEditRequest::SetVectorComponents { components } => {
            plan_material_graph_parameter_vector_edit(graph, &arguments.node_id, *components)
        }
    };
    let (edit_error, plan) = match candidate {
        Err(errors) => (
            json!({"diagnostics": errors.iter().map(diagnostic_json).collect::<Vec<_>>()}),
            None,
        ),
        Ok(plan) => (
            Value::Null,
            Some(json!({
                "node_id": plan.node().id,
                "key": plan.key(),
                "previous_present": plan.previous_value().is_some(),
                "previous_value": plan.previous_value(),
                "proposed_present": plan.proposed_value().is_some(),
                "proposed_value": plan.proposed_value(),
                "affected_node_ids": plan.affected_nodes().map(|node| node.id.as_str())
                    .collect::<Vec<_>>(),
                "changes_source": plan.changes_source(),
            })),
        ),
    };
    // A repair candidate may exist while unrelated graph errors remain visible.
    // Native OK means completion; this result is neither a compile nor a commit.
    json!({
        "graph_uri": graph.uri,
        "ok": plan.is_some() && diagnostics.is_empty(),
        "diagnostics": diagnostics.iter().map(diagnostic_json).collect::<Vec<_>>(),
        "edit_error": edit_error,
        "plan": plan,
    })
}

/// Delegates type, direction, replacement and cycle checks to the original domain.
/// The borrowed old edge's source index qualifies one occurrence in this submitted
/// revision; it is not a persistent edge ID, history record or authority to apply.
fn graph_connection_result(arguments: &ConnectionArguments) -> Value {
    let graph = &arguments.graph;
    let link = MaterialGraphLinkAsset {
        from_node: arguments.from_node.clone(),
        from_pin: arguments.from_pin.clone(),
        to_node: arguments.to_node.clone(),
        to_pin: arguments.to_pin.clone(),
    };
    let diagnostics = validate_material_graph_with_diagnostics(graph);
    let candidate = can_connect_material_graph(graph, &link).and_then(|plan| {
        let action = match plan.action {
            MaterialGraphConnectionAction::Connect => json!({"kind": "connect"}),
            MaterialGraphConnectionAction::Unchanged => json!({"kind": "unchanged"}),
            MaterialGraphConnectionAction::Replace(previous) => {
                let source_index = graph
                    .links
                    .iter()
                    .position(|source| std::ptr::eq(source, previous))
                    .ok_or_else(|| {
                        "domain replacement is not an original source link".to_owned()
                    })?;
                json!({
                    "kind": "replace",
                    "previous_source_index": source_index,
                    "previous_link": previous,
                })
            }
        };
        let source_value_type = match plan.source_type {
            MaterialGraphValueType::Scalar => "scalar",
            MaterialGraphValueType::Vector => "vector",
            MaterialGraphValueType::Texture => "texture",
            MaterialGraphValueType::Undetermined => "undetermined",
        };
        Ok(json!({
            "link": link,
            "action": action,
            "source_value_type": source_value_type,
            "has_undetermined_types": plan.has_undetermined_types,
            "changes_source": !matches!(plan.action, MaterialGraphConnectionAction::Unchanged),
        }))
    });
    let (connection_error, plan) = match candidate {
        Ok(plan) => (Value::Null, Some(plan)),
        Err(message) => (json!(message), None),
    };
    // Missing inputs are legal while authoring. Keep original diagnostics even
    // for a repair candidate; native completion does not mean compile or commit.
    json!({
        "graph_uri": graph.uri,
        "ok": plan.is_some() && diagnostics.is_empty(),
        "diagnostics": diagnostics.iter().map(diagnostic_json).collect::<Vec<_>>(),
        "connection_error": connection_error,
        "plan": plan,
    })
}

/// Indices describe occurrences in this submitted source revision, not persistent
/// IDs or an undo permit. Parameters and protected Output nodes remain authored.
/// The host qualifies the entire retained selection and prepares one delta/inverse
/// before confirmation, mutation, preview reset, focus and rendering side effects.
fn graph_node_deletion_result(arguments: &DetailsArguments) -> Value {
    let graph = &arguments.graph;
    let selected: Vec<&str> = arguments
        .selected_node_ids
        .iter()
        .map(String::as_str)
        .collect();
    let diagnostics = validate_material_graph_with_diagnostics(graph);
    let (deletion_error, plan) = match plan_material_graph_node_deletion(graph, &selected) {
        Err(errors) => (
            json!({"diagnostics": errors.iter().map(diagnostic_json).collect::<Vec<_>>()}),
            None,
        ),
        Ok(plan) => (
            Value::Null,
            Some(json!({
                "can_delete": plan.can_delete(),
                "removed_nodes": plan.removed_nodes().iter().map(|(source_index, node)| {
                    json!({"source_index": source_index, "node": node})
                }).collect::<Vec<_>>(),
                "removed_links": plan.removed_links().iter().map(|(source_index, link)| {
                    json!({"source_index": source_index, "link": link})
                }).collect::<Vec<_>>(),
                "protected_nodes": plan.protected_nodes(),
            })),
        ),
    };
    // Deletion may repair a broken graph. Existing diagnostics do not disappear,
    // and an empty/root-only no-op must not enter the original document history.
    json!({
        "graph_uri": graph.uri,
        "selected_node_ids": arguments.selected_node_ids,
        "ok": plan.is_some() && diagnostics.is_empty(),
        "diagnostics": diagnostics.iter().map(diagnostic_json).collect::<Vec<_>>(),
        "deletion_error": deletion_error,
        "plan": plan,
    })
}

/// Reads the actual domain palette. `initial_action_id` is its new-menu default,
/// not the current host keyboard selection or permission to open a popup.
fn graph_node_search_result(arguments: &NodeSearchArguments) -> Value {
    let search = MaterialGraphNodeSearch::new(arguments.query.clone());
    json!({
        "query": search.query(),
        "actions": search.matches(),
        "initial_action_id": search.selected().map(|action| action.id()),
    })
}

/// The request transports one host-qualified action and resource-kind observation.
/// It does not query ResourceManager, create IDs, transform coordinates, activate
/// or close the real popup, mutate a graph, or issue a history permit. The host
/// must qualify the original popup/document/catalog/native generations and use
/// one prepared delta/inverse for the node, removed edge and optional new edge.
fn graph_node_creation_result(arguments: &NodeCreationArguments) -> Value {
    let graph = &arguments.graph;
    let search = MaterialGraphNodeSearch::new(arguments.query.clone());
    let diagnostics = validate_material_graph_with_diagnostics(graph);
    let candidate = plan_material_graph_node_search_creation(
        &search,
        graph,
        &arguments.action_id,
        &arguments.node,
        arguments.autowire.as_ref(),
        |reference| match &arguments.resource_observation {
            Some(observed) if &observed.reference == reference => Ok(observed.kind),
            Some(_) => {
                Err("host resource observation does not match the full UUID and locator".to_owned())
            }
            None => Err(
                "texture creation requires the host's actual resource-kind observation".to_owned(),
            ),
        },
    );
    let (creation_error, plan) = match candidate {
        Err(errors) => (
            json!({"diagnostics": errors.iter().map(diagnostic_json).collect::<Vec<_>>()}),
            None,
        ),
        Ok(plan) => {
            let connection = plan.connection().map(|connection| {
                let action = match connection.action {
                    MaterialGraphConnectionAction::Connect => json!({"kind": "connect"}),
                    MaterialGraphConnectionAction::Replace(previous) => {
                        json!({"kind": "replace", "previous_link": previous})
                    }
                    MaterialGraphConnectionAction::Unchanged => json!({"kind": "unchanged"}),
                };
                let value_type = match connection.source_type {
                    MaterialGraphValueType::Scalar => "scalar",
                    MaterialGraphValueType::Vector => "vector",
                    MaterialGraphValueType::Texture => "texture",
                    MaterialGraphValueType::Undetermined => "undetermined",
                };
                json!({
                    "action": action,
                    "source_value_type": value_type,
                    "has_undetermined_types": connection.has_undetermined_types,
                })
            });
            (
                Value::Null,
                Some(json!({
                    "action_id": arguments.action_id,
                    "node": plan.node(),
                    "autowire": plan.link(),
                    "connection": connection,
                })),
            )
        }
    };
    // Incomplete new nodes are legal authoring candidates, not compiled material.
    // Existing source diagnostics stay visible even when a repair plan exists.
    json!({
        "graph_uri": graph.uri,
        "ok": plan.is_some() && diagnostics.is_empty(),
        "diagnostics": diagnostics.iter().map(diagnostic_json).collect::<Vec<_>>(),
        "creation_error": creation_error,
        "plan": plan,
    })
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ConnectionArguments {
    #[serde(deserialize_with = "deserialize_graph")]
    graph: MaterialGraphAsset,
    from_node: String,
    from_pin: String,
    to_node: String,
    to_pin: String,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct NodeSearchArguments {
    query: String,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct NodeCreationArguments {
    #[serde(deserialize_with = "deserialize_graph")]
    graph: MaterialGraphAsset,
    query: String,
    action_id: String,
    #[serde(deserialize_with = "deserialize_node")]
    node: MaterialGraphNodeAsset,
    autowire: Option<MaterialGraphLinkAsset>,
    resource_observation: Option<ResourceKindObservation>,
}

/// A DTO for the original host's observation, not a resource resolver or proof
/// that a matching registry entry is still current. That admission stays at host.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ResourceKindObservation {
    reference: AssetReference,
    kind: AssetKind,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ParameterEditArguments {
    #[serde(deserialize_with = "deserialize_graph")]
    graph: MaterialGraphAsset,
    node_id: String,
    edit: ParameterEditRequest,
}

// A strict command DTO maps fixed-size numeric payloads into the original asset
// enum; it is not another authoring document or parameter store.
#[derive(serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum ParameterEditRequest {
    SetScalar { value: f32 },
    SetVector { value: [f32; 4] },
    Reset {},
    SetVectorComponents { components: [Option<f32>; 4] },
}

fn details_value_json<T: serde::Serialize>(value: MaterialGraphDetailsValue<T>) -> Value {
    match value {
        MaterialGraphDetailsValue::Common(value) => json!({"kind": "common", "value": value}),
        MaterialGraphDetailsValue::Mixed => json!({"kind": "mixed"}),
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct DetailsArguments {
    #[serde(deserialize_with = "deserialize_graph")]
    graph: MaterialGraphAsset,
    selected_node_ids: Vec<String>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct MaterialArguments {
    document: ZMaterialDocument,
    shader_reference: Option<AssetReference>,
    shader: Option<ShaderAsset>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PropertyEditArguments {
    document: ZMaterialDocument,
    shader_reference: Option<AssetReference>,
    shader: Option<ShaderAsset>,
    name: String,
    edit: PropertyEditRequest,
}

#[derive(serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum PropertyEditRequest {
    Set {
        #[serde(deserialize_with = "deserialize_property_value")]
        value: RenderMaterialPropertyValue,
    },
    ClearOverride {},
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PropertyResetArguments {
    document: ZMaterialDocument,
    shader_reference: Option<AssetReference>,
    shader: Option<ShaderAsset>,
    name: String,
    default_observation: Option<PropertyDefaultObservation>,
}

/// The original host must supply its actual coherent parent/default lookup.
/// Equality here checks the wire identity, not registry readiness or a lease.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PropertyDefaultObservation {
    name: String,
    lookup_reference: AssetReference,
    #[serde(deserialize_with = "deserialize_property_value")]
    value: RenderMaterialPropertyValue,
}

// Keep the existing Runtime value enum; this command-only decoder additionally
// rejects unknown and duplicate members instead of dropping them through Value.
fn deserialize_property_value<'de, D>(
    deserializer: D,
) -> Result<RenderMaterialPropertyValue, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(serde::Deserialize)]
    #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
    enum WireValue {
        Bool { value: bool },
        Float { value: f32 },
        Int { value: i32 },
        UInt { value: u32 },
        String { value: String },
        Vec2 { value: [f32; 2] },
        Vec3 { value: [f32; 3] },
        Vec4 { value: [f32; 4] },
    }
    Ok(
        match <WireValue as serde::Deserialize<'de>>::deserialize(deserializer)? {
            WireValue::Bool { value } => RenderMaterialPropertyValue::Bool { value },
            WireValue::Float { value } => RenderMaterialPropertyValue::Float { value },
            WireValue::Int { value } => RenderMaterialPropertyValue::Int { value },
            WireValue::UInt { value } => RenderMaterialPropertyValue::UInt { value },
            WireValue::String { value } => RenderMaterialPropertyValue::String { value },
            WireValue::Vec2 { value } => RenderMaterialPropertyValue::Vec2 { value },
            WireValue::Vec3 { value } => RenderMaterialPropertyValue::Vec3 { value },
            WireValue::Vec4 { value } => RenderMaterialPropertyValue::Vec4 { value },
        },
    )
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ShaderArguments {
    #[serde(deserialize_with = "deserialize_graph")]
    graph: MaterialGraphAsset,
    shader_reference: AssetReference,
    shader: ShaderAsset,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Arguments {
    #[serde(deserialize_with = "deserialize_graph")]
    graph: MaterialGraphAsset,
}

// JSON cannot carry a non-finite f32 without losing it as null on output.
// Serde's f64-to-f32 conversion may overflow an otherwise finite JSON number.
// Reject that wire input before projection/compile; do not clamp author values or
// change the domain's ability to inspect and repair legacy non-finite documents.
fn deserialize_graph<'de, D>(deserializer: D) -> Result<MaterialGraphAsset, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let graph = <MaterialGraphAsset as serde::Deserialize<'de>>::deserialize(deserializer)?;
    let finite_nodes = graph.nodes.iter().all(node_numbers_finite);
    let finite_overrides = graph.parameters.values().all(|value| match value {
        MaterialGraphParameterAsset::Scalar(value) => value.is_finite(),
        MaterialGraphParameterAsset::Vector(values) => values.iter().all(|value| value.is_finite()),
    });
    if !finite_nodes || !finite_overrides {
        return Err(serde::de::Error::custom(
            "material graph JSON numbers must remain finite after f32 conversion",
        ));
    }
    Ok(graph)
}

fn deserialize_node<'de, D>(deserializer: D) -> Result<MaterialGraphNodeAsset, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let node = <MaterialGraphNodeAsset as serde::Deserialize<'de>>::deserialize(deserializer)?;
    if !node_numbers_finite(&node) {
        return Err(serde::de::Error::custom(
            "material graph node JSON numbers must remain finite after f32 conversion",
        ));
    }
    Ok(node)
}

fn node_numbers_finite(node: &MaterialGraphNodeAsset) -> bool {
    node.position.iter().all(|value| value.is_finite())
        && match &node.kind {
            MaterialGraphNodeKindAsset::ScalarParameter { default, .. } => default.is_finite(),
            MaterialGraphNodeKindAsset::VectorParameter { default, .. } => {
                default.iter().all(|value| value.is_finite())
            }
            MaterialGraphNodeKindAsset::Output
            | MaterialGraphNodeKindAsset::TextureSample { .. }
            | MaterialGraphNodeKindAsset::Add
            | MaterialGraphNodeKindAsset::Multiply => true,
        }
}

fn decode_graph(bytes: &[u8]) -> Option<MaterialGraphAsset> {
    // The derived struct visitor rejects duplicate graph fields as well as extras;
    // decoding through Value first would silently keep only the last duplicate.
    serde_json::from_slice::<Arguments>(bytes)
        .ok()
        .map(|arguments| arguments.graph)
}

fn diagnostic_json(diagnostic: &MaterialGraphDiagnostic) -> Value {
    let target = match &diagnostic.target {
        MaterialGraphDiagnosticTarget::Graph => json!({"kind": "graph"}),
        MaterialGraphDiagnosticTarget::Node { node_id } => {
            json!({"kind": "node", "node_id": node_id})
        }
        MaterialGraphDiagnosticTarget::Input { node_id, pin_name } => {
            json!({"kind": "input", "node_id": node_id, "pin_name": pin_name})
        }
        MaterialGraphDiagnosticTarget::Output { node_id, pin_name } => {
            json!({"kind": "output", "node_id": node_id, "pin_name": pin_name})
        }
        MaterialGraphDiagnosticTarget::Parameter { name } => {
            json!({"kind": "parameter", "name": name})
        }
    };
    json!({"message": diagnostic.message, "target": target})
}

struct BoundedOutput {
    bytes: Vec<u8>,
    limit: usize,
}

impl Write for BoundedOutput {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let end = self
            .bytes
            .len()
            .checked_add(bytes.len())
            .ok_or_else(|| io::Error::other("material graph command result length overflow"))?;
        if end > self.limit {
            return Err(io::Error::other(
                "material graph command result exceeds output budget",
            ));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/commands.rs"]
mod tests;
