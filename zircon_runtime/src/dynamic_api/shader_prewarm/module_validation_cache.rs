use std::cell::RefCell;
use std::collections::HashMap;

use crate::core::framework::render::{ShaderVariantPrewarmSource, ShaderVariantPrewarmSourceId};

/// Per-prewarm-batch cache for source-only WGPU module validation outcomes.
pub(super) struct ShaderPrewarmModuleValidationCache {
    outcomes: RefCell<HashMap<ShaderVariantPrewarmSourceId, Result<(), String>>>,
}

impl ShaderPrewarmModuleValidationCache {
    pub(super) fn new(source_capacity: usize) -> Self {
        Self {
            outcomes: RefCell::new(HashMap::with_capacity(source_capacity)),
        }
    }

    pub(super) fn validate(
        &self,
        source: &ShaderVariantPrewarmSource,
        validate: impl FnOnce() -> Result<(), String>,
    ) -> Result<(), String> {
        if let Some(outcome) = self.outcomes.borrow().get(&source.id).cloned() {
            return outcome;
        }
        let outcome = validate();
        self.outcomes
            .borrow_mut()
            .insert(source.id.clone(), outcome.clone());
        outcome
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use crate::core::framework::render::ShaderVariantPrewarmSource;

    use super::ShaderPrewarmModuleValidationCache;

    #[test]
    fn module_validation_outcome_is_cached_by_source_id() {
        let source = ShaderVariantPrewarmSource::new(
            "res://materials/shared.wgsl",
            "fn main() {}",
            Vec::new(),
            "template-r1",
            "naga-r1",
            "wgpu-r1",
        );
        let cache = ShaderPrewarmModuleValidationCache::new(1);
        let validation_count = Cell::new(0usize);

        cache
            .validate(&source, || {
                validation_count.set(validation_count.get() + 1);
                Ok(())
            })
            .expect("first validation should pass");
        cache
            .validate(&source, || {
                validation_count.set(validation_count.get() + 1);
                Ok(())
            })
            .expect("cached validation should pass");

        assert_eq!(validation_count.get(), 1);
    }

    #[test]
    fn module_validation_failure_is_cached_by_source_id() {
        let source = ShaderVariantPrewarmSource::new(
            "res://materials/invalid.wgsl",
            "fn main() {}",
            Vec::new(),
            "template-r1",
            "naga-r1",
            "wgpu-r1",
        );
        let cache = ShaderPrewarmModuleValidationCache::new(1);
        let validation_count = Cell::new(0usize);

        let first_error = cache
            .validate(&source, || {
                validation_count.set(validation_count.get() + 1);
                Err("mock WGPU validation failure".to_owned())
            })
            .expect_err("the first validation should fail");
        let cached_error = cache
            .validate(&source, || {
                validation_count.set(validation_count.get() + 1);
                Ok(())
            })
            .expect_err("the cached validation failure should be returned");

        assert_eq!(first_error, "mock WGPU validation failure");
        assert_eq!(cached_error, first_error);
        assert_eq!(validation_count.get(), 1);
    }
}
