use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::{plugin::RuntimePluginRegistrationReport, scene::SystemStage};

use crate::{
    runtime_plugin, NET_EVENT_ID, NET_FLUSH_EGRESS_SYSTEM, NET_MAIN_SYSTEM_SET, NET_MODULE_NAME,
    NET_POLL_INGRESS_SYSTEM, NET_TRANSPORT_SYSTEM_SET, PLUGIN_RUNTIME_MODULE_NAME,
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
        [NET_MAIN_SYSTEM_SET, NET_TRANSPORT_SYSTEM_SET].map(str::to_string)
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
fn net_runtime_systems_join_main_and_transport_sets() {
    assert_eq!(NET_MAIN_SYSTEM_SET, "net.main");
    assert_eq!(NET_TRANSPORT_SYSTEM_SET, "net.transport");

    let mut report = RuntimePluginRegistrationReport::from_plugin(&runtime_plugin());
    assert!(report.is_success(), "{:?}", report.diagnostics);

    let main_set = report
        .extensions
        .intern_system_set(NET_MAIN_SYSTEM_SET)
        .expect("net.main should be a valid system set");
    let transport_set = report
        .extensions
        .intern_system_set(NET_TRANSPORT_SYSTEM_SET)
        .expect("net.transport should be a valid system set");
    let runtime_systems = report
        .extensions
        .plugin_runtime_systems()
        .filter_map(|(owner, system)| {
            (report.extensions.plugin_module_name(owner) == Some(PLUGIN_RUNTIME_MODULE_NAME))
                .then_some(system)
        })
        .collect::<Vec<_>>();

    assert_eq!(runtime_systems.len(), 2);
    for system in runtime_systems {
        assert_eq!(
            system.sets,
            vec![main_set, transport_set],
            "{} must join net.main and net.transport",
            system.id
        );
    }
    assert_eq!(
        report.package_manifest.modules[0].system_sets,
        [NET_MAIN_SYSTEM_SET, NET_TRANSPORT_SYSTEM_SET].map(str::to_string)
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
