---
title: Editor Workbench Binding Key Single Buffer 574
category: zircon_editor
report_id: Editor574-workbench-binding-key-single-buffer-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor Workbench Binding Key Single Buffer 574

Workbench extension binding installation previously formatted every `view/control` key through the
general formatting machinery. Key construction now reserves the exact byte length and appends the
view, slash, and control segments directly into one buffer. Empty and normal segment behavior is
unchanged.

## Static and calibration evidence

- TDD coverage verifies a representative extension binding and the empty-segment edge case.
- Ignored benchmark marker: `EDITOR574_BINDING_KEY_SINGLE_BUFFER_BENCH_V1`.
- Focused test prefix: `optimization_batch_20260831gs_`.
- Performance gate: optimized P95 must be at least 15% below `format!` across 21 interleaved sample
  pairs and 500,000 keys per sample.
- A standalone Rust 1.94.1 `-O` calibration measured median ratio `0.2640`; this is preliminary and
  does not replace managed workspace Release evidence.
- Rust 1.94.1 `rustfmt --edition 2024 --check` and scoped `git diff --check` pass.
- Production/test source SHA-256: `4c49f1cb3b4443f313c934cbc59534410fc5994ab90ad8897e8dc281f890d914`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Editor574 tests pass.
2. Binding keys remain byte-identical for all view/control segments.
3. Managed ignored benchmark retains at least a 15% P95 reduction.
4. Commit/push and WeCom performance publication remain coordinator-owned after accepted
   validation.

No managed Cargo pass, commit, push, or WeCom success is claimed by this record.
