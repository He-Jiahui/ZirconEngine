use std::collections::HashMap;

use crate::graphics::scene::resources::PipelineKey;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshPassPipelineKind, MeshPipelineVariantId,
};

const FIRST_CACHE_PIPELINE_VARIANT_ID: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct MeshPipelineVariantKey {
    kind: MeshPassPipelineKind,
    pipeline_key: PipelineKey,
}

impl MeshPipelineVariantKey {
    fn new(kind: MeshPassPipelineKind, pipeline_key: &PipelineKey) -> Self {
        Self {
            kind,
            pipeline_key: pipeline_key.clone(),
        }
    }

    pub(crate) const fn kind(&self) -> MeshPassPipelineKind {
        self.kind
    }

    pub(crate) const fn pipeline_key(&self) -> &PipelineKey {
        &self.pipeline_key
    }
}

#[derive(Default)]
pub(crate) struct MeshPipelineVariantRegistry {
    variant_ids: HashMap<MeshPipelineVariantKey, MeshPipelineVariantId>,
    variant_keys: Vec<MeshPipelineVariantKey>,
}

pub(crate) trait MeshPipelineVariantResolver {
    fn resolve_variant(
        &mut self,
        kind: MeshPassPipelineKind,
        pipeline_key: &PipelineKey,
    ) -> MeshPipelineVariantId;
}

impl MeshPipelineVariantRegistry {
    pub(crate) fn resolve_variant(
        &mut self,
        kind: MeshPassPipelineKind,
        pipeline_key: &PipelineKey,
    ) -> MeshPipelineVariantId {
        let key = MeshPipelineVariantKey::new(kind, pipeline_key);
        if let Some(id) = self.variant_ids.get(&key) {
            return *id;
        }

        let id = MeshPipelineVariantId::new(
            FIRST_CACHE_PIPELINE_VARIANT_ID + self.variant_keys.len() as u32,
        );
        self.variant_keys.push(key.clone());
        self.variant_ids.insert(key, id);
        id
    }

    pub(crate) fn key_for_variant(
        &self,
        variant_id: MeshPipelineVariantId,
    ) -> Option<&MeshPipelineVariantKey> {
        let index = variant_id
            .value()
            .checked_sub(FIRST_CACHE_PIPELINE_VARIANT_ID)? as usize;
        self.variant_keys.get(index)
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.variant_keys.len()
    }
}

impl MeshPipelineVariantResolver for MeshPipelineVariantRegistry {
    fn resolve_variant(
        &mut self,
        kind: MeshPassPipelineKind,
        pipeline_key: &PipelineKey,
    ) -> MeshPipelineVariantId {
        MeshPipelineVariantRegistry::resolve_variant(self, kind, pipeline_key)
    }
}

#[cfg(test)]
mod tests {
    use crate::graphics::scene::resources::default_pipeline_key;
    use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
        MeshPassPipelineKind, MeshPipelineVariantId,
    };

    use super::MeshPipelineVariantRegistry;

    #[test]
    fn mesh_pipeline_variant_registry_reuses_pass_pipeline_shape_id() {
        let mut registry = MeshPipelineVariantRegistry::default();
        let key = default_pipeline_key();

        let first = registry.resolve_variant(MeshPassPipelineKind::Base, &key);
        let second = registry.resolve_variant(MeshPassPipelineKind::Base, &key);

        assert_eq!(first, second);
        assert_eq!(registry.len(), 1);
        assert_eq!(
            registry.key_for_variant(first).map(|key| key.kind()),
            Some(MeshPassPipelineKind::Base)
        );
    }

    #[test]
    fn mesh_pipeline_variant_registry_separates_pass_and_pipeline_shape() {
        let mut registry = MeshPipelineVariantRegistry::default();
        let base_key = default_pipeline_key();
        let mut alpha_key = base_key.clone();
        alpha_key.alpha_mask = true;

        let base = registry.resolve_variant(MeshPassPipelineKind::Base, &base_key);
        let alpha = registry.resolve_variant(MeshPassPipelineKind::Base, &alpha_key);
        let shadow = registry.resolve_variant(MeshPassPipelineKind::ShadowDepth, &base_key);

        assert_ne!(base, MeshPipelineVariantId::new(0));
        assert_ne!(base, alpha);
        assert_ne!(base, shadow);
        assert_ne!(alpha, shadow);
        assert_eq!(registry.len(), 3);
    }
}
