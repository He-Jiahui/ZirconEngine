---
title: Runtime Mesh SDF Source Hash Batching 569
category: zircon_runtime
report_id: Runtime569-mesh-sdf-source-hash-batching-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Mesh SDF Source Hash Batching 569

Mesh SDF source identity previously called `blake3::Hasher::update` three times per vertex and once
per index. The implementation now materializes the same canonical little-endian bytes into a fixed
3 KiB vertex-position buffer and a fixed 1 KiB index buffer, then updates the hasher once per block.
There is no heap allocation and the hash schema and byte order are unchanged.

At the benchmark scale of 16,384 vertices and 49,152 indices, hasher update calls fall from
`98,313` to `265`, a `99.73%` reduction. Real BLAKE3 wall-clock evidence remains pending because
the shared clean-copy Cargo batch is blocked on unrelated source provenance; this record does not
substitute the standalone segmentation-invariant compile harness for managed performance data.

## Static evidence

- TDD RED: the ignored benchmark compares the legacy per-scalar update path with production and
  requires at least a 25% P95 reduction.
- TDD GREEN: the focused regression covers empty input, 255/256/257 block boundaries, a partial
  tail, negative sign bits, and NaN payload bits against the legacy implementation.
- Standalone Rust 1.94.1 production-source compile/test: `1 passed; 0 failed; 1 ignored`.
- Ignored benchmark marker: `RUNTIME569_MESH_SDF_HASH_BATCH_BENCH_V1`.
- Focused test prefix: `optimization_batch_20260831fd_runtime569_`.
- `rustfmt 1.94.1 --edition 2021` and scoped `git diff --check` pass.
- Production/test source SHA-256: `feee59317418237d329d290c8cc9087171284f9a4902df96e5af9063cf52cf07`.

## Acceptance gates

1. Managed Windows native Release compilation and focused mesh SDF hash tests pass.
2. Managed tests retain byte-identical BLAKE3 hashes across block boundaries and special float
   bit patterns.
3. Managed ignored BLAKE3 benchmark retains at least a 25% P95 improvement.
4. Commit/push and WeCom performance publication remain coordinator-owned after accepted
   validation.

No direct Cargo validation, commit, push, or WeCom success is claimed by this record.
