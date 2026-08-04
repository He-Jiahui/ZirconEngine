---
handoff_kind: fixed
status: fixed
created_at: 2026-07-27
summary_slug: validation-copy-cargo-materialization-nonterminal
origin_plan: docs/plans/zircon_editor/editor/12-plugin-management.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_editor/editor/12
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/workspace_copy.py
  - tools/session_coordinator/workspace_copy_terminal.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/tests/test_workspace_copy.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_workspace_copy
  - validation_copy.materialize_cargo persists a terminal materialized or failed state after closure persistence
  - Editor12 pinned external-source cargo-copy and catalog_store test rerun
resolved_at: 2026-08-04
---


# Coordinator01: validation-copy Cargo 物化非终态

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/12-plugin-management.md`
- 来源执行切片：Editor12 immutable catalog snapshot 的 source-bound Rust 验收
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Cargo 输入闭包、异步物化 worker、输入 hash 与终态持久化均由 Coordinator01 `WorkspaceCopyService` 所有；Editor12 只能保留受管 job 与精确输入，不能复制工作区、直接写数据库或在共享树运行 Cargo。

## 失败现象与复现证据

Editor12 union Session `editor12-plugin-catalog-unified-validation-r1-20260727` 的 cargo-copy
`f8726b18912e49d7a74dcb10051f3006` 已按受管 API 接受。它固定四个 editor overlay：
`zircon_editor/build.rs`、`core/plugin/catalog_gen.rs`、`catalog_snapshot.rs`、`catalog_store.rs`，并使用
`cargo test -p zircon_editor --lib catalog_store --locked --jobs 1 --color never -- --test-threads=1`。

首次 `materialize-cargo` 因未声明 sibling `zr_vm` 输入以
`validation_copy_external_source_missing` 终态，未启动 Cargo。重试明确传入
`E:\Git\zr_vm` commit `503fb72163cd20ddf32a38f8a330083712f5d648`、`mountPath: zr_vm`，及
`zr_vm_rust_binding/rust/zr_vm_rust_binding`、`zr_vm_rust_binding/rust/zr_vm_rust_binding_sys` 两个
include roots；服务端确认 external source hash 为
`f4d20ea8a8ebcec4a7bf89ac293208c2479d2b35a7ab380407be0af3e0fc17f6`。

随后 closure 已持久化 18,032 个 repository paths 和一个 external source，但连续受管 `validation-copy status`
均返回 `status: materializing`、`materializationPhase: materializing`、`inputManifestHash: null`、无
`errorCode/errorStage`。没有 validation-copy run、Cargo 进程或 Rust 测试结果。Coordinator 进程仍健康，
物化 worker 有 CPU 活动，因此该证据要求状态机给出确定的成功或失败终态，而不是由来源计划猜测或清理 job。

终态更新：该 job 最终转为 `status: failed`、`materializationPhase: failed`，但仍没有 Cargo run、
source root 或 input manifest hash。它报告 `validation_copy_unowned_path` / `materialization_prepare`，精确
`errorPath` 是 `zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps/code_review_guard_maps/folder_backed_summary_rows.rs`。
该文件是当前仓库已跟踪的 Runtime15 plan-status source，而非 Editor12 四个 overlay 或 `zr_vm` source；它不在
该 validation job 创建时的 Git baseline，且不能由 Editor12 Session 归属。因此这个 typed terminal 不能作为
Editor12 Rust 验收，也不能要求来源计划为外部 Runtime15 文件补 attribution。

## 最低共享层根因

已证明的最窄边界是：`WorkspaceCopyService._persist_cargo_closure()` 已将 closure 与 phase 写为
`materializing`，随后才在长时间后暴露 typed terminal。对于包含创建后加入当前 Git baseline 的已跟踪 closure
文件，`_require_untracked_overlay_attribution()` 将其按无主 overlay 拒绝，而来源 Session 既不能也不应归属
另一计划的 Runtime15 source。Coordinator01 需要把 job baseline 中的 committed closure content 与当前 Session
的 attributable overlay 明确区分，并为这类基线漂移给出可重放的受管行为和精确错误归因。根因不在 Editor12
catalog 源码，也不是未固定 external source。

## 架构修复验收

- Cargo closure 持久化后，worker 必须在同一 job 上原子写入 `materialized + inputManifestHash`，或写入带
  `errorCode/errorStage/errorPath` 的 `failed` 终态；不得无限保留无 hash、无错误的 `materializing` 行。
- 对创建后出现的已跟踪 foreign closure path，必须从受管 Git baseline 取 committed content，或返回明确的
  baseline-drift terminal/replay contract；不得把它伪装为要求来源 Session attribution 的 untracked overlay。
- 服务重启、worker 异常与 archive/I/O 失败必须恢复或终结同一 job id，且 status 暴露当前受管阶段，不能要求
  来源计划清理后盲目重建。
- 增加 focused Coordinator01 回归，覆盖大 Cargo closure、pinned sibling Git archive、四个 attributed
  untracked overlay 与终态 hash/typed failure；不得把成功限制为小型纯 tracked fixture。
- 使用同一 Editor12 command、同一 zr_vm pin 与当前 four-path overlay 重放 `materialize-cargo`，获得 immutable
  source copy 后才允许 `validation-copy run`；随后重跑 catalog_store Rust 测试和上层 Editor12 acceptance。

## 禁止临时方案

- 不得以共享工作区 Cargo、手工复制 `Cargo.toml`、修改 source root、直接写 SQLite 或清理后重建来绕过该 job。
- 不得将 `materializing` 误报为 Cargo 已启动或 Rust GREEN，也不得丢弃 external-source commit/hash 证据。
- 不得通过延长调用方轮询、降低闭包规模、移除 zr_vm pin 或排除真实 workspace package 来掩盖非终态。

## 修复结果与回传

- 根因：After a Cargo closure was persisted, a tracked foreign path added after the job's pinned Git baseline was classified as an unattributed untracked overlay. The worker could remain materializing for a long interval and eventually returned validation_copy_unowned_path, even though the origin Session could not own that foreign tracked source.
- 架构修复：The committed worker reads each job's pinned baseline tree once and consults the current tracked tree only for closure paths missing from that baseline. Such foreign tracked additions terminalize the same durable job as validation_copy_baseline_drift with exact stage and path; true untracked paths remain ownership errors, malformed requests and restart recovery terminalize exactly once, and successful jobs persist materialized plus inputManifestHash.
- 验证：Four exact local terminalization regressions passed in 5.841s. Managed ticket c7dd772dc0354e67a55134d4f77e8317, source manifest 68becd7dc156702fcf2bdaa669b62bb82c06c5495563ef73a80f5037bc604355, copy job 7675f897fb854d2aa25781f687be2d71 passed 4/4 in 5.913s with exit code 0. The ticket uses committed terminalization code and excludes a later terminal-output evidence change. Handoff graph before return validated 561 artifacts with 0 errors.
- 回传：The original Editor12 Session is archived. Preserved jobs f8726b18912e49d7a74dcb10051f3006 and 416b041cd7524ae6a983f8801bf9bcfc remain typed failed with no Cargo run and the historical Runtime15 path evidence; neither was retried or cleaned. A new current-source Editor12 Session must perform any catalog_store product validation.
