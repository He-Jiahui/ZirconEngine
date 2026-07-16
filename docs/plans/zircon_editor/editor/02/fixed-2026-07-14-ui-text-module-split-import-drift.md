---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
summary_slug: ui-text-module-split-import-drift
origin_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
fixing_plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
origin_child_dir: docs/plans/zircon_editor/editor/02
fixing_child_dir: docs/plans/zircon_runtime/text/01
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/resolved_batches.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/prepare_report.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/font_assets.rs
tests:
  - cargo test -p zircon_runtime --lib scene:: --locked
resolved_at: 2026-07-14
---


# Text01：screen-space UI text 模块拆分导入漂移

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 来源执行切片：Editor02 M1 声明的默认特性 runtime scene 验收门禁
- 修复责任计划：`docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md`
- 交接原因：失败由 Text01 当前 variable-font product 切片把 text batch resolution 抽入 folder-backed sibling owner 后，既有 report/test consumer 未同步显式导入导致；Editor02 不拥有字体准备与 UI text renderer 模块边界。

## 失败现象与复现证据

受管 Windows job `5cdb16f8ac874ccfba5581ba6d66f725` 执行：

```powershell
cargo test -p zircon_runtime --lib scene:: --locked
```

编译 `zircon_runtime` lib-test 时出现 8 条错误：

- `prepare_report.rs:4` 从 `super` 导入已不在 parent scope 的 `ResolvedScreenSpaceUiTextBatches`，产生 1 条 `E0432`；
- `text/tests.rs` 仍依赖 `use super::*` 获得该类型，产生 4 条 `E0433`；
- 同一测试仍依赖 parent scope 的 `effective_text_render_mode`，而当前 owner 已在 `font_assets.rs`，产生 3 条 `E0425`。

完整日志：`E:\ZirconBuilds\editor02-m1-runtime-scene-default-final-20260714.log`。错误出现前没有 Editor02 scene/world/inspection 诊断。

## 最低共享层根因

Text01 将 `resolve_text_batches` 与 `ResolvedScreenSpaceUiTextBatches` 收口到 `resolved_batches.rs`，并把 render-mode 裁决保留在 `font_assets.rs`；但 sibling report 与 tests 仍把这些符号当作 `text.rs` parent 的隐式私有导入。当前 folder-backed owner 是正确方向，漂移位于 sibling consumer 没有指向真实 owner，而不是类型或函数缺失。

## 架构修复验收

- `prepare_report.rs` 与 `tests.rs` 从各自 canonical sibling owner 显式导入所需类型/函数；`text.rs` 继续保持薄装配，不恢复供 child 测试搭便车的 parent 私有导入。
- 不公开 `resolved_batches`/`font_assets` child module，不新增 compatibility re-export、重复类型或测试影子函数。
- 原 `cargo test -p zircon_runtime --lib scene:: --locked` 不再出现上述 8 条 `E0432/E0433/E0425`，并继续执行 Editor02 scene 测试。

## 禁止临时方案

- 不把 child module 改为 public/pub(crate) 来迁就旧隐式路径。
- 不在 `text.rs` 恢复无生产用途的 test-only parent import，不复制 `ResolvedScreenSpaceUiTextBatches` 或 render-mode 逻辑。
- 不修改 Editor02 world-sync、scene inspection 或 Shader04 文件来绕过 Text01 编译失败。

## 修复结果与回传

- 根因：Text01 folder-backed split moved ResolvedScreenSpaceUiTextBatches and effective_text_render_mode to sibling owners while prepare_report/tests still depended on parent implicit imports.
- 架构修复：prepare_report.rs and tests.rs now import ResolvedScreenSpaceUiTextBatches from resolved_batches and effective_text_render_mode from font_assets; parent modules remain thin and no compatibility re-export was added.
- 验证：Managed job 6884ef43242443ffa2c793116b36b443 reran cargo test -p zircon_runtime --lib scene:: --locked after the fix and reached test execution with no E0432/E0433/E0425 Text01 diagnostics; complete prior managed log E:\ZirconBuilds\editor02-m1-runtime-scene-default-after-text01-fix-20260714.log compiled and ran 1702 tests, with remaining failures already routed to Shader04, Shader06 stale-race resolution, and Plugins08.
- 回传：Text01 sibling-owner imports are fixed and the original compile blocker is closed; remaining scene-filter test failures are external lifecycle handoffs.
