---
title: Runtime09 pane button source contract single read
category: zircon_runtime
report_id: Runtime09
date: 2026-09-01
status: local_candidate
implementation_files:
  - tools/analysis/performance/ui/ui_pane_button_fallback_damage_pressure.py
tests:
  - tools/tests/test_ui_pane_button_fallback_damage_pressure.py
  - tools/tests/test_ui_pane_button_source_contract_single_read_performance_contract.py
---

# Runtime09 pane button source contract single read

## Problem

The PaneButton fallback-damage source contract read each of its two Rust sources
once as text and again as bytes for SHA-256. This doubled content I/O and allowed
the guard text and digest to observe different filesystem snapshots.

## Change

Each source is now read once as bytes. Strict UTF-8 text and uppercase SHA-256 are
derived from the same payload. Contract keys, drift checks, and error behavior are
unchanged.

## Performance evidence

Windows benchmark with two 4 MiB UTF-8 Rust sources and 31 alternating samples:

| Measurement | Two-read path | Single-read path | Improvement |
|---|---:|---:|---:|
| p50 | 24,501,600 ns | 16,116,300 ns | 34.223% lower, 1.52x |
| p95 | 44,056,000 ns | 26,837,100 ns | 39.084% lower, 1.64x |
| Reads per source | 2 | 1 | 50.000% lower |

Every benchmark pair produced the same text-length and digest checksum.

## Validation

- RED: the new static contract found no `read_bytes` calls and observed the
  legacy `read_text` path.
- GREEN: the complete PaneButton pressure suite plus the performance contract
  passed 6/6 in 0.309 seconds.
- This candidate joins the existing asynchronous batch; coordinator status is not
  polled.
