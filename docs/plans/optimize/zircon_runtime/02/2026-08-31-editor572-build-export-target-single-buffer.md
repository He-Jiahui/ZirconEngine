---
title: Editor Build Export Target Single Buffer 572
category: zircon_editor
report_id: Editor572-build-export-target-single-buffer-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor Build Export Target Single Buffer 572

Duplicate-platform export target IDs previously allocated a normalized profile `String` and then
allocated again through `format!`. Normalization now appends directly into one pre-sized target
buffer. The public key builder uses the same append helper, preserving ASCII lowercasing,
separator runs, trailing separator removal, and the `target` fallback.

## Static evidence

- TDD RED: focused compatibility tests cover standalone key normalization, append-to-existing
  behavior, trailing separator trimming, and the empty-key fallback.
- TDD GREEN: duplicate target construction owns one capacity-planned buffer and does not construct
  an intermediate profile key.
- Ignored benchmark marker: `EDITOR572_BUILD_EXPORT_TARGET_SINGLE_BUFFER_BENCH_V1`.
- Focused test prefix: `optimization_batch_20260831gq_`.
- Performance gate: optimized P95 must be at least 25% below the two-allocation path across 21
  interleaved sample pairs and 200,000 iterations per sample.
- Rust 1.94.1 `rustfmt --edition 2024 --check` and scoped `git diff --check` pass.
- Production/test source SHA-256: `2a1c1eb473c834e3759eca43499b7fb287152a7d34875e6036cfbd08fce742f6`.

## Acceptance gates

1. Managed Windows native Release compilation and the focused Editor572 tests pass.
2. Standalone and appended normalized keys remain byte-identical to the legacy behavior.
3. Managed ignored benchmark retains at least a 25% P95 reduction.
4. Commit/push and WeCom performance publication remain coordinator-owned after accepted
   validation.

No direct Cargo validation, performance pass, commit, push, or WeCom success is claimed by this
record.
