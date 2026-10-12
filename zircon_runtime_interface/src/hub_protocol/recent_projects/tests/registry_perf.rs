use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use crate::project::{ProjectManifestSummary, PROJECT_MANIFEST_FORMAT_VERSION};

use super::{
    hub_recent_project_path_key, merge_hub_recent_projects, HubProtocolVersionV1,
    HubRecentProjectV1, HubRecentProjectsError, HubRecentProjectsV1, HUB_RECENT_PROJECT_LIMIT_V1,
};

const PERF_NAME_BYTES: usize = 16 * 1024;
const PERF_SAMPLE_PAIRS: usize = 21;
const PERF_VALIDATIONS_PER_SAMPLE: usize = 128;

#[test]
fn single_pass_validation_preserves_duplicate_error_precedence() {
    let registry = HubRecentProjectsV1 {
        protocol_version: HubProtocolVersionV1,
        revision: 1,
        projects: vec![
            recent_project("Older", "E:/Projects/Shared", 1),
            recent_project("Newer", "E:/Projects/Other", 2),
            recent_project("Duplicate", "e:\\Projects\\Shared\\", 0),
        ],
        tombstones: Vec::new(),
    };

    assert_eq!(
        registry.validate(),
        Err(HubRecentProjectsError::DuplicateProjectPath {
            path_key: "e:/projects/shared".to_string(),
        })
    );
}

#[test]
fn removal_persists_a_tombstone_and_a_later_open_advances_its_logical_clock() {
    let mut registry = HubRecentProjectsV1::default();
    registry
        .record(recent_project("Game", "E:/Projects/Game", 100))
        .unwrap();
    registry.remove("e:/projects/game/").unwrap();

    assert_eq!(registry.revision(), 2);
    assert!(registry.projects.is_empty());
    assert_eq!(registry.tombstones[0].path_key(), "e:/projects/game");
    assert_eq!(registry.tombstones[0].deleted_logical_unix_ms(), 101);

    registry
        .record(recent_project("Game", "E:/Projects/Game", 1))
        .unwrap();

    assert_eq!(registry.revision(), 3);
    assert!(registry.tombstones.is_empty());
    assert_eq!(registry.projects[0].last_opened_unix_ms, 102);
    assert!(registry.validate().is_ok());
}

#[test]
#[ignore = "release performance evidence"]
fn canonical_validation_borrows_registry_entries() {
    let registry = canonical_fixture();
    assert!(legacy_is_valid(&registry));
    assert!(registry.validate().is_ok());
    let mut legacy_ns = Vec::with_capacity(PERF_SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(PERF_SAMPLE_PAIRS);

    for sample in 0..PERF_SAMPLE_PAIRS {
        let (legacy, optimized) = if sample % 2 == 0 {
            (
                measure_batch(|| legacy_is_valid(&registry)),
                measure_batch(|| registry.validate().is_ok()),
            )
        } else {
            let optimized = measure_batch(|| registry.validate().is_ok());
            let legacy = measure_batch(|| legacy_is_valid(&registry));
            (legacy, optimized)
        };
        legacy_ns.push(legacy);
        optimized_ns.push(optimized);
    }

    let legacy_p50 = percentile(&legacy_ns, 50);
    let legacy_p95 = percentile(&legacy_ns, 95);
    let optimized_p50 = percentile(&optimized_ns, 50);
    let optimized_p95 = percentile(&optimized_ns, 95);
    println!(
        "PERF_RESULT runtime_interface06_recent_registry_validation legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} entries={HUB_RECENT_PROJECT_LIMIT_V1} name_bytes_per_entry={PERF_NAME_BYTES} samples={PERF_SAMPLE_PAIRS} validations_per_sample={PERF_VALIDATIONS_PER_SAMPLE} legacy_entry_clones=8 optimized_entry_clones=0 legacy_path_normalizations=16 optimized_path_normalizations=8 legacy_accepted_path_key_clones=8 optimized_accepted_path_key_clones=0 legacy_entry_visits=16 optimized_entry_visits=8"
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(25),
        "optimized P95 {optimized_p95}ns must be at most 25% of legacy P95 {legacy_p95}ns"
    );
}

fn canonical_fixture() -> HubRecentProjectsV1 {
    HubRecentProjectsV1::new((0..HUB_RECENT_PROJECT_LIMIT_V1).map(|index| {
        HubRecentProjectV1::new(
            ProjectManifestSummary {
                name: format!("project-{index}-{}", "x".repeat(PERF_NAME_BYTES)),
                engine_version_req: None,
                template_receipt: None,
                default_scene: "res://scenes/main.scene.toml".to_string(),
                format_version: PROJECT_MANIFEST_FORMAT_VERSION,
                project_guid: None,
            },
            format!("E:/Projects/Project-{index}"),
            (HUB_RECENT_PROJECT_LIMIT_V1 - index) as u64,
        )
        .expect("performance fixture project must be valid")
    }))
}

fn recent_project(name: &str, path: &str, last_opened_unix_ms: u64) -> HubRecentProjectV1 {
    HubRecentProjectV1::new(
        ProjectManifestSummary {
            name: name.to_string(),
            engine_version_req: None,
            template_receipt: None,
            default_scene: "res://scenes/main.scene.toml".to_string(),
            format_version: PROJECT_MANIFEST_FORMAT_VERSION,
            project_guid: None,
        },
        path,
        last_opened_unix_ms,
    )
    .expect("recent project fixture must be valid")
}

fn legacy_is_valid(registry: &HubRecentProjectsV1) -> bool {
    let mut keys = BTreeMap::new();
    for project in &registry.projects {
        if project.validate().is_err() {
            return false;
        }
        let key = hub_recent_project_path_key(&project.path);
        if keys.insert(key.clone(), ()).is_some() {
            return false;
        }
    }
    merge_hub_recent_projects(registry.projects.iter().cloned(), []) == registry.projects
}

fn measure_batch(mut measure: impl FnMut() -> bool) -> u64 {
    let started = Instant::now();
    for _ in 0..PERF_VALIDATIONS_PER_SAMPLE {
        black_box(measure());
    }
    (started.elapsed().as_nanos() / PERF_VALIDATIONS_PER_SAMPLE as u128) as u64
}

fn percentile(samples: &[u64], percentile: usize) -> u64 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered
        .len()
        .saturating_mul(percentile)
        .div_ceil(100)
        .saturating_sub(1);
    ordered[rank]
}
