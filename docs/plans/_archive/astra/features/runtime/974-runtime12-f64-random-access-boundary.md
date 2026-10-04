---
doc_type: milestone-detail
status: implementation_complete_validation_deferred
plan_sources:
  - docs/plans/optimize/zircon_runtime/12/2026-08-26-constant-width-f64-random-access.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
---

# Runtime974 · fixed-width f64 random-access boundary

The Runtime12 plan reports an implementation-complete change in the external
WOC/Zr binary-codec package: fixed-width `readF64LeAt` validates and decodes only
the selected eight bytes instead of copying/scanning the complete payload. Its
model reports byte touches `131,072→8` and P95 `231.3680ms→0.2277ms`, while the
dedicated ZrVM package and seven-contract managed validator remain required.

No corresponding package source exists in this ZirconEngine checkout, and the
user explicitly deferred tooling migration; therefore this record is not
promoted to a local source implementation claim. The external package compile,
parity, and managed ZrVM evidence remain deferred rather than inferred.
