# Runtime74 Default Interaction Semantic Roles

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: P2 cleanup
Status: validation_pending
Files: [".codex/state/session-coordinator/cargo-runs/zircon-validation-runtime74-p0-p1-endpoint-superbatch.ps1",".codex/state/session-coordinator/cargo-runs/zircon-validation-runtime74-p2-schema-cleanup.ps1","docs/plans/optimize/zircon_runtime/74/2026-08-22-default-interaction-semantic-roles.md","docs/zircon_runtime/ui/surface/default_interactions.md","zircon_runtime/src/ui/surface/surface/default_interactions.rs","zircon_runtime/src/ui/surface/surface/default_interactions/range.rs","zircon_runtime/src/ui/surface/surface/default_interactions/semantics.rs","zircon_runtime/src/ui/surface/surface/default_interactions/table/columns.rs","zircon_runtime/src/ui/surface/surface/default_interactions/table/mod.rs","zircon_runtime/src/ui/surface/surface/default_interactions/table/selection.rs","zircon_runtime/src/ui/surface/surface/default_interactions/timers.rs","zircon_runtime/src/ui/surface/surface/default_interactions/toast_timer.rs","zircon_runtime/src/ui/surface/surface/default_interactions/tree_view_support.rs","zircon_runtime/src/ui/template/asset/compiler/component_props.rs","zircon_runtime/src/ui/template/asset/compiler/node_expander.rs","zircon_runtime/src/ui/tests/asset_binding.rs","zircon_runtime/src/ui/tests/asset_binding/default_interaction_schema.rs","zircon_runtime/src/ui/tests/runtime_input_reply_routes/keyboard_navigation_routes.rs","zircon_runtime/src/ui/tests/runtime_input_reply_routes/table_pointer_routes.rs","zircon_runtime/src/ui/tests/runtime_input_reply_routes/tree_view_pointer_routes.rs","zircon_runtime/src/ui/tests/v2_asset.rs","zircon_runtime/src/ui/v2/compiler.rs","zircon_runtime_interface/src/ui/widget.rs"]

- Date: 2026-08-22
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source item: `RTB-P2-004`
- Delivery state: implementation complete; grouped coordinator validation pending

## Scope Delivered

- `UI_WIDGET_COMPONENT_ROLE_ATTRIBUTE` is the single cross-module key for descriptor-owned
  interaction semantics. Asset and V2 compilers project `UiComponentDescriptor::role` through it.
- Asset compilation resolves `UiWidgetBehavior::Auto` from the descriptor role and preserves an
  explicit authored behavior. Unknown or non-interactive roles resolve to `Passive` rather than
  consulting the component id.
- The default interaction root and table, tree, range-slider, menu-timer, and toast-timer
  specializations contain no `metadata.component`, `*_COMPONENTS`, or
  `resolved_behavior(component)` fallback.
- Manual route fixtures now declare semantic roles or explicit typed behavior, so behavior tests do
  not encode the removed CamelCase component-name convention.

## Validation Contract

The P2 child source guard locks both compiler projection paths and rejects component-name fallback
in the default-interaction tree. It groups three asset-compiler regressions and one V2 projection
regression with the existing P2 tests. Its SHA-256 is
`1E8F1D955ED3AA4EDB282B1B253631BB91E19CE57AB81687B115330E698E1459`.
It is pinned by the 84-task / 56-Cargo-group / 18-performance-row Runtime74 super-batch with
SHA-256 `92059C051B5E6B4341AB2B93242CE201E3F408A88251F313CDA021D8E846A9CC`.

This cleanup removes component-name scans of up to six string candidates from specialized default
interaction classification and replaces the general asset path with one typed behavior read. This
is deterministic operation-count evidence; the slice adds no independent wall-clock threshold or
performance row. The grouped Runtime74 batch retains 18 existing 21-pair release gates. Coordinator
execution is pending, so no Cargo, behavior, or measured performance pass is claimed.
