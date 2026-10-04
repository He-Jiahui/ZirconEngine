//! 作者入口点在此转换为受支持的渲染阶段描述；导入就绪检查应先拒绝未知阶段，避免编译阶段才出现不明确错误。

use serde::{Deserialize, Serialize};

use crate::core::framework::render::{RenderShaderEntryPointDescriptor, RenderShaderStage};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderEntryPointAsset {
    pub name: String,
    pub stage: String,
}

impl ShaderEntryPointAsset {
    pub fn descriptor(&self) -> Option<RenderShaderEntryPointDescriptor> {
        let stage = parse_stage(&self.stage)?;
        Some(RenderShaderEntryPointDescriptor {
            name: self.name.clone(),
            stage,
        })
    }
}

fn parse_stage(stage: &str) -> Option<RenderShaderStage> {
    let stage = stage.trim();
    if stage.eq_ignore_ascii_case("vertex")
        || stage.eq_ignore_ascii_case("vert")
        || stage.eq_ignore_ascii_case("vs")
    {
        Some(RenderShaderStage::Vertex)
    } else if stage.eq_ignore_ascii_case("fragment")
        || stage.eq_ignore_ascii_case("frag")
        || stage.eq_ignore_ascii_case("fs")
    {
        Some(RenderShaderStage::Fragment)
    } else if stage.eq_ignore_ascii_case("compute")
        || stage.eq_ignore_ascii_case("comp")
        || stage.eq_ignore_ascii_case("cs")
    {
        Some(RenderShaderStage::Compute)
    } else {
        None
    }
}

#[cfg(test)]
#[path = "tests/entry_point_optimization_batch_gx_runtime579_tests.rs"]
mod optimization_batch_gx_runtime579_tests;
