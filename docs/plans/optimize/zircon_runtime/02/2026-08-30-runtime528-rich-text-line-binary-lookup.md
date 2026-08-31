---
title: Runtime Rich Text Line Binary Lookup 528
category: zircon_runtime
report_id: Runtime528-rich-text-line-binary-lookup-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Rich Text Line Binary Lookup 528

Rich-text rendering previously scanned every resolved layout line for every paint run before it
could validate source-isomorphic fallback provenance. Canonical layouts now locate the source-range
candidate with binary search, then retain the exact visual-range/text predicate. The original
linear search remains a semantic fallback for reordered, duplicate-range, or externally decoded
payloads; a focused regression verifies a deliberately reordered layout.

The ignored Release evidence `RUNTIME528_TEXT_LINE_RANGE_BINARY_LOOKUP_BENCH_V1` models 65,536 run
lookups cycling 256 resolved lines. The deterministic model performs 8,421,376 legacy candidate
checks versus a 589,824 comparison upper bound for binary lookup, a 9,299 basis-point reduction.
Fallback scans for noncanonical payloads are intentionally outside the canonical fast-path model;
this is not an end-to-end rich-text render-time claim.

## Static evidence

- TDD RED: `source_isomorphic_text_paint_line` still scanned `layout.lines` for every run.
- TDD GREEN: range binary search validates the same full predicate before the preserved fallback.
- The behavior regression resolves the correct line from a deliberately reordered payload.
- `rustfmt +1.94.1 --edition 2024 --check` passes.
- `git diff --check` passes (PowerShell reports the repository LF/CRLF notice).
- Source SHA-256:
  `21f3999cb46ffd8ce70a0ac6a36fd6615b8b63077a85e3921192fb1f83b5be14`.

## Acceptance gates

1. Managed Windows native Release compilation and the focused Runtime tests pass.
2. The ignored evidence emits the Runtime528 marker and exact candidate-check model.
3. Canonical, reordered, and duplicate-range layout payloads preserve the previous match semantics.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
