---
related_code:
  - zircon_editor/src/ui/animation_editor/session.rs
  - zircon_editor/src/ui/animation_editor/session/lifecycle.rs
  - zircon_editor/src/ui/animation_editor/session/support.rs
  - zircon_editor/src/ui/animation_editor/session/timeline_foundation.rs
  - zircon_editor/src/ui/animation_editor/session/curve_foundation.rs
  - zircon_editor/src/ui/timeline/mod.rs
  - zircon_editor/src/ui/curve/mod.rs
  - zircon_editor/src/ui/preview_scene/mod.rs
plan_sources:
  - docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/engine-code-structure-convention.md
doc_type: module-detail
---

# Animation Editor Session

`session.rs` is the structural entry for animation editor session state, DTO projection, and focused session errors. It keeps the public `AnimationEditorSessionError(pub String)` boundary stable for editor callers while the runtime animation asset layer owns typed serialization and parsing errors.

`session/lifecycle.rs` owns session load/save transitions. It reads animation skeleton and clip bytes through the runtime asset APIs, converts asset payloads into editor-session state, and maps runtime `AnimationAssetError` values into the editor session error boundary at the call site.

`session/support.rs` owns byte extraction and shared support routines used by the lifecycle path. It converts `to_bytes()` failures with `error.to_string()` only when returning through `AnimationEditorSessionError`, so typed runtime errors do not leak into unrelated editor UI APIs and are not flattened earlier than the current public boundary requires.

`session/timeline_foundation.rs` projects a sequence session into the shared `ui::timeline` model and the shared `PreviewPlayback` value. It reads the runtime-owned sequence asset, editor-local visible frame range, current frame, and playback settings without creating a second sequence document. Track ids derive from `AnimationTrackPath`; key ids derive from that path plus the authored key time bit pattern, which is stable while the key remains at that time. Channel value shapes map to common lanes (`bool`, scalar/vector curves, discrete keys, or untyped) according to runtime evaluation semantics.

`session/curve_foundation.rs` projects only the current timeline-selected sequence track into shared scalar curve views. It expands scalar/vector channel values into one curve per component and carries authored Hermite tangents through unchanged. Integer, bool, and quaternion channels deliberately remain outside this scalar curve path because their runtime behavior is discrete or orientation-aware rather than scalar interpolation.

## Boundary Rules

- Keep runtime animation asset parse/serialization error types in `zircon_runtime`.
- Keep `AnimationEditorSessionError(pub String)` as the editor session boundary until the editor API is intentionally migrated to typed errors.
- Do not add compatibility shims, root facade error wrappers, or broad `Result<_, String>` public surfaces when the focused `AnimationEditorSessionError` boundary is available.
- Keep load/save lifecycle orchestration in `session/lifecycle.rs` and byte/error support in `session/support.rs`.
- Request `timeline_foundation()` only for a visible shared timeline surface. Do not call it from the generic pane-summary path: it intentionally projects key rows and therefore must not replace the lightweight track-label summary used during normal host recomputation.
- Request `curve_foundation()` only after a timeline track is selected. It materializes the selected track's curve components and must not expand every sequence track during generic host recomputation.
- Keep preview-session creation and runtime asset resolution out of this session. The future Editor04 backend consumes the projected `PreviewPlayback` through `ui::preview_scene::PreviewSceneBackend`.

## Validation Notes

The 2026-06-27 Workbench content-panel slice exposed an editor package build failure after runtime animation asset APIs returned typed `AnimationAssetError` values. The fix kept the typed errors in runtime and converted them explicitly at the existing editor boundaries in `session/lifecycle.rs`, `session/support.rs`, and `retained_host/app/helpers/animation_assets.rs`.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a focused content-panel style regression, direct compiled `template_style` test execution, and `cargo build -q -p zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0627-content`.
