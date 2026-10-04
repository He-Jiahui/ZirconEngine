---
handoff_kind: failure
status: open
created_at: 2026-08-30
summary_slug: runtime136-composition-test-validation-materialization
origin_plan: docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
fixing_plan: docs/plans/optimize/zircon_runtime/99zk-runtime-builtin-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-current-source-review.md
origin_child_dir: docs/plans/optimize/zircon_runtime/02
fixing_child_dir: docs/plans/optimize/zircon_runtime/99zk
plan_link_mode: child_record_only
failure_scope: cross_plan
related_code:
  - zircon_runtime/src/builtin/runtime_modules/tests/registration/structure.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/registration/composition.rs
tests:
  - validation ticket cc62406c73b5435ab3c8dada05132535
  - validation copy job e3fb1b77eea14cde851f6f585222acd9
---

# Runtime136: composition regression is outside validation materialization

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md`
- 来源执行切片：aggregate Runtime/Editor Release validation for optimization batches 501-504 and 506-512
- 修复责任计划：`docs/plans/optimize/zircon_runtime/99zk-runtime-builtin-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-current-source-review.md`
- 交接原因：the missing compile-time resource contains Runtime136 composition compiler regressions and `runtime136.*` fixtures, below Runtime02 optimization ownership.

## 失败现象与复现证据

The original artifact was stored at `docs/plans/optimize/zircon_runtime/02/failure-2026-08-30-runtime136-composition-test-validation-materialization.md`; its evidence is preserved here under the canonical fixing child.

Validation ticket `cc62406c73b5435ab3c8dada05132535`, copy job
`e3fb1b77eea14cde851f6f585222acd9`, failed during closure planning with
`validation_copy_compile_time_resource_missing`. The source guard
`zircon_runtime/src/builtin/runtime_modules/tests/registration/structure.rs` includes
`composition.rs`, but that target is not materializable from the ticket source closure.

Current hashes are:

- `structure.rs`:
  `4e653d23e8b74f09fecbf0afa56bec9257a9b04827f82e04a87b05655004cf09`
- `composition.rs`:
  `37ac596b5a2d4e832c0ad76830545f74ffbf006dc1e6cb9cb3004b0ec672e248`

`composition.rs` exists as an untracked, unattributed file. `structure.rs` retains archived
Frameworks02 attribution. Runtime02 did not edit, claim, attribute, or add either file to its
candidate.

## 最低共享层根因

Runtime136 split its composition regressions into a child source and made the structural guard
require that child, but the child has no durable coordinator attribution or integrated source
owner. Any validation copy that discovers `structure.rs` therefore fails before Rust compilation.

## 架构修复验收

- Runtime136 legally claims and attributes the exact composition regression source together with
  its registration module/structure closure, or reconciles it through the owning Runtime136
  session without absorbing unrelated files.
- The split test owner remains canonical; the tests are not copied back into `structure.rs`.
- Managed Runtime validation advances beyond closure planning and compiles the Runtime136
  composition tests.
- Runtime02 reruns one aggregate validation for the affected optimization batches after the source
  is integrated or legally transferred.

## 禁止临时方案

- Do not delete or weaken the `include_str!("composition.rs")` structural assertion.
- Do not inline duplicate composition tests into `structure.rs` or add a fallback include path.
- Do not claim the unattributed source through maintenance override or include it in a Runtime02
  commit candidate.

## 修复结果与回传

待修复；the coordinator must retain the original reproduction, ownership handoff, and managed validation evidence until `composition.rs` is materializable.

### 2026-09-11 failure rolling repair: current-source attribution correction

The original failure evidence remains valid for ticket
`cc62406c73b5435ab3c8dada05132535`: its validation copy failed in closure
planning because the source closure then omitted
`tests/registration/composition.rs`. That evidence must not be rewritten as a
test failure.

The current source state is different. At `c37155ba304740b3762b20585f77fb53a6da47fb`, both
`structure.rs` and `composition.rs` are tracked in `HEAD`; `git ls-files --stage`
resolves the latter to blob `3dd9aef210bdec96e4225a41e4760c5ed95adb2b`.
Their current SHA-256 values remain respectively
`4e653d23e8b74f09fecbf0afa56bec9257a9b04827f82e04a87b05655004cf09` and
`37ac596b5a2d4e832c0ad76830545f74ffbf006dc1e6cb9cb3004b0ec672e248`.
The earlier description of `composition.rs` as an untracked, unattributed file
is therefore historical diagnosis rather than a current ownership fact.

This rolling-repair session owns only this failure record and makes no source
change. It will retry managed validation against the current tracked
registration closure, covering the `composition` test module and compilation
of the structural `include_str!("composition.rs")` guard. A passed managed
ticket and Runtime02's required upward aggregate validation remain prerequisites
for a fixed return.

### 2026-09-19 rolling successor source reconciliation

The first successor registration `failure-roll-01a084c8-runtime136-composition-r2`
was cancelled immediately after admission because its comma-joined scope was
not represented as separate coordinator paths; it performed no transfer, lease,
source edit or validation request. Corrected successor Session
`failure-roll-01a084c8-runtime136-composition-r3` owns this record,
`structure.rs`, and `composition.rs` after transfer fingerprint
`faccec456ee89246c009e55c29df725a6f5bf0c99741f62d157141247be6e898`.
Current hashes are `4e653d23e8b74f09fecbf0afa56bec9257a9b04827f82e04a87b05655004cf09`
for `structure.rs` and
`37ac596b5a2d4e832c0ad76830545f74ffbf006dc1e6cb9cb3004b0ec672e248` for
`composition.rs`; no foreign source was reverted. Fresh managed Runtime136
composition Cargo, Runtime02 upward aggregation, review, return and closeout
remain pending.

The current-source static contract ticket
`3e41fa488e204c97b6c05244e7539462` (request
`failure-roll-01a084c8-runtime136-composition-20260919-r1`) is admitted and
queued. Its source-manifest hash is
`fc38eb5654ca30dd8ed9bc2e562395021fde55ab725d08c5ecdfdd6a6d99eaba`; the
command checks the tracked `structure.rs`/`composition.rs` closure and emits
`RUNTIME136_COMPOSITION_CURRENT_SOURCE_CLOSURE_CONTRACT_PASS`. This is
static-only evidence and does not claim Cargo compilation, Runtime02 upward
acceptance, review, return, or closeout.

The ticket executed as managed job `42516a13d9ba43c4a9905427de0c7be8` / run
`3e41fa488e204c97b6c05244e7539462`, exited `0`, and emitted the exact marker
`RUNTIME136_COMPOSITION_CURRENT_SOURCE_CLOSURE_CONTRACT_PASS`; cleanup completed.
This proves only the current-source closure contract. Runtime136 Cargo,
Runtime02 upward aggregation, independent C/I/M review, fixed return and
closeout are still outstanding.

### 2026-09-20 independent source review r3

Reviewer session: `review-runtime136-composition-r3`, child of
`failure-roll-01a084c8-runtime136-composition-r3`. The review covered the two
tracked registration-test sources at the current baseline; no source was edited
or re-owned.

Result: **Critical=0 / Important=0 / Moderate=0**.

- `structure.rs` materializes `composition.rs` through the same registration
  module that declares it, and its guard checks the child-owner boundary,
  composition compiler/outcome contracts, target/profile routing, plugin/load
  report ownership, and removal of the legacy flattened APIs. The positive
  `host_modules_participate_in_the_final_compiled_activation_graph` anchor is
  present in the included child source.
- `composition.rs` supplies four focused regressions: host dependency ordering,
  duplicate-host rejection without a ready plan, stable identity/fingerprint
  binding to the final graph, and profile/target mismatch rejection. The
  assertions exercise the public compiler result/rejection contracts rather than
  duplicating implementation details.
- The current-source closure marker passed and scoped `git diff --check` passed.
  `rustfmt --check` reports pre-existing line-wrap/style differences in these
  archived guard/test sources; the reviewer did not rewrite them or alter the
  owner snapshot.

Current reviewed hashes:

```text
zircon_runtime/src/builtin/runtime_modules/tests/registration/structure.rs
  4e653d23e8b74f09fecbf0afa56bec9257a9b04827f82e04a87b05655004cf09
zircon_runtime/src/builtin/runtime_modules/tests/registration/composition.rs
  37ac596b5a2d4e832c0ad76830545f74ffbf006dc1e6cb9cb3004b0ec672e248
```

Fresh managed Runtime136 Cargo compilation and the required Runtime02 upward
aggregate remain pending. The original validation-copy failure is retained as
closure-planning evidence only; this review does not promote the static marker to
dynamic acceptance or fixed closeout.
