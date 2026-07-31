use zircon_runtime::core::framework::animation::{
    AnimationIkCommand, AnimationIkCommandError, AnimationLookAtCommand, AnimationManager,
    AnimationTargetId,
};
use zircon_runtime::core::framework::project::ExportPackagingStrategy;
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::math::Vec3;
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::plugin::PluginModuleKind;

use super::*;

#[test]
fn plugin_default_animation_manager_is_not_runtime_fallback_type() {
    assert_ne!(
        std::any::TypeId::of::<DefaultAnimationManager>(),
        std::any::TypeId::of::<zircon_runtime::animation::DefaultAnimationManager>(),
    );
}

#[test]
fn plugin_manager_validates_and_drains_ik_commands_per_world() {
    let manager = DefaultAnimationManager::default();
    let world = WorldHandle::new(7);
    let other_world = WorldHandle::new(8);
    let bone = AnimationTargetId::from_segments(["Root", "Head"]);
    let command = AnimationIkCommand::LookAt(AnimationLookAtCommand {
        world,
        entity: 41,
        bone,
        target: Vec3::Y,
        axis: Vec3::X,
        clamp_degrees: 35.0,
        weight: 0.75,
    });

    manager.queue_ik_command(command.clone()).unwrap();

    assert!(manager.drain_ik_commands(other_world).is_empty());
    assert_eq!(manager.drain_ik_commands(world), vec![command]);
    assert!(manager.drain_ik_commands(world).is_empty());

    let invalid = AnimationIkCommand::LookAt(AnimationLookAtCommand {
        world,
        entity: 41,
        bone,
        target: Vec3::Y,
        axis: Vec3::ZERO,
        clamp_degrees: 35.0,
        weight: 1.0,
    });
    assert_eq!(
        manager.queue_ik_command(invalid),
        Err(AnimationIkCommandError::DegenerateAxis { world, entity: 41 })
    );
}

#[test]
fn animation_registration_contributes_runtime_module() {
    let report = plugin_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(report
        .extensions
        .modules()
        .iter()
        .any(|module| module.name == ANIMATION_MODULE_NAME));
    assert!(report
        .extensions
        .plugin_runtime_systems()
        .any(|(owner, system)| {
            report.extensions.plugin_module_name(owner) == Some(PLUGIN_RUNTIME_MODULE_NAME)
                && system.id == ANIMATION_EVALUATE_SYSTEM
                && system.stage == zircon_runtime::scene::SystemStage::PostUpdate
        }));
    assert!(report
        .extensions
        .plugin_resources()
        .any(|(owner, resource)| {
            report.extensions.plugin_module_name(owner) == Some(PLUGIN_RUNTIME_MODULE_NAME)
                && resource.type_name() == std::any::type_name::<AnimationEvaluationPipeline>()
        }));
    assert!(report.extensions.plugin_events().any(|(owner, event)| {
        report.extensions.plugin_module_name(owner) == Some(PLUGIN_RUNTIME_MODULE_NAME)
            && event.type_name() == std::any::type_name::<AnimationClipEvent>()
    }));
    assert!(report.extensions.plugin_events().any(|(owner, event)| {
        report.extensions.plugin_module_name(owner) == Some(PLUGIN_RUNTIME_MODULE_NAME)
            && event.type_name() == std::any::type_name::<AnimationIkDiagnostic>()
    }));
    assert!(report.extensions.plugin_events().any(|(owner, event)| {
        report.extensions.plugin_module_name(owner) == Some(PLUGIN_RUNTIME_MODULE_NAME)
            && event.type_name() == std::any::type_name::<AnimationStateMachineLayerDiagnostic>()
    }));
    assert!(report
        .extensions
        .plugin_event_catalogs()
        .iter()
        .any(|catalog| {
            catalog.namespace == "animation.events"
                && catalog.events.iter().any(|event| {
                    event.id == ANIMATION_CLIP_EVENT
                        && event.payload_schema == ANIMATION_CLIP_EVENT_SCHEMA
                })
        }));
    assert_eq!(
        report.package_manifest.modules[0].system_sets,
        vec![ANIMATION_SYSTEM_SET.to_string()]
    );
    assert_eq!(
        report.package_manifest.modules[0].system_anchors,
        vec![ANIMATION_EVALUATE_SYSTEM.to_string()]
    );
    assert_eq!(
        report.package_manifest.modules[0].target_modes,
        vec![
            zircon_runtime::core::framework::platform::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::core::framework::platform::RuntimeTargetMode::ServerRuntime,
            zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost,
        ]
    );
    assert_eq!(report.package_manifest.category, "runtime");
    assert_eq!(
        report.package_manifest.maturity,
        zircon_runtime::plugin::PluginMaturity::Beta
    );
    assert!(report
        .package_manifest
        .capability_statuses
        .iter()
        .any(|status| {
            status.capability == ANIMATION_RUNTIME_CAPABILITY
                && status.status == zircon_runtime::plugin::CapabilityStatus::Partial
                && status
                    .bevy_references
                    .iter()
                    .any(|reference| reference == "dev/bevy/crates/bevy_animation/src/lib.rs")
        }));
    assert!(report
        .package_manifest
        .capability_statuses
        .iter()
        .any(|status| {
            status.capability == ANIMATION_TIMELINE_EVENT_TRACK_CAPABILITY
                && status.status == zircon_runtime::plugin::CapabilityStatus::Partial
        }));
}

#[test]
fn animation_evaluate_runs_after_physics_sync() {
    let report = plugin_registration();
    let animation_stage = report
        .extensions
        .plugin_runtime_systems()
        .find_map(|(owner, system)| {
            (report.extensions.plugin_module_name(owner) == Some(PLUGIN_RUNTIME_MODULE_NAME)
                && system.id == ANIMATION_EVALUATE_SYSTEM)
                .then_some(system.stage)
        })
        .expect("animation evaluate system should be registered");

    assert_eq!(
        animation_stage,
        zircon_runtime::scene::SystemStage::PostUpdate
    );
    assert!(
        animation_stage.rank() > zircon_runtime::scene::SystemStage::FixedPostUpdate.rank(),
        "animation evaluation must run after physics sync in FixedPostUpdate"
    );
}

#[test]
fn animation_module_resolves_manager() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(ANIMATION_MODULE_NAME).unwrap();

    runtime
        .handle()
        .resolve_manager::<DefaultAnimationManager>(DEFAULT_ANIMATION_MANAGER_NAME)
        .unwrap();
}

#[test]
fn animation_package_manifest_declares_dist_contract() {
    let manifest = package_manifest();

    assert!(manifest
        .default_packaging
        .contains(&ExportPackagingStrategy::NativeDynamic));

    let distribution = manifest
        .distribution
        .as_ref()
        .expect("animation distribution manifest");
    assert_eq!(distribution.forms, vec!["dist".to_string()]);
    assert_eq!(
        distribution.default_packaging,
        vec![ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(distribution.engine_compat, ">=0.1, <0.2");
    assert_eq!(distribution.dist_crate, ANIMATION_DIST_CRATE_NAME);
    assert_eq!(
        distribution.descriptor_symbol,
        "zircon_native_plugin_descriptor_v3"
    );
    assert_eq!(distribution.runtime_entry, ANIMATION_DIST_RUNTIME_ENTRY);

    let native_module = manifest
        .modules
        .iter()
        .find(|module| module.name == "animation.dist")
        .expect("animation native dist module");
    assert_eq!(native_module.kind, PluginModuleKind::Native);
    assert_eq!(native_module.crate_name, ANIMATION_DIST_CRATE_NAME);
    assert_eq!(
        native_module.target_modes,
        vec![
            zircon_runtime::core::framework::platform::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::core::framework::platform::RuntimeTargetMode::ServerRuntime,
            zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost,
        ]
    );
    for capability in RUNTIME_CAPABILITIES {
        assert!(native_module.capabilities.contains(&capability.to_string()));
    }

    assert!(manifest
        .modules
        .iter()
        .any(|module| module.name == "animation.runtime"));
    assert_eq!(
        manifest.modules[0].system_sets,
        vec![ANIMATION_SYSTEM_SET.to_string()]
    );
    assert_eq!(
        manifest.modules[0].system_anchors,
        vec![ANIMATION_EVALUATE_SYSTEM.to_string()]
    );
}
