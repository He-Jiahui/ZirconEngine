use std::sync::{Condvar, Mutex};
use std::time::Duration;
use zircon_runtime::builtin::{RuntimePluginId, RuntimeTargetMode};

use crate::behavior_tree::{
    BehaviorNodeCategory, BehaviorNodeDescriptor, BehaviorNodeRegistry, BehaviorNodeRuntime,
    BehaviorNodeSemantics, BehaviorNodeTickContext,
};
use crate::{
    package_manifest, plugin_registration, runtime_capabilities, runtime_plugin_descriptor,
    AI_BEHAVIOR_TICK_SYSTEM, AI_DIST_CRATE_NAME, AI_DIST_RUNTIME_ENTRY, AI_MODULE_NAME,
    RUNTIME_CAPABILITIES,
};
use zircon_runtime::core::framework::ai::{
    AiAgentTickRequest, AiBehaviorNodeDescriptor, AiBehaviorNodeKind, AiBehaviorTreeDescriptor,
    AiDecisionStatus, AiManager,
};
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::plugin::{RuntimeExtensionRegistry, RuntimePlugin};

#[derive(Debug)]
struct BridgeNode;

impl BehaviorNodeRuntime for BridgeNode {
    fn tick(&mut self, _context: &BehaviorNodeTickContext<'_>) -> AiDecisionStatus {
        AiDecisionStatus::Succeeded
    }
}

fn bridge_node_factory() -> Box<dyn BehaviorNodeRuntime> {
    Box::new(BridgeNode)
}

static BLOCKING_NODE_STATE: Mutex<(bool, bool)> = Mutex::new((false, false));
static BLOCKING_NODE_SIGNAL: Condvar = Condvar::new();

#[derive(Debug)]
struct BlockingBridgeNode;

impl BehaviorNodeRuntime for BlockingBridgeNode {
    fn tick(&mut self, _context: &BehaviorNodeTickContext<'_>) -> AiDecisionStatus {
        let mut state = BLOCKING_NODE_STATE
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.0 = true;
        BLOCKING_NODE_SIGNAL.notify_all();
        while !state.1 {
            state = BLOCKING_NODE_SIGNAL
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        AiDecisionStatus::Succeeded
    }
}

fn blocking_bridge_node_factory() -> Box<dyn BehaviorNodeRuntime> {
    Box::new(BlockingBridgeNode)
}

#[test]
fn behavior_tick_anchor_in_update() {
    let report = plugin_registration();
    let behavior_tick = report
        .extensions
        .plugin_runtime_systems()
        .find(|(_, system)| system.id == AI_BEHAVIOR_TICK_SYSTEM)
        .map(|(_, system)| system)
        .expect("AI behavior tick system");

    assert_eq!(
        behavior_tick.stage,
        zircon_runtime::scene::SystemStage::Update
    );
}

#[test]
fn ai_registration_contributes_runtime_module_and_capabilities() {
    let report = plugin_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(report
        .extensions
        .modules()
        .iter()
        .any(|module| module.name == AI_MODULE_NAME));
    let behavior_tick = report
        .extensions
        .plugin_runtime_systems()
        .find(|(_, system)| system.id == AI_BEHAVIOR_TICK_SYSTEM)
        .map(|(_, system)| system)
        .expect("AI behavior tick system");
    assert_eq!(
        behavior_tick.stage,
        zircon_runtime::scene::SystemStage::Update
    );
    assert_eq!(report.package_manifest.category, "runtime");
    assert_eq!(
        report.package_manifest.maturity,
        zircon_runtime::plugin::PluginMaturity::Experimental
    );
    assert_eq!(
        report.package_manifest.supported_targets,
        vec![
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::ServerRuntime,
            RuntimeTargetMode::EditorHost,
        ]
    );
    assert_eq!(
        report.package_manifest.capabilities,
        runtime_capabilities()
            .iter()
            .map(|capability| capability.to_string())
            .collect::<Vec<_>>()
    );
    for capability in runtime_capabilities() {
        assert!(report
            .package_manifest
            .capability_statuses
            .iter()
            .any(|status| {
                status.capability == *capability
                    && status.status == zircon_runtime::plugin::CapabilityStatus::Partial
            }));
    }
}

#[test]
fn ai_runtime_descriptor_matches_builtin_catalog_row() {
    let descriptor = runtime_plugin_descriptor();
    let catalog_descriptor = zircon_runtime::plugin::RuntimePluginDescriptor::builtin_catalog()
        .into_iter()
        .find(|descriptor| descriptor.runtime_id() == RuntimePluginId::Ai)
        .expect("AI built-in catalog entry");

    assert_eq!(catalog_descriptor.package_id(), descriptor.package_id());
    assert_eq!(catalog_descriptor.crate_name(), descriptor.crate_name());
    assert_eq!(catalog_descriptor.category(), descriptor.category());
    assert_eq!(catalog_descriptor.maturity(), descriptor.maturity());
    assert_eq!(catalog_descriptor.target_modes(), descriptor.target_modes());
    assert_eq!(catalog_descriptor.capabilities(), descriptor.capabilities());
    assert_eq!(
        catalog_descriptor.capability_statuses(),
        descriptor.capability_statuses()
    );
}

#[test]
fn ai_package_manifest_declares_dist_contract() {
    let manifest = package_manifest();
    let distribution = manifest
        .distribution
        .as_ref()
        .expect("ai dist distribution");

    assert!(manifest
        .default_packaging
        .contains(&zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic));
    assert_eq!(distribution.forms, vec!["dist"]);
    assert_eq!(
        distribution.default_packaging,
        vec![zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(distribution.engine_compat, ">=0.1, <0.2");
    assert_eq!(distribution.dist_crate, AI_DIST_CRATE_NAME);
    assert_eq!(
        distribution.descriptor_symbol,
        "zircon_native_plugin_descriptor_v3"
    );
    assert_eq!(distribution.runtime_entry, AI_DIST_RUNTIME_ENTRY);
    assert!(manifest.modules.iter().any(|module| {
        module.kind == zircon_runtime::plugin::PluginModuleKind::Native
            && module.name == "ai.dist"
            && module.crate_name == AI_DIST_CRATE_NAME
            && module.capabilities == RUNTIME_CAPABILITIES
    }));
}

#[test]
fn generated_plugin_toml_matches_runtime_events_and_system_anchors() {
    let generated: zircon_runtime::plugin::PluginPackageManifest =
        toml::from_str(include_str!("../../../plugin.toml")).expect("generated AI plugin.toml");
    let runtime = package_manifest();

    assert_eq!(generated.event_catalogs, runtime.event_catalogs);
    assert_eq!(generated.provides_interfaces, runtime.provides_interfaces);
    let generated_runtime = generated
        .modules
        .iter()
        .find(|module| module.name == AI_MODULE_NAME)
        .expect("generated ai.runtime module");
    let runtime_module = runtime
        .modules
        .iter()
        .find(|module| module.name == AI_MODULE_NAME)
        .expect("runtime ai.runtime module");
    assert_eq!(
        generated_runtime.system_anchors,
        runtime_module.system_anchors
    );
}

#[test]
fn standard_nodes_bind_to_the_actual_nonzero_ai_registry_owner() {
    let plugin = crate::AiRuntimePlugin::new();
    let manager = plugin.manager();
    manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("standard_owner_tree", "Standard Owner", "root")
                .with_node(AiBehaviorNodeDescriptor::new(
                    "root",
                    AiBehaviorNodeKind::Task,
                    "Wait",
                )),
        )
        .expect("tree compiled before AI owner is interned");
    let mut registry = RuntimeExtensionRegistry::default();
    let unrelated_owner = registry
        .intern_plugin_module("test.unrelated.runtime")
        .expect("intern unrelated owner");
    assert_eq!(unrelated_owner.raw(), 0);
    plugin.register(&mut registry).expect("register AI plugin");
    let ai_owner = registry
        .intern_plugin_module(AI_MODULE_NAME)
        .expect("resolve AI owner");
    assert_ne!(ai_owner.raw(), 0);
    registry.finalize();
    let bridge = registry
        .frozen_bridge_table()
        .resolve_strong::<dyn BehaviorNodeRegistry>()
        .expect("AI behavior-node registry bridge");

    registry.revoke_owner_registrations(unrelated_owner);
    assert_eq!(bridge.descriptors().len(), 18);
    assert_eq!(manager.behavior_trees().len(), 1);

    registry.revoke_owner_registrations(ai_owner);
    assert!(bridge.descriptors().is_empty());
    assert!(manager.behavior_trees().is_empty());
}

#[test]
fn runtime_registry_bridge_adds_a_node_to_the_live_ai_manager_catalog() {
    let plugin = crate::AiRuntimePlugin::new();
    let mut registration_report =
        zircon_runtime::plugin::RuntimePluginRegistrationReport::from_plugin(&plugin);
    let bridge = registration_report
        .extensions
        .frozen_bridge_table()
        .resolve_strong::<dyn BehaviorNodeRegistry>()
        .expect("AI behavior-node registry bridge");
    let contributor = registration_report
        .extensions
        .intern_plugin_module("test.behavior_node_contributor.runtime")
        .expect("intern contributor owner");
    bridge
        .add_node(
            contributor,
            BehaviorNodeDescriptor::new(
                "bridge.node",
                "Bridge Node",
                BehaviorNodeCategory::Task,
                BehaviorNodeSemantics::External,
            )
            .with_factory(bridge_node_factory),
        )
        .expect("bridge node contribution");
    let manager = plugin.manager();
    let tree = manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("bridge_tree", "Bridge Tree", "root").with_node(
                AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Task, "Bridge")
                    .with_implementation("bridge.node"),
            ),
        )
        .expect("tree sees bridge-contributed node");
    let tick_report = manager
        .tick_agent(AiAgentTickRequest {
            world: WorldHandle::new(501),
            entity: 1,
            behavior_tree: Some(tree),
            blackboard_schema: None,
            delta_seconds: 0.1,
            blackboard: Vec::new(),
            perception: None,
        })
        .expect("bridge node tick");
    assert_eq!(tick_report.status, AiDecisionStatus::Succeeded);

    let revoked = registration_report
        .extensions
        .revoke_owner_registrations(contributor);
    assert!(revoked.is_empty());
    assert!(!bridge
        .descriptors()
        .iter()
        .any(|descriptor| descriptor.id() == "bridge.node"));
    assert!(manager.behavior_trees().is_empty());
}

#[test]
fn owner_revoke_waits_for_in_flight_node_execution_and_runtime_drop() {
    {
        let mut state = BLOCKING_NODE_STATE
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *state = (false, false);
    }
    let plugin = crate::AiRuntimePlugin::new();
    let mut registration_report =
        zircon_runtime::plugin::RuntimePluginRegistrationReport::from_plugin(&plugin);
    let bridge = registration_report
        .extensions
        .frozen_bridge_table()
        .resolve_strong::<dyn BehaviorNodeRegistry>()
        .expect("AI behavior-node registry bridge");
    let contributor = registration_report
        .extensions
        .intern_plugin_module("test.blocking_node_contributor.runtime")
        .expect("intern blocking contributor");
    bridge
        .add_node(
            contributor,
            BehaviorNodeDescriptor::new(
                "bridge.blocking_node",
                "Blocking Bridge Node",
                BehaviorNodeCategory::Task,
                BehaviorNodeSemantics::External,
            )
            .with_factory(blocking_bridge_node_factory),
        )
        .expect("blocking bridge node contribution");
    let manager = plugin.manager();
    let tree = manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("blocking_tree", "Blocking Tree", "root").with_node(
                AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Task, "Blocking")
                    .with_implementation("bridge.blocking_node"),
            ),
        )
        .expect("blocking tree");

    let tick_manager = manager.clone();
    let tick_thread = std::thread::spawn(move || {
        tick_manager.tick_agent(AiAgentTickRequest {
            world: WorldHandle::new(777),
            entity: 1,
            behavior_tree: Some(tree),
            blackboard_schema: None,
            delta_seconds: 0.1,
            blackboard: Vec::new(),
            perception: None,
        })
    });
    {
        let state = BLOCKING_NODE_STATE
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let (state, timeout) = BLOCKING_NODE_SIGNAL
            .wait_timeout_while(state, Duration::from_secs(2), |state| !state.0)
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(state.0, "blocking node entered tick before timeout");
        assert!(!timeout.timed_out());
    }

    let (revoke_done_tx, revoke_done_rx) = std::sync::mpsc::channel();
    let revoke_thread = std::thread::spawn(move || {
        registration_report
            .extensions
            .revoke_owner_registrations(contributor);
        revoke_done_tx.send(()).expect("report revoke completion");
    });
    assert!(
        revoke_done_rx
            .recv_timeout(Duration::from_millis(50))
            .is_err(),
        "owner revoke must wait while plugin node code is in flight"
    );
    let (add_done_tx, add_done_rx) = std::sync::mpsc::channel();
    let add_thread = std::thread::spawn(move || {
        let result = bridge.add_node(
            contributor,
            BehaviorNodeDescriptor::new(
                "bridge.next_generation",
                "Next-generation Bridge Node",
                BehaviorNodeCategory::Task,
                BehaviorNodeSemantics::External,
            )
            .with_factory(bridge_node_factory),
        );
        add_done_tx
            .send(result)
            .expect("next-generation add completion");
    });
    assert!(
        add_done_rx.recv_timeout(Duration::from_millis(50)).is_err(),
        "next-generation registration must wait for old revoke cleanup"
    );
    {
        let mut state = BLOCKING_NODE_STATE
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.1 = true;
        BLOCKING_NODE_SIGNAL.notify_all();
    }
    let tick_report = tick_thread
        .join()
        .expect("blocking tick thread")
        .expect("blocking tick result");
    assert_eq!(tick_report.status, AiDecisionStatus::Succeeded);
    revoke_done_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("revoke completes after tick lease releases");
    revoke_thread.join().expect("revoke thread");
    add_done_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("next-generation add completes after revoke")
        .expect("next-generation node registration");
    add_thread.join().expect("next-generation add thread");
    assert!(manager.behavior_trees().is_empty());
    let catalog = manager.behavior_node_catalog();
    let catalog = catalog
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert!(catalog
        .snapshot()
        .resolve("bridge.next_generation")
        .is_some());
    assert!(catalog.snapshot().resolve("bridge.blocking_node").is_none());
}
