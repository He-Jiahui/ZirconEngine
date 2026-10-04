---
title: Runtime1060 Headless Startup Cancellation Clean Exit
category: zircon_runtime
date: 2026-09-29
session_id: astra-optimize-20260926-batch-a
plan_source: docs/plans/optimize/zircon_runtime/02/2026-09-29-runtime02-headless-startup-cancel-clean-exit.md
status: source_candidate_validation_pending
pending_apply: false
implementation_status: applied_source_candidate
validation_status: managed_tests_pending
managed_tests_pending: true
plan_sources:
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/mvp/01-f0-reproducible-bootstrap.md
---

# Runtime1060 Headless Startup Cancellation Clean Exit

This F0 completion candidate records the headless startup cancellation repair.
The managed startup handoff now returns a clean `Cancelled` report for the
cooperative `OperationCancelled("startup")` signal and preserves genuine startup
failures as terminal errors.

The applied regression, `headless_host_mid_start_cancel_returns_clean_report_without_retaining_owner`,
uses `managed::run_owned` with a gated startup checkpoint and caller cancellation.
The companion startup-failure case verifies error preservation while cancellation
is active. Both remain pending the grouped managed validation recorded in the
[Runtime02 source record](../../../optimize/zircon_runtime/02/2026-09-29-runtime02-headless-startup-cancel-clean-exit.md).

The pending `zircon_app` `target-server` batch is:

- `cargo check -p zircon_app --locked --no-default-features --features target-server --lib`
- `cargo test -p zircon_app --locked --no-default-features --features target-server --lib headless_host_`

The common `headless_host_` filter includes both new cases. The broader
`target-editor-host` profile is separate. No performance measurement or speedup
is claimed.

Shared source and both regression cases were applied and attributed on 2026-09-29. Managed validation and F0 product acceptance remain pending.

## Implementation owner

The implementation belongs to ZirconApp01 product-host startup. This Runtime02/F0
record tracks the cross-module cancellation handoff; it does not close the Runtime02
event/task report. Acceptance must also be routed to the linked ZirconApp01 host
review and F0 bootstrap gate. The two target-server regressions remain unexecuted.
