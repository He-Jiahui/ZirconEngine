---
handoff_kind: failure
status: open
created_at: 2026-08-01
summary_slug: failure-dependency-graph-cycles
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/session_coordinator/failure_dependency_graph.py
  - tools/session_coordinator/failures.py
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/control_plane/snapshot.py
  - tools/session_coordinator/tests/test_failures.py
  - tools/session_coordinator/tests/test_database.py
  - tools/session_coordinator/tests/test_control_snapshot.py
tests:
  - .\tools\zircon-session.ps1 failure audit
  - python -m unittest tools.session_coordinator.tests.test_failures -v
  - python -m unittest tools.session_coordinator.tests.test_database -v
  - python -m unittest tools.session_coordinator.tests.test_control_snapshot.ControlSnapshotTests.test_failure_diagnostics_project_structured_details -v
---

# Coordinator01: failure dependency graph contains ownership cycles

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：2026-08-01 plan, failure and session consistency review
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns failure import and dependency-graph diagnostics. Individual plans own the meaning of each handoff edge, but the coordinator must keep one canonical graph-repair inventory and prevent cyclic ownership from being accepted as a valid execution order.

## 失败现象与复现证据

After the Markdown handoff validator reached `547 artifacts / 0 errors`, `failure audit` reported 27 graph diagnostics: 14 dependency cycles, seven excessive-depth results and six invalid origin workflow nodes. The six stale workflow-node fields were removed directly because their values do not identify current plan nodes. A later current-source and managed-evidence review returned the proven-fixed Plugins01 -> Text01 font-fallback edge, removing two cycles without changing factual ownership. The 2026-08-02 import now contains `549 artifacts / 0 handoff errors` and 19 graph diagnostics: 12 dependency cycles plus seven depth results derived from those cycles.

The 2026-08-03 current ledger contains 13 cycle diagnostics plus eight derived depth diagnostics. They collapse into two strongly connected components: a two-plan/two-edge Performance01-Runtime12 component and a 22-plan/50-edge editor/runtime/plugin component. Changing an `origin_plan` or `fixing_plan` only to silence the graph would corrupt the actual ownership record.

## 最低共享层根因

The repository has accumulated valid-looking pairwise handoffs without enforcing a global acyclic fixing order. Several plans can therefore be both an upstream origin and a downstream fixing owner through different failures. Markdown schema, placement and backlinks can all be correct while the aggregate dependency graph has no topological execution order.

## 架构修复验收

- Materialize every current cyclic edge as a stable inventory grouped by strongly connected component, including the exact failure artifacts that contribute each edge.
- For each cycle, identify the lowest shared architecture owner and consolidate or reverse only the handoff edges whose recorded ownership is factually wrong.
- Where both directions are genuinely required, replace the pair with one shared lower-layer fixing plan and keep the other plan as a consumer, not a reciprocal fixer.
- Re-import failures and require `failure audit` to report zero `cycle`, `excessive_depth` and `invalid_origin_workflow_node` diagnostics.
- Re-run the Markdown handoff validator and plan-output audit so graph repair cannot break artifact schema, placement, backlinks or archive limits.

## 禁止临时方案

- Do not delete open failures, change status to fixed, or rewrite origin/fixing plans solely to make the graph acyclic.
- Do not raise the maximum depth, suppress cycle diagnostics or add an allowlist for current paths.
- Do not merge unrelated product failures into a coordinator-owned implementation failure; Coordinator01 owns the graph repair process, not the product code.
- Do not claim derived excessive-depth diagnostics are independently fixed until the cycles that create them are removed.

## 当前强连通边清单（2026-08-03）

只读 ledger 审计：`561` nodes，`13 cycle + 8 excessive_depth` diagnostics。下列清单按计划路径和 artifact 路径 ordinal 排序；它记录事实，不改变 ownership。

### SCC 1: 2 plans / 2 edges

- `docs/plans/performance/01-mvp-performance-audit-and-optimization.md` -> `docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md`
  - `docs/plans/zircon_runtime/runtime/12/failure-2026-07-19-app-entry-input-and-gamepad-storm-budget.md`
- `docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md` -> `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
  - `docs/plans/performance/01/failure-2026-07-18-runtime-picking-indexed-hit-type-inference.md`

### SCC 2: 22 plans / 50 edges

- `docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md` -> `docs/plans/zircon_editor/editor/04-pie-and-simulation.md`
  - `docs/plans/zircon_editor/editor/04/failure-2026-07-29-authoring-runtime-gateway-ownership-conflation.md`
- `docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md` -> `docs/plans/zircon_editor/editor/09-editor-asset-management.md`
  - `docs/plans/zircon_editor/editor/09/failure-2026-07-29-editor-asset-catalog-project-close-deactivation-missing.md`
- `docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md` -> `docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
  - `docs/plans/zircon_runtime/frameworks/05/failure-2026-07-29-project-asset-manager-close-contract-missing.md`
- `docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md` -> `docs/plans/zircon_runtime/render/17-performance-and-profiling.md`
  - `docs/plans/zircon_runtime/render/17/failure-2026-07-29-render-graph-profile-metrics-root-export-drift.md`
- `docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md` -> `docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md`
  - `docs/plans/zircon_runtime/runtime/10/failure-2026-07-17-editor-selection-state-runtime-session-boundary.md`
- `docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md` -> `docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md`
  - `docs/plans/zircon_editor/editor/14/failure-2026-07-22-message-subscriber-result-consumer-drift.md`
- `docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md` -> `docs/plans/zircon_plugins/01-plugin-architecture-core.md`
  - `docs/plans/zircon_plugins/01/failure-2026-07-22-plugin-event-drain-frame-budget.md`
- `docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md` -> `docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md`
  - `docs/plans/zircon_editor/editor/16/failure-2026-07-18-editor-state-context-constructor-hardcut.md`
- `docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md` -> `docs/plans/zircon_plugins/05-navigation.md`
  - `docs/plans/zircon_plugins/05/failure-2026-07-15-navigation-bake-selection-operation-arguments.md`
- `docs/plans/zircon_editor/editor/04-pie-and-simulation.md` -> `docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
  - `docs/plans/zircon_editor/editor/03/failure-2026-07-23-pending-edit-retention-contract-missing.md`
  - `docs/plans/zircon_editor/editor/03/failure-2026-07-30-fallible-exclusive-transition-context-update.md`
- `docs/plans/zircon_editor/editor/04-pie-and-simulation.md` -> `docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md`
  - `docs/plans/zircon_editor/editor/16/failure-2026-07-18-runtime-preview-play-scene-report-args.md`
- `docs/plans/zircon_editor/editor/04-pie-and-simulation.md` -> `docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md`
  - `docs/plans/zircon_editor/editor/17/failure-2026-07-22-play-pending-edit-decision-notification-contract.md`
- `docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md` -> `docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
  - `docs/plans/zircon_editor/editor/01/failure-2026-07-31-highlight-set-gateway-contract.md`
- `docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md` -> `docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
  - `docs/plans/zircon_editor/editor/02/failure-2026-08-01-plugin-registration-runtime-consumer-atomicity.md`
- `docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md` -> `docs/plans/zircon_plugins/05-navigation.md`
  - `docs/plans/zircon_plugins/05/failure-2026-07-30-navigation-overlay-frame-publication.md`
- `docs/plans/zircon_editor/editor/06-ui-extension-framework.md` -> `docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md`
  - `docs/plans/zircon_editor/editor/08/failure-2026-08-01-ticketed-command-routing-revoke-missing.md`
- `docs/plans/zircon_editor/editor/06-ui-extension-framework.md` -> `docs/plans/zircon_editor/editor/12-plugin-management.md`
  - `docs/plans/zircon_editor/editor/12/failure-2026-08-01-plugin-manager-inspector-customization-guard-drift.md`
- `docs/plans/zircon_editor/editor/06-ui-extension-framework.md` -> `docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md`
  - `docs/plans/zircon_editor/editor/14/failure-2026-08-01-interactive-save-batch-admission-lane.md`
- `docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md` -> `docs/plans/zircon_editor/editor/04-pie-and-simulation.md`
  - `docs/plans/zircon_editor/editor/04/failure-2026-07-12-command-eval-play-state-projection.md`
- `docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md` -> `docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md`
  - `docs/plans/zircon_editor/editor/10/failure-2026-07-12-project-asset-reference-full-gate-regressions.md`
  - `docs/plans/zircon_editor/editor/10/failure-2026-08-02-scene-open-create-project-authority-route-missing.md`
- `docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md` -> `docs/plans/zircon_editor/editor/12-plugin-management.md`
  - `docs/plans/zircon_editor/editor/12/failure-2026-07-27-plugin-list-canonical-catalog-projection-owner-boundary.md`
- `docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md` -> `docs/plans/zircon_plugins/01-plugin-architecture-core.md`
  - `docs/plans/zircon_plugins/01/failure-2026-07-27-native-live-key-hot-reload-contract-drift.md`
- `docs/plans/zircon_editor/editor/09-editor-asset-management.md` -> `docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
  - `docs/plans/zircon_editor/editor/03/failure-2026-07-22-history-dirty-batch-generation-contract-missing.md`
  - `docs/plans/zircon_editor/editor/03/failure-2026-07-22-saved-top-compare-and-mark-save-token.md`
- `docs/plans/zircon_editor/editor/09-editor-asset-management.md` -> `docs/plans/zircon_editor/editor/06-ui-extension-framework.md`
  - `docs/plans/zircon_editor/editor/06/failure-2026-07-22-document-toolkit-save-hook-contract-missing.md`
- `docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md` -> `docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md`
  - `docs/plans/zircon_editor/editor/14/failure-2026-07-23-welcome-project-probe-admission-budget.md`
- `docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md` -> `docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md`
  - `docs/plans/zircon_editor/editor/16/failure-2026-07-11-migrate-assets-commandlet-registry.md`
- `docs/plans/zircon_editor/editor/11-serialization-and-versioning.md` -> `docs/plans/zircon_plugins/01-plugin-architecture-core.md`
  - `docs/plans/zircon_plugins/01/failure-2026-07-30-native-discovery-compile-boundary.md`
- `docs/plans/zircon_editor/editor/12-plugin-management.md` -> `docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
  - `docs/plans/zircon_editor/editor/01/failure-2026-07-29-document-message-producer-missing.md`
- `docs/plans/zircon_editor/editor/12-plugin-management.md` -> `docs/plans/zircon_editor/editor/04-pie-and-simulation.md`
  - `docs/plans/zircon_editor/editor/04/failure-2026-07-29-play-mode-message-producer-missing.md`
- `docs/plans/zircon_editor/editor/12-plugin-management.md` -> `docs/plans/zircon_editor/editor/06-ui-extension-framework.md`
  - `docs/plans/zircon_editor/editor/06/failure-2026-07-28-plugin-contribution-ticket-revoke-contract.md`
- `docs/plans/zircon_editor/editor/12-plugin-management.md` -> `docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
  - `docs/plans/zircon_runtime/runtime/11/failure-2026-07-27-native-plugin-discovery-bounded-refresh-publication.md`
- `docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md` -> `docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
  - `docs/plans/zircon_runtime/runtime/11/failure-2026-07-13-editor-full-harness-runtime-thread-budget.md`
- `docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md` -> `docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md`
  - `docs/plans/zircon_editor/editor/08/failure-2026-07-26-plugin-list-commandlet-registry-projection.md`
- `docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md` -> `docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
  - `docs/plans/zircon_editor/editor/03/failure-2026-07-23-rust-2021-let-chain-operation-group-parse-regression.md`
- `docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md` -> `docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md`
  - `docs/plans/zircon_editor/editor/08/failure-2026-07-23-settings-registry-keymap-user-layer-migration.md`
- `docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md` -> `docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md`
  - `docs/plans/zircon_editor/editor/14/failure-2026-07-23-autosave-job-admission-and-save-mutex-adapter.md`
  - `docs/plans/zircon_editor/editor/14/failure-2026-07-23-settings-registry-job-category-quota-migration.md`
- `docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md` -> `docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md`
  - `docs/plans/zircon_editor/editor/16/failure-2026-07-23-project-session-lock-reuse-for-recovery.md`
- `docs/plans/zircon_plugins/01-plugin-architecture-core.md` -> `docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
  - `docs/plans/zircon_runtime/frameworks/05/failure-2026-07-22-preference-quota-error-kind-toolchain-drift.md`
- `docs/plans/zircon_plugins/04-animation.md` -> `docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
  - `docs/plans/zircon_editor/editor/01/failure-2026-07-31-authoring-world-test-concrete-level-manager.md`
- `docs/plans/zircon_plugins/04-animation.md` -> `docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
  - `docs/plans/zircon_runtime/runtime/11/failure-2026-07-29-dynamic-runtime-animation-module-duplication.md`
- `docs/plans/zircon_plugins/05-navigation.md` -> `docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md`
  - `docs/plans/zircon_editor/editor/05/failure-2026-07-13-plugin-viewport-overlay-provider-runtime-wiring.md`
- `docs/plans/zircon_plugins/05-navigation.md` -> `docs/plans/zircon_editor/editor/12-plugin-management.md`
  - `docs/plans/zircon_editor/editor/12/failure-2026-07-27-plugin-ui-template-v2-runtime-host-wiring.md`
  - `docs/plans/zircon_editor/editor/12/failure-2026-07-27-template-v2-pane-dynamic-control-state-projection.md`
- `docs/plans/zircon_plugins/08-zr-vm.md` -> `docs/plans/zircon_plugins/01-plugin-architecture-core.md`
  - `docs/plans/zircon_plugins/01/failure-2026-07-22-plugin-workspace-lockfile-drift.md`
- `docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md` -> `docs/plans/zircon_plugins/04-animation.md`
  - `docs/plans/zircon_plugins/04/failure-2026-07-29-animation-sequence-caller-root-drift.md`
- `docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md` -> `docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
  - `docs/plans/zircon_runtime/runtime/11/failure-2026-07-31-preference-storage-bounded-persistence-lane.md`
- `docs/plans/zircon_runtime/render/17-performance-and-profiling.md` -> `docs/plans/zircon_editor/editor/11-serialization-and-versioning.md`
  - `docs/plans/zircon_editor/editor/11/failure-2026-07-30-canonical-text-tuple-variant-mutable-finish.md`
- `docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md` -> `docs/plans/zircon_plugins/05-navigation.md`
  - `docs/plans/zircon_plugins/05/failure-2026-08-02-navigation-editor-operation-status-v2-cutover.md`
- `docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md` -> `docs/plans/zircon_plugins/08-zr-vm.md`
  - `docs/plans/zircon_plugins/08/failure-2026-08-01-zrvm-vampire-behavior-test-ownership-gap.md`
- `docs/plans/zircon_runtime/runtime/11-job-system-task-model.md` -> `docs/plans/zircon_editor/editor/11-serialization-and-versioning.md`
  - `docs/plans/zircon_editor/editor/11/failure-2026-07-29-canonical-text-streaming-output.md`
- `docs/plans/zircon_runtime/runtime/11-job-system-task-model.md` -> `docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md`
  - `docs/plans/zircon_runtime/runtime/10/failure-2026-07-29-operation-phase-detail-abi-owner-thread-apply.md`

## 事实归属审查

- SCC 1 的两条边均为事实 owner，禁止通过互换 frontmatter 消环：Runtime12 确实拥有 app input/gamepad storm 的 producer budget；Performance01 确实拥有其 picking 热路径重构引入的 `Vec<IndexedHit>` 类型锚漂移。
- Runtime12 的 storm-budget artifact 仍列出 pointer/axis coalescing、gilrs wake、queue observability 与压力证据等真实未完成项，不能提前 return。
- Performance01 的 picking 类型锚已在当前源码落地并多次不再复现原 E0277；该 artifact 仍因外部整库编译 blocker 缺少 focused Picking 与 Runtime12 mirror 的最终 managed 1/1。完成这条现有验收并按原 lifecycle return，是 SCC 1 最早且不篡改 ownership 的安全断边。
- SCC 2 必须按上方 55 个 exact artifacts 分别复核。`source complete / managed validation pending` 的 artifact 应优先完成原 gate 与 fixed return；仍缺生产合同的 artifact 保持原 owner。不得把 22-plan SCC 批量重定向到 Coordinator01，也不得把 plan-level cycle 当作任一产品 Failure 已修复。

## 前一快照审计（2026-08-15）

- An immutable current-source parse of 614 handoff artifacts reports
  `15 cycle + 8 excessive_depth + 40 schema_validation + 1 self_edge`.
  Coordinator-owned schema errors were reduced to zero by commits `d41424f86`
  and `28502aa55`; the remaining 40 schema errors belong to other numbered plans.
- The exact self-edge is
  `docs/plans/zircon_editor/editor/14/failure-2026-08-12-thread-ownership-guard-test-scope.md`.
  It declares Editor14 as both origin and fixing plan but omits
  `failure_scope: local`. The content describes an Editor14-owned guard repair
  whose Runtime11 and Editor12 prerequisites already have separate failures.
- Editor14 must therefore make the factual choice: declare this artifact local
  if it remains the owner of its guard lifecycle, or name the real lower fixing
  plan if the artifact itself is intended as a cross-plan handoff. Coordinator
  will not edit the foreign untracked file merely to suppress the diagnostic.
- A durable `failure.import` request `71cbeec9bb8a4f3c9c2b6fa0361c481b`
  correctly rejected with `failure_snapshot_stale` after another owner changed
  a handoff during parsing. No mixed snapshot was published; current counts come
  from the same immutable snapshot preparation path without the database write.

## Schema 63 SCC 清单投影（2026-08-15）

- RED proof first established that `GraphDiagnostic` had no structured details,
  `failure_diagnostics` had no `details_json`, and the control snapshot could not
  expose an exact SCC edge inventory.
- Coordinator now computes deterministic strongly connected components rather
  than reporting traversal-dependent DFS back-edges. Each `cycle` diagnostic
  carries a stable SHA-256 `componentId`, the sorted component plans, every
  internal `originPlan -> fixingPlan` edge, and the sorted exact failure
  artifacts that contribute that edge.
- Schema 63 persists those details in `failure_diagnostics.details_json`; legacy
  diagnostics migrate with `{}` and retain their code, message, paths and
  timestamp. The control snapshot projects the same structured object.
- An immutable current-source preparation over `619` handoff artifacts produced
  `2 cycle SCC + 8 excessive_depth + 65 schema_validation + 3 self_edge`.
  The nontrivial SCC is `a26ff7596d79` with `22 plans / 59 edges / 69 exact
  artifacts`; the second is the independent self-loop `4854bc37d50e` with
  `1 plan / 1 edge / 1 artifact`.
- The eight depth diagnostics remain present. This change makes repair evidence
  exact and durable; it does not suppress depth, relabel foreign ownership, or
  claim that the underlying cycles are fixed.

## 修复结果与回传

Open state: `durable SCC inventory implemented / owner forward repairs pending`.
Schema 63 has replaced traversal-shaped cycle paths with two deterministic SCC
inventories and exact edge artifacts. The historical edge list above remains
useful review context, while the durable `details_json` projection is now the
canonical current-source inventory. Foreign plans must still close or correct
their factual handoffs one artifact at a time. No cycle, depth, schema or
self-edge diagnostic is claimed fixed by this Coordinator-only repair.
