---
title: Runtime IBL Front Hit 548
category: zircon_runtime
report_id: Runtime548-ibl-front-hit-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime IBL Front Hit 548

The environment IBL hydration cache uses a four-entry LRU queue. Its dominant steady-state lookup
finds the current request at index zero, but the old path still removed that entry and pushed it
back to the front on every frame. The cache now clones the already-front environment directly and
keeps the existing remove-and-promote behavior for non-front hits.

The ignored evidence `RUNTIME548_IBL_FRONT_HIT_BENCH_V1` models 65,536 front hits. Queue mutations
fall from 131,072 to zero, a 100% reduction. A standalone Rust 1.94.1 `opt-level=3` benchmark used
eight million front hits per sample with four entries and Arc-backed payloads; the 11-sample median
changed from 276.870 ms to 99.894 ms, a 63.92% improvement on this machine. IBL decoding, upload,
and GPU work are excluded.

## Static evidence

- TDD RED: `get` had no front-hit branch before `VecDeque::remove`.
- TDD GREEN: index zero returns from `front()` before any LRU queue mutation.
- Existing reuse and bounded-LRU tests preserve runtime-control refresh and non-front promotion.
- `rustfmt 1.94.1 --edition 2021` and scoped `git diff --check` pass.
- Production/test source SHA-256: `898db611ff017909d71a39345eb6bbee6f99c3328dd119f3e2f9ae31d9e071cf`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Runtime tests pass.
2. Front hits preserve payload sharing and current intensity/rotation projection.
3. Non-front hits still promote, capacity remains four, and least-recent eviction is unchanged.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
