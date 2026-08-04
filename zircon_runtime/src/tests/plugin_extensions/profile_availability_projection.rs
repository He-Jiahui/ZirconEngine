use std::collections::HashSet;
use std::time::Instant;

use crate::builtin::RuntimePluginId;
use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::{
    ExportPackagingStrategy, ProjectPluginManifest, ProjectPluginSelection, RuntimeProfileId,
};
use crate::plugin::{
    PluginMaturity, RuntimePluginCatalog, RuntimePluginDescriptor,
    RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
    RuntimeProfileDescriptor,
};

#[test]
fn availability_projection_runtime_profile_lookup_does_not_build_full_catalog() {
    let source = include_str!("../../plugin/runtime_profile/assembly_presets.rs");
    let for_id = source
        .split_once("pub fn for_id")
        .expect("runtime profile lookup should exist")
        .1
        .split_once("pub fn builtin_profiles")
        .expect("builtin profile catalog should follow lookup")
        .0;

    assert!(
        !for_id.contains("builtin_profiles"),
        "single-profile lookup must construct only the requested profile"
    );
}

#[test]
fn availability_projection_uses_one_manifest_selection_builder() {
    let source = include_str!("../../plugin/runtime_profile/availability_projection.rs");
    assert_eq!(
        source
            .matches("for selection in manifest.enabled_for_target")
            .count(),
        1,
        "production reports and test metrics must share one selection builder"
    );
}

#[test]
fn availability_generation_uses_dedicated_owner_module() {
    let projection = include_str!("../../plugin/runtime_profile/availability_projection.rs");
    let generation =
        include_str!("../../plugin/runtime_profile/availability_projection/generation.rs");

    assert!(projection.contains("mod generation;"));
    assert!(projection.contains("pub use generation::{"));
    assert!(!projection.contains("pub struct RuntimePluginAvailabilityGeneration"));
    assert_eq!(
        generation
            .matches("pub struct RuntimePluginAvailability")
            .count(),
        3,
        "generation, row, and summary types must share one owner module"
    );
    assert!(
        projection.lines().count() < 800,
        "projection orchestration must stay below the owner soft budget"
    );
}

#[test]
fn availability_projection_borrows_indexed_descriptor_rows() {
    let source = include_str!("../../plugin/runtime_profile/availability_projection.rs");
    assert!(source.contains("self.descriptors.get(&plugin_id)"));
    assert!(!source.contains("self.descriptors.get(&plugin_id).cloned()"));
}

#[test]
fn availability_projection_membership_steps_scale_linearly() {
    for row_count in [1usize, 100, 1_000] {
        let descriptors = (0..row_count).map(descriptor).collect::<Vec<_>>();
        let registrations = descriptors
            .iter()
            .enumerate()
            .map(|(index, descriptor)| {
                let mut registration = RuntimePluginRegistrationReport::from_plugin(descriptor);
                registration.project_selection.packaging = if index % 2 == 0 {
                    ExportPackagingStrategy::LibraryEmbed
                } else {
                    ExportPackagingStrategy::NativeDynamic
                };
                registration
            })
            .collect::<Vec<_>>();
        let profile = RuntimeProfileDescriptor::new(
            RuntimeProfileId::Dev,
            "registration projection scaling",
            RuntimeTargetMode::ClientRuntime,
        )
        .with_minimum_maturity(PluginMaturity::Beta);
        let manifest = ProjectPluginManifest {
            selections: (0..row_count)
                .map(|index| {
                    ProjectPluginSelection::runtime_plugin(
                        RuntimePluginId::new(format!("profile_perf_{index}")),
                        true,
                        false,
                    )
                })
                .collect(),
        };

        let linked_membership = (0..row_count)
            .map(|index| format!("profile_perf_{index}"))
            .collect::<HashSet<_>>();
        let (linked_report, linked_metrics) = profile
            .availability_report_for_manifest_with_linked_membership_and_metrics(
                descriptors.iter(),
                &manifest,
                &linked_membership,
            );

        let started = Instant::now();
        let (report, metrics) = profile
            .availability_report_for_manifest_and_registration_reports_with_metrics(
                descriptors.iter(),
                &manifest,
                registrations.iter(),
            );
        let elapsed = started.elapsed();

        assert_eq!(metrics.descriptor_rows, row_count);
        assert_eq!(metrics.linked_provider_rows, row_count.div_ceil(2));
        assert_eq!(metrics.native_dynamic_provider_rows, row_count / 2);
        assert_eq!(metrics.membership_build_steps(), row_count * 2);
        assert_eq!(report.linked.len(), row_count.div_ceil(2));
        assert_eq!(report.native_dynamic.len(), row_count / 2);
        assert_eq!(linked_metrics.descriptor_rows, row_count);
        assert_eq!(linked_metrics.linked_provider_rows, row_count);
        assert_eq!(linked_metrics.native_dynamic_provider_rows, 0);
        assert_eq!(linked_metrics.membership_build_steps(), row_count * 2);
        assert_eq!(linked_report.linked.len(), row_count);
        eprintln!(
            "runtime_profile_availability_projection rows={row_count} steps={} elapsed_us={}",
            metrics.membership_build_steps(),
            elapsed.as_micros()
        );
    }
}

#[test]
fn availability_projection_builtin_catalog_registration_report_is_byte_equivalent() {
    let descriptors = RuntimePluginDescriptor::builtin_catalog();
    let catalog = RuntimePluginCatalog::from_descriptors(descriptors.clone());
    let profile = RuntimeProfileDescriptor::for_id(RuntimeProfileId::Client2d);
    let direct = profile.availability_report_with_providers(
        descriptors.iter(),
        std::iter::empty::<String>(),
        std::iter::empty::<String>(),
    );
    let linked = HashSet::new();
    let from_catalog = profile.availability_report_for_catalog_with_provider_membership(
        &catalog,
        &linked,
        std::iter::empty::<&str>(),
    );

    assert_eq!(
        serde_json::to_vec(&from_catalog).expect("catalog availability should serialize"),
        serde_json::to_vec(&direct).expect("descriptor availability should serialize")
    );
}

#[test]
fn availability_projection_preserves_first_selection_order_and_required_or() {
    let descriptors = [descriptor(0), descriptor(1)];
    let profile = RuntimeProfileDescriptor::new(
        RuntimeProfileId::Dev,
        "projection parity",
        RuntimeTargetMode::ClientRuntime,
    )
    .with_minimum_maturity(PluginMaturity::Beta);
    let manifest = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(
                RuntimePluginId::new("profile_perf_0"),
                true,
                false,
            ),
            ProjectPluginSelection::runtime_plugin(
                RuntimePluginId::new("profile_perf_1"),
                true,
                true,
            ),
            ProjectPluginSelection::runtime_plugin(
                RuntimePluginId::new("profile_perf_0"),
                true,
                true,
            ),
        ],
    };

    let report = profile.availability_report_for_manifest_with_providers(
        descriptors.iter(),
        &manifest,
        ["profile_perf_0"],
        ["profile_perf_1"],
    );

    assert_eq!(report.linked.len(), 1);
    assert_eq!(report.linked[0].id, "profile_perf_0");
    assert!(report.linked[0].required);
    assert_eq!(report.native_dynamic.len(), 1);
    assert_eq!(report.native_dynamic[0].id, "profile_perf_1");
    assert!(report.native_dynamic[0].required);
    assert!(report.missing_required.is_empty());

    let bytes = serde_json::to_vec(&report).expect("availability report should serialize");
    let expected = concat!(
        r#"{"available":[],"linked":[{"id":"profile_perf_0","runtime_id":"profile_perf_0","#,
        r#""required":true,"maturity":"beta","reason":"plugin runtime was supplied by linked registration"}],"#,
        r#""native_dynamic":[{"id":"profile_perf_1","runtime_id":"profile_perf_1","required":true,"#,
        r#""maturity":"beta","reason":"plugin runtime was supplied by native dynamic registration"}],"#,
        r#""externalized_missing":[],"stub":[],"blocked_by_target":[],"blocked_by_maturity":[],"#,
        r#""missing_required":[]}"#,
    );
    assert_eq!(bytes.as_slice(), expected.as_bytes());
}

#[test]
fn availability_projection_manifest_selection_index_steps_scale_linearly() {
    for row_count in [1usize, 100, 1_000] {
        let descriptors = (0..row_count).map(descriptor).collect::<Vec<_>>();
        let linked = (0..row_count)
            .map(|index| format!("profile_perf_{index}"))
            .collect::<Vec<_>>();
        let manifest = ProjectPluginManifest {
            selections: (0..row_count)
                .map(|index| {
                    ProjectPluginSelection::runtime_plugin(
                        RuntimePluginId::new(format!("profile_perf_{index}")),
                        true,
                        index % 2 == 0,
                    )
                })
                .collect(),
        };
        let profile = RuntimeProfileDescriptor::new(
            RuntimeProfileId::Dev,
            "projection selection scaling",
            RuntimeTargetMode::ClientRuntime,
        )
        .with_minimum_maturity(PluginMaturity::Beta);

        let started = Instant::now();
        let (report, metrics) = profile
            .availability_report_for_manifest_with_providers_and_metrics(
                descriptors.iter(),
                &manifest,
                linked.iter(),
                std::iter::empty::<String>(),
            );
        let elapsed = started.elapsed();

        assert_eq!(metrics.manifest_selection_rows, row_count);
        assert_eq!(metrics.indexed_lookup_rows, row_count);
        assert_eq!(metrics.unique_plugin_rows, row_count);
        assert_eq!(metrics.duplicate_merge_rows, 0);
        assert_eq!(metrics.selection_build_steps(), row_count);
        assert_eq!(report.linked.len(), row_count);
        eprintln!(
            "runtime_profile_manifest_selection rows={row_count} steps={} elapsed_us={}",
            metrics.selection_build_steps(),
            elapsed.as_micros()
        );
    }
}

#[test]
fn availability_projection_registration_reports_filter_and_deduplicate_providers() {
    let descriptors = (0..4).map(descriptor).collect::<Vec<_>>();
    let mut linked = RuntimePluginRegistrationReport::from_plugin(&descriptors[0]);
    linked.project_selection.packaging = ExportPackagingStrategy::LibraryEmbed;
    let linked_duplicate = linked.clone();
    let mut native = RuntimePluginRegistrationReport::from_plugin(&descriptors[1]);
    native.project_selection.packaging = ExportPackagingStrategy::NativeDynamic;
    let mut wrong_target = RuntimePluginRegistrationReport::from_plugin(&descriptors[2]);
    wrong_target.project_selection.target_modes = vec![RuntimeTargetMode::ServerRuntime];
    let mut disabled = RuntimePluginRegistrationReport::from_plugin(&descriptors[3]);
    disabled.project_selection.enabled = false;
    let registrations = [linked, linked_duplicate, native, wrong_target, disabled];
    let profile = RuntimeProfileDescriptor::new(
        RuntimeProfileId::Dev,
        "registration projection",
        RuntimeTargetMode::ClientRuntime,
    )
    .with_minimum_maturity(PluginMaturity::Beta)
    .allow_externalized_required_plugins(true);
    let manifest = ProjectPluginManifest {
        selections: (0..4)
            .map(|index| {
                ProjectPluginSelection::runtime_plugin(
                    RuntimePluginId::new(format!("profile_perf_{index}")),
                    true,
                    false,
                )
            })
            .collect(),
    };

    let (report, metrics) = profile
        .availability_report_for_manifest_and_registration_reports_with_metrics(
            descriptors.iter(),
            &manifest,
            registrations.iter(),
        );

    assert_eq!(metrics.descriptor_rows, 4);
    assert_eq!(metrics.linked_provider_rows, 2);
    assert_eq!(metrics.native_dynamic_provider_rows, 1);
    assert_eq!(report.linked.len(), 1);
    assert_eq!(report.linked[0].id, "profile_perf_0");
    assert_eq!(report.native_dynamic.len(), 1);
    assert_eq!(report.native_dynamic[0].id, "profile_perf_1");
    assert_eq!(
        report
            .externalized_missing
            .iter()
            .map(|entry| entry.id.as_str())
            .collect::<Vec<_>>(),
        ["profile_perf_2", "profile_perf_3"]
    );
}

#[test]
fn availability_projection_profile_feature_path_preserves_manifest_bytes() {
    let manifest = ProjectPluginManifest::default();
    let without_features =
        crate::builtin::runtime_modules_for_runtime_profile_manifest_with_plugin_registration_reports(
            RuntimeProfileId::Client2d,
            &manifest,
            std::iter::empty::<&RuntimePluginRegistrationReport>(),
        );
    let with_features = crate::builtin::runtime_modules_for_runtime_profile_manifest_with_plugin_and_feature_registration_reports(
        RuntimeProfileId::Client2d,
        &manifest,
        std::iter::empty::<&RuntimePluginRegistrationReport>(),
        std::iter::empty::<&RuntimePluginFeatureRegistrationReport>(),
    );

    assert_eq!(
        serde_json::to_vec(&with_features.runtime_plugin_availability)
            .expect("feature-path availability should serialize"),
        serde_json::to_vec(&without_features.runtime_plugin_availability)
            .expect("plugin-path availability should serialize")
    );
}

#[test]
fn availability_generation_shares_required_rows_and_materializes_report_bytes() {
    let descriptors = [descriptor(0)];
    let profile = RuntimeProfileDescriptor::new(
        RuntimeProfileId::Dev,
        "availability generation",
        RuntimeTargetMode::ClientRuntime,
    )
    .with_minimum_maturity(PluginMaturity::Beta);
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::new("profile_perf_0"),
            true,
            false,
        )],
    };

    let generation: crate::plugin::RuntimePluginAvailabilityGeneration<'_> = profile
        .availability_generation_for_manifest_with_providers(
            descriptors.iter(),
            &manifest,
            std::iter::empty::<&str>(),
            std::iter::empty::<&str>(),
        );
    let externalized = generation
        .entries(crate::plugin::RuntimePluginAvailabilityCategory::ExternalizedMissing)
        .next()
        .expect("required provider failure belongs to its primary category");
    let missing = generation
        .entries(crate::plugin::RuntimePluginAvailabilityCategory::MissingRequired)
        .next()
        .expect("required provider failure appears in missing-required index");

    assert!(std::ptr::eq(externalized, missing));
    assert_eq!(externalized.id(), "profile_perf_0");
    assert_eq!(
        externalized.category(),
        crate::plugin::RuntimePluginAvailabilityCategory::ExternalizedMissing
    );
    assert_eq!(
        externalized.detail().reason,
        "plugin runtime has no linked or native dynamic provider registration"
    );
    let summary: crate::plugin::RuntimePluginAvailabilitySummary = generation.summary();
    assert_eq!(summary.row_count(), 1);
    assert_eq!(
        summary
            .category_count(crate::plugin::RuntimePluginAvailabilityCategory::ExternalizedMissing),
        1
    );
    assert_eq!(summary.missing_required_count(), 1);
    let indexed = generation
        .row_for(RuntimePluginId::new("profile_perf_0"))
        .expect("generation provides indexed lookup");
    assert!(std::ptr::eq(indexed, externalized));

    let materialized = generation.materialize_report();
    let direct = profile.availability_report_for_manifest_with_providers(
        descriptors.iter(),
        &manifest,
        std::iter::empty::<&str>(),
        std::iter::empty::<&str>(),
    );
    assert_eq!(
        serde_json::to_vec(&materialized).expect("generation report serializes"),
        serde_json::to_vec(&direct).expect("direct report serializes"),
    );
}

fn descriptor(index: usize) -> RuntimePluginDescriptor {
    let id = format!("profile_perf_{index}");
    RuntimePluginDescriptor::builder(
        id.clone(),
        format!("Profile Perf {index}"),
        RuntimePluginId::new(&id),
        format!("zircon_plugin_profile_perf_{index}_runtime"),
    )
    .with_target_modes([RuntimeTargetMode::ClientRuntime])
    .with_maturity(PluginMaturity::Beta)
    .build()
}
