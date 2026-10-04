---
title: Editor01 inspector source binding single read
category: zircon_editor
report_id: Editor01
date: 2026-09-01
status: local_candidate
implementation_files:
  - tools/analysis/performance/editor/editor_inspector_projection_pressure.py
tests:
  - tools/tests/test_editor_inspector_projection_pressure.py
  - tools/tests/test_editor_inspector_source_binding_single_read_performance_contract.py
---

# Editor01 inspector source binding single read

## Problem

The Inspector source-binding receipt accessed each present critical and reference
source three times: text read, metadata probe, and binary read for SHA-256. The
receipt therefore paid redundant I/O and could derive its fields from different
filesystem snapshots.

## Change

Both source groups now read bytes once. Critical sources retain strict UTF-8
decoding; reference sources retain replacement decoding. Byte length and SHA-256
are derived from the same payload. Missing-file behavior, guard validation,
ordering, and uppercase digest format remain unchanged.

## Performance evidence

Windows benchmark with 32 one-MiB UTF-8 sources, 21 alternating samples:

| Measurement | Three-access path | Single-read path | Improvement |
|---|---:|---:|---:|
| p50 | 190,431,400 ns | 108,195,300 ns | 43.184% lower, 1.76x |
| p95 | 324,625,400 ns | 185,867,200 ns | 42.744% lower, 1.75x |
| Per-file content/metadata accesses | 3 | 1 | 66.667% lower |

All benchmark samples produced the same decoded-length, byte-length, and digest
checksum.

## Validation

- RED: the new static contract observed zero `read_bytes` calls and found the
  legacy `read_text`/`stat` path.
- GREEN: the complete Inspector pressure suite plus the performance contract
  passed 10/10 in 0.725 seconds.
- The candidate will be validated with the existing Tooling06, Tooling29, and
  Editor01 batch; coordinator status is not polled.
