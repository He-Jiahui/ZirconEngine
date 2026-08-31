use std::collections::HashSet;
use std::sync::Arc;

use crate::builtin::{
    manifest_with_mode_baseline, RuntimeModuleCompositionCompiler, RuntimeModuleCompositionPlan,
};
use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::ProjectPluginManifest;
use crate::plugin::{
    CompiledProjectPluginPlan, RuntimePluginCatalog, RuntimePluginCatalogSnapshot,
    RuntimePluginRegistrationReport,
};

use super::error::{RuntimeDynamicSessionError, RuntimeDynamicSessionResult};

pub(super) struct LinkedRuntimePluginPlan {
    modules: RuntimeModuleCompositionPlan,
    runtime_plugin_catalog_snapshot: Arc<RuntimePluginCatalogSnapshot>,
    compiled_project_plugin_plan: Arc<CompiledProjectPluginPlan>,
}

impl LinkedRuntimePluginPlan {
    pub(super) fn prepare(
        registrations: &[RuntimePluginRegistrationReport],
        project_manifest: Option<&ProjectPluginManifest>,
        target_mode: RuntimeTargetMode,
    ) -> RuntimeDynamicSessionResult<Self> {
        let mut effective_manifest = manifest_with_mode_baseline(target_mode, project_manifest);
        let mut selected_plugin_ids = effective_manifest
            .selections
            .iter()
            .map(|selection| selection.id.as_str())
            .collect::<HashSet<_>>();
        let mut additional_selections = Vec::with_capacity(registrations.len());
        for registration in registrations {
            if admit_linked_plugin_id(
                &mut selected_plugin_ids,
                registration.project_selection.id.as_str(),
            ) {
                additional_selections.push(registration.project_selection.clone());
            }
        }
        drop(selected_plugin_ids);
        effective_manifest.selections.extend(additional_selections);

        let catalog = RuntimePluginCatalog::from_registration_reports(
            registrations.iter().cloned(),
            std::iter::empty(),
        );
        let compiled_project_plugin_plan =
            catalog.compiled_project_plan(&effective_manifest, target_mode);
        let package_ids = compiled_project_plugin_plan
            .linked_provider_package_ids()
            .iter()
            .chain(compiled_project_plugin_plan.native_dynamic_provider_package_ids())
            .cloned()
            .collect::<HashSet<_>>();
        let mut compiler = RuntimeModuleCompositionCompiler::new(&compiled_project_plugin_plan);
        if !package_ids.contains("navigation") {
            compiler =
                compiler.with_host_module(Arc::new(crate::navigation::BuiltinNavigationModule));
        }
        if !package_ids.contains("animation") {
            compiler = compiler.with_host_module(Arc::new(crate::animation::AnimationModule));
        }
        let modules = compiler.compile().map_err(|rejection| {
            RuntimeDynamicSessionError::ModuleDiscovery {
                message: rejection.to_string(),
            }
        })?;
        let runtime_plugin_catalog_snapshot =
            Arc::new(RuntimePluginCatalogSnapshot::from_catalog(catalog));
        debug_assert_eq!(
            runtime_plugin_catalog_snapshot.generation(),
            compiled_project_plugin_plan.catalog_generation()
        );

        Ok(Self {
            modules,
            runtime_plugin_catalog_snapshot,
            compiled_project_plugin_plan,
        })
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        RuntimeModuleCompositionPlan,
        Arc<RuntimePluginCatalogSnapshot>,
        Arc<CompiledProjectPluginPlan>,
    ) {
        (
            self.modules,
            self.runtime_plugin_catalog_snapshot,
            self.compiled_project_plugin_plan,
        )
    }
}

fn admit_linked_plugin_id<'a>(seen: &mut HashSet<&'a str>, plugin_id: &'a str) -> bool {
    seen.insert(plugin_id)
}

#[cfg(test)]
mod optimization_tests {
    use std::collections::HashSet;
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use super::*;
    use crate::builtin::RuntimePluginId;
    use crate::core::framework::project::ProjectPluginSelection;
    use crate::core::ModuleDescriptor;
    use crate::plugin::{PluginModuleManifest, PluginPackageManifest, RuntimeExtensionRegistry};

    const REGISTRATION_ADMISSION_COUNT: usize = 16_384;
    const UNIQUE_SELECTION_COUNT: usize = 2_048;
    const SAMPLE_COUNT: usize = 17;

    const LINKED_PLAN_MODULE: &str = "runtime136.dynamic.linked";

    fn linked_registration_with_module() -> RuntimePluginRegistrationReport {
        let descriptor = ModuleDescriptor::new(LINKED_PLAN_MODULE, "Dynamic linked runtime");
        let mut extensions = RuntimeExtensionRegistry::default();
        extensions
            .register_module(descriptor)
            .expect("linked module fixture registers");
        RuntimePluginRegistrationReport {
            package_manifest: PluginPackageManifest::new("sound", "Sound").with_runtime_module(
                PluginModuleManifest::runtime(LINKED_PLAN_MODULE, "zircon_plugin_sound_runtime"),
            ),
            project_selection: ProjectPluginSelection::runtime_plugin(
                RuntimePluginId::Sound,
                true,
                true,
            ),
            extensions,
            diagnostics: Vec::new(),
        }
    }

    #[test]
    fn linked_runtime_plugin_plan_materializes_each_selected_module_once() {
        let plan = LinkedRuntimePluginPlan::prepare(
            &[linked_registration_with_module()],
            None,
            RuntimeTargetMode::ClientRuntime,
        )
        .expect("linked runtime plugin plan compiles");

        assert_eq!(
            plan.modules
                .modules()
                .iter()
                .filter(|module| module.module_name() == LINKED_PLAN_MODULE)
                .count(),
            1
        );
        assert_eq!(
            plan.compiled_project_plugin_plan
                .runtime_extensions()
                .registry
                .modules()
                .iter()
                .filter(|descriptor| descriptor.name == LINKED_PLAN_MODULE)
                .count(),
            1
        );
        assert!(plan
            .compiled_project_plugin_plan
            .linked_provider_package_ids()
            .iter()
            .any(|package_id| package_id == "sound"));
        assert_eq!(
            plan.runtime_plugin_catalog_snapshot.generation(),
            plan.compiled_project_plugin_plan.catalog_generation()
        );
    }

    #[cfg(feature = "ui")]
    #[test]
    fn linked_runtime_plugin_plan_preserves_client_baseline_without_project_manifest() {
        let plan = LinkedRuntimePluginPlan::prepare(&[], None, RuntimeTargetMode::ClientRuntime)
            .expect("client baseline plan compiles");
        let completed_manifest = plan.compiled_project_plugin_plan.completed_manifest();

        for runtime_id in [RuntimePluginId::Ui, RuntimePluginId::UiDocumentImporter] {
            assert!(completed_manifest
                .enabled_for_target(RuntimeTargetMode::ClientRuntime)
                .any(|selection| selection.id == runtime_id.key()));
        }
        assert!(plan
            .modules
            .modules()
            .iter()
            .any(|module| { module.module_name() == crate::core::framework::ui::UI_MODULE_NAME }));
    }

    #[test]
    fn dynamic_session_consumes_one_frozen_plugin_plan_without_module_reregistration() {
        let linked_source = include_str!("linked_plugins.rs");
        let construction_source = include_str!("construction.rs");
        let linked_production = linked_source.split("#[cfg(test)]").next().unwrap();

        assert_eq!(
            linked_production
                .matches("RuntimePluginCatalog::from_registration_reports")
                .count(),
            1
        );
        assert!(linked_production.contains("RuntimePluginCatalogSnapshot::from_catalog(catalog)"));
        assert!(construction_source.contains("_runtime_plugin_catalog_snapshot"));
        assert_eq!(
            linked_production.matches(".compiled_project_plan(").count(),
            1
        );
        assert!(linked_production
            .contains("manifest_with_mode_baseline(target_mode, project_manifest)"));
        assert!(linked_production
            .contains("RuntimeModuleCompositionCompiler::new(&compiled_project_plugin_plan)"));
        assert!(linked_production.contains("compiler.with_host_module"));
        assert!(!linked_production
            .contains("runtime_modules_for_target_with_plugin_registration_reports"));
        assert!(!linked_production.contains("runtime_extensions_for_project"));
        assert!(!construction_source.contains("for module in linked_extensions.registry.modules()"));
        assert!(construction_source.contains("for descriptor in modules.module_descriptors()"));
        assert!(!construction_source.contains("module.descriptor()"));
        assert!(!construction_source.contains("BuiltinNavigationModule"));
        assert!(!construction_source.contains("AnimationModule"));
    }

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() - 1) * 95 / 100]
    }

    fn selection_ids() -> Vec<String> {
        (0..REGISTRATION_ADMISSION_COUNT)
            .map(|index| {
                format!(
                    "plugin.selection.{:04}",
                    (index * 1_031) % UNIQUE_SELECTION_COUNT
                )
            })
            .collect()
    }

    fn legacy_selection_admission_count(selection_ids: &[String]) -> usize {
        let mut seen = Vec::<&str>::new();
        for selection_id in selection_ids {
            if seen
                .iter()
                .all(|candidate| *candidate != selection_id.as_str())
            {
                seen.push(selection_id);
            }
        }
        black_box(seen).len()
    }

    fn optimized_selection_admission_count(selection_ids: &[String]) -> usize {
        let mut seen = HashSet::new();
        selection_ids
            .iter()
            .filter(|selection_id| admit_linked_plugin_id(&mut seen, selection_id.as_str()))
            .count()
    }

    #[test]
    fn optimization_batch_20260826s_runtime07_borrowed_hash_admission_preserves_first_seen() {
        let mut seen = HashSet::new();
        let mut admitted = Vec::new();
        for selection_id in ["plugin.b", "plugin.a", "plugin.b"] {
            if admit_linked_plugin_id(&mut seen, selection_id) {
                admitted.push(selection_id);
            }
        }

        assert_eq!(admitted, vec!["plugin.b", "plugin.a"]);
        assert_eq!(seen.len(), 2);
    }

    #[test]
    fn optimization_batch_20260826s_runtime07_linked_plugins_use_hash_indexes() {
        let source = include_str!("linked_plugins.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("use std::collections::HashSet;"));
        assert!(production.contains("let package_ids = compiled_project_plugin_plan"));
        assert_eq!(production.matches("collect::<HashSet<_>>()").count(), 2);
        assert!(production.contains("selected_plugin_ids"));
        assert!(production.contains(".linked_provider_package_ids()"));
        assert!(production.contains(".native_dynamic_provider_package_ids()"));
        assert!(production.contains("package_ids.contains(\"navigation\")"));
        assert!(production.contains("package_ids.contains(\"animation\")"));
        assert!(!production.contains("BTreeSet"));
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn optimization_batch_20260826s_runtime07_linked_plugin_hash_admission_performance_evidence() {
        let selection_ids = selection_ids();
        assert_eq!(
            legacy_selection_admission_count(&selection_ids),
            UNIQUE_SELECTION_COUNT
        );
        assert_eq!(
            optimized_selection_admission_count(&selection_ids),
            UNIQUE_SELECTION_COUNT
        );

        let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample in 0..SAMPLE_COUNT {
            if sample % 2 == 0 {
                let started = Instant::now();
                black_box(legacy_selection_admission_count(black_box(&selection_ids)));
                legacy_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(optimized_selection_admission_count(black_box(
                    &selection_ids,
                )));
                optimized_samples.push(started.elapsed());
            } else {
                let started = Instant::now();
                black_box(optimized_selection_admission_count(black_box(
                    &selection_ids,
                )));
                optimized_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(legacy_selection_admission_count(black_box(&selection_ids)));
                legacy_samples.push(started.elapsed());
            }
        }

        let legacy_p95 = percentile_95(&mut legacy_samples);
        let optimized_p95 = percentile_95(&mut optimized_samples);
        println!(
            "RUNTIME07_LINKED_PLUGIN_HASH_INDEXES_BENCH_V1 admissions={REGISTRATION_ADMISSION_COUNT} \
             unique_selections={UNIQUE_SELECTION_COUNT} legacy_max_membership_scan={UNIQUE_SELECTION_COUNT} \
             optimized_expected_probe_count=1 legacy_p95_ns={} optimized_p95_ns={}",
            legacy_p95.as_nanos(),
            optimized_p95.as_nanos(),
        );
        assert!(
            optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 35,
            "hash-admission P95 {:?} exceeded 35% of linear-admission P95 {:?}",
            optimized_p95,
            legacy_p95,
        );
    }
}
