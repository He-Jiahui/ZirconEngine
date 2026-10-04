---
handoff_kind: failure
status: open
failure_scope: cross_plan
plan_link_mode: child_record_only
created_at: 2026-09-08
summary_slug: resource-readiness-payload-type-erasure
origin_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
fixing_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
origin_child_dir: docs/plans/zircon_runtime/runtime/04
fixing_child_dir: docs/plans/zircon_runtime/frameworks/01
related_code:
  - zircon_runtime/crates/zr_resource/src/manager/resource_manager.rs
  - zircon_runtime/crates/zr_resource/src/manager/tests/readiness_payload.rs
tests:
  - managed Windows static/no-default/locked zr_resource --lib
  - managed Windows static/no-default/locked zircon_runtime --lib asset::tests::facade
---

# Frameworks01: readiness publication records the erased Arc container type

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 来源执行切片：Runtime04 asset-readiness-generation-snapshot upward facade validation.
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 交接原因：The shared resource authority is now owned by the internal zr_resource foundation crate.

## 失败现象与复现证据

Managed Windows static/no-default/locked job
`195f6995f19e49cc9e6c9a9f7b7bd919` passed Cargo check, then executed
`zircon_runtime --lib asset::tests::facade`: 15 passed, 11 failed, 0 ignored,
6823 filtered out. The failures include freshly registered typed assets reported
as `NotLoaded`, successful reloads remaining `NotLoaded`, and recursive failures
masked by root `NotLoaded`.

The immutable input `editorui12-button-focus-3228-20260908` has manifest
`e1bb2d3955a335f2f0ba3462155d9dd5c9d6db866d535c760ac1ebd77f5401e6`.
The receipt and log are under its `results/` directory as
`runtime04-readiness-facade-3228.json` and `.log`.
Current resource-manager source matches that input, SHA-256
`0115b346b9a26ea3f572d4df03c528b6c6d70a850de9f73a06575c034ca173d1`,
preserved in preimage snapshot 3234. Current facade/readiness, manager and all
nine facade test files also matched the input at execution. This evidence does
not cover unrelated current project-import source drift.

## 最低共享层根因

The diagnosed support chain is resource payload storage, authority source-update
materialization, immutable readiness projection, then typed facade queries.
`ResourceData` has a blanket implementation covering both concrete payloads and
`Arc<dyn ResourceData>`. `readiness_source_update` invokes `as_any()` directly on
the Arc stored in the payload map, so it records the container TypeId instead of
the erased payload TypeId. Untyped readiness remains loaded while
`typed_load_state<TData>` correctly rejects this incorrect type identity.
Projection-only tests construct TypeId explicitly and do not exercise this
production materialization boundary.

## 架构修复验收

- Dispatch `as_any()` through the erased payload reference at the resource authority.
- Test real registration, replacement with another concrete type, preservation
  of old immutable generations, wrong-type rejection and an actual Arc-valued payload.
- Run the complete zr_resource library, then rerun all 26 original Runtime facade tests.
- Preserve residency, failed/reloading precedence and public API contracts.
- Bind exact source and executed commands to formal fixing validation, obtain
  independent C0/I0/M0 review, return and coordinator closeout before acceptance.

## 禁止临时方案

- Do not remove TypeId checks, force facade results to Loaded, or restore payloads during queries.
- Do not add consumer caches, special cases, compatibility APIs or skipped regressions.
- Do not claim Runtime04's separate performance matrix passed from this functional fix.
- Do not modify or revalidate external zr_vm.

## 修复结果与回传

Open state: `source-repaired_lower-and-original-gates-passed_independent-review-passed_formal-binding-pending`.
Existing fixing Session `failure-roll-01a07160-frameworks01` is retained with its
original pinned base and earlier ticket ownership. Exact-path ownership transfer
fingerprint `336f4ccdc77cc03b75d715d3486f28ad7e8c3e883ed49b7c6de911016e701858`
preserves the current preimage. Its existing import-order-only difference from
HEAD was inspected and retained; no unrelated behavior was adopted or reverted.
Source snapshot 3235 adds three focused lower-layer regressions without the
behavior change. Derived input `frameworks01-readiness-payload-red-3235-20260908`,
manifest `861cc5c4a026ac6ca9815d0ab9f89340245c075a33c1dd294da09f421572d913`,
executed managed job `f29fa90ad31641218c772ce03b2ba80f`: 0 passed, 3 failed,
0 ignored, 234 filtered out. All failures are the expected concrete TypeId or
typed `Loaded` assertion failures, after successful compilation.
The single production change now calls `payload.as_ref().as_any().type_id()`
at the authority boundary. The test source remains unchanged from the RED run.
Fixed source snapshot 3237 retains the unchanged test hash
`e9f91a6bd52dfe656000ecbe8cb80f4628ba9ddd602dbce2a98daf3cf522a7be`
and resource-manager hash
`def276dda695adcf29cd8079b2d569327f7e984d2dbb9ad597a1acee145a9866`.
The derived input `frameworks01-readiness-payload-3237-20260908` has manifest
`5dc7c3d6891874a69436ba9e7641c9e0863eb7ed548c05db7306233c5da9543d`.
It differs from the RED input only in the resource-manager expression.

Managed Windows static/no-default/locked validation completed bottom-up:

- Job `74f5113895dd40d78b1dcf7d1bf8a9fd`, complete `zr_resource --lib`:
  227 passed, 0 failed, 10 ignored, 0 filtered out. All three new regressions
  executed and passed. Existing ignored tests are not counted as executed.
- Job `fede77172a564c74baa6ce25f9dcd425`, complete Runtime
  `asset::tests::facade`: 26 passed, 0 failed, 0 ignored, 6823 filtered out.
  All eleven original failures now pass without changing a facade assertion.

Logs and source-bound receipts are under that input's `results/` directory as
`frameworks01-readiness-payload-library-3237.{json,log}` and
`frameworks01-readiness-payload-facade-3237.{json,log}`. Current source and
attribution hashes still match snapshot 3237 after both jobs.
These managed operational jobs are not formal fixing-Session validation tickets.
Formal external pinning and reviewer lifecycle restrictions remain separately
tracked; no identical blocked request is resubmitted. Formal binding, canonical
return and coordinator closeout are pending. Runtime04
performance acceptance remains independent and external zr_vm is still skipped.

Independent review in the existing task "优化协调器验证效率" completed at
`2026-09-08T14:43:57Z`: Critical 0, Important 0, Moderate 0 for both source
correctness and evidence. The review binds source snapshot 3237 and record
snapshot 3239 (`1ed6fc343b240b17f4bcc8fb5c305e225d086665dc3907b30020de7a8f032d05`),
including the actual RED and GREEN commands, manifest and job results. Report:
`.codex/tmp/frameworks01-readiness-payload-3237-review-20260908-result.txt`.
The reviewer found no ownership conflict on the three reviewed paths. Its
coordinator Session remains archived, so this independent result is not a
successful formal closeout-review binding. No commit or notification is claimed.

## 2026-09-19 rolling successor formal source binding

- Successor Session `failure-roll-01a084c8-frameworks01-readiness-payload-r2` reclaimed the
  archived exact-path ownership through coordinator transfer fingerprint
  `283bc9fed1c5c76a0e3c2b8d820db0ff31c0b704c943a0dd3537864f6f897eec` at baseline epoch
  `611`. The current source hashes were rechecked before submission and are recorded below;
  no source bytes were changed by the transfer.
- Formal non-Cargo source-contract ticket `0498a529f0eb474e82bc3693a9b9f8da` was admitted
  from request `failure-roll-01a084c8-frameworks01-readiness-payload-20260919-r2` and is
  currently `queued`. Its sealed source-manifest hash is
  `d63b174c70e770161c9444d6f0ed751b22ec5a359487ea8f76a0f63cb1a7b9b9`:

  | path | SHA-256 |
  | --- | --- |
  | `docs/plans/zircon_runtime/frameworks/01/failure-2026-09-08-resource-readiness-payload-type-erasure.md` | `4a985085131a3b197ccbbc22005e87f935a33a2c6b3bebe8f573d46f87370360` |
  | `zircon_runtime/crates/zr_resource/src/manager/resource_manager.rs` | `def276dda695adcf29cd8079b2d569327f7e984d2dbb9ad597a1acee145a9866` |
  | `zircon_runtime/crates/zr_resource/src/manager/tests/readiness_payload.rs` | `e9f91a6bd52dfe656000ecbe8cb80f4628ba9ddd602dbce2a98daf3cf522a7be` |

- The ticket executes a Windows PowerShell/rustfmt source-contract parse for the authority
  dispatch and all three lower-layer regressions. It explicitly defers fresh managed Cargo
  `zr_resource --lib` and Runtime facade gates, independent C/I/M review, canonical fixed
  return, and closeout. The previously recorded jobs `74f5113895dd40d78b1dcf7d1bf8a9fd`
  and `fede77172a564c74baa6ce25f9dcd425` remain supporting evidence only, not a substitute
  for this ticket's current-source terminal result.
- Failure remains `open`; no fixed return, commit, or notification is claimed. The external
  `E:\Git\zr_vm` dirty-worktree blocker remains explicitly retained for any gate that needs it.

### Ticket correction after coordinator materialization failure

- Prior ticket `0498a529f0eb474e82bc3693a9b9f8da` did not execute its command: the coordinator
  materialization phase returned `validation_copy_dependency_archive_failed` because the
  untracked owned regression file was included in the Git baseline dependency archive. This
  is coordinator input handling, not a source or test result.
- Corrected request `failure-roll-01a084c8-frameworks01-readiness-payload-20260919-r3` admits
  ticket `12de497a257845669b7eb8063c968239` with dependency root restricted to the tracked
  authority file; the test remains an owned overlay in the sealed manifest. Its manifest hash is
  `ce3c515472328ce9cc1a4f6d09458ca69f8f1fa4d4f76438ac57e934f1202ef1` and status is `queued`.
  The old ticket is retained as failed coordinator evidence and is not reused.

### Ticket correction after current-source contract assertion failure

- Ticket `12de497a257845669b7eb8063c968239` did materialize and execute, but its terminal
  result was `failed` at `2026-09-19T04:59:45.996063Z` with coordinator evidence
  `resource authority contract missing: typed_load_state`. The assertion searched the
  authority implementation file for a regression-test symbol that is defined in the owned
  `readiness_payload.rs` overlay; this is a validator-command error, not a source or test
  failure. Its sealed manifest was `ce3c515472328ce9cc1a4f6d09458ca69f8f1fa4d4f76438ac57e934f1202ef1`.
- The failed ticket remains retained as non-reusable evidence. A successor will keep the
  tracked authority file as its only dependency root and will assert authority symbols from
  `resource_manager.rs` separately from regression symbols in `readiness_payload.rs`, so the
  command matches the current source layout without changing source bytes.
- Corrected request `failure-roll-01a084c8-frameworks01-readiness-payload-20260919-r4` admitted
  ticket `935bf1a26ab248dfabe7f69694b83941` at `2026-09-19T05:05Z`. Its sealed manifest hash is
  `8fd15d1f537b567b96e0bd88ec02648f5963cf618446281697a6531dbb51536c`, with the tracked
  authority file as the sole dependency root; status is `queued` pending the coordinator
  terminal result. The prior ticket's command-anchor error is recorded in the rerun reason.

### Corrected source-contract ticket terminal result

- Corrected ticket `935bf1a26ab248dfabe7f69694b83941` completed `passed` at
  `2026-09-19T05:07:32.805033Z` (exit code 0), emitting
  `FRAMEWORKS01_READINESS_PAYLOAD_SOURCE_CONTRACT_PARSE_PASS`. Its sealed manifest hash is
  `8fd15d1f537b567b96e0bd88ec02648f5963cf618446281697a6531dbb51536c`.
- This result covers only the current-source static authority/test contract. Fresh managed
  `zr_resource --lib` and Runtime facade Cargo gates, independent C/I/M review, canonical
  fixed return and closeout remain pending; the external `E:\Git\zr_vm` dirty-worktree blocker
  is retained.

### 2026-09-21 independent source review (review-frameworks01-readiness-payload-r2)

- The current production/test owners match corrected ticket manifest
  `8fd15d1f537b567b96e0bd88ec02648f5963cf618446281697a6531dbb51536c`:
  `resource_manager.rs` `def276dda695adcf29cd8079b2d569327f7e984d2dbb9ad597a1acee145a9866`
  and `manager/tests/readiness_payload.rs`
  `e9f91a6bd52dfe656000ecbe8cb80f4628ba9ddd602dbce2a98daf3cf522a7be`. The failure record has
  only the expected receipt drift.
- `ResourceAuthority::readiness_source_update` dispatches `as_any()` through the erased payload
  reference (`payload.as_ref().as_any().type_id()`), so an `Arc<dyn ResourceData>` container is
  never mistaken for the concrete payload. The authority remains the single materialization
  boundary; no facade-side type special case, forced Loaded state, cache, or compatibility API is
  introduced.
- The focused regressions cover a concrete payload TypeId, replacement with a different concrete
  type while preserving the prior immutable generation, wrong-type `NotLoaded` behavior, and an
  explicitly nested `Arc<TestPayload>` whose own concrete type remains observable. The tests
  assert both current and prior generation rows, including `Arc::ptr_eq` separation and typed
  load-state transitions.
- Independent probes passed for erased dispatch, payload-map ownership, test mounting,
  concrete/replacement/nested-Arc regression symbols, and wrong-type rejection. Scoped Rust 1.94.1
  `rustfmt --check` and `git diff --check` passed for both attributed paths.
- Independent review result: `Critical=0, Important=0, Moderate=0`. This is source-contract
  evidence only. Fresh managed `zr_resource --lib` and Runtime facade Cargo gates, Runtime04
  upward/product acceptance, formal fixed return, and coordinator closeout remain pending; the
  historical 227/0 and 26/0 jobs are supporting evidence only and are not reused as current
  acceptance.
