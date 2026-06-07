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
  - zircon_plugins/net/runtime/src/tests/mod.rs
  - zircon_plugins/net/runtime/src/tests/diagnostics.rs
  - zircon_plugins/net/runtime/src/tests/feature_registration.rs
  - zircon_plugins/net/runtime/src/tests/http_routes.rs
  - zircon_plugins/net/runtime/src/tests/manifest.rs
  - zircon_plugins/net/runtime/src/tests/rpc_descriptor.rs
  - zircon_plugins/net/runtime/src/tests/support.rs
  - zircon_plugins/net/runtime/src/tests/tcp.rs
  - zircon_plugins/net/runtime/src/tests/udp.rs
  - zircon_plugins/net/runtime/src/tests/websocket.rs
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
  - zircon_runtime/src/core/framework/net/manager.rs
implementation_files:
  - zircon_plugins/net/plugin.toml
  - zircon_plugins/net/runtime/Cargo.toml
  - zircon_plugins/net/runtime/src/lib.rs
  - zircon_plugins/net/runtime/src/config.rs
  - zircon_plugins/net/runtime/src/http.rs
  - zircon_plugins/net/runtime/src/module.rs
  - zircon_plugins/net/runtime/src/package.rs
  - zircon_plugins/net/runtime/src/runtime_state.rs
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
  - zircon_plugins/net/runtime/src/tests/mod.rs
  - zircon_plugins/net/runtime/src/tests/diagnostics.rs
  - zircon_plugins/net/runtime/src/tests/feature_registration.rs
  - zircon_plugins/net/runtime/src/tests/http_routes.rs
  - zircon_plugins/net/runtime/src/tests/manifest.rs
  - zircon_plugins/net/runtime/src/tests/rpc_descriptor.rs
  - zircon_plugins/net/runtime/src/tests/support.rs
  - zircon_plugins/net/runtime/src/tests/tcp.rs
  - zircon_plugins/net/runtime/src/tests/udp.rs
  - zircon_plugins/net/runtime/src/tests/websocket.rs
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
tests:
  - zircon_plugins/net/runtime/src/tests/mod.rs
  - net_plugin_registration_contributes_runtime_module
  - net_plugin_manifest_advertises_layered_optional_features
  - default_net_manager_sends_udp_packet_to_bound_socket
  - net_runtime_manager_accepts_tcp_client_and_echoes_payloads
  - net_runtime_manager_reports_mode_diagnostics_and_events
  - net_runtime_diagnostics_count_listeners_by_transport
  - net_runtime_manager_closes_listeners_across_transports
  - net_runtime_dispatches_registered_http_route
  - net_runtime_queues_websocket_frames_with_budget
  - rpc_feature_manager_closes_sessions_from_transport_events
  - zircon_runtime/src/core/framework/net/tests.rs
  - rustfmt --edition 2021 --check over touched Net event/runtime/test files (passed 2026-06-07)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-net-lifecycle-events-0607 --message-format short --color never (passed 2026-06-07)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime default_net_manager_sends_udp_packet_to_bound_socket --locked --jobs 1 --target-dir E:\cargo-targets\zircon-net-lifecycle-events-0607 --message-format short --color never -- --test-threads=1 --nocapture (passed 2026-06-07)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime net_runtime_manager_closes_listeners_across_transports --locked --jobs 1 --target-dir E:\cargo-targets\zircon-net-lifecycle-events-0607 --message-format short --color never -- --test-threads=1 --nocapture (passed 2026-06-07)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime net_runtime_dispatches_registered_http_route --locked --jobs 1 --target-dir E:\cargo-targets\zircon-net-lifecycle-events-0607 --message-format short --color never -- --test-threads=1 --nocapture (passed 2026-06-07)
  - cargo check -p zircon_runtime --tests --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-net-lifecycle-events-0607-runtime-core-min-check --message-format short --color never (passed 2026-06-07)
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
- `runtime_state.rs` stores plugin-owned Tokio runtime handles, UDP sockets, TCP listeners/connections, HTTP routes/listeners, WebSocket listeners/connections, and queued events.
- `service_types.rs` is the structural manager facade. It owns `DefaultNetManager`, `NetDriver`, id allocation, backend injection, and the `NetManager` trait implementation that delegates to focused service modules.
- `service_types/udp.rs`, `tcp.rs`, and `http_routes.rs` own protocol-specific base runtime operations. `service_types/websocket.rs` is now a structural WebSocket service root whose child modules separate optional-backend lookup, real connect calls, real listener accept loops, deterministic loopback pairs, frame send/poll behavior, and close handling. `service_types/listeners.rs` and `connections.rs` own cross-protocol listener/connection lifecycle helpers. `service_types/diagnostics.rs` owns copied diagnostics, backend-name projection, and bounded event draining.
- `tests/mod.rs` is now a structural base runtime test entry. Its child modules separate plugin registration, package manifest rows, UDP loopback, TCP loopback, diagnostics/listener lifecycle, RPC descriptor DTO checks, local HTTP route dispatch, WebSocket loopback behavior, and shared polling helpers.
- `http.rs` and `websocket.rs` define plugin-local backend traits/adapters used by optional feature crates.
- Each optional feature crate keeps `src/lib.rs` as a public re-export surface and `src/feature.rs` as the runtime feature-registration surface. Those files own feature IDs, capability names, module descriptors, manager factories, dependencies, and package feature manifests. Backend and manager behavior stays below them instead of accumulating in the feature descriptor.
- `features/http/runtime/src/backend.rs` is now a structural facade for the Hyper/Reqwest HTTP backend. Its child modules separate outbound request/retry execution, HTTP security policy validation, method conversion, and Hyper listener route dispatch/body-limit handling. The facade still exposes `HTTP_ROUTE_REQUEST_BODY_LIMIT_BYTES` for existing public-surface tests. The matching `features/http/runtime/src/tests/` tree separates backend injection, feature registration, socket route behavior, and security-policy assertions.
- `features/websocket/runtime/src/backend.rs` is now a structural facade for the Tungstenite WebSocket backend. Its child modules separate outbound client setup, listener accept/upgrade, server handshake policy, security policy validation, connection state and sending, reader task eventing, shared stream aliases, and frame conversion. The matching `features/websocket/runtime/src/tests/` tree separates backend injection, feature registration, real handshake/policy behavior, security-policy assertions, and shared polling helpers.
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

The manager service boundary is intentionally folder-backed. Godot keeps UDP packet peers, TCP servers/streams, HTTP clients, and WebSocket peers as separate runtime services; Bevy Remote separates protocol registration from HTTP transport startup. Zircon translates that into one public `NetManager` facade with protocol-specific implementation modules below it, so future RPC, replication, reliable UDP, and content-download work can extend the network stack without appending more behavior to one mixed service file.

`NetDiagnostics` is copied from the manager's actual handle tables and now reports listener ownership by transport: `open_tcp_listeners`, `open_http_listeners`, and `open_websocket_listeners`. This keeps editor network inspectors and export/profile diagnostics from treating "listener count" as TCP-only while optional HTTP and WebSocket backends are installed. HTTP and WebSocket listener counts still stay zero when those feature backends are absent, matching the profile-gated transport model.

Lifecycle events are intentionally emitted from the concrete close/unregister paths instead of inferred from diagnostics snapshots. `close_socket`, `close_listener`, `close_connection`, and `unregister_http_route` remove the runtime-owned handle first, then queue `NetEvent` records such as `UdpSocketClosed`, `ListenerClosed`, `ConnectionClosed`, or `HttpRouteUnregistered`. Connection accept and close events carry their `NetTransportKind`, so network inspectors, RPC session cleanup, and export/profile diagnostics can classify TCP versus WebSocket lifecycle changes without looking up a connection that may already have been removed from the manager table.

The base WebSocket service follows the same rule below the protocol module. Godot keeps WebSocket peer state, packet buffering, multiplayer integration, debugger peers, and platform implementations separate; Bevy Remote keeps remote protocol registration independent from transport startup; Fyrox keeps listener/stream wrappers narrow. Zircon keeps `DefaultNetManager` as the only public manager while splitting base WebSocket behavior into backend resolution, connect/listen/accept, loopback, frame queueing, and close modules. This keeps deterministic test loopbacks, optional backend calls, listener polling, frame budgets, and state-close semantics independent before future browser transports, compression, subprotocol dispatch, RPC bridging, or editor network inspectors grow this service area.

The optional HTTP backend follows the same boundary rule. Godot keeps HTTP client behavior separate from request-node dispatch, and Bevy Remote keeps HTTP transport startup distinct from protocol method registration. Zircon keeps `HyperReqwestHttpBackend` as the public optional-feature backend while moving Reqwest send/retry behavior, security policy checks, Hyper listener dispatch, body-limit handling, and method conversion into child modules. The test tree follows route, security, backend-injection, and registration boundaries so future proxy policy, streaming bodies, authentication, and route middleware can add coverage without growing one backend test file.

The optional WebSocket backend follows the same internal boundary rule. Godot's WebSocket module separates peer, multiplayer, debugger, packet buffering, and platform-specific WebSocket implementations; Bevy Remote keeps transport startup outside the method registry. Zircon keeps `TungsteniteWebSocketBackend` as the public optional-feature backend while moving client connection setup, listener upgrade, handshake policy, security checks, connection state, reader tasks, and frame conversion into child modules. The test tree follows backend-injection, registration, handshake/policy, security, and shared polling-helper boundaries so future TLS policy, compression, close-code handling, RPC subprotocols, and browser/platform variants do not accumulate in one backend test file.

The optional Replication manager is split around the same multiplayer data-flow boundaries. Godot separates scene replication config, replication interface, synchronizers, spawners, and editor tooling; Unreal/Iris separates replication state, filtering, scheduling, and replication system responsibilities. Zircon keeps `NetReplicationRuntimeManager` as the public optional-feature facade while separating descriptor registration, interest filtering, snapshot/delta publication, budgeted scheduling, despawn cleanup, and shared state. The test tree follows the same split so authority policy, channel ownership, baseline compression, and interest-grid expansion can add coverage without reviving a monolithic module-local test file.

The optional Reliable UDP manager is split around the same runtime data-flow boundaries. Godot's ENet and packet-peer surfaces keep transport, packet buffering, and higher-level multiplayer responsibilities distinct, while Bevy-style networking crates usually separate channels, send queues, receive assembly, and reliability bookkeeping. Zircon keeps `NetReliableUdpRuntimeManager` as the public optional-feature facade while separating fragment assembly, outbound queueing, acknowledgement, resend timing, simulated delivery, recovery status, and stats access. The test tree follows send/ack, simulated delivery, receive assembly, recovery, resend, and registration boundaries so future channel QoS, congestion policy, packet windows, and transport adapters do not grow one mixed manager test file.

The optional RPC manager follows the same boundary rule. Bevy Remote separates method registry, protocol message processing, and transport startup; Godot keeps JSON-RPC, scene RPC, and multiplayer synchronization as distinct runtime surfaces. Zircon keeps `NetRpcRuntimeManager` as the public optional-feature facade while moving handshake/session logic, registry, dispatch queue, pending-request correlation, and quota tracking into child modules. This keeps future reflection-backed RPC discovery, authority policies, and transport adapters from growing one mixed manager file.

The optional content-download manager uses the same facade-plus-child-modules rule. Godot separates low-level HTTP clients from request nodes, Bevy separates asset IO from transport concerns, and Unreal's platform downloader separates progress/service/manifest responsibilities. Zircon keeps `NetContentDownloadRuntimeManager` as the feature-facing manager while separating manifest validation, attempts/mirrors, HTTP fetch validation, resume prefixes, progress state, and hashing so future CDN policy, cache storage, and asset-pipeline integration do not grow one mixed file. The test tree follows the same split so adding cache policy, CDN selection, or asset-pipeline integration tests does not revive a monolithic module-local test file.

## Reference Alignment

Bevy Remote is the strongest profile/reference precedent: `RemotePlugin` registers methods, but an additional `RemoteHttpPlugin` starts the HTTP transport. Zircon follows the same split at a broader networking scale. The base Net plugin contributes shared protocol/service infrastructure; transports and advanced gameplay systems stay optional.

Godot provides the broad engine precedent for multiple network surfaces: HTTP client, UDP packet peer, TCP server, ENet/multiplayer, and WebSocket peer are distinct runtime concepts. Zircon keeps those distinctions in package features and framework DTOs but exposes one manager facade for engine consumers.

Godot's low-level UDP peer and TCP server APIs expose explicit bind/listen, close/stop, local-port, and status queries, while `StreamPeerTCP` tracks connection status transitions. Zircon does not mirror those object APIs directly, but it preserves the same lifecycle boundary by emitting typed events when manager-owned sockets, listeners, routes, and connections start or close, and by projecting per-transport listener counts through `NetDiagnostics`.

Fyrox provides the Rust-native precedent for small listener/stream abstractions. Zircon keeps simple TCP/UDP behavior directly testable in the base runtime and avoids forcing all higher-level protocols into that low-level stream abstraction.

## Edge Cases

The base runtime should fail explicitly when an optional backend is absent. `listen_http`, real outbound HTTP requests with explicit ports, `listen_websocket`, and real WebSocket connect calls return `NetError::ProtocolUnavailable` until a backend is installed. This keeps missing capabilities visible to export/profile diagnostics instead of silently running partial behavior.

HTTP route handlers are deterministic local dispatch only. They are useful for tests, editor mock endpoints, and local tools, but they are not a replacement for the optional HTTP runtime backend. WebSocket loopback pairs are similarly deterministic test infrastructure, not an external network listener.

## Test Coverage

Runtime tests cover plugin module registration, package optional features, UDP loopback packets plus socket-close events, TCP loopback accept/send/poll plus transport-qualified accept/close events, diagnostics/events, per-transport listener counts through fake HTTP/WebSocket backends, listener close events, local HTTP route dispatch plus route-unregister events, dynamic HTTP route handlers, explicit HTTP backend absence, WebSocket loopback frame budgets, transport-qualified WebSocket close events, and explicit WebSocket backend absence. Optional RPC coverage keeps session cleanup transport-agnostic while consuming the richer close event, and optional WebSocket backend reader code emits WebSocket-qualified close events for real transport tasks. The base runtime test tree follows those same boundaries so adding protocol-specific assertions does not revive a monolithic `tests.rs`.

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
