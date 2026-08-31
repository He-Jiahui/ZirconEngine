use std::sync::Arc;

use crate::builtin::RuntimeModuleCompositionCompiler;
use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::{ProjectPluginManifest, RuntimeProfileId};
use crate::core::{ModuleDependencySpec, ModuleDescriptor};
use crate::engine_module::EngineModule;
use crate::plugin::RuntimePluginCatalog;

#[derive(Debug)]
struct TestModule {
    descriptor: ModuleDescriptor,
}

impl TestModule {
    fn new(descriptor: ModuleDescriptor) -> Self {
        Self { descriptor }
    }
}

impl EngineModule for TestModule {
    fn module_name(&self) -> &str {
        &self.descriptor.name
    }

    fn module_description(&self) -> &str {
        &self.descriptor.description
    }

    fn descriptor(&self) -> ModuleDescriptor {
        self.descriptor.clone()
    }
}

fn empty_client_plan() -> Arc<crate::plugin::CompiledProjectPluginPlan> {
    RuntimePluginCatalog::from_registration_reports(std::iter::empty(), std::iter::empty())
        .compiled_project_plan(
            &ProjectPluginManifest::default(),
            RuntimeTargetMode::ClientRuntime,
        )
}

#[test]
fn host_modules_participate_in_the_final_compiled_activation_graph() {
    const FIRST: &str = "runtime136.host.first";
    const LAST: &str = "runtime136.host.last";

    let plan = empty_client_plan();
    let composition = RuntimeModuleCompositionCompiler::new(&plan)
        .with_host_module(Arc::new(TestModule::new(ModuleDescriptor::new(
            FIRST,
            "First host module",
        ))))
        .with_host_module(Arc::new(TestModule::new(
            ModuleDescriptor::new(LAST, "Last host module")
                .with_module_dependency(ModuleDependencySpec::named(FIRST)),
        )))
        .compile()
        .expect("host graph should compile");
    let host_names = composition
        .modules()
        .iter()
        .map(|module| module.module_name())
        .filter(|name| name.starts_with("runtime136.host."))
        .collect::<Vec<_>>();

    assert_eq!(host_names, vec![FIRST, LAST]);
}

#[test]
fn duplicate_host_module_rejects_the_composition_without_a_ready_plan() {
    const DUPLICATE: &str = "runtime136.host.duplicate";

    let plan = empty_client_plan();
    let rejection = RuntimeModuleCompositionCompiler::new(&plan)
        .with_host_module(Arc::new(TestModule::new(ModuleDescriptor::new(
            DUPLICATE,
            "First duplicate",
        ))))
        .with_host_module(Arc::new(TestModule::new(ModuleDescriptor::new(
            DUPLICATE,
            "Second duplicate",
        ))))
        .compile()
        .expect_err("duplicate host module must reject the whole composition");

    assert!(rejection
        .fatal_messages()
        .iter()
        .any(|message| message.contains(DUPLICATE)));
}

#[test]
fn composition_identity_is_stable_and_binds_the_final_module_graph() {
    let plan = empty_client_plan();
    let compile = |description| {
        RuntimeModuleCompositionCompiler::new(&plan)
            .with_host_module(Arc::new(TestModule::new(ModuleDescriptor::new(
                "runtime136.host.identity",
                description,
            ))))
            .compile()
            .expect("identity fixture should compile")
    };

    let first = compile("Identity version one");
    let repeated = compile("Identity version one");
    let changed = compile("Identity version two");

    assert_eq!(first.identity(), repeated.identity());
    assert_ne!(first.identity().composition_hash(), [0; 32]);
    assert_ne!(
        first.identity().composition_hash(),
        changed.identity().composition_hash()
    );
    assert_eq!(
        first.identity().catalog_generation(),
        Some(plan.catalog_generation())
    );
    assert_eq!(
        first.identity().source_manifest_fingerprint(),
        Some(plan.source_manifest_fingerprint())
    );
}

#[test]
fn profile_target_mismatch_is_a_rejected_composition() {
    let plan = empty_client_plan();
    let rejection = RuntimeModuleCompositionCompiler::new(&plan)
        .for_runtime_profile(RuntimeProfileId::Server)
        .compile()
        .expect_err("server profile must reject a client-target plan");

    assert!(rejection
        .fatal_messages()
        .iter()
        .any(|message| message.contains("runtime plugin plan target mismatch")));
}
