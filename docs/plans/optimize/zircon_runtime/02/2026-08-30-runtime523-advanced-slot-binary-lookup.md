---
title: Runtime Advanced Render Feature Slot Lookup 523
category: zircon_runtime
report_id: Runtime523-advanced-slot-binary-lookup-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Advanced Render Feature Slot Lookup 523

The descriptor-only advanced render-feature table is ordered by `BuiltinRenderFeature`, but every
query previously scanned all 22 slots. Lookup now uses `binary_search_by_key`; the sorted-table
regression fixes the ordering invariant, checks every built-in feature against the legacy result,
and preserves `None` for features outside this catalog.

The ignored Release evidence
`RUNTIME523_ADVANCED_SLOT_BINARY_LOOKUP_BENCH_V1` executes 65,536 lookups across all 41 built-in
features. Its deterministic model performs 1,072,576 legacy candidate checks; a 22-entry binary
search requires at most 393,216 comparisons, establishing a minimum 6,333 basis-point comparison
reduction. This is a comparison-count model, not an end-to-end frame-time claim.

## Static evidence

- TDD RED: the production lookup still used `.iter().find` and had no binary search.
- TDD GREEN: the production lookup contains `binary_search_by_key` and no linear lookup.
- The focused regression fixes strict catalog ordering and legacy result equivalence.
- `rustfmt +1.94.1 --edition 2024 --check` passes.
- `git diff --check` passes (PowerShell reports the repository LF/CRLF notice).
- Source SHA-256:
  `870d11055b52bfbaa7b30ccf72e2a13ebc2ad17cf9307efe91d9704c2dbffb67`.

## Acceptance gates

1. Managed Windows native Release compilation and the focused Runtime tests pass.
2. The ignored evidence emits the Runtime523 marker and exact comparator counts.
3. Catalog ordering, hit results, and miss behavior remain unchanged.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
