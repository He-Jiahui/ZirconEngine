---
handoff_kind: failure
status: open
created_at: 2026-09-01
last_reproduced_at: 2026-09-01
last_reproduced_head: 9963f8eb72e2d725d2536eb50b393b30387a1ffa
summary_slug: validation-copy-pinned-cargo-metadata-global-failure
origin_plan: docs/plans/designment/02-milestone-execution-and-evidence.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/designment/02
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: cross_plan
related_code:
tests:
  - python -B -m unittest tools.session_coordinator.tests.test_pinned_cargo_planner -v
  - .\tools\dev\zircon-session.ps1 --json validation-copy materialize-cargo --session-id root-designment02-penpot-zui-roundtrip-20260831 --path zircon_runtime/src/dynamic_api/session/tests/runtime_ui_surface.rs --path zircon_runtime/tests/fixtures/ui/penpot_roundtrip.zui -- cargo test -p zircon_runtime --locked --lib project_runtime_ui_bootstraps_penpot_roundtrip_asset
  - .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -LibTests -TestFilter project_runtime_ui_bootstraps_penpot_roundtrip_asset
---

# Coordinator01: pinned Cargo metadata 全局阻断 validation copy

## 来源执行者

- 来源计划：`docs/plans/designment/02-milestone-execution-and-evidence.md`
- 来源执行切片：A2-C Penpot `.zui` project Runtime bootstrap managed validation
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 拥有 validation-copy Cargo closure planner、pinned view、错误持久化和 materialization worker；designment/runtime 调用方只能提交 overlay 与托管 Cargo 命令，不能在产品计划中绕过闭包规划。

## 失败现象与复现证据

在基线提交 `5798051603e7f7f565538125c9aba96d5beabae2` 上，Cargo validation copy 在创建源码副本前统一失败。2026-09-01 的只读数据库聚合显示，三个独立 Session 的 15 个 `materialization_kind=cargo` job 均以 `pinned_cargo_metadata_failed`、`error_stage=closure_planning`、`exitCode=101` 终止；没有一个进入 materialized 状态。

A2-C 的代表性 job `b6e0f1392f9748efbf6ace338ad01143` 使用命令 `cargo test -p zircon_runtime --locked --lib project_runtime_ui_bootstraps_penpot_roundtrip_asset`，overlay 仅包含已由来源 Session 精确归属的 Runtime test 与 `.zui` fixture。该 job 在闭包规划阶段失败并已由协调器清理。相同 HEAD 上 `root-runtime-editor-optimize-20260901-r6` 与 `01a019a3-02af-7012-84cd-f8d605f12c4d-r1` 的不同 package/overlay 也得到相同终态，因此不是 A2 fixture 或单一 package 的特例。

固定 HEAD 中 `zircon_editor/Cargo.toml` 与 `Cargo.lock` 一致；live worktree 的 Text04 依赖删除/lockfile 漂移是另一条独立阻塞，不能解释 clean pinned view 的这组失败。持久化 `error_details_json` 仅保留 `{"exitCode": 101}`，虽然 planner 内部异常原本携带 Cargo stderr，来源 Session 无法从终态记录继续缩小 Cargo 拒绝原因。

2026-09-01 的后续源码审计已定位 durable diagnostics 丢失的确定原因：`tools/session_coordinator/pinned_cargo_planner.py::_run_cargo_metadata` 在非零退出时构造 `details={"exitCode": ..., "stderr": result.stderr[-4096:]}`，但 `tools/session_coordinator/workspace_copy.py::_materialization_error_details` 的白名单保留 `exitCode` 而不保留 `stderr`。因此当前数据库终态中的“只有 exit code”不是 Cargo 没有产生 stderr 的证据，而是 materialization failure persistence 主动丢弃了该字段。真正导致 Cargo 退出 101 的原因仍需在修复 bounded/sanitized stderr 持久化后由一次 managed clean-pinned 复现确定。

在 Coordinator01 后续合入 validation-copy closeout dedupe 与 cleanup evidence 修复后，来源 Session 又于提交时 `main` HEAD `9963f8eb72e2d725d2536eb50b393b30387a1ffa` 执行同一精确 `materialize-cargo` 命令。新 job `5ff634d72d67424faa2b7f0370976d5a` 被接受并进入 `closure_planning`，随后仍以 `pinned_cargo_metadata_failed`、`exitCode=101` 终止；`errorDetails` 仍只有 exit code，`errorPath` 与 `terminalEvidence` 均为空。该 job 已经由协调器 cleanup，未遗留 validation-copy 目录。此复现说明 cleanup/dedupe 修复没有关闭 pinned metadata 根因或诊断缺口，不应继续盲目重复提交。

## 最低共享层根因

已证明的最低执行边界是 `PinnedCargoInputClosurePlanner` 调用 `_run_cargo_metadata` 后、validation-copy materialization 前：clean pinned baseline 上的 metadata subprocess 对多组合法 `--locked` 命令统一退出 101。已证明的诊断根因是 `_materialization_error_details` 丢弃 `_run_cargo_metadata` 提供的 `stderr`；尚未证明的 Cargo 101 根因仍可能位于 metadata argv、隔离 Cargo home/config 或 baseline 投影。最终执行根因必须由 Coordinator01 在恢复 durable diagnostics 后于同一边界内确定。

## 架构修复验收

- 增加真实 subprocess contract 覆盖，证明从 build/test/check 命令派生的 metadata argv 能在 isolated pinned view 上以 `--locked` 成功解析当前工作区；mocked `CompletedProcess(returncode=0)` 不能单独作为验收。
- 非零 metadata 终态必须把 `_run_cargo_metadata` 捕获的 bounded stderr（或等价的安全脱敏 tail 字段）纳入 `_materialization_error_details` 的 durable schema，并让 validation-copy status、数据库记录和 worker 日志指向同一失败原因；不得只留下 exit code。增加持久化回归测试，直接断言非零 metadata 的诊断 tail 能从终态 status 读回。
- 在同一 immutable HEAD 上重新执行上面的 A2 `materialize-cargo` 命令，job 必须进入 `materialized`，随后在该副本中运行原 focused Cargo test。
- 修复回传后，A2-C 仍需经 managed `validate-matrix.ps1` 重跑 project Runtime lib test 与 `zui_penpot_bridge_contract` integration target；只有两者均为 GREEN 才可把 A2-C 提升为 validated。

## 禁止临时方案

- 不得移除 `--locked`/`--frozen`、使用 live worktree 代替 pinned view、手工复制不完整 workspace，或用未受管 Cargo 冒充 validation-copy 证据。
- 不得把 metadata 失败归咎于 A2 fixture 后跳过 closure planning，也不得把 mock-only planner unit test 当成真实 subprocess GREEN。
- 不得删除或改写既有终态 job 来掩盖 15 次跨 Session 复现。

## 修复结果与回传

待修复；最新 HEAD `9963f8eb72e2d725d2536eb50b393b30387a1ffa` 仍可复现，当前没有 Cargo GREEN 声明。A2-C 可继续保留 source-ready 证据，但必须等待本交接返回以及 Text04 恢复 live `Cargo.lock` 基线后再执行最终托管验证。

## 2026-09-11 rolling repair closure completion

The stable Coordinator01 fixing Session preserved the original failure evidence
and repaired only the immutable Python import closure that prevented the
focused regression from starting. Ticket
`95c89b976bf64ff0ba9067de1661d586` had stopped at module import with
`ModuleNotFoundError` for `tools.session_coordinator.pinned_metadata_cache`;
that exit 1 was a validation-copy materialization failure, not a metadata
subprocess result.

The exact current `pinned_metadata_cache.py`, its focused test, and the
planner's `cargo_storage.py` dependency were transferred through coordinator
ownership review into this fixing Session. The focused planner test no longer
imports unrelated WorkspaceCopy services before its real metadata subprocess;
WorkspaceCopy coverage remains in its own direct consumer tests. No Cargo
metadata command, lockfile setting, live-worktree fallback, or diagnostic
policy was relaxed.

Fresh immutable Windows validation ticket
`b393e37f855f4dc383a9d4d53ab50169` (copy/job
`7d7a231ba4fa43c0b366256ab48485b9`) used the complete seven-path planner/cache
closure with source-manifest hash
`a837bf54deda09f286944c092a0813edfd93b91db518ad4d3246b52a86cd7498`.
It reached and passed the real pinned-sibling metadata subprocess regression
plus all ten pinned metadata-cache regressions: `11 passed`, exit `0`.
This is lower-layer dynamic evidence that the sealed metadata route can run;
it does not convert a Python test pass into A2-C product validation.

The exact original A2 `materialize-cargo` reproduction and the subsequent
Runtime project / `zui_penpot_bridge_contract` managed gates remain pending.
Those gates must still execute against their own source snapshots before this
lifecycle can be returned, independently reviewed, or closed.

## 2026-09-13 rolling repair: durable metadata diagnostic regression

The current planner source already preserves a bounded, sanitized metadata
stderr tail in `_materialization_error_details`.  To prove that the producer
contract is exercised by a real child process, the fixing Session added
`PinnedCargoPlannerTests.test_metadata_failure_captures_bounded_stderr_from_real_subprocess`.
The test binds a Python child as the trusted Cargo executable, makes it emit a
diagnostic longer than the persistence bound, exits with status 101, and
asserts the typed `pinned_cargo_metadata_failed` error contains an exit code
and a bounded tail ending in the actionable diagnostic.

Managed immutable Windows ticket `526410e693384d68a9e64f80f3b6345b` (copy
`e93c408e9f12460790c0bf6241b77ff4`, run
`526410e693384d68a9e64f80f3b6345b`) sealed source-manifest hash
`9592fe86db2f5180cf4329ec9807f909c35bf1290793654bdbfce70bf985d861` and
executed the real success and failure subprocess contracts plus the ten
metadata-cache regressions.  It completed with exit code 0 and `12 passed`
on Windows Python 3.14; the coordinator receipt records the full test output
and terminal status `passed`.

This receipt closes only the durable-diagnostics regression gap.  The A2
`materialize-cargo` job, focused Cargo test, and the two managed Runtime gates
listed above have not yet run on the matching immutable source snapshot, so
this lifecycle remains `open` and must not be returned or closed yet.
