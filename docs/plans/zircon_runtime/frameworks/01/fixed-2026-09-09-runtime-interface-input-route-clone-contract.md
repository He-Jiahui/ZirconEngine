---
handoff_kind: fixed
status: fixed
created_at: 2026-08-31
summary_slug: runtime-interface-input-route-clone-contract
origin_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
fixing_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/01
fixing_child_dir: docs/plans/optimize/zircon_runtime_interface/03
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/ui/dispatch/input/result.rs
  - zircon_runtime_interface/src/ui/surface/hit.rs
tests:
  - validate-matrix.ps1 -Package zr_resource -LibTests -TargetDir E:\cargo-targets\frameworks01-resource-identity-i1-green -VerboseOutput
resolved_at: 2026-09-09
---

# Frameworks01: Runtime Interface input route promises an unproven Clone iterator

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 来源执行切片：Resource generation, event ordering and durable-artifact validation.
- 修复责任计划：`docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md`
- 交接原因：The lower iterator contract belongs to Runtime Interface UI.

## 失败现象与复现证据

Frameworks01 reran the full managed Windows `zr_resource` library validation from shared current
source after the Resource generation, event-order and durable-artifact hard cuts:

```text
validate-matrix.ps1 -Package zr_resource -LibTests
  -TargetDir E:\cargo-targets\frameworks01-resource-identity-i1-green -VerboseOutput
```

Coordinator evidence:

- job `8af64b6fdf4a4d928cc31fb92ea934ae`;
- validation Session
  `validate-matrix:019ffe2b-296a-7023-9433-8654b9ea8f18:successor:fd746df3825b4f80b933d46218f4cf69`;
- start `2026-08-31T08:47:50.502965+08:00`;
- finish `2026-08-31T08:53:16.982510+08:00`;
- release `2026-08-31T08:53:26.393098+08:00`, exit `1`;
- target and managed scratch remained on `E:`; no artifact was written to `C:`.

Cargo stopped while compiling `zircon_runtime_interface`, before it compiled the changed
`zr_resource` crate. Rustc 1.94.1 reported E0277 at
`zircon_runtime_interface/src/ui/dispatch/input/result.rs:116`: `physical_bubble_route()` promises
`Clone`, but the opaque iterator returned by `UiHitPath::bubble_route()` does not expose that trait
bound. The same fingerprint contains six unrelated non-blocking unused-import warnings.

## 最低共享层根因

Current `result.rs` SHA-256 is
`f420794ef68e44e3a1ed37288fede517b96813101c06aa8d894d7f6dfe576bf2`. The error is an API
contract mismatch between two Runtime Interface UI route layers, not a Resource implementation
failure.

Coordinator ownership matrix request `10d393e3fe7b4c559a3dc867b4d210e1` identifies owner Session
`root-runtime-interface03-activate-link-failure-20260831`, status `waiting_validation`. Session-show
request `3102cd55692f4c88afadbff14740ed37` confirms the exact file is already in that Session's
immutable write scope under RuntimeInterface03. Frameworks01 did not claim or edit the source.

## 架构修复验收

- Preserve one coherent public iterator contract across `UiHitPath::bubble_route()` and
  `UiPointerRoutingReceipt::physical_bubble_route()`; either prove and expose `Clone` from the
  lower layer or remove the unsupported upper-layer promise based on actual consumers.
- Add or retain owner-focused tests that compile the public route iterator and cover forward,
  reverse and repeated traversal semantics required by callers.
- Compile and test `zircon_runtime_interface` in the owner Session and return the final file hash
  plus validation/integration receipt.
- Frameworks01 then reruns the exact managed `zr_resource` command above.

## 修复结果与回传

- 根因：The upper physical_bubble_route promise required Clone while the authoritative UiHitPath borrowed iterator contract was not exposed consistently across the migrated UI route layers.
- 架构修复：Kept one borrowed route authority: UiHitPath::bubble_route exposes DoubleEndedIterator, ExactSizeIterator and Clone, and UiPointerRoutingReceipt::physical_bubble_route forwards that proven contract without collecting or allocating.
- 验证：Managed Windows Rust 1.94.1 route regression passed in job dd18d653fe3c494c91fbe57f5959580a; managed zr_resource locked release library job 64402e468e94498f96357bb26d716f81 passed 224 with 0 failed and 10 explicitly ignored; focused routing contract 9/9 passed with rustfmt and diff checks.
- 回传：Returned after current-source hashes matched the repaired result.rs and hit.rs. The original Resource compile blocker is absent; unrelated ignored performance/readiness cases remain explicitly outside this return.

## 2026-08-31 Current-Source Copy-Complete Reconciliation

The lower authoritative `UiHitPath::bubble_route()` implementation is part of the same current
Runtime Interface UI migration and explicitly exposes the borrowed
`DoubleEndedIterator + ExactSizeIterator + Clone` bound over `root_to_leaf.iter().rev().copied()`.
The upper `UiPointerRoutingReceipt::physical_bubble_route()` promise therefore remains valid and
must not be weakened to manufacture a compile fix.

- Current `zircon_runtime_interface/src/ui/surface/hit.rs` SHA-256:
  `81F28910DCC5F634C984E76F9B13BEBF473BF18A8F1C1B272199A5B09F01FF99`.
- Current `zircon_runtime_interface/src/ui/dispatch/input/result.rs` SHA-256:
  `BC41084F0E62240CEDB7DDE85F91FD20347B900181F2BED5CCFEDAB04ADDB195`.
- The owner Session now leases both exact paths and the input-routing contract test; the complete
  source closure must include `hit.rs` so the managed copy can prove the lower bound. Batch46 also
  removed a temporary reverse-route allocation from `UiHitPath::with_route` without changing this
  lower iterator contract.
- Focused static routing contract remains `9/9` GREEN. A copy-complete snapshot/managed ticket is
  pending coordinator capacity; no source bytes were changed in this reconciliation.

The record remains open until that managed current-source interface compile/test returns terminal
evidence; Frameworks01 may then rerun its exact `zr_resource` command. The copy-complete Batch47/48
snapshot `2744` was submitted as batched request
`runtime-interface03-batch47-48-20260901-zrvm-dirty-r1`, but the coordinator rejected it before
ticket creation with `validation_ticket_external_worktree_dirty` for external worktree
`E:\\Git\\zr_vm`. No Cargo run or terminal compile/benchmark evidence is claimed; the external
worktree remains untouched.

## 2026-09-01 clone-bound focused retry

- The owner lease covers `result.rs`, `hit.rs`, and this canonical failure artifact under
  `root-runtime-interface03-activate-link-failure-20260831`.
- The focused owner test now clones the borrowed `physical_bubble_route()` iterator before
  independent forward, reverse, and repeated traversal. Static contract result: `9/9` passed;
  scoped rustfmt and `git diff --check` also passed.
- Current source hashes are `result.rs` `BC41084F0E62240CEDB7DDE85F91FD20347B900181F2BED5CCFEDAB04ADDB195`
  and `hit.rs` `81F28910DCC5F634C984E76F9B13BEBF473BF18A8F1C1B272199A5B09F01FF99`.
- Managed Windows validator job `cb770fd5b03042dca334ac632c4ad623` reached Cargo but exited `101`
  because the shared checkout `Cargo.lock` is stale under required `--locked` mode.
- Copy-complete coordinator request `aa5cf2cc9bdb439baf10e74144419f06` was accepted then failed
  before ticket creation with `validation_ticket_external_worktree_dirty` for `E:\\Git\\zr_vm`.
  No terminal compile/test evidence is claimed; the handoff remains open pending a clean external
  worktree and a fresh managed validation ticket.

## 禁止临时方案

- Frameworks01 must not claim, rewrite or commit this foreign Runtime Interface input blob.
- Do not allocate or collect the route merely to manufacture `Clone`; the route remains a borrowed
  exact-size iterator over the existing hit path.
- This failure blocks managed compile/test evidence only. Frameworks01 continues durable Resource
  profile and plan-record work while the owner closes it.

## 2026-09-02 coordinated current-source batch receipt

RuntimeInterface03 renewed the copy-complete lease for `result.rs`, `hit.rs`, this canonical
Failure, and the companion interface failures under request
`437bd9bac8194aaf9eaff5849b4da574`; attribution request
`cd08b4bd45e24aa19c63fcb63a67cf33` accepted the exact current hashes.

- `result.rs`: `BC41084F0E62240CEDB7DDE85F91FD20347B900181F2BED5CCFEDAB04ADDB195`.
- `hit.rs`: `81F28910DCC5F634C984E76F9B13BEBF473BF18A8F1C1B272199A5B09F01FF99`.
- The focused borrowed-route contract remains `9/9` GREEN and proves cloned forward, reverse, and
  repeated traversal without collecting the route.

Combined managed request `runtime-interface03-runtime200-current-source-20260902-r1` was submitted
for Windows Rust 1.94.1 release tests of `zircon_runtime_interface` and `zircon_runtime`, but
admission rejected it before ticket creation and before Cargo execution with
`validation_ticket_external_worktree_dirty` for external worktree `E:\\Git\\zr_vm`. No compile,
test, performance, integration, or fixed-return receipt is claimed. The borrowed iterator contract
remains unchanged, the external worktree remains untouched, and this Failure remains `open`.

After this receipt was appended, document-only lease request
`134b23729daf4ba39e17f989d6773017` and attribution request
`cde75b5e97f64013bb50d587aa61c7c7` refreshed all three canonical Failure artifacts.

## 2026-09-08 canonical record placement

The original open record was stored in the origin child directory instead of its
declared fixing directory. Audited transfer
`93f845d9d38c71628995db528d3e0e01292bc43c125bae8c9498debf98630fc8`
adopted it from the archived source owner and reserved the canonical destination
for `failure-roll-01a07160-interface03`. Preimage snapshot 3202 preserves every
original byte at hash
`dba5b263bb1119d061a90831581c7a12d4f4533733c4ce00d88d5cea265481e2`.
The same lifecycle (created_at, summary_slug, origin and fixing plans) is now
located under Interface03; required headings and source executor fields are
normalized without deleting original evidence. This is a placement repair, not
failure return. Existing managed ticket identities and owners remain unchanged.
The historical origin-plan receipt's old path remains historical evidence.

## 2026-09-08 source-matched lower and original upward execution

Current result.rs and hit.rs still match the final hashes above and the frozen
`interface03-v8-layout-3199-20260908` source input. The lower iterator regression
`ui::dispatch::input::result::tests::pointer_routing_receipt_physical_bubble_route_is_bidirectional_and_repeatable`
actually passed in full interface job `dd18d653fe3c494c91fbe57f5959580a`, including
the cloned independent traversal. Its original log is
`interface-project-identity-3170-20260908/results/interface-library-3170.log`.
Neither lower source file changed between those inputs.

The original upward package still exists at `zircon_runtime/crates/zr_resource`.
Managed Windows job `64402e468e94498f96357bb26d716f81` executed its entire library
with no default features and `--locked`: 224 passed, zero failed, ten ignored.
The input manifest is
`58164b9667051219bec66342bcb79d9325648eb2cfcea1f3ad307ebc743c1abd`;
receipt and log are `results/interface03-clone-resource-upward-3199.{json,log}`.
The ten ignored cases include explicitly separate resource performance and RED
readiness harnesses; they are not passing evidence for those independent goals.

This operational managed run proves the original Resource compilation blocker
is absent for the exact frozen sources. Formal fixing-Session acceptance and
independent closeout review remain required. Existing tickets and their owners
were not rewritten or resubmitted; the excluded zr_vm prerequisite remains
suspended. No failure return or commit is claimed yet.
