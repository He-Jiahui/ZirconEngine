---
handoff_kind: failure
status: open
created_at: 2026-07-26
summary_slug: core-root-facade-atomic-child-closure
origin_plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
fixing_plan: docs/plans/zircon_editor/editor/00-editor-architecture-overview.md
origin_child_dir: docs/plans/zircon_editor/editor/17
fixing_child_dir: docs/plans/zircon_editor/editor/00
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/mod.rs
  - zircon_editor/src/core/notifications
  - zircon_editor/src/core/recovery
  - zircon_editor/src/core/settings
  - zircon_editor/src/core/script_build
  - zircon_editor/src/core/sync
tests:
  - managed validation-copy replay from the immutable Editor00 root-facade integration SHA
  - original Editor17 P0 Decision publish-present-receipt regression gate
  - Editor13 script-build facade closure replay and Editor02 world-sync watch-map replay
---

# Editor00: core root facade child-manifest atomic closure

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md`
- 来源执行切片：M3.2 P0 Play pending-edit Decision notification authority
- 修复责任计划：`docs/plans/zircon_editor/editor/00-editor-architecture-overview.md`
- 交接原因：`00` 已将 `zircon_editor/src/core/mod.rs` 定义为 L1-L3 的薄聚合根和目标目录布局的唯一接线点。该根文件同时接入多个编号子计划，不能由任一个业务子计划通过删除导出、复制外部源码或依赖共享工作树来制造独立快照。

## 失败现象与复现证据

当前工作树的 `core/mod.rs` 同时声明 `notifications`、`recovery`、`settings`、`script_build` 与 `sync`。其中 P0 的 `notifications` 属于 Editor17；`recovery/settings` 属于 Editor17 的其他切片；`script_build` 属于 Editor13；`sync` 属于 Editor02。

现有 exact-manifest 验证副本若仅携带其中任一业务子计划的归因文件和 `core/mod.rs`，Rust 会在缺失的兄弟 `mod.rs` 上触发 E0583。该缺口已在 Editor13 的 `failure-2026-07-22-script-build-facade-validation-copy-closure.md` 中以 Editor02 validation copy 复现；P0 重新核验时确认该根因现已扩大为五个并行未提交子树的同一原子闭包问题。

因此，P0 已完成的 typed Decision 源码不能合法生成独立的 immutable validation copy 或 milestone commit；同样，Editor13/Editor02 不能吸收 Editor17 的 recovery/settings 或 notifications 业务实现来绕过根接线。

## 最低共享层根因

根接线 `core/mod.rs` 的所有权和 snapshot 闭包粒度不一致：计划架构将它指定为 Editor00 聚合根，但受管提交仍要求单一编号子计划的 exact manifest。多子树在同一根文件上并行新增 `pub mod` 后，任何单子计划的冻结副本都不再是可编译闭包，且根文件不能被四个子计划分别独占提交。

这不是 `notifications`、`script_build`、`sync`、`recovery` 或 `settings` 的领域逻辑缺陷；它是 Editor00 必须明确的根门面原子集成契约。

## 架构修复验收

- Editor00 建立一次受管的 root-facade integration manifest：`core/mod.rs` 与当前被它导出的新增子树必须在同一 immutable source snapshot 中自包含，并保留每个文件的原子 SHA、来源子计划和当前生命周期状态。
- 集成不得把领域所有权转移给 Editor00：Editor17、Editor13、Editor02 的业务代码分别需要其原计划的当前归因、评审和验证证据；Editor00 只拥有根接线与跨子树快照闭包的原子化提交机制。
- 所有相关子树的受管验证通过后，产生可作为 validation-copy baseline 的 integration SHA；Editor17 P0、Editor13 script-build 和 Editor02 watch-map 必须从该 SHA 重建各自的 immutable replay，而不是复用共享工作树或旧 reservation。
- 回传时必须明确每个子计划可恢复的 gate，并将本记录移动为来源 Editor17 的 `fixed-*` 记录；Editor13 与 Editor02 的既有 failure 记录继续保持各自领域验收，直到其原始 gate 真实通过。

## 禁止临时方案

- 不得删除 `core/mod.rs` 的外部 facade 行、添加空 `mod.rs`、test-only stub、compatibility shim 或条件编译占位来让单个副本通过编译。
- 不得把 `recovery/settings`、`script_build`、`sync` 或 `notifications` 的业务文件伪归因给另一个子计划，也不得用目录通配或共享工作树替代 immutable manifest。
- 不得将静态格式检查、历史 snapshot 或外部 Cargo 结果标记为本次原子 integration 通过。

## 修复结果与回传

Open state: `待 Editor00 root-facade integration manifest 与受管 SHA`; P0 的代码保持 `source_complete_static_green / managed_validation_blocked`，不宣称产品闭环。

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-07-26 | Editor17 M3.2 P0 -> Editor00 root facade integration | `open / architecture-handoff-recorded` | 核验 `core/mod.rs` 同时导出 five uncommitted child owners；复核 Editor13 已有 E0583 validation-copy handoff 和 Editor00 `core/` 目标布局归属。已建立此 child-record-only 交接，明确 immutable manifest、原子 SHA/来源归因、受管 replay 和无临时 shim 的验收要求；未执行 Cargo、未创建代码提交。 |
| 2026-07-26 | Editor00 root facade integration r1 | `source_closure_static_green / managed_validation_pending` | 精确 9 条租约已由 `editor00-core-facade-integration-r1-20260726` 领取。对根接线、P0 Context wiring 和五个 child roots 的 29 个 Rust 文件执行 Rust 2024 `rustfmt --check` 通过；所有 `core/mod.rs` 声明都有物理模块入口；`core/{notifications,recovery,settings,script_build,sync}` 的 `crate::ui` 依赖扫描为 0；最大生产 owner 为 `recovery/autosave.rs` 551 行。已机械规范化现有格式漂移；未执行 Cargo、未进行独立审查、未创建 integration SHA。 |
| 2026-07-26 | Editor00 immutable validation-copy admission | `managed_validation_blocked / handed_to_coordinator01` | snapshot `1089` 已冻结且 29 个 Rust SHA 当前复核为 `0` drift；固定 `zr_vm` commit descriptor 已消除外部 local-path 未钉入问题。随后 validation-copy `d1c2fb6f…` 仍在 `baseline_archive` 返回 `validation_copy_unowned_path`，而同一 29 个文件的 `lease claim` 回执为 acquired、`baseline attribute` 却返回 `baseline_lease_missing`。已交接至 Coordinator01 `failure-2026-07-26-live-lease-attribution-validation-copy-divergence.md`；禁止直接 SQLite attribution、共享工作树 Cargo 或空壳模块绕过。 |
