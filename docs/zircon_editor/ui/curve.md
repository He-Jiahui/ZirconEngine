---
related_code:
  - zircon_editor/src/ui/curve/mod.rs
  - zircon_editor/src/ui/curve/model.rs
  - zircon_editor/src/ui/curve/canvas.rs
  - zircon_editor/src/ui/animation_editor/session/curve_foundation.rs
plan_sources:
  - docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
doc_type: module-detail
---

# Curve Foundation

`ui::curve` is the renderer-neutral vocabulary shared by animation and future inspector curve
surfaces. It contains scalar curve views, keys with optional in/out tangents, key/tangent-aware
selection, and curve-to-screen transforms. It does not own or clone an animation asset.

## Boundary Rules

- `CurveModel` returns a domain-owned reversible delta; the foundation never writes animation
  channels or inspector fields directly.
- Runtime animation remains the sole authority for Step, Linear, Hermite, and quaternion
  evaluation. The foundation displays authored keys and tangents only; it must not introduce a
  second interpolation implementation.
- A selected vector track projects all of its scalar components. Scalar and vector channels are
  curve-editable; bool and integer channels stay discrete, while quaternion channels remain
  keyframe-only until an orientation-aware editor is designed.
- Curve selection identifies `curve_id + key_id + element kind`, so moving a key and dragging one
  of its tangents cannot collide in a multi-component view.
- `CurveCanvasTransform` treats value as upward-positive and pixel Y as downward-positive. It
  guards zero-size viewports and degenerate ranges without adding UI state to the domain model.

## Performance Notes

Animation session projection only materializes curves for the current timeline-selected track.
This follows the Fyrox track-selection model and avoids allocating component curves for every
unselected animation track during host presentation recomputation.
