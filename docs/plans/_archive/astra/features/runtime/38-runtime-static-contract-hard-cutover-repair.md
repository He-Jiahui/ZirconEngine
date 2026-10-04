---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/10/2026-08-26-status-diagnostics-abi-hard-cut-pre-review.md
  - docs/plans/optimize/zircon_runtime/43-dynamic-runtime-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-review.md
related_code:
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_runtime/src/graphics/shader/invocation/fullscreen_pass.rs
  - zircon_runtime/src/dynamic_api/session/runtime_ui.rs
tests:
  - tools/tests/test_runtime_absorption_current_source_fixture.py
  - tools/tests/test_runtime_receipt_hard_cut.py
  - tools/tests/test_runtime_fullscreen_pass_owner_structure.py
  - tools/tests/test_runtime_dynamic_ui_extract_generation_contract.py
  - tools/tests/test_runtime_dynamic_api_boundary_archive_ownership.py
---

# Runtime Static Contract Hard-Cutover Repair

The static regression owners now follow the current hard-cut topology. Retired
`plan_status` fixtures remain absent, fullscreen-pass verification reads the
canonical `graphics/shader/invocation` owner, and the Runtime UI allocation
assertion reads its folder-backed test module rather than requiring a test-only
symbol in production source.

Runtime session creation now uses the existing
`From<RuntimeLibraryError> for RuntimeSessionCreateFailure` conversion through
`?`. This keeps the status-validation failure type unchanged while satisfying
the current status-diagnostics archive contract.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| R38 | Repair Runtime hard-cutover static contracts and preserve the session-create failure conversion | implemented_pending_validation | The focused five-file Runtime static batch passed 15/15, and the combined R38/E33 contract batch passed 24/24. Python compilation and scoped diff checks passed. Whole-file rustfmt reports pre-existing unrelated formatting drift in the concurrently modified session source, so it was not rewritten. Managed Rust compilation and Windows Release p50/p95/p99 evidence remain pending; no product performance target is claimed. |

## Coordinator dispatch log

- 2026-09-11: the combined `zircon_app`/`zircon_runtime`/`zircon_editor`
  Cargo batch was submitted with request id
  `astra-runtime-editor-compile-batch-20260911-r1`, but immutable admission
  rejected it before a ticket was created because external worktree
  `E:\\Git\\zr_vm` is dirty. No Cargo command ran and this record does not infer
  a compile result from the rejection.
