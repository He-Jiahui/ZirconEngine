---
handoff_kind: failure
status: open
created_at: 2026-07-19
summary_slug: viewport-shared-extract-arc-slice-iteration-compile-regression
origin_plan: docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
fixing_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
origin_child_dir: docs/plans/zircon_editor/editor/15
fixing_child_dir: docs/plans/zircon_editor/editor/05
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/scene/viewport/pointer/candidates/precision_candidates_from_layout.rs
tests:
  - cargo test -p zircon_editor --lib viewport --locked --jobs 1 --color never -- --test-threads=1
---

# Editor05：共享 viewport extract 的 Arc slice 迭代编译回归

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/15-build-export-and-publishing.md`
- 来源执行者：`editor15-export-report-parse-once-r7-20260718`
- 来源执行切片：受管 focused gate `f5cd31cd719042ce88cb133cde113cef` / run `b510c09846f94d5aaf63e43985a31d9a`
- 修复责任计划：`docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md`
- 交接原因：共享 viewport interaction extract 与 pointer candidate 迭代归 Editor05，Editor15 不应修改或吸收该选择/命中边界。

## 失败现象与复现证据

Editor15 的受管 focused gate `f5cd31cd719042ce88cb133cde113cef` / run `b510c09846f94d5aaf63e43985a31d9a` 在编译 `zircon_editor --lib` 时，于 `precision_candidates_from_layout.rs:25/34/43` 产生 3 条 E0277。`layout.handles`、`layout.scene_gizmos` 与 `layout.renderables` 已硬切为 `Arc<[T]>`，当前代码仍使用 `for value in &layout.field`；`&Arc<[T]>` 不实现 `IntoIterator`，因此测试尚未执行。

原始 stderr：`.codex/state/session-coordinator/cargo-runs/f5cd31cd719042ce88cb133cde113cef/b510c09846f94d5aaf63e43985a31d9a/stderr.log`。该作业同时发生外部 Runtime 源竞态，不能作为 Editor15 验收，但这 3 条 rustc 诊断来自 Editor05 当前文件并具有精确路径/行号。

## 最低共享层根因

共享 extract 的容器已经从 owned `Vec<T>` 硬切为 immutable `Arc<[T]>`，pointer candidate consumer 未同步迁移 slice 迭代合同，仍按旧容器引用语法消费。

## 架构修复验收

- 由 Editor05 在共享 interaction extract 边界改为 slice 迭代（如 `.iter()` / `.as_ref()`），不得 clone `Arc` 内容或重新生成候选数组。
- 先运行 viewport focused test，再运行新鲜、无源竞态的 `zircon_editor --lib` 门禁；记录 job/run/raw log 与测试计数。
- 复核 handle、gizmo、renderable 的既有优先级与命中行为；本 compile fix 不关闭既有空间 broad-phase / p95 failure。

## 禁止临时方案

- 不得 clone `Arc<[T]>` 为临时 `Vec<T>` 或逐事件复制候选数据。
- 不得回退共享 extract 的 `Arc` 所有权合同。
- 不得把 source-raced Editor15 作业登记为 Editor05 验收 GREEN。

## 修复结果与回传

Open state: `Editor05 slice 迭代源码修复与静态门已完成；受管 current-source Cargo、独立复审、fixed return 与 owner commit 待完成。`

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 验证与未完成项 |
|---|---|---|---|
| 2026-07-19 01:19 +08:00 | `open / Editor05 待修复` | 已把 3 条 `Arc<[T]>` 直接迭代 E0277 从 Editor15 诊断门禁回传到 Editor05；未修改 viewport 业务代码。 | 受管作业 exit 101、tests 0；待 Editor05 完成 slice 迭代修复、focused/broad current-source 验证和独立复审。 |
| 2026-07-19 02:17 +08:00 | `源码修复完成 / 静态门通过` | `precision_candidates_from_layout` 的 handle、scene gizmo、renderable 三个共享 `Arc<[T]>` consumer 已硬切为 `.iter()`；未 clone slice、未恢复 `Vec<T>` 双轨，候选追加顺序保持不变。当前精确源码由 snapshot `559` 固定，`rustfmt +1.94.1 --edition 2024 --config skip_children=true`、`git diff --check` 与静态 consumer 扫描均通过。 | 本记录不把 source-raced Editor15 作业解释为验收；Coordinator01 immutable validation-copy 终态证据 failure 尚未 fixed，故 focused/broad Cargo、独立复审、failure return 与受管提交仍为 open。 |
| 2026-07-30 22:11 +08:00 | `validation-copy admission 未完成` | Editor05 fixing session `editor05-arc-slice-iteration-return-r1-20260730` 已领有 consumer 与本 handoff 的精确租约；`rustfmt --check`、`git diff --check` 与旧 `&Arc<[T]>` 直接迭代扫描均通过。immutable copy `5929fa078870417f91aa6064f2a59464` 使用原 focused command 被协调器接收。 | copy 在 Cargo 进程启动前的 `closure_planning` 失败，`errorCode=validation_copy_external_source_missing`；未创建 Cargo run/PID/测试计数，不能作为 Rust 验收。后续必须提供受管 external-source pin 后创建新的 copy；本 failure 保持 `open`。 |
| 2026-07-30 22:15 +08:00 | `validation-copy attribution 未完成` | 使用固定 `E:/Git/zr_vm@d06c8cd2e70eddd5b31ee1cca46066183f1ef7ed`、`mountPath=zr_vm` 及双 crate include root 创建新副本 `f9509361dde2417cbe395f3df2b86094`。 | copy 在 Cargo 前的 `overlay_ownership` 失败，`errorCode=validation_copy_overlay_not_owned`、`errorPath=precision_candidates_from_layout.rs`；有效 lease 未能接管旧 baseline attribution。该跨会话 union-attribution 根因归现有 Coordinator01 `live-lease-attribution-validation-copy-divergence` handoff；无 source root/Cargo/PID/测试计数，本 failure 继续 `open`。 |
| 2026-07-31 23:xx +08:00 | `independent source review 0/0/0` | 审查确认三处 `.iter()` 正确消费 `Arc<[T]>`，handles/scene gizmos/renderables 的稳定前向候选顺序不变；每次 pipeline 只构建一个短生命周期 projection context，预分配上界不低估、无 Arc clone 或 `Vec` 双轨回退。 | 此为源码审查，不替代受管 current-source Cargo；既有 attribution failure 未解决，artifact 保持 `open`。 |
