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
mod tests {
    use super::ShaderVariantPrewarmSource;
    use crate::core::framework::render::{
        ShaderFeatureBits, ShaderPassType, ShaderQualityTier, ShaderVariantKey,
        GEOMETRY_SOURCE_ID_STATIC_MESH, SHADING_MODEL_ID_STANDARD_PBR,
    };
    use crate::core::resource::ResourceId;

    use super::super::{ShaderVariantPrewarmManifest, ShaderVariantPrewarmRequest};

    #[test]
    fn shader_prewarm_source_id_tracks_every_cache_relevant_field() {
        let source = ShaderVariantPrewarmSource::new(
            "res://shader.wgsl",
            "fn main() {}",
            vec!["include-a".to_string()],
            "template-r1",
            "naga-r1",
            "wgpu-r1",
        );
        let changed_include = ShaderVariantPrewarmSource::new(
            "res://shader.wgsl",
            "fn main() {}",
            vec!["include-b".to_string()],
            "template-r1",
            "naga-r1",
            "wgpu-r1",
        );
        let changed_source = ShaderVariantPrewarmSource::new(
            "res://shader.wgsl",
            "fn changed() {}",
            vec!["include-a".to_string()],
            "template-r1",
            "naga-r1",
            "wgpu-r1",
        );
        let changed_template = ShaderVariantPrewarmSource::new(
            "res://shader.wgsl",
            "fn main() {}",
            vec!["include-a".to_string()],
            "template-r2",
            "naga-r1",
            "wgpu-r1",
        );
        let changed_naga = ShaderVariantPrewarmSource::new(
            "res://shader.wgsl",
            "fn main() {}",
            vec!["include-a".to_string()],
            "template-r1",
            "naga-r2",
            "wgpu-r1",
        );
        let changed_wgpu = ShaderVariantPrewarmSource::new(
            "res://shader.wgsl",
            "fn main() {}",
            vec!["include-a".to_string()],
            "template-r1",
            "naga-r1",
            "wgpu-r2",
        );
        let changed_label = source.with_source_label("res://renamed-shader.wgsl");

        assert!(source.has_canonical_id());
        assert_ne!(source.id, changed_include.id);
        assert_ne!(source.id, changed_source.id);
        assert_ne!(source.id, changed_template.id);
        assert_ne!(source.id, changed_naga.id);
        assert_ne!(source.id, changed_wgpu.id);
        assert_eq!(source.id, changed_label.id);
        assert_eq!(source.id.as_str().len(), 64);
    }

    #[test]
    fn legacy_source_without_persisted_hash_parses_but_is_not_canonical() {
        let source = ShaderVariantPrewarmSource::new(
            "res://shader.wgsl",
            "fn main() {}",
            Vec::new(),
            "template-r1",
            "naga-r1",
            "wgpu-r1",
        );
        let mut value = serde_json::to_value(source).expect("serialize source");
        value
            .as_object_mut()
            .expect("source is an object")
            .remove("source_hash");

        let legacy = serde_json::from_value::<ShaderVariantPrewarmSource>(value)
            .expect("old manifest source should reach the schema gate");

        assert!(legacy.source_hash.is_empty());
        assert!(!legacy.has_canonical_id());
    }

    #[test]
    fn shader_prewarm_source_table_resolves_requests_by_source_id() {
        let first = ShaderVariantPrewarmSource::new(
            "res://first.wgsl",
            "fn first() {}",
            Vec::new(),
            "template-r1",
            "naga-r1",
            "wgpu-r1",
        );
        let second = ShaderVariantPrewarmSource::new(
            "res://second.wgsl",
            "fn second() {}",
            Vec::new(),
            "template-r1",
            "naga-r1",
            "wgpu-r1",
        );
        let manifest = ShaderVariantPrewarmManifest::new(
            vec![first, second.clone()],
            vec![ShaderVariantPrewarmRequest {
                key: ShaderVariantKey {
                    material_shader: ResourceId::from_stable_label("res://second.wgsl"),
                    material_revision: 1,
                    material_layout_hash: 0,
                    material_option_bits: 0,
                    geometry_source: GEOMETRY_SOURCE_ID_STATIC_MESH,
                    shading_model: SHADING_MODEL_ID_STANDARD_PBR,
                    pass_type: ShaderPassType::Forward,
                    features: ShaderFeatureBits::new(0),
                    quality: ShaderQualityTier::Medium,
                    platform_token: "test".to_string(),
                },
                pipeline_state: None,
                source_id: second.id.clone(),
            }],
        );

        let source_table = manifest.source_table();
        assert_eq!(
            source_table
                .source_for(&manifest.variants[0])
                .expect("source table should resolve a non-first source")
                .id,
            second.id
        );
    }

    #[test]
    #[ignore = "scale gate: 100k source-table lookups without per-variant WGSL payloads"]
    fn shader_prewarm_source_table_scales_to_100k_variant_lookups() {
        const VARIANT_COUNT: usize = 100_000;

        let source = ShaderVariantPrewarmSource::new(
            "res://shared.wgsl",
            "fn main() {}",
            Vec::new(),
            "template-r1",
            "naga-r1",
            "wgpu-r1",
        );
        let variants = (0..VARIANT_COUNT)
            .map(|index| ShaderVariantPrewarmRequest {
                key: ShaderVariantKey {
                    material_shader: ResourceId::from_stable_label("res://shared.wgsl"),
                    material_revision: index as u64,
                    material_layout_hash: 0,
                    material_option_bits: 0,
                    geometry_source: GEOMETRY_SOURCE_ID_STATIC_MESH,
                    shading_model: SHADING_MODEL_ID_STANDARD_PBR,
                    pass_type: ShaderPassType::Forward,
                    features: ShaderFeatureBits::new(0),
                    quality: ShaderQualityTier::Medium,
                    platform_token: "scale".to_string(),
                },
                pipeline_state: None,
                source_id: source.id.clone(),
            })
            .collect();
        let manifest = ShaderVariantPrewarmManifest::new(vec![source], variants);
        let source_table = manifest.source_table();
        let shared_source = &manifest.sources[0];

        for request in &manifest.variants {
            assert!(std::ptr::eq(
                source_table
                    .source_for(request)
                    .expect("scale request must resolve its shared source"),
                shared_source,
            ));
        }
    }

    #[test]
    fn shader_prewarm_source_table_rejects_legacy_inline_payload_fields() {
        let mut manifest_value = serde_json::to_value(ShaderVariantPrewarmManifest::empty())
            .expect("manifest should serialize");
        manifest_value
            .as_object_mut()
            .expect("manifest should serialize as an object")
            .insert("legacy_sources".to_string(), serde_json::json!([]));
        assert!(
            serde_json::from_value::<ShaderVariantPrewarmManifest>(manifest_value).is_err(),
            "schema-3 manifests must reject removed compatibility fields"
        );

        let source = ShaderVariantPrewarmSource::new(
            "res://shader.wgsl",
            "fn main() {}",
            Vec::new(),
            "template-r1",
            "naga-r1",
            "wgpu-r1",
        );
        let mut source_value = serde_json::to_value(&source).expect("source should serialize");
        source_value
            .as_object_mut()
            .expect("source should serialize as an object")
            .insert(
                "legacy_variant_wgsl".to_string(),
                serde_json::json!("fn old() {}"),
            );
        assert!(
            serde_json::from_value::<ShaderVariantPrewarmSource>(source_value).is_err(),
            "schema-3 sources must reject removed compatibility payload fields"
        );

        let request = ShaderVariantPrewarmRequest {
            key: ShaderVariantKey {
                material_shader: ResourceId::from_stable_label("res://shader.wgsl"),
                material_revision: 1,
                material_layout_hash: 0,
                material_option_bits: 0,
                geometry_source: GEOMETRY_SOURCE_ID_STATIC_MESH,
                shading_model: SHADING_MODEL_ID_STANDARD_PBR,
                pass_type: ShaderPassType::Forward,
                features: ShaderFeatureBits::new(0),
                quality: ShaderQualityTier::Medium,
                platform_token: "test".to_string(),
            },
            pipeline_state: None,
            source_id: source.id,
        };
        let mut request_value = serde_json::to_value(request).expect("request should serialize");
        request_value
            .as_object_mut()
            .expect("request should serialize as an object")
            .insert("wgsl_source".to_string(), serde_json::json!("fn old() {}"));
        assert!(
            serde_json::from_value::<ShaderVariantPrewarmRequest>(request_value).is_err(),
            "schema-3 requests must only reference a source id"
        );
    }
}
