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

`SelectionModel` owns one ordered Edit selection and an instance-qualified `BTreeMap<PlayInstanceId, DomainSelection>`, plus primary entity, per-domain generation and cross-domain revision. `core::play::WorldDomain` is the only world-domain type; the former unqualified scene-selection enum is deleted. Viewport, hierarchy and inspector consumers read the active domain, so equal numeric entity IDs in Edit and two Play instances never name the same selection state.

A Play selection domain is created only after `PlayDomainLink` has attached a real session and issued a non-zero `PlayInstanceId`. Activation seeds that instance from the Edit selection without sharing mutable state. The retained tick settles runtime lifecycle, synchronizes the instance-qualified selection domain, then synchronizes hierarchy and Inspector. Exit restores the captured Edit model; `retire_play_domain` supports explicit per-instance retirement. The serialized `WorldDomain` representation rejects the reserved zero instance identity.

`replace` materializes a new ordered set because it replaces the complete snapshot and must normalize duplicates/primary. Incremental `extend`, `toggle` and `clear` mutate the existing `IndexSet` in place. They update primary and generation exactly once when the logical set changes and do not clone all previously selected entities. This keeps a single Ctrl-click or incremental box-selection batch proportional to inserted/removed items instead of the existing selection size plus a full-set allocation.

The behavior tests cover order, duplicate normalization, primary fallback, generation/revision stability, Edit/two-Play-instance isolation, instance retirement and viewport ownership. A source guard prevents `self.items.clone()` from returning to incremental mutations. Current-source Cargo validation remains pending; the Editor04 contract and evidence are recorded in `docs/plans/zircon_editor/editor/04/2026-08-26-play-domain-startup-hardcut.md`.
