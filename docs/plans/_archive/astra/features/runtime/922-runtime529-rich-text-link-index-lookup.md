---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-runtime529-rich-text-link-index-lookup.md
implementation_files:
  - zircon_runtime/src/ui/text/rich_text/link_hit.rs
tests:
  - zircon_runtime/src/ui/text/rich_text/link_hit.rs
---

# Runtime922 Runtime529 rich-text link index lookup

Pointer hit testing now maps caret ownership to the compiled run index rather
than scanning every link run. Boundary, gap, padding, affinity, and metadata
equivalence regressions remain in the owner module.

Marker `RUNTIME529_RICH_TEXT_LINK_INDEX_BENCH_V1` reports indexed versus legacy
candidate checks. Managed Release validation and elapsed-performance evidence
remain pending.
