---
plan_source: docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
related_code:
  - zircon_runtime/src/scene/dynamic_scene/document/
  - zircon_runtime/src/scene/dynamic_scene/scene/
  - zircon_runtime/src/scene/dynamic_scene/session/slot/summary.rs
tests:
  - zircon_runtime/tests/plan11_scene_serialization_contract.rs
status: slice-accepted-managed-native-slice-closeout-ready
---

# Editor11 M2.2 DynamicScene 当前源验收

Plan: docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
Milestone: M2.2
Status: accepted
Files: ["docs/plans/zircon_editor/editor/11/2026-07-17-dynamic-scene-current-source-acceptance.md"]

## 产出记录与时间

| 时间 | 里程碑/切片 | 状态 | 完成项目与证据 | 后续门禁 |
| --- | --- | --- | --- | --- |
| 2026-07-17 | M2.2 DynamicScene 双写字段硬删除 | `当前源验收-8/8-复核重要项已修正-原生切片受管关闭就绪` | M2.2 exact16 已由集成提交 `ad2c6f989cfff927ff5679467ca0cc71e2e20c0e` 吸收；验证基线 `6c8957ab86925d0eed3e55a5914157dc891f382e` 与关闭前 HEAD `7ed443cddea6ea24ccc58e43343bb7b2f201f38e` 相对该集成提交均不存在 exact16 差异。M2.2 业务代码、测试、既有 child record 与模块文档均为 clean；业务 milestone manifest 仅包含本验收记录，父计划复选框/链接属于受保护计划定义，另走 plan-maintenance 提交。先前内容独立复核为 Critical/Important/Minor=`0/0/0`；本轮关闭复核发现 `0/1/0` 的 lifecycle 措辞错误，现已改为区分“checker 能力可用”与“canonical Failure 尚未回传”，最终 current-hash review 由协调器账本记录。12 个 Rust 文件的 scoped `rustfmt --edition 2024 --check --config skip_children=true` 与 scoped `git diff --check` 通过。Windows canonical reservation `5e6fa561e77747528e9f4f2cf353b23c`、job `133ed517e6c146ecaf6717a52f8fc5bb`、run `35fd83c41e5c4c40bdb70afdfa1267bd` 在 target `D:\cargo-targets\zircon-engine\pool\5aa6444e0ec3fcfaa759c4e0c51385df96b1cc3340d097a344bf63fb32eda602` 执行 `cargo test -p zircon_runtime --locked --offline --test plan11_scene_serialization_contract --jobs 1 -- --test-threads=1`，结果 8 passed / 0 failed、exit 0；job 已 `released`，`live_process_pids=[]`。此前唯一外部 Text consumer 阻断已由 Frameworks05 回传 `fixed-2026-07-16-text-raster-pool-zircon-error-consumer.md`。 | M2.2 切片当前源已验收，但父 M2 仍等待 M2.1 preferences/keymap/journal/layout 四面接壳，不得标记父里程碑完成。Coordinator01 checker 当前能力已支持原生 `M2.2`/空 index 合同；其 canonical `failure-2026-07-16-native-slice-closeout-checker-staged-index-contract-drift.md` 仍等待 Editor02 M1.3 复放后正式回传。该 lifecycle 的 origin/fixer 为 Editor02/Coordinator01，Plan11 的 `failure open` 为空，故不构成本节点 blocker；本节点只提交本记录，不重复制造业务实现提交，也不手工 staging。 |

## 架构验收结论

- `$zircon.header.schema_version` 是当前 DynamicScene 的唯一版本权威；生产结构、capture、writer 与 root 不保留退役 `format_version` 字段、常量、别名或兼容重导出。
- v0→v1→v2 迁移保持顺序链；v1→v2 必须校验历史内嵌版本为 1 后删除，future header 在 payload 解码前拒绝，当前 v2 重新携带旧字段返回 typed decode error。
- 本记录只确认 M2.2 当前源与证据，不改变 M2.1、M3.2 或父 M2/M3 的 pending 状态。

## Scope Delivered

- 业务关闭清单仅包含本记录；父计划状态同步由受保护 plan-maintenance 路径提交。M2.2 exact16 实现已在 `ad2c6f989cfff927ff5679467ca0cc71e2e20c0e` 吸收，未重复提交业务文件。

## Fresh Testing Evidence

- Windows 受管作业 `133ed517e6c146ecaf6717a52f8fc5bb` / run `35fd83c41e5c4c40bdb70afdfa1267bd` 通过 8/8；关闭前 HEAD 对 exact16 的集成后差异为 0。

## Review

- 关闭复核发现的 1 个 Important（把 checker 能力可用误写成 canonical Failure 已回传）已修正；最终 current-hash review evidence 写入协调器，父 M2 和 M3 保持 pending。
