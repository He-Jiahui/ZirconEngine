# Runtime74 Bounded Binding Execution Telemetry

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: M4
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/74/2026-08-22-bounded-binding-execution-telemetry.md","docs/zircon_runtime/ui/surface/binding_targets.md","zircon_runtime/src/ui/surface/binding_targets.rs","zircon_runtime/src/ui/template/asset/compiler/binding_program.rs","zircon_runtime/src/ui/tests/asset_binding.rs","zircon_runtime/src/ui/tests/asset_binding/compiled_program.rs","zircon_runtime/src/ui/tests/asset_binding/telemetry_performance.rs","zircon_runtime/src/ui/tests/event_routing/component_events.rs","zircon_runtime_interface/src/ui/binding/mod.rs","zircon_runtime_interface/src/ui/binding/model/execution_receipt.rs","zircon_runtime_interface/src/ui/binding/model/mod.rs","zircon_runtime_interface/src/ui/binding/model/update.rs","zircon_runtime_interface/src/ui/template/asset/compiler/binding_program.rs"]

- Date: 2026-08-22
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source item: `RTB-P1-046`
- Delivery state: implementation complete; grouped coordinator validation pending

## Problem

Binding target reports exposed applied, unchanged, rejected, dirty-domain, and transaction data, but
operators could not attribute execution attempts, endpoint misses, evaluation errors, or cost to the
compiled asset, binding, and generation. Encoding those identities into profiling counter names
would create unbounded metric cardinality and a second diagnostic authority.

## Scope Delivered

- `UiCompiledBindingProgram` retains the source asset identity. The field is serde-defaulted and
  omitted when empty, so older artifacts without the field remain readable and well formed.
- Every target-bearing binding report carries at most one `UiBindingExecutionReceipt` with asset,
  binding, generation, execution/miss/error counts, and elapsed nanoseconds.
- Receipt identity payloads are capped at 256 asset bytes and 128 binding bytes. Over-budget UTF-8
  identifiers retain a stable 64-bit hash suffix inside the same limit.
- Stale, missing, and mismatched compiled endpoints produce a miss receipt. Valid endpoint
  evaluation produces an execution receipt; rejected preparation or commit also increments error.
- The binding executor emits only four fixed `ui.binding.*` counters from that receipt. Asset and
  binding values never enter counter names or create another high-cardinality profiling owner.

## Regression Contract

- Compilation must retain `editor.binding.valid` as the binding program asset identity.
- Deserializing the same program with `asset_id` removed must yield no asset identity while remaining
  well formed.
- Successful atomic target execution must report `execution=1, miss=0, error=0`.
- Unresolved target expressions must report `execution=1, miss=0, error=1` and preserve rollback.
- A stale generation must report `execution=0, miss=1, error=0` and suppress mutation.

## Performance Contract

`bounded_binding_execution_receipt_p95_beats_dynamic_metric_cardinality` runs 21 alternating sample
pairs. Each sample processes 4,096 receipts. The rejected baseline formats and updates 128 dynamic
asset/binding/generation metric keys; the delivered path constructs the bounded receipt and retains
four fixed metric identities.

External validation must independently enforce:

- exactly 21 raw samples per side with 11 dynamic-first and 10 bounded-first pairs;
- dynamic metric keys per sample are 128 and bounded metric keys are 4;
- nearest-rank bounded P95 is at least 50% lower than the dynamic-cardinality baseline.

The standalone validator is
`.codex/state/session-coordinator/cargo-runs/zircon-validation-runtime74-bounded-binding-telemetry.ps1`
with SHA-256 `1EB20A7610C35BCA3923579337953E71B5E7E9B9006730ABCCB023E52B5B6C98`.
It is pinned by the 84-task / 56-Cargo-group / 18-performance-row Runtime74 super-batch with
SHA-256 `92059C051B5E6B4341AB2B93242CE201E3F408A88251F313CDA021D8E846A9CC`.

Measured values remain pending coordinator execution. No Cargo or performance pass is claimed.

## Validation History

Ticket `a1a60498f9374d66baf07dc7245f6f34` materialized the preceding 63-task snapshot before this slice
was edited. Its result can validate the prior direct compiled-event work but is not acceptance
evidence for RTB-P1-046. Ticket `137cab94338a4ffb87f7f6f31e73fb16` used an incomplete overlay
manifest and is not acceptance evidence. Corrected ticket `9ef04781a7a3417daa8000bca490ebfc`
was rejected during materialization with `validation_copy_overlay_not_owned`; Cargo never started.
The active-owner profiling overlay was removed, the three new receipt/gate/record paths received an
audited scope transfer, and this slice will be included with RTB-P1-047 in the next grouped ticket.

## Remaining Scope

The receipt currently covers pointer-originated compiled target execution. Model subscription polls,
safe-point batches, two-way writes, commands, and asynchronous operation outcomes require the same
receipt contract when their Runtime74 execution owners are delivered.
