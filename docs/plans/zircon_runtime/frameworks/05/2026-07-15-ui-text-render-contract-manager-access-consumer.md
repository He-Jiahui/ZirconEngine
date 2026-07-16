---
related_code:
  - zircon_runtime/tests/runtime_ui_text_render_contract.rs
  - zircon_runtime/tests/support/project_asset_runtime.rs
implementation_files:
  - zircon_runtime/tests/runtime_ui_text_render_contract.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
tests:
  - managed Windows cargo test -p zircon_runtime --locked (job 8f545c28290d42f791e00940e764c659; red E0308 at the stale integration-test consumer)
  - managed Windows cargo test -p zircon_runtime --locked (job e9535b360a6642498bcac074488eccd4; old E0308 absent, advanced to unrelated font_asset_manifest_contract E0432)
  - managed Windows cargo test -p zircon_runtime --test runtime_ui_text_render_contract --locked --offline (job 8f0af97d9f1d4a5a94b72c21a1ceab03; consumer compiled, SDF 2/2 passed, native-font 6/6 failed with zero raster pixels under active Editor Layout 15 text changes)
  - rustfmt --edition 2021 --check zircon_runtime/tests/runtime_ui_text_render_contract.rs
  - git diff --check -- zircon_runtime/tests/runtime_ui_text_render_contract.rs
doc_type: milestone-detail
---

# Frameworks05 UI Text Render Contract Manager Access Consumer

Plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
Parent milestone: M4
Status: consumer_hard_cut_passed_upward_native_text_owner_blocked
Date: 2026-07-15
Files: ["docs/plans/zircon_runtime/frameworks/05/2026-07-15-ui-text-render-contract-manager-access-consumer.md", "zircon_runtime/tests/runtime_ui_text_render_contract.rs"]

## 状态与完成项目

| 切片 | 状态 | 完成证据 |
|---|---|---|
| UI text integration consumer manager-access hard cut | `consumer_hard_cut_passed_upward_native_text_owner_blocked` | `runtime_ui_text_render_contract` 已复用共享 `ProjectAssetTestRuntime`，通过 `ProjectAssetManagerAccess` 构造 `WgpuRenderFramework`，并在测试函数作用域内保持 `CoreRuntime` 存活。fresh 受管集成门已完整编译并执行：SDF 2/2 通过，6 个 native-font 用例在活动 Editor Layout 15 lazy-coverage 变更下全部产生零像素；失败位于更低的共享字体 owner，不回退本次访问层硬切。 |

## 根因与修复

Frameworks05 已将 `WgpuRenderFramework::new` 的构造边界硬切为
`ProjectAssetManagerAccess`，但 `runtime_ui_text_render_contract` 仍直接传入
`Arc<ProjectAssetManager>`。这不是生产 owner 缺少适配器，而是集成测试消费者仍停留在旧
API。

修复复用现有 `tests/support/project_asset_runtime.rs`：测试先注册并激活真实的
`ProjectAssetManager` manager service，再把 versioned access 传给 render framework。
`ProjectAssetTestRuntime` 保留到函数返回，避免构造后 manager service 生命周期失效。
没有增加 concrete-Arc adapter、旧 resolver、compatibility shim、双 owner 或静默 fallback。

## 验证

- job `8f545c28290d42f791e00940e764c659` 提供红态：编译器在测试调用点报告
  `WgpuRenderFramework::new` 期望 `ProjectAssetManagerAccess`、实际收到
  `Arc<ProjectAssetManager>`。
- job `e9535b360a6642498bcac074488eccd4` 在同一默认特性 Runtime 测试矩阵中不再报告该
  E0308，并继续编译到 `font_asset_manifest_contract` 的独立 E0432，证明最低消费者修复已
  进入更深验证层。
- job `8f0af97d9f1d4a5a94b72c21a1ceab03` fresh 编译并运行当前集成测试，证明 manager-access
  consumer 已越过编译、框架初始化、frame submit 与 capture；结果为 SDF `2 passed / 0 failed`，
  native-font `0 passed / 6 failed`，所有失败均报告可见 glyph 像素或 opacity delta 为 0。
- 当前活动会话 `editor-layout15-visual-refinement-20260714` 独占修改
  `font/coverage.rs`、`font/database.rs`、`fallback_spans.rs` 与
  `cosmic/font_system_cache.rs`，状态也明确包含 Runtime Text lazy-coverage 验证。该回归按最低
  共享层归该 owner；Frameworks05 不建立 concrete-manager、SDF 强制路由或字体 fallback 绕路。
- 尝试建立正式 Failure 时，协调器审计确认该边会闭合现有
  `Editor Layout 15 -> ... -> Runtime15 -> Frameworks05 -> Editor Layout 15` 依赖环，因此未保留
  Failure 节点或计划链接；当前以活动 owner 阻塞状态跟踪，待其提交或恢复后重新执行同一门禁。
- scoped rustfmt 与 diff check 通过。
- 在 native-font 8/8 集成门恢复前，本记录不把 Frameworks05 M4 或 Runtime15 标记为完成。

## 剩余范围

本记录只覆盖 UI text integration consumer。Frameworks05 M4 总里程碑、Runtime15 render
owner 收口、depth-prepass Failure 回传和全 workspace 验证仍按原计划拓扑继续执行。
