use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::ShaderVariantPrewarmRequest;

/// Content-addressed identity for one immutable shader prewarm source artifact.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ShaderVariantPrewarmSourceId(String);

impl ShaderVariantPrewarmSourceId {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn from_cache_contract(
        wgsl_source_hash: &str,
        include_content_hashes: &[String],
        template_revision: &str,
        naga_version: &str,
        wgpu_version: &str,
    ) -> Self {
        Self(source_artifact_hash(
            wgsl_source_hash,
            include_content_hashes,
            template_revision,
            naga_version,
            wgpu_version,
        ))
    }
}

/// The one stored WGSL payload shared by every prewarm variant that uses it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShaderVariantPrewarmSource {
    pub id: ShaderVariantPrewarmSourceId,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub source_label: String,
    pub wgsl_source: String,
    #[serde(default)]
    pub source_hash: String,
    pub include_content_hashes: Vec<String>,
    pub template_revision: String,
    pub naga_version: String,
    pub wgpu_version: String,
}

/// Borrowed O(1) lookup for a manifest's immutable source table.
pub struct ShaderVariantPrewarmSourceTable<'a> {
    sources_by_id: HashMap<&'a ShaderVariantPrewarmSourceId, &'a ShaderVariantPrewarmSource>,
}

impl<'a> ShaderVariantPrewarmSourceTable<'a> {
    pub(super) fn new(sources: &'a [ShaderVariantPrewarmSource]) -> Self {
        Self {
            sources_by_id: sources.iter().map(|source| (&source.id, source)).collect(),
        }
    }

    pub fn source_for(
        &self,
        request: &ShaderVariantPrewarmRequest,
    ) -> Option<&'a ShaderVariantPrewarmSource> {
        self.sources_by_id.get(&request.source_id).copied()
    }
}

impl ShaderVariantPrewarmSource {
    pub fn new(
        source_label: impl Into<String>,
        wgsl_source: impl Into<String>,
        include_content_hashes: Vec<String>,
        template_revision: impl Into<String>,
        naga_version: impl Into<String>,
        wgpu_version: impl Into<String>,
    ) -> Self {
        let source_label = source_label.into();
        let wgsl_source = wgsl_source.into();
        let template_revision = template_revision.into();
        let naga_version = naga_version.into();
        let wgpu_version = wgpu_version.into();
        let source_hash = shader_source_hash(&wgsl_source);
        let id = ShaderVariantPrewarmSourceId::from_cache_contract(
            &source_hash,
            &include_content_hashes,
            &template_revision,
            &naga_version,
            &wgpu_version,
        );
        Self {
            id,
            source_label,
            wgsl_source,
            source_hash,
            include_content_hashes,
            template_revision,
            naga_version,
            wgpu_version,
        }
    }

    /// 预热执行前验证内容地址仍对应 WGSL、include 哈希和工具链版本，防止错误缓存复用。
    pub fn has_canonical_id(&self) -> bool {
        self.source_hash == shader_source_hash(&self.wgsl_source)
            && self.id
                == ShaderVariantPrewarmSourceId::from_cache_contract(
                    &self.source_hash,
                    &self.include_content_hashes,
                    &self.template_revision,
                    &self.naga_version,
                    &self.wgpu_version,
                )
    }

    pub fn with_source_label(&self, source_label: impl Into<String>) -> Self {
        Self::new(
            source_label,
            self.wgsl_source.clone(),
            self.include_content_hashes.clone(),
            self.template_revision.clone(),
            self.naga_version.clone(),
            self.wgpu_version.clone(),
        )
    }

    pub fn source_hash(&self) -> String {
        self.source_hash.clone()
    }

    /// 为预热预算估算持久源表的堆驻留；调用方需对总和做溢出检查后再启动串行 worker。
    pub fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.id.0.capacity()
            + self.source_label.capacity()
            + self.wgsl_source.capacity()
            + self.source_hash.capacity()
            + self.include_content_hashes.capacity() * std::mem::size_of::<String>()
            + self
                .include_content_hashes
                .iter()
                .map(String::capacity)
                .sum::<usize>()
            + self.template_revision.capacity()
            + self.naga_version.capacity()
            + self.wgpu_version.capacity()
    }
}

fn source_artifact_hash(
    wgsl_source_hash: &str,
    include_content_hashes: &[String],
    template_revision: &str,
    naga_version: &str,
    wgpu_version: &str,
) -> String {
    let mut hasher = blake3::Hasher::new();
    hash_field(&mut hasher, wgsl_source_hash.as_bytes());
    hasher.update(&(include_content_hashes.len() as u64).to_le_bytes());
    for include_content_hash in include_content_hashes {
        hash_field(&mut hasher, include_content_hash.as_bytes());
    }
    hash_field(&mut hasher, template_revision.as_bytes());
    hash_field(&mut hasher, naga_version.as_bytes());
    hash_field(&mut hasher, wgpu_version.as_bytes());
    hasher.finalize().to_hex().to_string()
}

fn shader_source_hash(wgsl_source: &str) -> String {
    blake3::hash(wgsl_source.as_bytes()).to_hex().to_string()
}

fn hash_field(hasher: &mut blake3::Hasher, field: &[u8]) {
    hasher.update(&(field.len() as u64).to_le_bytes());
    hasher.update(field);
}

#[cfg(test)]
#[path = "tests/source.rs"]
mod tests;
