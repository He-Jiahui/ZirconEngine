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

#[cfg(feature = "target-editor-host")]
#[test]
fn runtime_session_satisfies_editor_gateway_thread_safety_contract() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<RuntimeSession>();
}

#[cfg(feature = "target-editor-host")]
#[test]
fn editor_gateway_is_owned_by_session_gateway_instead_of_runtime_session() {
    let runtime_session_source = include_str!("../runtime_session.rs");

    assert!(runtime_session_source.contains("pub(crate) fn editor_gateway("));
    assert!(runtime_session_source.contains("SessionGateway::new_with_identity("));
    assert!(runtime_session_source.contains("self.gateway_identity()"));
    assert!(!runtime_session_source.contains("SessionGateway::new("));
    assert!(runtime_session_source.contains("let owner: Arc<dyn Send + Sync> = self.clone();"));
    assert!(runtime_session_source.contains("self.runtime().editor_gateway_api_table()"));
    assert!(runtime_session_source.contains(".with_viewport_surface_bindings("));
    assert!(runtime_session_source.contains("self.viewport_surface_bindings.clone()"));
    assert!(!runtime_session_source
        .contains("impl zircon_editor::core::gateway::EditorRuntimeGateway for RuntimeSession"));
}

#[cfg(feature = "target-editor-host")]
#[test]
fn editor_gateway_retains_the_app_issued_runtime_session_identity() {
    use zircon_editor::core::gateway::EditorRuntimeGateway;

    let runtime = Arc::new(
        RuntimeSession::create_linked_with_profile_and_project(
            LoadedRuntime::linked().expect("load linked runtime API"),
            b"editor",
            None,
            Vec::new(),
        )
        .expect("create linked runtime session"),
    );
    let expected = runtime.gateway_identity();
    let gateway = runtime
        .editor_gateway(RuntimeCapabilities::editor_default())
        .expect("create editor gateway");

    assert_ne!(expected.runtime_instance(), 0);
    assert_eq!(expected.transport_epoch(), 1);
    assert_eq!(gateway.session_identity(), expected);
}

#[cfg(feature = "target-editor-host")]
#[test]
fn editor_gateway_api_table_excludes_session_lifecycle_but_retains_surface_presentation_authority()
{
    let runtime = LoadedRuntime::linked().expect("load linked runtime API");
    let editor_api = runtime.editor_gateway_api_table();

    assert!(
        editor_api.create_session.is_none(),
        "SessionGateway must not receive session creation authority"
    );
    assert!(
        editor_api.destroy_session.is_none(),
        "SessionGateway must not receive session destruction authority"
    );
    assert!(
        editor_api.release_allocation.is_some(),
        "SessionGateway must receive allocation-release authority"
    );
    assert!(
        editor_api.bind_viewport_surface.is_some(),
        "SessionGateway needs viewport-surface bind authority"
    );
    assert!(
        editor_api.unbind_viewport_surface.is_some(),
        "SessionGateway needs viewport-surface unbind authority"
    );
    assert!(
        editor_api.present_viewport.is_some(),
        "SessionGateway needs viewport-surface present authority"
    );
    assert!(editor_api.drain_host_requests.is_none());

    let runtime_session_source = include_str!("../runtime_session.rs");
    let try_destroy_body = runtime_session_source
        .split("pub(in crate::entry) fn try_destroy")
        .nth(1)
        .and_then(|body| body.split("fn runtime(&self)").next())
        .expect("RuntimeSession should expose retryable explicit teardown");
    let drop_body = runtime_session_source
        .split("impl Drop for RuntimeSession")
        .nth(1)
        .expect("RuntimeSession should own session teardown");
    assert!(try_destroy_body.contains("let destroy_session = self.runtime().destroy_session();"));
    assert!(try_destroy_body.contains("destroy_session(self.handle)"));
    assert!(drop_body.contains("self.teardown_failure_state.record(error);"));
    assert!(
        try_destroy_body.contains("ensure_status(destroy_status, \"destroy runtime session\")?")
    );
    assert!(drop_body.contains("self.try_destroy()"));
    assert!(runtime_session_source.contains("runtime: Option<LoadedRuntime>"));
    assert!(drop_body.contains("abort_after_runtime_session_teardown_failure(&detail);"));
    assert!(!drop_body.contains("std::mem::forget("));
    assert!(runtime_session_source.contains("fn abort_after_runtime_session_teardown_failure"));
    assert!(runtime_session_source.contains("std::process::abort();"));
    assert!(
        !try_destroy_body.contains("let _ = self.unbind_viewport_surface"),
        "RuntimeSession Drop must not discard surface-unbind failures"
    );
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
    let mut editor_registrations =
        crate::entry::first_party_editor_plugin_registrations_for_manifest(
            RuntimeTargetMode::EditorHost,
            &manifest,
        );
    assert_eq!(runtime_registrations.len(), 1);
    assert_eq!(editor_registrations.len(), 1);
    let capabilities = RuntimeCapabilities::from_runtime_plugin_registrations(
        SessionProfileKind::Editor,
        &runtime_registrations,
    );

    let runtime = Arc::new(
        RuntimeSession::create_linked_with_profile_and_project(
            LoadedRuntime::linked().unwrap(),
            b"editor",
            None,
            runtime_registrations,
        )
        .unwrap(),
    );
    let gateway: Arc<zircon_editor::core::gateway::SessionGateway> =
        runtime.editor_gateway(capabilities).unwrap();
    let host = EditorRuntimeEventConsumerHost::new(EditorRuntimeGatewayHandle::new(gateway));
    let editor_registration = editor_registrations.remove(0);
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
