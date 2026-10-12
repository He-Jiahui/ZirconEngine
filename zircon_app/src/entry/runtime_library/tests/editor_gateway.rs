//! Behavioral contracts for App-owned dynamic V8 sessions, gateway identity and plugin consumers.

#[cfg(feature = "target-editor-host")]
use std::sync::Arc;

#[cfg(feature = "target-editor-host")]
use zircon_editor::core::gateway::{
    EditorRuntimeGatewayHandle, RuntimeCapabilities, SessionProfileKind,
};
#[cfg(feature = "target-editor-host")]
use zircon_editor::core::runtime_event_consumer::EditorRuntimeEventConsumerHost;
#[cfg(feature = "target-editor-host")]
use zircon_runtime::builtin::RuntimePluginId;
#[cfg(feature = "target-editor-host")]
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
#[cfg(feature = "target-editor-host")]
use zircon_runtime::core::framework::project::{ProjectPluginManifest, ProjectPluginSelection};

#[cfg(feature = "target-editor-host")]
use super::super::loaded_runtime::LoadedRuntime;
#[cfg(feature = "target-editor-host")]
use super::super::runtime_session::RuntimeSession;

#[cfg(all(
    feature = "target-editor-host",
    target_os = "windows",
    feature = "dynamic-api"
))]
use super::super::loaded_runtime::surface_teardown_fixture::{self, Call, Policy};

#[cfg(feature = "target-editor-host")]
#[test]
fn runtime_session_satisfies_editor_gateway_thread_safety_contract() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<RuntimeSession>();
}

#[cfg(all(
    feature = "target-editor-host",
    target_os = "windows",
    feature = "dynamic-api"
))]
#[test]
fn editor_gateway_is_owned_by_session_gateway_instead_of_runtime_session() {
    surface_teardown_fixture::with_session(Policy::default(), |session, control| {
        let session = Arc::new(session);
        let owner = Arc::downgrade(&session);
        let gateway = session
            .editor_gateway(RuntimeCapabilities::editor_default())
            .unwrap();
        drop(session);
        assert!(
            owner.upgrade().is_some(),
            "gateway must retain the actual session owner"
        );
        assert!(control.snapshot().active);
        drop(gateway);
        assert!(owner.upgrade().is_none());
        let snapshot = control.snapshot();
        assert!(!snapshot.active);
        assert_eq!(snapshot.allocations, 0);
        assert_eq!(
            snapshot.calls,
            [
                Call::Create,
                Call::Receipt,
                Call::ReleaseAllocation,
                Call::Destroy
            ]
        );
    });
}

#[cfg(all(
    feature = "target-editor-host",
    target_os = "windows",
    feature = "dynamic-api"
))]
#[test]
fn editor_gateway_retains_the_app_issued_runtime_session_identity() {
    use zircon_editor::core::gateway::EditorRuntimeGateway;

    let project_root = std::path::Path::new("E:/projects/identity-项目");
    surface_teardown_fixture::with_session_for_project(
        Policy::default(),
        Some(project_root),
        |session, _| {
            let session = Arc::new(session);
            let expected = session.gateway_identity();
            let first = session
                .editor_gateway(RuntimeCapabilities::editor_default())
                .unwrap();
            let second = session
                .editor_gateway(RuntimeCapabilities::editor_default())
                .unwrap();
            assert_ne!(expected.runtime_instance(), 0);
            assert!(expected.runtime_session().is_valid());
            assert_eq!(expected.transport_epoch(), 1);
            assert_eq!(expected.project(), project_root.to_str());
            assert_eq!(expected.gateway_generation(), 0);
            assert_eq!(expected.play_instance(), None);
            assert_eq!(first.session_identity(), expected);
            assert_eq!(second.session_identity(), expected);
        },
    );
}

#[cfg(all(
    feature = "target-editor-host",
    target_os = "windows",
    feature = "dynamic-api"
))]
#[test]
fn editor_gateway_api_table_excludes_session_lifecycle_but_retains_surface_presentation_authority()
{
    surface_teardown_fixture::with_runtime(Policy::default(), |runtime, _| {
        let editor_api = runtime.editor_gateway_api_table();
        assert!(editor_api.create_session.is_none());
        assert!(editor_api.destroy_session.is_none());
        assert!(editor_api.drain_host_requests.is_none());
        assert!(editor_api.release_allocation.is_some());
        assert!(editor_api.bind_viewport_surface.is_some());
        assert!(editor_api.unbind_viewport_surface.is_some());
        assert!(editor_api.present_viewport.is_some());
    });
}

#[cfg(all(
    feature = "target-editor-host",
    target_os = "windows",
    feature = "dynamic-api"
))]
#[test]
fn live_loaded_runtime_owners_with_the_same_raw_handle_have_distinct_gateway_identities() {
    use zircon_editor::core::gateway::EditorRuntimeGateway;

    surface_teardown_fixture::with_session(Policy::default(), |first, first_control| {
        let first = Arc::new(first);
        let first_gateway = first
            .editor_gateway(RuntimeCapabilities::editor_default())
            .unwrap();
        surface_teardown_fixture::with_session(Policy::default(), |second, second_control| {
            let second = Arc::new(second);
            let second_gateway = second
                .editor_gateway(RuntimeCapabilities::editor_default())
                .unwrap();
            let first_identity = first_gateway.session_identity();
            let second_identity = second_gateway.session_identity();
            assert!(first_control.snapshot().active && second_control.snapshot().active);
            assert_eq!(
                first_identity.runtime_session(),
                second_identity.runtime_session()
            );
            assert_ne!(first_identity.runtime_instance(), 0);
            assert_ne!(second_identity.runtime_instance(), 0);
            assert_ne!(
                first_identity.runtime_instance(),
                second_identity.runtime_instance()
            );
            assert_ne!(first_identity, second_identity);
            assert_eq!(first_identity.transport_epoch(), 1);
            assert_eq!(second_identity.transport_epoch(), 1);
            assert_eq!(first_control.snapshot().allocations, 0);
            assert_eq!(second_control.snapshot().allocations, 0);
            drop(first);
            drop(second);
            assert!(first_control.snapshot().active && second_control.snapshot().active);
            drop(first_gateway);
            assert!(!first_control.snapshot().active);
            assert!(second_control.snapshot().active);
            assert_eq!(
                first_control
                    .snapshot()
                    .calls
                    .iter()
                    .filter(|call| **call == Call::Destroy)
                    .count(),
                1
            );
            assert_eq!(
                second_control
                    .snapshot()
                    .calls
                    .iter()
                    .filter(|call| **call == Call::Destroy)
                    .count(),
                0
            );
            drop(second_gateway);
            assert!(!second_control.snapshot().active);
            assert_eq!(
                second_control
                    .snapshot()
                    .calls
                    .iter()
                    .filter(|call| **call == Call::Destroy)
                    .count(),
                1
            );
        });
    });
}

#[cfg(feature = "target-editor-host")]
#[test]
fn editor_product_ticks_selected_navigation_plugin_into_typed_consumer() {
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Navigation,
            true,
            false,
        )
        .with_target_modes([RuntimeTargetMode::EditorHost])],
    };
    let runtime_registrations = crate::entry::first_party_runtime_plugin_registrations_for_manifest(
        RuntimeTargetMode::EditorHost,
        &manifest,
    );
    let editor_registrations = crate::entry::first_party_editor_plugin_registrations_for_manifest(
        RuntimeTargetMode::EditorHost,
        &manifest,
    );
    assert_eq!(runtime_registrations.len(), 1);
    assert_eq!(editor_registrations.len(), 1);
    let capabilities = RuntimeCapabilities::from_runtime_plugin_registrations(
        SessionProfileKind::Editor,
        &runtime_registrations,
    );

    // CR-APP-ENTRY-0003: this real regression remains a hard gate until dynamic bootstrap
    // admits the project's selected navigation provider. CoreOnly currently discards that plan.
    // The native campaign must stage a current admitted Runtime and navigation-enabled project.
    let project_root = std::env::var_os("ZIRCON_NAVIGATION_TEST_PROJECT")
        .map(std::path::PathBuf::from)
        .expect(
            "ZIRCON_NAVIGATION_TEST_PROJECT must select the navigation-enabled campaign project",
        );
    let runtime = Arc::new(
        RuntimeSession::create_with_profile_and_project(
            LoadedRuntime::load_default().expect("load the current admitted Runtime artifact"),
            b"editor",
            Some(project_root.as_path()),
            None,
            None,
            None,
        )
        .unwrap(),
    );
    let gateway: Arc<zircon_editor::core::gateway::SessionGateway> =
        runtime.editor_gateway(capabilities).unwrap();
    let host = EditorRuntimeEventConsumerHost::new(EditorRuntimeGatewayHandle::new(gateway));
    let editor_registration = editor_registrations
        .into_iter()
        .next()
        .expect("the selected navigation editor registration must be present");
    let capability = editor_registration.runtime_event_consumers.manifests()[0]
        .required_capability
        .clone();
    host.register(editor_registration.runtime_event_consumers)
        .unwrap();
    host.begin_play_session(1, &[capability]).unwrap();

    runtime.tick_frame().unwrap();
    assert_eq!(host.pump().unwrap(), 0);
    runtime.tick_frame().unwrap();
    assert_eq!(host.pump().unwrap(), 1);

    host.reconcile_enabled_capabilities(&[]).unwrap();
    runtime.tick_frame().unwrap();
    assert_eq!(host.pump().unwrap(), 0);
    host.end_play_session(1).unwrap();
}
