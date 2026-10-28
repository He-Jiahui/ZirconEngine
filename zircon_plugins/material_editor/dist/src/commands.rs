//! Native command transport for the existing material graph validator and compiler.
//! The host retains document/revision authority and decides whether a result may be applied.

use std::io::{self, Write};

use serde_json::{json, Value};
use zircon_plugin_material_editor_editor::{
    compile_material_graph_with_diagnostics, compile_material_graph_with_shader_contract,
    project_authored_material_document, validate_material_graph_with_diagnostics,
    MaterialDocumentProjectionError, MaterialGraphDiagnostic, MaterialGraphDiagnosticTarget,
    MaterialGraphPresentation, MaterialGraphShaderCompileError,
};
use zircon_plugin_sdk::native::{
    self, callback_status, NativePluginByteSliceV3, NativePluginCallbackStatusV3,
    NativePluginOutputSinkV4, ZIRCON_NATIVE_PLUGIN_STATUS_DENIED,
    ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, ZIRCON_NATIVE_PLUGIN_STATUS_OK,
};
use zircon_runtime::asset::{AssetReference, MaterialGraphAsset, ShaderAsset, ZMaterialDocument};

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
          "command.material_editor.material.inspect.description": "Read local material overrides and optional shader schema without flattening inheritance"
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
          "command.material_editor.material.inspect.description": "读取材质局部覆盖与可选着色器结构，保留原继承关系"
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
    b"material command requires one bounded JSON material command argument\0";
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
        if slot > 4 {
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
        let result = if slot == 4 {
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

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct MaterialArguments {
    document: ZMaterialDocument,
    shader_reference: Option<AssetReference>,
    shader: Option<ShaderAsset>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ShaderArguments {
    graph: MaterialGraphAsset,
    shader_reference: AssetReference,
    shader: ShaderAsset,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Arguments {
    graph: MaterialGraphAsset,
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
