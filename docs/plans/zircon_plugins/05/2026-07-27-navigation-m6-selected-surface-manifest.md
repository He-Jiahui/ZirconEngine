---
related_code:
  - zircon_plugins/navigation/editor/bake.zui
  - zircon_plugins/navigation/editor/src/tests/bake_panel_retained.rs
implementation_files:
  - zircon_plugins/navigation/editor/bake.zui
  - zircon_plugins/navigation/editor/src/tests/bake_panel_retained.rs
plan_sources:
  - docs/plans/zircon_plugins/05-navigation.md
tests:
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_plugin_navigation_editor -SkipBuild -VerboseOutput
doc_type: milestone-detail
---

# 2026-07-27 Navigation M6 selected-surface validation manifest

Plan: docs/plans/zircon_plugins/05-navigation.md
Milestone: M6
Status: pending_validation
Files: ["zircon_plugins/navigation/editor/bake.zui", "zircon_plugins/navigation/editor/src/tests/bake_panel_retained.rs", "docs/plans/zircon_plugins/05/2026-07-27-navigation-m6-selected-surface-manifest.md"]
Date: 2026-07-27

## Scope Delivered

- The Bake Selected and Clear Selected template bindings project the Navigation panel's stable `surface_entity`; Bake Selected also projects `force_full_rebuild`.
- The surface table starts unselected and does not submit its display index as an entity identifier.

## Fresh Testing Evidence

- The source guard, `rustfmt --check`, and scoped `git diff --check` pass.
- Managed Windows package validation is pending coordinator admission.

## Review

- Pending focused package validation and independent review; this record does not accept M6 or close the open selected-surface handoff.
