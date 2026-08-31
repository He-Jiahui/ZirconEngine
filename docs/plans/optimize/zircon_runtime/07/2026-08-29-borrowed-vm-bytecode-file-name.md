---
title: Runtime07 Borrowed VM Bytecode File Name
category: zircon_runtime
report_id: Runtime07-borrowed-vm-bytecode-file-name-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Borrowed VM Bytecode File Name

## Scope

This slice removes a temporary owned bytecode file-name string from VM package discovery. Both a
manifest-provided file name and the default `plugin.bin` value remain borrowed until the existing
path resolver creates the required owned `PathBuf`.

## Change

- Project the manifest's optional bytecode string with `as_deref`.
- Return either the borrowed custom file name or the static default from one helper.
- Remove the default helper that allocated a new `String` for `plugin.bin`.
- Preserve relative-path validation, package-root joining, custom/default selection, and the
  resulting bytecode path.
- Add a Rust custom/default regression plus a Python source contract.

## Deterministic Performance Evidence

The standalone optimized Rust model measures complete package-root path construction, including the
common `PathBuf::join` allocations. It alternates a representative 43-byte custom bytecode name and
the default name across 65,536 resolutions per sample, alternates legacy and optimized order across
31 samples, counts allocations for each branch, and verifies identical path checksums.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Custom-name path allocation calls | 4 | 3 | 25.000% |
| Custom-name requested bytes | 337 | 294 | 12.760% |
| Default-name path allocation calls | 3 | 2 | 33.333% |
| Default-name requested bytes | 136 | 126 | 7.353% |
| Mixed path construction P50 | 36.2030 ms | 31.4941 ms | 13.007% |
| Mixed path construction P95 | 63.3234 ms | 54.0880 ms | 14.584% |

Evidence marker: `RUNTIME07_BORROWED_VM_BYTECODE_FILE_NAME_MODEL_V1`.

A second complete run remained favorable: P50 improved 17.475% and P95 improved 24.958%.
Both paths produced checksum `534099166516576256`.

## Validation

- `python tools/tests/test_runtime07_borrowed_vm_bytecode_file_name_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust 1.94.1 model compiled and passed two complete 31-sample runs with identical
  custom/default paths and checksums.
- The Rust guard verifies exact `module/runtime.zrbc` custom selection and `plugin.bin` fallback.
- Parent-file Rust formatting uses `skip_children=true` so already-dirty discovery I/O and payload
  cache child modules remain untouched.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required
  before snapshot freeze.
- Managed Rust compilation and focused tests remain pending in an asynchronous Runtime07 batch.

Managed batch request: `runtime07-borrowed-gameplay-seven-task-batch-20260830-v1`.

Validation attempt: ticket `a9dc9a55e9044c239cc7dfda8bbc64b6` failed before Cargo at
coordinator artifact governance for `D:\ZirconBuilds\mvp-test-fixtures-36724`; the 22 local contract
checks remain green while integrated acceptance and success publication remain pending.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
