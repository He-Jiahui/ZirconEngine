---
title: Runtime Animation Target Short Segment Hash 559
category: zircon_runtime
report_id: Runtime559-animation-target-short-segment-hash-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Animation Target Short Segment Hash 559

Animation target derivation previously submitted each path segment's eight-byte length and UTF-8
body to BLAKE3 in separate calls. Segments whose complete framed representation fits in one 64-byte
hash block now use a fixed stack buffer and one update; longer segments retain the original
two-update path. Length encoding, segment order, namespace, digest truncation, and public IDs are
unchanged.

For the representative four-segment animation paths in the gate, segment update calls fall from 8
to 4, a 50% reduction; including the namespace update, total calls fall from 9 to 5. A standalone
Rust 1.94.1 `opt-level=3` benchmark linked the workspace's real `blake3` rlib, verified hashes for
empty, 56-byte boundary, 57-byte fallback, and longer segments, and used 13 interleaved samples.
P95 changed from 1.5912 s to 1.2781 s, a 19.68% improvement on this machine.

## Static evidence

- TDD RED: every segment used one length update followed by one content update.
- TDD GREEN: `update_segment_hash` frames short segments once and preserves the long fallback.
- A focused regression compares optimized IDs against the original byte stream at both boundaries.
- Focused tests use prefix `optimization_batch_20260830ev_runtime559_`.
- Ignored evidence marker: `RUNTIME559_SHORT_SEGMENT_HASH_BENCH_V1`.
- `rustfmt 1.94.1 --edition 2021` and scoped `git diff --check` pass.
- Production/test source SHA-256: `cfba31822b35f903593362e75e235f36dc67ad19d934c5c08ecb93fd090c689c`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Runtime tests pass.
2. Optimized and legacy IDs remain byte-identical for empty, short, boundary, and long segments.
3. Short segment framing stays stack-only and long segments retain the exact original byte stream.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
