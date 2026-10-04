---
title: Editor01 F0 product startup diagnostics
category: zircon_app
session_id: astra-optimize-20260926-batch-a
status: source_candidate_validation_pending
implementation_status: applied_test_candidate
validation_status: managed_product_tests_pending
plan_sources:
  - docs/plans/mvp/01-f0-reproducible-bootstrap.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
---

# Editor01 F0 product startup diagnostics

## Finding

F0 requires product-level assertions for editor startup failures. The current editor unit tests
exercise the argument and host diagnostic formatters, and the runtime-loading source guard checks
ordering, but neither launches the `zircon_editor` product binary to assert its process exit and
captured stderr.

The applied test candidate adds one feature-gated integration target. It launches the actual
`CARGO_BIN_EXE_zircon_editor` binary for a missing `ZIRCON_RUNTIME_LIBRARY` override and for the
incompatible `--project` plus `--builtin-view editor.scene` arguments. The latter case gives the
process an invalid runtime override as well; the expected argument diagnostic proves startup
argument routing rejects the conflict before runtime preflight.

## Required acceptance

- Missing runtime override exits with code 1 and stderr includes the `runtime_build_set` component,
  the exact `ZIRCON_RUNTIME_LIBRARY` request, the BuildSet preflight cause, and actionable staging
  recovery text. The sidecar probe's platform-specific OS wording is not asserted.
- Mutually exclusive startup arguments exit with code 1 and stderr includes the `editor_app`
  component, both requested flags, the conflict cause, and recovery guidance. Stderr must not
  include runtime preflight diagnostics.
- Both failures remain ordinary startup errors without panic or backtrace diagnostics on either
  captured stream.
- Each subprocess has a 20-second deadline. While it runs, stdout and stderr go to files under a
  unique test-owned temporary root, avoiding undrained pipes. Timeout and wait errors kill and reap
  the child before failing with captured output.
- `ZIRCON_LOG_ROOT` points to a unique directory under that same temporary root. The test clears
  inherited `ZIRCON_LOG_FILTER`, `ZIRCON_LOG`, `RUST_LOG`, and `ZIRCON_LOG_LEVEL` values so logger
  output remains isolated and diagnostics deterministic. An RAII guard removes the complete
  temporary root, including logs and captures, after success, assertion failure, or timeout.
- Run the new target once in the grouped managed Windows `target-editor-host` integration stage.

## Scope

- Applied new file: `zircon_app/tests/f0_editor_product_startup_diagnostics.rs`.
- No editor entry, binary, runtime loader, or shared test file is changed by this candidate.
- Invalid built-in descriptor and unreadable staging asset product assertions remain separate F0
  requirements; this slice only covers the two pre-window process failures above.

## Validation status

Cargo and product execution remain pending after source application. The candidate test's rustfmt
check and the new-file patch applicability check are recorded in its manifest. After root
admission, include
`cargo test --locked -p zircon_app --no-default-features --features target-editor-host --test f0_editor_product_startup_diagnostics`
once in the grouped coordinator-managed validation batch.
