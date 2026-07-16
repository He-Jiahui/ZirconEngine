# Frameworks 06 G7 ZrVM Owner Hard Cut Batch 5

> 本文件记录 `06-development-conventions-and-guardrails.md` 的 G7 docs 勾稽修正批次；父计划仍持有完整里程碑定义。

| 切片 | 完成项目 | 状态锚 | 日期 | 当前状态与证据 |
|---|---|---|---|---|
| M1 / G7 | ZrVM backend implementation owner hard cut, batch 5 | `frameworks_06_g7_zrvm_owner_batch5_accepted_global_red` | 2026-07-16 | **batch accepted / global G7 RED**。将 5 份原本 clean 的 Runtime/Plugin 文档中 12 个重复 machine-path violation 从已吸收到插件的 `zircon_runtime/src/script/vm/backend/zr_vm_project_backend*` owner 硬切到唯一现存且已跟踪的 `zircon_plugins/zr_vm_language/runtime/src/{backend,real_backend/{host_modules,instance}}.rs`；同步删除“两份 real adapter”叙述，明确单一 feature-gated plugin instance owner，不保留 runtime facade、alias、shim 或重复实现。fresh `python tools/check_conventions.py --only docs --json` 从 batch4 后 263 missing / 53 affected docs 收敛到 251 / 48；exact scope violation 为 0，3 个新增唯一源码目标均已跟踪，scoped `git diff --check` 通过。独立 review 首轮为 0 Critical / 1 Important / 0 Low；修正 runtime-neutral state-migration 文档中的复数 adapter 叙述后，最终 fresh re-review 为 0/0/0。预提交完整性复核发现两份 host-interface 文档拟指向的 `real_backend/extension_host.rs` 仍属于 Plugins08 M4 未提交源码；插件 host-interface 文档还同时保留未落地的 `host_interfaces.rs` owner，故两份 host-interface 文档均整体退出本批并保留为 Plugins08 owner 落地后的 G7 工作，未用工作树存在性冒充 HEAD 架构事实。G7 仍全局 RED，因此不声明 M1 或 Plan06 完成。 |

## 精确文档范围

- `docs/zircon_plugins/zr_vm_language/state_migration.md`
- `docs/zircon_runtime/plugin/bridge.md`
- `docs/zircon_runtime/script/vm/host/bridge_host_module.md`
- `docs/zircon_runtime/script/vm/state_migration.md`
- `docs/zircon_runtime/script/vm/vampire_gameplay.md`

## 验收边界

- 本批为 docs-only G7 support correction，不运行 Cargo，也不冒充分支 CI。
- fresh 独立 review 已达 0 Critical / 0 Important；通过 coordinator maintenance finalize 提交精确 6 文件 manifest。
- 后续批次继续从剩余 251 missing 中选择 clean owner；其中两份 deferred host-interface 文档贡献 6 条待 Plugins08 源码 owner 进入 HEAD 后再硬切的 violation，foreign dirty 文档保持原会话归属。
