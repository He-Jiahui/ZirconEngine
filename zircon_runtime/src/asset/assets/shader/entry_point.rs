use serde::{Deserialize, Serialize};

use crate::core::framework::render::{RenderShaderEntryPointDescriptor, RenderShaderStage};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderEntryPointAsset {
    pub name: String,
    pub stage: String,
}

impl ShaderEntryPointAsset {
    pub fn descriptor(&self) -> Option<RenderShaderEntryPointDescriptor> {
        Some(RenderShaderEntryPointDescriptor {
            name: self.name.clone(),
            stage: parse_stage(&self.stage)?,
        })
    }
}

fn parse_stage(stage: &str) -> Option<RenderShaderStage> {
    match stage.trim().to_ascii_lowercase().as_str() {
        "vertex" | "vert" | "vs" => Some(RenderShaderStage::Vertex),
        "fragment" | "frag" | "fs" => Some(RenderShaderStage::Fragment),
        "compute" | "comp" | "cs" => Some(RenderShaderStage::Compute),
        _ => None,
    }
}
