use std::collections::{hash_map::Entry, HashMap};
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
#[path = "tests/pipeline_shader_module_references.rs"]
mod tests;
