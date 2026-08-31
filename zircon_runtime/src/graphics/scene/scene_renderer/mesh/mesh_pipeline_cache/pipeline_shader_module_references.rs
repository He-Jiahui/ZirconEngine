use std::collections::{HashMap, hash_map::Entry};
use std::sync::Arc;

use super::mesh_pipeline_cache::PipelineAdmissionKey;

/// Owns the reverse edge from a cached PSO identity to its shader-module key.
///
/// Module keys are interned across PSO edges. A release returns the key only
/// when the last pipeline reference disappears, allowing the caller to remove
/// the corresponding WGPU module without scanning every pipeline map.
#[derive(Default)]
pub(super) struct PipelineShaderModuleReferences {
    pipeline_modules: HashMap<PipelineAdmissionKey, Arc<str>>,
    module_reference_counts: HashMap<Arc<str>, usize>,
}

impl PipelineShaderModuleReferences {
    pub(super) fn bind(&mut self, pipeline: PipelineAdmissionKey, shader_key: &str) {
        if let Some(existing) = self.pipeline_modules.get(&pipeline) {
            assert_eq!(
                existing.as_ref(),
                shader_key,
                "pipeline identity cannot change its shader module key"
            );
            return;
        }

        let interned_key = self
            .module_reference_counts
            .get_key_value(shader_key)
            .map(|(key, _)| Arc::clone(key))
            .unwrap_or_else(|| Arc::from(shader_key));
        let count = self
            .module_reference_counts
            .entry(Arc::clone(&interned_key))
            .or_default();
        *count = count
            .checked_add(1)
            .expect("shader module pipeline reference count overflowed");
        self.pipeline_modules.insert(pipeline, interned_key);
    }

    pub(super) fn release(&mut self, pipeline: PipelineAdmissionKey) -> Option<Arc<str>> {
        let shader_key = self.pipeline_modules.remove(&pipeline)?;
        let Entry::Occupied(mut entry) =
            self.module_reference_counts.entry(Arc::clone(&shader_key))
        else {
            panic!("bound pipeline must retain its shader module reference count");
        };
        let count = entry.get_mut();
        *count = count
            .checked_sub(1)
            .expect("shader module pipeline reference count must be positive");
        if *count != 0 {
            return None;
        }
        entry.remove();
        Some(shader_key)
    }

    #[cfg(test)]
    pub(super) fn is_bound(&self, pipeline: PipelineAdmissionKey) -> bool {
        self.pipeline_modules.contains_key(&pipeline)
    }

    pub(super) fn shader_key(&self, pipeline: PipelineAdmissionKey) -> Option<&str> {
        self.pipeline_modules.get(&pipeline).map(AsRef::as_ref)
    }

    #[cfg(test)]
    pub(super) fn reference_count(&self, shader_key: &str) -> usize {
        self.module_reference_counts
            .get(shader_key)
            .copied()
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::PipelineShaderModuleReferences;
    use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
        MeshPassPipelineKind, MeshPipelineVariantId,
    };

    use super::super::PipelineCreationTarget;
    use super::super::mesh_pipeline_cache::PipelineAdmissionKey;

    fn key(target: PipelineCreationTarget, value: u32) -> PipelineAdmissionKey {
        PipelineAdmissionKey::new(target, MeshPipelineVariantId::new(value))
    }

    #[test]
    fn shared_module_is_released_only_after_its_last_pipeline() {
        let mut references = PipelineShaderModuleReferences::default();
        let base = key(
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
            7,
        );
        let gbuffer = key(
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::GBuffer),
            11,
        );

        references.bind(base, "shared-module");
        references.bind(gbuffer, "shared-module");
        assert_eq!(references.reference_count("shared-module"), 2);

        assert_eq!(references.release(base), None);
        assert_eq!(references.reference_count("shared-module"), 1);
        assert_eq!(
            references.release(gbuffer).as_deref(),
            Some("shared-module")
        );
        assert_eq!(references.reference_count("shared-module"), 0);
    }

    #[test]
    fn base_and_oit_bindings_with_one_variant_id_remain_distinct() {
        let mut references = PipelineShaderModuleReferences::default();
        let base = key(
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
            13,
        );
        let oit = key(PipelineCreationTarget::Oit, 13);

        references.bind(base, "base-module");
        references.bind(oit, "oit-module");

        assert!(references.is_bound(base));
        assert!(references.is_bound(oit));
        assert_eq!(references.release(base).as_deref(), Some("base-module"));
        assert!(references.is_bound(oit));
    }

    #[test]
    fn repeated_binding_of_the_same_pipeline_and_module_is_idempotent() {
        let mut references = PipelineShaderModuleReferences::default();
        let base = key(
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
            17,
        );

        references.bind(base, "base-module");
        references.bind(base, "base-module");

        assert_eq!(references.reference_count("base-module"), 1);
    }

    #[test]
    #[should_panic(expected = "pipeline identity cannot change its shader module key")]
    fn immutable_pipeline_identity_rejects_module_rebinding() {
        let mut references = PipelineShaderModuleReferences::default();
        let base = key(
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
            19,
        );

        references.bind(base, "first-module");
        references.bind(base, "different-module");
    }

    #[test]
    fn releasing_an_unknown_pipeline_is_a_noop() {
        let mut references = PipelineShaderModuleReferences::default();
        let unknown = key(
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Velocity),
            23,
        );

        assert_eq!(references.release(unknown), None);
    }

    #[test]
    fn every_mesh_pipeline_creation_path_binds_the_module_reverse_edge() {
        let base = include_str!("ensure_pipeline.rs");
        let pass_sources = [
            include_str!("ensure_depth_prepass_pipeline.rs"),
            include_str!("ensure_gbuffer_pipeline.rs"),
            include_str!("ensure_oit_pipeline.rs"),
            include_str!("ensure_shadow_pipeline.rs"),
            include_str!("ensure_taa_reactive_mask_pipeline.rs"),
            include_str!("ensure_velocity_pipeline.rs"),
        ];

        assert!(
            base.matches("bind_pipeline_shader_module_reference(")
                .count()
                >= 2,
            "synchronous and asynchronous Base installation must bind reverse edges"
        );
        for source in pass_sources {
            assert!(source.contains("bind_pipeline_shader_module_reference("));
        }

        let diagnostics = include_str!("pipeline_creation_diagnostics.rs");
        assert!(diagnostics.contains("release_pipeline_shader_module_reference("));
        assert!(!diagnostics.contains("self.shader_modules.remove(shader_key);"));
    }
}
