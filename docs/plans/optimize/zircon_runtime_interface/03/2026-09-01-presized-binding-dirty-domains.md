record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/binding/model/update.rs
related_tests:
  - tools/tests/test_runtime_interface03_binding_dirty_domain_dedup_performance_contract.py
  - zircon_runtime_interface/src/ui/binding/model/update.rs::presized_binding_dirty_domains_release_benchmark
---

# Presized binding dirty-domain projection

## Scope

`UiBindingDirtyDomain::from_dirty_flags` can emit exactly seven ordered domains but previously
started with an empty-capacity vector. A fully dirty update therefore grew the vector more than
once while projecting flags into the binding report.

The projection now allocates capacity for all seven possible outputs once. Domain order,
conditional inclusion, the public return type, and all flag semantics remain unchanged.

## Verification

- TDD RED: the paired static guard found `Vec::new()` in dirty-flag projection.
- Focused paired binding performance contracts after implementation: `2/2` passed.
- Rust behavior test covers the complete seven-domain output and canonical order.
- `python -m compileall`, scoped Rust 1.94.1 `rustfmt --check`, and scoped `git diff --check`:
  passed.
- Managed Windows Rust 1.94.1 behavior tests and release benchmark: pending one asynchronous
  multi-task coordinator batch; no terminal performance number is claimed yet.

Managed batch submission:

- snapshot: `2661`
- snapshot request: `c2de33c65cfb4141831fcc67da963827`
- submit request: `e8a58410368748c7bd6ace7384b48b31`
- coordinator request: `1ea7cda7ce0d455dbf1c9daca5820ce0`
- validation ticket: `e29dff7fdabe4d3188874426d7ac79c9`
- source manifest: `67cfb120ccb3f033e85f97d9a4318b1983a0de50142f2679640670df1202a545`
- command: `cargo +1.94.1 test -p zircon_runtime_interface --locked --release --jobs 1 binding_dirty -- --include-ignored --nocapture`
- submitted state: `queued` (asynchronous; intentionally not polled)

## Performance contract

The ignored release benchmark performs 200,000 fully dirty projections per sample and compares
the previous growth path with exact preallocation over 11 alternating samples. The P95 gate
requires at least 20% improvement; terminal P50/P95 nanosecond values must come from the managed
Windows receipt before integration or WeCom reporting.
