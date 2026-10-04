---
handoff_kind: failure
status: open
created_at: 2026-09-09
summary_slug: input-manager-bound-text-owner-budget
origin_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
fixing_plan: docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
origin_child_dir: docs/plans/zircon_runtime/runtime/15
fixing_child_dir: docs/plans/zircon_runtime/runtime/09
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/dispatch/input_manager/manager.rs
  - zircon_runtime/src/ui/dispatch/input_manager/mod.rs
  - zircon_runtime/src/ui/dispatch/input_manager/manager/toast_timer_queue.rs
  - zircon_runtime/src/ui/dispatch/input_manager/manager/tests.rs
  - zircon_runtime/src/ui/dispatch/input_manager/bound_text_model_updates.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/ui_dispatch_bound_text_model_updates.rs
tests:
  - cargo test -p zircon_runtime --no-default-features --locked --lib tests::runtime_absorption::structure_convention::production_file_budget::ui_dispatch_bound_text_model_updates::runtime_text_bound_model_updates_are_bounded_child_owner
  - managed Windows input-manager dispatch and bound-text behavior regression
---

# Runtime09: input-manager root exceeds the bound-text owner budget

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 来源执行切片：Shared production-view repair, exact bound-text consumer validation.
- 修复责任计划：`docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`
- 交接原因：The failing size constraint belongs to the production input-manager
  owner; Runtime15 must retain the source boundary and diagnose the consumer.

## 失败现象与复现证据

Managed Windows job `6b9d00a92d864c57b63c3a07f0082178` executed the full
test filter above on immutable input `runtime15-production-view-3271-20260909`,
manifest `532caeb7e8cda84b9b5e5a05ad099875a958a8f9a87803aa55ff683a595712b7`.
Result: 0 passed, 1 failed, 0 ignored, 6849 filtered out. At guard line 151,
`ui/dispatch/input_manager/manager.rs` has 833 lines, exceeding the existing
strictly-below-800 owner budget. Complete receipt and output are retained under
`E:/cargo-targets/zircon-engine/cache/build-benchmarks/runtime15-production-view-3271-20260909/results/runtime15-bound-text-view-3271.{json,log}`.

Current manager hash is
`bc653f76784cc2b917fb999c4dd1a3e2667ac45d791573812826320f14461cd5`.
Its old attribution belongs to archived Runtime09 Session
`runtime09-ui-timer-frame-visible-deadline-leaf-20260722` at a different hash;
the current diff must be reconciled before claiming or editing that source.
Runtime15 acquired no production-manager ownership and changed no production
file for this diagnosis.

## 最低共享层根因

The input-manager root combines manager state/configuration and dispatch or
timer entry points beyond its declared owner budget. The current guard reached
its profile-production assertions successfully, then rejected this independent
size constraint. The final plan-status assertions did not execute, so this
handoff proves the production boundary failure only.

## 架构修复验收

- Reconcile current manager changes and choose a coherent existing child owner
  for any moved responsibility; preserve public dispatch, timer, IME, clipboard
  and bound-model behavior without duplicate state or compatibility aliases.
- Keep the manager and affected child files within the existing budget and
  verify module mounts and direct callers after the move.
- Run lower affected behavior regressions and the original exact guard on one
  complete, managed Windows source input. Report any later Runtime15 document
  status failure separately, without treating a removed first panic as GREEN.
- Return evidence to the [origin failure](../15/failure-2026-08-13-lock-poison-cfg-test-tail-masking.md).

## 禁止临时方案

- Do not increase the budget, remove the manager from the scan, or hide code
  with whitespace compression, test-only cfg, aliases or disabled assertions.
- Do not overwrite the manager's unattributed current changes or copy plan
  status strings into production source to satisfy the later cohort.

## 修复结果与回传

Open state: `current-manager-budget-reproduced_owner-repair-pending`.
The origin production-view support tests passed 2/2, but the affected bound-text
consumer failed 0/1. No Runtime09 source fix, accepted upward gate, canonical
fixed return, closeout SHA or WeCom result is claimed.

## 2026-09-09 current-source owner repair

The existing fixing Session `failure-roll-01a07160-runtime09` is reused without
changing its identity or its earlier pending tickets. Reconciliation showed
that current manager bytes exactly match the current HEAD blob at
`bc653f76784cc2b917fb999c4dd1a3e2667ac45d791573812826320f14461cd5`;
the different archived attribution is historical, not an uncommitted source
change. Transfer fingerprint
`151504294677c5af6c4ba46e750c3ca7df07880e9879b980ec5db21539e24db0`
acquired only the manager and the future parser child. Snapshot 3284 preserves
the current preimage against baseline 602. Existing Session baseline 601 and
earlier validation tickets remain unchanged.

Snapshot 3285 extracts the existing six toast queue-value parsing helpers into
`manager/toast_timer_queue.rs`. The manager mounts this private child and calls
its `pub(super)` entry; the five recursive/conversion helpers remain private.
String, enum, array and map parsing, key precedence, rounding, invalid duration
handling and surface fallback remain unchanged. No public API, state field,
timer logic, IME, clipboard, pointer or bound-model method moves or changes.
The structure follows the existing manager folder and the separation of value
interpretation from lifecycle management seen in Unreal's Slate text processing
and notification owners; no upstream implementation is copied.

- `manager.rs`: 758 lines, hash
  `6982da444be0eee859fe1bfe1571326fe471a94fdf21cc6846e4ecc30f010c9d`.
- `manager/toast_timer_queue.rs`: 79 lines, hash
  `f330c2e6c1323209039e9dd4db1b992a30843a57569fccdd05d5f6f8d4baf6a7`.

Scoped formatting and whitespace checks pass. A recursive rustfmt check also
reported pre-existing formatting in unchanged `manager/tests.rs`; that file
was neither acquired nor reformatted. Existing tests cover string and structured
queue replacement, clear, expiration and surrounding tooltip behavior. No test
has been weakened or rewritten for this extraction.

Input `runtime09-input-manager-3285-20260909` contains 10,967 files and only
these two source overlays on the 3282 input, manifest
`ef17397ec086dfdb9d2d93c90b8ccca4e44ba0c63700ee0597565d6cc53bd788`.
Actual manager behavior and the exact upward guard are pending. The later
Runtime15 document-status assertions remain independent acceptance obligations.

The original exact guard completed as job
`367b263ba0aa4a668dc2a7ab61a05030` on that input: 0 passed, 1 failed,
0 ignored, 6849 filtered out. It passed every source and size assertion, then
failed only at the first structure-document status cohort. Runtime15 has
corrected that independent mismatch in snapshot 3291; no document or budget
was weakened. Artifact: `results/runtime09-bound-text-view-3285.{json,log}`.

An initial `ui::dispatch::input_manager::` run without the `ui` feature returned
job `578f38ef82184d26a7a4537dbdffe41a`, command exit 0 but 0 tests executed.
The wrapper correctly rejected it as `not_accepted`. It neither compiles the
feature-gated input-manager implementation nor proves behavior. Artifact:
`results/runtime09-input-manager-behavior-3285.{json,log}`. A new command with
explicit `--features ui` has been submitted once against the same input;
its actual terminal result is required before accepting the production move.

The explicit UI-feature run has now terminated as job
`41500d963f7241009fc783e0d2d8724a`, prerequisite Cargo check exit 101;
validation exit 1, no test binary executed. The compiler reports 139 errors.
Six are missing `include_str!` assets in this immutable input: the editor
run/save/search icon TOML files and the material/layout/Fyrox demo ZUI files.
All six exist in the current main checkout, so these six errors are input
closure omissions, not evidence of missing repository assets. The other
diagnostics include existing Text/UI/graphics test imports and type-contract
errors; none is attributed to the parser extraction without a matching cause.
Full artifact: `results/runtime09-input-manager-ui-behavior-3285.{json,log}`
under the same 3285 input. The receipt records 584.8149714 seconds of check
time and zero test-execution time. Production behavior remains unvalidated;
neither the zero-test command nor this compile failure is acceptance evidence.

The final Runtime15 consumer input is
`runtime15-consumer-guards-3291-20260909`, manifest
`564214103c8dfbbf4cd93a0063676b65f68e8e89c403808f7064a0b54e79a4ba`.
It preserves the exact two Runtime09 production hashes above and adds only
the later Runtime15 guard corrections. The exact upward guard result is still
required, independently of the feature-enabled behavior obligation.

The final exact upward guard has now passed as job
`05374c70f4e34c8887ffeb4aea08bd77` on that combined input: 1 passed,
0 failed, 0 ignored, 6849 filtered out. It executed all six source/test budgets
and all three document cohorts. Artifact:
`results/runtime15-bound-text-view-3291.{json,log}`. This no-default-feature
structure test reads the manager source; it does not compile or execute its
UI-feature implementation. The separate behavior gate remains blocked by the
139-error prerequisite check above. No canonical fixed return is claimed.

## 2026-09-09 borrowed toast timer follow-up

The shared tree now extends the existing child split without changing the owner
budget: queue parsing and surface toast-state reads return borrowed IDs, and the
timer state reuses an unchanged ID while refreshing its deadline. The manager
still measures 763 lines (below the strict 800-line guard); the child includes
the parser tests and remains folder-backed. Current hashes are:

- `ui/dispatch/input_manager/manager.rs`:
  `316729D9B235086B420CCAE47B408837C30E23A08FA53B2AA31609F2D296F41C`
- `ui/dispatch/input_manager/manager/toast_timer_queue.rs`:
  `5FF3FCE95D2F2D8B8711E2900E8F849228D882AD3F85526F3FF32E7A8C094636`
- `ui/dispatch/input_manager/timers.rs`:
  `C668DC33FD0AECDB34DE3FB6F3B60CCC65D2F7590A19F83B051DD175C536CDDF`

Exact rustfmt, source contracts, and scoped diff checks pass. Focused runtime
Python contracts pass `36/36`, while UI-feature Cargo behavior is still not
executed; this follow-up does not close the original failure or claim a managed
fixed return.

## 2026-09-19 rolling successor source reconciliation

- Successor Session `failure-roll-01a084c8-runtime09-bound-text-r2` was admitted
  at baseline epoch `611` and received the archived owner scope through transfer
  fingerprint `2fc163eceffefd26692b4d5e91182825c827b22400548931fcad86f557b1dc02`.
  The transfer preserved the current manager, toast parser child, timer state and
  failure-record bytes; no foreign edits were reverted.
- Current source hashes at handoff are `f9a51f5fac3694dace32e90ffa0491d5ad9da4ac8531e24b5a5b87263d0cf654`
  (`manager.rs`), `5ff3fce95d2f2d8b8711e2900e8f849228d882ad3f85526f3ff32e7a8c094636`
  (`manager/toast_timer_queue.rs`) and
  `c668dc33fd0aecdb34de3fb6f3b60ccc65d2f7590a19f83b051dd175c536cddf`
  (`timers.rs`). The manager is 754 lines and the borrowed toast/timer path is
  present; this note does not claim dynamic acceptance.
- Fresh managed Runtime09 UI-feature behavior, the exact original upward guard,
  independent C/I/M review, canonical fixed return and closeout remain pending.

## 2026-09-19 managed static successor ticket

- The first successor submission (`failure-roll-01a084c8-runtime09-bound-text-20260919-r1`)
  was rejected at admission with `validation_copy_overlay_not_owned` because the
  Runtime15 production-budget guard file is still attributed to the archived
  `failure-roll-01a07160-runtime15` session. No validation ticket was created;
  the rejection is retained as ownership evidence and no archived edits were
  taken over.
- A corrected submission keeps the overlay to the four paths owned by this
  Session and was admitted as ticket
  `b62175e3fcf24a2c923bb7efd8c2dbbc` (request
  `failure-roll-01a084c8-runtime09-bound-text-20260919-r2`) with source-manifest
  hash `76cee112a96953533a6e2a9779dd61de932d16987b58ee59e1c24e0e4730477e`.
  Its managed command is the exact borrowed queue/timer/owner-budget contract
  check and must emit
  `RUNTIME09_INPUT_MANAGER_BOUND_TEXT_BORROWED_OWNER_CURRENT_SOURCE_CONTRACT_PASS`.
  This is static-only evidence (`upwardAcceptance=false`); the fresh UI-feature
  Cargo behavior gate, current-source Runtime15 guard, independent C/I/M review,
  canonical fixed return and closeout remain pending.

- Ticket `b62175e3fcf24a2c923bb7efd8c2dbbc` reached materialization job
  `8dc6652c9db04ce68265bc9a838944c8` but failed before command execution with
  coordinator error `validation_copy_dependency_archive_failed` at
  `template_dependencies`. The dependency-root list incorrectly treated the
  newly split, untracked `manager/toast_timer_queue.rs` as a pinned Git archive
  dependency; no Rust test or static command result was produced. This is a
  coordinator materialization failure, not a source-contract failure. A retry
  must keep that owned child in the overlay manifest while limiting pinned
  dependency roots to tracked paths.

### 2026-09-21 independent source review (review-runtime09-bound-text-r2)

- The current passed ticket `76bbd673c04f4f7988962717104a2c8c` was reconciled
  immediately before review. Its production hashes match the owner snapshot:
  `manager.rs` `f9a51f5fac3694dace32e90ffa0491d5ad9da4ac8531e24b5a5b87263d0cf654`,
  `manager/toast_timer_queue.rs` `5ff3fce95d2f2d8b8711e2900e8f849228d882ad3f85526f3ff32e7a8c094636`,
  and `timers.rs` `2a5a0b8d4a7f5a8aba09472c23969364d40f89f86357501944fcae2f7982b721`.
  The failure-record hash has the expected receipt drift and is not treated as a production
  mismatch.
- `manager.rs` is 754 lines, remains the orchestration root, mounts the private
  `toast_timer_queue` child, and passes the event-owned `UiValue` directly to a borrowed parser.
  The resulting `&str` is handed to `arm_toast_expiration_ref`; only the timer state retains an
  owned ID for the lifetime of an armed expiration. Public dispatch, IME, clipboard, pointer,
  text-document, and bound-model ownership remain on the manager and were not duplicated.
- `toast_timer_queue.rs` returns `Option<(&str, u64)>` for string, enum, map, and nested-array
  values, preserves key precedence and positive-duration filtering, and includes pointer-identity
  regressions for string/map storage plus a source guard against the former intermediate ID/value
  clones. `timers.rs` refreshes an existing entry in place, owns the ID only at the timer-state
  boundary, computes saturating millisecond deadlines, and drains expired IDs without retaining
  stale queue storage.
- Independent source probes passed for the strict owner budget, child mount/call chain, borrowed
  parser signature and tests, legacy-clone guard, timer reference API/deadline helper, and open
  failure status. Scoped Rust 1.94.1 `rustfmt --check` and `git diff --check` passed for all three
  production owners.
- Independent review result: `Critical=0, Important=0, Moderate=0`. This is source-contract
  evidence only. The feature-enabled Windows input-manager behavior run, fresh Runtime15 exact
  upward guard, UI/Text consumer parity, canonical fixed return, and coordinator closeout remain
  pending; the prior zero-test and compile-failed artifacts are not reused as acceptance.

- Corrected retry ticket `cb7958b85dce49a8b34184719406befb` (request
  `failure-roll-01a084c8-runtime09-bound-text-20260919-r3`) was admitted with
  source-manifest hash `ce5241ef08e208f72ff5b0c78a94efe930462e1607c93611a058c84b26cb1c8c`.
  It retains the queue child as an owned overlay and pins only tracked manager
  and timer roots; execution is queued and no pass is claimed yet.

- The corrected ticket executed as managed job
  `0d728be5704145a19e787239db38daf7` / run
  `cb7958b85dce49a8b34184719406befb` and passed with exit code `0`. Its terminal
  output is the exact marker
  `RUNTIME09_INPUT_MANAGER_BOUND_TEXT_BORROWED_OWNER_CURRENT_SOURCE_CONTRACT_PASS`.
  Cleanup completed; this remains static contract evidence only and does not
  claim the UI-feature behavior, fresh Runtime15 upward guard, review, return,
  or closeout gates.

- Ticket `b62175e3fcf24a2c923bb7efd8c2dbbc` reached materialization job
  `8dc6652c9db04ce68265bc9a838944c8` but failed before command execution with
  coordinator error `validation_copy_dependency_archive_failed` at
  `template_dependencies`. The dependency-root list incorrectly treated the
  newly split, untracked `manager/toast_timer_queue.rs` as a pinned Git archive
  dependency; no Rust test or static command result was produced. This is a
  coordinator materialization failure, not a source-contract failure. A retry
  must keep that owned child in the overlay manifest while limiting pinned
  dependency roots to tracked paths.
