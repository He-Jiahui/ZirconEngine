---
title: Editor01 menu pointer source binding single read
category: zircon_editor
report_id: Editor01
date: 2026-09-01
status: local_candidate
implementation_files:
  - tools/analysis/performance/editor/editor_menu_pointer_resize_pressure.py
tests:
  - tools/tests/test_editor_menu_pointer_source_binding_single_read_performance_contract.py
  - tools/tests/test_editor_menu_pointer_resize_pressure.py
---

# Editor01 menu pointer source binding single read

## Problem

The menu-pointer resize evidence collector accessed every present critical source
three times: `read_text()` for guard validation, `stat()` for byte length, and
`read_bytes()` for SHA-256. The receipt describes one source snapshot, but the
implementation performed redundant I/O and could theoretically mix metadata and
content observed at different instants.

## Change

Each source is now read once as bytes. Strict UTF-8 decoding supplies the guard
text, `len(payload)` supplies the exact receipt byte length, and the same payload
supplies SHA-256. Missing-source behavior, strict UTF-8 failure behavior, source
ordering, uppercase digest format, manifest construction, and guard semantics are
unchanged.

The static contract requires one `read_bytes()` and rejects `read_text()` or
`stat()` inside `build_source_binding`. A behavioral fixture verifies complete
critical-source receipt equality for byte lengths and hashes while satisfying all
source guards.

## Performance evidence

The Windows benchmark used 32 one-MiB UTF-8 Rust sources, one warm-up pair, and 21
alternating legacy/current pairs. Each pair asserted identical decoded lengths,
byte lengths, and SHA-256 digests.

| Measurement | Three-access path | Single-read path | Improvement |
|---|---:|---:|---:|
| P50 | 145,133,400 ns | 99,554,800 ns | 31.405% (1.46x) |
| P95 | 198,107,100 ns | 119,477,300 ns | 39.691% (1.66x) |
| Per-file content/metadata accesses | 3 | 1 | 66.667% |

## Validation

- RED: the static contract failed because `build_source_binding` still contained
  `read_text()` and `stat()`.
- GREEN: the new performance contract and complete pressure module passed together,
  8/8 in 0.534 seconds.
- `python -m py_compile` passed for production and both test modules.
- This is queued in the accumulated asynchronous validation lane; coordinator
  status was not polled.
