---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_geometry/frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_geometry/pixel_rect.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_geometry/rect_ops.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_debug_reflector_overlay/draw.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/paint_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_geometry/frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_geometry/pixel_rect.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_geometry/rect_ops.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-20 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor --check
  - host_contract paint-geometry frame/rect/pixel ownership scan
  - scoped trailing whitespace scan
  - scoped git diff --check
  - milestone testing stage: cargo check/test deferred until runtime render-history errors are resolved
doc_type: module-detail
---

# Paint Geometry

`paint_geometry.rs` is the retained-host software-paint geometry boundary. It now stays as a structural re-export entry for common frame, rectangle, and pixel-clip helpers used by paint primitives, retained text, debug overlays, and Workbench painters.

`paint_geometry/frame.rs` owns visible-frame filtering, fallback frame selection, and conversion from template-node frame DTOs into `FrameRect`.

`paint_geometry/rect_ops.rs` owns rectangle transforms: translation by origin, intersection with visibility checks, and inset shrink/expand math.

`paint_geometry/pixel_rect.rs` owns conversion from floating-point `FrameRect` plus an optional clip into integer pixel bounds. This is the gateway used by primitive image, line, shape, and text-raster paths before they mutate RGBA backbuffers.

## Validation Notes

This slice is implementation-first. Formatting, root ownership scans, scoped whitespace scans, and scoped diff checks cover the handoff. Full Cargo check/test validation remains deferred to the milestone testing stage because current package checks fail in unrelated `zircon_runtime` render-history code before editor diagnostics are reached.
