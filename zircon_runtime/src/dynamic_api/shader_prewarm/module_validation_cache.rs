use std::cell::RefCell;
use std::collections::HashMap;

use crate::core::framework::render::{ShaderVariantPrewarmSource, ShaderVariantPrewarmSourceId};

/// Per-prewarm-batch cache for source-only WGPU module validation outcomes.
pub(super) struct ShaderPrewarmModuleValidationCache<'source> {
    outcomes: RefCell<HashMap<&'source ShaderVariantPrewarmSourceId, Option<Result<(), String>>>>,
}

impl<'source> ShaderPrewarmModuleValidationCache<'source> {
    pub(super) fn new(sources: &'source [ShaderVariantPrewarmSource]) -> Self {
        Self {
            outcomes: RefCell::new(sources.iter().map(|source| (&source.id, None)).collect()),
        }
    }

    pub(super) fn validate(
        &self,
        source: &ShaderVariantPrewarmSource,
        validate: impl FnOnce() -> Result<(), String>,
    ) -> Result<(), String> {
        if let Some(outcome) = self
            .outcomes
            .borrow()
            .get(&source.id)
            .and_then(Option::as_ref)
            .cloned()
        {
            return outcome;
        }
        let outcome = validate();
        *self
            .outcomes
            .borrow_mut()
            .get_mut(&source.id)
            .expect("module validation source must belong to the indexed manifest") =
            Some(outcome.clone());
        outcome
    }
}

#[cfg(test)]
#[path = "tests/module_validation_cache.rs"]
mod tests;
