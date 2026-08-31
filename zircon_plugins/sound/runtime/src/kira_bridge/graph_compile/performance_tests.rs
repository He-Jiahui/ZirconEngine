use std::collections::{HashMap, HashSet};
use std::hint::black_box;
use std::time::{Duration, Instant};

use zircon_runtime::core::framework::sound::{SoundMixerGraph, SoundTrackDescriptor, SoundTrackId};

use super::TrackHierarchyIndex;

const PERFORMANCE_TRACK_COUNT: usize = 2_048;
const PERFORMANCE_SAMPLE_PAIRS: usize = 21;

#[test]
fn hierarchy_index_matches_legacy_depth_ancestor_and_subtree_queries() {
    let graph = branching_graph(257);
    let candidates = structural_candidates(&graph);
    let hierarchy = TrackHierarchyIndex::new(&graph);

    for track in &graph.tracks {
        assert_eq!(
            hierarchy.depth(track.id),
            legacy_depth(&graph, track.id),
            "depth changed for track {:?}",
            track.id,
        );
        assert_eq!(
            hierarchy.has_ancestor_in(track.id, &candidates),
            legacy_has_ancestor_in(&graph, track.id, &candidates),
            "candidate ancestor result changed for track {:?}",
            track.id,
        );
    }

    for root in graph.tracks.iter().step_by(17).map(|track| track.id) {
        assert_eq!(
            hierarchy.subtree_ids(root),
            legacy_subtree_ids(&graph, root),
            "subtree membership changed for root {:?}",
            root,
        );
    }
}

#[test]
#[ignore = "release-only Runtime99zn hierarchy projection benchmark"]
fn runtime99zn_sound_graph_hierarchy_release_benchmark_evidence() {
    let graph = branching_graph(PERFORMANCE_TRACK_COUNT);
    let candidates = structural_candidates(&graph);
    let legacy_checksum = legacy_projection(&graph, &candidates);
    let indexed_checksum = indexed_projection(&graph, &candidates);
    assert_eq!(legacy_checksum, indexed_checksum);

    let mut legacy_samples = Vec::with_capacity(PERFORMANCE_SAMPLE_PAIRS);
    let mut indexed_samples = Vec::with_capacity(PERFORMANCE_SAMPLE_PAIRS);
    for pair in 0..PERFORMANCE_SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(|| legacy_projection(&graph, &candidates)));
            indexed_samples.push(measure(|| indexed_projection(&graph, &candidates)));
        } else {
            indexed_samples.push(measure(|| indexed_projection(&graph, &candidates)));
            legacy_samples.push(measure(|| legacy_projection(&graph, &candidates)));
        }
    }

    let legacy_p50 = nearest_rank(&mut legacy_samples, 50);
    let legacy_p95 = nearest_rank(&mut legacy_samples, 95);
    let indexed_p50 = nearest_rank(&mut indexed_samples, 50);
    let indexed_p95 = nearest_rank(&mut indexed_samples, 95);
    let p50_improvement = improvement_percent(legacy_p50, indexed_p50);
    let p95_improvement = improvement_percent(legacy_p95, indexed_p95);
    let roots = legacy_roots(&graph, &candidates);
    let legacy_parent_map_builds = candidates
        .len()
        .saturating_mul(2)
        .saturating_add(roots.len());
    let legacy_ns = duration_csv(&legacy_samples);
    let indexed_ns = duration_csv(&indexed_samples);

    eprintln!(
        "RUNTIME99ZN_SOUND_GRAPH_HIERARCHY_PERF tracks={} candidates={} roots={} sample_pairs={} sample_order=alternating_legacy_first_even percentile_method=nearest_rank threshold_percent=40 legacy_ns={} indexed_ns={} legacy_p50_ns={} indexed_p50_ns={} p50_improvement_pct={:.3} legacy_p95_ns={} indexed_p95_ns={} p95_improvement_pct={:.3} legacy_parent_map_builds={} indexed_parent_map_builds=1",
        graph.tracks.len(),
        candidates.len(),
        roots.len(),
        PERFORMANCE_SAMPLE_PAIRS,
        legacy_ns,
        indexed_ns,
        legacy_p50.as_nanos(),
        indexed_p50.as_nanos(),
        p50_improvement,
        legacy_p95.as_nanos(),
        indexed_p95.as_nanos(),
        p95_improvement,
        legacy_parent_map_builds,
    );

    assert!(
        p50_improvement >= 40.0,
        "hierarchy index must improve structural projection P50 by at least 40%"
    );
    assert!(
        p95_improvement >= 40.0,
        "hierarchy index must improve structural projection P95 by at least 40%"
    );
}

fn branching_graph(track_count: usize) -> SoundMixerGraph {
    let mut graph = SoundMixerGraph::default_stereo(48_000);
    graph.tracks.reserve(track_count);
    for offset in 0..track_count {
        let id = SoundTrackId::new(offset as u64 + 2);
        let mut track = SoundTrackDescriptor::child(id, format!("Track {offset}"));
        track.parent = (offset != 0)
            .then(|| SoundTrackId::new(((offset - 1) / 2) as u64 + 2))
            .or(Some(SoundTrackId::master()));
        graph.tracks.push(track);
    }
    graph
}

fn structural_candidates(graph: &SoundMixerGraph) -> HashSet<SoundTrackId> {
    graph
        .tracks
        .iter()
        .skip(1)
        .filter(|track| track.id.raw() % 17 == 0 || track.id.raw() % 31 == 0)
        .map(|track| track.id)
        .collect()
}

fn legacy_projection(graph: &SoundMixerGraph, candidates: &HashSet<SoundTrackId>) -> usize {
    let roots = legacy_roots(graph, candidates);
    let subtree_total = roots
        .iter()
        .map(|root| legacy_subtree_ids(graph, *root).len())
        .sum::<usize>();
    let depth_total = candidates
        .iter()
        .map(|track| legacy_depth(graph, *track))
        .sum::<usize>();
    roots.len() ^ subtree_total ^ depth_total
}

fn indexed_projection(graph: &SoundMixerGraph, candidates: &HashSet<SoundTrackId>) -> usize {
    let hierarchy = TrackHierarchyIndex::new(graph);
    let roots = candidates
        .iter()
        .copied()
        .filter(|candidate| !hierarchy.has_ancestor_in(*candidate, candidates))
        .collect::<Vec<_>>();
    let subtree_total = roots
        .iter()
        .map(|root| hierarchy.subtree_ids(*root).len())
        .sum::<usize>();
    let depth_total = candidates
        .iter()
        .map(|track| hierarchy.depth(*track))
        .sum::<usize>();
    roots.len() ^ subtree_total ^ depth_total
}

fn legacy_roots(graph: &SoundMixerGraph, candidates: &HashSet<SoundTrackId>) -> Vec<SoundTrackId> {
    candidates
        .iter()
        .copied()
        .filter(|candidate| !legacy_has_ancestor_in(graph, *candidate, candidates))
        .collect()
}

fn legacy_depth(graph: &SoundMixerGraph, track: SoundTrackId) -> usize {
    let parents = legacy_parent_lookup(graph);
    let mut depth = 0;
    let mut cursor = parents.get(&track).copied().flatten();
    while let Some(parent) = cursor {
        depth += 1;
        cursor = parents.get(&parent).copied().flatten();
    }
    depth
}

fn legacy_has_ancestor_in(
    graph: &SoundMixerGraph,
    track: SoundTrackId,
    candidates: &HashSet<SoundTrackId>,
) -> bool {
    let parents = legacy_parent_lookup(graph);
    let mut cursor = parents.get(&track).copied().flatten();
    while let Some(parent) = cursor {
        if candidates.contains(&parent) {
            return true;
        }
        cursor = parents.get(&parent).copied().flatten();
    }
    false
}

fn legacy_subtree_ids(graph: &SoundMixerGraph, root: SoundTrackId) -> HashSet<SoundTrackId> {
    let parents = legacy_parent_lookup(graph);
    let mut subtree = HashSet::from([root]);
    loop {
        let before = subtree.len();
        for (track, parent) in &parents {
            if parent.is_some_and(|parent| subtree.contains(&parent)) {
                subtree.insert(*track);
            }
        }
        if subtree.len() == before {
            return subtree;
        }
    }
}

fn legacy_parent_lookup(graph: &SoundMixerGraph) -> HashMap<SoundTrackId, Option<SoundTrackId>> {
    graph
        .tracks
        .iter()
        .map(|track| (track.id, track.parent))
        .collect()
}

fn measure(mut projection: impl FnMut() -> usize) -> Duration {
    let started = Instant::now();
    black_box(projection());
    started.elapsed()
}

fn nearest_rank(samples: &mut [Duration], percentile: usize) -> Duration {
    samples.sort_unstable();
    let rank = samples
        .len()
        .saturating_mul(percentile)
        .div_ceil(100)
        .max(1);
    samples[rank - 1]
}

fn improvement_percent(before: Duration, after: Duration) -> f64 {
    100.0 * (before.as_nanos() as f64 - after.as_nanos() as f64) / before.as_nanos() as f64
}

fn duration_csv(samples: &[Duration]) -> String {
    samples
        .iter()
        .map(|sample| sample.as_nanos().to_string())
        .collect::<Vec<_>>()
        .join(",")
}
