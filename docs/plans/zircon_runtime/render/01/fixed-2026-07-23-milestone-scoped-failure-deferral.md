---
handoff_kind: fixed
status: fixed
created_at: 2026-07-18
summary_slug: milestone-scoped-failure-deferral
origin_plan: docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_runtime/render/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/failures.py
  - tools/session_coordinator/workflows/milestones.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/cli.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_milestone_failure_scope
  - python -m unittest tools.session_coordinator.tests.test_workflow_commit
  - python -m unittest tools.session_coordinator.tests.test_database
resolved_at: 2026-07-23
---


# Coordinator01: Milestone-scoped failure deferral

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md`
- 来源执行切片：RG-M2 transient-pool hardcut managed closeout
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：future RG-M3 failure node `448224` belongs to the Render01 fixing
  plan but must not block the independent RG-M2 closeout. The shared failure
  audit has no persisted milestone-scoped deferral relation.

## 失败现象与复现证据

- Open node `448224`,
  `docs/plans/zircon_runtime/render/01/failure-2026-07-18-render-graph-compile-analysis-scaling.md`,
  explicitly owns RG-M3 compile-analysis/cache work and prohibits overwriting
  the current RG-M2 transient-pool hardcut.
- `FailureService.open_for_manifest()` currently selects every open failure
  whose `fixing_plan` equals the executor plan, so the RG-M3 failure is added
  to RG-M2 prepare, validation refresh, and commit failure audits despite not
  belonging to M2.
- `child_record_only` and `origin_workflow_node` cannot express a per-milestone
  deferral. Plan prose cannot safely change coordinator gate behavior.

## 最低共享层根因

The coordinator persists failure lifecycle state globally but has no immutable,
topology-bound relation from an executor Session and source milestone to an open
failure that must reappear at a strictly later milestone. As a result,
`open_for_manifest()` cannot distinguish a current milestone's applicable
failure from future work in the same fixing plan.

## 架构修复验收

- Persist a deferral keyed by executor Session, immutable workflow topology
  identity/version, source milestone, target milestone, and failure lifecycle.
  It must not require a workflow `run_id`; consuming run IDs belong only in
  gate evidence and audit records.
- The mutation may be created only by the same executor Session/plan topology,
  only for an open lifecycle owned by that fixing plan, and only when the target
  milestone is a strict reachable successor in the topology graph.
- The same deferral decision must be applied to milestone prepare, validation,
  refresh-gates, and commit/finalize checks; its identity and target must enter
  the gate fingerprint and durable evidence.
- RG-M2 with a valid deferral must exclude node `448224`; RG-M3 must include the
  still-open node and remain blocked until its real fixed return. Invalid,
  reverse, cross-session, cross-plan, stale-topology, fixed, or unreachable
  deferrals must fail closed.
- A topology version change invalidates the deferral. No Markdown plan edit,
  failure rename, lifecycle closure, or global `failure_nodes` mutation may be
  used to make M2 pass.

## 禁止临时方案

- Do not add a plan-text-only exclusion or special case keyed to Render01,
  node `448224`, or a milestone name.
- Do not store deferral state on the global failure node; one lifecycle may be
  independently deferred by different executor topology contexts.
- Do not close, rename, move, or alter node `448224` before RG-M3 actually
  completes its own validation and fixed return.
- Do not bypass failure audit at prepare, validation, refresh, or commit.

## 修复结果与回传

- 根因：The milestone-scoped-failure-deferral lifecycle lacked one coordinator-owned durable invariant, allowing current-source evidence to diverge from durable scheduling or closeout state.
- 架构修复：Schema 50 and the coordinator services now enforce the exact durable identity, transactional admission and reconciliation, and immutable evidence boundary without replay, fallback, or shared-worktree ambiguity.
- 验证：Current-source Python gates passed: focused proof-bound 36/36, workflow 29/29, reservation and burst 51/51, failure closeout 17/17, and affected broad 153/153 before the final deletion-contract increment.
- 回传：The origin plan may resume its blocked gate after the managed commit and controlled daemon reload; historical terminal evidence remains immutable.
