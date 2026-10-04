---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-editor527-functional-window-indexed-lookup.md
implementation_files:
  - zircon_editor/src/ui/workbench/preset/functional_window.rs
tests:
  - zircon_editor/src/ui/workbench/preset/functional_window.rs
---

# Editor961 Editor527 functional-window indexed lookup

Canonical functional-window queries now check the enum's expected slot and
retain a linear fallback for reordered or custom payloads. The focused
regression preserves both ends of a reordered preset. Marker
`EDITOR527_FUNCTIONAL_WINDOW_INDEXED_LOOKUP_BENCH_V1` models 65,536 lookups and
the 294,912-to-65,536 candidate-check reduction. Managed Editor Release
validation remains pending.
