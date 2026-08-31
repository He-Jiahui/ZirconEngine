---
title: Runtime Indirect Args Single Traversal 552
category: zircon_runtime
report_id: Runtime552-indirect-args-single-traversal-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Indirect Args Single Traversal 552

The compiled-scene render path previously traversed the selected indirect-draw index vector once
to encode source-buffer copies and again to assign execution-owned argument slices. It now computes
the execution offset once and performs both operations in one ordered traversal. A draw without a
source buffer still skips only the copy and receives its execution-owned slice, preserving the
existing assignment contract.

For 65,536 selected draws, index-vector visits and execution-offset calculations both fall from
131,072 to 65,536, a 50% reduction. A standalone Rust 1.94.1 `opt-level=3` benchmark used 262,144
draw records, selected two thirds of them, and took the 11-sample median. The modeled loop changed
from 2.7762 ms to 1.8882 ms, a 31.99% improvement on this machine. WGPU command encoding and GPU
execution are excluded.

## Static evidence

- TDD RED: production contained both `iter().copied()` and `into_iter()` selected-index loops.
- TDD GREEN: production contains one consuming selected-index traversal and no borrowed loop.
- Existing order tests preserve opaque-before-transparent submission and mesh order.
- The new focused test is `optimization_batch_20260830es_runtime552_copies_and_assigns_in_one_index_traversal`.
- Ignored evidence marker: `RUNTIME552_SINGLE_INDEX_TRAVERSAL_BENCH_V1`.
- `rustfmt 1.94.1 --edition 2021` and scoped `git diff --check` pass.
- Production/test source SHA-256: `e1060d13d3bed641a8585fee35c9716f8d698758c22152ec99c7837717259a67`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Runtime tests pass.
2. Copy commands retain selected-draw order, source offsets, and fixed five-word stride.
3. Every selected draw receives its execution-owned slice even when it has no source buffer.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
