---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-runtime541-font-primary-detach-entry.md
implementation_files:
  - zircon_runtime/src/text/font/backend.rs
tests:
  - zircon_runtime/src/text/font/backend.rs
---

# Runtime933 Runtime541 font-primary detach entry

Primary-face rebinding now reuses the retained alias entry discovered during the
detach step instead of performing a second lookup. Existing primary/alias
promotion and stable-order behavior remain covered.

Marker `RUNTIME541_FONT_PRIMARY_DETACH_ENTRY_BENCH_V1` reports the avoided second
lookup model. Managed Release validation remains pending.
