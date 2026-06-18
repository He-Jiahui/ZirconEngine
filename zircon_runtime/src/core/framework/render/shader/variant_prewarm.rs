use serde::{Deserialize, Serialize};

use super::ShaderVariantKey;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderVariantPrewarmManifest {
    pub schema_version: u32,
    pub variants: Vec<ShaderVariantPrewarmRequest>,
}

impl ShaderVariantPrewarmManifest {
    pub const SCHEMA_VERSION: u32 = 1;

    pub fn new(variants: Vec<ShaderVariantPrewarmRequest>) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION,
            variants,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderVariantPrewarmRequest {
    pub key: ShaderVariantKey,
    pub wgsl_source: String,
    pub include_content_hashes: Vec<String>,
    pub template_revision: String,
    pub naga_version: String,
    pub wgpu_version: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderVariantPrewarmReport {
    pub requested_count: usize,
    pub written_count: usize,
    pub failed_count: usize,
    pub failures: Vec<ShaderVariantPrewarmFailure>,
}

impl ShaderVariantPrewarmReport {
    pub fn record_written(&mut self) {
        self.requested_count += 1;
        self.written_count += 1;
    }

    pub fn record_failure(&mut self, variant_index: usize, error: impl Into<String>) {
        self.requested_count += 1;
        self.failed_count += 1;
        self.failures.push(ShaderVariantPrewarmFailure {
            variant_index,
            error: error.into(),
        });
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderVariantPrewarmFailure {
    pub variant_index: usize,
    pub error: String,
}
