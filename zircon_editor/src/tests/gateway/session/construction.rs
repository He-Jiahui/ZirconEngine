use std::sync::Arc;

use zircon_runtime_interface::{GatewaySessionIdentity, ZrRuntimeSessionHandle};

use crate::core::gateway::{
    EditorRuntimeGateway, GatewayError, PluginActivationState, SessionGateway, SessionProfileKind,
};

use super::fixture::{api_table, capabilities, gateway};

#[test]
fn session_gateway_rejects_an_invalid_session_handle() {
    let owner: Arc<dyn Send + Sync> = Arc::new(());
    let error = unsafe {
        SessionGateway::new_with_identity(
            owner,
            api_table(),
            ZrRuntimeSessionHandle::invalid(),
            GatewaySessionIdentity::new(1, ZrRuntimeSessionHandle::invalid(), 1, None),
            capabilities(),
            Arc::new(zircon_runtime_host::foreign_output::RuntimeForeignOutputState::default()),
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
        SessionGateway::new_with_identity(
            owner,
            api,
            ZrRuntimeSessionHandle::new(17),
            GatewaySessionIdentity::new(17, ZrRuntimeSessionHandle::new(17), 1, None),
            capabilities(),
            Arc::new(zircon_runtime_host::foreign_output::RuntimeForeignOutputState::default()),
        )
        .expect_err("a foreign runtime API cannot back a gateway")
    };

    let GatewayError::Protocol { message } = error else {
        panic!("a foreign runtime API must return a protocol error");
    };
    assert_eq!(
        message,
        format!(
            "runtime API V8 requires version 8, received version {}",
            foreign_version
        )
    );
}

#[test]
fn session_gateway_rejects_an_identity_for_a_different_abi_session() {
    let owner: Arc<dyn Send + Sync> = Arc::new(());
    let error = unsafe {
        SessionGateway::new_with_identity(
            owner,
            api_table(),
            ZrRuntimeSessionHandle::new(17),
            GatewaySessionIdentity::new(18, ZrRuntimeSessionHandle::new(18), 1, None),
            capabilities(),
            Arc::new(zircon_runtime_host::foreign_output::RuntimeForeignOutputState::default()),
        )
        .expect_err("an identity for another ABI session cannot back this gateway")
    };

    assert_eq!(
        error,
        GatewayError::Protocol {
            message: "gateway session identity does not match the ABI session handle".to_owned(),
        }
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
