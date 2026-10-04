---
title: Runtime845 Style Plan Rule Capacity
category: zircon_runtime
report_id: Runtime845-style-plan-rule-capacity-2026-09-20
date: 2026-09-20
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime845 · style-plan rule capacity

## Scope

The template asset compiler flattens every authored rule from the resolved
stylesheet list into `ParsedStyleRule` values. The total authored rule count is
available before selector parsing, but the collector previously started at
zero capacity.

## Optimization

- Fold all stylesheet rule lengths with saturating addition before parsing.
- Initialize the parsed-rule vector with that exact lower bound.
- Preserve selector error propagation, global rule order, per-sheet token-map
  sharing, empty-sheet behavior, and declaration cloning.

## TDD and deterministic evidence

The Python source/model contract was intentionally run RED before the capacity
collector and lower tests existed, then GREEN after the reservation and lower
marker were wired (`4/4`). The existing style-plan lower module now covers
bounded capacity and empty-sheet zero capacity and emits the ignored
`RUNTIME845_STYLE_PLAN_RULE_CAPACITY_BENCH_V1` marker. A dense 4,096-rule model
changes geometric vector growth from `11→0` events.

## Local validation

- `tools/tests/test_runtime_style_plan_rule_capacity_performance_contract.py`:
  `4/4`.
- Exact-file Rustfmt and Python compilation pass for the production, lower, and
  contract files.
- The combined focused Runtime/Editor loader passes `86/86` tests across `22`
  modules in `0.069s`; the broad non-tooling performance/pressure loader passes
  `2374/2374` tests across `649` modules in `5.699s`, with zero load errors,
  failures, errors, or skips. Managed Windows Cargo/Release, allocator, and
  style-plan product p50/p95/p99 evidence remain pending behind the external
  worktree gate.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/template/asset/compiler/style_apply.rs` | `9BEBDF2406824F7D55AE581D2F0901DFE23A6CEC23549701706C8A816877E7EE` |
| `zircon_runtime/src/ui/template/asset/compiler/style_apply/token_map_sharing_tests.rs` | `0371D9B29514628BB1CCA276F9E415D8B5839490719D5BD19833930AC66DF427` |
| `tools/tests/test_runtime_style_plan_rule_capacity_performance_contract.py` | `82738B86BF7BB7C767C73CC07A9CD7742ECB2647196966097F93C033ECA898B8` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the owner-attributed managed Windows Release lane compiles the current tree,
executes the lower regression and ignored marker, and supplies allocator plus
style-plan product p50/p95/p99 evidence. Tooling production remains deferred
for the later Rust migration.
