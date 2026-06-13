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
  - zircon_plugins/net/features/http/runtime/Cargo.toml
  - zircon_plugins/net/features/http/runtime/src/lib.rs
  - zircon_plugins/net/features/http/runtime/src/feature.rs
  - zircon_plugins/net/features/http/runtime/src/backend.rs
  - zircon_plugins/net/features/http/runtime/src/backend/client.rs
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
  - zircon_plugins/net/features/rpc/runtime/src/manager/dispatch.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/handshake.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/quota.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/registry.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/session.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/state.rs
  - zircon_plugins/net/features/rpc/runtime/src/tests/mod.rs
  - zircon_plugins/net/features/rpc/runtime/src/tests/dispatch.rs
  - zircon_plugins/net/features/rpc/runtime/src/tests/feature_registration.rs
  - zircon_plugins/net/features/rpc/runtime/src/tests/handlers.rs
  - zircon_plugins/net/features/rpc/runtime/src/tests/queue.rs
  - zircon_plugins/net/features/rpc/runtime/src/tests/session.rs
  - zircon_plugins/net/features/rpc/runtime/src/tests/support.rs
  - zircon_plugins/net/features/replication/runtime/src/lib.rs
  - zircon_plugins/net/features/replication/runtime/src/feature.rs
  - zircon_plugins/net/features/replication/runtime/src/manager.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/budget.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/interest.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/lifecycle.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/registry.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/schedule.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/snapshot.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/state.rs
  - zircon_plugins/net/features/replication/runtime/src/tests/mod.rs
  - zircon_plugins/net/features/replication/runtime/src/tests/budget.rs
  - zircon_plugins/net/features/replication/runtime/src/tests/delta_interest.rs
  - zircon_plugins/net/features/replication/runtime/src/tests/feature_registration.rs
  - zircon_plugins/net/features/replication/runtime/src/tests/lifecycle.rs
  - zircon_plugins/net/features/replication/runtime/src/tests/schedule.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/lib.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/feature.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/manager.rs
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
  - zircon_plugins/net/features/reliable_udp/runtime/src/tests/receive.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/tests/recovery.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/tests/resend.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/tests/send.rs
  - zircon_plugins/net/features/content_download/runtime/src/lib.rs
  - zircon_plugins/net/features/content_download/runtime/src/feature.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager/attempts.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager/hash.rs
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
  - zircon_runtime/src/core/framework/net/http.rs
  - zircon_runtime/src/core/framework/net/manager.rs
  - zircon_runtime/src/core/framework/net/tests.rs
  - zircon_runtime/src/core/framework/net/transport.rs
implementation_files:
  - zircon_plugins/net/plugin.toml
  - zircon_runtime/src/core/framework/net/http.rs
  - zircon_runtime/src/core/framework/net/tests.rs
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
  - zircon_plugins/net/features/http/runtime/Cargo.toml
  - zircon_plugins/net/features/http/runtime/src/lib.rs
  - zircon_plugins/net/features/http/runtime/src/feature.rs
  - zircon_plugins/net/features/http/runtime/src/backend.rs
  - zircon_plugins/net/features/http/runtime/src/backend/client.rs
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
  - zircon_plugins/net/features/replication/runtime/src/manager/budget.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/interest.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/lifecycle.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/registry.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/schedule.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/snapshot.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/state.rs
  - zircon_plugins/net/features/replication/runtime/src/tests/mod.rs
  - zircon_plugins/net/features/replication/runtime/src/tests/budget.rs
  - zircon_plugins/net/features/replication/runtime/src/tests/delta_interest.rs
  - zircon_plugins/net/features/replication/runtime/src/tests/feature_registration.rs
  - zircon_plugins/net/features/replication/runtime/src/tests/lifecycle.rs
  - zircon_plugins/net/features/replication/runtime/src/tests/schedule.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/lib.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/feature.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/manager.rs
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
  - zircon_plugins/net/features/reliable_udp/runtime/src/tests/receive.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/tests/recovery.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/tests/resend.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/tests/send.rs
  - zircon_plugins/net/features/rpc/runtime/src/lib.rs
  - zircon_plugins/net/features/rpc/runtime/src/feature.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/dispatch.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/handshake.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/quota.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/registry.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/session.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/state.rs
  - zircon_plugins/net/features/rpc/runtime/src/tests/mod.rs
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
  - zircon_plugins/net/features/content_download/runtime/src/manager/hash.rs
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
  - net_runtime_diagnostics_count_listeners_by_transport
  - net_runtime_manager_closes_listeners_across_transports
  - net_runtime_dispatches_registered_http_route
  - net_runtime_queues_websocket_frames_with_budget
  - websocket_connection_send_path_is_queue_driven
  - ws_frame_order_preserved
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
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-tls-m2-0614 --message-format short --color never (passed 2026-06-14 with Cargo.lock restored afterward)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_http_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-tls-m2-0614 --message-format short --color never (passed 2026-06-14 with Cargo.lock restored afterward)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_http_runtime --offline --jobs 1 --no-run --target-dir D:\cargo-targets\zircon-net-tls-m2-0614 --message-format short --color never (passed 2026-06-14 with Cargo.lock restored afterward)
  - D:\cargo-targets\zircon-net-tls-m2-0614\debug\deps\zircon_plugin_net_http_runtime-4b0511cca9036008.exe --test-threads=1 --nocapture (11 passed 2026-06-14)
  - rustfmt --edition 2021 --check over zircon_plugins/net/features/websocket/runtime/src/backend/{client,connection,listener}.rs and zircon_plugins/net/features/websocket/runtime/src/tests/{backend,handshake,support}.rs (passed 2026-06-14)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_websocket_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never (blocked 2026-06-14 by unrelated active render-session GpuSceneEntry initializer drift before WebSocket diagnostics)
doc_type: module-detail
---

# Net Runtime Plugin

## Purpose

`zircon_plugins/net/runtime` owns the executable networking service for the first-party Net plugin. It implements the neutral `zircon_runtime::core::framework::net::NetManager` contract with a Tokio-backed base runtime, plugin-owned state, in-memory HTTP route dispatch, loopback WebSocket support, diagnostics, and package metadata for optional network features.

The runtime plugin is intentionally not the whole networking stack. HTTP sockets, WebSocket handshakes, RPC, replication, reliable UDP, and content-download behavior are layered as optional features and feature crates. The base plugin contributes the shared manager and the project/export catalog rows that make those features selectable.

## Runtime Boundary

- `plugin.toml` and `runtime_plugin_descriptor()` classify Net as a `runtime` plugin with `beta` maturity and `runtime.plugin.net` capability.
- `module.rs` contributes the `net.runtime` module, `NetDriver`, and `DefaultNetManager` service through the runtime module system.
- `package.rs` contributes options, optional feature rows, dependencies, and event catalog metadata. The static plugin manifest and runtime manifest must stay synchronized.
- `runtime_state.rs` stores plugin-owned Tokio runtime handles, the dedicated `NetWorker`, metadata tables for UDP sockets and TCP listeners/connections, HTTP routes/listeners, WebSocket listeners/connections, and queued events.
- `worker/{mod,egress,ingress,shutdown}.rs` owns the dedicated network worker thread. It keeps Tokio socket/listener/stream handles out of the caller thread and accepts bounded egress commands for UDP/TCP bind, listen, connect, send, poll, close, and shutdown.
- `runtime_system.rs` registers `net.transport`, `net.poll_ingress`, `net.flush_egress`, and the typed `NetEvent` channel. `net.poll_ingress` runs in `SystemStage::First` and drains manager events into the world event queue; `net.flush_egress` is a `SystemStage::Last` scheduling anchor reserved for the later frame-command collection path.
- `transport/{mod,reconnect,state_machine,tls}.rs` owns shared transport control helpers. `ReconnectPolicy` provides deterministic exponential retry delays with max-delay capping and optional deterministic jitter, while `TransportStateMachine` centralizes TCP connection state transitions and emits typed `ConnectionStateChanged` events. `transport/tls.rs` centralizes rustls root-store construction, certificate SHA-256 pin calculation/matching, and `TlsServerIdentity` server-config injection for optional transport features.
- `service_types.rs` is the structural manager facade. It owns `DefaultNetManager`, `NetDriver`, id allocation, backend injection, and the `NetManager` trait implementation that delegates to focused service modules.
- `service_types/udp.rs`, `tcp.rs`, and `http_routes.rs` own protocol-specific base runtime operations. UDP/TCP operations now validate local metadata on the manager facade and then route actual socket IO through `NetWorker` commands instead of blocking the caller thread on Tokio. `service_types/websocket.rs` is a structural WebSocket service root whose child modules separate optional-backend lookup, real connect calls, real listener accept loops, deterministic loopback pairs, frame send/poll behavior, and close handling. `service_types/listeners.rs` and `connections.rs` own cross-protocol listener/connection lifecycle helpers. `service_types/diagnostics.rs` owns copied diagnostics, backend-name projection, worker ingress polling, and bounded event draining.
- `tests/mod.rs` is now a structural base runtime test entry. Its child modules separate plugin registration, package manifest rows, UDP loopback, TCP loopback, diagnostics/listener lifecycle, RPC descriptor DTO checks, local HTTP route dispatch, WebSocket loopback behavior, worker shutdown/source-structure guards, and shared polling helpers.
- `http.rs` and `websocket.rs` define plugin-local backend traits/adapters used by optional feature crates.
- Each optional feature crate keeps `src/lib.rs` as a public re-export surface and `src/feature.rs` as the runtime feature-registration surface. Those files own feature IDs, capability names, module descriptors, manager factories, dependencies, and package feature manifests. Backend and manager behavior stays below them instead of accumulating in the feature descriptor.
- `features/http/runtime/src/backend.rs` is now a structural facade for the Hyper/Reqwest HTTP backend. Its child modules separate outbound request/retry execution, HTTP security policy validation, method conversion, and Hyper listener route dispatch/body-limit handling. The outbound path uses hyper for `http://` requests; HTTPS uses reqwest/rustls with optional custom roots and certificate pin verification through the shared TLS helper before response bodies are read. The facade still exposes `HTTP_ROUTE_REQUEST_BODY_LIMIT_BYTES` for existing public-surface tests. The matching `features/http/runtime/src/tests/` tree separates backend injection, feature registration, socket route behavior, and security-policy assertions.
- `features/websocket/runtime/src/backend.rs` is now a structural facade for the Tungstenite WebSocket backend. Its child modules separate outbound client setup, listener accept/upgrade, server handshake policy, security policy validation, connection state and sending, reader task eventing, shared stream aliases, and frame conversion. `backend/connection.rs` owns the feature-local WebSocket egress worker: client/server constructors move each Tungstenite sink into a Tokio writer task behind a bounded `mpsc` queue, while `send_websocket_frame` only enqueues frames and marks close requests as `Closing`. The matching `features/websocket/runtime/src/tests/` tree separates backend injection, feature registration, real handshake/policy behavior, frame-order guarantees, security-policy assertions, and shared polling helpers.
- `features/replication/runtime/src/manager.rs` is now a structural facade for the replication manager. Its child modules separate component descriptor registration, session interest filtering, snapshot/delta publication, scheduling and budget math, despawn lifecycle cleanup, and shared manager state. The matching `features/replication/runtime/src/tests/` tree mirrors those public manager behavior boundaries with focused registration, delta/interest, lifecycle, schedule, and budget test files.
- `features/reliable_udp/runtime/src/manager.rs` is now a structural facade for the reliable datagram manager. Its child modules separate shared state, inbound fragment assembly, send/fragment queueing, receive/reassembly, simulated delivery, recovery state, resend/ack bookkeeping, and stats access. The matching `features/reliable_udp/runtime/src/tests/` tree mirrors registration, send/ack, delivery simulation, receive assembly, recovery, and resend behavior boundaries.
- `features/rpc/runtime/src/manager.rs` is now a structural facade for the optional RPC feature manager. Its child modules separate long-lived state construction, session lifecycle, control-message handshake, descriptor/handler registry, dispatch/queue/pending-request flow, and per-session quota accounting. The matching `features/rpc/runtime/src/tests/` tree mirrors those boundaries with focused registration, session, dispatch, handler, queue, and shared-support test files.
- `features/content_download/runtime/src/manager.rs` is now a structural facade for content-download orchestration. Its child modules separate manifest validation, mirror/attempt selection, partial-range resume storage, HTTP request/response validation, progress/cache/cancel mutation, shared state, and chunk hashing. The matching `features/content_download/runtime/src/tests/` tree mirrors those behavior boundaries with focused feature-registration, manifest, attempt, progress, resume, HTTP-fetch, and shared-support test files.

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

The optional WebSocket backend follows the same internal boundary rule. Godot's WebSocket module separates peer, multiplayer, debugger, packet buffering, and platform-specific WebSocket implementations; Bevy Remote keeps transport startup outside the method registry. Zircon keeps `TungsteniteWebSocketBackend` as the public optional-feature backend while moving client connection setup, listener upgrade, handshake policy, security checks, connection state, reader tasks, and frame conversion into child modules. The test tree follows backend-injection, registration, handshake/policy, security, and shared polling-helper boundaries so future TLS policy, compression, close-code handling, RPC subprotocols, and browser/platform variants do not accumulate in one backend test file.

The optional Replication manager is split around the same multiplayer data-flow boundaries. Godot separates scene replication config, replication interface, synchronizers, spawners, and editor tooling; Unreal/Iris separates replication state, filtering, scheduling, and replication system responsibilities. Zircon keeps `NetReplicationRuntimeManager` as the public optional-feature facade while separating descriptor registration, interest filtering, snapshot/delta publication, budgeted scheduling, despawn cleanup, and shared state. The test tree follows the same split so authority policy, channel ownership, baseline compression, and interest-grid expansion can add coverage without reviving a monolithic module-local test file.

The optional Reliable UDP manager is split around the same runtime data-flow boundaries. Godot's ENet and packet-peer surfaces keep transport, packet buffering, and higher-level multiplayer responsibilities distinct, while Bevy-style networking crates usually separate channels, send queues, receive assembly, and reliability bookkeeping. Zircon keeps `NetReliableUdpRuntimeManager` as the public optional-feature facade while separating fragment assembly, outbound queueing, acknowledgement, resend timing, simulated delivery, recovery status, and stats access. The test tree follows send/ack, simulated delivery, receive assembly, recovery, resend, and registration boundaries so future channel QoS, congestion policy, packet windows, and transport adapters do not grow one mixed manager test file.

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

Optional HTTP feature tests now cover plan-named M2-T1/M2-T2 behavior: `http_round_trip_against_local_hyper_server` proves the feature manager can serve and fetch through the local Hyper listener with the hyper client path, `range_request_returns_partial` proves `NetHttpRequestDescriptor::with_byte_range(...)` reaches the route handler and preserves `Content-Range` response metadata, and `self_signed_cert_rejected_then_pinned_accepted` proves an unpinned self-signed HTTPS server is rejected before a matching host certificate pin allows the response. Optional content-download tests continue to cover resumed HTTP range fetches with existing prefixes through that shared request contract.

Optional WebSocket feature tests now cover the M2-T3 send-worker slice. `ws_frame_order_preserved` sends multiple text/binary frames in both directions over a real Tungstenite handshake and polls until the peer observes the exact order. `websocket_connection_send_path_is_queue_driven` is a source-structure guard for the feature backend: frame sends must use the bounded queue and must not reintroduce caller-thread `block_on`.

Focused validation after landing the M2-T3 WebSocket send-worker slice ran on 2026-06-14. `rustfmt --edition 2021 --check` passed over the touched WebSocket backend and test files; conflict-marker, trailing-whitespace, and send-path fallback scans were clean. A lockfile-backup protected `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_websocket_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never` reached `zircon_runtime` dependency compilation and stopped on the active render-session drift `zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs:212` missing `GpuSceneEntry.has_rolled_previous_transform`; no WebSocket compile diagnostics were returned, root/plugin lockfiles were restored, and no WebSocket Cargo pass is claimed.

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
