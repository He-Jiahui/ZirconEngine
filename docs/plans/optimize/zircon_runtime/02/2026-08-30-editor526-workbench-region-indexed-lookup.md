---
title: Editor Workbench Region Indexed Lookup 526
category: zircon_editor
report_id: Editor526-workbench-region-indexed-lookup-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor Workbench Region Indexed Lookup 526

The canonical workbench skeleton stores its six `EditorRegion` bindings in enum order, but every
region query previously scanned the vector. Lookup now checks the enum's expected slot first and
retains the original linear search as a compatibility fallback for reordered serialized payloads.
A focused regression swaps the first and last regions and verifies both remain resolvable.

The ignored Release evidence `EDITOR526_WORKBENCH_REGION_INDEXED_LOOKUP_BENCH_V1` executes 65,536
canonical lookups cycling all six regions. The deterministic model performs 229,372 legacy
candidate checks versus 65,536 expected-slot checks, a 7,142 basis-point candidate-check reduction.
The reordered-payload fallback is intentionally outside this canonical fast-path model; this is
not an end-to-end workbench layout-time claim.

## Static evidence

- TDD RED: `WorkbenchSkeleton::region` only contained a linear iterator lookup.
- TDD GREEN: canonical data checks `expected_region_index` before the preserved linear fallback.
- The focused regression preserves lookups after a serialized-order permutation.
- `rustfmt +1.94.1 --edition 2024 --check` passes.
- `git diff --check` passes (PowerShell reports the repository LF/CRLF notice).
- Source SHA-256:
  `cf15e1aefa6a803073b0150d0db654002a65f84a965a0053d63d7bd8973c767e`.

## Acceptance gates

1. Managed Windows native Release compilation and the focused Editor tests pass.
2. The ignored evidence emits the Editor526 marker and exact candidate-check model.
3. Canonical and reordered skeleton lookups return the same region binding as before.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
