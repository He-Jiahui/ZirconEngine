---
plan_sources:
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
related_code:
  - zircon_editor/assets/ui/**/*.zui
  - zircon_runtime/assets/ui/**/*.zui
  - zircon_editor/src/tests/ui/boundary/zui_asset_governance/slot.rs
  - zircon_editor/src/tests/ui/boundary/zui_asset_governance/workbench_shell.rs
  - zircon_editor/assets/ui/editor/components/workbench/shell/activity_drawer_window.zui
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
tests:
  - zui_asset_governance
  - global_material_surface_assets
  - runtime_ui_golden
  - bootstrap_layout
  - pane_body_documents
  - builtin_window_descriptors
  - tests::host::template_runtime
---

# Editor M1 ZUI Governance Hard Cutover Acceptance

## Scope

This record accepts only the Editor Layout 15 failure handoff for the Editor M1 ZUI governance group. It covers production ZUI asset identity, child-slot metadata, style-self schema, Activity Drawer ownership, and L4 shell structure. It does not accept the remaining Editor M1 full-suite failures or complete Unreal-level visual parity.

## Root Cause And Repair

The initial 68/71 result had three lowest-layer causes: 51 production asset IDs did not match file-derived `res://` locators, `child_mount.slot.layout` was consumed by runtime layout owners but rejected by governance, and viewport gizmo assets duplicated props-only fields in `style.self`. The locator cutover then exposed the old Activity Drawer host path, a missing L4 `Slot` structural classification, and 24 builtin runtime registry keys that still gave the same files a second logical identity.

All production asset IDs and builtin runtime registry keys now use their file locators. The alias table, resolution helpers, and alias-permitting test branches were deleted. Activity Drawer moved to `editor/components/workbench/shell` with all imports, fixtures, and assertions updated. Slot layout metadata is accepted only as a table across component and view documents, invalid gizmo style-self entries were removed, and L4 recognizes `Slot` as a component projection boundary. No compatibility alias, loader fallback, path exemption, or test-only bypass remains.

## Verification

- Production `.zui` static inventory: 244 assets, 0 locator mismatches.
- Old Activity Drawer host path absent; new shell path present.
- 24 old builtin registry IDs: 0 hits; alias table/helpers/branches: 0 hits.
- Activity Drawer exact regression: 1/1 passed.
- ZUI governance: 72/72 passed in 120.61s.
- Builtin window descriptors: 10/10 passed.
- Floating template layout route: 1/1 passed.
- Global material surface assets: 3/3 passed.
- Runtime UI golden: 3/3 passed.
- Bootstrap layout: 11/11 passed.
- Pane body documents: 11/11 passed.
- Scoped Rustfmt and `git diff --check`: passed.

Before the post-review hard-cut guard was added, the same-day Editor binary completed all 2928 library tests single-threaded: 2761 passed, 133 failed, and 34 ignored in 2497.76s. No governance test failed. The post-review binary passes all 72 governance tests and the direct-consumer groups above. Its broader template-runtime filter passes 48/50; the two failures reproduce existing dual-host attribute and style-override baselines.

`template_assets` passed 15/21; its six failures concern host invalidation, pointer slow-path rules, retained DTO ownership, and welcome routing. The integration-contract binary did not compile because `workbench_autolayout.rs` has four existing 7-versus-8 argument E0061 call sites. These are upward-gate evidence, not reasons to weaken or reopen the governance repair.

## Screenshot Placement

The refreshed component evidence remains under `docs/tests/editor`. Matching-name scans for the five refreshed PNGs found zero copies in the repository `target` directory and the external Cargo target used for validation.

## Decision

Accepted for the Editor M1 ZUI governance handoff. The handoff returns to Editor 01 as fixed; Editor M1 remains open until its independent full-suite and integration-contract failures are resolved.
