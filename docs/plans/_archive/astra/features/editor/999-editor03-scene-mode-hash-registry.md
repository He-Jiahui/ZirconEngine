---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/03/2026-08-26-scene-mode-hash-registry.md
related_records:
  - docs/plans/astra/features/editor/997-editor03-scene-history-selection-completion-list.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/scene/modes/scene_mode_registry.rs
  - zircon_editor/src/scene/modes/scene_mode_registry/hash_lookup_tests.rs
tests:
  - zircon_editor/src/scene/modes/scene_mode_registry/hash_lookup_tests.rs
---

# Editor999 · scene-mode hash registry

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor03 scene-mode registration | Factory/descriptor lookup uses `HashMap`; a sorted ID vector with binary-position insertion preserves deterministic `registrations()` order, duplicate rejection, plugin boundaries, and factory validation. | Focused tests cover shuffled registration, stable iteration, descriptor lookup, and the ordered-index source contract. The ignored marker `EDITOR03_SCENE_MODE_HASH_REGISTRY_BENCH_V1` requires hash lookup P95 at least 30% below the legacy ordered path. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/scene/modes/scene_mode_registry.rs` | `F2F5998C07862C5A14538185A7CE899C206F18A33E5A0385B930227779336FAE` |
| `zircon_editor/src/scene/modes/scene_mode_registry/hash_lookup_tests.rs` | `DBC395D19DC63EA0A18EFE8817A9FC38BB833E70237B58AB94701C9702E24594` |

## Validation handoff

The scene-mode behavior/source contracts and Release marker are included in the
grouped Editor package validation; exact managed P50/P95 and product scene-mode
evidence remain pending.
