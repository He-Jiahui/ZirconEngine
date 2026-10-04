---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-runtime535-duplicate-feature-status-borrow.md
implementation_files:
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_status_record/mutation.rs
tests:
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_status_record/mutation.rs
---

# Runtime927 Runtime535 duplicate feature-status borrowing

Duplicate plugin and capability status updates now accept borrowed identifiers,
check membership first, and allocate only on first insertion. Ordered
diagnostics, deduplication, and capability resolution remain unchanged.

Marker `RUNTIME535_DUPLICATE_FEATURE_STATUS_BORROW_BENCH_V1` reports the
reduced repeated-update clone model. Managed Release validation remains pending.
