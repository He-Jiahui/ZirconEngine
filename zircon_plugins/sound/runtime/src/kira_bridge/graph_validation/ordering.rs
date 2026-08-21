//! Provides deterministic cycle detection for compiled track routes.

use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
};

use zircon_runtime::core::framework::sound::{
    SoundEffectKind, SoundError, SoundMixerGraph, SoundTrackId,
};

pub(super) fn topological_track_order(
    graph: &SoundMixerGraph,
) -> Result<Vec<SoundTrackId>, SoundError> {
    let track_ids = graph
        .tracks
        .iter()
        .map(|track| track.id)
        .collect::<Vec<_>>();
    let mut outgoing = track_ids
        .iter()
        .copied()
        .map(|track| (track, Vec::new()))
        .collect::<HashMap<_, _>>();
    let mut indegree = track_ids
        .iter()
        .copied()
        .map(|track| (track, 0_usize))
        .collect::<HashMap<_, _>>();
    let track_positions = track_ids
        .iter()
        .copied()
        .enumerate()
        .map(|(position, track)| (track, position))
        .collect::<HashMap<_, _>>();

    for (source, target) in render_dependencies(graph) {
        outgoing.entry(source).or_default().push(target);
        *indegree.entry(target).or_default() += 1;
    }

    let mut ready = track_ids
        .iter()
        .enumerate()
        .filter_map(|(position, track)| {
            (indegree.get(track).copied().unwrap_or_default() == 0).then_some(Reverse(position))
        })
        .collect::<BinaryHeap<_>>();
    let mut order = Vec::with_capacity(track_ids.len());

    while let Some(Reverse(track_position)) = ready.pop() {
        let track = track_ids[track_position];
        order.push(track);
        if let Some(targets) = outgoing.get(&track) {
            for target in targets {
                let Some(target_indegree) = indegree.get_mut(target) else {
                    continue;
                };
                *target_indegree = target_indegree.saturating_sub(1);
                if *target_indegree == 0 {
                    if let Some(target_position) = track_positions.get(target) {
                        ready.push(Reverse(*target_position));
                    }
                }
            }
        }
    }

    if order.len() == track_ids.len() {
        Ok(order)
    } else {
        Err(SoundError::InvalidMixerGraph(
            "track routing contains a cycle".to_string(),
        ))
    }
}

fn render_dependencies(graph: &SoundMixerGraph) -> Vec<(SoundTrackId, SoundTrackId)> {
    let mut edges = Vec::new();
    for track in &graph.tracks {
        if let Some(parent) = track.parent {
            edges.push((track.id, parent));
        }
        for send in &track.sends {
            edges.push((track.id, send.target));
        }
        for effect in &track.effects {
            if let SoundEffectKind::Compressor(compressor) = &effect.kind {
                if let Some(sidechain) = compressor.sidechain {
                    if !sidechain.pre_effects {
                        edges.push((sidechain.track, track.id));
                    }
                }
            }
        }
    }
    edges
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, hint::black_box, time::Instant};

    use zircon_runtime::core::framework::sound::{
        SoundMixerGraph, SoundTrackDescriptor, SoundTrackId,
    };

    use super::{render_dependencies, topological_track_order};

    const BENCHMARK_TRACK_COUNT: usize = 50_000;
    const BENCHMARK_SAMPLE_COUNT: usize = 21;

    #[test]
    fn topological_order_preserves_authored_priority_when_lower_slot_becomes_ready() {
        let mut graph = SoundMixerGraph::default_stereo(48_000);
        graph
            .tracks
            .push(SoundTrackDescriptor::child(SoundTrackId::new(2), "Music"));
        graph
            .tracks
            .push(SoundTrackDescriptor::child(SoundTrackId::new(3), "Effects"));
        let mut independent = SoundTrackDescriptor::child(SoundTrackId::new(4), "Independent");
        independent.parent = None;
        graph.tracks.push(independent);

        assert_eq!(
            topological_track_order(&graph).unwrap(),
            vec![
                SoundTrackId::new(2),
                SoundTrackId::new(3),
                SoundTrackId::master(),
                SoundTrackId::new(4),
            ]
        );
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn stable_ready_heap_release_benchmark_evidence() {
        let graph = independent_track_graph(BENCHMARK_TRACK_COUNT);
        assert_eq!(
            legacy_topological_track_order(&graph),
            topological_track_order(&graph).unwrap()
        );

        let (legacy_samples, optimized_samples) = benchmark_paired_samples(
            || legacy_topological_track_order(&graph),
            || topological_track_order(&graph).expect("acyclic benchmark graph"),
        );
        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let legacy_entry_moves = BENCHMARK_TRACK_COUNT * (BENCHMARK_TRACK_COUNT - 1) / 2;
        let legacy_ns = benchmark_samples_csv(&legacy_samples);
        let optimized_ns = benchmark_samples_csv(&optimized_samples);

        println!(
            "PERF_RESULT plugins11_stable_ready_heap tracks={} samples={} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_entry_moves={} optimized_entry_moves=0 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={legacy_ns} optimized_ns={optimized_ns}",
            BENCHMARK_TRACK_COUNT,
            BENCHMARK_SAMPLE_COUNT,
            legacy_entry_moves,
            legacy_p50,
            legacy_p95,
            optimized_p50,
            optimized_p95
        );
        assert!(
            optimized_p95 * 2 <= legacy_p95,
            "optimized P95 {optimized_p95}ns must be no more than 50% of legacy P95 {legacy_p95}ns"
        );
    }

    fn benchmark_samples_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    fn independent_track_graph(track_count: usize) -> SoundMixerGraph {
        let mut graph = SoundMixerGraph::default_stereo(48_000);
        graph.tracks.clear();
        graph.tracks.reserve(track_count);
        for raw in 1..=track_count as u64 {
            let mut track = SoundTrackDescriptor::child(SoundTrackId::new(raw), "Root");
            track.parent = None;
            graph.tracks.push(track);
        }
        graph
    }

    fn benchmark_paired_samples<L, O>(
        mut legacy: impl FnMut() -> L,
        mut optimized: impl FnMut() -> O,
    ) -> (Vec<u128>, Vec<u128>) {
        black_box(legacy());
        black_box(optimized());
        let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
            if sample_index % 2 == 0 {
                legacy_samples.push(benchmark_sample(&mut legacy));
                optimized_samples.push(benchmark_sample(&mut optimized));
            } else {
                optimized_samples.push(benchmark_sample(&mut optimized));
                legacy_samples.push(benchmark_sample(&mut legacy));
            }
        }
        (legacy_samples, optimized_samples)
    }

    fn benchmark_sample<T>(operation: &mut impl FnMut() -> T) -> u128 {
        let started = Instant::now();
        let result = black_box(operation());
        let elapsed = started.elapsed().as_nanos();
        black_box(&result);
        elapsed
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        assert!(!sorted.is_empty());
        assert!((1..=100).contains(&percentile));
        let index = (sorted.len() * percentile).div_ceil(100) - 1;
        sorted[index]
    }

    fn legacy_topological_track_order(graph: &SoundMixerGraph) -> Vec<SoundTrackId> {
        let track_ids = graph
            .tracks
            .iter()
            .map(|track| track.id)
            .collect::<Vec<_>>();
        let mut outgoing = track_ids
            .iter()
            .copied()
            .map(|track| (track, Vec::new()))
            .collect::<HashMap<_, _>>();
        let mut indegree = track_ids
            .iter()
            .copied()
            .map(|track| (track, 0_usize))
            .collect::<HashMap<_, _>>();

        for (source, target) in render_dependencies(graph) {
            outgoing.entry(source).or_default().push(target);
            *indegree.entry(target).or_default() += 1;
        }

        let mut ready = track_ids
            .iter()
            .copied()
            .filter(|track| indegree.get(track).copied().unwrap_or_default() == 0)
            .collect::<Vec<_>>();
        let mut order = Vec::with_capacity(track_ids.len());
        while let Some(track) = ready.first().copied() {
            ready.remove(0);
            order.push(track);
            if let Some(targets) = outgoing.get(&track) {
                for target in targets {
                    let target_indegree = indegree.get_mut(target).expect("known target");
                    *target_indegree = target_indegree.saturating_sub(1);
                    if *target_indegree == 0 {
                        ready.push(*target);
                        ready.sort_by_key(|candidate| {
                            track_ids
                                .iter()
                                .position(|track| track == candidate)
                                .unwrap_or(usize::MAX)
                        });
                    }
                }
            }
        }
        order
    }
}
