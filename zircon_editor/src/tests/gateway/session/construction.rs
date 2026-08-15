use std::sync::Arc;

use zircon_runtime_interface::ZrRuntimeSessionHandle;

use crate::core::gateway::{
    EditorRuntimeGateway, GatewayError, PluginActivationState, SessionGateway, SessionProfileKind,
};

use super::fixture::{api_table, capabilities, gateway};

#[test]
fn session_gateway_rejects_an_invalid_session_handle() {
    let owner: Arc<dyn Send + Sync> = Arc::new(());
    let error = unsafe {
        SessionGateway::new(
            owner,
            api_table(),
            ZrRuntimeSessionHandle::invalid(),
            capabilities(),
        )
        .expect_err("an invalid runtime session cannot back a gateway")
    };

    assert_eq!(error, GatewayError::SessionLost);
}

#[test]
fn session_gateway_rejects_a_foreign_runtime_api_version() {
    let owner: Arc<dyn Send + Sync> = Arc::new(());
    let mut api = api_table();
    api.abi_version += 1;
    let foreign_version = api.abi_version;
    let error = unsafe {
        SessionGateway::new(owner, api, ZrRuntimeSessionHandle::new(17), capabilities())
            .expect_err("a foreign runtime API cannot back a gateway")
    };

    let GatewayError::Protocol { message } = error else {
        panic!("a foreign runtime API must return a protocol error");
    };
    assert_eq!(
        message,
        format!(
            "session gateway requires runtime API V6, received version {}",
            foreign_version
        )
    );
}

#[test]
fn session_gateway_materializes_canonical_runtime_capabilities() {
    let gateway = gateway(api_table());

    assert_eq!(
        gateway.capabilities().session_profile(),
        SessionProfileKind::Editor
    );
    assert_eq!(
        gateway.capabilities().core_capabilities(),
        &["editor.host.scene_interaction", "editor.host.ui_shell"]
    );
    assert_eq!(gateway.capabilities().plugin_summary().len(), 1);
    assert_eq!(
        gateway.capabilities().plugin_summary()[0].activation(),
        PluginActivationState::Active
    );
}

#[test]
fn session_gateway_rejects_borrowed_world_access_without_calling_runtime() {
    let gateway = gateway(api_table());
    let mut read = |_: &zircon_runtime::scene::World| {};
    let mut write = |_: &mut zircon_runtime::scene::World| {};

    assert_eq!(
        gateway.with_world(&mut read),
        Err(GatewayError::RequiresSerializedAccess)
    );
    assert_eq!(
        gateway.with_world_mut(&mut write),
        Err(GatewayError::RequiresSerializedAccess)
    );
}
