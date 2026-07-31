---
related_code:
  - zircon_editor/src/scene/selection
plan_sources:
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
tests:
  - zircon_editor/src/scene/selection/tests.rs
  - cargo test -p zircon_editor --lib scene:: --locked --jobs 1
doc_type: module-detail
---

# Scene Selection

`SelectionModel` owns ordered Edit/Play selection sets, primary entity, per-domain generation and cross-domain revision. Viewport, hierarchy and inspector consumers read the active domain; changing one domain never mutates the other.

`replace` materializes a new ordered set because it replaces the complete snapshot and must normalize duplicates/primary. Incremental `extend`, `toggle` and `clear` mutate the existing `IndexSet` in place. They update primary and generation exactly once when the logical set changes and do not clone all previously selected entities. This keeps a single Ctrl-click or incremental box-selection batch proportional to inserted/removed items instead of the existing selection size plus a full-set allocation.

The behavior tests cover order, duplicate normalization, primary fallback, generation/revision stability, Edit/Play isolation and viewport ownership. A source guard prevents `self.items.clone()` from returning to incremental mutations. Current-source crate validation remains pending in the shared Cargo FIFO; static source guard and formatting evidence are recorded in `docs/plans/performance/01/2026-07-18-editor-scene-static-review.md`.
