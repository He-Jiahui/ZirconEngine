use crate::core::framework::scene::EntityId;

/// Runtime-owned, editor-neutral overlay input for one viewport frame.
///
/// Entity IDs are canonicalized on construction so every consumer observes a
/// stable order regardless of the editor-side container that produced them.
#[derive(Clone, Debug, PartialEq)]
pub struct HighlightSet {
    entities: Vec<EntityId>,
    attributes: HighlightRenderAttributes,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HighlightRenderAttributes {
    pub outline_enabled: bool,
    pub tint_rgba: [f32; 4],
}

impl HighlightRenderAttributes {
    pub const fn outlined(tint_rgba: [f32; 4]) -> Self {
        Self {
            outline_enabled: true,
            tint_rgba,
        }
    }

    pub fn is_valid(self) -> bool {
        self.tint_rgba.iter().all(|component| component.is_finite())
    }
}

impl HighlightSet {
    pub fn new(
        entities: impl IntoIterator<Item = EntityId>,
        attributes: HighlightRenderAttributes,
    ) -> Self {
        let mut entities = entities.into_iter().collect::<Vec<_>>();
        entities.sort_unstable();
        entities.dedup();
        Self {
            entities,
            attributes,
        }
    }

    pub fn entities(&self) -> &[EntityId] {
        &self.entities
    }

    pub(crate) fn entity_capacity(&self) -> usize {
        self.entities.capacity()
    }

    pub const fn attributes(&self) -> HighlightRenderAttributes {
        self.attributes
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::hint::black_box;
    use std::time::Instant;

    use super::{HighlightRenderAttributes, HighlightSet};

    #[test]
    fn canonicalizes_entity_order_and_duplicates() {
        let set = HighlightSet::new(
            [9, 2, 9, 4],
            HighlightRenderAttributes::outlined([0.2, 0.4, 0.6, 1.0]),
        );

        assert_eq!(set.entities(), &[2, 4, 9]);
        assert!(set.entity_capacity() >= set.entities().len());
    }

    #[test]
    fn optimization_batch_20260826i_runtime10_highlight_normalization_preserves_attributes() {
        let attributes = HighlightRenderAttributes::outlined([0.2, 0.4, 0.6, 1.0]);
        let set = HighlightSet::new([11, 3, 7, 3, 11, 2], attributes);

        assert_eq!(set.entities(), &[2, 3, 7, 11]);
        assert_eq!(set.attributes(), attributes);
        assert!(set.attributes().is_valid());
    }

    #[test]
    fn optimization_batch_20260826i_runtime10_highlight_normalization_uses_one_vec() {
        let source = include_str!("highlight_set.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("highlight production source");
        let constructor = production
            .split("pub fn new")
            .nth(1)
            .expect("highlight constructor")
            .split("pub fn entities")
            .next()
            .expect("bounded highlight constructor");

        assert!(!production.contains("BTreeSet"));
        assert!(constructor.contains("collect::<Vec<_>>()"));
        assert!(constructor.contains("sort_unstable"));
        assert!(constructor.contains("dedup"));
    }

    #[test]
    #[ignore = "release performance evidence; run through the validation coordinator"]
    fn optimization_batch_20260826i_runtime10_highlight_vec_normalization_performance_evidence() {
        fn legacy_new(entities: Vec<u64>, attributes: HighlightRenderAttributes) -> HighlightSet {
            HighlightSet {
                entities: entities
                    .into_iter()
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect(),
                attributes,
            }
        }

        let entities = (0..32_768_u64)
            .map(|index| index.wrapping_mul(0x9E37_79B9_7F4A_7C15) & 0x7FFF)
            .collect::<Vec<_>>();
        let attributes = HighlightRenderAttributes::outlined([0.2, 0.4, 0.6, 1.0]);
        let mut legacy_samples = Vec::with_capacity(17);
        let mut vec_samples = Vec::with_capacity(17);
        for _ in 0..17 {
            let input = entities.clone();
            let started = Instant::now();
            black_box(legacy_new(input, attributes));
            legacy_samples.push(started.elapsed().as_nanos());

            let input = entities.clone();
            let started = Instant::now();
            black_box(HighlightSet::new(input, attributes));
            vec_samples.push(started.elapsed().as_nanos());
        }

        legacy_samples.sort_unstable();
        vec_samples.sort_unstable();
        let legacy_p95 = legacy_samples[16];
        let vec_p95 = vec_samples[16];
        println!(
            "RUNTIME10_HIGHLIGHT_VEC_NORMALIZATION_BENCH_V1 entities={} legacy_p95_ns={} vec_p95_ns={} legacy_transient_ordered_set_entries={} vec_transient_ordered_set_entries=0 target_ratio_bp=6000",
            entities.len(),
            legacy_p95,
            vec_p95,
            entities.len(),
        );
        assert!(
            vec_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(6_000),
            "Vec highlight normalization P95 {vec_p95} ns exceeded 60% of legacy {legacy_p95} ns"
        );
    }
}
