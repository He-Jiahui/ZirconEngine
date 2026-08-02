---
related_code:
  - zircon_runtime_interface/src/runtime_api/api_table.rs
  - zircon_runtime_interface/src/lib.rs
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map_tests.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/ffi.rs
  - zircon_runtime/src/dynamic_api/session/operation.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - docs/zircon_runtime/dynamic_api/session.md
  - docs/plans/zircon_editor/editor/03/fixed-2026-07-16-dynamic-runtime-v1-fallback-reintroduced.md
implementation_files:
  - zircon_runtime_interface/src/runtime_api/api_table.rs
  - zircon_runtime_interface/src/lib.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
tests:
  - dynamic_runtime_api_boundary_audit (48 source files, 17 wrappers, legacy V1 hits 0, risks 0)
  - managed Windows zircon_runtime_interface job d52d3bd9891941bb9d72ca3f8746dffc (exit 0)
  - managed Windows zircon_app job 376c7ad2e2f141b79b3c9ad39e979557 (exit 0)
  - managed Windows zircon_runtime dynamic_api green3 job dae66a6c77cb420d9012dd95f56e0d1b (98 passed, 1 stale source guard failed, 10 ignored)
  - managed Windows current-source exact compile job 1f2ee891ade941eb9c21229bb82f9b3e (exit 0)
doc_type: milestone-detail
---

# Runtime10 Dynamic Runtime V2-only Hard Cut Closeout

Plan: docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
Milestone: M1/M3 cross-cut
Status: runtime_10_dynamic_runtime_v2_only_failure_fixed_and_returned
Date: 2026-07-16

## 状态与完成项目

| 范围 | 状态 | 完成证据 |
|---|---|---|
| Runtime 函数表硬切 | `fixed` | 产品面只保留 `ZrRuntimeApiV2`、V2 symbol 与 V2 loader；V1 table/export/loader fallback 为零。 |
| required group | `fixed` | plugin-event mirror 与 operation submit/poll/harvest 均为加载期必需组，共 17 个 session operation wrapper。 |
| session owner 拆分 | `fixed` | `session.rs` 为 51 行路由门面；FFI、construction、state、operation 与 linked-session 分属子 owner，未恢复兼容 façade。 |
| interface test owner | `fixed` | source-map tests 从生产 `source_map.rs` 的嵌套路径移到 sibling `source_map_tests.rs`，生产 owner 不再挂 test-only child。 |
| Failure 回传 | `fixed` | 原 open artifact 已原子返回 Editor03：`fixed-2026-07-16-dynamic-runtime-v1-fallback-reintroduced.md`。 |

## 架构结果

DTO 与 host callback 中稳定的 `V1` 命名不等于保留旧 runtime 产品表。当前唯一可装载产品表是
`ZrRuntimeApiV2`；缺 symbol、表长度不足或 mirror/operation required function 缺失都会返回
显式 loader error，不回退、不静默降级，也不在 Editor03 增加 capability-unavailable 绕行。

动态 session 的正常错误校验仍在 Rust ABI owner；`exports.rs` 只负责最终 C ABI panic
containment。结构守卫已跟随 `ffi.rs`、`state.rs`、`operation.rs` 的真实 owner，避免旧
`session.rs` 文本布局成为兼容约束。

## 验证判词

- 结构审计：48/48 source files，10/10 function tables，17/17 FFI wrappers，V1 legacy hits
  `[]`，missing owner/wrapper `[]`，`risks = []`。
- `zircon_runtime_interface` 与 `zircon_app` 完整受管 Windows 门均 exit 0。
- Runtime `dynamic_api` 当前源码运行在最后一次全过滤中达到 98 passed / 1 failed / 10
  ignored；唯一失败是本轮随后修正的 headless source guard。修正后受管 exact current-source
  编译 exit 0，并以直接 source assertion 验证 rendered profile 集合不含 Minimal/Headless。
- 完整 `zircon_runtime` package gate 仍受 Frameworks05/Text 并行 integration 工作影响；本记录
  不把该 foreign gate 冒充为 Runtime10 通过，也不提升 Runtime10 总计划状态。

## 剩余范围

Runtime10 保持 `in_progress`。M2 runtime UI/editor 上行门与 M3 完整 package/cdylib product
回归仍按原计划执行；本记录只关闭 V1 fallback 回归 Failure 与当前 V2-only owner 切片。
