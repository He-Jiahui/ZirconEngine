---
title: Runtime OBJ Face Component Single Scan 572
category: zircon_runtime
report_id: Runtime572-obj-face-component-single-scan-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime OBJ Face Component Single Scan 572

OBJ face vertex parsing previously advanced a `split('/')` iterator three times for every face
vertex. The parser now performs one byte traversal, records at most the first three separators, and
returns borrowed position, UV, and normal slices. Empty UV/normal fields and the legacy behavior of
ignoring components after the normal field remain unchanged.

## Static evidence

- TDD RED: a focused compatibility test covers `v`, `v/vt`, `v//vn`, `v/vt/vn`, and the existing
  extra-component behavior before the production parser consumes the helper.
- TDD GREEN: production parsing and the benchmark share the same single-traversal helper.
- Ignored benchmark marker: `RUNTIME572_OBJ_FACE_COMPONENT_SINGLE_SCAN_BENCH_V1`.
- Focused test prefix: `optimization_batch_20260831gq_`.
- Performance gate: optimized P95 must be at least 5% below the three-step split traversal across
  21 interleaved sample pairs and 250,000 iterations per sample.
- Rust 1.94.1 `rustfmt --edition 2024 --check` and scoped `git diff --check` pass.
- Production/test source SHA-256: `7f329627b6ed9a5eedaa4191bd23c73f78257dc06c5dc30343b309ead454d652`.

## Acceptance gates

1. Managed Windows native Release compilation and the focused Runtime572 tests pass.
2. All supported and legacy face-token component forms preserve their borrowed slices.
3. Managed ignored benchmark retains at least a 5% P95 reduction.
4. Commit/push and WeCom performance publication remain coordinator-owned after accepted
   validation.

No direct Cargo validation, performance pass, commit, push, or WeCom success is claimed by this
record.
