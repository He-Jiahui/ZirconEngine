---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-runtime528-rich-text-line-binary-lookup.md
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/text_provenance.rs
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/text_provenance.rs
---

# Runtime921 Runtime528 rich-text line range lookup

Canonical rich-text layouts now use a range binary search before validating the
full visual/text predicate. Reordered, duplicate, and noncanonical payloads
retain the linear fallback semantics.

Marker `RUNTIME528_TEXT_LINE_RANGE_BINARY_LOOKUP_BENCH_V1` reports the
canonical candidate-check reduction model. Managed Release validation remains
pending.
