//! 计算着色器来源统一为图管线标签与源引用；素材 locator 改变时，调用方应按新的管线身份处理。
use zircon_runtime_interface::resource::{AssetReference, ResourceScheme};

use crate::render_graph::RenderGraphComputeShaderSource;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComputeShaderSource {
    BuiltinWgsl {
        label: &'static str,
        source: &'static str,
    },
    Asset {
        asset: AssetReference,
    },
    InlineWgsl {
        label: String,
        source: String,
    },
}

impl ComputeShaderSource {
    pub const fn builtin_wgsl(label: &'static str, source: &'static str) -> Self {
        Self::BuiltinWgsl { label, source }
    }

    pub fn asset(asset: AssetReference) -> Self {
        Self::Asset { asset }
    }

    pub fn inline_wgsl(label: impl Into<String>, source: impl Into<String>) -> Self {
        Self::InlineWgsl {
            label: label.into(),
            source: source.into(),
        }
    }

    pub(crate) fn pipeline_label(&self) -> String {
        match self {
            Self::BuiltinWgsl { label, .. } => (*label).to_string(),
            Self::Asset { asset } => asset_pipeline_label(asset),
            Self::InlineWgsl { label, .. } => label.clone(),
        }
    }

    pub(crate) fn graph_source(&self) -> RenderGraphComputeShaderSource {
        match self {
            Self::BuiltinWgsl { label, source } => {
                RenderGraphComputeShaderSource::wgsl(*label, *source)
            }
            Self::Asset { asset } => RenderGraphComputeShaderSource::asset(asset.clone()),
            Self::InlineWgsl { label, source } => {
                RenderGraphComputeShaderSource::wgsl(label.clone(), source.clone())
            }
        }
    }
}

fn asset_pipeline_label(asset: &AssetReference) -> String {
    const PREFIX: &str = "compute.asset:";
    let locator = &asset.locator;
    let scheme = match locator.scheme() {
        ResourceScheme::Res => "res",
        ResourceScheme::Library => "lib",
        ResourceScheme::Package => "package",
        ResourceScheme::Builtin => "builtin",
        ResourceScheme::Memory => "mem",
    };
    let label_capacity = locator.label().map_or(0, |label| 1 + label.len());
    let mut output = String::with_capacity(
        PREFIX.len() + scheme.len() + "://".len() + locator.path().len() + label_capacity,
    );
    output.push_str(PREFIX);
    output.push_str(scheme);
    output.push_str("://");
    output.push_str(locator.path());
    if let Some(label) = locator.label() {
        output.push('#');
        output.push_str(label);
    }
    output
}

#[cfg(test)]
#[path = "tests/compute_shader_source_optimization_batch_fi_tests.rs"]
mod optimization_batch_fi_tests;
