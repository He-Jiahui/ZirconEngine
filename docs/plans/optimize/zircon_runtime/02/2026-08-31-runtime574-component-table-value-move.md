---
title: Runtime Component Table Value Move 574
category: zircon_runtime
report_id: Runtime574-component-table-value-move-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Component Table Value Move 574

Editor showcase layout builders previously created owned `toml::Value` entries, borrowed them as a
slice, and cloned every value while collecting the table. The helper now accepts fixed arrays by
value and moves each `Value` into the table. Layout keys and serialized values remain unchanged.

## Static and calibration evidence

- TDD coverage verifies the four-field scrollable layout table byte-for-value.
- Ignored benchmark marker: `RUNTIME574_TABLE_VALUE_MOVE_BENCH_V1`.
- Focused test prefix: `optimization_batch_20260831gs_`.
- Performance gate: optimized P95 must be at least 15% below the cloned-value path across 21
  interleaved sample pairs and 120,000 table constructions per sample.
- An ownership-equivalent Rust 1.94.1 `-O` calibration measured median ratio `0.8035`; this is
  preliminary and does not replace managed workspace Release evidence.
- Rust 1.94.1 `rustfmt --edition 2024 --check` and scoped `git diff --check` pass.
- Production/test source SHA-256: `7c35cc8fbb677608105733d0099b0cf1daa2723685d684ac97b27b16b321b358`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Runtime574 tests pass.
2. Layout table keys and values remain identical while production contains no clone step.
3. Managed ignored benchmark retains at least a 15% P95 reduction.
4. Commit/push and WeCom performance publication remain coordinator-owned after accepted
   validation.

No managed Cargo pass, commit, push, or WeCom success is claimed by this record.
