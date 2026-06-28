---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/helpers.rs
  - zircon_editor/src/ui/retained_host/app/helpers/animation_assets.rs
  - zircon_editor/src/ui/retained_host/app/helpers/animation_assets/clip.rs
  - zircon_editor/src/ui/retained_host/app/helpers/animation_assets/clip/channels.rs
  - zircon_editor/src/ui/retained_host/app/helpers/animation_assets/clip/tracks.rs
  - zircon_editor/src/ui/retained_host/app/helpers/animation_assets/paths.rs
  - zircon_editor/src/ui/retained_host/app/helpers/animation_assets/skeleton.rs
  - zircon_editor/src/ui/retained_host/app/helpers/animation_assets/tests.rs
  - zircon_editor/src/ui/retained_host/app/helpers/asset_visibility.rs
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface.rs
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface/source_window.rs
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface/source_window/focus.rs
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface/source_window/resolution.rs
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface/surface_size.rs
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface/surface_size/asset_surface.rs
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface/surface_size/frame.rs
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface/surface_size/host_frames.rs
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface/surface_size/host_frames/source_window.rs
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface/surface_size/host_frames/workbench.rs
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface/surface_size/workbench_regions.rs
  - zircon_editor/src/ui/retained_host/app/helpers/geometry.rs
  - zircon_editor/src/ui/retained_host/app/helpers/model_staging.rs
  - zircon_editor/src/ui/retained_host/app/assets.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/helpers.rs
  - zircon_editor/src/ui/retained_host/app/helpers/animation_assets.rs
  - zircon_editor/src/ui/retained_host/app/helpers/animation_assets/clip.rs
  - zircon_editor/src/ui/retained_host/app/helpers/animation_assets/clip/channels.rs
  - zircon_editor/src/ui/retained_host/app/helpers/animation_assets/clip/tracks.rs
  - zircon_editor/src/ui/retained_host/app/helpers/animation_assets/paths.rs
  - zircon_editor/src/ui/retained_host/app/helpers/animation_assets/skeleton.rs
  - zircon_editor/src/ui/retained_host/app/helpers/animation_assets/tests.rs
  - zircon_editor/src/ui/retained_host/app/helpers/asset_visibility.rs
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface.rs
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface/source_window.rs
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface/source_window/focus.rs
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface/source_window/resolution.rs
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface/surface_size.rs
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface/surface_size/asset_surface.rs
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface/surface_size/frame.rs
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface/surface_size/host_frames.rs
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface/surface_size/host_frames/source_window.rs
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface/surface_size/host_frames/workbench.rs
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface/surface_size/workbench_regions.rs
  - zircon_editor/src/ui/retained_host/app/helpers/geometry.rs
  - zircon_editor/src/ui/retained_host/app/helpers/model_staging.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app helpers animation-assets ownership scan
  - app helpers animation-assets subowner ownership scan
  - app helpers animation clip subowner ownership scan
  - app helpers callback-surface ownership scan
  - app helpers callback-surface source/size ownership scan
  - app helpers callback-source window focus/resolution subowner ownership scan
  - app helpers callback-surface surface-size subowner ownership scan
  - app helpers callback-surface host-frame source/workbench subowner ownership scan
  - app helpers visibility/geometry/model-staging ownership scan
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Retained Host App Helpers

`app/helpers.rs` owns the structural retained-host app helper entry and re-exports focused helper families shared across app callback modules.

It should not become a catch-all for feature implementations. When a helper grows into a workflow with its own data structures, file IO, parsing, or tests, it belongs in a child module.

## Animation Asset Derivation

`app/helpers/animation_assets.rs` owns derived animation asset generation for glTF/glb model sources. It imports glTF data, derives a sibling `.skeleton.zranim` from the first skin, derives one `.clip.zranim` per animation, maps keyframe interpolation and channels into runtime animation assets, writes generated asset bytes, and keeps the regression coverage for stable sibling file naming plus project asset-id preservation across reimport.

`helpers.rs` re-exports `derive_animation_assets_from_model_source(...)` for the asset import flow in `app/assets.rs`. The animation root remains the glTF import and sibling asset orchestration entry; `animation_assets/skeleton.rs` owns skin/joint parent mapping and derived skeleton records, `animation_assets/clip.rs` owns glTF animation channel traversal and final clip asset assembly, `animation_assets/clip/channels.rs` owns channel interpolation plus constant/sample key conversion, `animation_assets/clip/tracks.rs` owns default skeleton-track seeding and final bone-track conversion, `animation_assets/paths.rs` owns sibling path/name sanitization and generated file writes, and `animation_assets/tests.rs` owns the glTF fixture plus reimport regressions.

## Callback Surface Context

`app/helpers/callback_surface.rs` is the structural entry for retained-host callback source window context and callback surface sizing.

`app/helpers/callback_surface/source_window.rs` is the structural callback source-window entry. It separates native floating-window source id resolution from callback-origin focus tracking.

`app/helpers/callback_surface/source_window/resolution.rs` owns native floating-window callback source-id resolution and the callback source-window tests.

`app/helpers/callback_surface/source_window/focus.rs` owns callback-source scoping through `with_callback_source_window(...)`, callback-origin floating-window focus, and last-focused callback-window tracking.

`app/helpers/callback_surface/surface_size.rs` owns callback surface-size fallback dispatch. It preserves the public app-boundary helpers that resolve callback sizes from explicit callback dimensions, cached sizes, and host-frame fallbacks.

`surface_size/frame.rs` owns positive-size validation and frame-to-size conversion. `surface_size/workbench_regions.rs` owns drawer/main-page matching for `ViewContentKind`. `surface_size/host_frames.rs` is the structural host-frame fallback entry. `surface_size/host_frames/source_window.rs` owns callback-source floating/native window fallback size lookup. `surface_size/host_frames/workbench.rs` owns workbench drawer, document, viewport, and root-pane host-frame fallback lookup. `surface_size/asset_surface.rs` owns asset-surface mode to `ViewContentKind` mapping.

The child module keeps this host-state-aware behavior out of the generic helper root while preserving the app boundary. Its methods use `pub(in crate::ui::retained_host::app)` so retained-host action owners can reuse the helpers without exporting callback focus or size resolution outside the app module.

## Small Helper Families

`app/helpers/asset_visibility.rs` owns active workbench/drawer inspection for asset-surface visibility. `app/helpers/geometry.rs` owns viewport-size rounding, window-menu popup height, and shell region group-key helpers. `app/helpers/model_staging.rs` owns project-asset model staging plus resource-locator normalization shared with animation asset derivation.

## Boundary Rules

- Keep `app/helpers.rs` as the structural helper entry and re-export surface.
- Keep active workbench/drawer asset-surface visibility in `app/helpers/asset_visibility.rs`.
- Keep viewport-size rounding, window-menu sizing, and shell-region group keys in `app/helpers/geometry.rs`.
- Keep model source staging and helper-family resource locator normalization in `app/helpers/model_staging.rs`.
- Keep glTF import and derived asset orchestration in `app/helpers/animation_assets.rs`.
- Keep skeleton derivation in `app/helpers/animation_assets/skeleton.rs`.
- Keep glTF animation traversal and final clip assembly in `app/helpers/animation_assets/clip.rs`.
- Keep channel interpolation and constant/sample key conversion in `app/helpers/animation_assets/clip/channels.rs`.
- Keep default skeleton-track seeding and final bone-track conversion in `app/helpers/animation_assets/clip/tracks.rs`.
- Keep generated path/write helpers in `app/helpers/animation_assets/paths.rs`, and animation derivation regressions in `app/helpers/animation_assets/tests.rs`.
- Keep `app/helpers/callback_surface.rs` as the structural callback-surface helper entry.
- Keep `app/helpers/callback_surface/source_window.rs` as the structural callback source-window entry.
- Keep callback source-window attribution and source-id tests in `app/helpers/callback_surface/source_window/resolution.rs`.
- Keep callback-source scoping and floating-window focus tracking in `app/helpers/callback_surface/source_window/focus.rs`.
- Keep callback surface-size fallback dispatch in `app/helpers/callback_surface/surface_size.rs`.
- Keep frame size validation/conversion in `app/helpers/callback_surface/surface_size/frame.rs`.
- Keep drawer/main-page content-kind matching in `app/helpers/callback_surface/surface_size/workbench_regions.rs`.
- Keep `app/helpers/callback_surface/surface_size/host_frames.rs` as the structural host-frame fallback entry.
- Keep callback-source floating/native window fallback size lookup in `app/helpers/callback_surface/surface_size/host_frames/source_window.rs`.
- Keep workbench drawer, document, viewport, and root-pane fallback lookup in `app/helpers/callback_surface/surface_size/host_frames/workbench.rs`.
- Keep asset-surface mode mapping in `app/helpers/callback_surface/surface_size/asset_surface.rs`.
- Do not add feature workflow state, file generation, parser logic, or large tests directly to `helpers.rs`; split a child module by feature domain.

## Validation Notes

The 2026-06-18 animation asset helper split reduced `helpers.rs` from 1061 lines to 453 lines. `helpers/animation_assets.rs` is 627 lines and owns the full derived animation asset workflow plus the moved animation regression fixtures. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app helpers animation-assets ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only.

The 2026-06-18 callback-surface split reduced `helpers.rs` from 402 lines to 139 lines. `helpers/callback_surface.rs` is 274 lines and owns callback source-window tests plus the retained-host callback focus and size fallback methods. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app helpers callback-surface ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 visibility/geometry/model-staging helper split reduced `helpers.rs` from 152 lines to 13 lines. `helpers/asset_visibility.rs` is 51 lines, `helpers/geometry.rs` is 32 lines, and `helpers/model_staging.rs` is 68 lines. `model_staging.rs` keeps `asset_uri_from_relative_path(...)` helper-family internal so `animation_assets.rs` can continue to share locator normalization without exporting it outside helpers. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app helpers visibility/geometry/model-staging ownership scan, and scoped `git diff --check`; scoped diff check only reported the existing CRLF working-tree conversion warning. The follow-up owner-split batch cargo check caught the moved `active_workspace_tab(...)` import in `helpers/callback_surface.rs`; making the helper `pub(super)` from `asset_visibility.rs` and importing that path fixed it. `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never` then passed with existing warning noise only (`zircon_runtime` 142 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 animation asset subowner split reduced `helpers/animation_assets.rs` from 627 lines to 74 lines. The new child owners are `animation_assets/clip.rs` at 168 lines, `animation_assets/skeleton.rs` at 83 lines, `animation_assets/paths.rs` at 37 lines, and `animation_assets/tests.rs` at 224 lines. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app helpers animation-assets subowner ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 142 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 animation clip subowner split reduced `helpers/animation_assets/clip.rs` from 168 lines to 64 lines. `animation_assets/clip/channels.rs` is 82 lines and owns interpolation mapping, constant Vec3/quaternion channels, sampled Vec3 channels, sampled quaternion channels, and key-count mismatch diagnostics. `animation_assets/clip/tracks.rs` is 42 lines and owns derived clip-track defaults plus final `AnimationClipBoneTrackAsset` conversion.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app helpers animation clip subowner ownership scan, and scoped `git diff --check`, all of which passed except for existing CRLF conversion warnings in the dirty worktree. Focused `cargo check` was not rerun for this slice because independent `zircon_runtime` Cargo test processes were still active; full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 callback-surface source/size split reduced `helpers/callback_surface.rs` from 274 lines to 3 lines. `helpers/callback_surface/source_window.rs` is 94 lines and owns source-window resolution, focus scoping/tracking, and callback source tests. `helpers/callback_surface/surface_size.rs` is 184 lines and owns callback-surface size fallback across floating, native, drawer, document, viewport, and asset surfaces.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app helpers callback-surface source/size ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 142 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 callback-source window focus/resolution subowner split reduced `helpers/callback_surface/source_window.rs` from 107 lines to a 4-line structural entry. `source_window/focus.rs` is 65 lines and owns callback-source scoping plus floating-window focus tracking. `source_window/resolution.rs` is 43 lines and owns native floating-window source-id resolution plus source-window tests.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app helpers callback-source window focus/resolution subowner ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 callback-surface surface-size subowner split reduced `helpers/callback_surface/surface_size.rs` from 184 lines to 39 lines. New child owners are `surface_size/frame.rs` (13 lines), `surface_size/workbench_regions.rs` (65 lines), `surface_size/host_frames.rs` (87 lines), and `surface_size/asset_surface.rs` (8 lines). The root no longer owns drawer/main-page matching, floating/native frame lookup, or frame-size conversion.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app helpers callback-surface surface-size subowner ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 143 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 callback-surface host-frame source/workbench subowner split reduced `helpers/callback_surface/surface_size/host_frames.rs` from 95 lines to a 19-line structural fallback entry. `host_frames/source_window.rs` is 42 lines and owns callback-source floating/native window fallback size lookup. `host_frames/workbench.rs` is 55 lines and owns drawer, document, viewport, and root-pane fallback lookup.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app helpers callback-surface host-frame source/workbench subowner ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-27 animation asset error-boundary follow-up keeps runtime animation serialization errors typed in the runtime asset layer while preserving the existing editor helper API shape. `helpers/animation_assets.rs` now converts `AnimationAssetError` from `to_bytes()` into `String` only at the existing `Result<_, String>` boundary, which unblocked the editor package build used by the Workbench content-panel surface slice. Validation for that slice used `cargo fmt -p zircon_editor --check` and `cargo build -q -p zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0627-content`, with build output outside the repository `target` directory.
