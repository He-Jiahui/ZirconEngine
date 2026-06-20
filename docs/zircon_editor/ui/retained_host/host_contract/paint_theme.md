---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme/tokens.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme/tokens.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor --check
  - paint-theme model/token ownership scan
  - scoped trailing whitespace scan
  - scoped git diff --check
doc_type: module-detail
---

# Paint Theme

`paint_theme.rs` is the retained-host software paint theme entry. It stays as a structural module that re-exports the palette model and the active palette tokens used by Workbench fallback paint, template-node drawing, diagnostics, and primitive replay.

`paint_theme/model.rs` owns the `HostMaterialPalette` data shape. `paint_theme/tokens.rs` owns the concrete `PALETTE` color values for the current editor shell.

The 2026-06-20 model/token split reduced `paint_theme.rs` from 57 lines to a 4-line structural entry. `model.rs` is 28 lines and owns the palette field schema; `tokens.rs` is 30 lines and owns the concrete RGBA values. Validation used `cargo fmt -p zircon_editor --check`, a root ownership scan confirming palette struct fields and token values no longer live in `paint_theme.rs`, a scoped trailing-whitespace scan, and scoped `git diff --check`. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction, and package-level Cargo check is still waiting on unrelated `zircon_runtime` render-history compile errors.
