use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::time::Instant;

use super::DesktopExportWizardSessions;
use crate::core::jobs::test_job_system;
use crate::ui::host::{ExportWizardPanelAction, ExportWizardPipelineOptions};
use zircon_runtime::core::framework::project::ExportPackagingStrategy;

const ENTRY_COUNT: usize = 4_096;
const HIT_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;

#[test]
fn optimization_batch_20260826ca_export_wizard_session_hash_index_isolates_profiles() {
    let mut sessions = DesktopExportWizardSessions::new(test_job_system());
    for profile_name in ["profile_z", "profile_a", "profile_m"] {
        sessions
            .dispatch_profile_action(
                profile_name,
                ExportWizardPanelAction::GeneratePlan,
                Some(ready_options(profile_name)),
            )
            .expect("profile plan should create an isolated session");
    }

    for profile_name in ["profile_a", "profile_m", "profile_z"] {
        assert_eq!(
            sessions
                .view_model(profile_name)
                .expect("profile session should exist")
                .snapshot()
                .profile
                .as_str(),
            profile_name
        );
    }
    assert!(sessions.view_model("profile_missing").is_none());
}

#[test]
fn optimization_batch_20260826ca_export_wizard_session_hash_index_preserves_poll_order() {
    let owner_source = include_str!("../session_state.rs");
    let polling_source = include_str!("polling.rs");

    assert!(owner_source.contains("use std::collections::HashMap;"));
    assert!(owner_source.contains("sessions: HashMap<String, ExportWizardPanelSession>"));
    assert!(!owner_source.contains("BTreeMap"));
    assert!(polling_source.contains("for (profile_name, session) in &mut self.sessions"));
    assert!(polling_source.contains("updates.sort_unstable_by"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826ca_export_wizard_session_hash_index_p95() {
    let profile_names = (0..ENTRY_COUNT)
        .map(|index| format!("desktop.export.shared.profile.namespace.{index:04}"))
        .collect::<Vec<_>>();
    let ordered = profile_names
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, profile_name)| (profile_name, index + 1))
        .collect::<BTreeMap<_, _>>();
    let hashed = ordered
        .iter()
        .map(|(profile_name, value)| (profile_name.clone(), *value))
        .collect::<HashMap<_, _>>();
    let target = profile_names.last().unwrap().as_str();

    let mut ordered_lookup = || repeated_lookup(&ordered, target);
    let mut hash_lookup = || repeated_lookup(&hashed, target);
    assert_eq!(black_box(ordered_lookup()), black_box(hash_lookup()));

    let mut ordered_ns = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_ns = Vec::with_capacity(SAMPLE_COUNT);
    for sample_index in 0..SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            ordered_ns.push(measure_ns(&mut ordered_lookup));
            hash_ns.push(measure_ns(&mut hash_lookup));
        } else {
            hash_ns.push(measure_ns(&mut hash_lookup));
            ordered_ns.push(measure_ns(&mut ordered_lookup));
        }
    }

    let ordered_p50 = nearest_rank(&ordered_ns, 50);
    let ordered_p95 = nearest_rank(&ordered_ns, 95);
    let hash_p50 = nearest_rank(&hash_ns, 50);
    let hash_p95 = nearest_rank(&hash_ns, 95);
    assert!(
        hash_p95.saturating_mul(10) <= ordered_p95.saturating_mul(7),
        "export-wizard session hash lookup P95 must be at least 30% below BTreeMap: ordered={ordered_p95}ns hash={hash_p95}ns"
    );

    println!(
        "EDITOR09_EXPORT_WIZARD_SESSION_HASH_INDEX_BENCH_V1 entries={ENTRY_COUNT} hits={HIT_COUNT} samples={SAMPLE_COUNT} ordered_p50_ns={ordered_p50} ordered_p95_ns={ordered_p95} hash_p50_ns={hash_p50} hash_p95_ns={hash_p95} ordered_lookups_before={HIT_COUNT} ordered_lookups_after=0 hash_lookups_after={HIT_COUNT} ordered_ns={} hash_ns={}",
        join_samples(&ordered_ns),
        join_samples(&hash_ns),
    );
}

fn ready_options(profile_name: &str) -> ExportWizardPipelineOptions {
    let out = format!("D:\\zircon-export\\{profile_name}");
    let mut options = ExportWizardPipelineOptions::for_test_profile(
        profile_name,
        "zircon-project.toml",
        out.clone(),
    )
    .with_strategies([
        ExportPackagingStrategy::SourceTemplate,
        ExportPackagingStrategy::LibraryEmbed,
        ExportPackagingStrategy::NativeDynamic,
    ]);
    options.source_asset_manifest = Some(format!("{out}\\assets\\assets.json"));
    options.host_executable = Some(format!("{out}\\host\\zircon_game.exe"));
    options
}

fn repeated_lookup<V>(map: &V, target: &str) -> usize
where
    V: Lookup,
{
    let mut total = 0_usize;
    for _ in 0..HIT_COUNT {
        total = total.wrapping_add(black_box(map.lookup(black_box(target))).unwrap_or_default());
    }
    total
}

trait Lookup {
    fn lookup(&self, key: &str) -> Option<usize>;
}

impl Lookup for BTreeMap<String, usize> {
    fn lookup(&self, key: &str) -> Option<usize> {
        self.get(key).copied()
    }
}

impl Lookup for HashMap<String, usize> {
    fn lookup(&self, key: &str) -> Option<usize> {
        self.get(key).copied()
    }
}

fn measure_ns(operation: &mut impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    assert_ne!(black_box(operation()), 0);
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
