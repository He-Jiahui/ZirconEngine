---
record_kind: milestone
status: partially_implemented
created_at: 2026-09-19
plan: docs/plans/optimize/zircon_app/08-product-host-bootstrap-loop-dynamic-runtime-shutdown-current-source-review.md
milestone: APP-P1-30 Play report primary/secondary failure preservation
session: astra-runtime-play-report-secondary-20260918
---

# Runtime Play report secondary-failure preservation

This bounded slice closes the error-masking part of App08 P1-30. Startup,
Ready, and Terminal Play-report writes now flow through the product failure
ledger. When a startup operation already has a primary error, a report-write
failure is merged after it and cannot replace the primary diagnostic. If the
runtime session is created successfully, early report failures are transferred
to the session-owned ledger before the event loop starts.

The newline-delimited stdout outlet, session/build token, payload length,
checksum, backpressure, and real child-process protocol remain open; this is
not a claim of a completed Play IPC contract or product acceptance.

## Evidence

- Focused static contract batch:
  `python -m unittest tools.tests.test_runtime10_plan_current_contract tools.tests.test_runtime_api_boundary tools.tests.test_runtime_dynamic_api_boundary_archive_ownership tools.tests.test_app08_runtime_request_encoding_performance_contract -q`
  — **8/8 passed**.
- Direct source guard for the new ledger path — **4/4 passed**: startup
  report failures have a recorder and startup-failure merger, the session
  ledger receives early failures, and no `report_play_startup(...)?` path
  remains.
- `rustfmt +1.94.1 --edition 2021 --check` passed for the runtime runner and
  its source guard; scoped `git diff --check` passed with only the repository's
  existing LF/CRLF notices.

Post-edit SHA-256 fingerprints:

```text
zircon_app/src/entry/entry_runner/runtime.rs 7C09A60CA517EA96C41375C4D0857D81BE93249D11287574BFE201DC2D710BDD
zircon_app/src/entry/tests/runtime_entry_source_guards/runtime_session.rs 6F8E3E18024C1EE129697162D5C12861775459153CCF181926A965980F67D4E4
docs/plans/optimize/zircon_app/08-product-host-bootstrap-loop-dynamic-runtime-shutdown-current-source-review.md 07A40AE59222393F23EC41515B42B46220E3948E7D9844B1B531E53B01AC6185
```

## Boundary

No Cargo, native DLL, GPU, or product command ran. The managed validation
request `4b801300236c492d968c9089379ca996` was rejected before Cargo by
`unmanaged_artifacts_detected`, with the authoritative cleanup reservation
pointing to `D:\ZirconBuilds\mvp-test-fixtures-28916`; the external dirty
`E:\Git\zr_vm` dependency remains a second native gate. The full static
discovery baseline still contains active Text03 failures; they are outside
this App08 slice. No commit was created.
