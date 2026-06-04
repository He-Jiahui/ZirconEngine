use zircon_plugin_net_runtime::DefaultNetManager;
use zircon_runtime::core::framework::net::NetManager;

use crate::http_runtime_manager;

#[test]
fn default_type_can_receive_http_backend_for_direct_tests() {
    let net: DefaultNetManager = http_runtime_manager();

    assert!(net.backend_name().contains("+http"));
}
