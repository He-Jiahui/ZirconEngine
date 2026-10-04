---
title: Editor268 session ledger publication before Ready
category: zircon_editor
report_id: Editor268-P0-01-ordering-2026-09-29
date: 2026-09-29
session_id: astra-optimize-20260926-batch-a
plan_source: docs/plans/optimize/zircon_editor/268-editor-project-startup-open-create-activation-session-recent-recovery-current-working-tree-review.md
implementation_status: source_candidate_pending_validation
validation_status: scoped_static_checks_passed_managed_validation_pending
performance_status: not_measured
---

# Editor268: commit the session ledger before publishing Ready

## Source change

The project admission path prepared the Session effect, committed the guard's Ready generation, and only then committed the Session effect in the durable ledger. A crash between those writes could leave a Ready session record with a Prepared Session effect. The admission path now commits the Session effect and advances the ledger to Ready before `guard.commit_ready()`. Both persistence-failure branches request recovery for active ledger effects, mark the guard RecoveryRequired, and return a quarantined activation failure; they install the exclusive guard in the session slot when that slot is vacant. Recent-project projection remains after successful admission.

The existing source-order regression was corrected to match the multiline ledger calls and to require `prepare < commit < ledger Ready < guard Ready`. A separate recent-project projection regression was updated to the current preflight/disposition function boundary; it now follows the disposition method to the admission owner. A real-file Recovery regression now writes both post-Session-commit/pre-ledger-Ready and post-ledger-Ready/pre-guard-Ready states, drops the owners, reloads the ledger and residual guard, and checks that neither state permits automatic takeover. These tests are queued in the grouped Editor library batch.

## Evidence and limits

- Rustfmt 1.94.1 `--check --edition 2021 --config skip_children=true` passed for all three touched Rust files; scoped `git diff --check` passed.
- A deterministic source-order check returned `session_order 8109 8854 8928 10755 True`, `projection_order 399 647 True`, and `admission_link True`. This is structural evidence, not a compiled Rust test result.
- Independent source review confirmed that the ledger store writes each transition before changing its in-memory state and that `begin_ready()` requires all activation effects committed. It found no concrete bug in the reordered path. Neither that review nor the structural test exercises crash boundaries or injected persistence failures.
- Ledger-order candidate snapshot SHA-256: `editor_manager_project_session.rs` = `f01e0ed6bdbafbedd128edfaf59d124966c5aebec8bd5fe73493c6c683bedffd`; `editor_manager_project_activation_effects.rs` = `418a6896eff2c3086f08c6a393917bbe59d7901ee601e4d11c27732cd9060f1b`. The session source later changed for the separate published-create-artifact retention candidate; see `2026-09-29-published-create-artifact-retention.md` for its current hash and gate.
- Recovery regression SHA-256: `zircon_editor/src/core/recovery/tests.rs` = `545f1cc15dffef46f2faac935daa582f5acc94ae645e565fb62a5e5f5bc1db2c`. The test has been added and source-attributed; it has not been executed by Cargo yet.
- No extra allocation, read, or write was added on the successful path; the two existing durable ledger writes now precede the existing guard Ready write. No Release latency or product p50/p95/p99 result has been measured, so this record makes no performance acceptance claim.

This closes only the known Ready-before-Session-ledger ordering window in Editor268 P0-01. The guard and ledger remain separate atomic files. The new regression covers two adjacent on-disk states but does not inject a write failure or kill a second process. A common ActivationReceipt digest, the full crash/restart fault matrix, Hub/focus receipt validation, and deterministic split-brain recovery remain open. `PROJ-GATE-03` remains partial rather than accepted. The historical 2026-08-31 review describes its observed baseline; this dated record describes the newer source candidate.

## Pending grouped gate

The Editor package check and library regression batch must run from a coordinator-issued current-source manifest after the already accepted v11 request reaches a terminal receipt. Keep this candidate pending until those tests, focused crash/restart tests, and relevant product/Release performance gates have real receipts. Do not infer acceptance from the offline successor draft.
