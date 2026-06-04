use zircon_runtime::graphics::RenderPassStage;
use zircon_runtime::plugin::RuntimePluginRegistrationReport;
use zircon_runtime::render_graph::QueueLane;

use crate::{
    render_feature_descriptor, runtime_plugin, PARTICLES_DYNAMIC_EVENT_NAMESPACE,
    PARTICLES_FEATURE_NAME, PARTICLES_MODULE_NAME, PARTICLES_RUNTIME_CAPABILITY,
    PARTICLE_SYSTEM_COMPONENT_TYPE,
};

#[test]
fn particles_plugin_registration_contributes_runtime_module_render_feature_and_component() {
    let report = RuntimePluginRegistrationReport::from_plugin(&runtime_plugin());

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(report
        .extensions
        .modules()
        .iter()
        .any(|module| module.name == PARTICLES_MODULE_NAME));
    assert!(report
        .extensions
        .render_features()
        .iter()
        .any(|feature| feature.name == PARTICLES_FEATURE_NAME));
    assert!(report
        .extensions
        .components()
        .iter()
        .any(|descriptor| descriptor.type_id == PARTICLE_SYSTEM_COMPONENT_TYPE));
    let backend_option = report
        .extensions
        .plugin_options()
        .iter()
        .find(|option| option.key == "particles.backend")
        .expect("particles backend option");
    assert_eq!(backend_option.value_type, "enum");
    assert_eq!(backend_option.default_value, "cpu");
    assert_eq!(
        backend_option.enum_values,
        vec!["cpu".to_string(), "gpu".to_string()]
    );
    let fixed_preview_dt = report
        .extensions
        .plugin_options()
        .iter()
        .find(|option| option.key == "particles.fixed_preview_dt")
        .expect("particles fixed preview step option");
    assert_eq!(fixed_preview_dt.value_type, "number");
    let executor_ids = report
        .extensions
        .render_pass_executors()
        .iter()
        .map(|registration| registration.executor_id().as_str())
        .collect::<Vec<_>>();
    assert!(executor_ids.contains(&"particle.gpu.spawn-update"));
    assert!(executor_ids.contains(&"particle.gpu.compact-alive"));
    assert!(executor_ids.contains(&"particle.gpu.indirect-args"));
    assert!(executor_ids.contains(&"particle.transparent"));
    let event_catalog = report
        .package_manifest
        .event_catalogs
        .iter()
        .find(|catalog| catalog.namespace == PARTICLES_DYNAMIC_EVENT_NAMESPACE)
        .expect("particles dynamic event catalog");
    assert_eq!(
        event_catalog
            .events
            .iter()
            .map(|event| event.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "particles.dynamic_events.spawn_once",
            "particles.dynamic_events.begin_emission",
            "particles.dynamic_events.end_emission",
        ]
    );
    assert_eq!(report.package_manifest.category, "runtime");
    assert_eq!(
        report.package_manifest.maturity,
        zircon_runtime::plugin::PluginMaturity::Experimental
    );
    assert!(report
        .package_manifest
        .capability_statuses
        .iter()
        .any(|status| {
            status.capability == PARTICLES_RUNTIME_CAPABILITY
                && status.status == zircon_runtime::plugin::CapabilityStatus::Partial
                && status.note.as_deref()
                    == Some("Advanced optional VFX capability; not a Bevy default parity blocker.")
        }));
    assert_eq!(
        report
            .package_manifest
            .optional_features
            .iter()
            .map(|feature| feature.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "particles.physics",
            "particles.animation_control",
            "particles.gpu_simulation",
        ]
    );

    let descriptor = render_feature_descriptor();
    let pass_names = descriptor
        .stage_passes
        .iter()
        .map(|pass| pass.pass_name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        pass_names,
        vec![
            "particle-gpu-spawn-update",
            "particle-gpu-compact-alive",
            "particle-gpu-build-indirect-args",
            "particle-render"
        ]
    );
    assert!(descriptor
        .stage_passes
        .iter()
        .all(|pass| pass.stage == RenderPassStage::Transparent3d));
    assert_eq!(descriptor.stage_passes[0].queue, QueueLane::AsyncCompute);
    assert_eq!(descriptor.stage_passes[3].queue, QueueLane::Graphics);
}
