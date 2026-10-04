---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu.rs
  - zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu/classifier.rs
  - zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu/path.rs
  - zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu/provider.rs
  - zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu/request.rs
  - zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/button_dispatch.rs
  - zircon_editor/src/ui/retained_host/app/workbench_context_menu.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu.rs
  - zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu/classifier.rs
  - zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu/path.rs
  - zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu/provider.rs
  - zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu/request.rs
  - zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu_tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-20 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor --check
  - host_contract workbench-context-menu request/provider/classifier/path/test ownership scan
  - scoped whitespace scan
  - scoped git diff --check
doc_type: module-detail
---

# Workbench Context Menu

`workbench_context_menu.rs` is the retained-host request entry for Workbench secondary-press context menus. Native pointer dispatch keeps calling the root entry, while request assembly, provider definitions, hit classification, target-path normalization, and regressions live in child modules.

## Request Ownership

`workbench_context_menu/request.rs` owns `workbench_context_menu_request_for_hit(...)`. It rejects empty targets and already-open popup rows, asks the classifier for a provider, and assembles `WorkbenchContextMenuRequestData` with target metadata, anchor coordinates, target path, and menu rows.

## Provider Ownership

`workbench_context_menu/provider.rs` owns `WorkbenchContextMenuProvider`. It defines the scene-node, module-node, and generic Workbench provider menu rows and target-path prefixes.

## Classification Ownership

`workbench_context_menu/classifier.rs` owns hit classification. It identifies scene tree rows, module/effect rows, and actionable generic Workbench controls based on control ids, action ids, bindings, and edit/commit actions.

`workbench_context_menu/path.rs` owns target value fallback and URL-safe path segment normalization.

## Root Boundary

The root `workbench_context_menu.rs` only declares child modules, re-exports `workbench_context_menu_request_for_hit(...)`, and attaches the external test module. It should not regain request assembly, provider menu catalogs, classifier predicates, path normalization, or inline tests.

## Test Ownership

`workbench_context_menu_tests.rs` owns local regressions for scene-node context menu projection and popup-row suppression.

## Validation Notes

This slice used `cargo fmt -p zircon_editor --check`, a root ownership scan confirming `workbench_context_menu.rs` no longer owns request, provider, classifier, path, or inline test bodies, a scoped trailing-whitespace scan, and scoped `git diff --check`. Full Cargo check/test validation remains deferred because current package checks are blocked before editor diagnostics by unrelated `zircon_runtime` render-history errors, and the active instruction is to implement functionality first.
