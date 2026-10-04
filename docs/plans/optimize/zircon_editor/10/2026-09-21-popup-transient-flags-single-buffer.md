---
title: Editor Popup Transient Flags Single Buffer
category: zircon_editor
report_id: Editor894-popup-transient-flags-single-buffer-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor894 Popup Transient Flags Single Buffer

## Finding and optimization

Workbench popup refresh used a borrowed flag vector, `join(",")`, and a
formatted outer row for each menu item even though the original row already
bounded the final UTF-8 output. `menu_item_without_transient_flags` now
streams surviving flags into one `String::with_capacity(raw.len())`, appending
the trimmed label and optional shortcut directly. It retains the case-insensitive
transient filter, flag order, whitespace trimming, `---` separator, and the
empty flag field required when a shortcut remains but no persistent flag does.
The checked-state operation remains a separate owner and was not changed.

## TDD and bounded evidence

The combined Editor894/Runtime875 source-model contracts were RED `2/11` and
GREEN `11/11`. A lower Rust regression compares nine edge-case rows with the
retired implementation. For 4,096 rows containing 32 flags, the deterministic
model removes up to `131072` borrowed vector slots and `4096` joined child
strings; the required final row remains. Ignored
`EDITOR894_POPUP_TRANSIENT_FLAGS_SINGLE_BUFFER_BENCH_V1` records 101 alternating
Release p50/p95/p99 pairs and requires optimized p95 <= 110% of the retired
implementation; it has not yet been run.

The pair and adjacent popup, visual, live-state, notification, and diagnostic
contracts pass `57/57`. Exact Rustfmt, Python bytecode compilation, and scoped
diff checks pass. No per-task Cargo run was made. The combined current-source
Runtime→Editor→App Windows validation v21 was launched as PID `34372` at
`2026-09-21T22:39:27.9275627+08:00`; no live receipt was read or monitored.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/popup_primitives.rs` | `8AB211B0F02EE262A0DF126AA72C8FD75A69D02EF570A1A3AFF7E6E01902C560` |
| `zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/popup_primitives/transient_flags_single_buffer_tests.rs` | `B8D94DBD6FA1017AFDDD06076206C4A9B5DE84D587699BCD7BE87296B1B8A4C3` |
| `tools/tests/test_editor894_popup_transient_flags_single_buffer_performance_contract.py` | `92B4FDD072B90CA91752402A01F0D4AF6D1D1D4A8A7A3474F9787B46775D757B` |

## Acceptance boundary

The one-time v21 receipt exposed an unrelated-to-this-function Runtime871
`E0596` double-borrow error; Runtime and Editor builds failed before validating
this lower regression. The owned Runtime871 source has been repaired and the
combined current-source Runtime→Editor→App matrix was resubmitted as v22 (PID
`35456`) at `2026-09-21T22:51:32.5348309+08:00`. No v22 receipt was read.

Implementation and static allocation-model targets are met, but current-source
Windows compilation, lower regression, ignored Release marker, allocator
measurement, and popup-refresh product p50/p95/p99 gates are pending. Do not
claim performance acceptance from this model or the asynchronous launch alone.
