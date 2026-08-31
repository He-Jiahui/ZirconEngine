use std::any::TypeId;
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::hint::black_box;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use super::*;
use crate::test_profile::{
    begin_allocation_profile, finish_allocation_profile, AllocationSnapshot,
};

const MIN_PROFILE_SAMPLE_COUNT: usize = 31;
const DEFAULT_PROFILE_WARMUP_COUNT: usize = 3;
const PROFILE_DIRECTORY_ENV: &str = "ZR_RESOURCE_READINESS_PROFILE_DIR";
const PROFILE_SAMPLES_ENV: &str = "ZR_RESOURCE_READINESS_PROFILE_SAMPLES";
const PROFILE_WARMUPS_ENV: &str = "ZR_RESOURCE_READINESS_PROFILE_WARMUPS";
const PROFILE_SCENARIO_ENV: &str = "ZR_RESOURCE_READINESS_PROFILE_SCENARIO";
const PROFILE_SCOPE_ENV: &str = "ZR_RESOURCE_READINESS_PROFILE_SCOPE";

#[derive(Clone, Copy, Debug)]
enum ProfileTopology {
    Chain,
    Fanout,
    Diamond,
    Independent,
    Cycle,
}

impl ProfileTopology {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Chain => "chain",
            Self::Fanout => "fanout",
            Self::Diamond => "diamond",
            Self::Independent => "independent",
            Self::Cycle => "cycle",
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum ProfileOperation {
    InitialBuild,
    LeafReload,
    DenseReload,
    NoChange,
    RootEdgeReplacement,
    MissingDependencyArrival,
}

impl ProfileOperation {
    const fn as_str(self) -> &'static str {
        match self {
            Self::InitialBuild => "initial_build",
            Self::LeafReload => "leaf_reload",
            Self::DenseReload => "dense_reload",
            Self::NoChange => "no_change",
            Self::RootEdgeReplacement => "root_edge_replacement",
            Self::MissingDependencyArrival => "missing_dependency_arrival",
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum ProfileMeasurementScope {
    ManagerEndToEnd,
    EvaluatorOnly,
}

impl ProfileMeasurementScope {
    const ALL: [Self; 2] = [Self::ManagerEndToEnd, Self::EvaluatorOnly];

    const fn as_str(self) -> &'static str {
        match self {
            Self::ManagerEndToEnd => "manager_end_to_end",
            Self::EvaluatorOnly => "evaluator_only",
        }
    }

    fn parse(value: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|scope| scope.as_str() == value)
            .expect("known readiness profile measurement scope")
    }
}

#[derive(Clone, Copy, Debug)]
struct ProfileScenario {
    name: &'static str,
    node_count: usize,
    topology: ProfileTopology,
    operation: ProfileOperation,
}

const PROFILE_SCENARIOS: &[ProfileScenario] = &[
    ProfileScenario {
        name: "initial_chain_1000",
        node_count: 1_000,
        topology: ProfileTopology::Chain,
        operation: ProfileOperation::InitialBuild,
    },
    ProfileScenario {
        name: "leaf_reload_chain_1000",
        node_count: 1_000,
        topology: ProfileTopology::Chain,
        operation: ProfileOperation::LeafReload,
    },
    ProfileScenario {
        name: "leaf_reload_chain_10000",
        node_count: 10_000,
        topology: ProfileTopology::Chain,
        operation: ProfileOperation::LeafReload,
    },
    ProfileScenario {
        name: "initial_chain_100000",
        node_count: 100_000,
        topology: ProfileTopology::Chain,
        operation: ProfileOperation::InitialBuild,
    },
    ProfileScenario {
        name: "leaf_reload_fanout_64",
        node_count: 64,
        topology: ProfileTopology::Fanout,
        operation: ProfileOperation::LeafReload,
    },
    ProfileScenario {
        name: "leaf_reload_fanout_4096",
        node_count: 4_096,
        topology: ProfileTopology::Fanout,
        operation: ProfileOperation::LeafReload,
    },
    ProfileScenario {
        name: "leaf_reload_fanout_100000",
        node_count: 100_000,
        topology: ProfileTopology::Fanout,
        operation: ProfileOperation::LeafReload,
    },
    ProfileScenario {
        name: "leaf_reload_diamond_4096",
        node_count: 4_096,
        topology: ProfileTopology::Diamond,
        operation: ProfileOperation::LeafReload,
    },
    ProfileScenario {
        name: "leaf_reload_diamond_100000",
        node_count: 100_000,
        topology: ProfileTopology::Diamond,
        operation: ProfileOperation::LeafReload,
    },
    ProfileScenario {
        name: "dense_reload_4096",
        node_count: 4_096,
        topology: ProfileTopology::Independent,
        operation: ProfileOperation::DenseReload,
    },
    ProfileScenario {
        name: "self_cycle",
        node_count: 1,
        topology: ProfileTopology::Cycle,
        operation: ProfileOperation::InitialBuild,
    },
    ProfileScenario {
        name: "two_node_cycle",
        node_count: 2,
        topology: ProfileTopology::Cycle,
        operation: ProfileOperation::InitialBuild,
    },
    ProfileScenario {
        name: "cycle_4096",
        node_count: 4_096,
        topology: ProfileTopology::Cycle,
        operation: ProfileOperation::InitialBuild,
    },
    ProfileScenario {
        name: "root_edge_replacement_chain_4096",
        node_count: 4_096,
        topology: ProfileTopology::Chain,
        operation: ProfileOperation::RootEdgeReplacement,
    },
    ProfileScenario {
        name: "missing_dependency_arrival_chain_4096",
        node_count: 4_096,
        topology: ProfileTopology::Chain,
        operation: ProfileOperation::MissingDependencyArrival,
    },
    ProfileScenario {
        name: "no_change_100000",
        node_count: 100_000,
        topology: ProfileTopology::Independent,
        operation: ProfileOperation::NoChange,
    },
];

struct PreparedScenario {
    scenario: ProfileScenario,
    baseline: Option<ResourceReadinessProjection>,
    updates: Vec<ResourceReadinessSourceUpdate>,
    manager_sources: PreparedManagerSources,
}

struct PreparedManagerSources {
    update_ids: Vec<ResourceId>,
    records: HashMap<ResourceId, ResourceRecord>,
    runtime_states: HashMap<ResourceId, RuntimeResourceState>,
    payload_type_ids: HashMap<ResourceId, TypeId>,
}

impl PreparedManagerSources {
    fn from_updates(updates: &[ResourceReadinessSourceUpdate]) -> Self {
        let mut records = HashMap::with_capacity(updates.len());
        let mut runtime_states = HashMap::with_capacity(updates.len());
        let mut payload_type_ids = HashMap::with_capacity(updates.len());
        for update in updates {
            if let Some(record) = update.record.as_ref() {
                records.insert(update.id, record.clone());
            }
            runtime_states.insert(update.id, update.runtime_state);
            if let Some(payload_type_id) = update.payload_type_id {
                payload_type_ids.insert(update.id, payload_type_id);
            }
        }
        Self {
            update_ids: updates.iter().map(|update| update.id).collect(),
            records,
            runtime_states,
            payload_type_ids,
        }
    }

    fn materialize_updates(&self) -> Vec<ResourceReadinessSourceUpdate> {
        self.update_ids
            .iter()
            .map(|id| ResourceReadinessSourceUpdate {
                id: *id,
                record: self.records.get(id).cloned(),
                runtime_state: self
                    .runtime_states
                    .get(id)
                    .copied()
                    .unwrap_or(RuntimeResourceState::Unloaded),
                payload_type_id: self.payload_type_ids.get(id).copied(),
            })
            .collect()
    }
}

#[test]
fn prepared_manager_sources_materialize_authority_shaped_updates() {
    let existing = source_update(ready_record("profile/manager-source", Vec::new()));
    let removed_id = ResourceId::from_stable_label("profile-manager-removed");
    let removed = ResourceReadinessSourceUpdate {
        id: removed_id,
        record: None,
        runtime_state: RuntimeResourceState::Unloaded,
        payload_type_id: None,
    };
    let sources = PreparedManagerSources::from_updates(&[existing.clone(), removed]);

    let materialized = sources.materialize_updates();

    assert_eq!(materialized.len(), 2);
    assert_eq!(materialized[0].record, existing.record);
    assert_eq!(materialized[0].runtime_state, existing.runtime_state);
    assert_eq!(materialized[0].payload_type_id, existing.payload_type_id);
    assert_eq!(materialized[1].id, removed_id);
    assert!(materialized[1].record.is_none());
    assert_eq!(
        materialized[1].runtime_state,
        RuntimeResourceState::Unloaded
    );
    assert!(materialized[1].payload_type_id.is_none());
}

#[derive(Clone, Copy, Debug)]
struct ProfileSample {
    elapsed_ns: u64,
    allocations: AllocationSnapshot,
    generation_advanced: bool,
    row_count: usize,
    changed_row_count: usize,
    affected_closure_count: usize,
    edge_visit_count: usize,
}

#[test]
#[ignore = "requires an explicit non-C profile directory and a release single-thread run"]
fn resource_readiness_profile_orchestrator() {
    assert_release_profile();
    let report_directory = profile_report_directory();
    let sample_count = profile_count(PROFILE_SAMPLES_ENV, MIN_PROFILE_SAMPLE_COUNT);
    assert!(sample_count >= MIN_PROFILE_SAMPLE_COUNT);
    let warmup_count = profile_count(PROFILE_WARMUPS_ENV, DEFAULT_PROFILE_WARMUP_COUNT);
    let executable = env::current_exe().expect("resolve current release test executable");
    let status_path = report_directory.join("resource-readiness-current-orchestration.csv");
    let mut status = BufWriter::new(File::create(status_path).expect("create orchestration CSV"));
    writeln!(
        status,
        "scenario,measurement_scope,node_count,topology,operation,status,exit_code,elapsed_ms,raw_report,summary_report,stdout_blake3,stderr_blake3"
    )
    .expect("write orchestration header");

    for scenario in PROFILE_SCENARIOS {
        for measurement_scope in ProfileMeasurementScope::ALL {
            let scenario_directory = report_directory
                .join(scenario.name)
                .join(measurement_scope.as_str());
            fs::create_dir_all(&scenario_directory).expect("create scenario report directory");
            let started = Instant::now();
            let output = Command::new(&executable)
                .arg("readiness_profile_worker")
                .arg("--ignored")
                .arg("--test-threads=1")
                .arg("--nocapture")
                .env(PROFILE_DIRECTORY_ENV, &scenario_directory)
                .env(PROFILE_SCENARIO_ENV, scenario.name)
                .env(PROFILE_SCOPE_ENV, measurement_scope.as_str())
                .env(PROFILE_SAMPLES_ENV, sample_count.to_string())
                .env(PROFILE_WARMUPS_ENV, warmup_count.to_string())
                .output()
                .expect("launch isolated readiness profile worker");
            let elapsed_ms = duration_ms(started.elapsed());
            let stdout_path = scenario_directory.join("worker.stdout.log");
            let stderr_path = scenario_directory.join("worker.stderr.log");
            fs::write(&stdout_path, &output.stdout).expect("write worker stdout");
            fs::write(&stderr_path, &output.stderr).expect("write worker stderr");
            let raw_exists = scenario_directory
                .join("resource-readiness-current-raw-samples.csv")
                .is_file();
            let summary_exists = scenario_directory
                .join("resource-readiness-current-summary.csv")
                .is_file();
            let status_name = if output.status.success() {
                assert!(raw_exists, "successful worker omitted raw report");
                assert!(summary_exists, "successful worker omitted summary report");
                "completed"
            } else {
                "current_algorithm_failed"
            };
            writeln!(
                status,
                "{},{},{},{},{},{},{},{},{},{},{},{}",
                scenario.name,
                measurement_scope.as_str(),
                scenario.node_count,
                scenario.topology.as_str(),
                scenario.operation.as_str(),
                status_name,
                output.status.code().unwrap_or(-1),
                elapsed_ms,
                raw_exists,
                summary_exists,
                blake3::hash(&output.stdout).to_hex(),
                blake3::hash(&output.stderr).to_hex()
            )
            .expect("write orchestration row");
        }
    }
    status.flush().expect("flush orchestration CSV");
    fs::write(
        report_directory.join("resource-readiness-current-orchestration-metadata.txt"),
        format!(
            "schema=zr_resource_readiness_orchestration_v2\nprofile=release\nrequired_test_threads=1\nsamples={sample_count}\nwarmups={warmup_count}\nworker_isolation=one_process_per_scenario_and_measurement_scope\nmeasurement_scopes=manager_end_to_end,evaluator_only\nrss=unavailable\npower=unavailable\n"
        ),
    )
    .expect("write orchestration metadata");
    println!(
        "ZR_RESOURCE_READINESS_PROFILE_REPORT path={}",
        report_directory.display()
    );
}

#[test]
#[ignore = "launched only by resource_readiness_profile_orchestrator"]
fn readiness_profile_worker() {
    assert_release_profile();
    let scenario_name =
        env::var(PROFILE_SCENARIO_ENV).expect("profile worker scenario must be set");
    let scenario = PROFILE_SCENARIOS
        .iter()
        .copied()
        .find(|scenario| scenario.name == scenario_name)
        .expect("known profile worker scenario");
    let measurement_scope = ProfileMeasurementScope::parse(
        &env::var(PROFILE_SCOPE_ENV).expect("profile worker measurement scope must be set"),
    );
    let report_directory = profile_report_directory();
    let sample_count = profile_count(PROFILE_SAMPLES_ENV, MIN_PROFILE_SAMPLE_COUNT);
    assert!(sample_count >= MIN_PROFILE_SAMPLE_COUNT);
    let warmup_count = profile_count(PROFILE_WARMUPS_ENV, DEFAULT_PROFILE_WARMUP_COUNT);
    let prepared = prepare_scenario(scenario);
    for _ in 0..warmup_count {
        let _ = execute_scenario(&prepared, measurement_scope, false);
    }
    let samples = (0..sample_count)
        .map(|_| execute_scenario(&prepared, measurement_scope, true))
        .collect::<Vec<_>>();
    write_profile_reports(
        &report_directory,
        scenario,
        measurement_scope,
        warmup_count,
        &samples,
    );
}

fn assert_release_profile() {
    assert!(
        !cfg!(debug_assertions),
        "profile must run with cargo test --release"
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
        env::var_os(PROFILE_DIRECTORY_ENV).expect("ZR_RESOURCE_READINESS_PROFILE_DIR must be set"),
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

fn prepare_scenario(scenario: ProfileScenario) -> PreparedScenario {
    let records = scenario_records(scenario);
    if matches!(scenario.operation, ProfileOperation::InitialBuild) {
        let updates = records.into_iter().map(source_update).collect::<Vec<_>>();
        return prepared_scenario(scenario, None, updates);
    }

    let mut baseline = ResourceReadinessProjection::default();
    let updates = if matches!(
        scenario.operation,
        ProfileOperation::MissingDependencyArrival
    ) {
        let missing_index = records.len() - 1;
        baseline.apply_updates(records[..missing_index].iter().cloned().map(source_update));
        vec![source_update(records[missing_index].clone())]
    } else {
        baseline.apply_updates(records.iter().cloned().map(source_update));
        match scenario.operation {
            ProfileOperation::LeafReload => {
                let leaf_index = match scenario.topology {
                    ProfileTopology::Chain | ProfileTopology::Diamond => scenario.node_count - 1,
                    ProfileTopology::Fanout => 1,
                    ProfileTopology::Independent | ProfileTopology::Cycle => {
                        unreachable!("leaf reload requires a leaf topology")
                    }
                };
                vec![reload_update(records[leaf_index].clone())]
            }
            ProfileOperation::DenseReload => records.into_iter().map(reload_update).collect(),
            ProfileOperation::NoChange => vec![source_update(records[0].clone())],
            ProfileOperation::RootEdgeReplacement => {
                assert!(records.len() >= 3);
                let mut replacement = records[0].clone();
                replacement.dependency_ids = vec![records[2].id];
                vec![source_update(replacement)]
            }
            ProfileOperation::InitialBuild | ProfileOperation::MissingDependencyArrival => {
                unreachable!()
            }
        }
    };
    prepared_scenario(scenario, Some(baseline), updates)
}

fn prepared_scenario(
    scenario: ProfileScenario,
    baseline: Option<ResourceReadinessProjection>,
    updates: Vec<ResourceReadinessSourceUpdate>,
) -> PreparedScenario {
    let manager_sources = PreparedManagerSources::from_updates(&updates);
    PreparedScenario {
        scenario,
        baseline,
        updates,
        manager_sources,
    }
}

fn scenario_records(scenario: ProfileScenario) -> Vec<ResourceRecord> {
    assert!(scenario.node_count > 0);
    let locators = (0..scenario.node_count)
        .map(|index| {
            ResourceLocator::parse(&format!(
                "res://readiness-profile/{}/{index:06}.asset",
                scenario.name
            ))
            .expect("valid readiness profile locator")
        })
        .collect::<Vec<_>>();
    let ids = locators
        .iter()
        .map(ResourceId::from_locator)
        .collect::<Vec<_>>();
    locators
        .into_iter()
        .enumerate()
        .map(|(index, locator)| {
            let dependencies = match scenario.topology {
                ProfileTopology::Chain => ids.get(index + 1).copied().into_iter().collect(),
                ProfileTopology::Fanout if index == 0 => ids[1..].to_vec(),
                ProfileTopology::Fanout => Vec::new(),
                ProfileTopology::Diamond if index == 0 => ids[1..scenario.node_count - 1].to_vec(),
                ProfileTopology::Diamond if index + 1 < scenario.node_count => {
                    vec![ids[scenario.node_count - 1]]
                }
                ProfileTopology::Diamond => Vec::new(),
                ProfileTopology::Independent => Vec::new(),
                ProfileTopology::Cycle => vec![ids[(index + 1) % scenario.node_count]],
            };
            ResourceRecord::new(ids[index], ResourceKind::Data, locator)
                .with_state(ResourceState::Ready)
                .with_dependency_ids(dependencies)
        })
        .collect()
}

fn reload_update(mut record: ResourceRecord) -> ResourceReadinessSourceUpdate {
    record.state = ResourceState::Reloading;
    ResourceReadinessSourceUpdate {
        id: record.id,
        record: Some(record),
        runtime_state: RuntimeResourceState::Reloading,
        payload_type_id: Some(TypeId::of::<()>()),
    }
}

fn clone_baseline(baseline: &ResourceReadinessProjection) -> ResourceReadinessProjection {
    ResourceReadinessProjection {
        generation: Arc::clone(&baseline.generation),
        sources: baseline.sources.clone(),
        reverse_dependencies: baseline.reverse_dependencies.clone(),
    }
}

fn execute_scenario(
    prepared: &PreparedScenario,
    measurement_scope: ProfileMeasurementScope,
    measured: bool,
) -> ProfileSample {
    let mut projection = prepared
        .baseline
        .as_ref()
        .map(clone_baseline)
        .unwrap_or_default();
    let evaluator_updates = matches!(measurement_scope, ProfileMeasurementScope::EvaluatorOnly)
        .then(|| prepared.updates.clone());
    let previous_publication_count = projection.generation.diagnostics().publication_count;
    if measured {
        begin_allocation_profile();
    }
    let started = Instant::now();
    match measurement_scope {
        ProfileMeasurementScope::ManagerEndToEnd => {
            projection.apply_updates(prepared.manager_sources.materialize_updates());
        }
        ProfileMeasurementScope::EvaluatorOnly => {
            projection.apply_updates(evaluator_updates.expect("prepared evaluator updates"));
        }
    }
    black_box(projection.generation.diagnostics().publication_count);
    let elapsed = started.elapsed();
    let allocations = if measured {
        finish_allocation_profile()
    } else {
        AllocationSnapshot::default()
    };
    let generation_advanced =
        projection.generation.diagnostics().publication_count != previous_publication_count;
    let diagnostics = projection.generation.diagnostics();
    assert_eq!(diagnostics.row_count, prepared.scenario.node_count);
    ProfileSample {
        elapsed_ns: duration_ns(elapsed),
        allocations,
        generation_advanced,
        row_count: diagnostics.row_count,
        changed_row_count: generation_advanced
            .then_some(diagnostics.changed_row_count)
            .unwrap_or(0),
        affected_closure_count: generation_advanced
            .then_some(diagnostics.affected_closure_count)
            .unwrap_or(0),
        edge_visit_count: generation_advanced
            .then_some(diagnostics.edge_visit_count)
            .unwrap_or(0),
    }
}

fn duration_ns(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

fn duration_ms(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
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

fn source_blake3(source: &str) -> String {
    blake3::hash(source.as_bytes()).to_hex().to_string()
}

fn write_profile_reports(
    directory: &Path,
    scenario: ProfileScenario,
    measurement_scope: ProfileMeasurementScope,
    warmup_count: usize,
    samples: &[ProfileSample],
) {
    let projection_source_blake3 = source_blake3(include_str!("../../readiness_projection.rs"));
    let generation_source_blake3 = source_blake3(include_str!("../../../readiness_generation.rs"));
    let profile_source_blake3 = source_blake3(include_str!("profile.rs"));
    let allocation_profile_source_blake3 = source_blake3(include_str!("../../../test_profile.rs"));
    let raw_path = directory.join("resource-readiness-current-raw-samples.csv");
    let summary_path = directory.join("resource-readiness-current-summary.csv");
    let metadata_path = directory.join("resource-readiness-current-metadata.txt");
    let mut raw = BufWriter::new(File::create(raw_path).expect("create raw profile CSV"));
    writeln!(
        raw,
        "scenario,measurement_scope,node_count,topology,operation,sample,elapsed_ns,allocation_count,requested_bytes,peak_live_bytes,generation_advanced,row_count,changed_row_count,affected_closure_count,edge_visit_count"
    )
    .expect("write raw profile header");
    for (index, sample) in samples.iter().enumerate() {
        writeln!(
            raw,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            scenario.name,
            measurement_scope.as_str(),
            scenario.node_count,
            scenario.topology.as_str(),
            scenario.operation.as_str(),
            index,
            sample.elapsed_ns,
            sample.allocations.allocation_count,
            sample.allocations.requested_bytes,
            sample.allocations.peak_live_bytes,
            sample.generation_advanced,
            sample.row_count,
            sample.changed_row_count,
            sample.affected_closure_count,
            sample.edge_visit_count
        )
        .expect("write raw profile row");
    }
    raw.flush().expect("flush raw profile CSV");

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
    let affected = samples
        .iter()
        .map(|sample| sample.affected_closure_count as u64)
        .collect::<Vec<_>>();
    let edges = samples
        .iter()
        .map(|sample| sample.edge_visit_count as u64)
        .collect::<Vec<_>>();
    let p50 = percentile(&elapsed, 50);
    let mut summary = BufWriter::new(File::create(summary_path).expect("create summary CSV"));
    writeln!(
        summary,
        "scenario,measurement_scope,node_count,topology,operation,samples,warmups,p50_ns,p95_ns,mad_ns,allocation_count_p50,requested_bytes_p50,peak_live_bytes_p50,affected_closure_count_p50,edge_visit_count_p50,projection_source_blake3,generation_source_blake3,profile_source_blake3,allocation_profile_source_blake3"
    )
    .expect("write summary header");
    writeln!(
        summary,
        "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
        scenario.name,
        measurement_scope.as_str(),
        scenario.node_count,
        scenario.topology.as_str(),
        scenario.operation.as_str(),
        samples.len(),
        warmup_count,
        p50,
        percentile(&elapsed, 95),
        median_absolute_deviation(&elapsed, p50),
        percentile(&allocation_count, 50),
        percentile(&requested_bytes, 50),
        percentile(&peak_live_bytes, 50),
        percentile(&affected, 50),
        percentile(&edges, 50),
        projection_source_blake3,
        generation_source_blake3,
        profile_source_blake3,
        allocation_profile_source_blake3
    )
    .expect("write summary row");
    summary.flush().expect("flush summary CSV");

    fs::write(
        metadata_path,
        format!(
            "schema=zr_resource_readiness_profile_v2\nprofile=release\nrequired_test_threads=1\nscenario={}\nmeasurement_scope={}\nnode_count={}\ntopology={}\noperation={}\nsamples={}\nwarmups={warmup_count}\nprojection_source_blake3={projection_source_blake3}\ngeneration_source_blake3={generation_source_blake3}\nprofile_source_blake3={profile_source_blake3}\nallocation_profile_source_blake3={allocation_profile_source_blake3}\nqueue_pushes=unavailable\ntouched_publication_shards=unavailable\nrss=unavailable\npower=unavailable\n",
            scenario.name,
            measurement_scope.as_str(),
            scenario.node_count,
            scenario.topology.as_str(),
            scenario.operation.as_str(),
            samples.len()
        ),
    )
    .expect("write profile metadata");
}
