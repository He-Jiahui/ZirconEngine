---
title: Runtime07 Borrowed Extension Registration Strings
category: zircon_runtime
report_id: Runtime07-borrowed-extension-registration-strings-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Borrowed Extension Registration Strings

## Scope

This slice removes the temporary `Vec<String>` created for every ZrVM extension registration
callback. The callback validates and borrows its string arguments only for the synchronous host
registration call, so no owned string container is required at that boundary.

## Change

- Change the registration callback shape to receive the borrowed `ScriptHostArguments` source and
  its dynamically generated module label.
- Read the three or four guest strings through nested borrowed visitors.
- Preserve string-kind validation, callback labels, registration order, capability checks, and all
  existing error text; this path no longer records guest-string copy bytes because it performs no
  string copy.
- Keep each borrow within its visitor invocation; no reference escapes the ZrVM call boundary.
- Add a Python source performance contract for the nested visitor and no-copy shape.

## Deterministic Performance Evidence

The standalone Rust model executes 131,072 extension-registration-equivalent callbacks with four
string arguments over 31 alternating samples. Its optimized path uses nested borrowed visitors,
matching the production lifetime boundary; it isolates argument projection and excludes ZrVM FFI,
registry mutation, and host authorization work. Both implementations produced checksum
`17411752368240706263` in both runs.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Temporary allocation calls per 131,072 callbacks | 655,360 | 0 | 100.000% |
| Requested allocation bytes | 17,563,648 | 0 | 100.000% |
| Run 1 wrapper P50 | 49.8380 ms | 0.0863 ms | 99.827% |
| Run 1 wrapper P95 | 94.0717 ms | 0.1145 ms | 99.878% |
| Run 2 wrapper P50 | 47.5772 ms | 0.0813 ms | 99.829% |
| Run 2 wrapper P95 | 66.4433 ms | 0.0916 ms | 99.862% |

Evidence marker: `RUNTIME07_BORROWED_EXTENSION_REGISTRATION_STRINGS_MODEL_V1`.

The latency percentages apply only to the temporary argument projection represented by the model;
they are not an end-to-end ZrVM registration-throughput claim.

## Validation

- `python tools/tests/test_runtime07_borrowed_extension_registration_strings_performance_contract.py`:
  4 passed after the pre-change source failed during helper discovery.
- The standalone Rust model compiled with `rustc -C opt-level=3` and passed twice with identical
  allocation profiles and checksums.
- Exact-file Rust/model formatting, Python compilation, the Runtime07 source-contract batch, and
  scoped diff checks are required before snapshot publication.
- Managed tests must compile the real `backend-zr-vm` feature and retain extension registration
  authorization and argument-validation coverage.

## Remaining Parent-plan Work

This local projection optimization does not remove ZrVM value materialization, the process-global
VM lock, execution-budget gaps, typed ABI work, debugger/profiler gaps, or product-scale
editor/app/export/cook acceptance owned by the Runtime07 parent plan.
