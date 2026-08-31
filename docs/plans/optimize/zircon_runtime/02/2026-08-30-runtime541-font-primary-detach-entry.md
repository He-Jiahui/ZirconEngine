---
title: Runtime Font Primary Detach Entry 541
category: zircon_runtime
report_id: Runtime541-font-primary-detach-entry-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Font Primary Detach Entry 541

Detaching a face's primary backend previously retained the alias vector, released that mutable
borrow, looked up the same vector again for its first survivor, then performed separate get and
insert traversals of the primary map. The retain pass now captures the first surviving backend in
the same borrow, and `HashMap::entry` conditionally replaces the primary in one traversal. Empty
face removal and non-primary alias detachment keep their existing behavior.

The ignored Release evidence `RUNTIME541_FONT_PRIMARY_DETACH_ENTRY_BENCH_V1` models 65,536
primary rebinds with a surviving alias. Post-retain hash lookups fall from 196,608 to 65,536, a
66.67% reduction. The initial reverse-map removal, retain scan, and empty-face cleanup are unchanged
and excluded from this count. This is deterministic lookup-count evidence, not elapsed-time or
whole-font-system evidence.

## Static evidence

- TDD RED: the structural test failed while detach re-read `backend_entries_by_face` and used
  `face_to_backend.get` followed by insert.
- TDD GREEN: the retain borrow returns `(remove_face, next_backend)`, the second alias-vector lookup
  is absent, and primary replacement uses `face_to_backend.entry(previous_face)`.
- Focused behavior moves the previous primary to a second face and proves the surviving alias is
  promoted for the first face.
- `rustfmt 1.94.1 --edition 2021` and scoped `git diff --check` pass.
- `zircon_runtime/src/text/font/backend.rs` SHA-256:
  `3e1321fefe9e5c4394f8d25bac61437ca489eff80eb61ec51170c575bf130f81`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Runtime tests pass.
2. Primary promotion, alias removal order, and forward/reverse mappings remain consistent.
3. The ignored evidence emits the Runtime541 marker and reports the 3-to-1 post-retain lookup gate.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
