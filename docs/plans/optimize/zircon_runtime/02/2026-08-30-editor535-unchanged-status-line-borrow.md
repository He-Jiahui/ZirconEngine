---
title: Editor Unchanged Status Line Borrow 535
category: zircon_editor
report_id: Editor535-unchanged-status-line-borrow-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor Unchanged Status Line Borrow 535

Retained-host status publication previously cloned the complete message before the controller
acquired the shell lock and determined whether the value had changed. The controller now compares
a borrowed `&str` and owns the message only on the changed branch; the caller retains its original
message for workbench bridge patching. Invalidation, bridge refresh, and unchanged early-return
behavior are unchanged.

The ignored Release evidence `EDITOR535_UNCHANGED_STATUS_LINE_BORROW_BENCH_V1` models 32,768
unchanged status updates. The legacy pre-comparison path performs 32,768 string clones; the
borrowed comparison performs zero, a 100% reduction. This is an exact clone-operation model rather
than elapsed-time or allocator-byte evidence.

## Static evidence

- TDD RED: the static contract required a borrowed setter and borrowed call while production still
  used `message.clone()`.
- TDD GREEN: the retained setter accepts `&str`, compares under the existing shell lock, and calls
  `to_owned()` only after the value differs.
- The single retained production caller passes `&message`; all other public status APIs are
  unchanged.
- The previously transferred console-output consolidation and Editor534 progress-borrow contract
  in the same source were preserved.
- `rustfmt 1.94.1 --edition 2021` passes on both owned Editor sources.
- Scoped `git diff --check` passes with only the repository LF/CRLF notice.
- Source SHA-256:
  `zircon_editor/src/ui/host/editor_event_runtime_access/status.rs` =
  `65a903f5a7cfed60b6de54dfd7cf0c67990026c393b2852b04fe7540c7111bb9`.
- Source SHA-256:
  `zircon_editor/src/ui/retained_host/app/host_lifecycle/dispatch_effects/status.rs` =
  `71814ffefffc603af303979d75b12e96643043ec2a277d966788a12376120efc`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Editor tests pass.
2. Changed messages still update state and bridge projection; unchanged messages return early.
3. The ignored evidence emits the Editor535 marker with zero optimized pre-comparison clones.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
