---
title: Editor Callback Source Window Owner Move 525
category: zircon_editor
report_id: Editor525-callback-source-window-replace-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor Callback Source Window Owner Move 525

Every retained-host callback previously cloned the current optional `MainPageId` before installing
the callback source window. The scope now moves the previous owner out with `std::mem::replace` and
restores it after the callback, preserving nested callback and restoration semantics without
copying the backing string.

The ignored Release evidence `EDITOR525_CALLBACK_SOURCE_WINDOW_REPLACE_BENCH_V1` executes 65,536
save/restore rounds with a 57-byte previous ID. The deterministic model removes all 65,536 previous
ID clones and 3,735,552 cloned ID bytes, a 10,000 basis-point clone-count reduction. This is an
allocation/copy model, not an end-to-end UI frame-time claim.

## Static evidence

- TDD RED: the callback scope still cloned `callback_source_window` before reassignment.
- TDD GREEN: the scope uses `std::mem::replace` and contains no previous-owner clone.
- Existing callback behavior retains the same callback-visible source and restores its prior owner.
- `rustfmt +1.94.1 --edition 2024 --check` passes.
- `git diff --check` passes (PowerShell reports the repository LF/CRLF notice).
- Source SHA-256:
  `f1cf50fbc63419bdd8e0c90eabb1e8fbc0dab69adc3bebcc222c8c3a9f7ac742`.

## Acceptance gates

1. Managed Windows native Release compilation and the focused Editor tests pass.
2. The ignored evidence emits the Editor525 marker and exact clone/copy model.
3. Nested callback source visibility and previous-owner restoration remain unchanged.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
