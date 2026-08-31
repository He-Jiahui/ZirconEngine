---
title: Runtime Animation Channel Projection Single Dispatch 568
category: zircon_runtime
report_id: Runtime568-animation-channel-projection-single-dispatch-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Animation Channel Projection Single Dispatch 568

Animation sequence projection previously matched every channel value once to validate finite
components, optionally again for Quaternion normalization, and again to construct the scene
property value. Validation and projection now share one top-level enum dispatch. Vector components
are scanned once; Quaternion finite and normalization checks stay in its branch with the same
typed errors and sample-kind labels.

A Rust 1.94.1 `opt-level=3` standalone benchmark used 13 interleaved pairs and 20,000,000 mixed
Scalar/Vec3/Vec4/Quaternion projections per sample. P95 changed from `562,773,600 ns` to
`385,800,800 ns`, a `31.45%` reduction.

## Static evidence

- TDD RED: production used separate finite-validation and projection matches.
- TDD GREEN: focused regression compares all value variants with the legacy projection and checks
  non-finite Vec3 plus zero-length Quaternion errors.
- Ignored benchmark marker: `RUNTIME568_CHANNEL_PROJECTION_BENCH_V1`.
- Focused test prefix: `optimization_batch_20260831fc_runtime568_`.
- `rustfmt 1.94.1 --edition 2021` and scoped `git diff --check` pass.
- Production/test source SHA-256: `c5c0f12bc35ff4d634e1e5da62bc22a7f2bb038b34ae6d422e4b746f03824997`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Runtime animation tests pass.
2. All supported values and typed validation failures match the legacy projection.
3. Managed ignored mixed-value benchmark retains at least a 15% P95 improvement.
4. Commit/push and WeCom performance publication remain coordinator-owned after accepted validation.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
