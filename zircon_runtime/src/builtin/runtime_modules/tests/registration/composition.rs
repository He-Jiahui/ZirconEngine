use std::sync::Arc;

use crate::builtin::{RuntimeModuleCompositionCompiler, RuntimePluginId};
use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::{
    ProjectPluginManifest, ProjectPluginSelection, RuntimeProfileId,
};
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

fn plugin_plan(descriptor: ModuleDescriptor) -> Arc<crate::plugin::CompiledProjectPluginPlan> {
    let registration =
        super::behavior::registration_with_modules(RuntimePluginId::Animation, [descriptor]);
    RuntimePluginCatalog::from_registration_reports([registration], []).compiled_project_plan(
        &ProjectPluginManifest {
            selections: vec![ProjectPluginSelection::runtime_plugin(
                RuntimePluginId::Animation,
                true,
                true,
            )],
        },
        RuntimeTargetMode::ClientRuntime,
    )
}

#[test]
fn plugin_host_dependency_is_resolved_in_final_activation_graph() {
    const PLUGIN: &str = "composition.plugin.host_dependent";
    const HOST: &str = "composition.host.provider";
    let plan = plugin_plan(
        ModuleDescriptor::new(PLUGIN, "Plugin with a host dependency")
            .with_module_dependency(ModuleDependencySpec::named(HOST)),
    );
    let composition = RuntimeModuleCompositionCompiler::new(&plan)
        .with_host_module(Arc::new(TestModule::new(ModuleDescriptor::new(
            HOST,
            "Host provider",
        ))))
        .compile()
        .expect("host dependencies must be resolved after the complete graph is assembled");
    let names = composition
        .modules()
        .iter()
        .map(|module| module.module_name())
        .collect::<Vec<_>>();
    let host = names.iter().position(|name| *name == HOST).unwrap();
    let plugin = names.iter().position(|name| *name == PLUGIN).unwrap();
    assert!(host < plugin);
}

#[test]
fn missing_plugin_host_dependency_rejects_final_composition() {
    const MISSING: &str = "composition.host.missing";
    let plan = plugin_plan(
        ModuleDescriptor::new("composition.plugin.missing_host", "Missing host fixture")
            .with_module_dependency(ModuleDependencySpec::named(MISSING)),
    );
    let rejection = RuntimeModuleCompositionCompiler::new(&plan)
        .compile()
        .expect_err("missing host dependencies must reject the final activation graph");
    assert!(rejection
        .fatal_messages()
        .iter()
        .any(|message| message.contains(MISSING)));
}

#[test]
fn plugin_host_dependency_cycle_rejects_final_composition() {
    const PLUGIN: &str = "composition.plugin.cyclic";
    const HOST: &str = "composition.host.cyclic";
    let plan = plugin_plan(
        ModuleDescriptor::new(PLUGIN, "Cyclic plugin fixture")
            .with_module_dependency(ModuleDependencySpec::named(HOST)),
    );
    let rejection = RuntimeModuleCompositionCompiler::new(&plan)
        .with_host_module(Arc::new(TestModule::new(
            ModuleDescriptor::new(HOST, "Cyclic host fixture")
                .with_module_dependency(ModuleDependencySpec::named(PLUGIN)),
        )))
        .compile()
        .expect_err("cycles through plugin and host modules must reject the final graph");
    let messages = rejection.fatal_messages().join("\n");
    assert!(messages.contains("cycle"), "{messages}");
    assert!(messages.contains(PLUGIN), "{messages}");
    assert!(messages.contains(HOST), "{messages}");
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
