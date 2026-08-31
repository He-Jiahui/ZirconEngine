---
title: Runtime Bake Artifact Hash Header Batch 563
category: zircon_runtime
report_id: Runtime563-bake-artifact-hash-header-batch-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Bake Artifact Hash Header Batch 563

The source-cubemap bake artifact signature previously submitted every scalar descriptor field and
each of the 16 bake-key words to BLAKE3 separately. It now encodes the unchanged little-endian
field stream into one fixed 108-byte stack header and submits that header once before the payload.
Field order, integer widths, payload bytes, digest projection, and the separate PMREM hash remain
unchanged.

Fixed-header update calls fall from 25 to 1, a 96% reduction. A standalone Rust 1.94.1
`opt-level=3` benchmark linked the workspace's real `blake3` rlib, verified complete digest
equality with a 4 KiB payload, and used 21 interleaved samples. P95 changed from 1.9814 s to
1.1534 s for 65,536 hashes, a 41.79% improvement. A shorter 13-sample confirmation changed from
958.738 ms to 569.693 ms, a 40.58% improvement.

## Static evidence

- TDD RED: production performed 25 fixed-header `Hasher::update` calls before payload hashing.
- TDD GREEN: `bake_artifact_payload_hash_header` emits one exact fixed header update.
- A focused regression compares the optimized digest with the original per-field byte stream.
- Focused tests use prefix `optimization_batch_20260830ex_runtime563_`.
- Ignored evidence marker: `RUNTIME563_ARTIFACT_HASH_HEADER_BATCH_BENCH_V1`.
- `rustfmt 1.94.1 --edition 2021` and scoped `git diff --check` pass.
- Production/test source SHA-256: `aa38755eafb8d2ba7b5a578317dd37d5eacb5184729fd0736b35125cc3bee9d6`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Runtime tests pass.
2. Optimized and legacy digests remain exactly equal for the complete descriptor and payload.
3. The fixed header remains stack-only, exactly 108 bytes, and is submitted in one update.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
