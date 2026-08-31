---
title: Runtime60 Borrowed System-set Intern
category: zircon_runtime
report_id: Runtime60-borrowed-system-set-intern-2026-08-28
date: 2026-08-28
session_id: root-runtime60-single-write-conflict-probe-20260828
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime60 Borrowed System-set Intern

## Scope

This slice improves repeated system-set name admission during schedule construction. It contributes
local compile-path allocation and latency evidence but does not close RECS-P1-40, schedule DAG
compilation, set conditions, product-scene qualification, or the Runtime60 parent plan.

## Implementation

`SystemSetRegistry::intern` now accepts `Into<Cow<str>>`. Validation and the existing-name HashMap
lookup operate on the borrowed view, and `into_owned()` occurs only after lookup proves the name is
new. Repeated `&str` and `&String` inputs therefore do not allocate. New borrowed names still
materialize one owned key and clone once for the dense name table, preserving the existing two-owner
registry representation; new owned `String` inputs keep their allocation through `Cow::Owned`.

Three Rust regressions preserve dense-ID reuse for borrowed names, `&String` plus moved `String`
compatibility, stable name projection, and zero registry mutation after invalid borrowed input. The
current-source caller audit found only string literals; `String`, `&str`, and `&String` are all
covered by the new boundary.

## Performance Evidence

The release model primes one system-set name, then performs 262,144 borrowed lookups per sample. It
uses 31 alternating legacy/optimized sample pairs after five warmups and verifies the same dense ID
checksum. The acceptance result uses the second conservative run.

| Metric | Before | After | Change |
| --- | ---: | ---: | ---: |
| Lookup allocations | 262,144 | 0 | -100% |
| Requested lookup payload bytes | 3,145,728 | 0 | -100% |
| P50 per 262,144 lookups | 30,406,200 ns | 11,051,800 ns | -63.653% |
| P95 per 262,144 lookups | 60,502,900 ns | 17,259,900 ns | -71.473% |

Both implementations retained checksum `262144`. A preceding independent run measured P50
`30,241,900 -> 10,838,000 ns` (-64.162%) and P95 `84,453,800 -> 15,094,500 ns` (-82.127%). The
allocation count covers repeated lookup strings; registry setup and first-name ownership are outside
the measured lookup window and remain semantically unchanged.

## Validation

- Source contract: 3/3 passed after a confirmed 0/3 initial state.
- Exact Rust formatting and Python contract compilation: passed.
- Scoped `git diff --check`: passed for the exact three candidate paths.
- This task is queued in one Runtime60 five-task asynchronous validation batch. The batch runs 15
  source contracts, 15 `runtime60_batch_` Rust regressions, and six release models for five exact
  performance rows; no local Cargo lane was launched.
- Commit and WeCom publication remain pending independent review and managed validation.

## Remaining Parent-plan Work

System-set conditions and dependency/access DAG semantics remain open under RECS-P1-40. Runtime60
still needs bounded schedule compilation, product workload adoption, p99/RSS/cache/worker evidence,
and the remaining identity, storage, query, command, event, observer, and lifecycle gates.
