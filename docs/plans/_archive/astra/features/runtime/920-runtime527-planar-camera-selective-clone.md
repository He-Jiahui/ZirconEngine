---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-runtime527-planar-camera-selective-clone.md
implementation_files:
  - zircon_runtime/src/core/framework/render/advanced_lighting/planar/derive_camera.rs
tests:
  - zircon_runtime/src/core/framework/render/advanced_lighting/planar/derive_camera.rs
---

# Runtime920 Runtime527 planar-camera selective clone

Planar reflection derivation now constructs only retained camera fields instead
of cloning and discarding the complete main-camera descriptor. Render, clear,
volume, transform, projection, and ownership overrides remain covered by the
focused regression.

Marker `RUNTIME527_PLANAR_CAMERA_SELECTIVE_CLONE_BENCH_V1` reports the removed
discarded stack-clone count. Managed Release validation remains pending.
