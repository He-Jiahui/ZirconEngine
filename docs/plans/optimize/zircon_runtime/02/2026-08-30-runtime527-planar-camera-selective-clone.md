---
title: Runtime Planar Camera Selective Clone 527
category: zircon_runtime
report_id: Runtime527-planar-camera-selective-clone-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Planar Camera Selective Clone 527

Planar reflection camera derivation previously cloned the complete main-camera descriptor and then
discarded its entity, stack, target, and culling mask. Derivation now constructs the reflection
descriptor from only the fields that survive, while preserving render type, viewport, clear state,
volume mask, camera state, render-order adjustment, probe culling mask, and reflection projection.

The ignored Release evidence `RUNTIME527_PLANAR_CAMERA_SELECTIVE_CLONE_BENCH_V1` models 65,536
derivations from a camera with 32 stacked entities. The old full clone performs 2,097,152 discarded
stack-entity clones; selective construction performs zero, a 10,000 basis-point reduction for this
discarded-stack operation. This is a deterministic clone-count model, not an end-to-end render-time
claim.

## Static evidence

- TDD RED: reflection derivation still started from `main_camera.clone()`.
- TDD GREEN: derivation constructs `CameraRenderDescriptor` and clones only retained owned fields.
- The behavior regression verifies retained render/clear/volume state and overridden ownership,
  stack, target, culling, transform, and projection behavior.
- `rustfmt +1.94.1 --edition 2024 --check` passes.
- `git diff --check` passes (PowerShell reports the repository LF/CRLF notice).
- Source SHA-256:
  `0b0bdc8d9d005dec956a0209fa701ed0d5d4d5c34543b950b14ca4c3b3a24356`.
- Behavior-test SHA-256:
  `490e4a503b3aa342137a48bb1698ec4d3c47f5b899dabc692c2d45e9cf32ecab`.

## Acceptance gates

1. Managed Windows native Release compilation and the focused Runtime tests pass.
2. The ignored evidence emits the Runtime527 marker and exact clone-count model.
3. Reflection-camera descriptor semantics remain unchanged for every retained and overridden field.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
