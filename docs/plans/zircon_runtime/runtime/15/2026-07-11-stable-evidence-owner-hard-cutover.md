---
related_code:
  - zircon_runtime/src/tests/runtime_absorption/structure_convention.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/evidence_ownership.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/asset_artifact_store.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/asset_tests/project.rs
implementation_files:
  - zircon_runtime/src/tests/runtime_absorption/structure_convention
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/evidence_ownership.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/asset_artifact_store.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/asset_tests/project.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
tests:
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/evidence_ownership.rs
  - tests/acceptance/runtime-priority-structure-review-managed-baseline.md
doc_type: milestone-detail
status: static_and_focused_passed_full_standalone_1297_1304_external_7
---

# Runtime 15 稳定证据归属硬切记录

日期：2026-07-11

## 动机与根因

最新 current-source standalone 结构门禁最初为 1224/1303。失败清单中 71
项并非产品结构错误，而是守卫读取三份已经退出活动流转的
`.codex/sessions` 会话记录，因文件不存在而在真实结构断言之前直接失败。会话记录按
仓库规则只承载短期协调状态，具体实现、验证与完成证据必须归编号计划或其编号归档；
因此恢复这些文件或增加缺失路径 fallback 都会维持错误的证据所有权。

## 实现

- 从 72 个结构守卫 owner 中删除三份已退役 Render/Text 会话记录的 75 个读取声明及
  81 个 session-source tuple 消费；随后按同一所有权规则删除剩余 active note 依赖，
  最终从 449 个守卫文件硬切全部 456 个 `.codex/sessions/` 路径消费。每个守卫已有的
  编号计划、编号归档、模块文档与 status-row assertions 保持不变。
- 新增 `runtime_15_structure_guards_use_durable_evidence_not_session_notes`，递归扫描
  `structure_convention` Rust 测试树，禁止整个 `.codex/sessions/` 路径族重新进入结构
  门禁，而不只是锁定本次已删除的三个文件名。
- 没有恢复旧 note、没有兼容路径、没有 shim、没有修改生产代码或放宽任何预算。
- 同步当前资产术语硬切：
  `asset/tests/assets/artifact_store/artifact_cache_assets.rs` 与
  `asset/tests/project/manager/artifact_cache_imports.rs` 取代已删除的 library 命名，守卫
  锚点改为当前 artifact-cache 测试名称。

## 验证

- Standalone harness 在全族硬切后从 current source 编译成功，退出码 0。
- 全族稳定证据守卫 1/1；代表 Render 01、Render 08、Text 03 三个此前缺失文件失败各
  1/1；两个当前 artifact-cache 路径锚点同步后也通过。
- 完整 standalone：1297 passed / 7 failed / 0 ignored，1304 tests，用时 305.04 秒。
  三份已退役 note 的 71 个失败全部消失，剩余 active note 依赖的硬切也没有引入新
  回归。剩余 7 项是
  Render/UI owner 漂移：production/test budgets、deferred-lighting dispatch 与
  render-graph workload，继续由对应 owner 处理。
- 独立 review-findings 回归 80/80，plan-status 回归 48/48；454 个受影响 Rust 文件
  `rustfmt --check` 通过，461 个范围路径 `git diff --check` 与冲突标记扫描通过。
- 规格复审与修正后的质量复审均通过；质量复审确认整个 session 路径族不变量已关闭，
  且 canonical plan/archive/module/status-row assertions 未被误删。
- 计划产出审计仍报告 19 个既有 Editor UI/Render notice 问题，本记录命中为 0。

## 状态裁决

状态：`static_and_focused_passed_full_standalone_1297_1304_external_7`。

本切片完成稳定证据归属硬切，但不将 Runtime 15 整体标记完成。完整 package/workspace
Cargo、Render/UI 文件拆分、shader dispatch 与其他跨计划门禁仍按各自 owner 保持可见。
