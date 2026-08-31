---
title: Runtime82 Keyboard Payload Byte Control Scan
category: zircon_runtime
report_id: Runtime82-keyboard-payload-byte-control-scan-2026-08-30
date: 2026-08-30
session_id: root-runtime82-two-task-keyboard-performance-batch-20260830
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime82 Keyboard Payload Byte Control Scan

## Scope

This slice accelerates the printable keyboard-text admission check. Product keyboard payloads are
usually ASCII, but the previous owner decoded every byte through the Unicode scalar iterator only
to reject C0/DEL controls. The new helper scans ASCII payloads as bytes and preserves the exact
`char::is_control` path for any non-ASCII text.

Empty text, modifier and Tab rejection, pressed/repeated state policy, borrowed payload ownership,
Unicode input, and every control-character result remain unchanged. No string is cloned or
allocated by either path.

## Performance Workload

The ignored release gate scans one 65,536-byte printable ASCII payload 256 times per sample using
17 alternating legacy/byte sample pairs.

| Work per sample | Before | After |
|---|---:|---:|
| Unicode scalar scan passes | 256 | 0 on ASCII |
| ASCII byte scan passes | 0 | 256 |
| Unicode fallback changes | 0 | 0 |
| Payload allocations | 0 | 0 |

The marker `RUNTIME82_KEYBOARD_PAYLOAD_BYTE_CONTROL_SCAN_BENCH_V1` includes nearest-rank P50/P95,
workload identity, order balance, and checksum. Acceptance requires byte-scan P95 to be at least
30% below the legacy scalar path.

## Acceptance

- `runtime82_batch_keyboard_payload_byte_control_scan_preserves_results` covers printable ASCII,
  C0, DEL, C1, CJK, emoji, and empty input parity.
- `runtime82_batch_keyboard_payload_uses_ascii_bytes_with_unicode_fallback` locks the byte fast path
  and the unchanged Unicode fallback.
- `runtime82_batch_keyboard_payload_byte_control_scan_p95` verifies checksum parity and the 30% P95
  reduction gate.
- The managed `runtime82_batch_` release gate seals this work with inline keyboard edit actions in
  one Cargo invocation: two source contracts, seven Rust tests, and two performance rows. Dynamic
  Windows values, commit attribution, and WeCom publication remain pending the coordinator result.

## Remaining Parent-plan Work

Runtime82 still owns retained document storage, IME/clipboard transactions, secure input policy,
revision-bound snapshots, and product-scale editing qualification. This slice only optimizes the
existing keyboard payload control-character admission.
