---
title: Editor Functional Window Indexed Lookup 527
category: zircon_editor
report_id: Editor527-functional-window-indexed-lookup-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor Functional Window Indexed Lookup 527

The canonical Unreal-style window preset stores its eight functional windows in enum order, but
every query previously scanned the vector. Lookup now checks the enum's expected slot first and
retains the original linear search as a compatibility fallback for reordered serialized or custom
payloads. A focused regression swaps the first and last windows and verifies both still resolve.

The ignored Release evidence `EDITOR527_FUNCTIONAL_WINDOW_INDEXED_LOOKUP_BENCH_V1` executes 65,536
canonical lookups cycling all eight window kinds. The deterministic model performs 294,912 legacy
candidate checks versus 65,536 expected-slot checks, a 7,777 basis-point candidate-check reduction.
The reordered-payload fallback is intentionally outside this canonical fast-path model; this is not
an end-to-end editor startup-time claim.

## Static evidence

- TDD RED: `UnrealWindowModelPreset::window` only contained a linear iterator lookup.
- TDD GREEN: canonical data checks `expected_functional_window_index` before the preserved fallback.
- The focused regression preserves lookups after a serialized-order permutation.
- `rustfmt +1.94.1 --edition 2024 --check` passes.
- `git diff --check` passes (PowerShell reports the repository LF/CRLF notice).
- Source SHA-256:
  `e31f756ad6f3f5d08dd51c98555b6b250d331dbcff0a405a290bfec8f6b06f14`.

## Acceptance gates

1. Managed Windows native Release compilation and the focused Editor tests pass.
2. The ignored evidence emits the Editor527 marker and exact candidate-check model.
3. Canonical and reordered preset lookups return the same functional window as before.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
