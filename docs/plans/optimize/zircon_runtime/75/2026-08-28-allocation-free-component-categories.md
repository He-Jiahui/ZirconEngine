---
title: Runtime75 Allocation-free Component Categories
category: zircon_runtime
report_id: Runtime75-allocation-free-component-categories-2026-08-28
date: 2026-08-28
session_id: root-runtime75-borrowed-toast-queue-scan-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime75 Allocation-free Component Categories

## Scope

This slice removes transient tree allocation from component-category projection. It contributes a
focused 258-descriptor allocation and timing data point toward RUW-P1-047 and RUW-GATE-047. It does
not close component authority, descriptor-driven execution, conformance, or the 10/1K/100K widget
workload matrix in the Runtime75 parent plan.

## Implementation

`UiComponentDescriptorRegistry::categories()` now scans descriptors once into an eight-element
stack presence table, then yields present categories in the same declaration/`Ord` order as the
former `BTreeSet`. The helper uses an exhaustive category-to-index match, so a future enum variant
cannot compile without an explicit projection decision.

The public iterator signature and `missing_capabilities` set behavior remain unchanged. Three Rust
regressions cover stable ordering from scrambled input, repeated-category deduplication, and empty
input.

## Performance Evidence

The release model uses the current combined-catalog scale of 258 descriptors distributed across
all eight categories. Each of 31 alternating sample pairs performs 1,000 category projections
after five warmups. A counting global allocator measures one projection, and both implementations
must produce the same category checksum. The acceptance figures use the conservative corrected
run before a later outlier-heavy corroborating run.

| Metric | Before | After | Change |
| --- | ---: | ---: | ---: |
| Allocations per projection | 2 | 0 | -100% |
| Requested bytes per projection | 282 | 0 | -100% |
| P50 per 1,000 projections | 3,150,000 ns | 107,300 ns | -96.594% |
| P95 per 1,000 projections | 5,613,700 ns | 128,700 ns | -97.707% |

Both implementations produced checksum `7,905,000`. A subsequent independent run measured P50
`3,022,300 -> 107,900 ns` (-96.430%) and P95 `18,907,500 -> 165,200 ns` (-99.126%); the earlier
corrected P95 is retained above because it is less affected by the later legacy-side outlier.

## Validation

- Source contract: 3/3 passed after a confirmed 0/3 initial state.
- Exact Rust formatting and Python contract compilation: passed.
- Scoped `git diff --check`: passed for the exact three candidate paths.
- Three Rust unit regressions were added but have not yet been executed by Cargo.
- This candidate will be grouped with another completed Runtime75 slice in one managed validation
  batch; no local Cargo lane was launched.
- Commit and WeCom publication remain pending independent review and managed validation.

## Remaining Parent-plan Work

Runtime75 still requires a single component authority, descriptor admission and live behavior
convergence, schema-safe state transactions, component-specific accessibility, product-asset
migration, and the full conformance/fault/soak/scale gates. This local read-only projection
optimization does not substitute for those gates.
