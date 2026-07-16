# Frameworks 06 G7 Tracked Owner Hard Cut Batch 8

> 本文件记录 `06-development-conventions-and-guardrails.md` 的 G7 docs 勾稽修正批次；父计划仍持有完整里程碑定义。

| 切片 | 完成项目 | 状态锚 | 日期 | 当前状态与证据 |
|---|---|---|---|---|
| M1 / G7 | AssetReference、export profile 与 first-party editor catalog owner hard cut, batch 8 | `frameworks_06_g7_tracked_owner_batch8_accepted_global_red` | 2026-07-16 | **batch accepted / global G7 RED**。将 3 份原本 clean 的计划/固定回传文档中 3 个 machine-path violation 从已删除的 runtime-local asset reference、扁平 export profile 与无实体 wildcard editor-plugin 路径，硬切到 batch8 基线 HEAD 中唯一现存且已跟踪的 `zircon_runtime_interface` AssetReference 定义、folder-backed project export profile 与生成式 editor plugin catalog owner；同步删除正文中的旧扁平 export 路径描述，不保留 facade、alias、shim、wildcard owner 或双实现。fresh `python tools/check_conventions.py --only docs --json` 的共享工作树快照由 227 missing / 37 affected docs 降为 224 missing / 34 affected docs，exact scope violation 为 0；新增源码目标均已在 batch8 基线 HEAD 跟踪，旧路径/通配 owner 精确扫描为 0，scoped `git diff --check` 通过。独立 final review 为 0 Critical / 0 Important / 0 Minor。G7 仍全局 RED，因此不声明 M1 或 Plan06 完成。 |

## 精确文档范围

- `docs/plans/zircon_runtime/render/18/fixed-2026-07-16-material-redirect-asset-contract-drift.md`
- `docs/plans/zircon_editor/editor/15-build-export-and-publishing.md`
- `docs/plans/zircon_editor/editor/09/2026-07-13-m1-extension-registry-hard-cut.md`

## 验收边界

- 本批为 docs-only G7 support correction，不运行 Cargo，也不冒充分支 CI。
- 仅引用 batch8 基线 HEAD 中已跟踪的源码 owner；foreign dirty 或 untracked Rust 文件不进入 manifest。
- 后续批次继续从共享快照剩余 224 missing 中选择已落入 HEAD 且无 active lease 的 owner。
