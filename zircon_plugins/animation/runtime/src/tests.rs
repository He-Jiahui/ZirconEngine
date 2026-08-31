use zircon_runtime::core::framework::project::ExportPackagingStrategy;
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
fn animation_runtime_has_no_process_wide_ik_inbox() {
    let framework_manager =
        include_str!("../../../../zircon_runtime/src/core/framework/animation/manager.rs");
    let plugin_manager = include_str!("manager.rs");
    let tick = include_str!("evaluation/pipeline/tick.rs");
    let queue_method = ["queue", "ik", "command"].join("_");
    let drain_method = ["drain", "ik", "commands"].join("_");

    for (owner, source) in [
        ("framework manager", framework_manager),
        ("plugin manager", plugin_manager),
        ("animation tick", tick),
    ] {
        assert!(
            !source.contains(&queue_method) && !source.contains(&drain_method),
            "{owner} must not restore the retired process-wide IK inbox"
        );
    }
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

#[test]
fn property_sequence_pipeline_caches_compiled_projection_by_asset_revision() {
    let pipeline_source = include_str!("evaluation/pipeline/animation_evaluation_pipeline.rs");
    let request_source = include_str!("evaluation/pipeline/requests.rs");
    let scan_source = include_str!("evaluation/pipeline/parameter_apply.rs");
    let tick_source = include_str!("evaluation/pipeline/tick.rs");
    let sequence_source = include_str!("evaluation/pipeline/sequences.rs");

    assert!(pipeline_source.contains("sequence_cache"));
    assert!(request_source.contains("asset_revision: Option<u64>"));
    assert!(scan_source.contains("asset_revision: revision.asset_revision"));
    assert!(tick_source.contains("asset_revision: pending.asset_revision"));
    assert!(sequence_source.contains("struct CachedCompiledSequence"));
    assert!(sequence_source.contains("compile_sequence_for_world"));
    assert!(sequence_source.contains("apply_compiled_sequence_to_world"));
    assert!(sequence_source.contains("compiled.is_current_for(world)"));
    assert!(sequence_source.contains("sample.asset_revision.is_none()"));
    assert!(!sequence_source.contains("apply_sequence_to_world"));
}
