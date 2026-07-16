# Frameworks 06 G7 Tracked Owner Hard Cut Batch 7

> 本文件记录 `06-development-conventions-and-guardrails.md` 的 G7 docs 勾稽修正批次；父计划仍持有完整里程碑定义。

| 切片 | 完成项目 | 状态锚 | 日期 | 当前状态与证据 |
|---|---|---|---|---|
| M1 / G7 | Asset preview、ZrVM、reflection、download、viewport 与 Editor entry owner hard cut, batch 7 | `frameworks_06_g7_tracked_owner_batch7_accepted_global_red` | 2026-07-16 | **batch accepted / global G7 RED**。将 6 份原本 clean 的模块/固定回传文档中 8 个 machine-path violation 从已删除的 preview-refresh palette、runtime-local ZrVM backend、`scene/reflect/fixed` 刚体适配器、plugin-local hash、flat retained viewport 与旧 Editor binary path，硬切到 HEAD 中唯一现存且已跟踪的 asset-type registry/thumbnail descriptor、ZrVM plugin backend、built-in reflection registration、现有 BLAKE3 fetch/progress owner、folder-backed viewport 与 `bin/editor.rs` owner；删除重复 hash owner，不保留 facade、alias、shim 或双实现。fresh `python tools/check_conventions.py --only docs --json` 的共享工作树快照为 227 missing / 37 affected docs，exact scope violation 为 0；相对于 batch7 基线 HEAD 的 234 / 42，本批精确移除 8 条并使 6 份候选文档退出 G7 违规集，但同时有 foreign session 新增 1 条不属于本 manifest 的 violation，因此不把共享快照的净变化冒充本批独占结果。新增源码目标均已跟踪，scoped `git diff --check` 通过。独立 final review 为 0 Critical / 0 Important / 0 Minor。G7 仍全局 RED，因此不声明 M1 或 Plan06 完成。 |

## 精确文档范围

- `docs/zircon_runtime/asset/project_asset_type_convergence.md`
- `docs/zircon_runtime/graphics/tests/project_render.md`
- `docs/plans/zircon_editor/editor/08/fixed-2026-07-12-rigid-body-sleep-policy-consumer-cutover.md`
- `docs/plans/zircon_plugins/07/fixed-2026-07-14-zrpack-blake3-contract-drift.md`
- `docs/zircon_editor/ui/retained_host/app/render-submission.md`
- `docs/plans/zircon_runtime/render/18/fixed-2026-07-13-navigation-runtime-driver-manager-layering.md`

## 验收边界

- 本批为 docs-only G7 support correction，不运行 Cargo，也不冒充分支 CI。
- 仅引用 batch7 基线 HEAD 中已跟踪的源码 owner；foreign dirty Rust 文件不进入 manifest。
- 后续批次继续从共享快照剩余 227 missing 中选择已落入 HEAD 且无 active lease 的 owner。
