use zircon_runtime_interface::resource::AssetReference;

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
            Self::Asset { asset } => format!("compute.asset:{asset}"),
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
