---
title: Runtime02 Headless Startup Mid-Start Cancellation
category: zircon_runtime
report_id: Runtime02-headless-startup-cancel-clean-exit-2026-09-29
date: 2026-09-29
session_id: astra-optimize-20260926-batch-a
status: source_candidate_validation_pending
pending_apply: false
implementation_status: applied_source_candidate
validation_status: managed_tests_pending
managed_tests_pending: true
plan_sources:
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/mvp/01-f0-reproducible-bootstrap.md
---

# Runtime02 Headless Startup Mid-Start Cancellation

When cancellation arrives after headless startup enters its managed owner, the next
`ensure_starting` checkpoint returns `OperationCancelled("startup")`. The startup
handoff now treats that exact result as clean cooperative cancellation and returns a
zero-tick, not-ready `Cancelled` report. Other startup failures still enter the
terminal composition-failure path. Cancellation at these checkpoints occurs before
a `RuntimeSession` is created, so the managed owner can exit normally without a
retained runtime or destroy receipt.

The applied regression gates the managed startup operation, cancels from the caller,
then releases the operation through the real `ensure_starting` checkpoint. It
asserts the clean report, an empty failure ledger, and that the owner can be reaped
without a destroy receipt. A companion case returns a genuine startup error while
cancellation is set and verifies the error remains terminal.

`rustfmt --edition 2021 --check` passed on the offline source and test copies. The
managed Runtime tests have not run. Queue these two commands together as one
`zircon_app` `target-server` application batch; the shared `headless_host_` test
filter covers both new regression cases:

- `cargo check -p zircon_app --locked --no-default-features --features target-server --lib`
- `cargo test -p zircon_app --locked --no-default-features --features target-server --lib headless_host_`

The broader `target-editor-host` profile is a separate gate. This F0 repair has
no measured performance result and makes no speedup claim.

Shared source and both regression cases were applied and attributed on 2026-09-29. Managed validation and F0 product acceptance remain pending.

## Implementation owner

The implementation belongs to ZirconApp01 product-host startup. This Runtime02/F0
record tracks the cross-module cancellation handoff; it does not close the Runtime02
event/task report. Acceptance must also be routed to the linked ZirconApp01 host
review and F0 bootstrap gate. The two target-server regressions remain unexecuted.
