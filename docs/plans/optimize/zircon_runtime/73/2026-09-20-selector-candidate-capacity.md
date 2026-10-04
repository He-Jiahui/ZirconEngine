---
title: Runtime860 Selector Candidate Scratch Capacity
category: zircon_runtime
report_id: Runtime860-selector-candidate-capacity-2026-09-20
date: 2026-09-20
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime860 Selector Candidate Scratch Capacity

## Finding

The Runtime73 terminal selector index already reused a caller-owned candidate
buffer, but a dense first lookup still started from zero capacity. A node that
matched several terminal buckets therefore paid geometric `Vec` growth before
the existing sort/dedup and full selector matcher ran.

## Optimization

`ResolvedRuleTerminalIndex::collect_candidate_indices` now clears the reused
scratch, computes a saturating upper bound from the universal, host, id, class,
component, and pseudo-state buckets, and reserves only the capacity deficit
before extending those buckets. Empty nodes still perform no reservation;
candidate order, duplicate removal, singleton fast paths, and selector
matching remain unchanged.

## TDD and deterministic evidence

The Python source/model contract was intentionally run RED before the helper,
lower owner, and test-module wiring existed, then GREEN at `4/4` after the
cutover. The lower Rust owner covers a dense `4,096`-rule bucket, an empty
zero-capacity path, and the ignored managed marker
`RUNTIME860_SELECTOR_CANDIDATE_CAPACITY_BENCH_V1`. The deterministic model
changes the dense scratch from `13` geometric growth events to `0`; empty input
remains zero-capacity.

## Local validation boundary

- The combined Runtime/Editor focused source-contract batch covering the
  existing slices plus Runtime860 passes `61/61` with zero failures, errors, or
  skips.
- Exact-file `rustfmt --edition 2021 --check` passes for the production and
  lower Rust owners; Python compilation and scoped diff checks are local
  evidence only.
- The post-Runtime860 one-process expanded source-contract loader covers `968`
  files and passes `4096/4096` tests in `142.495s`, with zero load errors,
  failures, errors, or skips. Two shader-prewarm Cargo command lines printed
  by fixture tests are not managed Windows Release/Cargo acceptance.
- No managed Windows Cargo/Release command was started locally. Allocator
  observations and selector-style product p50/p95/p99 evidence remain pending
  behind the shared external gate.
- Tooling production remains deferred for the later Rust migration.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/v2/style/rule_index.rs` | `65E3E8EC1E9C0DE40D3095B24EE244EB00BA207570A06F26A0F742813B232296` |
| `zircon_runtime/src/ui/v2/style/rule_index/candidate_capacity_tests.rs` | `3FED1CC22947E56DF0149F38F5CFC156EF4513A3440B2EBCC1779BDE00E286A0` |
| `tools/tests/test_runtime860_selector_candidate_capacity_performance_contract.py` | `2C5A25F4A067F8A0175A7B84272F5CCF5055A89486CCF4B56987DDC81D2AD067` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the owner-attributed Windows Release lane compiles the current Runtime tree,
executes the lower regression and ignored marker, and supplies allocator plus
selector-style p50/p95/p99 measurements. Do not infer product acceptance from
the local source/model receipts.
