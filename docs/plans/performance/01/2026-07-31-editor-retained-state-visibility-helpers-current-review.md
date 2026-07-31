---
related_code:
  - zircon_editor/src/ui/retained_host/app/asset_surface_pointer_state.rs
  - zircon_editor/src/ui/retained_host/app/reference_drop_payload.rs
  - zircon_editor/src/ui/retained_host/app/runtime_diagnostics_visibility.rs
  - zircon_editor/src/ui/retained_host/app/workbench_snapshot_access.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/properties.rs
  - dev/godot/editor/editor_node.cpp
  - dev/godot/scene/gui/popup_menu.cpp
tests:
  - direct rustfmt --check 4/4 passed
  - current-source Cargo, scale counters, F4 trace, and independent review pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor retained state and visibility helpers current review (2026-07-31)

## Scope

These current-source helpers contain **4/4 Rust files, 208 physical lines, and 0 inline tests**. All four are clean in the working tree at review time.

| Module/file | SHA-256 | Current-source conclusion |
|---|---|---|
| `asset_surface_pointer_state.rs` | `EA7537B68EDC08497BDC1440F73C4C27B01434A34BD7C34DBAA5A4494387E5C9` | Construction and `activity`/`browser`, `references`/`used_by` selection are constant-size matches over retained state. This file is not a hotspot and needs no task. Its reference bridge consumer still returns a full state and the host writes all eight asset UI fields for one stable list event; that downstream issue remains PERF-MVP-109. |
| `reference_drop_payload.rs` | `59ECFF8DD8FEFCDFD2C71E8FEBB44D5CDE52F81ED3ECCBABDB443C91F9EF2B96` | Drop-only routing checks three fixed action families, probes at most three optional payload slots, moves exactly one payload, then clears three slots. It is bounded O(1), allocates no collection and is a positive baseline. Payload construction before the drag threshold remains PERF-MVP-109, not this take path. |
| `runtime_diagnostics_visibility.rs` | `8435E0B5DB2E12E8AA4C557BBDB0FB7BB1C9A23788EDCC1617082FF267A1BB25` | It performs another full document/tool/floating-tab traversal for two diagnostic kinds. Combined with five generic main-pane visibility queries, main slow recompute performs six traversals; native payload projection adds two more, for eight over the same model. Fold all kinds once into the PERF-MVP-106 committed visibility generation. |
| `workbench_snapshot_access.rs` | `0345E985A2673B57D78E53BEF025AB745D88225D93FE4F993E82198308696B47` | Production active-document queries build complete chrome before a linear active-page lookup; floating source mapping linearly scans windows, and Welcome projects an owned path Vec. These are existing PERF-MVP-105/117 consumers. Publish active document/surface/window identities with the committed workbench generation; do not add helper-local caches. |

## Call and test boundary

The review traced asset reference move/scroll dispatch and eight-field UI writeback, drag-source/drop behavior, host lifecycle pane payload collection, child-window focus, Welcome pointer layout and componentized Workbench gates. Drop behavior has explicit asset/scene/object priority, consume/reject/clear and real showcase integration tests. Existing pointer source guards protect committed asset snapshots and size no-ops; componentized Workbench tests cover active template identity. There is no direct test proving diagnostic visibility demand or stable reference-list zero writeback.

Slint evaluates retained dependencies only when their tracker is dirty. Godot queries committed control visibility, rejects zero popup motion, and returns early when focus is unchanged. Zircon's model generation should likewise publish a compact visible-kind/active-surface artifact once, while pointer dispatch returns `changed=false` before cloning/projecting state.

## Dynamic acceptance still open

Run coordinator-managed Cargo and 1/100/10k tabs/windows/reference rows with 1M stable reference move/scroll events. Record model/tab visits, chrome builds, window scans, state/path/UUID clone bytes, asset UI setters, surface rebuilds and UI p95. Stable requirements: visibility traversal <= 1 per model generation across main/native consumers, active document/surface query full chrome build = 0, same reference hover and zero/clamped scroll setter/rebuild = 0, and drop take remains one owner move with constant probes. Preserve unknown surfaces/lists, known/unknown references, focus, Welcome paths and payload priority/clear semantics. These files remain in `pending.md` until dynamic gates and independent review pass.
