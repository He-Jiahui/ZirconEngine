use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;

const RESOURCE_ADMISSION_COUNT: usize = 65_536;
const UNIQUE_RESOURCE_COUNT: usize = 8_192;
const PREALLOCATED_RESOURCE_COUNT: usize = 32_768;
const SAMPLE_COUNT: usize = 17;

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

fn resource_ids() -> Vec<String> {
    (0..RESOURCE_ADMISSION_COUNT)
        .map(|index| {
            format!(
                "native.generated.resource.with.long.shared.identity.{:05}",
                (index * 4_099) % UNIQUE_RESOURCE_COUNT
            )
        })
        .collect()
}

fn ordered_unique_count(resource_ids: &[String]) -> usize {
    let mut unique = BTreeSet::new();
    resource_ids
        .iter()
        .filter(|resource_id| unique.insert(resource_id.as_str()))
        .count()
}

fn hash_unique_count(resource_ids: &[String]) -> usize {
    let mut unique = HashSet::new();
    resource_ids
        .iter()
        .filter(|resource_id| unique.insert(resource_id.as_str()))
        .count()
}

fn preallocation_resource_ids() -> Vec<String> {
    (0..PREALLOCATED_RESOURCE_COUNT)
        .map(|index| format!("native.generated.resource.preallocated.identity.{index:05}"))
        .collect()
}

fn unreserved_hash_unique_count(resource_ids: &[String]) -> usize {
    let mut unique = HashSet::new();
    resource_ids
        .iter()
        .filter(|resource_id| unique.insert(resource_id.as_str()))
        .count()
}

fn reserved_hash_unique_count(resource_ids: &[String]) -> usize {
    let mut unique = HashSet::with_capacity(resource_ids.len());
    resource_ids
        .iter()
        .filter(|resource_id| unique.insert(resource_id.as_str()))
        .count()
}

#[test]
fn native_registration_manifest_parses_bridge_systems() {
    let manifest = NativePluginRegistrationManifest::from_toml(
        r#"
schema = "zircon.native.registration-manifest/3"
capabilities = ["runtime.plugin.physics"]

[[modules]]
name = "runtime"
kind = "runtime"

[[systems]]
id = "physics.runtime_tick"
module = "runtime"
stage = "Update"
order = 2
sets = ["physics.tick"]
before = ["physics.render"]
after = ["physics.bootstrap"]
access = ["write:world"]
bridge_interface = "native.live_host.bridge.v1"
bridge_method = "sample_count"
"#,
    )
    .expect("native registration manifest should parse");

    assert_eq!(
        manifest.schema,
        ZIRCON_NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3
    );
    assert_eq!(manifest.modules.len(), 1);
    assert_eq!(manifest.systems.len(), 1);
    assert_eq!(manifest.systems[0].stage().unwrap(), SystemStage::Update);
    assert_eq!(
        manifest.systems[0].bridge_interface().unwrap(),
        "native.live_host.bridge.v1"
    );
    let access = manifest.systems[0]
        .access_plan(&manifest.capabilities)
        .unwrap();
    assert_eq!(
        access.affinity(),
        crate::scene::ecs::SceneSystemThreadAffinity::MainThreadOnly
    );
    assert!(access.has_conservative_world_access());
}

#[test]
fn native_registration_manifest_reports_unsupported_stage_with_typed_error() {
    let error = NativePluginRegistrationManifest::from_toml(
        r#"
schema = "zircon.native.registration-manifest/3"

[[systems]]
id = "physics.runtime_tick"
module = "runtime"
stage = "BeforeBreakfast"
bridge_interface = "native.live_host.bridge.v1"
bridge_method = "sample_count"
"#,
    )
    .expect_err("unsupported stage should report typed registration manifest error");

    assert!(matches!(
        error,
        NativePluginRegistrationManifestError::UnsupportedSystemStage { stage }
            if stage == "BeforeBreakfast"
    ));
}

#[test]
fn native_registration_manifest_reports_missing_bridge_method_with_typed_error() {
    let error = NativePluginRegistrationManifest::from_toml(
        r#"
schema = "zircon.native.registration-manifest/3"

[[systems]]
id = "physics.runtime_tick"
module = "runtime"
stage = "Update"
bridge_interface = "native.live_host.bridge.v1"
"#,
    )
    .expect_err("missing bridge method should report typed registration manifest error");

    assert!(matches!(
        error,
        NativePluginRegistrationManifestError::MissingSystemField {
            system_id,
            field_name: "bridge_method"
        } if system_id == "physics.runtime_tick"
    ));
}

#[test]
fn native_registration_manifest_compiles_explicit_worker_access_contract() {
    let manifest = NativePluginRegistrationManifest::from_toml(
        r#"
schema = "zircon.native.registration-manifest/3"
capabilities = ["runtime.plugin.physics", "runtime.native.system.worker_safe"]

[[systems]]
id = "physics.runtime_tick"
module = "runtime"
stage = "Update"
thread_affinity = "worker-safe"
access = ["write:resource:physics.solver", "read:component:physics.Body"]
bridge_interface = "native.live_host.bridge.v1"
bridge_method = "sample_count"
"#,
    )
    .expect("worker-safe access contract should parse");

    let access = manifest.systems[0]
        .access_plan(&manifest.capabilities)
        .unwrap();
    assert_eq!(
        access.affinity(),
        crate::scene::ecs::SceneSystemThreadAffinity::WorkerSafe
    );
    assert!(!access.has_conservative_world_access());
    assert_eq!(
        access.declarations(),
        &[
            NativeSystemAccessDeclaration {
                mode: NativeSystemAccessMode::Read,
                domain: NativeSystemAccessDomain::Component,
                stable_id: "physics.Body".to_string(),
            },
            NativeSystemAccessDeclaration {
                mode: NativeSystemAccessMode::Write,
                domain: NativeSystemAccessDomain::Resource,
                stable_id: "physics.solver".to_string(),
            },
        ]
    );
}

#[test]
fn native_registration_manifest_rejects_ungranted_or_ambiguous_worker_access() {
    let missing_capability = NativePluginRegistrationManifest::from_toml(
        r#"
schema = "zircon.native.registration-manifest/3"

[[systems]]
id = "physics.runtime_tick"
module = "runtime"
stage = "Update"
thread_affinity = "worker-safe"
access = ["read:component:physics.Body"]
bridge_interface = "native.live_host.bridge.v1"
bridge_method = "sample_count"
"#,
    )
    .expect_err("worker-safe declaration without capability must fail");
    assert!(matches!(
        missing_capability,
        NativePluginRegistrationManifestError::InvalidSystemAccess {
            source: NativeSystemAccessContractError::MissingWorkerSafeCapability,
            ..
        }
    ));

    let conflicting = NativePluginRegistrationManifest::from_toml(
        r#"
schema = "zircon.native.registration-manifest/3"

[[systems]]
id = "physics.runtime_tick"
module = "runtime"
stage = "Update"
access = ["read:resource:physics.solver", "write:resource:physics.solver"]
bridge_interface = "native.live_host.bridge.v1"
bridge_method = "sample_count"
"#,
    )
    .expect_err("read/write ambiguity must fail");
    assert!(matches!(
        conflicting,
        NativePluginRegistrationManifestError::InvalidSystemAccess {
            source: NativeSystemAccessContractError::ConflictingAccess {
                domain: NativeSystemAccessDomain::Resource,
                stable_id,
            },
            ..
        } if stable_id == "physics.solver"
    ));
}

#[test]
fn optimization_batch_20260826ad_runtime07_hash_resource_validation_preserves_first_duplicate_error(
) {
    let error = NativePluginRegistrationManifest::from_toml(
        r#"
schema = "zircon.native.registration-manifest/3"

[[resources]]
id = "physics.solver"
module = "runtime"
schema = "physics.solver.v1"

[[resources]]
id = "physics.world"
module = "runtime"
schema = "physics.world.v1"

[[resources]]
id = "physics.solver"
module = "duplicate"
schema = "physics.solver.v2"
"#,
    )
    .expect_err("duplicate resource should fail validation");

    assert!(matches!(
        error,
        NativePluginRegistrationManifestError::DuplicateResourceId { resource_id }
            if resource_id == "physics.solver"
    ));
}

#[test]
fn optimization_batch_20260826ad_runtime07_native_resource_validation_uses_borrowed_hash_set() {
    let source = include_str!("../registration_manifest.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("use std::collections::HashSet;"));
    assert!(
        production.contains("let mut resource_ids = HashSet::with_capacity(self.resources.len());")
    );
    assert!(production.contains("resource_ids.insert(resource.id.as_str())"));
    assert!(!production.contains("BTreeSet"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826ad_runtime07_native_resource_hash_validation_performance_evidence() {
    let resource_ids = resource_ids();
    assert_eq!(
        ordered_unique_count(&resource_ids),
        hash_unique_count(&resource_ids)
    );

    let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(ordered_unique_count(black_box(&resource_ids)));
            ordered_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(hash_unique_count(black_box(&resource_ids)));
            hash_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(hash_unique_count(black_box(&resource_ids)));
            hash_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(ordered_unique_count(black_box(&resource_ids)));
            ordered_samples.push(started.elapsed());
        }
    }

    let ordered_p95 = percentile_95(&mut ordered_samples);
    let hash_p95 = percentile_95(&mut hash_samples);
    println!(
        "RUNTIME07_NATIVE_RESOURCE_HASH_VALIDATION_BENCH_V1 \
             admissions={RESOURCE_ADMISSION_COUNT} unique_resources={UNIQUE_RESOURCE_COUNT} \
             borrowed_identity=true ordered_p95_ns={} hash_p95_ns={}",
        ordered_p95.as_nanos(),
        hash_p95.as_nanos(),
    );
    assert!(
        hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 60,
        "hash-validation P95 {:?} exceeded 60% of ordered-validation P95 {:?}",
        hash_p95,
        ordered_p95,
    );
}

#[test]
fn optimization_batch_ih_runtime618_native_resource_validation_preallocates_hash_storage() {
    let source = include_str!("../registration_manifest.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(
        production.contains("let mut resource_ids = HashSet::with_capacity(self.resources.len());")
    );
    assert!(production.contains("resource_ids.insert(resource.id.as_str())"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_ih_runtime618_preallocated_native_resource_validation_performance_evidence() {
    let resource_ids = preallocation_resource_ids();
    assert_eq!(
        unreserved_hash_unique_count(&resource_ids),
        reserved_hash_unique_count(&resource_ids)
    );

    black_box(unreserved_hash_unique_count(black_box(&resource_ids)));
    black_box(reserved_hash_unique_count(black_box(&resource_ids)));

    let mut unreserved_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut reserved_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(unreserved_hash_unique_count(black_box(&resource_ids)));
            unreserved_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(reserved_hash_unique_count(black_box(&resource_ids)));
            reserved_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(reserved_hash_unique_count(black_box(&resource_ids)));
            reserved_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(unreserved_hash_unique_count(black_box(&resource_ids)));
            unreserved_samples.push(started.elapsed());
        }
    }

    let unreserved_p95 = percentile_95(&mut unreserved_samples);
    let reserved_p95 = percentile_95(&mut reserved_samples);
    println!(
        "RUNTIME618_PREALLOCATED_NATIVE_RESOURCE_BENCH_V1 \
             resources={PREALLOCATED_RESOURCE_COUNT} borrowed_identity=true \
             unreserved_p95_ns={} reserved_p95_ns={}",
        unreserved_p95.as_nanos(),
        reserved_p95.as_nanos(),
    );
    assert!(
        reserved_p95.as_nanos() * 100 <= unreserved_p95.as_nanos() * 85,
        "preallocated validation P95 {:?} exceeded 85% of unreserved P95 {:?}",
        reserved_p95,
        unreserved_p95,
    );
}
