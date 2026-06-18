---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/template_nodes.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node_tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - template-node hit-test test ownership scan
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Template Node Surface Hit Test

`surface_hit_test/template_node.rs` owns retained-host hit testing for template-projected pane and Workbench window nodes. It converts `TemplatePaneNodeData` frames into a `UiSurface`-style surface frame, filters non-dispatchable decorative nodes, resolves popup option/menu rows, and returns the `HostPointerHitData` used by native pointer routing.

The production module intentionally depends on neutral host-contract helpers such as `paint_geometry`, `template_popup_layout`, and `template_component_family` instead of painter namespaces. That keeps hit testing aligned with the runtime UI architecture while the remaining retained host bridge is being converged.

## Test Ownership

`surface_hit_test/template_node_tests.rs` owns the former inline regression bodies. The coverage stays module-local because it exercises private hit-test semantics for componentized Workbench nodes, TextInput family detection, decorative viewport layer filtering, bounded dropdown option rows, popup menu rows, and separator-row fallback blocking.

Keeping these tests out of the production file leaves `template_node.rs` focused on dispatchable geometry and popup-row hit behavior. Future popup or component-family regressions should extend the test owner unless they require a broader native pointer integration test.

## Validation Notes

The 2026-06-18 split is implementation-first. Evidence for this slice is formatting, ownership scans, trailing-whitespace/diff checks, and scoped `zircon_editor` library type checks. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
