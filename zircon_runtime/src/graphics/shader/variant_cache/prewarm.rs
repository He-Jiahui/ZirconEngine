use std::path::Path;

use crate::core::framework::render::{ShaderVariantPrewarmManifest, ShaderVariantPrewarmReport};

use super::disk::{ShaderVariantCacheDisk, ShaderVariantCacheDiskKey};

pub(crate) fn prewarm_shader_variants_to_disk(
    manifest: &ShaderVariantPrewarmManifest,
    cache_root: impl AsRef<Path>,
) -> ShaderVariantPrewarmReport {
    let mut report = ShaderVariantPrewarmReport::default();
    if manifest.schema_version != ShaderVariantPrewarmManifest::SCHEMA_VERSION {
        report.record_failure(
            0,
            format!(
                "shader variant prewarm manifest schema {} is not supported; expected {}",
                manifest.schema_version,
                ShaderVariantPrewarmManifest::SCHEMA_VERSION
            ),
        );
        return report;
    }

    let cache = ShaderVariantCacheDisk::new(cache_root.as_ref());
    for (variant_index, request) in manifest.variants.iter().enumerate() {
        let disk_key = ShaderVariantCacheDiskKey::from_variant_key(
            &request.key,
            request.include_content_hashes.iter().map(String::as_str),
        );
        match cache.write(
            &disk_key,
            &request.wgsl_source,
            &request.template_revision,
            &request.naga_version,
            &request.wgpu_version,
        ) {
            Ok(_) => report.record_written(),
            Err(error) => report.record_failure(variant_index, format!("{error:?}")),
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::core::framework::render::{
        GeometrySourceId, ShaderFeatureBits, ShaderPassType, ShaderQualityTier, ShaderVariantKey,
        ShaderVariantPrewarmManifest, ShaderVariantPrewarmRequest, SHADING_MODEL_ID_STANDARD_PBR,
    };
    use crate::core::resource::ResourceId;
    use crate::graphics::shader::{ShaderVariantCacheDisk, ShaderVariantCacheDiskKey};

    use super::prewarm_shader_variants_to_disk;

    #[test]
    fn render_shader_variant_prewarm_writes_disk_entries() {
        let root = std::env::temp_dir().join(format!(
            "zircon_shader_variant_prewarm_test_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let request = ShaderVariantPrewarmRequest {
            key: variant_key(),
            wgsl_source: "fn main() {}".to_string(),
            include_content_hashes: vec!["include-a".to_string()],
            template_revision: "template-r1".to_string(),
            naga_version: "naga-test".to_string(),
            wgpu_version: "wgpu-test".to_string(),
        };
        let disk_key = ShaderVariantCacheDiskKey::from_variant_key(
            &request.key,
            request.include_content_hashes.iter().map(String::as_str),
        );
        let manifest = ShaderVariantPrewarmManifest::new(vec![request]);

        let report = prewarm_shader_variants_to_disk(&manifest, &root);

        assert_eq!(report.requested_count, 1);
        assert_eq!(report.written_count, 1);
        assert_eq!(report.failed_count, 0);
        assert!(matches!(
            ShaderVariantCacheDisk::new(&root).lookup(&disk_key),
            super::super::disk::ShaderVariantCacheDiskLookup::Hit(_)
        ));
        let _ = fs::remove_dir_all(root);
    }

    fn variant_key() -> ShaderVariantKey {
        ShaderVariantKey {
            material_shader: ResourceId::from_stable_label("res://materials/prewarm-test.wgsl"),
            material_revision: 3,
            geometry_source: GeometrySourceId::new(0),
            shading_model: SHADING_MODEL_ID_STANDARD_PBR,
            pass_type: ShaderPassType::Forward,
            features: ShaderFeatureBits::new(ShaderFeatureBits::ALPHA_TEST),
            quality: ShaderQualityTier::Medium,
            platform_token: "wgpu-test".to_string(),
        }
    }
}
