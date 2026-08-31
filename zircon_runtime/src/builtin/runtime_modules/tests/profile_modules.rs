use std::collections::{BTreeSet, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::builtin::{
    runtime_modules_for_runtime_profile, runtime_modules_for_target, BuiltinRuntimeModuleId,
};
use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::{ProjectPluginManifest, RuntimeProfileId};
use crate::core::ModuleDescriptor;
use crate::engine_module::EngineModule;
use crate::plugin::RuntimeProfileDescriptor;

#[derive(Debug)]
struct CountingFoundationModule {
    descriptor_calls: Arc<AtomicUsize>,
}

impl EngineModule for CountingFoundationModule {
    fn module_name(&self) -> &'static str {
        crate::core::framework::foundation::FOUNDATION_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "counting foundation candidate"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        self.descriptor_calls.fetch_add(1, Ordering::SeqCst);
        ModuleDescriptor::new(self.module_name(), self.module_description())
    }
}

#[test]
fn builtin_profiles_own_unique_typed_module_membership() {
    let profiles = RuntimeProfileDescriptor::builtin_profiles();

    assert_eq!(profiles.len(), 6);
    for profile in &profiles {
        assert!(
            !profile.builtin_modules.is_empty(),
            "profile {:?} must declare its builtin module membership",
            profile.id
        );
        assert_eq!(
            profile
                .builtin_modules
                .iter()
                .copied()
                .collect::<BTreeSet<_>>()
                .len(),
            profile.builtin_modules.len(),
            "profile {:?} must not declare duplicate builtin modules",
            profile.id
        );
    }

    let minimal = RuntimeProfileDescriptor::for_id(RuntimeProfileId::Minimal);
    assert_eq!(
        minimal.builtin_modules,
        vec![
            BuiltinRuntimeModuleId::Foundation,
            BuiltinRuntimeModuleId::Tasks,
            BuiltinRuntimeModuleId::Time,
            BuiltinRuntimeModuleId::FrameCount,
            BuiltinRuntimeModuleId::DiagnosticsCore,
        ]
    );
}

#[test]
fn builtin_profile_assembly_matches_declared_members_and_closes_dependencies() {
    for profile in RuntimeProfileDescriptor::builtin_profiles() {
        let report = match runtime_modules_for_runtime_profile(profile.id) {
            Ok(report) => report,
            Err(rejection) => {
                assert!(
                    !rejection.diagnostics().iter().any(|diagnostic| matches!(
                        diagnostic,
                        crate::builtin::RuntimeModuleLoadDiagnostic::Core(_)
                    )),
                    "profile {:?} must not fail final module/service graph validation: {}",
                    profile.id,
                    rejection
                );
                continue;
            }
        };

        let module_names = report
            .modules()
            .iter()
            .map(|module| module.module_name())
            .collect::<HashSet<_>>();
        for module in report.modules() {
            for dependency in module.descriptor().module_dependencies {
                assert!(
                    module_names.contains(dependency.module_name.as_str()),
                    "profile {:?} module {} is missing dependency {}",
                    profile.id,
                    module.module_name(),
                    dependency.module_name
                );
            }
        }

        let actual_builtin_modules = report
            .modules()
            .iter()
            .filter_map(|module| BuiltinRuntimeModuleId::for_module_name(module.module_name()))
            .collect::<BTreeSet<_>>();
        let declared_builtin_modules = profile
            .builtin_modules
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        assert_eq!(
            actual_builtin_modules, declared_builtin_modules,
            "profile {:?}",
            profile.id
        );
    }
}

#[test]
fn profile_selection_completes_builtin_dependency_closure_from_one_candidate_registry() {
    let profile = RuntimeProfileDescriptor::new(
        RuntimeProfileId::Server,
        "scene-only",
        RuntimeTargetMode::ServerRuntime,
    )
    .with_builtin_module(BuiltinRuntimeModuleId::Scene);
    let candidates = runtime_modules_for_target(
        RuntimeTargetMode::ServerRuntime,
        Some(&ProjectPluginManifest::default()),
    )
    .expect("server candidates should compile")
    .modules()
    .to_vec();

    let selection =
        super::super::assembly::profile_selection::select_runtime_profile_builtin_module_descriptors(
            &profile, candidates,
        )
        .expect("scene-only selection should complete its builtin dependency closure");

    assert_eq!(
        selection
            .modules
            .iter()
            .filter_map(|module| BuiltinRuntimeModuleId::for_module_name(module.module_name()))
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            BuiltinRuntimeModuleId::Foundation,
            BuiltinRuntimeModuleId::Tasks,
            BuiltinRuntimeModuleId::Time,
            BuiltinRuntimeModuleId::Asset,
            BuiltinRuntimeModuleId::Scene,
        ])
    );
}

#[test]
fn profile_selection_reuses_frozen_descriptors_in_the_final_sorter() {
    let descriptor_calls = Arc::new(AtomicUsize::new(0));
    let profile = RuntimeProfileDescriptor::new(
        RuntimeProfileId::Minimal,
        "foundation-only",
        RuntimeTargetMode::ClientRuntime,
    )
    .with_builtin_module(BuiltinRuntimeModuleId::Foundation);
    let selection = super::super::assembly::profile_selection::select_runtime_profile_builtin_module_descriptors(
        &profile,
        vec![Arc::new(CountingFoundationModule {
            descriptor_calls: Arc::clone(&descriptor_calls),
        })],
    )
    .expect("foundation candidate selection should succeed");

    super::super::core_modules::sort_runtime_modules_by_descriptor_order_with_cache(
        selection.modules,
        selection.descriptors_by_name,
    )
    .expect("selected foundation descriptor should sort");

    assert_eq!(descriptor_calls.load(Ordering::SeqCst), 1);
}
