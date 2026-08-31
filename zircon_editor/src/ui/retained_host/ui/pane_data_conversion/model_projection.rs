use crate::ui::retained_host::primitives::ModelRc;

pub(super) fn map_model_rc<T, U, F>(model: &ModelRc<T>, mut map: F) -> ModelRc<U>
where
    F: FnMut(&T) -> U,
{
    model.map_preserving_metadata(&mut map)
}

#[cfg(test)]
mod tests {
    use std::{
        hint::black_box,
        rc::Rc,
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        },
        time::Instant,
    };

    use crate::ui::layouts::common::model_rc;

    use super::*;

    const SAMPLE_PAIRS: usize = 31;
    const ITERATIONS: usize = 50_000;
    const ROWS: usize = 8;

    struct CloneProbe(Arc<AtomicUsize>);

    impl Clone for CloneProbe {
        fn clone(&self) -> Self {
            self.0.fetch_add(1, Ordering::Relaxed);
            Self(Arc::clone(&self.0))
        }
    }

    #[test]
    fn pane_model_mapping_borrows_source_rows() {
        let clone_count = Arc::new(AtomicUsize::new(0));
        let source = model_rc(vec![CloneProbe(Arc::clone(&clone_count))]);

        let mapped = map_model_rc(&source, |_| 7_u8);

        assert_eq!(mapped.row_data(0), Some(7));
        assert_eq!(clone_count.load(Ordering::Relaxed), 0);
    }

    #[derive(Debug, PartialEq, Eq)]
    struct FixtureMetadata {
        generation: u64,
    }

    #[test]
    fn optimization_batch_gt_editor575_model_projection_preserves_metadata() {
        let source =
            ModelRc::with_metadata(vec![1_u64, 2_u64], FixtureMetadata { generation: 575 });
        let source_metadata = source
            .metadata_rc::<FixtureMetadata>()
            .expect("source metadata");

        let mapped = map_model_rc(&source, |value| value * 2);
        let mapped_metadata = mapped
            .metadata_rc::<FixtureMetadata>()
            .expect("mapped metadata");

        assert_eq!(mapped.iter().copied().collect::<Vec<_>>(), vec![2, 4]);
        assert!(Rc::ptr_eq(&source_metadata, &mapped_metadata));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_gt_editor575_model_projection_direct_map_p95() {
        let source = ModelRc::with_metadata(
            (0..ROWS as u64).collect(),
            FixtureMetadata { generation: 575 },
        );
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure(&source, false));
                optimized_samples.push(measure(&source, true));
            } else {
                optimized_samples.push(measure(&source, true));
                legacy_samples.push(measure(&source, false));
            }
        }

        let legacy_p95_ns = p95(&mut legacy_samples);
        let optimized_p95_ns = p95(&mut optimized_samples);
        println!(
            "EDITOR575_MODEL_PROJECTION_METADATA_BENCH_V1 sample_pairs={SAMPLE_PAIRS} iterations={ITERATIONS} rows={ROWS} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(90),
            "expected direct model projection to lower p95 by at least 10%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn measure(source: &ModelRc<u64>, optimized: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0_u64;
        for iteration in 0..ITERATIONS {
            let mapped = if optimized {
                map_model_rc(source, |value| value.wrapping_add(iteration as u64))
            } else {
                legacy_map_model_rc(source, |value| value.wrapping_add(iteration as u64))
            };
            checksum ^= mapped
                .row_data(iteration % ROWS)
                .expect("fixture row must exist");
        }
        black_box(checksum);
        started.elapsed().as_nanos()
    }

    fn legacy_map_model_rc<T, U, F>(model: &ModelRc<T>, mut map: F) -> ModelRc<U>
    where
        T: Clone + 'static,
        U: Clone + 'static,
        F: FnMut(&T) -> U,
    {
        model_rc(model.iter().map(&mut map).collect())
    }

    fn p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[samples.len() * 95 / 100]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
