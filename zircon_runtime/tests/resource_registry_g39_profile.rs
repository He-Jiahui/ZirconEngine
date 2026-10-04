//! Actual public registry queries with a full-scan semantic reference, not a readiness substitute.

#[path = "resource_registry_g39_profile/allocation.rs"]
mod allocation;
#[cfg(windows)]
#[path = "resource_registry_g39_profile/process_counters.rs"]
mod process_counters;

use std::hint::black_box;
use std::time::Instant;
use zircon_runtime::asset::registry::{AssetRegistryEntry, AssetRegistryIndex};
use zircon_runtime::asset::{AssetKind, AssetUri, AssetUuid};

const SCALES: [usize; 3] = [10_000, 100_000, 1_000_000];
const HIGH_FANOUT: usize = 4_096;
const WARMUP_PAIRS: usize = 3;
const SAMPLE_PAIRS: usize = 101;

#[derive(Clone, Copy, Debug)]
enum Topology {
    Ring,
    IncomingFanout,
    OutgoingFanout,
}
#[derive(Clone, Copy, Debug)]
enum Query {
    ReferencersByUuid,
    ReferencersByPath,
    DependenciesByUuid,
}

struct Corpus {
    index: AssetRegistryIndex,
    target: AssetUuid,
    target_path: AssetUri,
    expected: Vec<AssetUuid>,
    node_count: usize,
    fanout: usize,
    topology: Topology,
}

fn corpus(node_count: usize, topology: Topology, fanout: usize) -> Corpus {
    assert!(node_count >= 2 && fanout < node_count);
    let uuids = (0..node_count)
        .map(|i| AssetUuid::from_stable_label(&format!("runtime51-g39-query-{i}")))
        .collect::<Vec<_>>();
    let target = uuids[0];
    let target_path = AssetUri::parse("res://runtime51-g39/node-0.asset").unwrap();
    let entries = (0..node_count).map(|i| {
        let path = AssetUri::parse(&format!("res://runtime51-g39/node-{i}.asset")).unwrap();
        let dependencies = match topology {
            Topology::Ring => vec![uuids[(i + 1) % node_count]],
            Topology::IncomingFanout if (1..=fanout).contains(&i) => vec![target],
            Topology::OutgoingFanout if i == 0 => uuids[1..=fanout].to_vec(),
            _ => Vec::new(),
        };
        AssetRegistryEntry::new(uuids[i], path, AssetKind::Data, "g39-query-fixture")
            .with_dependencies(dependencies)
    });
    let index = AssetRegistryIndex::from_entries(entries).expect("valid actual registry fixture");
    let mut expected = match topology {
        Topology::Ring => vec![uuids[node_count - 1]],
        Topology::IncomingFanout => uuids[1..=fanout].to_vec(),
        // Forward queries preserve declared dependency order; they do not sort the UUIDs.
        Topology::OutgoingFanout => uuids[1..=fanout].to_vec(),
    };
    if !matches!(topology, Topology::OutgoingFanout) {
        expected.sort_unstable_by(|left, right| left.binary_key().cmp(right.binary_key()));
    }
    Corpus {
        index,
        target,
        target_path,
        expected,
        node_count,
        fanout,
        topology,
    }
}

fn execute(fixture: &Corpus, query: Query, legacy: bool) -> Vec<AssetUuid> {
    let index = black_box(&fixture.index);
    let target = black_box(fixture.target);
    if !legacy {
        return match query {
            Query::ReferencersByUuid => index.get_referencers_by_uuid(target),
            Query::ReferencersByPath => {
                index.get_referencers_by_path(black_box(&fixture.target_path))
            }
            Query::DependenciesByUuid => index.get_dependencies_by_uuid(target),
        };
    }
    // Reference scans authoritative public rows. Its result and ordering match the API.
    // One-edge owner fanout is O(N); the forward case scans rows once then clones O(fanout).
    let target = if matches!(query, Query::ReferencersByPath) {
        match index
            .entries_iter()
            .find(|entry| entry.path() == &fixture.target_path)
        {
            Some(entry) => entry.uuid(),
            None => return Vec::new(),
        }
    } else {
        target
    };
    if matches!(query, Query::DependenciesByUuid) {
        let mut dependencies = None;
        // Visit every authoritative row even when the matching target is first.
        // black_box keeps each comparison observable in an optimized reference scan.
        for entry in index.entries_iter() {
            if black_box(entry.uuid() == target) {
                dependencies = Some(entry.dependencies());
            }
        }
        return dependencies.map(<[_]>::to_vec).unwrap_or_default();
    }
    let mut result = index
        .entries_iter()
        .filter(|entry| entry.dependencies().contains(&target))
        .map(AssetRegistryEntry::uuid)
        .collect::<Vec<_>>();
    result.sort_unstable_by(|left, right| left.binary_key().cmp(right.binary_key()));
    result
}

#[test]
fn public_high_fanout_queries_match_authoritative_rows_and_declared_order() {
    for topology in [
        Topology::Ring,
        Topology::IncomingFanout,
        Topology::OutgoingFanout,
    ] {
        let fixture = corpus(32, topology, 17);
        let queries: &[Query] = if matches!(topology, Topology::OutgoingFanout) {
            &[Query::DependenciesByUuid]
        } else {
            &[Query::ReferencersByUuid, Query::ReferencersByPath]
        };
        for &query in queries {
            assert_eq!(execute(&fixture, query, true), fixture.expected);
            assert_eq!(execute(&fixture, query, false), fixture.expected);
        }
        let absent = AssetUuid::from_stable_label("runtime51-g39-query-absent");
        assert!(fixture.index.get_referencers_by_uuid(absent).is_empty());
        assert!(fixture.index.get_dependencies_by_uuid(absent).is_empty());
    }
}

#[cfg(windows)]
#[derive(Clone, Copy, Debug)]
struct Sample {
    wall_ns: u128,
    allocations: allocation::Snapshot,
    before: process_counters::Snapshot,
    after: process_counters::Snapshot,
    delta: process_counters::Delta,
}

#[cfg(windows)]
fn measure(fixture: &Corpus, query: Query, legacy: bool) -> Sample {
    // One immutable, prebuilt fixture is shared by both arms. No fixture cloning occurs.
    // Output destruction, assertions, OS calls and allocation counting are outside latency.
    let started = Instant::now();
    let output = execute(fixture, query, legacy);
    black_box(&output);
    let wall_ns = started.elapsed().as_nanos();
    assert_eq!(output, fixture.expected);
    drop(output);

    // Separate equivalent query for actual requested-byte/allocation accounting.
    let (output, allocations) = allocation::measure(|| execute(fixture, query, legacy));
    assert_eq!(output, fixture.expected);
    drop(output);

    // OS accounting brackets another equivalent query with its output held through the
    // after snapshot. CPU deltas include bracket-call overhead and all process threads.
    let before = process_counters::Snapshot::current().expect("actual Windows counters before");
    let output = execute(fixture, query, legacy);
    black_box(&output);
    let after = process_counters::Snapshot::current().expect("actual Windows counters after");
    assert_eq!(output, fixture.expected);
    drop(output);
    let delta = process_counters::Delta::between(before, after);
    Sample {
        wall_ns,
        allocations,
        before,
        after,
        delta,
    }
}

#[cfg(windows)]
fn pair(fixture: &Corpus, query: Query, legacy_first: bool) -> (Sample, Sample) {
    if legacy_first {
        (
            measure(fixture, query, true),
            measure(fixture, query, false),
        )
    } else {
        let candidate = measure(fixture, query, false);
        let legacy = measure(fixture, query, true);
        (legacy, candidate)
    }
}

#[cfg(windows)]
fn percentile(values: &[u128], percent: usize) -> u128 {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percent).div_ceil(100).saturating_sub(1)]
}

#[cfg(windows)]
fn profile_case(fixture: &Corpus, query: Query) {
    for warmup in 0..WARMUP_PAIRS {
        let _ = pair(fixture, query, warmup % 2 == 0);
    }
    let mut raw = Vec::with_capacity(SAMPLE_PAIRS);
    for sample in 0..SAMPLE_PAIRS {
        raw.push(pair(fixture, query, sample % 2 == 0));
    }
    let old = raw.iter().map(|(old, _)| old.wall_ns).collect::<Vec<_>>();
    let new = raw.iter().map(|(_, new)| new.wall_ns).collect::<Vec<_>>();
    println!(
        "g39_query_summary,nodes={},topology={:?},fanout={},query={:?},samples={},reference_p50_ns={},reference_p95_ns={},reference_p99_ns={},candidate_p50_ns={},candidate_p95_ns={},candidate_p99_ns={},numeric_budget=pending",
        fixture.node_count,
        fixture.topology,
        fixture.fanout,
        query,
        SAMPLE_PAIRS,
        percentile(&old, 50),
        percentile(&old, 95),
        percentile(&old, 99),
        percentile(&new, 50),
        percentile(&new, 95),
        percentile(&new, 99)
    );
    for (arm, column) in [
        ("reference_full_scan_authoritative_rows", true),
        ("candidate_public_indexed_api", false),
    ] {
        let samples = raw
            .iter()
            .map(|(old, new)| if column { old } else { new })
            .collect::<Vec<_>>();
        let cpu = samples
            .iter()
            .map(|sample| {
                u128::from(sample.delta.user_100ns) + u128::from(sample.delta.kernel_100ns)
            })
            .collect::<Vec<_>>();
        let rss = samples
            .iter()
            .map(|sample| sample.after.rss_bytes as u128)
            .collect::<Vec<_>>();
        let allocations = samples
            .iter()
            .map(|sample| u128::from(sample.allocations.allocations))
            .collect::<Vec<_>>();
        println!(
            "g39_query_counter_summary,nodes={},topology={:?},fanout={},query={:?},arm={},cpu_p50_100ns={},cpu_p95_100ns={},cpu_p99_100ns={},rss_endpoint_p50_bytes={},rss_endpoint_p95_bytes={},rss_endpoint_p99_bytes={},allocations_p50={},allocations_p95={},allocations_p99={}",
            fixture.node_count,
            fixture.topology,
            fixture.fanout,
            query,
            arm,
            percentile(&cpu, 50),
            percentile(&cpu, 95),
            percentile(&cpu, 99),
            percentile(&rss, 50),
            percentile(&rss, 95),
            percentile(&rss, 99),
            percentile(&allocations, 50),
            percentile(&allocations, 95),
            percentile(&allocations, 99)
        );
    }
    for (number, (old, new)) in raw.iter().enumerate() {
        print_raw(
            fixture,
            query,
            number,
            number % 2 == 0,
            "reference_full_scan_authoritative_rows",
            old,
        );
        print_raw(
            fixture,
            query,
            number,
            number % 2 == 0,
            "candidate_public_indexed_api",
            new,
        );
    }
}

#[cfg(windows)]
fn print_raw(
    fixture: &Corpus,
    query: Query,
    pair: usize,
    legacy_first: bool,
    arm: &str,
    sample: &Sample,
) {
    println!(
        "g39_query_raw,nodes={},topology={:?},fanout={},query={:?},pair={},legacy_first={},arm={},wall_ns={},allocations={},requested_bytes={},net_live_byte_delta={},peak_net_live_bytes_above_window_start={},cpu_user_100ns={},cpu_kernel_100ns={},rss_before_bytes={},rss_after_bytes={},lifetime_peak_rss_before_bytes={},lifetime_peak_rss_after_bytes={},read_operations={},write_operations={},other_operations={},read_bytes={},write_bytes={},other_bytes={}",
        fixture.node_count,
        fixture.topology,
        fixture.fanout,
        query,
        pair,
        legacy_first,
        arm,
        sample.wall_ns,
        sample.allocations.allocations,
        sample.allocations.requested_bytes,
        sample.allocations.net_live_byte_delta,
        sample.allocations.peak_net_live_bytes_above_window_start,
        sample.delta.user_100ns,
        sample.delta.kernel_100ns,
        sample.before.rss_bytes,
        sample.after.rss_bytes,
        sample.before.lifetime_peak_rss_bytes,
        sample.after.lifetime_peak_rss_bytes,
        sample.delta.read_operations,
        sample.delta.write_operations,
        sample.delta.other_operations,
        sample.delta.read_bytes,
        sample.delta.write_bytes,
        sample.delta.other_bytes
    );
}

#[cfg(windows)]
#[test]
#[ignore = "explicit isolated Windows Release registry query scale evidence; --test-threads=1"]
fn resource_registry_g39_public_queries_release_profile() {
    assert!(!cfg!(debug_assertions), "requires Release");
    let probe = allocation::verify_preexisting_free_window();
    println!(
        "g39_allocator_preexisting_free_probe,allocations={},requested_bytes={},net_live_byte_delta={},peak_net_live_bytes_above_window_start={}",
        probe.allocations,
        probe.requested_bytes,
        probe.net_live_byte_delta,
        probe.peak_net_live_bytes_above_window_start
    );
    // Compile-time hashes bind actual production query/index and this profiling source.
    println!(
        "g39_query_environment,os={},arch={},pid={},processor={:?},cpus={:?},samples={},warmups={},pair_order=alternating,baseline=semantic_full_scan_not_historical_build,allocation_scope=process_window_separate_invocation,live_bytes=signed_net_requested_live_change_including_preexisting_frees_and_reallocations,live_peak=maximum_net_change_above_window_start_not_query_owned_or_RSS,allocator_limits=Rust_global_allocator_only_System_transient_bytes_unobserved_concurrent_activity_included,cpu_unit=100ns_process_sum_including_bracket_overhead,rss=current_working_set_endpoints,peak_rss=process_lifetime_not_query_peak,io=process_operations_and_transfers_not_disk_only,product_scan_import_persistence_io=not_measured,output_drop=outside_wall_timer,query_source_blake3={},index_source_blake3={},profile_source_blake3={},counter_source_blake3={},allocator_source_blake3={},numeric_budget=pending",
        std::env::consts::OS,
        std::env::consts::ARCH,
        std::process::id(),
        std::env::var("PROCESSOR_IDENTIFIER").ok(),
        std::thread::available_parallelism().ok(),
        SAMPLE_PAIRS,
        WARMUP_PAIRS,
        blake3::hash(include_bytes!("../src/asset/registry/query.rs")).to_hex(),
        blake3::hash(include_bytes!(
            "../src/asset/registry/asset_registry_index.rs"
        ))
        .to_hex(),
        blake3::hash(include_bytes!("resource_registry_g39_profile.rs")).to_hex(),
        blake3::hash(include_bytes!(
            "resource_registry_g39_profile/process_counters.rs"
        ))
        .to_hex(),
        blake3::hash(include_bytes!(
            "resource_registry_g39_profile/allocation.rs"
        ))
        .to_hex()
    );
    for count in SCALES {
        // One corpus at a time. Queries are at most O(N + fanout log fanout); there is
        // no million-owner legacy update or quadratic dependency closure in this test.
        let ring = corpus(count, Topology::Ring, 1);
        profile_case(&ring, Query::ReferencersByUuid);
        profile_case(&ring, Query::ReferencersByPath);
        drop(ring);
        for fanout in [HIGH_FANOUT, count - 1] {
            let incoming = corpus(count, Topology::IncomingFanout, fanout);
            profile_case(&incoming, Query::ReferencersByUuid);
            profile_case(&incoming, Query::ReferencersByPath);
            drop(incoming);
            let outgoing = corpus(count, Topology::OutgoingFanout, fanout);
            profile_case(&outgoing, Query::DependenciesByUuid);
            drop(outgoing);
        }
    }
}
