use zircon_runtime::builtin::RuntimeTargetMode;
use zircon_runtime::{plugin::RuntimePluginRegistrationReport, scene::SystemStage};

use crate::{
    runtime_plugin, NET_EVENT_ID, NET_FLUSH_EGRESS_SYSTEM, NET_MODULE_NAME,
    NET_POLL_INGRESS_SYSTEM, NET_SYSTEM_SET, PLUGIN_RUNTIME_MODULE_NAME,
};

#[test]
fn net_plugin_registration_contributes_runtime_module() {
    let report = RuntimePluginRegistrationReport::from_plugin(&runtime_plugin());

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(report
        .extensions
        .modules()
        .iter()
        .any(|module| module.name == NET_MODULE_NAME));
    assert_eq!(
        report.package_manifest.modules[0].target_modes,
        vec![
            RuntimeTargetMode::ServerRuntime,
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ]
    );
    assert_eq!(
        report.package_manifest.modules[0].system_sets,
        vec![NET_SYSTEM_SET.to_string()]
    );
    assert_eq!(
        report.package_manifest.modules[0].system_anchors,
        vec![
            NET_POLL_INGRESS_SYSTEM.to_string(),
            NET_FLUSH_EGRESS_SYSTEM.to_string()
        ]
    );
}

#[test]
fn ingress_anchor_in_first_egress_in_last() {
    let report = RuntimePluginRegistrationReport::from_plugin(&runtime_plugin());
    assert!(report.is_success(), "{:?}", report.diagnostics);

    let systems = report
        .extensions
        .plugin_runtime_systems()
        .map(|(owner, system)| {
            (
                report.extensions.plugin_module_name(owner),
                system.id.as_str(),
                system.stage,
            )
        })
        .collect::<Vec<_>>();
    assert!(systems.iter().any(|(owner, id, stage)| {
        *owner == Some(PLUGIN_RUNTIME_MODULE_NAME)
            && *id == NET_POLL_INGRESS_SYSTEM
            && *stage == SystemStage::First
    }));
    assert!(systems.iter().any(|(owner, id, stage)| {
        *owner == Some(PLUGIN_RUNTIME_MODULE_NAME)
            && *id == NET_FLUSH_EGRESS_SYSTEM
            && *stage == SystemStage::Last
    }));

    assert!(report.extensions.plugin_events().any(|(owner, event)| {
        report.extensions.plugin_module_name(owner) == Some(PLUGIN_RUNTIME_MODULE_NAME)
            && event.manifest().id == NET_EVENT_ID
    }));
}
