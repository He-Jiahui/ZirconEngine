---
title: Editor268 published project artifact retention
category: zircon_editor
report_id: Editor268-P1-05-publication-2026-09-29
date: 2026-09-29
session_id: astra-optimize-20260926-batch-a
plan_source: docs/plans/optimize/zircon_editor/268-editor-project-startup-open-create-activation-session-recent-recovery-current-working-tree-review.md
implementation_status: source_candidate_pending_validation
validation_status: scoped_static_checks_passed_managed_validation_pending
performance_status: not_measured
---

# Editor268: retain a published project when activation fails

## Source change

The old create path published the project directory before session admission, then rolled it back on a releasable activation failure. The project creation lease and the session ownership lease are separate. Another editor could claim the published directory before the creator's admission attempt; the creator's rollback could then move that active editor's project away.

`ProjectAuthority` now validates the manifest on the staging directory and rebinds the data-only preflight receipt to the final target identity resolved before publication. The successful staging-to-target rename is the irreversible artifact boundary. A returned `CreatedProject` is finalized and has no commit or rollback capability. Failure before publication can restore an original empty target. Failure finalizing its backup after publication returns a typed error with target, backup, and underlying cause while retaining the new project.

`EditorManager` activates from that finalized receipt. Composition mismatch, admission/revalidation failure, and releasable or quarantined activation failure leave the created directory in place. Ordinary activation errors include the created path. If another Ready editor receives the Hub focus request, the typed focus-forwarded error also carries the created path and remains recognizable by the Hub process-ID accessor. The existing project/session contract documentation was corrected accordingly.

## Evidence and remaining gates

- Core source hashes: `authority/create_project.rs` = `4edfee1ccc8c730f9bc325d5229d74ac13260725e4c9669cbcbd3f556de13c94`; `created_project.rs` = `4609a32edf6861bf5c60f8596a0a57dda2efe7d45e8ad59028d754c9a136678b`; `authority/transaction.rs` = `40d9a26ab1cfd80c02b1cdb394dcde24bf632208009be05eca20b6d3029e89dc`.
- Editor source hashes: `editor_manager_project_session.rs` = `31f18a5616fba077f271cc18cb9f840827dbe1b1f96361a26c6c5240a8e6bab5`; `editor_error.rs` = `32e78ded0e4ec8cb6b8c5c6bb96e64076ea9110fcf308cdb12ce78004aecffaf`.
- Rustfmt and scoped diff checks passed. The new or repaired Rust tests cover oversized staged manifests, finalized target identity, preservation on backup-finalization failure, the typed focus error, and the Editor activation branch's source ordering. These Rust tests have not run in the managed batch. The source-order assertion is structural evidence only.
- A controlled two-process create/admission race, kill/restart recovery, finalization fault injection, and full Hub handoff remain untested. Creation and activation still lack durable parent-child receipts and terminal replay, so Editor268 P1-05 remains partial. This change does not close the separate ActivationReceipt fence in P0-01.
- Staged preflight moves the existing manifest read before publication and rebinds its identity without another manifest read. No Release startup latency, p50/p95/p99, allocation, RSS, or I/O comparison has run; no performance acceptance is claimed.

The candidate joins the next grouped Editor package/library validation after the accepted v11 request has a terminal receipt. Do not treat the offline batch draft as a passing result.
