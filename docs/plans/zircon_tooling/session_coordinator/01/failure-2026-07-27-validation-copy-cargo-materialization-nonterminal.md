---
handoff_kind: failure
status: open
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

Open state: `待 Coordinator01 完成 cargo closure 后物化终态、恢复与可观测错误合同`。Editor12 保留 job
`f8726b18912e49d7a74dcb10051f3006`，不宣称 Cargo、review、fixed return 或提交已完成。

## 产出记录与时间

### 2026-07-27

- 状态：`open`；Coordinator01 最低共享层 handoff 已建立，来源 Editor12 继续保持 `resolving_failure`。
- 完成项目：记录了 closure 已持久化、external pin 已验证、但无 hash/无 typed terminal 的精确 job 证据与上层重放条件。
- 状态更新：job `f8726b18912e49d7a74dcb10051f3006` 已以 `validation_copy_unowned_path` 终态失败；`errorPath` 是
  Runtime15 已跟踪 plan-status source。无 Cargo、无 Rust 测试、无 review、无 fixed return；此记录将 Coordinator01
  修复范围收窄到 baseline 与 attributable overlay 的分类和可重放合同。

### 2026-07-29 00:xx CST

- 状态：`resolving_failure`，本 failure 保持 `open`；本条记录 Coordinator01 source repair 与 Python 验证，不将其写为 live reload、Editor12 Cargo、独立 review、fixed return 或 commit。
- 完成项目：`WorkspaceCopyService._require_untracked_overlay_attribution` 由逐路径 `git show` 改为每个 job 一次读取 pinned baseline tree；仅当 closure 含 baseline 缺失路径时再一次读取当前 tracked tree。当前已跟踪但不在 pinned baseline 的 foreign closure path 以 `validation_copy_baseline_drift` / `materialization_prepare` / 精确 `errorPath` 终态返回，并要求受管 replay；真正未跟踪路径仍为 `validation_copy_unowned_path`。因此不会读取新 HEAD 内容、要求来源 Session 归属 foreign 路径，或在 18k closure 上启动数万个 Git 子进程。
- TDD 与验证：先新增“worker 固定 baseline 后 closure 出现新 tracked path”的回归，旧实现如预期错误为 `validation_copy_unowned_path`；修复后该测试通过。`python -m unittest tools.session_coordinator.tests.test_workspace_copy` 的 39 项以 5 个单用例和 5 个独立批次完成，合计 `39/39`；一次性顺序执行超过客户端 184 秒上限，不能据此报为单次 suite terminal。`python -m compileall` 和 scoped `git diff --check` 通过。
- 独立复审：Coordinator01 baseline-drift repair 复审结果为 Critical/Important/Minor `0/0/0`；未发现错误分类、不可变输入或回归覆盖缺口。
- 后续：在独立 review 和受管 commit 后，使用受管 Coordinator reload 让 live service 载入该分类/批量 baseline 行为；随后对 Editor12 同一 command 与固定 `zr_vm` pin 创建新的 immutable copy，只有取得 `materialized + inputManifestHash` 或 typed terminal 后才可进行 Cargo run。现有 `416b041cd7524ae6a983f8801bf9bcfc` 不被清理、伪造为绿色或用共享工作树绕过。

### 2026-07-29 02:xx CST

- 状态：`resolving_failure`；受管 Cargo 队列曾清空，但 `tools/zircon-session.ps1 stop` 被控制面拒绝：`Global stop, restart, and force-stop are disabled while task admission is open`。该操作未终止或释放任何外部 job。
- 完成项目：将 live reload 作为本 failure 的受控生命周期依赖明确记录。修复源码、focused TDD、39/39 分批 Python 验证和独立 review `0/0/0` 仍有效；但旧进程尚未加载源码，所以不得宣称 replay、immutable source hash、Editor12 Rust gate、fixed return 或提交已完成。
- 后续：只能通过 Coordinator01 已授权的 drain/restart control-plane action 建立持久维护 hold，等待现有受管 Cargo 自然排空后重载；不得以直接停止、SQLite 写入、共享工作树 Cargo 或释放外部预留绕过 admission policy。本 Session 保持 `resolving_failure`，不标记为 `blocked`。
