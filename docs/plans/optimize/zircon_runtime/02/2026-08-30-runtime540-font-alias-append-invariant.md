---
title: Runtime Font Alias Append Invariant 540
category: zircon_runtime
report_id: Runtime540-font-alias-append-invariant-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Font Alias Append Invariant 540

`BackendFaceMap::insert_alias` first detaches the backend ID from every prior face entry, then
looked for that same backend in the selected face vector before appending it. Under the map's sole
mutation API that membership test is always false. Release builds now append directly; debug builds
retain a `debug_assert!` that checks the invariant. Rebinding an existing alias still removes and
re-appends one entry, preserving stable alias order without duplicates.

The ignored Release evidence `RUNTIME540_FONT_ALIAS_APPEND_INVARIANT_BENCH_V1` models 65,536
rebinds on a 64-alias face. The legacy post-detach `contains` performs 4,128,768 comparisons; the
Release append path performs zero, a 100% reduction. The preceding `retain` scan remains and is not
counted as removed. This is exact comparison-count evidence for the local path, not elapsed-time or
font-system throughput evidence.

## Static evidence

- TDD RED: the structural test failed while `insert_alias` still guarded append with
  `if !entries.contains(&backend)`.
- TDD GREEN: the Release path contains no conditional membership scan and the debug invariant is
  explicit.
- Focused behavior rebinds the same alias twice and proves `remove_face` returns it exactly once.
- `rustfmt 1.94.1 --edition 2021` and scoped `git diff --check` pass.
- `zircon_runtime/src/text/font/backend.rs` SHA-256:
  `3e1321fefe9e5c4394f8d25bac61437ca489eff80eb61ec51170c575bf130f81`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Runtime tests pass.
2. Repeated alias rebinds retain one forward/reverse entry and stable removal order.
3. The ignored evidence emits the Runtime540 marker with zero optimized contains comparisons.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
