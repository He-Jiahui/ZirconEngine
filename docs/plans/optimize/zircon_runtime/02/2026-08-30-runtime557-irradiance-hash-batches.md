---
title: Runtime Irradiance Hash Batches 557
category: zircon_runtime
report_id: Runtime557-irradiance-hash-batches-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Irradiance Hash Batches 557

The irradiance-cubemap content signature previously called `blake3::Hasher::update` once per color
channel. It now encodes 64 texels at a time into a fixed 768-byte stack buffer and submits each
contiguous batch. Channel order, `to_bits`, little-endian encoding, face-size prefix, and final
digest projection are unchanged.

For a 32 by 32 six-face irradiance cube, texel-channel update calls fall from 18,432 to 96, a
99.48% reduction. A standalone Rust 1.94.1 `opt-level=3` benchmark linked the workspace's real
`blake3` rlib, compared both hashes for equality, and used 11-sample medians. The path changed from
3.7693 ms to 190.1 us, a 94.96% improvement on this machine. Irradiance convolution is excluded.

## Static evidence

- TDD RED: content hashing invoked `Hasher::update` inside the per-channel loop.
- TDD GREEN: the content hash routes through `update_irradiance_texel_hash` in 64-texel batches.
- A focused regression compares the batched digest with the original byte-stream implementation.
- Focused tests use prefix `optimization_batch_20260830eu_runtime557_`.
- Ignored evidence marker: `RUNTIME557_IRRADIANCE_HASH_BATCH_BENCH_V1`.
- `rustfmt 1.94.1 --edition 2021` and scoped `git diff --check` pass.
- Production/test source SHA-256: `7c859e43e4f60c70306bf0dce76de09b4170a732e9a9ce555c0177a35063b937`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Runtime tests pass.
2. Batched and legacy hashing remain identical for partial and full batches.
3. Content hash stability remains independent of batching boundaries.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
