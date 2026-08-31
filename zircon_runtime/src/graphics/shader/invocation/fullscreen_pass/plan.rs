use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::super::compiler::{ShaderNamedResourceBinding, ShaderParameterValue};
use super::parameter_encoding::fullscreen_parameter_words;
use super::pipeline_cache_key::FullscreenPipelineCacheKey;
use super::shader_ref::FullscreenShaderRef;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FullscreenPassPlan {
    pub shader: FullscreenShaderRef,
    pub vertex_entry: String,
    pub parameters: BTreeMap<String, ShaderParameterValue>,
    pub resources: Vec<ShaderNamedResourceBinding>,
    pub pipeline_key: FullscreenPipelineCacheKey,
    pub pipeline_label: String,
}

impl FullscreenPassPlan {
    pub fn resource_binding(&self, name: &str) -> Option<&ShaderNamedResourceBinding> {
        self.resources.iter().find(|resource| resource.name == name)
    }

    pub fn parameter_slot(&self, name: &str) -> Option<u32> {
        self.parameters
            .keys()
            .position(|parameter_name| parameter_name == name)
            .and_then(|slot| u32::try_from(slot).ok())
    }

    pub fn parameter_byte_len(&self) -> u64 {
        u64::try_from(self.parameters.len())
            .unwrap_or(u64::MAX / 16)
            .saturating_mul(16)
    }

    pub fn parameter_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.parameters.len().saturating_mul(16));
        self.write_parameter_bytes(&mut bytes);
        bytes
    }

    pub(crate) fn write_parameter_bytes(&self, bytes: &mut Vec<u8>) {
        let byte_len = self.parameters.len().saturating_mul(16);
        bytes.clear();
        if bytes.capacity() < byte_len {
            bytes.reserve_exact(byte_len);
        }
        for value in self.parameters.values() {
            for word in fullscreen_parameter_words(value) {
                // WebGPU uniform bytes have a fixed little-endian representation; do not
                // make the shader ABI depend on the host CPU byte order.
                bytes.extend_from_slice(&word.to_le_bytes());
            }
        }
    }
}
