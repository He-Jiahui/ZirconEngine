---
related_code:
  - zircon_editor/src/ui/retained_host/app/pane_payload_visibility.rs
  - zircon_editor/src/ui/retained_host/app/native_keyboard_actions.rs
  - zircon_editor/src/ui/retained_host/app/workbench_context_menu.rs
  - zircon_editor/src/ui/retained_host/app/menu_pointer.rs
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

# Editor retained small input adapters current review (2026-07-31)

## Scope

The four single-file adapters below are reviewed at their current SHA. Together they contain **4/4 Rust files, 127 physical lines, and 0 inline tests**. `native_keyboard_actions.rs` contains an external uncommitted F2 hierarchy-rename routing change; this review preserves that current content and does not claim ownership of it.

| Module/file | SHA-256 | Current-source conclusion |
|---|---|---|
| `pane_payload_visibility.rs` | `5F28A6E12D896F95435CE89AE82AC1745E7822112934E636A36CF37CE646FB85` | Each query scans document tabs, every visible tool stack and all floating tabs. One main slow recompute calls it five times; native-window payload preparation adds two more scans. Fold the visible `ViewContentKind` set once per committed model generation and share it between main/native projection under PERF-MVP-106. |
| `native_keyboard_actions.rs` | `E8E0829BCB76812781EBA384041CAB5516E0854C95962BD289D7132AC29737E7` | Non-F2 input performs an O(1) rename gate then reuses the controller-owned keymap. F2 alone scans selected hierarchy entries until a second match and clones one name. No new performance task; an end-to-end F2 adapter test and managed Cargo remain open. |
| `workbench_context_menu.rs` | `1C9AC80530FAA55187EABBCD3474D1ABA62450D5BDE6AF5CCAAB44FD2B9618F3` | Same-window focus already has a no-op guard, but the next active-document predicate builds a complete `chrome_snapshot()` for one document id. Opening then mutates the context-menu surface and refreshes presentation. The gate belongs to PERF-MVP-105; surface/property work remains in PERF-MVP-106/128 rather than a consumer cache. |
| `menu_pointer.rs` | `FC58D55CEA13DCD011B4B9CBF496BA721EC4E8D8E5C72DD8F992E44B790E9F2D` | Every move/scroll publishes unchanged invalidation diagnostics, accepts a full cloned menu state, and writes all menu state fields to Slint even when the hover/offset is unchanged. Effective scroll/submenu changes may still rebuild the full menu surface. Reuse PERF-MVP-601 for diagnostics and PERF-MVP-112 for typed changed/damage plus visible popup state. |

## Call and test boundary

The review traced `host_lifecycle/pane_payloads`, native-window payload preparation, committed pointer layout, menu bridge move/scroll/surface rebuild, callback wiring, hierarchy rename, keymap dispatch and the componentized context-menu bridge. Existing guards cover one shared editor-pane instance snapshot, three hierarchy-rename helpers, controller keymap dispatch, context-menu projection and the previously reviewed 40/40 retained-menu suite; none proves the new current-source 1M stable-input or 10k-pane/menu budgets.

Slint's `PropertyTracker::evaluate_if_dirty` evaluates only a dirty dependency generation. Godot reads committed control visibility directly, rejects zero mouse motion before popup work, and returns immediately when `set_focused_item` receives the current index. Zircon should likewise publish one visible-kind generation and return typed `changed=false` before state projection; event consumers must not build parallel caches.

## Dynamic acceptance still open

Run coordinator-managed current-source Cargo plus 1/100/10k tabs and menu rows, 1M stable move/scroll/key events, and main/native context-menu traces. Record model scans, chrome builds, state/path clone bytes, Slint setters, surface rebuilds, diagnostics writes and UI p95. Stable generation requirements are: visible-kind scan <= 1 per model generation, context-menu document-id gate full chrome build = 0, same menu hover/zero-clamped scroll state write = 0, and diagnostics write = 0. Preserve focus, F2 rename, menu route/submenu/scroll, context action and child-window behavior. Until those gates and independent review pass, all four files stay in `pending.md` and out of `review.md`.
