# Frameworks 06 G7 Tracked Owner Hard Cut Batch 9

> 本文件记录 `06-development-conventions-and-guardrails.md` 的 G7 docs 勾稽修正批次；父计划仍持有完整里程碑定义。

| 切片 | 完成项目 | 状态锚 | 日期 | 当前状态与证据 |
|---|---|---|---|---|
| M1 / G7 | Editor asset metadata historical owner hard cut, batch 9 | `frameworks_06_g7_tracked_owner_batch9_accepted_global_red` | 2026-07-16 | **batch accepted / global G7 RED**。从 clean 的 runtime/editor boundary 文档中删除已退役的 editor-local `editor_meta.rs`、两个已删除的细粒度 asset-metadata test owner，以及没有模块声明或测试的空 `asset_metadata/mod.rs`；文档已同时指向 batch9 基线 HEAD 中已跟踪的 runtime `asset/project/meta.rs`、current editor asset catalog/project sync owner、asset-manager boundary 与 runtime/editor boundary contract，因此不新建 facade、alias、shim、空测试替身或第二套 metadata 规则。fresh `python tools/check_conventions.py --only docs --json` 的共享工作树快照由 224 missing / 34 affected docs 降为 218 missing / 33 affected docs，exact scope violation 为 0；三个 missing 路径与空 route 精确扫描为 0，current metadata targets 均由 batch9 基线 HEAD 跟踪，scoped `git diff --check` 通过。独立审查首轮发现并修正 1 Important（空 route 仍被列为 owner），修正后 final review 为 0 Critical / 0 Important / 0 Minor。G7 仍全局 RED，因此不声明 M1 或 Plan06 完成。 |

## 精确文档范围

- `docs/editor-and-tooling/runtime-editor-boundary-cleanup.md`

## 验收边界

- 本批为 docs-only G7 support correction，不运行 Cargo，也不冒充分支 CI。
- 本批只删除不再存在的历史 owner；已在同一文档登记的 current tracked owner 保持唯一事实来源。
- 后续批次继续从共享快照剩余 218 missing 中选择已落入 HEAD 且无 active lease 的 owner。
