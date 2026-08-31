use zircon_runtime_interface::ZrRuntimeViewportHandle;

/// Editor-side value object for one runtime overlay submission.
///
/// Authoring state is projected before it crosses the gateway; the runtime
/// receives only entity identifiers plus render attributes.
#[derive(Clone, Debug, PartialEq)]
pub struct EditorRuntimeHighlightSet {
    viewport: ZrRuntimeViewportHandle,
    generation: u64,
    entities: Vec<u64>,
    outline_enabled: bool,
    tint_rgba: [f32; 4],
}

impl EditorRuntimeHighlightSet {
    pub fn new(
        viewport: ZrRuntimeViewportHandle,
        generation: u64,
        entities: impl IntoIterator<Item = u64>,
        outline_enabled: bool,
        tint_rgba: [f32; 4],
    ) -> Self {
        let mut entities = entities.into_iter().collect::<Vec<_>>();
        entities.sort_unstable();
        entities.dedup();
        Self {
            viewport,
            generation,
            entities,
            outline_enabled,
            tint_rgba,
        }
    }

    pub const fn viewport(&self) -> ZrRuntimeViewportHandle {
        self.viewport
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub fn entities(&self) -> &[u64] {
        &self.entities
    }

    pub const fn outline_enabled(&self) -> bool {
        self.outline_enabled
    }

    pub const fn tint_rgba(&self) -> [f32; 4] {
        self.tint_rgba
    }

    pub fn is_valid(&self) -> bool {
        self.viewport.is_valid() && self.tint_rgba.iter().all(|component| component.is_finite())
    }
}

#[cfg(test)]
mod optimization_tests {
    use std::collections::BTreeSet;
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    #[test]
    fn optimization_batch_20260826h_editor59_highlight_normalization_preserves_sorted_unique_entities(
    ) {
        let highlight = EditorRuntimeHighlightSet::new(
            ZrRuntimeViewportHandle::new(7),
            19,
            [9, 2, 7, 2, 9, 1],
            true,
            [0.25, 0.5, 0.75, 1.0],
        );

        assert_eq!(highlight.entities(), &[1, 2, 7, 9]);
        assert_eq!(highlight.viewport(), ZrRuntimeViewportHandle::new(7));
        assert_eq!(highlight.generation(), 19);
        assert!(highlight.outline_enabled());
        assert!(highlight.is_valid());
    }

    #[test]
    fn optimization_batch_20260826h_editor59_highlight_normalization_uses_one_vec() {
        let source = include_str!("highlight_set.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("highlight production source");
        let constructor = production
            .split("pub fn new")
            .nth(1)
            .expect("highlight constructor")
            .split("pub const fn viewport")
            .next()
            .expect("bounded highlight constructor");

        assert!(!production.contains("BTreeSet"));
        assert!(constructor.contains("collect::<Vec<_>>()"));
        assert!(constructor.contains("sort_unstable"));
        assert!(constructor.contains("dedup"));
    }

    #[test]
    #[ignore = "release performance evidence; run through the validation coordinator"]
    fn optimization_batch_20260826h_editor59_highlight_vec_normalization_performance_evidence() {
        fn legacy_new(
            viewport: ZrRuntimeViewportHandle,
            generation: u64,
            entities: Vec<u64>,
            outline_enabled: bool,
            tint_rgba: [f32; 4],
        ) -> EditorRuntimeHighlightSet {
            EditorRuntimeHighlightSet {
                viewport,
                generation,
                entities: entities
                    .into_iter()
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect(),
                outline_enabled,
                tint_rgba,
            }
        }

        let entities = (0..32_768_u64)
            .map(|index| index.wrapping_mul(0x9E37_79B9_7F4A_7C15) & 0x7FFF)
            .collect::<Vec<_>>();
        let viewport = ZrRuntimeViewportHandle::new(11);
        let mut legacy_samples = Vec::with_capacity(17);
        let mut vec_samples = Vec::with_capacity(17);
        for _ in 0..17 {
            let input = entities.clone();
            let started = Instant::now();
            black_box(legacy_new(viewport, 23, input, true, [1.0; 4]));
            legacy_samples.push(started.elapsed().as_nanos());

            let input = entities.clone();
            let started = Instant::now();
            black_box(EditorRuntimeHighlightSet::new(
                viewport, 23, input, true, [1.0; 4],
            ));
            vec_samples.push(started.elapsed().as_nanos());
        }

        legacy_samples.sort_unstable();
        vec_samples.sort_unstable();
        let legacy_p95 = legacy_samples[16];
        let vec_p95 = vec_samples[16];
        println!(
            "EDITOR59_HIGHLIGHT_VEC_NORMALIZATION_BENCH_V1 entities={} legacy_p95_ns={} vec_p95_ns={} legacy_transient_ordered_set_entries={} vec_transient_ordered_set_entries=0 target_ratio_bp=6000",
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
