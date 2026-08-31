use std::env;
use std::fs::{self, File};
use std::hint::black_box;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::*;
use crate::test_profile::{
    AllocationSnapshot, begin_allocation_profile, finish_allocation_profile,
};

const MIN_PROFILE_SAMPLE_COUNT: usize = 31;
const DEFAULT_PROFILE_WARMUP_COUNT: usize = 3;
const PROFILE_DIRECTORY_ENV: &str = "ZR_RESOURCE_MANAGEMENT_PROFILE_DIR";
const PROFILE_SAMPLES_ENV: &str = "ZR_RESOURCE_MANAGEMENT_PROFILE_SAMPLES";
const PROFILE_WARMUPS_ENV: &str = "ZR_RESOURCE_MANAGEMENT_PROFILE_WARMUPS";

#[derive(Clone, Copy, Debug)]
enum ProfileOperation {
    InitialBuild,
    Revision,
    Add,
    Mixed,
    Remove,
    Rename,
    NoProjectedChange,
}

#[derive(Clone, Copy, Debug)]
struct ProfileScenario {
    name: &'static str,
    published: usize,
    changed: usize,
    operation: ProfileOperation,
}

const PROFILE_SCENARIOS: &[ProfileScenario] = &[
    ProfileScenario {
        name: "initial_build_256",
        published: 0,
        changed: 256,
        operation: ProfileOperation::InitialBuild,
    },
    ProfileScenario {
        name: "initial_build_257",
        published: 0,
        changed: 257,
        operation: ProfileOperation::InitialBuild,
    },
    ProfileScenario {
        name: "initial_build_4096",
        published: 0,
        changed: 4_096,
        operation: ProfileOperation::InitialBuild,
    },
    ProfileScenario {
        name: "initial_build_100000",
        published: 0,
        changed: 100_000,
        operation: ProfileOperation::InitialBuild,
    },
    ProfileScenario {
        name: "dense_revision_4096",
        published: 4_096,
        changed: 4_096,
        operation: ProfileOperation::Revision,
    },
    ProfileScenario {
        name: "revision_100000_1",
        published: 100_000,
        changed: 1,
        operation: ProfileOperation::Revision,
    },
    ProfileScenario {
        name: "spread_revision_100000_64",
        published: 100_000,
        changed: 64,
        operation: ProfileOperation::Revision,
    },
    ProfileScenario {
        name: "spread_revision_100000_4096",
        published: 100_000,
        changed: 4_096,
        operation: ProfileOperation::Revision,
    },
    ProfileScenario {
        name: "add_100000_4096",
        published: 100_000,
        changed: 4_096,
        operation: ProfileOperation::Add,
    },
    ProfileScenario {
        name: "mixed_100000_4096",
        published: 100_000,
        changed: 4_096,
        operation: ProfileOperation::Mixed,
    },
    ProfileScenario {
        name: "remove_100000_4096",
        published: 100_000,
        changed: 4_096,
        operation: ProfileOperation::Remove,
    },
    ProfileScenario {
        name: "rename_100000_4096",
        published: 100_000,
        changed: 4_096,
        operation: ProfileOperation::Rename,
    },
    ProfileScenario {
        name: "dense_revision_100000",
        published: 100_000,
        changed: 100_000,
        operation: ProfileOperation::Revision,
    },
    ProfileScenario {
        name: "no_projected_change_100000",
        published: 100_000,
        changed: 100_000,
        operation: ProfileOperation::NoProjectedChange,
    },
];

struct PreparedScenario {
    scenario: ProfileScenario,
    baseline: Option<Arc<ResourceManagementGeneration>>,
    removed_ids: Vec<ResourceId>,
    records: Vec<ResourceRecord>,
    expected_row_count: usize,
}

#[derive(Clone, Copy, Debug)]
struct ProfileSample {
    elapsed_ns: u64,
    allocations: AllocationSnapshot,
}

#[test]
#[ignore = "requires an explicit non-C profile directory and a release single-thread run"]
fn resource_management_projection_current_source_profile() {
    assert!(
        !cfg!(debug_assertions),
        "profile must run with cargo test --release"
    );
    let report_directory = profile_report_directory();
    let sample_count = profile_count(PROFILE_SAMPLES_ENV, MIN_PROFILE_SAMPLE_COUNT);
    assert!(sample_count >= MIN_PROFILE_SAMPLE_COUNT);
    let warmup_count = profile_count(PROFILE_WARMUPS_ENV, DEFAULT_PROFILE_WARMUP_COUNT);
    let projection_source_blake3 = source_blake3(include_str!("../../management_projection.rs"));
    let generation_source_blake3 = source_blake3(include_str!("../../../management_generation.rs"));
    let mut all_samples = Vec::<(ProfileScenario, ProfileSample)>::new();

    for &scenario in PROFILE_SCENARIOS {
        let prepared = prepare_scenario(scenario);
        for _ in 0..warmup_count {
            let _ = execute_scenario(&prepared, false);
        }
        for _ in 0..sample_count {
            all_samples.push((scenario, execute_scenario(&prepared, true)));
        }
    }

    write_profile_reports(
        &report_directory,
        sample_count,
        warmup_count,
        &projection_source_blake3,
        &generation_source_blake3,
        &all_samples,
    );
    println!(
        "ZR_RESOURCE_MANAGEMENT_PROFILE_REPORT path={}",
        report_directory.display()
    );
}

fn profile_count(name: &str, default: usize) -> usize {
    env::var(name)
        .ok()
        .map(|value| value.parse::<usize>().expect("profile count must be usize"))
        .unwrap_or(default)
}

fn profile_report_directory() -> PathBuf {
    let directory = PathBuf::from(
        env::var_os(PROFILE_DIRECTORY_ENV).expect("ZR_RESOURCE_MANAGEMENT_PROFILE_DIR must be set"),
    );
    assert!(
        directory.is_absolute(),
        "profile directory must be absolute"
    );
    assert_profile_directory_is_not_on_c_drive(&directory);
    fs::create_dir_all(&directory).expect("create profile report directory");
    directory
}

#[cfg(windows)]
fn assert_profile_directory_is_not_on_c_drive(directory: &Path) {
    use std::path::{Component, Prefix};

    let prefix = directory
        .components()
        .next()
        .and_then(|component| match component {
            Component::Prefix(prefix) => Some(prefix.kind()),
            _ => None,
        })
        .expect("Windows profile directory must have a drive or UNC prefix");
    match prefix {
        Prefix::Disk(letter) | Prefix::VerbatimDisk(letter) => {
            assert_ne!(letter.to_ascii_uppercase(), b'C', "C drive is forbidden")
        }
        _ => {}
    }
}

#[cfg(not(windows))]
fn assert_profile_directory_is_not_on_c_drive(_directory: &Path) {}

fn source_blake3(source: &str) -> String {
    blake3::hash(source.as_bytes()).to_hex().to_string()
}

fn fixture_records(prefix: &str, count: usize) -> Vec<ResourceRecord> {
    (0..count)
        .map(|index| {
            record(
                &format!("res://profile/{prefix}/{index:06}.asset"),
                ResourceKind::Data,
                ResourceState::Ready,
            )
        })
        .collect()
}

fn selected_indices(published: usize, changed: usize) -> Vec<usize> {
    assert!(changed <= published);
    (0..changed)
        .map(|index| index.saturating_mul(published) / changed.max(1))
        .collect()
}

fn prepare_scenario(scenario: ProfileScenario) -> PreparedScenario {
    if matches!(scenario.operation, ProfileOperation::InitialBuild) {
        return PreparedScenario {
            scenario,
            baseline: None,
            removed_ids: Vec::new(),
            records: fixture_records(scenario.name, scenario.changed),
            expected_row_count: scenario.changed,
        };
    }

    let base_records = fixture_records("base", scenario.published);
    let mut baseline_projection = ResourceManagementProjection::default();
    baseline_projection.apply_delta([], base_records.iter());
    let baseline = baseline_projection.generation();
    let indices = selected_indices(scenario.published, scenario.changed);
    let mut removed_ids = Vec::new();
    let mut records = Vec::new();
    let expected_row_count = match scenario.operation {
        ProfileOperation::Revision => {
            records.extend(indices.iter().map(|&index| {
                let mut updated = base_records[index].clone();
                updated.revision = updated.revision.saturating_add(1);
                updated
            }));
            scenario.published
        }
        ProfileOperation::Add => {
            records.extend(fixture_records(scenario.name, scenario.changed));
            scenario.published.saturating_add(scenario.changed)
        }
        ProfileOperation::Remove => {
            removed_ids.extend(indices.iter().map(|&index| base_records[index].id));
            scenario.published.saturating_sub(scenario.changed)
        }
        ProfileOperation::Rename => {
            records.extend(indices.iter().enumerate().map(|(ordinal, &index)| {
                let mut renamed = base_records[index].clone();
                renamed.primary_locator =
                    ResourceLocator::parse(&format!("res://profile/renamed/{ordinal:06}.asset"))
                        .expect("valid renamed profile locator");
                renamed
            }));
            scenario.published
        }
        ProfileOperation::Mixed => {
            assert_eq!(scenario.changed % 4, 0);
            let quarter = scenario.changed / 4;
            removed_ids.extend(
                indices[..quarter]
                    .iter()
                    .map(|&index| base_records[index].id),
            );
            records.extend(indices[quarter..quarter * 2].iter().map(|&index| {
                let mut updated = base_records[index].clone();
                updated.revision = updated.revision.saturating_add(1);
                updated
            }));
            records.extend(indices[quarter * 2..quarter * 3].iter().enumerate().map(
                |(ordinal, &index)| {
                    let mut renamed = base_records[index].clone();
                    renamed.primary_locator = ResourceLocator::parse(&format!(
                        "res://profile/mixed-renamed/{ordinal:06}.asset"
                    ))
                    .expect("valid mixed profile locator");
                    renamed
                },
            ));
            records.extend(fixture_records("mixed-added", quarter));
            scenario.published
        }
        ProfileOperation::NoProjectedChange => {
            records.extend(indices.iter().map(|&index| base_records[index].clone()));
            scenario.published
        }
        ProfileOperation::InitialBuild => unreachable!(),
    };

    PreparedScenario {
        scenario,
        baseline: Some(baseline),
        removed_ids,
        records,
        expected_row_count,
    }
}

fn execute_scenario(prepared: &PreparedScenario, measured: bool) -> ProfileSample {
    let mut projection = prepared
        .baseline
        .as_ref()
        .map(|generation| ResourceManagementProjection {
            generation: Arc::clone(generation),
        })
        .unwrap_or_default();
    if measured {
        begin_allocation_profile();
    }
    let started = Instant::now();
    projection.apply_delta(
        prepared.removed_ids.iter().copied(),
        prepared.records.iter(),
    );
    black_box(projection.generation.summary().total_count());
    let elapsed = started.elapsed();
    let allocations = if measured {
        finish_allocation_profile()
    } else {
        AllocationSnapshot::default()
    };
    assert_eq!(
        projection.generation.summary().total_count(),
        prepared.expected_row_count,
        "profile scenario {} produced the wrong row count",
        prepared.scenario.name
    );
    ProfileSample {
        elapsed_ns: duration_ns(elapsed),
        allocations,
    }
}

fn duration_ns(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

fn percentile(values: &[u64], percentile: usize) -> u64 {
    assert!(!values.is_empty());
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).saturating_add(99) / 100;
    sorted[rank.saturating_sub(1).min(sorted.len() - 1)]
}

fn median_absolute_deviation(values: &[u64], median: u64) -> u64 {
    let deviations = values
        .iter()
        .map(|value| value.abs_diff(median))
        .collect::<Vec<_>>();
    percentile(&deviations, 50)
}

fn write_profile_reports(
    directory: &Path,
    sample_count: usize,
    warmup_count: usize,
    projection_source_blake3: &str,
    generation_source_blake3: &str,
    all_samples: &[(ProfileScenario, ProfileSample)],
) {
    let raw_path = directory.join("resource-management-current-raw-samples.csv");
    let summary_path = directory.join("resource-management-current-summary.csv");
    let metadata_path = directory.join("resource-management-current-metadata.txt");
    let mut raw = BufWriter::new(File::create(raw_path).expect("create raw profile CSV"));
    writeln!(
        raw,
        "scenario,published,changed,sample,elapsed_ns,allocation_count,requested_bytes,peak_live_bytes"
    )
    .expect("write raw profile header");
    for (index, (scenario, sample)) in all_samples.iter().enumerate() {
        writeln!(
            raw,
            "{},{},{},{},{},{},{},{}",
            scenario.name,
            scenario.published,
            scenario.changed,
            index % sample_count,
            sample.elapsed_ns,
            sample.allocations.allocation_count,
            sample.allocations.requested_bytes,
            sample.allocations.peak_live_bytes
        )
        .expect("write raw profile row");
    }
    raw.flush().expect("flush raw profile CSV");

    let mut summary = BufWriter::new(File::create(summary_path).expect("create summary CSV"));
    writeln!(
        summary,
        "scenario,published,changed,samples,warmups,p50_ns,p95_ns,mad_ns,allocation_count_p50,requested_bytes_p50,peak_live_bytes_p50,projection_source_blake3,generation_source_blake3"
    )
    .expect("write summary header");
    for scenario in PROFILE_SCENARIOS {
        let samples = all_samples
            .iter()
            .filter_map(|(candidate, sample)| (candidate.name == scenario.name).then_some(*sample))
            .collect::<Vec<_>>();
        let elapsed = samples
            .iter()
            .map(|sample| sample.elapsed_ns)
            .collect::<Vec<_>>();
        let allocation_count = samples
            .iter()
            .map(|sample| sample.allocations.allocation_count)
            .collect::<Vec<_>>();
        let requested_bytes = samples
            .iter()
            .map(|sample| sample.allocations.requested_bytes)
            .collect::<Vec<_>>();
        let peak_live_bytes = samples
            .iter()
            .map(|sample| sample.allocations.peak_live_bytes)
            .collect::<Vec<_>>();
        let p50 = percentile(&elapsed, 50);
        writeln!(
            summary,
            "{},{},{},{},{},{},{},{},{},{},{},{},{}",
            scenario.name,
            scenario.published,
            scenario.changed,
            samples.len(),
            warmup_count,
            p50,
            percentile(&elapsed, 95),
            median_absolute_deviation(&elapsed, p50),
            percentile(&allocation_count, 50),
            percentile(&requested_bytes, 50),
            percentile(&peak_live_bytes, 50),
            projection_source_blake3,
            generation_source_blake3
        )
        .expect("write summary row");
    }
    summary.flush().expect("flush summary CSV");

    fs::write(
        metadata_path,
        format!(
            "schema=zr_resource_management_profile_v1\nprofile=release\nrequired_test_threads=1\nsamples={sample_count}\nwarmups={warmup_count}\nprojection_source_blake3={projection_source_blake3}\ngeneration_source_blake3={generation_source_blake3}\nmetadata_queries=unavailable\nrss=unavailable\npower=unavailable\n"
        ),
    )
    .expect("write profile metadata");
}
