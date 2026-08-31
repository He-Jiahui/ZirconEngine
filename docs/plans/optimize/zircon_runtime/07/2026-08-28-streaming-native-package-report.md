---
title: Runtime07 Streaming Native Package Report
category: zircon_runtime
report_id: Runtime07-streaming-native-package-report-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Streaming Native Package Report

## Scope

This slice removes intermediate formatting strings from native dynamic package report generation.
It preserves the report format version, TOML field order and escaping, ABI contract values, blank
line placement, and final newline.

## Change

- Import `std::fmt::Write` and write package fields directly into the report output with one
  `writeln!` call.
- Write the complete ABI table into the same output with one additional `writeln!` call.
- Remove every `push_str(&format!(...))` intermediate allocation from both report functions.
- Add a Rust byte-for-byte regression for the complete generated TOML and a Python source
  performance contract.

## Deterministic Performance Evidence

The standalone optimized Rust model generates 4,096 complete reports with dynamic package,
directory, path, manifest, and ABI strings for 17 alternating samples. It first checks every
legacy and optimized report byte-for-byte. Both paths produced checksum `3100672`; the table
records the more conservative complete run.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 143,360 | 20,480 | 85.714% |
| Requested allocation bytes | 14,512,128 | 7,745,536 | 46.627% |
| Report generation P50 | 25.6092 ms | 10.9632 ms | 57.190% |
| Report generation P95 | 43.7492 ms | 22.0618 ms | 49.572% |

Evidence marker: `RUNTIME07_STREAMING_NATIVE_PACKAGE_REPORT_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_streaming_native_package_report_performance_contract.py`: 3
  passed after the pre-change contract failed 3 of 3 checks.
- The standalone model asserts byte-for-byte equality for all 4,096 generated reports.
- A Rust regression asserts the complete static package report contract.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks passed.
- Managed Rust compilation and focused tests remain pending in an asynchronous Runtime07 batch
  with the native package report-capacity candidate.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
