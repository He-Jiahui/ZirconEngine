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
    pub(super) fn prepare_core_only(
        target_mode: RuntimeTargetMode,
    ) -> RuntimeDynamicSessionResult<Self> {
        // The stable runtime ABI does not carry plugin registrations. The app host
        // has already admitted the project selections; this DLL owns core modules.
        Self::prepare_with_manifest(&[], ProjectPluginManifest::default(), target_mode)
    }

    pub(super) fn prepare(
        registrations: &[RuntimePluginRegistrationReport],
        project_manifest: Option<&ProjectPluginManifest>,
        target_mode: RuntimeTargetMode,
    ) -> RuntimeDynamicSessionResult<Self> {
        Self::prepare_with_manifest(
            registrations,
            manifest_with_mode_baseline(target_mode, project_manifest),
            target_mode,
        )
    }

    fn prepare_with_manifest(
        registrations: &[RuntimePluginRegistrationReport],
        mut effective_manifest: ProjectPluginManifest,
        target_mode: RuntimeTargetMode,
    ) -> RuntimeDynamicSessionResult<Self> {
        // 保留 manifest 已选项，并按首次出现补入 linked registrations，避免同一插件重复进入编译计划。
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
#[path = "tests/linked_plugins_optimization_tests.rs"]
mod optimization_tests;
