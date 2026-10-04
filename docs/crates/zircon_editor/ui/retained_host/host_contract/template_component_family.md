---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family/classify.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family/family.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family/layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family/roles.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family/visual_language.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family/workbench.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_input_semantics.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family/classify.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family/family.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family/layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family/roles.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family/visual_language.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family/workbench.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family_tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-20 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor --check
  - host_contract template-component-family enum/classify/role/layout/workbench/visual/test ownership scan
  - scoped whitespace scan
  - scoped git diff --check
doc_type: module-detail
---

# Template Component Family

`template_component_family.rs` is the retained-host semantic component-family entry. It preserves the existing import surface used by activation semantics, input semantics, and template hit testing while moving concrete family definitions and classification rules into focused child modules.

## Family Ownership

`template_component_family/family.rs` owns `TemplateComponentFamily` and its stable string labels. This module is the enum owner; it should not accumulate node inspection or Workbench control-id matching rules.

## Classification Ownership

`template_component_family/classify.rs` owns the classification pipeline for a `TemplatePaneNodeData`: declared component role first, host role second, category/layout fallback third, and Workbench control-id fallback last. It also owns the `is_component_family(...)` and `is_any_component_family(...)` helpers.

`template_component_family/roles.rs` owns declared runtime role and retained host-role mappings.

`template_component_family/layout.rs` owns category/layout-role fallback rules for collection, container, selection, and feedback nodes.

`template_component_family/workbench.rs` owns Workbench control-id heuristics. This keeps Workbench-specific string matching out of generic role/layout owners.

## Visual Language Ownership

`template_component_family/visual_language.rs` owns `uses_workbench_visual_language(...)`, which detects Workbench styling from control id and declared variant fields. This is intentionally separate from semantic family classification because visual language can be declared independently of component family.

## Root Boundary

The root `template_component_family.rs` only declares the child modules, re-exports the existing semantic helpers, and attaches the external test module. It should not regain enum bodies, role match tables, category/layout rules, Workbench id heuristics, visual-language checks, or inline tests.

## Test Ownership

`template_component_family_tests.rs` owns local regressions for role precedence, category/layout fallback, visual-language detection, and slider family matching. Tests stay outside production files so the root remains a structural owner boundary.

## Validation Notes

This slice used `cargo fmt -p zircon_editor --check`, a root ownership scan confirming `template_component_family.rs` no longer owns enum, classifier, role, layout, Workbench, visual-language, or inline test bodies, a scoped trailing-whitespace scan, and scoped `git diff --check`. Full Cargo check/test validation remains deferred because current package checks are blocked before editor diagnostics by unrelated `zircon_runtime` render-history errors, and the active instruction is to implement functionality first.
