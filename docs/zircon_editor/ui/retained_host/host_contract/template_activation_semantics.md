---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics/asset.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics/dispatch.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics/helpers.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics/route.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/button_dispatch.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_input_semantics.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics/asset.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics/dispatch.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics/helpers.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics/route.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics_tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-20 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor --check
  - host_contract template-activation route/asset/dispatch/test ownership scan
  - scoped whitespace scan
  - scoped git diff --check
doc_type: module-detail
---

# Template Activation Semantics

`template_activation_semantics.rs` is the retained-host primary activation entry for template-node hits. Native pointer button dispatch keeps calling the root entry, while route selection, asset-specific activation, concrete callback dispatch, shared helpers, and local regressions live in child modules.

## Route Ownership

`template_activation_semantics/route.rs` owns `TemplatePrimaryActivationRoute` and the route classifier. It resolves text-input focus-only hits first, then dispatch kind based routes for inspector, asset panes, Welcome, showcase, command palette options, Workbench options, Workbench menu items, export wizard actions, binding fallback, and generic surface actions.

The route owner depends on `template_input_semantics::hit_is_text_input(...)` and the command-palette control id so dispatch ordering stays explicit and testable.

## Asset Ownership

`template_activation_semantics/asset.rs` owns asset pane activation. It maps `asset` and `asset:*` dispatch kinds to activity/browser sources, chooses action id over control id, classifies search/filter/view/utility controls as change events, and invokes the correct asset callback.

## Dispatch Ownership

`template_activation_semantics/dispatch.rs` owns `dispatch_template_node_primary_press(...)`, the single callback fan-out entry called by native pointer primary presses after text focus handling. It delegates route decisions to `route.rs`, asset handling to `asset.rs`, and action/control id fallback to `helpers.rs`.

`template_activation_semantics/helpers.rs` owns the shared action-id fallback helper used by Welcome and asset dispatch.

## Root Boundary

The root `template_activation_semantics.rs` only declares child modules, re-exports `dispatch_template_node_primary_press(...)`, and attaches the external test module. It should not regain route enums, asset activation structs, route classifiers, callback fan-out match bodies, helper bodies, or inline tests.

## Test Ownership

`template_activation_semantics_tests.rs` owns local regressions for text-input focus-only routing, Workbench option precedence over binding fallback, command-palette commit routing, Workbench menu item routing, export wizard action precedence, and asset source/change classification.

## Validation Notes

This slice used `cargo fmt -p zircon_editor --check`, a root ownership scan confirming `template_activation_semantics.rs` no longer owns route, asset, dispatch, helper, or inline test bodies, a scoped trailing-whitespace scan, and scoped `git diff --check`. Full Cargo check/test validation remains deferred because current package checks are blocked before editor diagnostics by unrelated `zircon_runtime` render-history errors, and the active instruction is to implement functionality first.
