---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: script-build-facade-validation-copy-closure
origin_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
fixing_plan: docs/plans/zircon_editor/editor/13-script-compilation-management.md
origin_child_dir: docs/plans/zircon_editor/editor/02
fixing_child_dir: docs/plans/zircon_editor/editor/13
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/mod.rs
  - zircon_editor/src/core/script_build/mod.rs
  - zircon_editor/src/core/script_build/orchestrator.rs
  - zircon_editor/src/core/script_build/tests.rs
tests:
  - cargo test -p zircon_editor --test editor_world_sync_watch_map --locked --jobs 1 -- --nocapture --test-threads=1
---

# Editor13：script-build facade 验证副本闭包缺失

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 来源执行切片：Editor02 M2.1 world-sync watch-map exact8
- 修复责任计划：`docs/plans/zircon_editor/editor/13-script-compilation-management.md`
- 交接原因：Editor13 的 public facade 与 `core/script_build/**` 必须作为一个自包含编译闭包归因，Editor02 不得吸收或删除外部业务模块。

## 失败现象与复现证据

Editor02 snapshot 753 将当前 `zircon_editor/src/core/mod.rs` 覆盖进冻结副本；该文件同时含有 Editor13 的 `pub mod script_build;` 与 Editor02 的 `pub mod sync;`。冻结副本 `D:\cargo-targets\verify\f30b3f7f0e574e92bcfa5e1b260fe5dc\source` 中存在前者，但基线不含 `zircon_editor/src/core/script_build/mod.rs`，因此受管 Cargo replay 会确定触发 E0583。

该问题不是 watch-map 算法错误。Editor02 不得删除外部 facade 行、把 `script_build/**` 纳入自己的 manifest，或把未提交的 Editor13 当前树当作隐式编译输入。

## 最低共享层根因

validation-copy 冻结了声明 `pub mod script_build` 的共享 facade，却没有冻结 Editor13 对应模块闭包；跨计划 source attribution 不是自包含的。

## 架构修复验收

- Editor13 以自己的 exact manifest 完成 `core/mod.rs` 与 `core/script_build/**` 的受管验证、独立审查和 milestone commit。
- Editor02 只在该 SHA 成为 validation-copy 基线后重建 watch-map 副本、snapshot 与 reservation。
- 新副本的编译闭包自包含；Editor02 exact manifest 不吸收 Editor13 业务文件。

## 禁止临时方案

禁止用删除 facade、test-only stub、兼容空模块或共享工作树旁路闭包检查。

## 修复结果与回传

Open state: `Editor13 exact source/static review 已收敛，但 validation-copy 本地 manifest 闭包、受管 Cargo、milestone SHA 与 Editor02 successor replay 尚未完成`。

## 产出记录与时间

| 里程碑 | 状态 | 完成日期 | 完成项目与证据 |
|---|---|---|---|
| Editor13 M1 / Editor02 M2.1 dependency handoff | `open / waiting_editor13_managed_sha` | 2026-07-22 | 独立审查在 Editor02 snapshot 753 / validation copy `f30b3f7f0e574e92bcfa5e1b260fe5dc` 复核：`core/mod.rs` 有 `pub mod script_build;`，副本无 `core/script_build/mod.rs`，故 exact8 replay 必然 E0583。修复归 Editor13；Editor02 保持冻结，不吸收或删除该 facade。 |
| Editor13 M1 current-hash closure recheck | `open / current_attribution_drift` | 2026-07-22 | Editor02 已将 watch-map exact8 重冻结为 baseline221 snapshot906 并恢复自身 8/8 attribution。Editor13 `script_build/mod.rs` 当前 SHA256 `3ba8837741b0b994848e00ae0ffeb0d18b6f2fbf040235356df7c671b3e01843`、`request.rs` 当前 `a9b23f6099b3bfaeab031020881b0047f7f230ece33ca623d028206e9934f4c2` 与历史 attribution 一致；但 `orchestrator.rs` 当前 `519aff7b483f5fe48a62a72cb1932a9e28dee209eb431f0a0e2e77509b197221` 不等于归因 `3739dc71...`，`tests.rs` 当前 `971614e2a9dd727fedc4b784d14bda0b7bb0c1c6a4cd6b68be66b34e75b25a43` 不等于归因 `db93bbea...`。validation copy 必须继续拒绝陈旧外部归因；Editor13 owner 需冻结 current exact manifest、复审/验证并重新 attribute/commit，Editor02 不越权吸收。 |
| Editor13 M1 exact8 current-source convergence | `open / source_review_clean_cargo_blocked` | 2026-07-22 | Editor13 session `editor13-script-build-orchestrator-current-source-r2-20260722` 已完成线性 dispatch ticket、typed request-id exhaustion 与 20/21 边界/Play flush 覆盖；静态契约 `5/5`、exact rustfmt、diff-check 均通过，独立复审 Critical/Important/Minor=`0/0/0`（既有 continuous-watch starvation / Command、Play 无界队列性能 failure 不在本次 facade 闭包范围）。baseline221 snapshot `911` 的 exact8 预览 `8/8` 无漂移并已重新归因：`core/mod.rs`=`9f5c28fe...`、`script_build/mod.rs`=`bbb3ab10...`、`orchestrator.rs`=`4b4bd91e...`、`request.rs`=`8469faf3...`、`tests.rs`=`648d80eb...`。但 Coordinator01 validation-copy manifest graph 未包含 repo-local `zircon_plugins/first_party_editor_catalog/Cargo.toml`，Editor02 replay job `70e97f82a07b4456b7d1a1fb45ed830b` 已在 Rust 编译前 exit101；因此本 failure 继续保持 open，等待 Coordinator01 修复完整 Cargo local-path 闭包后再执行 Editor13 受管 Cargo、milestone commit 与 Editor02 successor replay，禁止把静态通过写成 managed SHA 完成。 |
