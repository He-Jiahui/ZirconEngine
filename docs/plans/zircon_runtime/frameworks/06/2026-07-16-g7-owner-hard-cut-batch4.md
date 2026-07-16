# Frameworks 06 G7 Owner Hard Cut Batch 4

> 本文件记录 `06-development-conventions-and-guardrails.md` 的 G7 docs 勾稽修正批次；父计划仍持有完整里程碑定义。

| 切片 | 完成项目 | 状态锚 | 日期 | 当前状态与证据 |
|---|---|---|---|---|
| M1 / G7 | Project contract 与 text-geometry owner hard cut, batch 4 | `frameworks_06_g7_owner_batch4_accepted_global_red` | 2026-07-16 | **batch accepted / global G7 RED**。将 6 份原本 clean 的文档中 12 个重复 machine-path violation 从退役 `plugin/export_profile.rs`、`plugin/project_plugin_manifest/*` 与 flat `ui/surface/render/text_geometry.rs` owner 硬切到唯一现存 `core/framework/project/{export_profile,project_plugin_manifest}/*` 与 folder-backed `ui/surface/render/text_geometry/mod.rs` owner；同步正/反斜杠历史 rustfmt/diff 命令与正文源码指针，不保留 alias、shim 或旧路径说明层。fresh `python tools/check_conventions.py --only docs --json` 从 batch3 后 275 missing / 59 affected docs 收敛到 263 / 53；exact scope violation 为 0，6 个新增唯一目标全部存在，scoped `git diff --check` 通过。独立 review 首轮为 0 Critical / 1 Important / 0 Low；修正 3 条反斜杠旧命令后，最终 fresh re-review 为 0/0/0。G7 仍全局 RED，因此不声明 M1 或 Plan06 完成。 |

## 精确文档范围

- `docs/editor-and-tooling/editor-host-minimal-plugin-loading.md`
- `docs/engine-architecture/plugin-optional-feature-bundles.md`
- `docs/engine-architecture/runtime-sound-extension.md`
- `docs/engine-architecture/runtime-tech-stack.md`
- `docs/runtime-plugins/profile-selection.md`
- `docs/zircon_runtime/ui/text/layout_engine.md`

## 验收边界

- 本批为 docs-only G7 support correction，不运行 Cargo，也不冒充分支 CI。
- fresh 独立 review 已达 0 Critical / 0 Important；通过 coordinator maintenance finalize 提交精确 7 文件 manifest。
- 后续批次继续从剩余 263 missing 中选择 clean owner；foreign dirty 文档保持原会话归属。
