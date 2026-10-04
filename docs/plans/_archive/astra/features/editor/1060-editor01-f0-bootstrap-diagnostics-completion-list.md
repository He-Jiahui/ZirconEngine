---
doc_type: feature-completion
status: source_candidate_validation_pending
implementation_status: applied_test_candidate
validation_status: static_checks_passed_managed_product_tests_pending
performance_status: not_applicable_no_performance_claim
plan_sources:
  - docs/plans/mvp/01-f0-reproducible-bootstrap.md
  - docs/plans/optimize/zircon_app/01/2026-09-29-editor01-f0-product-startup-diagnostic-gates.md
implementation_files:
  - zircon_app/tests/f0_editor_product_startup_diagnostics.rs
tests:
  - zircon_app/tests/f0_editor_product_startup_diagnostics.rs
---

# Editor1060: Editor01 F0 product startup diagnostics

| F0 failure case | Regression assertion | Status |
| --- | --- | --- |
| Missing runtime override | Launch the real `zircon_editor` binary with a unique nonexistent `ZIRCON_RUNTIME_LIBRARY`; assert exit 1, requested path, preflight cause, staging recovery, and no panic/backtrace output. | Applied test; managed run pending. |
| Mutually exclusive arguments | Launch with `--project <path> --builtin-view editor.scene` and an invalid runtime override; assert exit 1 before runtime preflight, with no panic/backtrace output. | Applied test; managed run pending. |

The candidate targets the explicitly declared `zircon_editor` binary under
`target-editor-host`. Both launches use a 20-second deadline, file-backed stdout/stderr, and a
test-owned `ZIRCON_LOG_ROOT`; the temporary root is cleaned after success, failure, or timeout.
Inherited log filter variables are cleared. One grouped managed integration run remains required
after source application. Static review and source application are complete; managed product execution and remaining F0 failure cases are open.

- [x] Add two bounded product diagnostic regressions and isolated capture/log roots.
- [x] Complete independent static review and fix Command mutability.
- [ ] Execute the named target in the grouped managed target-editor-host batch.
- [ ] Complete invalid-descriptor and unreadable-staging-asset F0 product cases.
