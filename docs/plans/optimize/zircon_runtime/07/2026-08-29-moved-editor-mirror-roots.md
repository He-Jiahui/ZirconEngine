---
title: Runtime07 Moved Editor Mirror Roots
category: zircon_runtime
report_id: Runtime07-moved-editor-mirror-roots-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Moved Editor Mirror Roots

## Scope

This slice removes deep copies of editor asset and content roots when the plugin SDK mirrors a
runtime package manifest. The old editor base manifest is replaced immediately, so its root
vectors can transfer ownership into the mirrored manifest instead of cloning every string first.

## Change

- Move editor asset and content root vectors out with `mem::take` before replacing the base
  manifest.
- Transfer the complete vector directly when the runtime manifest has no roots, preserving the
  vector and string buffers without allocation.
- Preserve stable unique merge behavior when the runtime manifest already contains roots.
- Keep the editor capability clone because descriptor and package manifest remain independent
  owners; this slice does not weaken that contract.
- Add a Rust pointer-identity regression and a Python source performance contract.

## Deterministic Performance Evidence

The standalone optimized Rust model mirrors 4,096 editor declarations per sample, each with eight
asset roots and eight content roots, across 31 alternating samples. Input declarations are built
before counters and timers start, so the evidence isolates the mirror operation. Both
implementations produced checksum `841563411435443060`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls per 4,096 declarations | 90,112 | 0 | 100.000% |
| Requested allocation bytes | 5,406,720 | 0 | 100.000% |
| Run 1 mirror P50 | 19.8023 ms | 7.2028 ms | 63.626% |
| Run 1 mirror P95 | 36.1554 ms | 15.8350 ms | 56.203% |
| Run 2 mirror P50 | 18.9322 ms | 7.3061 ms | 61.409% |
| Run 2 mirror P95 | 28.7473 ms | 13.2023 ms | 54.075% |

Evidence marker: `RUNTIME07_MOVED_EDITOR_MIRROR_ROOTS_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_moved_editor_mirror_roots_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust model excludes declaration setup from measurement, asserts equivalent root
  output through its checksum, and retained identical zero-allocation profiles in two runs.
- The Rust regression verifies that both root-vector buffer pointers survive an empty-target
  mirror and that the resulting values remain unchanged.
- Existing SDK mirror tests continue to define capability, runtime module, editor module, and root
  preservation semantics.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required
  before snapshot publication.
- Managed Rust compilation and plugin SDK tests remain pending in the next asynchronous Runtime07
  validation batch.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
