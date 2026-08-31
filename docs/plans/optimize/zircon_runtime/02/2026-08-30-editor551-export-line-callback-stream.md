---
title: Editor Export Line Callback Stream 551
category: zircon_editor
report_id: Editor551-export-line-callback-stream-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor Export Line Callback Stream 551

Incremental export output parsing previously collected every chunk's decoded lines into a temporary
`Vec<String>` before the capture path retained its bounded tail and emitted events. The parser now
delivers each decoded line directly through a callback. The test-only collecting adapter preserves
the old shape for boundary regression coverage, while the production path no longer allocates a
temporary line collection per chunk. Line decoding, order, length limits, tail retention, hashing,
and durable log writes are unchanged.

The ignored Release evidence `EDITOR551_CALLBACK_LINE_STREAM_BENCH_V1` models 65,536 three-line
chunks. Temporary collection allocations fall from 65,536 to zero, a 100% reduction; the 196,608
required line-string allocations remain unchanged. A standalone Rust 1.94.1 `opt-level=3` check
used 400,000 chunks and 1,200,000 decoded lines per sample; the 11-sample median changed from
230.2981 ms to 161.2486 ms, a 29.98% improvement on this machine. File I/O, hashing, bounded-tail
cloning, and event-channel work are outside that elapsed result.

## Static evidence

- TDD RED: the structural gate found the production `push` collection loop and no callback stream.
- TDD GREEN: production calls `for_each_line`; only the `cfg(test)` adapter collects a Vec.
- Focused behavior preserves line order across split chunks and finalization.
- Existing maximum-length and single-byte chunk tests continue to exercise the collecting adapter.
- `rustfmt 1.94.1 --edition 2021` and scoped `git diff --check` pass.
- `zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution/output_capture.rs`
  SHA-256: `bc8d96d7b379324b170efc8d7d924cbcb3e7bf61af9eb3b02b07788d7fbbb0d3`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Editor tests pass.
2. Split chunks, final lines, CRLF trimming, and maximum line length preserve existing output.
3. Ignored evidence emits the Editor551 marker and reports zero production temporary Vecs.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
