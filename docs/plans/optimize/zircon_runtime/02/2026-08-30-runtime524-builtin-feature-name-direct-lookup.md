---
title: Runtime Builtin Render Feature Name Lookup 524
category: zircon_runtime
report_id: Runtime524-builtin-feature-name-direct-lookup-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Builtin Render Feature Name Lookup 524

Renderer data-document parsing previously resolved each built-in render-feature name by scanning
all 41 variants and recomputing their authoring names. The parser now uses one exhaustive string
match. A round-trip regression covers every stable authoring name and preserves rejection of empty
and unknown names.

The ignored Release evidence
`RUNTIME524_BUILTIN_FEATURE_NAME_LOOKUP_BENCH_V1` executes 65,536 valid lookups across all 41
features. The same workload performs 1,376,049 legacy candidate checks versus 65,536 direct lookup
calls, a 9,523 basis-point candidate-check reduction model. This is an algorithmic source model,
not an end-to-end renderer-data load-time claim.

## Static evidence

- TDD RED: `from_authoring_name` still scanned `BuiltinRenderFeature::ALL`.
- TDD GREEN: the function contains 41 direct arms and no `ALL` scan.
- The focused regression round-trips all authoring names and checks unknown-name rejection.
- `rustfmt +1.94.1 --edition 2024 --check` passes.
- `git diff --check` passes (PowerShell reports the repository LF/CRLF notice).
- Source SHA-256:
  `397b50e3d87af074b2cdadddfdf5c795815fbb04e16fd5a246f5cf54305d982d`.

## Acceptance gates

1. Managed Windows native Release compilation and the focused Runtime tests pass.
2. The ignored evidence emits the Runtime524 marker and exact candidate-check model.
3. Every stable authoring name round-trips and unknown names still return `None`.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
