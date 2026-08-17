Plan: docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
Milestone: M2.4
Status: completed
Files: ["Cargo.lock", "Cargo.toml", "docs/plans/optimize/zircon_runtime_interface/01/2026-08-17-host-output-policy-convergence.md", "docs/plans/optimize/zircon_runtime_interface/01/fixed-2026-08-17-managed-validation-asset-creation-include-path.md", "zircon_app/Cargo.toml", "zircon_app/src/entry/runtime_library/runtime_library_error.rs", "zircon_app/src/entry/runtime_library/runtime_session.rs", "zircon_app/src/entry/runtime_library/runtime_session/foreign_output.rs", "zircon_app/src/entry/runtime_library/runtime_session/foreign_output/performance_tests.rs", "zircon_app/src/entry/runtime_library/runtime_session/foreign_output/tests.rs", "zircon_app/src/entry/runtime_library/runtime_session/operation.rs", "zircon_app/src/entry/runtime_library/runtime_session/owned_buffer.rs", "zircon_app/src/entry/runtime_library/runtime_session/tests.rs", "zircon_editor/Cargo.toml", "zircon_editor/src/core/gateway/error.rs", "zircon_editor/src/core/gateway/session/frame.rs", "zircon_editor/src/core/gateway/session/gateway.rs", "zircon_editor/src/core/gateway/session/operations.rs", "zircon_editor/src/core/gateway/session/output.rs", "zircon_editor/src/core/gateway/session/overlay.rs", "zircon_editor/src/core/gateway/session/plugin_events.rs", "zircon_editor/src/core/gateway/session/profile.rs", "zircon_editor/src/core/gateway/session/tests.rs", "zircon_editor/src/core/gateway/session/viewport.rs", "zircon_editor/src/core/gateway/session/world_sync.rs", "zircon_editor/src/core/play/tests.rs", "zircon_editor/src/tests/gateway/session/construction.rs", "zircon_editor/src/tests/gateway/session/fixture.rs", "zircon_editor/src/tests/gateway/session/output_ownership.rs", "zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_window_menus/asset_creation.rs", "zircon_editor/src/tests/runtime_event_consumer_bounded_pump.rs", "zircon_editor/src/tests/runtime_event_consumer_bounded_pump/real_runtime_abi.rs", "zircon_editor/tests/runtime_foreign_output_policy.rs", "zircon_runtime_host/Cargo.toml", "zircon_runtime_host/src/lib.rs", "zircon_runtime_host/src/foreign_output/budget.rs", "zircon_runtime_host/src/foreign_output/decode.rs", "zircon_runtime_host/src/foreign_output/error.rs", "zircon_runtime_host/src/foreign_output/item_count.rs", "zircon_runtime_host/src/foreign_output/kind.rs", "zircon_runtime_host/src/foreign_output/metrics.rs", "zircon_runtime_host/src/foreign_output/mod.rs", "zircon_runtime_host/src/foreign_output/owned_buffer.rs", "zircon_runtime_host/src/foreign_output/policy.rs", "zircon_runtime_host/src/foreign_output/state.rs", "zircon_runtime_host/src/foreign_output/tests.rs", "zircon_editor/src/core/recovery/autosave_adapter.rs"]

# Runtime Interface 01 M2.4 host-output policy convergence

## Scope Delivered

本切片以 `zircon_runtime_host` 为唯一 host-side foreign-output policy owner，App 与 Editor
共享 session-wide protocol fuse、预算、指标及释放合同；旧的分散 host 实现已直接收敛，
未保留兼容分支或临时旁路。

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M2.4 | M2.4-T testing：新增 `zircon_runtime_host` 作为 `zircon_runtime_interface` 之上的 host-only owner，统一 7 类 runtime 输出的所有权校验、释放、JSON 字节/条目/深度/时间预算、指标和 session-wide protocol fuse；App `RuntimeSession` 与 Editor `SessionGateway` 共享同一 `Arc<RuntimeForeignOutputState>`，所有 Editor FFI 分发在调用前检查该状态；纯 interface crate 不再承载 `std::sync` 或实现状态 | 通过 | 2026-08-17 | `zircon_runtime_host` 完整单测 `8/8`；App foreign-output 行为/边界/性能 `17/17`；Editor 外部注入熔断集成测试 `1/1`，确认熔断后 runtime present 调用数为 `0`。interface focused boundary 仅剩既有 `ui/surface/render/text_geometry/source_map/line.rs` 与 `ui/surface/render/text_layout.rs` 两处 `std::sync`，无 host-output 路径。release 926 B / 256 items / 2,000 iterations 独立执行 5 轮：P50 `6.0-10.1 us`，P99 `13.0-34.3 us`，吞吐 `104,288-159,750 payloads/s`；中位 P50/P99/吞吐为 `6.2 us / 14.6 us / 146,329 payloads/s`，最差 P99 为 10 ms 预算的 `0.343%`，保留 `99.657%` 余量。该结果证明共享策略达标，不宣称相对原 App 单入口基线有确定提升。初始 Editor 托管票据在 `closure_planning` 被 UI12 `asset_creation.rs` 少回退一级的 `include_str!` 路径阻断；support-first 修复已指向唯一 production owner，并通过 `fixed-2026-08-17-managed-validation-asset-creation-include-path.md` 回传。修复后的托管 lib-test 已进入 Cargo，暴露 243 个不属于本切片的既有 `cfg(test)` 编译错误；最终候选验收使用同一 48 文件 manifest 的独立非 `cfg(test)` Editor integration gate。M2 其余 typed carrier、producer-side limit source 与分页/stream 改造继续保持 pending。 |

## Fresh Testing Evidence

- 管理式 Windows 验证票据 `7c6cae14d08847e19963c09f142dccad` 执行
  `cargo test -p zircon_editor --locked --test runtime_foreign_output_policy`，结果 1 passed / 0 failed。
- `zircon_runtime_host` 单测 8/8、App foreign-output 行为/边界/性能测试 17/17 均通过。
- release 五轮中位 P50/P99/吞吐为 `6.2 us / 14.6 us / 146,329 payloads/s`；
  最差 P99 `34.3 us`，占 10 ms 预算 `0.343%`。

## Review

独立复核结论为 Critical 0 / Important 0。保留风险是 admission check 与 FFI 调用之间尚无
admission token，且 decode deadline 为事后检测；两项均已在父计划后续 producer-side 与
可中断 decode 工作中登记，不削弱本切片对当前 host-output 路径的统一熔断和预算保证。
