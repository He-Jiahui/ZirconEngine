---
related_code:
  - zircon_plugins/net/plugin.toml
  - zircon_plugins/net/runtime/Cargo.toml
  - zircon_plugins/net/runtime/src/lib.rs
  - zircon_plugins/net/runtime/src/config.rs
  - zircon_plugins/net/runtime/src/http.rs
  - zircon_plugins/net/runtime/src/module.rs
  - zircon_plugins/net/runtime/src/package.rs
  - zircon_plugins/net/runtime/src/runtime_state.rs
  - zircon_plugins/net/runtime/src/runtime_system.rs
  - zircon_plugins/net/runtime/src/transport/mod.rs
  - zircon_plugins/net/runtime/src/transport/reconnect.rs
  - zircon_plugins/net/runtime/src/transport/state_machine.rs
  - zircon_plugins/net/runtime/src/transport/tls.rs
  - zircon_plugins/net/runtime/src/service_types.rs
  - zircon_plugins/net/runtime/src/service_types/connections.rs
  - zircon_plugins/net/runtime/src/service_types/diagnostics.rs
  - zircon_plugins/net/runtime/src/service_types/http_routes.rs
  - zircon_plugins/net/runtime/src/service_types/listeners.rs
  - zircon_plugins/net/runtime/src/service_types/tcp.rs
  - zircon_plugins/net/runtime/src/service_types/udp.rs
  - zircon_plugins/net/runtime/src/service_types/websocket.rs
  - zircon_plugins/net/runtime/src/service_types/websocket/backend.rs
  - zircon_plugins/net/runtime/src/service_types/websocket/close.rs
  - zircon_plugins/net/runtime/src/service_types/websocket/connect.rs
  - zircon_plugins/net/runtime/src/service_types/websocket/frames.rs
  - zircon_plugins/net/runtime/src/service_types/websocket/listen.rs
  - zircon_plugins/net/runtime/src/service_types/websocket/loopback.rs
  - zircon_plugins/net/runtime/src/websocket.rs
  - zircon_plugins/net/runtime/src/worker/mod.rs
  - zircon_plugins/net/runtime/src/worker/egress.rs
  - zircon_plugins/net/runtime/src/worker/ingress.rs
  - zircon_plugins/net/runtime/src/worker/shutdown.rs
  - zircon_plugins/net/runtime/src/tests/mod.rs
  - zircon_plugins/net/runtime/src/tests/diagnostics.rs
  - zircon_plugins/net/runtime/src/tests/feature_registration.rs
  - zircon_plugins/net/runtime/src/tests/http_routes.rs
  - zircon_plugins/net/runtime/src/tests/manifest.rs
  - zircon_plugins/net/runtime/src/tests/rpc_descriptor.rs
  - zircon_plugins/net/runtime/src/tests/support.rs
  - zircon_plugins/net/runtime/src/tests/tcp.rs
  - zircon_plugins/net/runtime/src/tests/transport.rs
  - zircon_plugins/net/runtime/src/tests/udp.rs
  - zircon_plugins/net/runtime/src/tests/worker.rs
  - zircon_plugins/net/runtime/src/tests/websocket.rs
  - zircon_plugins/net/editor/Cargo.toml
  - zircon_plugins/net/editor/src/plugin.rs
  - zircon_plugins/net/editor/src/tests/authoring_extensions.rs
  - zircon_plugins/net/features/http/runtime/Cargo.toml
  - zircon_plugins/net/features/http/runtime/src/lib.rs
  - zircon_plugins/net/features/http/runtime/src/feature.rs
  - zircon_plugins/net/features/http/runtime/src/backend.rs
  - zircon_plugins/net/features/http/runtime/src/backend/client.rs
  - zircon_plugins/net/features/http/runtime/src/backend/http1_client_policy.rs
  - zircon_plugins/net/features/http/runtime/src/backend/method.rs
  - zircon_plugins/net/features/http/runtime/src/backend/security.rs
  - zircon_plugins/net/features/http/runtime/src/backend/server.rs
  - zircon_plugins/net/features/http/runtime/src/tests/mod.rs
  - zircon_plugins/net/features/http/runtime/src/tests/backend.rs
  - zircon_plugins/net/features/http/runtime/src/tests/feature_registration.rs
  - zircon_plugins/net/features/http/runtime/src/tests/routes.rs
  - zircon_plugins/net/features/http/runtime/src/tests/security.rs
  - zircon_plugins/net/features/websocket/runtime/Cargo.toml
  - zircon_plugins/net/features/websocket/runtime/src/lib.rs
  - zircon_plugins/net/features/websocket/runtime/src/feature.rs
  - zircon_plugins/net/features/websocket/runtime/src/backend.rs
  - zircon_plugins/net/features/websocket/runtime/src/backend/client.rs
  - zircon_plugins/net/features/websocket/runtime/src/backend/connection.rs
  - zircon_plugins/net/features/websocket/runtime/src/backend/frame.rs
  - zircon_plugins/net/features/websocket/runtime/src/backend/handshake.rs
  - zircon_plugins/net/features/websocket/runtime/src/backend/listener.rs
  - zircon_plugins/net/features/websocket/runtime/src/backend/reader.rs
  - zircon_plugins/net/features/websocket/runtime/src/backend/security.rs
  - zircon_plugins/net/features/websocket/runtime/src/backend/stream.rs
  - zircon_plugins/net/features/websocket/runtime/src/tests/mod.rs
  - zircon_plugins/net/features/websocket/runtime/src/tests/backend.rs
  - zircon_plugins/net/features/websocket/runtime/src/tests/feature_registration.rs
  - zircon_plugins/net/features/websocket/runtime/src/tests/handshake.rs
  - zircon_plugins/net/features/websocket/runtime/src/tests/security.rs
  - zircon_plugins/net/features/websocket/runtime/src/tests/support.rs
  - zircon_plugins/net/features/rpc/runtime/src/lib.rs
  - zircon_plugins/net/features/rpc/runtime/src/feature.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/channel.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/dispatch.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/handshake.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/quota.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/registry.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/session.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/state.rs
  - zircon_plugins/net/features/rpc/runtime/src/tests/mod.rs
  - zircon_plugins/net/features/rpc/runtime/src/tests/channel.rs
  - zircon_plugins/net/features/rpc/runtime/src/tests/dispatch.rs
  - zircon_plugins/net/features/rpc/runtime/src/tests/feature_registration.rs
  - zircon_plugins/net/features/rpc/runtime/src/tests/handlers.rs
  - zircon_plugins/net/features/rpc/runtime/src/tests/queue.rs
  - zircon_plugins/net/features/rpc/runtime/src/tests/session.rs
  - zircon_plugins/net/features/rpc/runtime/src/tests/support.rs
  - zircon_plugins/net/features/replication/runtime/src/lib.rs
  - zircon_plugins/net/features/replication/runtime/src/feature.rs
  - zircon_plugins/net/features/replication/runtime/src/manager.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/apply.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/budget.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/collect.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/interest.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/lifecycle.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/registry.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/schedule.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/snapshot.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/state.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/table.rs
  - zircon_plugins/net/features/replication/runtime/src/tests/mod.rs
  - zircon_plugins/net/features/replication/runtime/src/tests/budget.rs
  - zircon_plugins/net/features/replication/runtime/src/tests/collect_apply.rs
  - zircon_plugins/net/features/replication/runtime/src/tests/delta_interest.rs
  - zircon_plugins/net/features/replication/runtime/src/tests/feature_registration.rs
  - zircon_plugins/net/features/replication/runtime/src/tests/interpolation.rs
  - zircon_plugins/net/features/replication/runtime/src/tests/lifecycle.rs
  - zircon_plugins/net/features/replication/runtime/src/tests/schedule.rs
  - zircon_plugins/net/features/replication/runtime/src/tests/table.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/lib.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/feature.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/manager.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/packet.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/manager/assembly.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/manager/delivery.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/manager/receive.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/manager/recovery.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/manager/resend.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/manager/send.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/manager/state.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/manager/stats.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/tests/mod.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/tests/delivery.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/tests/feature_registration.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/tests/packet.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/tests/receive.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/tests/recovery.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/tests/resend.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/tests/send.rs
  - zircon_plugins/net/features/content_download/runtime/src/lib.rs
  - zircon_plugins/net/features/content_download/runtime/src/feature.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager/attempts.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager/bitmap.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager/http_fetch.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager/manifest.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager/progress.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager/resume.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager/state.rs
  - zircon_plugins/net/features/content_download/runtime/src/tests/mod.rs
  - zircon_plugins/net/features/content_download/runtime/src/tests/attempts.rs
  - zircon_plugins/net/features/content_download/runtime/src/tests/feature_registration.rs
  - zircon_plugins/net/features/content_download/runtime/src/tests/http_fetch.rs
  - zircon_plugins/net/features/content_download/runtime/src/tests/manifest.rs
  - zircon_plugins/net/features/content_download/runtime/src/tests/progress.rs
  - zircon_plugins/net/features/content_download/runtime/src/tests/resume.rs
  - zircon_plugins/net/features/content_download/runtime/src/tests/support.rs
  - zircon_runtime/src/core/framework/net/mod.rs
  - zircon_runtime/src/core/framework/net/download.rs
  - zircon_runtime/src/core/framework/net/http.rs
  - zircon_runtime/src/core/framework/net/manager.rs
  - zircon_runtime/src/core/framework/net/rpc.rs
  - zircon_runtime/src/core/framework/net/sync.rs
  - zircon_runtime/src/core/framework/net/tests.rs
  - zircon_runtime/src/core/framework/net/transport.rs
implementation_files:
  - zircon_plugins/net/plugin.toml
  - zircon_runtime/src/core/framework/net/download.rs
  - zircon_runtime/src/core/framework/net/http.rs
  - zircon_runtime/src/core/framework/net/rpc.rs
  - zircon_runtime/src/core/framework/net/sync.rs
  - zircon_runtime/src/core/framework/net/tests.rs
  - zircon_plugins/net/runtime/Cargo.toml
  - zircon_plugins/net/runtime/src/lib.rs
  - zircon_plugins/net/runtime/src/config.rs
  - zircon_plugins/net/runtime/src/http.rs
  - zircon_plugins/net/runtime/src/module.rs
  - zircon_plugins/net/runtime/src/package.rs
  - zircon_plugins/net/runtime/src/runtime_state.rs
  - zircon_plugins/net/runtime/src/runtime_system.rs
  - zircon_plugins/net/editor/Cargo.toml
  - zircon_plugins/net/editor/src/plugin.rs
  - zircon_plugins/net/editor/src/tests/authoring_extensions.rs
  - zircon_plugins/net/runtime/src/transport/mod.rs
  - zircon_plugins/net/runtime/src/transport/reconnect.rs
  - zircon_plugins/net/runtime/src/transport/state_machine.rs
  - zircon_plugins/net/runtime/src/transport/tls.rs
  - zircon_plugins/net/runtime/src/service_types.rs
  - zircon_plugins/net/runtime/src/service_types/connections.rs
  - zircon_plugins/net/runtime/src/service_types/diagnostics.rs
  - zircon_plugins/net/runtime/src/service_types/http_routes.rs
  - zircon_plugins/net/runtime/src/service_types/listeners.rs
  - zircon_plugins/net/runtime/src/service_types/tcp.rs
  - zircon_plugins/net/runtime/src/service_types/udp.rs
  - zircon_plugins/net/runtime/src/service_types/websocket.rs
  - zircon_plugins/net/runtime/src/service_types/websocket/backend.rs
  - zircon_plugins/net/runtime/src/service_types/websocket/close.rs
  - zircon_plugins/net/runtime/src/service_types/websocket/connect.rs
  - zircon_plugins/net/runtime/src/service_types/websocket/frames.rs
  - zircon_plugins/net/runtime/src/service_types/websocket/listen.rs
  - zircon_plugins/net/runtime/src/service_types/websocket/loopback.rs
  - zircon_plugins/net/runtime/src/websocket.rs
  - zircon_plugins/net/runtime/src/worker/mod.rs
  - zircon_plugins/net/runtime/src/worker/egress.rs
  - zircon_plugins/net/runtime/src/worker/ingress.rs
  - zircon_plugins/net/runtime/src/worker/shutdown.rs
  - zircon_plugins/net/runtime/src/tests/mod.rs
  - zircon_plugins/net/runtime/src/tests/diagnostics.rs
  - zircon_plugins/net/runtime/src/tests/feature_registration.rs
  - zircon_plugins/net/runtime/src/tests/http_routes.rs
  - zircon_plugins/net/runtime/src/tests/manifest.rs
  - zircon_plugins/net/runtime/src/tests/rpc_descriptor.rs
  - zircon_plugins/net/runtime/src/tests/support.rs
  - zircon_plugins/net/runtime/src/tests/tcp.rs
  - zircon_plugins/net/runtime/src/tests/transport.rs
  - zircon_plugins/net/runtime/src/tests/udp.rs
  - zircon_plugins/net/runtime/src/tests/worker.rs
  - zircon_plugins/net/runtime/src/tests/websocket.rs
  - zircon_plugins/net/features/http/runtime/Cargo.toml
  - zircon_plugins/net/features/http/runtime/src/lib.rs
  - zircon_plugins/net/features/http/runtime/src/feature.rs
  - zircon_plugins/net/features/http/runtime/src/backend.rs
  - zircon_plugins/net/features/http/runtime/src/backend/client.rs
  - zircon_plugins/net/features/http/runtime/src/backend/http1_client_policy.rs
  - zircon_plugins/net/features/http/runtime/src/backend/method.rs
  - zircon_plugins/net/features/http/runtime/src/backend/security.rs
  - zircon_plugins/net/features/http/runtime/src/backend/server.rs
  - zircon_plugins/net/features/http/runtime/src/tests/mod.rs
  - zircon_plugins/net/features/http/runtime/src/tests/backend.rs
  - zircon_plugins/net/features/http/runtime/src/tests/feature_registration.rs
  - zircon_plugins/net/features/http/runtime/src/tests/routes.rs
  - zircon_plugins/net/features/http/runtime/src/tests/security.rs
  - zircon_plugins/net/features/websocket/runtime/Cargo.toml
  - zircon_plugins/net/features/websocket/runtime/src/lib.rs
  - zircon_plugins/net/features/websocket/runtime/src/feature.rs
  - zircon_plugins/net/features/websocket/runtime/src/backend.rs
  - zircon_plugins/net/features/websocket/runtime/src/backend/client.rs
  - zircon_plugins/net/features/websocket/runtime/src/backend/connection.rs
  - zircon_plugins/net/features/websocket/runtime/src/backend/frame.rs
  - zircon_plugins/net/features/websocket/runtime/src/backend/handshake.rs
  - zircon_plugins/net/features/websocket/runtime/src/backend/listener.rs
  - zircon_plugins/net/features/websocket/runtime/src/backend/reader.rs
  - zircon_plugins/net/features/websocket/runtime/src/backend/security.rs
  - zircon_plugins/net/features/websocket/runtime/src/backend/stream.rs
  - zircon_plugins/net/features/websocket/runtime/src/tests/mod.rs
  - zircon_plugins/net/features/websocket/runtime/src/tests/backend.rs
  - zircon_plugins/net/features/websocket/runtime/src/tests/feature_registration.rs
  - zircon_plugins/net/features/websocket/runtime/src/tests/handshake.rs
  - zircon_plugins/net/features/websocket/runtime/src/tests/security.rs
  - zircon_plugins/net/features/websocket/runtime/src/tests/support.rs
  - zircon_plugins/net/features/replication/runtime/src/lib.rs
  - zircon_plugins/net/features/replication/runtime/src/feature.rs
  - zircon_plugins/net/features/replication/runtime/src/manager.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/apply.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/budget.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/collect.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/interest.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/lifecycle.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/registry.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/schedule.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/snapshot.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/state.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/table.rs
  - zircon_plugins/net/features/replication/runtime/src/tests/mod.rs
  - zircon_plugins/net/features/replication/runtime/src/tests/budget.rs
  - zircon_plugins/net/features/replication/runtime/src/tests/collect_apply.rs
  - zircon_plugins/net/features/replication/runtime/src/tests/delta_interest.rs
  - zircon_plugins/net/features/replication/runtime/src/tests/feature_registration.rs
  - zircon_plugins/net/features/replication/runtime/src/tests/interpolation.rs
  - zircon_plugins/net/features/replication/runtime/src/tests/lifecycle.rs
  - zircon_plugins/net/features/replication/runtime/src/tests/schedule.rs
  - zircon_plugins/net/features/replication/runtime/src/tests/table.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/lib.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/feature.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/manager.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/packet.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/manager/assembly.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/manager/delivery.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/manager/receive.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/manager/recovery.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/manager/resend.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/manager/send.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/manager/state.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/manager/stats.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/tests/mod.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/tests/delivery.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/tests/feature_registration.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/tests/packet.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/tests/receive.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/tests/recovery.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/tests/resend.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/tests/send.rs
  - zircon_plugins/net/features/rpc/runtime/src/lib.rs
  - zircon_plugins/net/features/rpc/runtime/src/feature.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/channel.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/dispatch.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/handshake.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/quota.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/registry.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/session.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/state.rs
  - zircon_plugins/net/features/rpc/runtime/src/tests/mod.rs
  - zircon_plugins/net/features/rpc/runtime/src/tests/channel.rs
  - zircon_plugins/net/features/rpc/runtime/src/tests/dispatch.rs
  - zircon_plugins/net/features/rpc/runtime/src/tests/feature_registration.rs
  - zircon_plugins/net/features/rpc/runtime/src/tests/handlers.rs
  - zircon_plugins/net/features/rpc/runtime/src/tests/queue.rs
  - zircon_plugins/net/features/rpc/runtime/src/tests/session.rs
  - zircon_plugins/net/features/rpc/runtime/src/tests/support.rs
  - zircon_plugins/net/features/content_download/runtime/src/lib.rs
  - zircon_plugins/net/features/content_download/runtime/src/feature.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager/attempts.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager/bitmap.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager/http_fetch.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager/manifest.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager/progress.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager/resume.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager/state.rs
  - zircon_plugins/net/features/content_download/runtime/src/tests/mod.rs
  - zircon_plugins/net/features/content_download/runtime/src/tests/attempts.rs
  - zircon_plugins/net/features/content_download/runtime/src/tests/feature_registration.rs
  - zircon_plugins/net/features/content_download/runtime/src/tests/http_fetch.rs
  - zircon_plugins/net/features/content_download/runtime/src/tests/manifest.rs
  - zircon_plugins/net/features/content_download/runtime/src/tests/progress.rs
  - zircon_plugins/net/features/content_download/runtime/src/tests/resume.rs
  - zircon_plugins/net/features/content_download/runtime/src/tests/support.rs
plan_sources:
  - user: 2026-06-04 plugin ecosystem infrastructure expansion
  - .codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md
  - docs/plans/zircon_plugins/07-net.md
tests:
  - zircon_plugins/net/runtime/src/tests/mod.rs
  - net_plugin_registration_contributes_runtime_module
  - net_plugin_manifest_advertises_layered_optional_features
  - ingress_anchor_in_first_egress_in_last
  - worker_shutdown_leaves_no_tasks
  - tcp_udp_service_paths_do_not_block_on_tokio_runtime
  - reconnect_backoff_timing_sequence
  - state_changes_emit_events
  - default_net_manager_sends_udp_packet_to_bound_socket
  - net_runtime_manager_accepts_tcp_client_and_echoes_payloads
  - net_runtime_manager_reports_mode_diagnostics_and_events
  - diagnostic_paths_registered
  - net_runtime_diagnostics_records_bandwidth_counters
  - net_runtime_diagnostics_count_listeners_by_transport
  - net_runtime_manager_closes_listeners_across_transports
  - net_runtime_dispatches_registered_http_route
  - net_runtime_queues_websocket_frames_with_budget
  - websocket_connection_send_path_is_queue_driven
  - ws_frame_order_preserved
  - rpc_handshake_frame_round_trips_magic_version_capabilities_and_token
  - handshake_version_mismatch_rejected
  - channels_isolate_message_order
  - wrong_direction_rpc_rejected
  - request_response_timeout_fires
  - rpc_payload_schema_uses_reflect_schema_request
  - bidirectional_rpc_accepts_valid_client_and_server_calls
  - rpc_descriptor_records_direction_schema_and_quota
  - replication_table_compiles_from_descriptors
  - dual_world_replicates_spawn_update_despawn
  - interpolation_window_smooths_updates
  - budget_caps_bytes_per_tick
  - reliable_udp_wire_packet_round_trips_header_ack_bitmap_and_fragment
  - reliable_udp_wire_ack_matches_pending_window_after_u16_wrap
  - thirty_percent_loss_delivers_in_order
  - oversize_payload_fragments_and_reassembles
  - resend_due_with_byte_budget_caps_payload_bytes_per_tick
  - interrupted_download_resumes_from_bitmap
  - corrupt_chunk_refetched
  - reliable_datagram_and_download_contracts_record_recovery_state
  - rpc_feature_manager_closes_sessions_from_transport_events
  - zircon_runtime/src/core/framework/net/tests.rs
  - rustfmt --edition 2021 --check over touched Net event/runtime/test files (passed 2026-06-07)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-net-lifecycle-events-0607 --message-format short --color never (passed 2026-06-07)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime default_net_manager_sends_udp_packet_to_bound_socket --locked --jobs 1 --target-dir E:\cargo-targets\zircon-net-lifecycle-events-0607 --message-format short --color never -- --test-threads=1 --nocapture (passed 2026-06-07)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime net_runtime_manager_closes_listeners_across_transports --locked --jobs 1 --target-dir E:\cargo-targets\zircon-net-lifecycle-events-0607 --message-format short --color never -- --test-threads=1 --nocapture (passed 2026-06-07)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime net_runtime_dispatches_registered_http_route --locked --jobs 1 --target-dir E:\cargo-targets\zircon-net-lifecycle-events-0607 --message-format short --color never -- --test-threads=1 --nocapture (passed 2026-06-07)
  - cargo check -p zircon_runtime --tests --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-net-lifecycle-events-0607-runtime-core-min-check --message-format short --color never (passed 2026-06-07)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-net-transport-events-0607 --message-format short --color never (passed 2026-06-07)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_rpc_runtime --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-net-transport-events-0607 --message-format short --color never (passed 2026-06-07)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_websocket_runtime --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-net-transport-events-0607 --message-format short --color never (passed 2026-06-07)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime net_runtime_manager_accepts_tcp_client_and_echoes_payloads --locked --jobs 1 --target-dir E:\cargo-targets\zircon-net-transport-events-0607 --message-format short --color never -- --test-threads=1 --nocapture (passed 2026-06-07)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime net_runtime_queues_websocket_frames_with_budget --locked --jobs 1 --target-dir E:\cargo-targets\zircon-net-transport-events-0607 --message-format short --color never -- --test-threads=1 --nocapture (passed 2026-06-07)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_rpc_runtime rpc_feature_manager_closes_sessions_from_transport_events --locked --jobs 1 --target-dir E:\cargo-targets\zircon-net-transport-events-0607 --message-format short --color never -- --test-threads=1 --nocapture (passed 2026-06-07)
  - cargo check -p zircon_runtime --tests --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-net-transport-events-0607-runtime-core-min-check --message-format short --color never (passed on rerun 2026-06-07 after a transient concurrent runtime-assembly file-state failure)
  - rustfmt --edition 2021 --check over zircon_plugins/net/runtime/src/service_types.rs, every child file under zircon_plugins/net/runtime/src/service_types/, and every child file under zircon_plugins/net/runtime/src/tests/ (passed 2026-06-04)
  - rustfmt --edition 2021 --check over zircon_plugins/net/features/rpc/runtime/src/manager.rs, every child file under zircon_plugins/net/features/rpc/runtime/src/manager/, and every child file under zircon_plugins/net/features/rpc/runtime/src/tests/ (passed 2026-06-04)
  - rustfmt --edition 2021 --check over zircon_plugins/net/features/content_download/runtime/src/manager.rs, every child file under zircon_plugins/net/features/content_download/runtime/src/manager/, and every child file under zircon_plugins/net/features/content_download/runtime/src/tests/ (passed 2026-06-04)
  - rustfmt --edition 2021 --check over zircon_plugins/net/features/websocket/runtime/src/backend.rs, every child file under zircon_plugins/net/features/websocket/runtime/src/backend/, and every child file under zircon_plugins/net/features/websocket/runtime/src/tests/ (passed 2026-06-04)
  - rustfmt --edition 2021 --check over zircon_plugins/net/features/reliable_udp/runtime/src/manager.rs, every child file under zircon_plugins/net/features/reliable_udp/runtime/src/manager/, and every child file under zircon_plugins/net/features/reliable_udp/runtime/src/tests/ (passed 2026-06-04)
  - rustfmt --edition 2021 --check over zircon_plugins/net/features/http/runtime/src/backend.rs, every child file under zircon_plugins/net/features/http/runtime/src/backend/, and every child file under zircon_plugins/net/features/http/runtime/src/tests/ (passed 2026-06-04)
  - rustfmt --edition 2021 --check over zircon_plugins/net/features/replication/runtime/src/manager.rs and every child file under zircon_plugins/net/features/replication/runtime/src/manager/ (passed 2026-06-04)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-net-service-split-0604 --message-format short --color never (attempted 2026-06-04; timed out before Rust diagnostics)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_rpc_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-net-rpc-manager-split-0604 --message-format short --color never (passed 2026-06-04 for production code with existing zircon_runtime warnings only)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_rpc_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-net-rpc-manager-split-0604 --message-format short --color never (pending test-tree compile validation)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-net-runtime-contract --message-format short --color never (pending while active Cargo lanes are busy)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-worker-m1-0614 --message-format short --color never (passed 2026-06-14 with Cargo.lock restored afterward)
  - D:\cargo-targets\zircon-net-worker-m1-0614\debug\deps\zircon_plugin_net_runtime-e4e45688f0188aad.exe --test-threads=1 --nocapture (18 passed 2026-06-14)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-net-worker-m1-0614 --message-format short --color never (blocked before compile 2026-06-14 by zircon_plugins/Cargo.lock ordering drift)
  - rustfmt --edition 2021 --check over zircon_runtime/src/core/framework/net/http.rs, zircon_runtime/src/core/framework/net/tests.rs, zircon_plugins/net/features/http/runtime/src/backend/{client,method}.rs, zircon_plugins/net/features/http/runtime/src/tests/routes.rs, and zircon_plugins/net/features/content_download/runtime/src/manager/http_fetch.rs (passed 2026-06-14)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_http_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-http-m2-0614 --message-format short --color never (passed 2026-06-14 with Cargo.lock restored afterward)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_http_runtime --offline --jobs 1 --no-run --target-dir D:\cargo-targets\zircon-net-http-m2-0614 --message-format short --color never (passed 2026-06-14 with Cargo.lock restored afterward)
  - D:\cargo-targets\zircon-net-http-m2-0614\debug\deps\zircon_plugin_net_http_runtime-77eb02890f7bcf41.exe --test-threads=1 --nocapture (10 passed 2026-06-14)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_content_download_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-http-m2-0614 --message-format short --color never (passed 2026-06-14 with Cargo.lock restored afterward)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_content_download_runtime --offline --jobs 1 --no-run --target-dir D:\cargo-targets\zircon-net-http-m2-0614 --message-format short --color never (passed 2026-06-14 with Cargo.lock restored afterward)
  - D:\cargo-targets\zircon-net-http-m2-0614\debug\deps\zircon_plugin_net_content_download_runtime-83c7497b4388f3c5.exe --test-threads=1 --nocapture (13 passed 2026-06-14)
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/net.rs::runtime_15_net_http_hyper_http1_client_policy_is_isolated (2026-06-27 Runtime 15 M2 Net HTTP backend Hyper HTTP/1 client policy hard cutover; static guard added; Cargo deferred because external cargo/rustc lanes were active)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-tls-m2-0614 --message-format short --color never (passed 2026-06-14 with Cargo.lock restored afterward)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_http_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-tls-m2-0614 --message-format short --color never (passed 2026-06-14 with Cargo.lock restored afterward)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_http_runtime --offline --jobs 1 --no-run --target-dir D:\cargo-targets\zircon-net-tls-m2-0614 --message-format short --color never (passed 2026-06-14 with Cargo.lock restored afterward)
  - D:\cargo-targets\zircon-net-tls-m2-0614\debug\deps\zircon_plugin_net_http_runtime-4b0511cca9036008.exe --test-threads=1 --nocapture (11 passed 2026-06-14)
  - rustfmt --edition 2021 --check over zircon_plugins/net/features/websocket/runtime/src/backend/{client,connection,listener}.rs and zircon_plugins/net/features/websocket/runtime/src/tests/{backend,handshake,support}.rs (passed 2026-06-14)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_websocket_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never (passed 2026-06-14 with Cargo.lock restored afterward)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_websocket_runtime ws_frame_order_preserved --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never -- --test-threads=1 --nocapture (passed 2026-06-14)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_websocket_runtime websocket_connection_send_path_is_queue_driven --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never -- --test-threads=1 --nocapture (passed 2026-06-14)
  - rustfmt --edition 2021 --check over zircon_plugins/net/features/rpc/runtime/src/lib.rs, manager.rs, manager/handshake.rs, and tests/session.rs (passed 2026-06-14)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_rpc_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never (passed 2026-06-14 with Cargo.lock restored afterward)
  - D:\cargo-targets\zircon-net-ws-m2-0614\debug\deps\zircon_plugin_net_rpc_runtime-2b23712ea6f21e3e.exe rpc_handshake_frame_round_trips_magic_version_capabilities_and_token --test-threads=1 --nocapture (passed 2026-06-14)
  - D:\cargo-targets\zircon-net-ws-m2-0614\debug\deps\zircon_plugin_net_rpc_runtime-2b23712ea6f21e3e.exe handshake_version_mismatch_rejected --test-threads=1 --nocapture (passed 2026-06-14)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_rpc_runtime channels_isolate_message_order --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never -- --test-threads=1 --nocapture (passed 2026-06-14)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_rpc_runtime wrong_direction_rpc_rejected --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never -- --test-threads=1 --nocapture (passed 2026-06-14)
  - D:\cargo-targets\zircon-net-ws-m2-0614\debug\deps\zircon_plugin_net_rpc_runtime-2b23712ea6f21e3e.exe request_response_timeout_fires --test-threads=1 --nocapture (passed 2026-06-14)
  - D:\cargo-targets\zircon-net-ws-m2-0614\debug\deps\zircon_plugin_net_rpc_runtime-2b23712ea6f21e3e.exe rpc_payload_schema_uses_reflect_schema_request --test-threads=1 --nocapture (passed 2026-06-14)
  - D:\cargo-targets\zircon-net-ws-m2-0614\debug\deps\zircon_plugin_net_rpc_runtime-2b23712ea6f21e3e.exe bidirectional_rpc_accepts_valid_client_and_server_calls --test-threads=1 --nocapture (passed 2026-06-14)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime rpc_descriptor_records_direction_schema_and_quota --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never -- --test-threads=1 --nocapture (passed 2026-06-14 with Cargo.lock restored afterward)
  - cargo test -p zircon_runtime --lib rpc_session_and_handshake_descriptors_are_runtime_mode_agnostic --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 -- --test-threads=1 --nocapture (attempted 2026-06-14; stopped during dependency compile with process-level -1/1 and no Rust diagnostics)
  - rustfmt --edition 2021 over zircon_runtime/src/core/framework/net/{sync,mod,tests}.rs and zircon_plugins/net/features/replication/runtime/src/{lib,manager,manager/table,tests/mod,tests/table}.rs (passed 2026-06-14)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_replication_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never (blocked 2026-06-14 before replication crate by unrelated zircon_runtime graphics motion-vector drift; Cargo.lock restored afterward)
  - rustfmt --edition 2021 over zircon_runtime/src/core/framework/net/{sync,tests}.rs and zircon_plugins/net/features/replication/runtime/src/{manager,manager/apply,manager/collect,manager/lifecycle,tests/mod,tests/collect_apply}.rs (passed 2026-06-14)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_replication_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never (passed 2026-06-14 with Cargo.lock restored afterward)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_replication_runtime dual_world_replicates_spawn_update_despawn --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never -- --test-threads=1 --nocapture (blocked 2026-06-14 before replication test execution by unrelated zircon_runtime UI tree_view/BeginEdit compile drift; Cargo.lock restored afterward)
  - rustfmt --edition 2021 over zircon_plugins/net/features/replication/runtime/src/{manager/apply,manager/collect,manager/lifecycle,manager/state,tests/budget,tests/interpolation,tests/mod}.rs (passed 2026-06-14)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_replication_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never (timed out 2026-06-14 after 304s with no Rust diagnostics; Cargo.lock restored afterward)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_replication_runtime --lib --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never (blocked 2026-06-14 before replication crate by unrelated zircon_runtime graphics/skinning private export drift; Cargo.lock restored afterward)
  - rustfmt --edition 2021 --check over zircon_plugins/net/features/reliable_udp/runtime/src/{lib,packet,manager,manager/receive,manager/resend,manager/state,tests/delivery,tests/mod,tests/packet}.rs (passed 2026-06-14)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_reliable_udp_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never (passed 2026-06-14 with Cargo.lock restored afterward)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_reliable_udp_runtime --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never -- --test-threads=1 --nocapture (timed out 2026-06-14 after 304s with no test output; Cargo.lock remained clean)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_reliable_udp_runtime reliable_udp_wire --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never -- --test-threads=1 --nocapture (timed out 2026-06-14 after 604s during zircon_runtime compile/link; target-dir cargo/rustc leftovers were stopped and Cargo.lock remained clean)
  - rustfmt --edition 2021 --check over zircon_plugins/net/features/reliable_udp/runtime/src/{manager,manager/resend,tests/receive,tests/resend}.rs (passed 2026-06-14)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_reliable_udp_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never (passed 2026-06-14 after M5-T2 with Cargo.lock restored afterward)
  - rustfmt --edition 2021 --check over zircon_runtime/src/core/framework/net/{download,mod,tests}.rs and zircon_plugins/net/features/content_download/runtime/src/{manager,manager/bitmap,manager/state,manager/progress,manager/http_fetch,tests/resume,tests/http_fetch}.rs (passed 2026-06-14)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_content_download_runtime --lib --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never (passed 2026-06-14 with Cargo.lock restored afterward)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_content_download_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never (passed 2026-06-14 with Cargo.lock restored afterward)
  - cargo test focused content-download resume/refetch tests under D:\cargo-targets\zircon-net-ws-m2-0614 (timed out 2026-06-14 after 304s with no test output; target-dir content_download processes were stopped and Cargo.lock remained clean)
doc_type: module-detail
---

# Net Runtime Plugin

## Purpose

`zircon_plugins/net/runtime` owns the executable networking service for the first-party Net plugin. It implements the neutral `zircon_runtime::core::framework::net::NetManager` contract with a Tokio-backed base runtime, plugin-owned state, in-memory HTTP route dispatch, loopback WebSocket support, diagnostics, and package metadata for optional network features.

Runtime 15 M2 Net HTTP backend Hyper HTTP/1 client policy hard cutover is recorded as `runtime_15_net_http_hyper_http1_client_policy_hard_cutover_static_passed_cargo_deferred`. The HTTP feature backend now keeps the third-party Hyper HTTP/1 legacy API path isolated in `zircon_plugins/net/features/http/runtime/src/backend/http1_client_policy.rs`; `zircon_plugins/net/features/http/runtime/src/backend/client.rs` only calls `http1_client_policy::plain_http_client()` and remains the request/response/retry/timeout owner. The structure audit classifies the policy owner as allowed `external-hyper-http1-client-policy`, guarded by `runtime_15_net_http_hyper_http1_client_policy_is_isolated`; HTTP behavior is unchanged and Cargo remains deferred while external lanes are active.

The runtime plugin is intentionally not the whole networking stack. HTTP sockets, WebSocket handshakes, RPC, replication, reliable UDP, and content-download behavior are layered as optional features and feature crates. The base plugin contributes the shared manager and the project/export catalog rows that make those features selectable.

## Runtime Boundary

- `plugin.toml` and `runtime_plugin_descriptor()` classify Net as a `runtime` plugin with `beta` maturity and `runtime.plugin.net` capability.
- `module.rs` contributes the `net.runtime` module, `NetDriver`, and `DefaultNetManager` service through the runtime module system.
- `package.rs` contributes options, optional feature rows, dependencies, and event catalog metadata. The static plugin manifest and runtime manifest must stay synchronized.
- `runtime_state.rs` stores plugin-owned Tokio runtime handles, the dedicated `NetWorker`, metadata tables for UDP sockets and TCP listeners/connections, HTTP routes/listeners, WebSocket listeners/connections, and queued events.
- `worker/{mod,egress,ingress,shutdown}.rs` owns the dedicated network worker thread. It keeps Tokio socket/listener/stream handles out of the caller thread and accepts bounded egress commands for UDP/TCP bind, listen, connect, send, poll, close, and shutdown.
- `RuntimePluginRegistrationReport` registers the descriptor-embedded Net module first; `RuntimePluginRegistrationBuilder` then opens the `net.runtime` owner scope for option/catalog metadata and runtime systems without accepting a second descriptor. `runtime_system.rs` registers `net.transport`, `net.poll_ingress`, `net.flush_egress`, and the typed `NetEvent` channel through `RuntimePluginModuleRegistration`; option/catalog metadata, the typed event, and scene systems stay on the SDK module handle rather than direct `PluginModuleId` / `RuntimeExtensionRegistry` calls. D8 runtime registration builder original evidence paths are locked by `review_d8_runtime_registration_builder_original_evidence_paths_use_sdk_builder` and `d8_runtime_registration_builder_original_paths_static_passed_cargo_deferred`. `net.poll_ingress` runs in `SystemStage::First`, publishes copied Net diagnostics into the shared rolling diagnostic store, and drains manager events into the world event queue; `net.flush_egress` is a `SystemStage::Last` scheduling anchor reserved for the later frame-command collection path.
- D5 editor authoring macro consumer guard keeps the editor package on the SDK macro path: `zircon_plugins/net/editor/src/plugin.rs` uses `zircon_plugin_sdk::authoring_plugin!` with `mirrors_runtime_manifest: zircon_plugin_net_runtime::package_manifest()` and only keeps the Net-specific extension registration body outside the macro. Status `d5_editor_authoring_macro_consumers_static_passed_cargo_deferred` is locked by `review_d5_editor_authoring_plugins_use_sdk_macro`.
- D9 editor/runtime mirror consumer guard keeps the editor package tied to this runtime package manifest through the SDK declaration projection: editor tests assert `mirrored_runtime_package_id()`, and the package manifest carries both `zircon_plugin_net_runtime::NET_RUNTIME_CAPABILITY` and the Net authoring capability. `tools/audit_plugin_structure.py --json` reports `editor_runtime_mirror_violations = 0` and `d9_editor_runtime_mirror_gate_status = editor-runtime-mirror-clean`; status `d9_editor_runtime_mirror_consumers_static_passed_cargo_deferred` is locked by `review_d9_editor_runtime_mirror_consumers_use_sdk_declaration`.
- `transport/{mod,reconnect,state_machine,tls}.rs` owns shared transport control helpers. `ReconnectPolicy` provides deterministic exponential retry delays with max-delay capping and optional deterministic jitter, while `TransportStateMachine` centralizes TCP connection state transitions and emits typed `ConnectionStateChanged` events. `transport/tls.rs` centralizes rustls root-store construction, certificate SHA-256 pin calculation/matching, and `TlsServerIdentity` server-config injection for optional transport features.
- `service_types.rs` is the structural manager facade. It owns `DefaultNetManager`, `NetDriver`, id allocation, backend injection, and the `NetManager` trait implementation that delegates to focused service modules.
- `service_types/udp.rs`, `tcp.rs`, and `http_routes.rs` own protocol-specific base runtime operations. UDP/TCP operations now validate local metadata on the manager facade and then route actual socket IO through `NetWorker` commands instead of blocking the caller thread on Tokio; send/poll paths also update outbound and inbound byte counters for diagnostics. HTTP request execution records request bytes, response bytes, and the last observed request latency. `service_types/websocket.rs` is a structural WebSocket service root whose child modules separate optional-backend lookup, real connect calls, real listener accept loops, deterministic loopback pairs, frame send/poll behavior, and close handling; frame send/poll paths feed the same byte counters. `service_types/listeners.rs` and `connections.rs` own cross-protocol listener/connection lifecycle helpers. `service_types/diagnostics.rs` owns copied diagnostics, backend-name projection, worker ingress polling, bounded event draining, and counter projection.
- `tests/mod.rs` is now a structural base runtime test entry. Its child modules separate plugin registration, package manifest rows, UDP loopback, TCP loopback, diagnostics/listener lifecycle, RPC descriptor DTO checks, local HTTP route dispatch, WebSocket loopback behavior, worker shutdown/source-structure guards, and shared polling helpers.
- `http.rs` and `websocket.rs` define plugin-local backend traits/adapters used by optional feature crates.
- Each optional feature crate keeps `src/lib.rs` as a public re-export surface and `src/feature.rs` as the runtime feature-registration surface. Those files own feature IDs, capability names, module descriptors, manager factories, dependencies, and package feature manifests. Backend and manager behavior stays below them instead of accumulating in the feature descriptor.
- `features/http/runtime/src/backend.rs` is now a structural facade for the Hyper/Reqwest HTTP backend. Its child modules separate outbound request/retry execution, HTTP security policy validation, method conversion, and Hyper listener route dispatch/body-limit handling. The outbound path uses hyper for `http://` requests; HTTPS uses reqwest/rustls with optional custom roots and certificate pin verification through the shared TLS helper before response bodies are read. `backend/server.rs` resolves route identity before applying the registered-route body limit; for an unmatched route it drains and discards incoming HTTP/1 frames without allocating a body buffer, then returns 404. This preserves the route-first status contract while preventing an unconsumed request body from racing the response with a connection reset. The facade still exposes `HTTP_ROUTE_REQUEST_BODY_LIMIT_BYTES` for existing public-surface tests. The matching `features/http/runtime/src/tests/` tree separates backend injection, feature registration, socket route behavior, and security-policy assertions.
- `features/websocket/runtime/src/backend.rs` is now a structural facade for the Tungstenite WebSocket backend. Its child modules separate outbound client setup, listener accept/upgrade, server handshake policy, security policy validation, connection state and sending, reader task eventing, shared stream aliases, and frame conversion. `backend/connection.rs` owns the feature-local WebSocket egress worker: client/server constructors move each Tungstenite sink into a Tokio writer task behind a bounded `mpsc` queue, while `send_websocket_frame` only enqueues frames and marks close requests as `Closing`. The matching `features/websocket/runtime/src/tests/` tree separates backend injection, feature registration, real handshake/policy behavior, frame-order guarantees, security-policy assertions, and shared polling helpers.
- `features/replication/runtime/src/manager.rs` is now a structural facade for the replication manager. Its child modules separate component descriptor registration, dense table compilation, session interest filtering, snapshot/delta publication, collect/apply delta flow, scheduling and budget math, despawn lifecycle cleanup, and shared manager state. `manager/table.rs` compiles registered `SyncComponentDescriptor` rows into stable dense entries that retain authority, replication strategy, fields, update frequency, priority, and interest group. `manager/collect.rs` produces `SyncDelta` values for dirty snapshots and tombstone despawns, while `manager/apply.rs` merges sequence-ordered deltas into replica snapshots, ignores stale deltas, and records default 100ms Transform interpolation samples for f32 fields. The matching `features/replication/runtime/src/tests/` tree mirrors those public manager behavior boundaries with focused registration, table, collect/apply, interpolation, delta/interest, lifecycle, schedule, and budget test files.
- `features/reliable_udp/runtime/src/manager.rs` is now a structural facade for the reliable datagram manager. Its child modules separate shared state, inbound fragment assembly, send/fragment queueing, receive/reassembly, simulated delivery, recovery state, resend/ack bookkeeping, and stats access. The matching `features/reliable_udp/runtime/src/tests/` tree mirrors registration, send/ack, delivery simulation, receive assembly, recovery, and resend behavior boundaries.
- `features/rpc/runtime/src/manager.rs` is now a structural facade for the optional RPC feature manager. Its child modules separate long-lived state construction, session lifecycle, control-message handshake, channel multiplexing, descriptor/handler registry, dispatch/queue/pending-request flow, and per-session quota accounting. `manager/channel.rs` keeps per-channel FIFO queues and per-channel sequence numbers for reliable-ordered and unreliable channel flags without mixing channel ordering. `manager/dispatch.rs` now validates RPC direction through `RpcDirection::allows_invocation(...)`, supports bidirectional descriptors, tracks request-response correlation ids through pending requests, reports timeout cleanup, and consumes `RpcPayloadSchema` ids that are backed by the shared `ReflectSchemaRequest` DTO. The matching `features/rpc/runtime/src/tests/` tree mirrors those boundaries with focused registration, session, channel, dispatch, handler, queue, and shared-support test files.
- `features/content_download/runtime/src/manager.rs` is now a structural facade for content-download orchestration. Its child modules separate manifest validation, mirror/attempt selection, partial-range resume storage, explicit resume bitmap storage/application, HTTP request/response validation, progress/cache/cancel mutation, shared state, and chunk hashing. `manager/bitmap.rs` stores an ordered bitmap keyed by download id, derives one from progress when no explicit bitmap exists, and applies completed bits back through cache-hit progress updates. `manager/http_fetch.rs` records hash mismatch attempts and lets a mirror refetch the corrupt chunk before marking the download failed. The matching `features/content_download/runtime/src/tests/` tree mirrors those behavior boundaries with focused feature-registration, manifest, attempt, progress, resume, HTTP-fetch, and shared-support test files. The shared `ZrPackManifest`/`ZrChunkEntry` DTO currently lives in `zircon_runtime::asset::pack`; Runtime 04 handoff `failure-2026-07-13-zrpack-blake3-contract-drift.md` remains open because the shared pack hash is still four seeded FNV-1a values while Content Download still verifies an independent SHA-256 string. The current 15 content-download tests therefore prove the existing resume/refetch behavior but do not satisfy Plugins 07 M6-T2's single BLAKE3 contract.

Runtime callers consume framework DTOs and manager handles. They should not depend on `DefaultNetManager` internals, Tokio types, route tables, or WebSocket connection implementations.

## Feature Shape

The base plugin exposes six optional features:

- `net.http` for HTTP(S) runtime backend support.
- `net.websocket` for WebSocket runtime backend support.
- `net.rpc` for remote/gameplay RPC contracts.
- `net.replication` for state replication contracts.
- `net.reliable_udp` for reliable datagram behavior above UDP.
- `net.content_download` for content download manifests and range-resume flow, depending on both `runtime.plugin.net` and `runtime.feature.net.http`.

The optional feature rows are not a promise that default client profiles listen on ports. They are profile/export selections. Server, editor, and dev profiles can opt into transports deliberately, while ordinary client profiles can keep the Net plugin present for outbound or data-only features without opening listeners.

## Behavior Model

`DefaultNetManager` supports deterministic base behavior:

- UDP bind/send/poll/close on loopback or configured endpoints.
- TCP listen/connect/accept/send/poll/close.
- In-memory HTTP route registration and route handler dispatch for URLs without an explicit socket port.
- Typed unavailable-feature errors for real HTTP listener/outgoing socket backend operations when no HTTP backend is installed.
- WebSocket loopback pairs for deterministic frame queue tests.
- Typed unavailable-feature errors for real WebSocket listen/connect behavior when no WebSocket backend is installed.
- Copied `NetDiagnostics` and bounded event draining, including explicit close/unregister lifecycle events for UDP sockets, TCP/HTTP/WebSocket listeners, TCP/WebSocket connections, queued WebSocket frames, and HTTP routes.

This gives tests, tooling, and editor diagnostics a functional baseline while leaving production HTTP/WebSocket implementation behind optional feature crates.

UDP and TCP socket work now lives behind the dedicated `NetWorker`. The manager facade allocates ids and keeps metadata tables for diagnostics and handle validation, while the worker owns Tokio `UdpSocket`, `TcpListener`, and `TcpStream` values on its own thread. Caller-facing bind/listen/connect/send/poll/close methods send bounded commands to the worker and wait only for that command's reply; `service_types/{tcp,udp}.rs` intentionally contains no `.block_on(...)` calls. The worker emits `NetEvent` values through ingress, and diagnostics/event drains fold those events into the existing manager event queue.

Shutdown is explicit. `NetWorker::shutdown()` sends a shutdown command, records how many UDP sockets, TCP listeners, and TCP connections were still open, joins the worker thread, and then rejects later commands with `NetError::Io`. TCP listener and connection close helpers keep local metadata until the worker close succeeds, so failed worker commands do not silently erase manager-visible handles.

Transport state transitions are driven inside the worker. Outbound TCP connect commands emit `Connecting` before the async connect attempt, then `Open` on success or `Failed` on connection error. Accepted TCP streams start from the same state-machine helper and publish `Open`. Active close emits `Closing`, then `Closed`, then the lifecycle `ConnectionClosed` event. Remote EOF transitions to `Closed` during `poll_tcp`. Reconnect scheduling is not yet attached to automatic reconnect loops; `ReconnectPolicy` is the deterministic M1 policy primitive that M2/M3 transport/session work can consume.

The manager service boundary is intentionally folder-backed. Godot keeps UDP packet peers, TCP servers/streams, HTTP clients, and WebSocket peers as separate runtime services; Bevy Remote separates protocol registration from HTTP transport startup. Zircon translates that into one public `NetManager` facade with protocol-specific implementation modules below it, so future RPC, replication, reliable UDP, and content-download work can extend the network stack without appending more behavior to one mixed service file.

`NetDiagnostics` is copied from the manager's actual handle tables and now reports listener ownership by transport: `open_tcp_listeners`, `open_http_listeners`, and `open_websocket_listeners`. This keeps editor network inspectors and export/profile diagnostics from treating "listener count" as TCP-only while optional HTTP and WebSocket backends are installed. HTTP and WebSocket listener counts still stay zero when those feature backends are absent, matching the profile-gated transport model.

Lifecycle events are intentionally emitted from the concrete close/unregister paths instead of inferred from diagnostics snapshots. UDP/TCP lifecycle events come from the worker-owned socket/listener/connection tables, while HTTP/WebSocket local lifecycle events are queued by their protocol modules. `close_socket`, `close_listener`, `close_connection`, and `unregister_http_route` queue `NetEvent` records such as `UdpSocketClosed`, `ListenerClosed`, `ConnectionClosed`, or `HttpRouteUnregistered` after the concrete owner accepts the lifecycle operation. Connection accept and close events carry their `NetTransportKind`, so network inspectors, RPC session cleanup, and export/profile diagnostics can classify TCP versus WebSocket lifecycle changes without looking up a connection that may already have been removed from the manager table.

The base WebSocket service follows the same rule below the protocol module. Godot keeps WebSocket peer state, packet buffering, multiplayer integration, debugger peers, and platform implementations separate; Bevy Remote keeps remote protocol registration independent from transport startup; Fyrox keeps listener/stream wrappers narrow. Zircon keeps `DefaultNetManager` as the only public manager while splitting base WebSocket behavior into backend resolution, connect/listen/accept, loopback, frame queueing, and close modules. This keeps deterministic test loopbacks, optional backend calls, listener polling, frame budgets, and state-close semantics independent before future browser transports, compression, subprotocol dispatch, RPC bridging, or editor network inspectors grow this service area.

The optional HTTP backend follows the same boundary rule. Godot keeps HTTP client behavior separate from request-node dispatch, and Bevy Remote keeps HTTP transport startup distinct from protocol method registration. Zircon keeps `HyperReqwestHttpBackend` as the public optional-feature backend while moving hyper HTTP client send/retry behavior, reqwest/rustls HTTPS handling, security policy checks, certificate pin validation, Hyper listener dispatch, body-limit handling, and method conversion into child modules. The test tree follows route, security, backend-injection, and registration boundaries so future proxy policy, streaming bodies, authentication, and route middleware can add coverage without growing one backend test file.

An HTTP/1 request body remains part of the connection lifecycle even when no Zircon route matches. Dropping Hyper's `Incoming` body and immediately returning 404 was timing-dependent: the client could observe `SendRequest` when the connection closed before its full body finished sending. The server now discards unmatched body frames incrementally before building the 404 response. Registered routes still use `Limited` and return 413 before handler dispatch when the 1 MiB limit is exceeded; unmatched routes remain 404 because route selection occurs first, and the discard path does not allocate memory proportional to payload size.

The optional WebSocket backend follows the same internal boundary rule. Godot's WebSocket module separates peer, multiplayer, debugger, packet buffering, and platform-specific WebSocket implementations; Bevy Remote keeps transport startup outside the method registry. Zircon keeps `TungsteniteWebSocketBackend` as the public optional-feature backend while moving client connection setup, listener upgrade, handshake policy, security checks, connection state, reader tasks, and frame conversion into child modules. The test tree follows backend-injection, registration, handshake/policy, security, and shared polling-helper boundaries so future TLS policy, compression, close-code handling, RPC subprotocols, and browser/platform variants do not accumulate in one backend test file.

The optional Replication manager is split around the same multiplayer data-flow boundaries. Godot separates scene replication config, replication interface, synchronizers, spawners, and editor tooling; Unreal/Iris separates replication state, filtering, scheduling, and replication system responsibilities. Zircon keeps `NetReplicationRuntimeManager` as the public optional-feature facade while separating descriptor registration, interest filtering, snapshot/delta publication, collect/apply state transfer, interpolation-buffer state, budgeted scheduling, despawn cleanup, and shared state. `SyncDelta` now carries a serde-default tombstone marker for despawn propagation, so the same sequence stream can represent spawn/update field merges and object/component removal without a second plugin-local deletion DTO. Transform-like components record receive-time f32 samples during apply and expose a 100ms delayed interpolation query so M4 has a deterministic smoothing primitive before ECS-facing interpolation systems land. The test tree follows the same split so authority policy, channel ownership, baseline compression, and interest-grid expansion can add coverage without reviving a monolithic module-local test file.

The optional Reliable UDP manager is split around the same runtime data-flow boundaries. Godot's ENet and packet-peer surfaces keep transport, packet buffering, and higher-level multiplayer responsibilities distinct, while Bevy-style networking crates usually separate channels, send queues, receive assembly, and reliability bookkeeping. Zircon keeps `NetReliableUdpRuntimeManager` as the public optional-feature facade while separating the fixed wire packet header, fragment assembly, outbound queueing, acknowledgement, resend timing, resend byte-budget selection, simulated delivery, ordered receive delivery, recovery status, and stats access. `ReliableUdpWirePacket` encodes the plan-defined `seq/ack/ack_bits/channel/flags` header in little-endian form with an optional fragment header, and `acknowledge_wire_header(...)` projects ack bitmaps back into the existing resend queue. `resend_due_with_byte_budget(...)` limits per-tick resend payload bytes without dropping deferred due packets, so higher-level session/replication code can cap recovery traffic per connection. The test tree follows packet/header, send/ack, simulated delivery, receive assembly, recovery, resend, and registration boundaries so future channel QoS, congestion policy, packet windows, and transport adapters do not grow one mixed manager test file.

The optional RPC manager follows the same boundary rule. Bevy Remote separates method registry, protocol message processing, and transport startup; Godot keeps JSON-RPC, scene RPC, and multiplayer synchronization as distinct runtime surfaces. Zircon keeps `NetRpcRuntimeManager` as the public optional-feature facade while moving handshake/session logic, registry, dispatch queue, pending-request correlation, and quota tracking into child modules. This keeps future reflection-backed RPC discovery, authority policies, and transport adapters from growing one mixed manager file.

The optional content-download manager uses the same facade-plus-child-modules rule. Godot separates low-level HTTP clients from request nodes, Bevy separates asset IO from transport concerns, and Unreal's platform downloader separates progress/service/manifest responsibilities. Zircon keeps `NetContentDownloadRuntimeManager` as the feature-facing manager while separating manifest validation, attempts/mirrors, HTTP fetch validation, resume prefixes, progress state, and hashing so future CDN policy, cache storage, and asset-pipeline integration do not grow one mixed file. Range resume now builds requests through `NetHttpRequestDescriptor::with_byte_range(...)`, so content-download and future package-fetch code share the same HTTP Range contract instead of hand-formatting headers. The test tree follows the same split so adding cache policy, CDN selection, or asset-pipeline integration tests does not revive a monolithic module-local test file.

## Reference Alignment

Bevy Remote is the strongest profile/reference precedent: `RemotePlugin` registers methods, but an additional `RemoteHttpPlugin` starts the HTTP transport. Zircon follows the same split at a broader networking scale. The base Net plugin contributes shared protocol/service infrastructure; transports and advanced gameplay systems stay optional.

Godot provides the broad engine precedent for multiple network surfaces: HTTP client, UDP packet peer, TCP server, ENet/multiplayer, and WebSocket peer are distinct runtime concepts. Zircon keeps those distinctions in package features and framework DTOs but exposes one manager facade for engine consumers.

Godot's low-level UDP peer and TCP server APIs expose explicit bind/listen, close/stop, local-port, and status queries, while `StreamPeerTCP` tracks connection status transitions. Zircon does not mirror those object APIs directly, but it preserves the same lifecycle boundary by emitting typed events when manager-owned sockets, listeners, routes, and connections start or close, and by projecting per-transport listener counts through `NetDiagnostics`.

Fyrox provides the Rust-native precedent for small listener/stream abstractions. Zircon keeps simple TCP/UDP behavior directly testable in the base runtime and avoids forcing all higher-level protocols into that low-level stream abstraction.

## Edge Cases

The base runtime should fail explicitly when an optional backend is absent. `listen_http`, real outbound HTTP requests with explicit ports, `listen_websocket`, and real WebSocket connect calls return `NetError::ProtocolUnavailable` until a backend is installed. This keeps missing capabilities visible to export/profile diagnostics instead of silently running partial behavior.

HTTP route handlers are deterministic local dispatch only. They are useful for tests, editor mock endpoints, and local tools, but they are not a replacement for the optional HTTP runtime backend. WebSocket loopback pairs are similarly deterministic test infrastructure, not an external network listener.

## Test Coverage

Runtime tests cover plugin module registration, package optional features, runtime system/event registration, UDP loopback packets plus socket-close events, TCP loopback accept/send/poll plus transport-qualified accept/close events, TCP state-change events, reconnect backoff timing, diagnostics/events, per-transport listener counts through fake HTTP/WebSocket backends, listener close events, local HTTP route dispatch plus route-unregister events, dynamic HTTP route handlers, explicit HTTP backend absence, WebSocket loopback frame budgets, transport-qualified WebSocket close events, explicit WebSocket backend absence, worker shutdown accounting, and a source-structure guard that keeps TCP/UDP service facade code free of direct Tokio `.block_on(...)` calls. Optional RPC coverage keeps session cleanup transport-agnostic while consuming the richer close event, and optional WebSocket backend reader code emits WebSocket-qualified close events for real transport tasks. The base runtime test tree follows those same boundaries so adding protocol-specific assertions does not revive a monolithic `tests.rs`.

M7 runtime diagnostics tests add `diagnostic_paths_registered` and `net_runtime_diagnostics_records_bandwidth_counters`. The first locks the exported `NET_DIAGNOSTIC_*` path constants, units, current values, and subsystem tags as they are written into `CoreRuntime`'s rolling diagnostic store. The second uses WebSocket loopback frames to prove runtime outbound and inbound byte counters flow through `NetDiagnostics` without requiring a real socket backend.

Optional HTTP feature tests now cover plan-named M2-T1/M2-T2 behavior: `http_round_trip_against_local_hyper_server` proves the feature manager can serve and fetch through the local Hyper listener with the hyper client path, `range_request_returns_partial` proves `NetHttpRequestDescriptor::with_byte_range(...)` reaches the route handler and preserves `Content-Range` response metadata, and `self_signed_cert_rejected_then_pinned_accepted` proves an unpinned self-signed HTTPS server is rejected before a matching host certificate pin allows the response. Optional content-download tests continue to cover resumed HTTP range fetches with existing prefixes through that shared request contract.

The route/body lifecycle regression is locked by `http_feature_manager_matches_route_before_applying_body_limit`. Before the server fix, the exact test failed 2/100 standalone runs and the HTTP suite failed 3/10 parallel runs with `Io("client error (SendRequest)")`; after the unmatched-body discard fix, the exact test passed 200/200 and the eight-thread HTTP suite passed 30/30. The final Windows testing stage passed HTTP 11/11, Replication 9/9, RPC 27/27, and WebSocket 8/8 under the locked offline plugin workspace, in addition to the earlier Content Download 15/15, Reliable UDP 12/12, and base Net runtime 21/21. The shared BLAKE3 handoff remains excluded from those pass claims.

Optional WebSocket feature tests now cover the M2-T3 send-worker slice. `ws_frame_order_preserved` sends multiple text/binary frames in both directions over a real Tungstenite handshake and polls until the peer observes the exact order. `websocket_connection_send_path_is_queue_driven` is a source-structure guard for the feature backend: frame sends must use the bounded queue and must not reintroduce caller-thread `block_on`.

Optional RPC feature tests now cover the M3-T1 byte-level handshake slice. `NetRpcHandshakeFrame` encodes the plan-defined control frame as `ZRPC` magic, u16 protocol version, u64 capability bits, u16 token length, and token bytes, then maps supported capability bits into the existing `NetControlMessage::Hello` policy path. `rpc_handshake_frame_round_trips_magic_version_capabilities_and_token` covers the binary round trip, and `handshake_version_mismatch_rejected` proves the byte-frame path still rejects unsupported protocol versions through the shared session failure state.

Optional RPC feature tests now also cover the M3-T2 channel multiplexing slice. `channels_isolate_message_order` interleaves messages into two `u8` channels, drains each channel independently, and verifies each channel's sequence numbers and FIFO payload order are isolated from the other channel. This is the data-plane foundation for the later session/RPC channel header that carries `channel_id` and `flags`.

Optional RPC feature tests now cover the M3-T3 dispatch slice. `wrong_direction_rpc_rejected` locks direction and caller-role rejection, `bidirectional_rpc_accepts_valid_client_and_server_calls` proves `RpcDirection::Bidirectional` accepts valid client-to-server and server-to-client calls while still rejecting mismatched caller roles, `request_response_timeout_fires` locks request id correlation through timeout cleanup, and `rpc_payload_schema_uses_reflect_schema_request` proves RPC payload schemas use the shared `ReflectSchemaRequest` DTO from the 08 reflection plan instead of introducing a second schema description. The base Net runtime descriptor test also covers `RpcPayloadSchema` and `Bidirectional` as public framework contracts.

Focused validation after landing the M3-T3 RPC dispatch slice ran on 2026-06-14. `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_rpc_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never` passed with existing warning noise while root and plugin lockfiles were restored. `wrong_direction_rpc_rejected` passed through Cargo, and direct warmed test-binary runs passed `request_response_timeout_fires`, `rpc_payload_schema_uses_reflect_schema_request`, and `bidirectional_rpc_accepts_valid_client_and_server_calls`. `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime rpc_descriptor_records_direction_schema_and_quota --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never -- --test-threads=1 --nocapture` also passed with lockfiles restored. The framework-level `zircon_runtime` focused test was attempted under `--no-default-features --features core-min --locked`, but dependency compilation stopped twice with process-level `-1`/`1` and no Rust diagnostics; no framework test execution pass is claimed for that command.

Optional replication feature tests now include the M4-T1 table-compilation slice. `replication_table_compiles_from_descriptors` registers server and client-owned component descriptors, constructs a `NetworkIdentity`, and verifies `compile_replication_table()` emits deterministic dense indexes plus authority, strategy, field, update-frequency, priority, and interest-group projections. The slice also extends the shared Net sync DTOs with `SyncReplicationStrategy` so the later collect/apply path can distinguish OnChange, Interval, and Once policies without introducing a plugin-local strategy enum.

Focused validation after landing the M4-T1 replication table slice was attempted on 2026-06-14. `rustfmt --edition 2021` passed over the touched Net framework and replication feature files. The first `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_replication_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never` attempt was blocked before reaching the replication crate by unrelated `zircon_runtime` graphics motion-vector drift (`ViewportMotionVectorObjectHistory` / `previous_motion_vector_object_history` missing from the active render state). Root and plugin lockfiles were restored. A later M4-T2 scoped check on the same replication crate did compile the replication test target successfully, so M4-T1 has type-check coverage but still no focused test-execution pass.

Optional replication feature tests now include the M4-T2 collect/apply slice. `dual_world_replicates_spawn_update_despawn` uses source and replica managers with the same `SyncComponentDescriptor`, collects a spawn delta, applies it to the replica, applies a one-field update without dropping unchanged fields, verifies a stale spawn delta does not roll back the newer value, then propagates a tombstone despawn delta and confirms late-join snapshots are empty on both sides. This locks the manager-level state-transfer contract before M4-T3 adds interpolation buffers and replication-budget behavior.

Focused validation after landing the M4-T2 collect/apply slice ran on 2026-06-14. `rustfmt --edition 2021` passed over the touched Net sync DTO, replication manager facade, collect/apply/lifecycle modules, and collect/apply tests. `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_replication_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never` passed with existing warning noise while restoring root and plugin lockfiles afterward. Focused `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_replication_runtime dual_world_replicates_spawn_update_despawn --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never -- --test-threads=1 --nocapture` is currently blocked before replication test execution by unrelated UI compile drift in `zircon_runtime/src/ui/component/state_reducer/{keyboard.rs,tree_view.rs}` and `zircon_runtime/src/ui/surface/surface/default_interactions.rs`; no focused test pass is claimed for that command.

Optional replication feature tests now include the M4-T3 interpolation and byte-budget aliases. `interpolation_window_smooths_updates` applies two Transform f32 deltas at 0ms and 100ms, then queries the default 100ms interpolation window at 150ms and 250ms to prove the replica reports the midpoint and then the latest value. `budget_caps_bytes_per_tick` locks the existing `SyncReplicationBudget::max_bytes` path under the plan name by sending only the first snapshot that fits the per-tick byte budget and deferring the next one.

Focused validation after landing the M4-T3 interpolation/budget slice ran on 2026-06-14. `rustfmt --edition 2021` passed over the touched replication apply/collect/lifecycle/state modules and tests. `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_replication_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never` timed out after 304 seconds without Rust diagnostics. The narrower `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_replication_runtime --lib --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never` reached `zircon_runtime` and then stopped before the replication crate on unrelated graphics/skinning private export drift: `SKINNED_MESH_MAX_JOINT_MATRICES` is private but re-exported, and `gpu_scene/prev_skinned_palette.rs` accesses a private `mesh` module. Root and plugin lockfiles were restored, and no Cargo pass is claimed for M4-T3 until that external compile drift is resolved.

Optional Reliable UDP feature tests now include the M5-T1 wire-header and sliding resend slice. `reliable_udp_wire_packet_round_trips_header_ack_bitmap_and_fragment` covers the little-endian sequence/ack/ack-bitmap/channel/flags header, optional fragment header, payload round trip, and ack bitmap expansion. `reliable_udp_wire_ack_matches_pending_window_after_u16_wrap` confirms the wire `u16` ack still matches the current pending resend window after the internal `u64` sequence crosses the 16-bit boundary. `thirty_percent_loss_delivers_in_order` sends ten one-byte datagrams through a deterministic 30% loss profile, observes the receiver withholding gaps after the first delivery pass, then uses `resend_due(...)` to fill the window and deliver payloads in sequence order.

Focused validation after landing the M5-T1 Reliable UDP slice ran on 2026-06-14. `rustfmt --edition 2021 --check` passed over the touched reliable_udp facade, packet, manager, receive/resend/state modules, and tests. Path-scoped `git diff --check`, conflict-marker scans, and trailing-whitespace scans were clean except for expected LF/CRLF warnings. `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_reliable_udp_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never` passed with existing warning noise while restoring root and plugin lockfiles. Full package test execution timed out after 304 seconds with no test output, and the narrower `reliable_udp_wire` focused test timed out after 604 seconds while compiling/linking `zircon_runtime`; target-dir-specific reliable_udp Cargo/rustc leftovers were stopped, unrelated active Cargo lanes were left alone, and root/plugin lockfiles remained clean. No test-execution pass is claimed for M5-T1 until those focused tests complete.

Optional Reliable UDP feature tests now also include the M5-T2 fragment/reassembly and resend-budget slice. `oversize_payload_fragments_and_reassembles` locks a 10-byte payload over a 4-byte MTU into three fragments, then receives them out of order and proves the final payload is reassembled exactly once. `resend_due_with_byte_budget_caps_payload_bytes_per_tick` locks the per-tick resend byte-budget path by allowing only one 4-byte payload through a 4-byte budget while leaving the next due payload available for the following resend call.

Focused validation after landing the M5-T2 Reliable UDP slice ran on 2026-06-14. `rustfmt --edition 2021 --check` passed over the touched reliable_udp manager/resend and receive/resend test files. Path-scoped `git diff --check`, conflict-marker scans, and trailing-whitespace scans stayed clean except for expected LF/CRLF warnings. `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_reliable_udp_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never` passed again with existing warning noise while restoring root and plugin lockfiles. Test execution remains pending because the immediately preceding reliable_udp `cargo test` attempts timed out during compile/link; no M5-T2 test-execution pass is claimed yet.

Optional content-download feature tests now include the M6 shared-manifest and resume/refetch slice. The framework test `reliable_datagram_and_download_contracts_record_recovery_state` covers `ZrPackManifest` and `ZrChunkEntry` serde round-trip behavior, chunk end offsets, covered byte totals, and complete byte-plan detection. `interrupted_download_resumes_from_bitmap` covers applying a stored ordered bitmap after an interrupted download, while `corrupt_chunk_refetched` covers a corrupt first mirror chunk recording a failed attempt and then completing from a backup mirror.

The Content Download hash contract is a hard cut to the Runtime-owned ZrPack BLAKE3 digest. `NetDownloadChunk` carries `content_hash: [u8; 32]`, and the HTTP fetch path verifies response bytes with `zircon_runtime::asset::pack::zrpack_content_hash`. The former plugin-local `manager/hash.rs`, `ring::digest::SHA256`, hexadecimal string field, and dual-algorithm interpretation are removed. `corrupt_chunk_refetched` now constructs its expected digest through a real `ZrPackManifest` / `ZrChunkEntry` before exercising primary corruption and mirror recovery.

Focused validation after landing the M6 Content Download slice ran on 2026-06-14. `rustfmt --edition 2021 --check` passed over the touched framework Net download facade/tests plus content-download manager bitmap/state/progress/http-fetch and tests. Path-scoped `git diff --check`, conflict-marker scans, and trailing-whitespace scans were clean except for expected LF/CRLF warnings. `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_content_download_runtime --lib --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never` passed, and the matching `--tests --offline` check also passed, with root and plugin lockfiles restored afterward. The focused resume/refetch test execution timed out after 304 seconds with no test output; target-dir-specific content_download processes were stopped, unrelated Cargo lanes were left alone, and no focused test-execution pass is claimed for M6 yet.

Focused validation after landing the M7 editor/runtime diagnostics slice ran on 2026-06-14. `rustfmt --edition 2021 --check` passed over the touched Net editor/runtime/framework files, and path-scoped `git diff --check` was clean except for expected LF/CRLF warnings. A lockfile-protected `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never` passed once after implementation with existing warning noise and restored locks. The matching Net editor package check also passed once. A later final rerun was blocked before reaching Net runtime/editor tests by an unrelated untracked UI file at `zircon_runtime/src/ui/surface/surface/default_interactions/tree_view.rs` with duplicate definitions and missing helper symbols. Root and plugin lockfiles remained clean, and no final rerun pass is claimed until that external UI drift is resolved.

Focused validation after landing the M3-T2 RPC channel multiplexing slice ran on 2026-06-14. `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_rpc_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never` passed with existing warning noise, and `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_rpc_runtime channels_isolate_message_order --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never -- --test-threads=1 --nocapture` passed the focused channel-order test. Root and plugin lockfiles were restored afterward.

Focused validation after landing the M3-T1 RPC handshake frame slice ran on 2026-06-14. `rustfmt --edition 2021 --check` passed over the touched RPC facade, manager, handshake, and session-test files. A lockfile-backup protected `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_rpc_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never` passed with existing `zircon_runtime` / `zircon_plugin_net_runtime` warning noise while restoring root and plugin lockfiles afterward. The outer Cargo command for two focused RPC tests timed out during compile/wait, but it had produced a warmed RPC test binary; direct execution of that binary passed both `rpc_handshake_frame_round_trips_magic_version_capabilities_and_token` and `handshake_version_mismatch_rejected`. Target-dir-specific Cargo/rustc leftovers were stopped and lockfiles remained clean.

Focused validation after landing the M2-T3 WebSocket send-worker slice ran on 2026-06-14. `rustfmt --edition 2021 --check` passed over the touched WebSocket backend and test files; conflict-marker, trailing-whitespace, and send-path fallback scans were clean. The first scoped check hit an active render-session dependency drift, and a later combined WebSocket/RPC test command timed out; both were cleaned up without leaving lockfile changes. The smaller rerun `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_websocket_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never` passed with existing warning noise, and focused Cargo tests passed for `ws_frame_order_preserved` and `websocket_connection_send_path_is_queue_driven`.

Focused validation after landing the M2-T2 rustls roots/pin slice ran on 2026-06-14. `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-tls-m2-0614 --message-format short --color never` and the matching `zircon_plugin_net_http_runtime --tests` check both passed with existing `zircon_runtime` warnings while restoring `zircon_plugins/Cargo.lock` afterward. `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_http_runtime --offline --jobs 1 --no-run --target-dir D:\cargo-targets\zircon-net-tls-m2-0614 --message-format short --color never` needed a warmed rerun after the first 15-minute link timeout, then passed and produced `zircon_plugin_net_http_runtime-4b0511cca9036008.exe`; direct execution of that warmed binary passed all 11 HTTP feature tests including the self-signed pin test.

Focused validation after landing the M2-T1 HTTP client and Range request slice ran on 2026-06-14. `rustfmt --edition 2021 --check` passed over the touched framework, HTTP feature, and content-download files. Lockfile-backup protected `cargo check` and `cargo test --no-run` passed for `zircon_plugin_net_http_runtime` and `zircon_plugin_net_content_download_runtime` under `--offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-http-m2-0614`, while restoring `zircon_plugins/Cargo.lock` afterward. Direct warmed test binary runs passed all 10 HTTP feature tests and all 13 content-download tests. Focused `zircon_runtime` contract-test builds for `http_and_websocket_descriptors_keep_protocol_state_data_only` timed out in cold target-dir compilation/linking under both the root workspace target and plugin workspace target, so no framework test execution pass is claimed; matching cargo/rustc processes were stopped and root/plugin lockfiles were restored.

Focused validation after landing the M1 NetWorker and transport-state baseline ran on 2026-06-14. `rustfmt --edition 2021 --check` passed over the touched runtime, worker, transport, and test files. `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-worker-m1-0614 --message-format short --color never` passed with existing `zircon_runtime` warnings only, while restoring `zircon_plugins/Cargo.lock` afterward. A direct warmed lib-test run of `D:\cargo-targets\zircon-net-worker-m1-0614\debug\deps\zircon_plugin_net_runtime-e4e45688f0188aad.exe --test-threads=1 --nocapture` passed all 18 runtime tests. Direct `--locked` validation remains blocked before compile because Cargo wants to rewrite `zircon_plugins/Cargo.lock` package ordering; this slice kept the lockfile unchanged.

Focused validation after adding transport-qualified connection lifecycle events ran on 2026-06-07. `rustfmt --edition 2021 --check` passed over the touched Net framework, base runtime, RPC feature, and WebSocket feature Rust files. Path-scoped `git diff --check`, conflict-marker scans, and trailing-whitespace scans passed for the touched Rust/docs/session paths. `cargo check --manifest-path zircon_plugins/Cargo.toml` passed for `zircon_plugin_net_runtime --tests`, `zircon_plugin_net_rpc_runtime --tests`, and `zircon_plugin_net_websocket_runtime --tests` with the existing `zircon_runtime` warning set. Focused tests passed for `net_runtime_manager_accepts_tcp_client_and_echoes_payloads`, `net_runtime_queues_websocket_frames_with_budget`, and `rpc_feature_manager_closes_sessions_from_transport_events`. The first `cargo check -p zircon_runtime --tests --no-default-features --features core-min` attempt failed during a concurrent runtime-assembly file-state transition with `RuntimeExtensionRegistry` visible in the stale `registration_inputs.rs` source shape; the same command reran after the file state settled and passed with existing warnings only.

Focused validation after adding explicit Net runtime lifecycle events ran on 2026-06-07. The change is scoped to `NetEvent` DTO variants, base runtime close/unregister paths, framework/runtime tests, this document, the framework net document, and the active session note. `rustfmt --edition 2021 --check` passed over the touched Net Rust files; conflict-marker and trailing-whitespace scans over touched Net/doc/session paths returned empty; and path-scoped `git diff --check` passed with expected LF-to-CRLF warnings only. `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-net-lifecycle-events-0607 --message-format short --color never` passed with existing `zircon_runtime` warnings only. Focused runtime tests passed for `default_net_manager_sends_udp_packet_to_bound_socket`, `net_runtime_manager_closes_listeners_across_transports`, and `net_runtime_dispatches_registered_http_route`.

The framework-level tests in `zircon_runtime/src/core/framework/net/tests.rs` now cover DTO defaults and protocol layering without network IO, including serde round-trips for the new lifecycle event variants. Direct `cargo test -p zircon_runtime --lib http_and_websocket_descriptors_keep_protocol_state_data_only` attempts with default features and with `--no-default-features --features core-min` both timed out while linking the `zircon_runtime` test binary, so no framework test execution pass is claimed. `cargo check -p zircon_runtime --tests --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-net-lifecycle-events-0607-runtime-core-min-check --message-format short --color never` passed, covering the new framework test code at type-check level with existing `zircon_runtime` warnings only.

Focused validation after splitting `src/service_types.rs` into protocol and lifecycle child modules and replacing the monolithic base runtime `tests.rs` with `tests/{mod,feature_registration,manifest,udp,tcp,diagnostics,rpc_descriptor,http_routes,websocket,support}.rs` ran on 2026-06-04. `rustfmt --edition 2021 --check` passed over the facade, service child files, and test tree. `git diff --check` passed for the split files, this doc, and the active session note with only the expected LF-to-CRLF warning on the tracked facade file, and explicit trailing-whitespace and conflict-marker scans returned empty. A scoped `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-net-service-split-0604 --message-format short --color never` attempt timed out after two minutes before returning Rust diagnostics and left no cargo/rustc process behind; rerun that check before treating the split as compile-accepted. Later process polls still showed other-session workspace and Hub Cargo/rustc lanes active, so this slice stayed on low-interference validation instead of starting another compile.

Focused validation after replacing the flat base-runtime WebSocket service file with folder-backed `service_types/websocket/{backend,connect,listen,loopback,frames,close}.rs` ran on 2026-06-04. `rustfmt --edition 2021 --check` passed over the WebSocket facade and every child file. `git diff --check` passed for the WebSocket service files, this doc, and the active session note with only expected LF-to-CRLF warnings on tracked files, and explicit trailing-whitespace and conflict-marker scans returned empty. A low-concurrency `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-net-runtime-websocket-service-split-0604 --message-format short --color never` attempt timed out after three minutes before returning Rust diagnostics. Process audit immediately afterward showed no retained Cargo/rustc process for that target directory, while other-session workspace and first-party catalog Cargo lanes were active. Focused Cargo validation remains pending; rerun that check, followed by `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime websocket --locked --jobs 1 --target-dir E:\cargo-targets\zircon-net-runtime-websocket-service-split-0604 --message-format short --color never` if the check passes.

Focused validation after splitting `features/rpc/runtime/src/manager.rs` into `manager/{state,session,handshake,registry,dispatch,quota}.rs` and replacing the monolithic RPC `tests.rs` with `tests/{mod,feature_registration,session,dispatch,handlers,queue,support}.rs` ran on 2026-06-04. `rustfmt --edition 2021 --check` passed over the facade, every manager child file, and every test-tree child file. `git diff --check` passed for the RPC manager/test split files, this doc, and the active session note with only the expected LF-to-CRLF warning on the tracked facade file, and the conflict-marker scan returned empty. `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_rpc_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-net-rpc-manager-split-0604 --message-format short --color never` passed on 2026-06-04 for production code; it reported the existing `zircon_runtime` warning set and no RPC compile diagnostics. The relocated `#[cfg(test)]` modules still need focused `cargo test` or `cargo check --tests` validation when Cargo lanes are quiet.

Focused validation after splitting `features/content_download/runtime/src/manager.rs` into `manager/{state,manifest,attempts,resume,http_fetch,progress,hash}.rs` and replacing the monolithic content-download `tests.rs` with `tests/{mod,feature_registration,manifest,attempts,progress,resume,http_fetch,support}.rs` ran on 2026-06-04. `rustfmt --edition 2021 --check` passed over the facade, manager child files, and test tree. `git diff --check` passed for tracked content-download changes with only the expected LF-to-CRLF warning on the tracked manager facade, and an explicit trailing-whitespace scan over the new manager/test files, this doc, and the active session note returned empty. The conflict-marker scan returned empty. Focused Cargo validation is pending while other editor/Hub/runtime Cargo lanes are active.

Focused validation after splitting `features/websocket/runtime/src/backend.rs` into `backend/{client,connection,frame,handshake,listener,reader,security,stream}.rs` and replacing the monolithic WebSocket `tests.rs` with `tests/{mod,backend,feature_registration,handshake,security,support}.rs` ran on 2026-06-04. `rustfmt --edition 2021 --check` passed over the facade, backend child files, and test tree. `git diff --check` passed for tracked WebSocket changes with only the expected LF-to-CRLF warning on the tracked backend facade, and explicit trailing-whitespace and conflict-marker scans over the backend/test files, this doc, and the active session note returned empty. Focused Cargo validation is pending while other editor/Hub/runtime Cargo lanes are active.

Focused validation after splitting `features/reliable_udp/runtime/src/manager.rs` into `manager/{state,assembly,send,receive,delivery,recovery,resend,stats}.rs` and replacing the monolithic Reliable UDP `tests.rs` with `tests/{mod,feature_registration,send,delivery,receive,recovery,resend}.rs` ran on 2026-06-04. `rustfmt --edition 2021 --check` passed over the facade, manager child files, and test tree. `git diff --check` passed for tracked Reliable UDP changes with only the expected LF-to-CRLF warning on the tracked manager facade, and explicit trailing-whitespace and conflict-marker scans over the manager/test files, this doc, and the active session note returned empty. Focused Cargo validation is pending while other editor/Hub/runtime Cargo lanes are active.

Focused validation after splitting `features/http/runtime/src/backend.rs` into `backend/{client,method,security,server}.rs` and replacing the monolithic HTTP `tests.rs` with `tests/{mod,backend,feature_registration,routes,security}.rs` ran on 2026-06-04. `rustfmt --edition 2021 --check` passed over the facade, backend child files, and test tree. `git diff --check` passed for tracked HTTP changes with only the expected LF-to-CRLF warning on the tracked backend facade, and explicit trailing-whitespace and conflict-marker scans over the backend/test files, this doc, and the active session note returned empty. Focused Cargo validation is pending while other editor/Hub/runtime Cargo lanes are active.

Focused validation after splitting `features/replication/runtime/src/manager.rs` into `manager/{state,registry,interest,snapshot,schedule,budget,lifecycle}.rs` and replacing the monolithic replication `tests.rs` with `tests/{mod,feature_registration,delta_interest,lifecycle,schedule,budget}.rs` ran on 2026-06-04. `rustfmt --edition 2021 --check` passed over the facade, manager child files, and test tree. `git diff --check` passed for tracked Replication changes with only the expected LF-to-CRLF warning on the tracked manager facade, and explicit trailing-whitespace and conflict-marker scans over the manager/test facade files, child files, this doc, and the active session note returned empty. Focused Cargo validation is pending while other editor/Hub/runtime Cargo lanes are active.

The 2026-07-04 full plugin workspace execution closeout is tracked as `plugins_13_m5_t1_plugin_workspace_locked_all_targets_test_execution_passed`. The Net-specific fix stayed in `zircon_plugins/net/runtime/src/tests/worker.rs`: the TCP/UDP source-structure guard now resolves source files from `env!("CARGO_MANIFEST_DIR")` instead of relying on the test process CWD, so the guard still rejects direct Tokio `.block_on(` in service facades while running under the plugin workspace test harness. Focused validation passed `cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_net_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plugin-net-runtime-worker-rerun-0703 --message-format short --color never -- --nocapture --test-threads=1` with 21/21 before the final workspace rerun; `E:\cargo-targets\zircon-plugin-workspace-test-0703-codex-rerun13-final.status.json` then recorded full workspace `ExitCode=0`.
