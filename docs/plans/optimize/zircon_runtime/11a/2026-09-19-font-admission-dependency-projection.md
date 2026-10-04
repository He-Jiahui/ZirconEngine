---
title: Runtime11A font-admission dependency projection
category: zircon_runtime
report_id: Runtime808-font-admission-dependency-projection-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime808 · font-admission dependency projection

## Scope

Cross-surface UI font admission merged each surface's owned dependency strings
into a `BTreeSet`, allocating one tree node per unique dependency before making
the shared `Arc<str>` projection. The new path collects one contiguous `Vec`,
sorts it, and deduplicates adjacent entries. Lexical order, duplicate collapse,
claim replacement, admission order, and all profile counters remain unchanged.

## Implementation

- Replace the temporary `BTreeSet<String>` with a sorted/deduplicated
  contiguous `Vec<String>`.
- Keep the existing owned `Arc<str>` projection and prepared-admission flow;
  no font database, claim lifetime, failure, or resident-generation contract
  changes.
- Add a lower source regression and the ignored
  `RUNTIME808_FONT_ADMISSION_DEPENDENCY_BUFFER_BENCH_V1` marker for the managed
  Release lane.

## TDD and deterministic model

The Python source/model contract was intentionally RED against the tree-backed
collector and GREEN after the sorted vector path was added. For a 4,096-entry
unique dependency projection, the structural model changes one tree-node
allocation per entry to one contiguous projection buffer. Duplicate entries
still collapse to one lexical output and claim/admission order remains stable.

This is allocation-shape evidence only; it is not allocator, RSS, CPU, font
load latency, or product p50/p95/p99 evidence.

## Local evidence

- Focused source/model contract:
  `tools/tests/test_runtime_font_admission_dependency_projection_performance_contract.py`
  (`4/4`).
- Lower Rust source regression and ignored Release marker are wired in
  `font_admission.rs`.
- Exact-file Rustfmt and Python AST checks pass. The eight-slice focused
  Runtime/Editor batch passes `32/32`; the current one-process non-tooling
  loader passes `3693/3693` across `877` files in `83.085s`, with zero
  failures, errors, or skips. These are local source/model receipts; managed
  Cargo/Release and product font-admission percentile evidence remain pending.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/dynamic_api/session/runtime_ui/font_admission.rs` | `2BE09CE110B94CEC9B66CE512B5C6C2C979304F1FAD6A126AFB416829BF88CFC` |
| `tools/tests/test_runtime_font_admission_dependency_projection_performance_contract.py` | `AEBAD207708166D8D4B015629A5495E003B2B51C31DEDC1F3F0154384D981B6E` |

## Acceptance boundary

Keep this record `implementation_complete` /
`managed_validation_pending` until the owner-attributed Windows Release batch
compiles the current Runtime UI tree, runs the lower regression and ignored
marker, and supplies font-admission allocation and product p50/p95/p99 evidence.
Tooling production remains deferred for the later Rust migration.
