---
title: Editor08 Missing Capability Diagnostics Preallocation
category: zircon_editor
report_id: Editor08-missing-capability-diagnostics-2026-08-26
date: 2026-08-26
session_id: root-editor06-plugin-admission-borrows-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor08 Missing Capability Diagnostics Preallocation

## Scope

Disabled command dispatch returns every required capability absent from the immutable evaluation
context. The previous iterator collect started with an empty vector and grew geometrically while
materializing the diagnostic list.

## Implementation

The helper now reserves the required-capability count once and appends missing entries through a
single explicit loop. Capability order and returned values are unchanged.

## Performance Evidence

| Evidence | Before | After / target |
| --- | ---: | ---: |
| Result vector allocation strategy | geometric growth | one preallocation |
| Missing capability order | declaration order | unchanged |
| Release p95 | dynamic evidence pending | <= 95% of legacy p95 |

The coordinator must publish `EDITOR08_COMMAND_DESCRIPTOR_MISSING_CAPABILITY_DIAGNOSTICS_BENCH_V1`
with both p95 durations, sample/iteration/capability counts, and allocation reduction.

## Validation

Scoped rustfmt, diff checks, source contracts, and legacy-equivalence tests are prepared. The
release benchmark is batched with keyword normalization in one Editor crate command; commit
integration, terminal p95 values, and WeCom delivery remain coordinator-owned.
