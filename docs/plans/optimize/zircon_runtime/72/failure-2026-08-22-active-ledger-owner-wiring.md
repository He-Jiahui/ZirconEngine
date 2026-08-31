---
handoff_kind: failure
status: open
created_at: 2026-08-22
summary_slug: active-ledger-owner-wiring
origin_plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
fixing_plan: docs/plans/optimize/zircon_runtime/72-runtime-core-lifecycle-registry-concurrency-shutdown-current-source-review.md
origin_child_dir: docs/plans/optimize/zircon_runtime/74
fixing_child_dir: docs/plans/optimize/zircon_runtime/72
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/core_handle.rs
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/activation/batch.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/veto_atomicity.rs
tests:
  - cargo test -p zircon_runtime --lib param_ref_compile_ --locked -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib runtime_shutdown_active_module_order_tracks_successful_reactivation --locked -- --nocapture --test-threads=1
---

# Runtime72: active ledger owner wiring is missing

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md`
- 来源执行切片：Runtime74 P0-002/P0-003/P0-005 grouped coordinator validation
- 修复责任计划：`docs/plans/optimize/zircon_runtime/72-runtime-core-lifecycle-registry-concurrency-shutdown-current-source-review.md`
- 交接原因：最低共享根因是 Runtime72 `RCL-P0-006` 的 activation ledger owner 尚未接入 Core Runtime；Runtime74 不拥有 module lifecycle 状态、提交钩子或 shutdown 顺序合同。

## 失败现象与复现证据

Coordinator validation copy `b7bdd22bdc8248cb94e5c0d119338e5d` compiled the Runtime74
overlay from base `bee4c707b714738346b49bba15c59468b8bd9b39`. The interface parser and compiler-schema
groups passed `2/2` and `1/1`. The first `zircon_runtime` test group then failed compilation with
seven `E0599` errors: production `runtime.rs` and six assertions in
`veto_atomicity.rs` call `CoreHandle::active_module_shutdown_order`, but no method exists on
`CoreHandle`. Cargo exited `101` before Runtime74 tests ran.

The Runtime72 child record `2026-08-22-active-ledger-shutdown.md` explicitly describes this snapshot
as TDD red and leaves the shared ledger field, handle accessor, and successful
activation/deactivation commit hooks pending. This is therefore a repository compile failure, not a
Runtime74 source failure or validation-copy closure omission.

## 最低共享层根因

`CoreRuntime::shutdown_registered_modules_with_drain_timeout` was switched from the frozen graph to
an active-ledger accessor before the ledger owner existed. `CoreRuntimeInner` has no active-order
state, `CoreHandle` has no accessor, and successful single/batch activation and deactivation do not
append/remove ledger entries. The call site and tests cannot compile, and a graph-scan fallback
would violate the Runtime72 ordering and performance contract.

## 架构修复验收

- Add one Core Runtime-owned active module order ledger initialized with the runtime and exposed by
  the real `CoreHandle` owner, without a compatibility facade.
- Successful single and batch activation append in dependency order; successful deactivation
  removes the module; reactivation appends at the tail. Failed/vetoed transitions do not corrupt
  the ledger.
- `runtime_shutdown_active_module_order_tracks_successful_reactivation` and the existing partial
  activation shutdown regression pass without weakening assertions.
- The Runtime72 16,384-declared/8-active, 21-pair alternating release gate must validate raw samples,
  nearest-rank P50/P95, operation counts, and the declared reduction threshold.
- Rerun the Runtime74 grouped validator after the lower-layer compile gate is green; its three child
  validators must all reach terminal pass.

## 禁止临时方案

- Do not restore the full frozen-graph scan, synthesize the answer in the test, or add a method that
  filters the registry on demand.
- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or
  call-site exceptions.
- Do not weaken Runtime72 or Runtime74 test/benchmark acceptance criteria to hide the failure.

## 修复结果与回传

Current-source owner wiring is present and statically accepted, but the Failure remains open until
managed clean-copy validation and the release performance gate pass.

2026-08-31 grouped failure evidence:

- Optimization tickets `d72cc9062ff744438f9365d44f26147d`,
  `7588c06a8fec4897b2165481148bb0f3`, and
  `12bc20b3c9fb4a7699972e4431789cd4` independently reached terminal `failed` with the same seven
  `E0599` errors and Cargo exit `101`; their selected tests never started.
- Exact path intake is recorded by transfer apply `be48188eb41c43408303288bc3f2fb34`, exact claims
  `681238e12ffc40c99d1e8f4298229ebf` and `6fe47e3206e3424fbbc0044aa876cbd6`, and fresh
  attributions `61e21f4b28f1417c86128f44250e3400` and
  `4c5179a4f5194177a6bcd15094eccd5d`. No maintenance override was used.
- Rust 1.94.1 `rustfmt --check`, scoped `git diff --check`, and the declared 16,384/8/21,
  nearest-rank P95, 75% reduction, and benchmark-marker shape checks are green for current source.
- The failed ticket base predates later task-graph/random/time/state owner migrations now co-located
  in `CoreHandle` and `CoreRuntimeInner`. A retry that overlays only the Runtime72 files would not
  be a complete source closure, while absorbing all foreign migrations would violate ownership.
  Rerun once as a grouped managed batch after the lower architecture closure is integrated.

Open state: `current source wired; managed validation pending`; no test, performance, commit, push,
or WeCom success is claimed.
