---
title: Editor Settings Path Single Buffer
category: zircon_editor
report_id: Editor892-settings-path-single-buffer-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor892 Settings Path Single Buffer

## Finding

Settings-window projection converted every borrowed `Arc<str>` category segment
to `&str`, collected those references into a temporary vector, and joined the
vector with `/`. Category key and localized-label paths repeat this work across
categories, settings, selected-category identity, and extension pages.

## Optimization

- Sum borrowed segment byte lengths and positional separator count without
  materializing child storage.
- Allocate the final path at its exact size and append each `Arc<str>` directly.
- Preserve empty input, empty authored segments, leading/trailing/consecutive
  separators, segment order, Unicode bytes, and all settings projection owners.

## TDD and deterministic evidence

The combined Editor892/Runtime873 source-model batch was observed RED at `2/10`
and GREEN at `10/10`. Lower regressions compare empty, one-segment,
empty-segment, uneven-length, and Unicode paths against the retired
collect/join implementation and require capacity to equal final byte length.

Across 4,096 paths with 64 segments, the deterministic model changes temporary
borrowed-reference slots from `262144` to `0`; the required path output remains.
Ignored marker `EDITOR892_SETTINGS_PATH_SINGLE_BUFFER_BENCH_V1` emits 101
alternating p50/p95/p99 sample pairs and requires direct append p95 to remain
within 10% of collect/join.

## Local validation boundary

- Exact-file Rustfmt, Python bytecode compilation, and scoped
  `git diff --check` pass.
- Editor892/Runtime873 plus adjacent settings-window and ZUI contracts pass
  `140/140`.
- Editor892 received no per-task Cargo run and was submitted with Runtime873 in
  asynchronous v19 (PID `32536`) at `2026-09-21T22:14:16.6877626+08:00`.
- No v13-v19 receipt was read or monitored after submission.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/settings_window.rs` | `5FA79D738D96FC088045DB9F2B22EF1526DC25EB3AAEC7D67A39AA74FE2614EB` |
| `zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/settings_window/path_single_buffer_tests.rs` | `530FBD520A1C9AFDCCFD9B94E1EA090CFBB300085E36334C76DD12418F984B99` |
| `tools/tests/test_editor892_settings_path_single_buffer_performance_contract.py` | `892F0DB16F5576E3E65360C40A5F5A78D3797BE67C85933984EAF871B3021419` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
a combined current-source Windows lane compiles Editor, executes the lower
regression and ignored Release marker, and supplies allocator plus real
settings-window projection p50/p95/p99 evidence. The deterministic allocation
model is not product acceptance.
