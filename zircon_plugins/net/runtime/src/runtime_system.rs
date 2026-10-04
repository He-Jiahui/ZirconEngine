//! ECS First 阶段发布 net manager 事件并同步诊断到 Core；Last 阶段保留出站命令收集的调度锚点。
//! 系统通过服务句柄访问 manager，插件登记时无需提前创建 socket。

use zircon_runtime::core::framework::net::NetDiagnostics;
use zircon_runtime::core::manager::{net_manager_handle, resolve_manager_service};
use zircon_runtime::core::{CoreError, CoreHandle};
use zircon_runtime::plugin::{PluginEventManifest, RuntimeExtensionRegistryError};
use zircon_runtime::scene::ecs::RuntimeSceneSystemContext;
use zircon_runtime::scene::SystemStage;

pub const NET_MAIN_SYSTEM_SET: &str = "net.main";
pub const NET_TRANSPORT_SYSTEM_SET: &str = "net.transport";
pub const NET_POLL_INGRESS_SYSTEM: &str = "net.poll_ingress";
pub const NET_FLUSH_EGRESS_SYSTEM: &str = "net.flush_egress";
pub const NET_EVENT_ID: &str = "net.events.runtime_event";
pub const NET_EVENT_SCHEMA: &str = "net.runtime_event.v1";
pub const NET_DIAGNOSTIC_OUTBOUND_BYTES: &str = "net.bandwidth.outbound_bytes";
pub const NET_DIAGNOSTIC_INBOUND_BYTES: &str = "net.bandwidth.inbound_bytes";
pub const NET_DIAGNOSTIC_LAST_LATENCY_MS: &str = "net.latency.last_observed_ms";
pub const NET_DIAGNOSTIC_OPEN_TCP_CONNECTIONS: &str = "net.connections.open_tcp";
pub const NET_DIAGNOSTIC_OPEN_WEBSOCKET_CONNECTIONS: &str = "net.connections.open_websocket";
pub const NET_DIAGNOSTIC_QUEUED_EVENTS: &str = "net.events.queued";
pub const NET_DIAGNOSTIC_PATHS: &[&str] = &[
    NET_DIAGNOSTIC_OUTBOUND_BYTES,
    NET_DIAGNOSTIC_INBOUND_BYTES,
    NET_DIAGNOSTIC_LAST_LATENCY_MS,
    NET_DIAGNOSTIC_OPEN_TCP_CONNECTIONS,
    NET_DIAGNOSTIC_OPEN_WEBSOCKET_CONNECTIONS,
    NET_DIAGNOSTIC_QUEUED_EVENTS,
];
const NET_POLL_INGRESS_EVENT_BUDGET: usize = 256;

pub fn register_runtime_systems(
    module: &mut zircon_plugin_sdk::RuntimePluginModuleRegistration<'_>,
) -> Result<(), RuntimeExtensionRegistryError> {
    module.event::<zircon_runtime::core::framework::net::NetEvent>(PluginEventManifest {
        id: NET_EVENT_ID.to_string(),
        display_name: "Network Runtime Event".to_string(),
        payload_schema: NET_EVENT_SCHEMA.to_string(),
    })?;
    module
        .runtime_scene_system(NET_POLL_INGRESS_SYSTEM, SystemStage::First, || {
            run_net_poll_ingress
        })
        .in_set(NET_MAIN_SYSTEM_SET)
        .in_set(NET_TRANSPORT_SYSTEM_SET)
        .register()?;
    module
        .runtime_scene_system(NET_FLUSH_EGRESS_SYSTEM, SystemStage::Last, || {
            run_net_flush_egress
        })
        .in_set(NET_MAIN_SYSTEM_SET)
        .in_set(NET_TRANSPORT_SYSTEM_SET)
        .register()
}

fn run_net_poll_ingress(context: RuntimeSceneSystemContext<'_>) -> Result<(), CoreError> {
    let Ok(net) = net_manager_handle(context.core)
        .and_then(|handle| resolve_manager_service(context.core, handle))
    else {
        return Ok(());
    };
    let diagnostics = net.diagnostics();
    // BUG: [CR-PLUGIN-NET-0003] 每帧都传 0，滚动诊断历史失去实际帧坐标。
    record_net_diagnostics(context.core, 0, &diagnostics);
    let events = net.drain_events(NET_POLL_INGRESS_EVENT_BUDGET);
    if events.is_empty() {
        return Ok(());
    }

    context.level.with_world_mut(|world| {
        for event in events {
            world.send_event(event);
        }
    });
    Ok(())
}

// Last 阶段暂作稳定调度锚点，供后续帧命令收集路径接入；当前 manager 方法直接提交 worker。
fn run_net_flush_egress(_context: RuntimeSceneSystemContext<'_>) -> Result<(), CoreError> {
    Ok(())
}

pub fn record_net_diagnostics(core: &CoreHandle, frame_index: u64, diagnostics: &NetDiagnostics) {
    core.record_diagnostic(
        NET_DIAGNOSTIC_OUTBOUND_BYTES,
        frame_index,
        diagnostics.outbound_bytes as f64,
        Some("byte"),
        ["net", "bandwidth", "outbound"],
    );
    core.record_diagnostic(
        NET_DIAGNOSTIC_INBOUND_BYTES,
        frame_index,
        diagnostics.inbound_bytes as f64,
        Some("byte"),
        ["net", "bandwidth", "inbound"],
    );
    if let Some(latency_ms) = diagnostics.last_observed_latency_ms {
        core.record_diagnostic(
            NET_DIAGNOSTIC_LAST_LATENCY_MS,
            frame_index,
            latency_ms as f64,
            Some("ms"),
            ["net", "latency"],
        );
    }
    core.record_diagnostic(
        NET_DIAGNOSTIC_OPEN_TCP_CONNECTIONS,
        frame_index,
        diagnostics.open_tcp_connections as f64,
        Some("count"),
        ["net", "connection", "tcp"],
    );
    core.record_diagnostic(
        NET_DIAGNOSTIC_OPEN_WEBSOCKET_CONNECTIONS,
        frame_index,
        diagnostics.open_websocket_connections as f64,
        Some("count"),
        ["net", "connection", "websocket"],
    );
    core.record_diagnostic(
        NET_DIAGNOSTIC_QUEUED_EVENTS,
        frame_index,
        diagnostics.queued_events as f64,
        Some("count"),
        ["net", "event"],
    );
}
