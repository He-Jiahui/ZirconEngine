# Runtime74 Authoritative Binding Apply Report

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: M4
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/74/2026-08-22-authoritative-binding-apply-report.md","docs/zircon_runtime/ui/surface/binding_targets.md","zircon_runtime/src/ui/surface/binding_targets.rs","zircon_runtime/src/ui/surface/binding_transaction.rs","zircon_runtime/src/ui/tests/asset_binding.rs","zircon_runtime/src/ui/tests/asset_binding/apply_report_performance.rs","zircon_runtime/src/ui/tests/event_routing/component_events.rs","zircon_runtime_interface/src/ui/binding/model/mutation_receipt.rs"]

- Date: 2026-08-22
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source item: `RTB-P1-047`
- Delivery state: implementation complete; grouped coordinator validation pending

## Problem

`UiBindingUpdateReport` mixed caller-built projections with actual surface mutation results. Its
update rows already carried previous/new values, but transaction receipts treated every prepared
target as applied, exposed no final revision, and could not distinguish dirty invalidation from
focus/accessibility or action-publication impact. A rollback therefore said only that zero targets
survived, without a complete revision/impact contract.

## Scope Delivered

- The executor records one target outcome immediately after the authoritative Property, Class,
  Visibility, Enabled, or ActionPayload owner returns. Secondary component-state update rows no
  longer inflate target counts.
- `UiBindingMutationReceipt` adds serde-defaulted `revision`, `unchanged_target_count`, and `impact`
  fields while retaining the existing base generation, target count, applied count, and terminal
  outcome contract.
- A committed surface mutation schedules `base_generation + 1`; unchanged, payload-only, rejected,
  and rolled-back transactions retain the base revision. Rollback always publishes zero applied
  targets and an empty impact set.
- Impact begins with the dirty domains returned by the real mutation report. Focus changes add
  Accessibility and Interaction. A changed ActionPayload adds Interaction but does not advance the
  surface revision because it only alters the invocation being published.
- Existing update rows remain the old/new value evidence, preserving public serialization and
  consumers while the transaction receipt becomes the final executor apply authority.

## Regression Contract

- The five-target atomic success path must retain old/new values, report five applied targets, zero
  unchanged targets, a one-step revision, and Layout/Style/Input/Interaction impact.
- Commit-stage rejection must restore state, retain the base revision, and expose no impact.
- A two-field ActionPayload-only commit must report two applied targets and Interaction impact while
  retaining the base surface revision.
- The existing Runtime74 transaction child runs the full component-event module in one Cargo group;
  no duplicate per-test Cargo invocation is added for this slice.

## Performance Contract

`authoritative_binding_apply_receipt_p95_beats_report_reconstruction` runs 21 alternating pairs.
Each sample creates 2,048 receipts with 32 update outcomes. The rejected path scans all 32 rows to
reconstruct applied/unchanged counts and impact; the executor path uses the summary already produced
while applying targets and performs zero post-report update scans.

External validation must independently enforce:

- exactly 21 raw samples per side with 11 legacy-first and 10 authoritative-first pairs;
- 32 legacy update scans and zero authoritative update scans per receipt;
- nearest-rank authoritative P95 at least 50% lower than reconstruction P95.

The standalone validator is
`.codex/state/session-coordinator/cargo-runs/zircon-validation-runtime74-authoritative-binding-apply.ps1`
with SHA-256 `5C5014832E0793147A15EC6EA88E3821593350B05A896F64BF0D61C15902AFC8`.
It is pinned by the 84-task / 56-Cargo-group / 18-performance-row Runtime74 super-batch with
SHA-256 `92059C051B5E6B4341AB2B93242CE201E3F408A88251F313CDA021D8E846A9CC`.

Measured values remain pending coordinator execution. No Cargo, behavior, or performance pass is
claimed.

## Remaining Scope

The receipt covers the current pointer-originated compiled target executor. Model/provider writes,
safe-point batches, commands, and asynchronous operation completion must produce the same
old/new/revision/impact/outcome authority when their Runtime74 owners are implemented.
