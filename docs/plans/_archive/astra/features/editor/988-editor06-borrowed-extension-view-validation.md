---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/06/2026-08-27-borrowed-extension-view-validation.md
related_records:
  - docs/plans/astra/features/editor/987-editor06-plugin-manager-optimization-completion-list.md
  - docs/plans/astra/features/editor/24-plugin-manager-contract-maintenance.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/host/editor_extension_views.rs
  - zircon_editor/src/tests/editor_event/runtime/extensions_validation.rs
tests:
  - zircon_editor/src/tests/editor_event/runtime/extensions_validation.rs
---

# Editor988 · borrowed extension-view validation

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor06 extension view admission | Candidate-batch validation reserves a `HashSet` from the view count and stores borrowed `&ViewDescriptorId` keys instead of cloning every descriptor ID. Registry-first checks, duplicate order, and error payloads remain unchanged. | The source contract covers the borrowed index and the two in-source Rust behavior tests cover unique and duplicate batches. The standalone 32,768-ID model reports allocations/bytes `32,768→0` / `3,538,944→0`, P50 `24,196,900→6,687,900ns` (`72.361%`), and P95 `101,465,300→25,082,200ns` (`75.280%`); checksum `1,146,880`. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/host/editor_extension_views.rs` | `175333BF38F7F9BD404A73CE01BDD15261EDAD84C4FABB87670F3302C2825F29` |
| `zircon_editor/src/tests/editor_event/runtime/extensions_validation.rs` | `65BD5A5CE492F67B5583A8B88AAB4DDDED837D6692732F793EC33F5DEE466D95` |

## Validation handoff

The local source contracts, exact-file Rustfmt, and standalone optimized model
pass. The Editor package lib-test batch remains the authoritative gate for the
two in-source tests; managed Release allocation/P50/P95 and plugin-product
evidence remain pending.
