# Frameworks 06 G7 Tracked Owner Hard Cut Batch 6

> 本文件记录 `06-development-conventions-and-guardrails.md` 的 G7 docs 勾稽修正批次；父计划仍持有完整里程碑定义。

| 切片 | 完成项目 | 状态锚 | 日期 | 当前状态与证据 |
|---|---|---|---|---|
| M1 / G7 | Runtime UI、asset layout、shader、plugin world extension 与 fixed reflection owner hard cut, batch 6 | `frameworks_06_g7_tracked_owner_batch6_accepted_global_red` | 2026-07-16 | **batch accepted / global G7 RED**。将 6 份原本 clean 的模块文档中 21 个 machine-path violation 从已删除的 flat text geometry、graphics-owned native bitmap source cache、retired font-manifest integration target、layout-local badge lookup、graphics fullscreen descriptor、CoreRuntime world-extension state 与 `scene/reflect/fixed/*` owner 硬切到 HEAD 中唯一现存且已跟踪的 folder-backed text geometry、asset-type presentation registry、framework fullscreen builder、scene module world driver/level lifecycle 与 built-in reflection registration owners；对尚未进入 HEAD 的 foreign text source migration不抢先声明新 owner，并移除已退休 test target 的 machine-path 元数据。同步修正文档叙述，不保留 facade、alias、shim 或重复 owner。fresh `python tools/check_conventions.py --only docs --json` 从 batch5 后 251 missing / 48 affected docs 收敛到 234 / 42；exact scope violation 为 0，新增源码目标均已跟踪，scoped `git diff --check` 通过。独立 final review 为 0 Critical / 0 Important / 0 Minor。G7 仍全局 RED，因此不声明 M1 或 Plan06 完成。 |

## 精确文档范围

- `docs/assets-and-rendering/runtime-ui-graphics-integration.md`
- `docs/zircon_editor/ui/workbench/asset_content_layout.md`
- `docs/zircon_runtime/core/framework/render/shader.md`
- `docs/zircon_runtime/plugin/extension_registry.md`
- `docs/zircon_editor/core/editing/command.md`
- `docs/zircon_editor/scene/viewport/edit_mode_projection.md`

## 验收边界

- 本批为 docs-only G7 support correction，不运行 Cargo，也不冒充分支 CI。
- 仅引用 batch6 基线 HEAD 中已跟踪的源码 owner；foreign dirty Rust 文件不进入 manifest。
- 后续批次继续从剩余 234 missing 中选择已落入 HEAD 且无 active lease 的 owner。
