---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-runtime540-font-alias-append-invariant.md
implementation_files:
  - zircon_runtime/src/text/font/backend.rs
tests:
  - zircon_runtime/src/text/font/backend.rs
---

# Runtime932 Runtime540 font-alias append invariant

After detaching a backend ID from prior faces, alias insertion now appends
directly under the map's uniqueness invariant and retains only a debug assertion
for that invariant. Rebind order and reverse removal behavior remain covered.

Marker `RUNTIME540_FONT_ALIAS_APPEND_INVARIANT_BENCH_V1` reports the removed
post-detach membership comparisons. Managed Release validation remains pending.
