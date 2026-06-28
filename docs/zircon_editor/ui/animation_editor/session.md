---
related_code:
  - zircon_editor/src/ui/animation_editor/session.rs
  - zircon_editor/src/ui/animation_editor/session/lifecycle.rs
  - zircon_editor/src/ui/animation_editor/session/support.rs
plan_sources:
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/engine-code-structure-convention.md
doc_type: module-detail
---

# Animation Editor Session

`session.rs` is the structural entry for animation editor session state, DTO projection, and focused session errors. It keeps the public `AnimationEditorSessionError(pub String)` boundary stable for editor callers while the runtime animation asset layer owns typed serialization and parsing errors.

`session/lifecycle.rs` owns session load/save transitions. It reads animation skeleton and clip bytes through the runtime asset APIs, converts asset payloads into editor-session state, and maps runtime `AnimationAssetError` values into the editor session error boundary at the call site.

`session/support.rs` owns byte extraction and shared support routines used by the lifecycle path. It converts `to_bytes()` failures with `error.to_string()` only when returning through `AnimationEditorSessionError`, so typed runtime errors do not leak into unrelated editor UI APIs and are not flattened earlier than the current public boundary requires.

## Boundary Rules

- Keep runtime animation asset parse/serialization error types in `zircon_runtime`.
- Keep `AnimationEditorSessionError(pub String)` as the editor session boundary until the editor API is intentionally migrated to typed errors.
- Do not add compatibility shims, root facade error wrappers, or broad `Result<_, String>` surfaces when a focused call-site conversion is enough.
- Keep load/save lifecycle orchestration in `session/lifecycle.rs` and byte/error support in `session/support.rs`.

## Validation Notes

The 2026-06-27 Workbench content-panel slice exposed an editor package build failure after runtime animation asset APIs returned typed `AnimationAssetError` values. The fix kept the typed errors in runtime and converted them explicitly at the existing editor boundaries in `session/lifecycle.rs`, `session/support.rs`, and `retained_host/app/helpers/animation_assets.rs`.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a focused content-panel style regression, direct compiled `template_style` test execution, and `cargo build -q -p zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0627-content`.
