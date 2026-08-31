---
title: Editor Asset Projection Full Reuse 581
category: zircon_editor
report_id: Editor581-asset-projection-full-reuse-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor Asset Projection Full Reuse 581

Asset workspace projection already reused unchanged item chunks individually, but an entirely
unchanged source generation still allocated a new chunk array and copied the selected-index array.
The aligned all-chunks-unchanged path now returns a generation that shares both projected arrays by
`Arc`, while partial changes continue through the existing chunk-level projection path.

## Static evidence

- Regression prefix: `optimization_batch_gz_editor581_`.
- Ignored benchmark marker: `EDITOR581_PROJECT_REUSE_BENCH_V1`.
- Performance gate: optimized P95 must be at most 70% of the legacy copy path across 17
  interleaved Release samples over 8,192 items.
- Rust 1.94.1 `rustfmt --edition 2024 --check` and scoped `git diff --check` pass.
- Production/test source SHA-256:
  `a5674de4c9747567669de5ae44f071b6d163b95004d104b6d35f25f1dcf35e49`.
- Performance ticket `323c4011b701478384384a5cf3cf00fb` and aggregate behavior ticket
  `6bebe849e7c24feaa38c3eecab138148` are queued; no terminal result is claimed.

## Acceptance gates

1. Managed Windows native Release compilation and focused Editor581 tests pass.
2. Full reuse shares projected chunks and selected indices; changed chunks retain existing behavior.
3. Managed ignored benchmark satisfies the 70% P95 gate.
4. Commit/push and WeCom publication remain coordinator-owned after accepted validation.

No managed Cargo pass, performance result, commit, push, or WeCom success is claimed by this
record.
