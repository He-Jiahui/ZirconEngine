//! Editor-side ONNX conversion and structured diagnostics for neural model assets.

mod capability;
mod plugin;

pub use zircon_plugin_neural_runtime::{NnModelAsset, NnOpCode};

pub use capability::{
    EDITOR_CAPABILITIES, EDITOR_CRATE_NAME, NEURAL_AUTHORING_CAPABILITY, PLUGIN_ID,
};
pub use plugin::{
    editor_capabilities, editor_plugin, editor_plugin_declaration, editor_plugin_descriptor,
    package_manifest, plugin_registration, NeuralEditorPlugin,
};

pub mod onnx;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NnConversionDiagnostic {
    pub node: String,
    pub op_type: String,
    pub reason: String,
    pub input_shapes: Vec<Vec<u32>>,
}

impl NnConversionDiagnostic {
    pub fn unsupported(node: impl Into<String>, op_type: impl Into<String>) -> Self {
        let op_type = op_type.into();
        Self {
            node: node.into(),
            reason: format!("ONNX operator {op_type} is not supported by the V1 neural runtime"),
            op_type,
            input_shapes: Vec::new(),
        }
    }

    pub fn to_json_line(&self) -> String {
        format!(
            "{{\"node\":\"{}\",\"op_type\":\"{}\",\"reason\":\"{}\",\"input_shapes\":[{}]}}",
            json_escape(&self.node),
            json_escape(&self.op_type),
            json_escape(&self.reason),
            self.input_shapes
                .iter()
                .map(|shape| {
                    format!(
                        "[{}]",
                        shape
                            .iter()
                            .map(u32::to_string)
                            .collect::<Vec<_>>()
                            .join(",")
                    )
                })
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests;
