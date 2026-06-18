---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/helpers.rs
  - zircon_editor/src/ui/retained_host/app/helpers/animation_assets.rs
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface.rs
  - zircon_editor/src/ui/retained_host/app/assets.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/helpers.rs
  - zircon_editor/src/ui/retained_host/app/helpers/animation_assets.rs
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app helpers animation-assets ownership scan
  - app helpers callback-surface ownership scan
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Retained Host App Helpers

`app/helpers.rs` owns small retained-host app helpers that are shared across app callback modules: visible asset-surface detection, window menu popup sizing, shell region group keys, model source staging, and resource locator normalization for project asset paths.

It should not become a catch-all for feature implementations. When a helper grows into a workflow with its own data structures, file IO, parsing, or tests, it belongs in a child module.

## Animation Asset Derivation

`app/helpers/animation_assets.rs` owns derived animation asset generation for glTF/glb model sources. It imports glTF data, derives a sibling `.skeleton.zranim` from the first skin, derives one `.clip.zranim` per animation, maps keyframe interpolation and channels into runtime animation assets, writes generated asset bytes, and keeps the regression coverage for stable sibling file naming plus project asset-id preservation across reimport.

`helpers.rs` re-exports `derive_animation_assets_from_model_source(...)` for the asset import flow in `app/assets.rs`, while the animation-specific structs, channel builders, glTF traversal, and test fixtures stay inside the child module.

## Callback Surface Context

`app/helpers/callback_surface.rs` owns retained-host callback source window context and callback surface sizing. It resolves native floating-window callback source ids, scopes callback execution through `with_callback_source_window(...)`, focuses callback-origin floating windows, tracks the last focused callback window, resolves docked/floating/native fallback frames for pointer callbacks, and exposes app-internal helpers for ViewContentKind and asset-surface size fallback.

The child module keeps this host-state-aware behavior out of the generic helper root while preserving the app boundary. Its methods use `pub(in crate::ui::retained_host::app)` so retained-host action owners can reuse the helpers without exporting callback focus or size resolution outside the app module.

## Boundary Rules

- Keep small geometry, visibility, window-menu sizing, shell-region keys, and model staging helpers in `app/helpers.rs`.
- Keep glTF animation parsing, derived skeleton/clip construction, generated animation file writes, and animation derivation tests in `app/helpers/animation_assets.rs`.
- Keep callback source-window attribution, floating-window focus tracking, and host-frame-backed callback surface-size fallback in `app/helpers/callback_surface.rs`.
- Do not add feature workflow state, file generation, parser logic, or large tests directly to `helpers.rs`; split a child module by feature domain.

## Validation Notes

The 2026-06-18 animation asset helper split reduced `helpers.rs` from 1061 lines to 453 lines. `helpers/animation_assets.rs` is 627 lines and owns the full derived animation asset workflow plus the moved animation regression fixtures. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app helpers animation-assets ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only.

The 2026-06-18 callback-surface split reduced `helpers.rs` from 402 lines to 139 lines. `helpers/callback_surface.rs` is 274 lines and owns callback source-window tests plus the retained-host callback focus and size fallback methods. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app helpers callback-surface ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
