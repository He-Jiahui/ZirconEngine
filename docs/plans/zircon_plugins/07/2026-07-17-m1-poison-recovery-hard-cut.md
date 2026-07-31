---
related_code:
  - zircon_plugins/net/runtime/src/poison_recovery.rs
  - zircon_plugins/net/runtime/src/runtime_state.rs
  - zircon_plugins/net/runtime/src/service_types.rs
  - zircon_plugins/net/runtime/src/service_types
  - zircon_plugins/net/runtime/src/worker/mod.rs
  - zircon_plugins/net/runtime/src/worker/net_worker.rs
  - zircon_plugins/net/runtime/src/worker/transport_runtime.rs
  - zircon_plugins/net/runtime/src/worker/transport_runtime/dispatch.rs
  - zircon_plugins/net/runtime/src/tests/poison_recovery.rs
tests:
  - poisoned_event_queue_recovers_for_public_manager_reads
  - poisoned_fallible_manager_state_returns_typed_error
  - poisoned_transport_tables_fail_before_send_or_poll_io
  - poisoned_worker_thread_fails_before_shutdown_side_effects
  - failed_worker_shutdown_remains_retryable_until_join_completes
  - dynamic_http_handler_can_reenter_manager_without_route_registry_deadlock
  - route_handler_destructor_can_reenter_manager_after_registry_release
  - http_listener_post_callback_poison_aborts_and_does_not_publish_or_register
  - websocket_callbacks_can_reenter_manager_without_registry_deadlock
  - websocket_connect_post_callback_poison_closes_without_orphan_event
  - websocket_listen_post_callback_poison_drops_without_orphan_event
  - websocket_accept_callback_failure_closes_every_staged_connection
  - worker_root_only_mounts_named_behavior_owners
  - shared_state_poison_error_keeps_typed_resource_identity
  - cargo +1.94.1 test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --locked --jobs 1 poison_recovery -- --nocapture --test-threads=1
  - cargo +1.94.1 test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --locked --jobs 1 -- --nocapture --test-threads=1
---

# Plugins07 M1：Net runtime poison-lock recovery hard cut

## 状态

`implementation_complete / broad_green / focused_review_commit_pending_fifo`

## 完成项目

- 新增 crate-local `poison_recovery` 单一策略 owner：`lock_recover` 只供 infallible API 恢复 poison payload 中的最后有效状态，`lock_or_error` 只供 fallible API 返回共享 typed failure；业务模块不复制 `PoisonError` 分支。
- 将 base Net runtime 的 52 个生产 `Mutex::lock().expect(...)` 全部硬切到统一 owner：16 个 infallible worker ingress/runtime event/backend-builder/diagnostics 路径恢复 guard；fallible worker shutdown、HTTP/WebSocket、route/listener/connection、TCP/UDP 路径统一返回 `NetError::SharedStatePoisoned { resource }`。
- `NetSharedState` 集中拥有十种稳定 resource identity；没有散落 poison 诊断 magic string、兼容 wrapper、旧路径 re-export、`allow(dead_code)` 或把失败伪装成成功的上层旁路。现有 IO/协议错误继续按原 typed contract 返回。
- 行为覆盖真实 poison 与重入边界：event queue 恢复；UDP/TCP send/poll typed preflight；worker thread poison 与 post-submit shutdown retry；HTTP handler/析构重入；HTTP listener post-callback poison 后 abort；WebSocket backend/listener/connection 重入；connect/listen post-callback poison 后 Close/Drop；第二次 accept 失败后整批 staged connection 回滚。framework test 另覆盖新 error variant serde round-trip。
- worker transport 路径在内部 registry guard 下执行不可重入 worker 请求；HTTP/WebSocket 可插拔边界只在 guard 内克隆 `Arc` 快照，释放 registry 后调用 handler/backend/listener/connection。创建路径先做 poison preflight，提交失败时关闭或丢弃新外部资源。
- shutdown 在命令提交后保存 pending reply；timeout 或注入的 post-submit 失败保持 `is_shutdown=false`，下一次调用复用同一 receiver 收 report 并 join。worker 已终止或 join panic 时也先收敛 terminal flag，不会返回伪造的空成功报告或永久重发到断开 channel。
- 将原 694 行 `worker/mod.rs` 硬切为 8 行挂载根，根只挂载 `egress`、`ingress`、`net_worker`、`shutdown`、`transport_runtime` 五个 named child；分发函数只对 `crate::worker` 可见，符合 `engine-code-structure-convention.md` 的 root/facade 与单一职责要求。

## 当前证据

- 生产 Rust 全树 regex 扫描：`lock/read/write().expect|unwrap` poison 路径 `0`。
- 52 个旧生产调用点精确分为 16 个 infallible 与 36 个 fallible；当前生产调用为 `lock_recover = 16`、`lock_or_error = 40`，新增的 typed preflight/commit 检查确保 send/poll 及可插拔创建路径在副作用前检测 poison；另外七个 `#[cfg(test)]` poison fixture 覆盖 events、worker、UDP、TCP、HTTP listeners、WebSocket listeners 与 WebSocket connections。
- 独立静态复评：Critical `0`、Important `0`、Minor `0`；包含 post-callback cleanup、批 accept rollback、析构重入、测试自环释放与 worker terminal ordering。
- canonical Rust 1.94.1 `rustfmt --check`：通过。
- scoped `git diff --check`：通过，仅仓库既有 LF/CRLF 提示。
- canonical Rust 1.94.1 current-source broad：reservation `eb3ccb674a614ef386847ff1a125ad1f`，job `aba96ff48cfd47b0a5ec41adfbb0f2dd`，run `1a81c8994a13411c88c0baea9d63138d`；base runtime `34 passed / 0 failed / 0 ignored / 0 measured / 0 filtered`，doc-tests `0 / 0`，退出码 `0`。

## 待完成验收

current-source broad 已绿色。Plugins07 保持源码不做非受管 Cargo，等待 FIFO 后 fresh 执行本记录
frontmatter 列出的 focused poison-recovery 行为门，随后完成最终独立复审并通过 Plugins07 M1
coordinator milestone commit。最终记录仍须补充 focused exact job/run、最终 review Critical/Important
结果及 immutable SHA；broad GREEN 不替代最终 focused 与原子提交。
