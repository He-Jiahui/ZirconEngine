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
mod tests {
    use std::cell::Cell;
    use std::collections::HashMap;
    use std::hint::black_box;
    use std::time::Instant;

    use crate::core::framework::render::{
        ShaderVariantPrewarmSource, ShaderVariantPrewarmSourceId,
    };

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
        let cache = ShaderPrewarmModuleValidationCache::new(std::slice::from_ref(&source));
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
        let cache = ShaderPrewarmModuleValidationCache::new(std::slice::from_ref(&source));
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

    #[test]
    fn optimization_batch_hd_runtime585_module_validation_cache_borrows_source_ids() {
        let source = test_source(7);
        let cache = ShaderPrewarmModuleValidationCache::new(std::slice::from_ref(&source));

        cache
            .validate(&source, || Ok(()))
            .expect("source validation should pass");

        let outcomes = cache.outcomes.borrow();
        let cached_id = outcomes.keys().next().expect("cached source id");
        assert!(std::ptr::eq(*cached_id, &source.id));
        assert_eq!(
            outcomes
                .values()
                .filter(|outcome| outcome.is_some())
                .count(),
            1
        );
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_hd_runtime585_borrowed_module_validation_ids_p95() {
        const SAMPLE_PAIRS: usize = 21;
        const SOURCE_COUNT: usize = 16_384;
        let sources = (0..SOURCE_COUNT).map(test_source).collect::<Vec<_>>();
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false, &sources));
                optimized.push(measure(true, &sources));
            } else {
                optimized.push(measure(true, &sources));
                legacy.push(measure(false, &sources));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "RUNTIME585_BORROWED_MODULE_VALIDATION_IDS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
source_count={SOURCE_COUNT} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy),
            csv(&optimized)
        );
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(60),
            "borrowed module-validation source IDs must improve P95 by at least 40%"
        );
    }

    fn test_source(index: usize) -> ShaderVariantPrewarmSource {
        ShaderVariantPrewarmSource::new(
            format!("res://materials/cache-{index}.wgsl"),
            format!("fn cache_{index}() {{}}"),
            Vec::new(),
            "template-r1",
            "naga-r1",
            "wgpu-r1",
        )
    }

    fn measure(optimized: bool, sources: &[ShaderVariantPrewarmSource]) -> u128 {
        let started = Instant::now();
        let entries = if optimized {
            let cache = ShaderPrewarmModuleValidationCache::new(sources);
            for source in sources {
                cache
                    .validate(black_box(source), || Ok(()))
                    .expect("fixture validation should pass");
            }
            let entries = cache
                .outcomes
                .borrow()
                .values()
                .filter(|outcome| outcome.is_some())
                .count();
            entries
        } else {
            let mut outcomes =
                HashMap::<ShaderVariantPrewarmSourceId, Result<(), String>>::with_capacity(
                    sources.len(),
                );
            for source in sources {
                let source = black_box(source);
                if !outcomes.contains_key(&source.id) {
                    outcomes.insert(source.id.clone(), Ok(()));
                }
            }
            outcomes.len()
        };
        black_box(entries);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
