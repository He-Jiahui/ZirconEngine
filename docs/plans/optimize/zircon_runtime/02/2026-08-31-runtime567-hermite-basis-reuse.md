---
title: Runtime Animation Hermite Basis Reuse 567
category: zircon_runtime
report_id: Runtime567-animation-hermite-basis-reuse-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Animation Hermite Basis Reuse 567

Hermite Vec2/Vec3/Vec4 sampling previously recomputed `t²`, `t³`, and four basis coefficients for
every component. A sample now constructs one immutable `HermiteBasis` and reuses it across all
components. Each component retains the original arithmetic order; scalar sampling computes one
basis as before and quaternion interpolation does not construct a Hermite basis.

For Vec4, basis construction falls from four times per sample to one, a 75% operation-count
reduction for that shared work. A Rust 1.94.1 `opt-level=3` standalone benchmark used 13
interleaved pairs and 15,000,000 Vec4 samples per pair. P95 changed from `337,005,700 ns` to
`242,420,600 ns`, a `28.07%` reduction, with bit-identical outputs at five interpolation points.

## Static evidence

- TDD RED: vector sampling called `hermite_scalar` per component and rebuilt the basis each time.
- TDD GREEN: focused regression compares shared-basis Vec4 output against the legacy formula.
- Ignored benchmark marker: `RUNTIME567_HERMITE_BASIS_BENCH_V1`.
- Focused test prefix: `optimization_batch_20260831fb_runtime567_`.
- `rustfmt 1.94.1 --edition 2021` and scoped `git diff --check` pass.
- Production/test source SHA-256: `cf3a46ee7a9f45fbc38180fa3e3fe7ae87e8b81db926cd9a629e6170661796fc`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Runtime animation tests pass.
2. Shared-basis and legacy Vec2/3/4 samples remain numerically identical.
3. Managed ignored Vec4 benchmark retains at least a 15% P95 improvement.
4. Commit/push and WeCom performance publication remain coordinator-owned after accepted validation.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
