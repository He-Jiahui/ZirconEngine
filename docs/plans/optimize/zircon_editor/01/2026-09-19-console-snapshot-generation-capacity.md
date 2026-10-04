---
title: Editor console snapshot generation capacity
category: zircon_editor
report_id: Editor834-console-snapshot-generation-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor834 · console snapshot generation capacity

## Scope

`ConsoleOutputSnapshot::generation_from_flat_parts` already clips the
logical history to the 256-line product bound, but it still collected the
retained line records into a zero-capacity vector. This slice reuses the
bounded retained-line upper bound before extending the records, without
changing source IDs, level fallback, jump actions, CRLF presentation, or
empty/blank-line behavior.

## Implementation

- Derive `retained_line_capacity` from the existing `line_count` and
  `retained_start` values, so the common path needs no additional text scan.
- Reserve that bounded capacity and extend the existing mapping directly,
  removing the intermediate `collect::<Vec<_>>()` growth path.
- Add the lower source regression and ignored
  `EDITOR834_CONSOLE_SNAPSHOT_GENERATION_CAPACITY_BENCH_V1` marker.

## Deterministic work model

For the 256-line retained snapshot, the retired zero-capacity collector models
seven geometric growth events (capacities 4 through 256); the optimized path
starts at the retained upper bound and models `7→0` growth events. This is a
structural allocation model only; it does not claim allocator, RSS, CPU, or
product-latency results.

## TDD and local evidence

- The source/model contract was intentionally RED against the prior
  zero-capacity `collect` and became GREEN after the bounded direct extension
  (`3/3`).
- The lower Rust source regression and ignored Release marker are wired in
  `console_output_snapshot.rs`.
- Exact-file Rustfmt and Python compilation pass. A single-process
  Runtime/Editor capacity/projection smoke loader covers `170` files and
  passes `629/629` tests in `9.426s`, with zero failures, errors, load errors,
  or skips. These are local source/model checks only; managed Cargo/Windows
  Release and Console product p50/p95/p99 evidence remain pending.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/workbench/snapshot/data/console_output_snapshot.rs` | `2B1392CA22059DEC03D5045DF8D488D14DC4174EE645DDC3B14C76FE5D13BCC9` |
| `tools/tests/test_editor_console_snapshot_generation_capacity_performance_contract.py` | `87EE42FD6CC0F3575575EB1A22096853E137475BA492F12A49A48583163E833D` |

## Managed acceptance gate

Keep this record at `managed_validation_pending` until the owner-attributed
batched Windows Release lane proves current-source compilation, snapshot
parity, allocation behavior, and the declared Console product p50/p95/p99
gates. Tooling production remains deferred for the later Rust migration.
