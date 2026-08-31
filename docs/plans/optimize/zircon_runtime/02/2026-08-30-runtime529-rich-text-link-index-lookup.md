---
title: Runtime Rich Text Link Index Lookup 529
category: zircon_runtime
report_id: Runtime529-rich-text-link-index-lookup-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Rich Text Link Index Lookup 529

Rich-text pointer hit testing previously scanned every compiled link run after resolving caret
geometry. It now converts the caret offset and affinity into the same one-byte ownership range and
uses `CompiledRichText::run_for_range`, whose canonical run lookup is logarithmic. The selected run
still supplies the original link metadata, full source range, and caret affinity.

The ignored Release evidence `RUNTIME529_RICH_TEXT_LINK_INDEX_BENCH_V1` models 65,536 hit tests over
256 canonical runs. The deterministic model performs 16,777,216 legacy candidate checks versus a
589,824 comparison upper bound for indexed lookup, a 9,648 basis-point reduction. This is an
algorithmic candidate-check model, not an end-to-end UI latency claim.

## Static evidence

- TDD RED: `link_at_layout_point` still used `link_runs().find_map` and did not call the compiled
  run index.
- TDD GREEN: production uses `run_for_range` and contains no legacy link-run scan.
- An exhaustive small-domain comparison covers 4,896 non-empty range, offset, and affinity cases
  with zero differences between the old membership predicate and the indexed query range.
- Unit coverage includes upstream/downstream query ownership plus underflow and overflow edges.
- `rustfmt 1.94.1 --edition 2024 --check` passes.
- `git diff --check` passes (PowerShell reports the repository LF/CRLF notice).
- Source SHA-256:
  `bd5c00a65c5441281e0d74791e2547f3229e2a211c1f944f6ae457f7025a9099`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Runtime tests pass.
2. Existing rich-link boundary, gap, padding, and line-wrap regressions remain green.
3. The ignored evidence emits the Runtime529 marker and exact candidate-check model.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
