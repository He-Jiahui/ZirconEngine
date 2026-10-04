---
title: Editor Notification Pipe Value Single Buffer
category: zircon_editor
report_id: Editor893-notification-pipe-value-single-buffer-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor893 Notification Pipe Value Single Buffer

## Finding

Notification history and toast-queue field escaping first mapped protocol
delimiters into an intermediate `String`, split that string into a temporary
borrowed-token vector, and joined the tokens into the required normalized field.
Every projected ID, title, message, and detail therefore created two avoidable
temporary owners around the final output.

## Optimization

- Allocate one output bounded by the original UTF-8 byte length.
- Scan characters once, treating `|`, `=`, CR/LF/tab, and every Unicode
  whitespace character as a pending separator.
- Emit one ASCII space only before the next visible character, preserving
  `split_whitespace().join(" ")` trimming and collapse semantics exactly.
- Preserve Unicode non-whitespace bytes and all notification projection owners.

## TDD and deterministic evidence

The combined Editor893/Runtime874 source-model batch was observed RED at `2/10`
and GREEN at `10/10`. Lower regressions compare empty/plain, leading/trailing,
protocol-delimiter, repeated-separator, Unicode-whitespace, CJK, and emoji text
against the retired map/collect/split/collect/join implementation.

Across 4,096 values with 64 visible tokens, the deterministic model changes
intermediate strings/reference slots from `4096/262144` to `0/0`; the required
normalized field remains. Ignored marker
`EDITOR893_NOTIFICATION_PIPE_VALUE_SINGLE_BUFFER_BENCH_V1` emits 101
alternating p50/p95/p99 sample pairs and requires the single-pass p95 to remain
within 10% of the retired chain.

## Local validation boundary

- Exact-file Rustfmt, Python bytecode compilation, and scoped
  `git diff --check` pass.
- Editor893/Runtime874 plus adjacent notification-center and Runtime toast
  contracts pass `101/101`.
- Editor893 received no per-task Cargo run and was submitted with Runtime874 in
  asynchronous v20 (PID `15628`) at `2026-09-21T22:24:02.7781628+08:00`.
- No v13-v20 receipt was read or monitored after submission.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/notifications.rs` | `060FA63F815A01FD0ABFD4E98347F16F5DB28AF5FBED0B28D6F6E40798AC805D` |
| `zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/notifications/pipe_value_single_buffer_tests.rs` | `36D666FB401899FD99B537C2F7933568B57B1D385F53D4D96C2B32A1C426CE82` |
| `tools/tests/test_editor893_notification_pipe_value_single_buffer_performance_contract.py` | `9F02ABFB998B5651B32A9BCEB6AADE12EAAB128554975B01077B891CCB953952` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
a combined current-source Windows lane compiles Editor, executes the lower
regression and ignored Release marker, and supplies allocator plus real
notification-projection p50/p95/p99 evidence. The deterministic allocation
model is not product acceptance.
