---
title: Runtime853 V2 Style Rule Filter Retain
category: zircon_runtime
report_id: Runtime853-v2-style-rule-filter-retain-2026-09-20
date: 2026-09-20
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime853 · V2 style-rule filter retain

## Scope

The V2 style resolver already compiled one complete, ordered `Vec<ResolvedRule>`
for an asset. Static resolution and the runtime pseudo-state index then filtered
that vector with `filter(...).collect::<Vec<_>>()`, allocating a second temporary
buffer and copying/moving every retained rule. This slice removes that redundant
filter buffer without changing selector parsing, specificity order, or matching.
The shared working tree also contains the adjacent Runtime785 rule-capacity,
wildcard-selector, and Runtime854 selector-path repairs in this owner file; this
record attributes only the new in-place filter cutover and preserves those
adjacent changes.

## Optimization

- `resolve_static` and `resolve_static_with_theme` retain non-pseudo-state rules
  in the compiled vector in place.
- `UiV2RuntimeStyleIndex` retains pseudo-state rules in the same vector before
  resolving token sources and building its terminal index.
- `retain_rules_by_pseudo_state` keeps the predicate in one owner so the two
  paths share the same state classification and preserve relative rule order.
- The existing authored-rule capacity reservation remains the sole allocation
  for the compiled rule table; no compatibility or tooling path changes.

## TDD and deterministic evidence

The new Python source/model contract was intentionally run RED before the
retain helper, lower regression, and marker existed (three structural failures),
then GREEN at `4/4` after the implementation and test wiring. The lower Rust
regression mixes static and pseudo-state selectors, verifies original rule-order
indices, and asserts that the retained buffers keep the compiled capacity. The
ignored Release probe emits
`RUNTIME853_V2_STYLE_RULE_FILTER_RETAIN_BENCH_V1` with alternating samples and
P50/P95/P99 fields. A 4,096-rule model changes the per-build filter-buffer
count from `1` to `0`.

## Local validation

- `tools/tests/test_runtime_v2_style_rule_filter_retain_performance_contract.py`:
  `4/4`.
- Existing V2 rule-capacity and pseudo-state contracts run with the new contract
  in one focused batch: `10/10`, zero failures/errors/skips.
- Exact-file `rustfmt --edition 2021 --check` passes for the production and
  lower Rust owners; scoped `git diff --check` reports no content errors.
- After this contract was added, the widened one-process non-tooling loader
  covered `946` files and passed `3959/3959` tests in `37.988s`, with zero
  failures, errors, load errors, or skips.
- After the adjacent Runtime854 and Runtime855 contracts were added, the latest
  widened loader covers `957` contract files and passes `4042/4042` tests in
  `125.129s`, with zero failures, errors, load errors, or skips.
- The current expanded source-contract loader covers `962` non-tooling files
  under the explicit performance-or-contract filename filter and passes
  `4072/4072` tests in `139.499s`, with zero failures, errors, load errors, or
  skips; the `957`/`4042` receipt remains pre-Editor856 historical context.
- No managed Windows Cargo/Release validation command was started locally.
  Allocator observations and product style p50/p95/p99 evidence remain pending
  behind the shared external worktree gate.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/v2/style.rs` | `C7A9BDABD7E0DA0CC98595662DFD7858CEDB2B6827C323D1339B20174A145DCE` |
| `zircon_runtime/src/ui/v2/style/rule_capacity_tests.rs` | `419BDDD7F4A02FCFBE353DFDF269563B0CE8A21718ACD286D05C987D8DB37DE9` |
| `tools/tests/test_runtime_v2_style_rule_filter_retain_performance_contract.py` | `C179BF71E7C8F7C0F5478E70FEBEDE7156580EA0419892F222DA315E89BE42E5` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the owner-attributed Windows Release lane compiles the current Runtime tree,
executes the lower regression and ignored marker, and supplies allocator plus
product style p50/p95/p99 measurements. Tooling production remains deferred for
the later Rust migration.
