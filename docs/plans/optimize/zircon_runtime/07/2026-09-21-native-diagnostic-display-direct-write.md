---
title: Runtime Native Diagnostic Display Direct Write
category: zircon_runtime
report_id: Runtime874-native-diagnostic-display-direct-write-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime874 Native Diagnostic Display Direct Write

## Finding

The typed native-plugin behavior error implemented `Display` by joining its
owned diagnostic vector into a temporary `String` and then copying that child
into the caller's formatter. Formatting an already-owned diagnostic set thus
allocated a redundant full-detail buffer.

## Optimization

- Borrow the first diagnostic and write it directly to the provided formatter.
- Append later diagnostics with positional `; ` separators and no child owner.
- Preserve empty, singleton, empty-authored, ordered, Unicode, label/status
  ownership, typed-error propagation, and exact display text.

## TDD and deterministic evidence

The combined Editor893/Runtime874 source-model batch was observed RED at `2/10`
and GREEN at `10/10`. Lower regressions compare empty, singleton, empty-authored,
multi-item, CJK, and emoji diagnostics against the retired join/write path.

Across 4,096 displays, the deterministic model changes temporary join outputs
from `4096` to `0`; the owned diagnostics and caller-provided destination remain.
Ignored marker `RUNTIME874_NATIVE_DIAGNOSTIC_DISPLAY_DIRECT_WRITE_BENCH_V1`
uses equal exact destination capacity, emits 101 alternating p50/p95/p99 sample
pairs, and requires direct-write p95 to remain within 10% of join/write.

## Local validation boundary

- Exact-file Rustfmt, Python bytecode compilation, and scoped
  `git diff --check` pass.
- Runtime874/Editor893 plus adjacent notification-center and Runtime toast
  contracts pass `101/101`.
- Runtime874 received no per-task Cargo run and was submitted with Editor893 in
  asynchronous v20 (PID `15628`) at `2026-09-21T22:24:02.7781628+08:00`.
- No v13-v20 receipt was read or monitored after submission.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/diagnostics.rs` | `0DC80F7431D6F9F6F5055365D16A7165B9C1208FAA34CCAD66CF398DCE5E13E0` |
| `zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/diagnostics/display_direct_write_tests.rs` | `88D83CBF86810EC428F3658D4825949FA508D7CFA8E414E163A8DA3FDD3B3E57` |
| `tools/tests/test_runtime874_native_diagnostic_display_direct_write_performance_contract.py` | `61A6DA78A1983FF78B0513979987B4427EB6EBAB0AD96D27447F2AE488D947D8` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
a combined current-source Windows lane compiles Runtime, executes the lower
regression and ignored Release marker, and supplies allocator plus real native
plugin behavior-error p50/p95/p99 evidence. The deterministic allocation model
is not product acceptance.
