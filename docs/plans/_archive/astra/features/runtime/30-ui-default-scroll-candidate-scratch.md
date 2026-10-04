---
related_code:
  - zircon_runtime/src/ui/tree/node/interaction.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/surface/event_routing.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_runtime/src/ui/tree/node/interaction/first_scrollable_short_circuit_tests.rs
  - zircon_runtime/src/ui/tests/shared_core/scroll_mutation/pointer_routes.rs
implementation_files:
  - zircon_runtime/src/ui/tree/node/interaction.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/surface/event_routing.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11a/2026-09-10-default-scroll-candidate-scratch.md
tests:
  - zircon_runtime/src/ui/tree/node/interaction/first_scrollable_short_circuit_tests.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_runtime/src/ui/tests/shared_core/scroll_mutation/pointer_routes.rs
  - tools/tests/test_runtime_pointer_hover_hot_paths_pressure.py
  - tools/tests/test_runtime_ui_scroll_geometry_patch_performance_contract.py
  - tools/tests/test_runtime_ui_scrollbar_target_index_performance_contract.py
doc_type: milestone-detail
status: implemented_pending_validation
---

# UI Default Scroll Candidate Scratch

The default nested-scroll fallback now reuses a surface-local candidate buffer rather than
allocating a temporary `Vec` for every unhandled wheel route. It preserves the prior behavior:
all candidate nodes are validated in route order before the first scroll mutation, and eligible
scroll owners are still tried in that same order.

The runtime-only buffer is cleared after every result and releases an oversized allocation beyond
the existing discrete UI limit. It is not serialized with authoring surface state.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M30 | Reuse the fully validated default nested-scroll candidate buffer per `UiSurface` | implemented_pending_validation | Runtime UI pointer/scroll/hot-path static batch `73/73`; Python syntax, scoped Rustfmt, and scoped diff checks pass. Rust regressions prove a late missing candidate cannot mutate an earlier scroll owner and cover warm-path capacity reuse. Managed Runtime Cargo and Windows Release allocation/time p50/p95/p99 evidence remain pending. |
