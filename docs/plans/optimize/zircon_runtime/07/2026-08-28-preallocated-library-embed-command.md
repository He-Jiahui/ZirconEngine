---
title: Runtime07 Preallocated Library Embed Command
category: zircon_runtime
report_id: Runtime07-preallocated-library-embed-command-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Preallocated Library Embed Command

## Scope

This slice removes dynamic path normalization and avoidable command-vector growth from library-embed
compile-host plan generation. It preserves target-mode selection, manifest and target paths,
Debug/Release profiles, command argument order, plugin projections, and owned plan fields.

## Change

- Replace the fixed `Path::join -> display -> String::replace` and `PathBuf::display` pipelines with
  canonical static manifest and forward-slash target paths.
- Build the compile-host command in a dedicated helper with exact capacity for thirteen Debug
  arguments or fourteen Release arguments.
- Preserve the required manifest and target path clones because both the plan fields and command
  arguments own those strings.
- Add a Rust regression for the complete client Release command and a Python source contract for the
  fixed path and capacity invariants.

## Deterministic Performance Evidence

The standalone optimized Rust model constructs 16,384 alternating Debug/Release compile-host plans
for 17 alternating samples. Both implementations first compare complete plan structures, including
package, binary, feature lists, paths, profile, and command. Both produced checksum `4014080`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 507,904 | 385,024 | 24.194% |
| Requested allocation bytes | 17,227,776 | 10,100,736 | 41.369% |
| Plan generation P50 | 51.0034 ms | 32.1958 ms | 36.875% |
| Plan generation P95 | 67.5855 ms | 39.6198 ms | 41.378% |

Evidence marker: `RUNTIME07_PREALLOCATED_LIBRARY_EMBED_COMMAND_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_preallocated_library_embed_command_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- `python tools/tests/test_runtime_export_library_embed_command_policy.py`: 1 passed.
- The standalone Rust model asserts complete Debug and Release plan equality.
- A Rust regression asserts the complete client Release compile-host command contract.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks passed.
- Managed Rust compilation and focused tests remain pending in a batched asynchronous Runtime07
  validation with the preallocated source-template command candidate.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
