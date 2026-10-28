//! Native command transport for the existing material graph validator and compiler.
//! The host retains document/revision authority and decides whether a result may be applied.

use std::io::{self, Write};

use serde_json::{json, Value};
use zircon_plugin_material_editor_editor::{
    compile_material_graph_with_diagnostics, validate_material_graph_with_diagnostics,
    MaterialGraphDiagnostic, MaterialGraphDiagnosticTarget,
};
use zircon_plugin_sdk::native::{
    self, callback_status, NativePluginByteSliceV3, NativePluginCallbackStatusV3,
    NativePluginOutputSinkV4, ZIRCON_NATIVE_PLUGIN_STATUS_DENIED,
    ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, ZIRCON_NATIVE_PLUGIN_STATUS_OK,
};
use zircon_runtime::asset::MaterialGraphAsset;

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
      "kind": "localization_bundle",
      "id": "material_editor",
      "schema": "zircon.editor.localization-bundle/1",
      "locales": {
        "en": {
          "command.material_editor.graph.validate.label": "Validate Material Graph",
          "command.material_editor.graph.validate.description": "Validate the submitted material graph",
          "command.material_editor.graph.compile.label": "Compile Material Graph",
          "command.material_editor.graph.compile.description": "Compile the submitted material graph into a MaterialAsset"
        },
        "zh-CN": {
          "command.material_editor.graph.validate.label": "校验材质图",
          "command.material_editor.graph.validate.description": "校验本次请求中的材质图",
          "command.material_editor.graph.compile.label": "编译材质图",
          "command.material_editor.graph.compile.description": "将本次请求中的材质图编译为 MaterialAsset"
        }
      }
    }
  ]
}"#,
    "\0"
).as_bytes();

const COMPLETED: &[u8] = b"material graph command completed; inspect result ok and diagnostics\0";
const UNKNOWN_SLOT: &[u8] = b"material graph command slot is unknown\0";
const INVALID_REQUEST: &[u8] = b"material graph command requires one bounded JSON graph argument\0";
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
        if slot > 1 {
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
        let Some(graph) = decode_graph(bytes) else {
            return callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, INVALID_REQUEST);
        };
        let (diagnostics, material) = if slot == 0 {
            (validate_material_graph_with_diagnostics(&graph), None)
        } else {
            match compile_material_graph_with_diagnostics(&graph) {
                Ok(material) => (Vec::new(), Some(material)),
                Err(diagnostics) => (diagnostics, None),
            }
        };
        // Native OK denotes a completed request, including a graph with compile errors.
        // Only result.ok with an empty diagnostic list qualifies a successful compile.
        let result = json!({
            "graph_uri": graph.uri,
            "ok": diagnostics.is_empty(),
            "diagnostics": diagnostics.iter().map(diagnostic_json).collect::<Vec<_>>(),
            "material": material,
        });
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
