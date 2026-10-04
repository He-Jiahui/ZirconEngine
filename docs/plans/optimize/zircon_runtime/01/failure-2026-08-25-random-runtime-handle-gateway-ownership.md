---
handoff_kind: failure
status: open
created_at: 2026-08-25
summary_slug: random-runtime-handle-gateway-ownership
origin_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
fixing_plan: docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/01
fixing_child_dir: docs/plans/optimize/zircon_runtime/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/runtime/handle/mod.rs
  - zircon_runtime/src/core/runtime/handle/random.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
  - zircon_runtime/src/core/runtime/random/mod.rs
tests:
  - tools/tests/test_frameworks_01_random_contract_kernel_boundary.py
  - cargo check -p zircon_runtime --lib --locked
---

# Runtime01: RandomService handle gateway has no executable current-source owner

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 来源执行切片：deterministic random contract/kernel owner hard cut
- 修复责任计划：`docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md`
- 交接原因：`core/runtime/handle/mod.rs` is a Runtime lifecycle gateway owned by an archived
  Runtime01 Session and is outside Frameworks01 r12's immutable scope.

## 失败现象与复现证据

At HEAD `0fd7df4ecdd157f9505cd51013780e3225cfb83c`, coordinator baseline epoch 436:

- `core/runtime/handle/mod.rs` SHA-256 is
  `4cfb5ceec8e2bdaa7490714e690a2b23b854a6ecba4627c0212ea475ae04bc1c`;
- `core/runtime/handle/random.rs` SHA-256 is
  `64a6f846854ded9baed91ea29f45c7de3b4dff09d6a634fe3d30697855ba8f3d`;
- `core/runtime/runtime.rs` SHA-256 is
  `32f1e225e54d5ed236d3b7572c8a29aafc8a74c2703024d025bf8dfe34888221`;
- `core/runtime/state/core_runtime_state.rs` SHA-256 is
  `da89f936fb8f5c160df5d8c8e2283ef7b8afb2732979356e6b2875af40531129`.

The Random contract/kernel implementation itself has a current executable owner. Transfer preview
request `1102cbf209764560b1c72abde14cf91a` proved 10/10 cancelled-r10 blobs unchanged, and apply request
`9b9931b9b0664362837f44c73b8cd044` transferred fingerprint
`ce98561b713ab6784d030145e8e9ad7d01731a4898eca55ad8417bbecbd0161c` to Frameworks01 r12 without
rewriting source. The current owner guard is 13/13 GREEN in 30.915 seconds.

## 最低共享层根因

`core/runtime/handle/mod.rs` contains the required single `mod random;` declaration. Its whole blob
is still attributed to archived Session `runtime-core-lifecycle-m0-veto-atomicity-20260815`, whose
plan family is Runtime01. The coordinator reports a stale content hash, stale baseline, no live
lease, and an owner that cannot execute. Frameworks01 r12 does not include this path in its immutable
write scope and therefore does not edit or claim it.

The neighboring Runtime construction blobs cannot be treated as substitutes. `runtime.rs` and
`core_runtime_state.rs` combine RandomService construction with Time, State, and module-lifecycle
changes. Moving only the `mod random;` line elsewhere, adding a forwarding module, or exposing the
implementation through a compatibility facade would break the approved hard-cut topology.

## 架构修复验收

- Register or rotate an executable Runtime01 Session whose immutable scope includes the complete
  current `core/runtime/handle/mod.rs` blob and the focused Runtime handle tests.
- Re-review the current blob against Runtime01 lifecycle/module-boundary rules. Preserve exactly one
  private Random handle module declaration and do not add root or framework implementation exports.
- Confirm that `CoreHandle::random_service`, `CoreRuntime::random_service`, and
  `CoreRuntimeInner::random_service` remain the only three shared-borrow authority accessors, with
  the backing field private and no `Clone`, `Copy`, value return, or mutable return escape.
- Run the 13-test Frameworks01 Random owner guard and a coordinator-managed Windows
  `cargo check -p zircon_runtime --lib --locked`. If a foreign lower-layer compile error intervenes,
  return its exact current-source fingerprint rather than changing another owner.
- Return a canonical fixed record to Frameworks01 with current hashes and validation receipts.

## 禁止临时方案

- Do not move `handle/random.rs` back into framework contracts or duplicate its accessor in another
  gateway.
- Do not use `pub use`, an alias module, shim, feature-gated legacy path, or compatibility wrapper.
- Do not submit the transferred Random implementation without the current gateway blob and managed
  Runtime validation.
- Do not claim BLAKE3 performance, power, replay, registry, CPU/GPU parity, or algorithm optimality;
  those remain Runtime22 work with separate profiling entry gates.

## 修复结果与回传

Open. Root-cause and owner routing are complete. Runtime01 scope rotation, implementation-owner
review, managed validation, fixed return, Frameworks01 milestone acceptance, commit, and WeCom
synchronization remain pending.

### 2026-09-05 lifecycle record normalization

The original artifact was stored in the origin directory despite naming Runtime01 as its fixing
owner. It has been moved to the declared fixing child directory, retaining its creation date,
summary slug, origin and fixing plans, and original evidence. Its pre-move SHA-256 was
`d2e9452998a89622b81777752bc45a8481e94ce6d9b9a4102004ab0347610949`.
Runtime01 Session `failure-roll-01a07160-runtime01` is registered as `resolving_failure` at
baseline 599 / HEAD `d3174741300b272774c56a37465f043a03debd6c`. Exact artifact leases
`df6c4281f846400e9fd4cf8d2dc478cd` and pre-edit attribution
`0477209701c641ba848b1498546e7682` bind this move. Source ownership, managed compilation,
and the canonical fixed return remain pending; this normalization is not acceptance evidence.

### 2026-09-19 rolling successor source reconciliation

The stable fixing Session `failure-roll-01a084c8-runtime01-random-gateway-r1` was
registered against the Runtime01 fixing plan at baseline epoch `611`. The failure
record, the complete `handle/mod.rs` gateway, `handle/random.rs`, and the existing
Frameworks01 random-boundary test were transferred from archived owners under
fingerprint
`f38f998c3be12e5f6be91fa584c097ca2c259b05f7433806189958dd04fe0794` and leased
without reverting foreign changes.

The transfer intentionally excludes `runtime.rs` and
`state/core_runtime_state.rs`: those construction/field blobs remain attributed to
the separate Runtime11/Runtime02 owners and are not silently absorbed by this
gateway-ownership repair. They remain explicit downstream validation dependencies.
The current transferred hashes are:

- `zircon_runtime/src/core/runtime/handle/mod.rs`:
  `4cfb5ceec8e2bdaa7490714e690a2b23b854a6ecba4627c0212ea475ae04bc1c`
- `zircon_runtime/src/core/runtime/handle/random.rs`:
  `85bc1e4a03f1809d4dc789e1ed843a0d3ddf9976adb50bb0157366c822c432bd`
- `tools/tests/test_frameworks_01_random_contract_kernel_boundary.py`:
  `28cba9e67463348b31a7516b9a4ab2331949ab40b879fbf6afb91409bbf38b67`

The exact local boundary suite executed 18 tests in 63.377 seconds with zero
failures or errors. It confirms the single private `mod random;` gateway,
borrowed `RandomService` accessors, private authority storage, no old framework
implementation imports, and the contract/kernel dependency boundary. This is
current-source evidence only; the managed Windows Runtime01 Cargo check and
upward Frameworks01 acceptance remain pending.

### 2026-09-19 managed static successor ticket

The owned four-path current-source overlay was submitted as ticket
`7b36f08a5d4143fa9cdfee184a72a015` (request
`failure-roll-01a084c8-runtime01-random-gateway-20260919-r1`) with source-manifest
hash `367caec4638655702a7dc95df3598c45a42c35bfad2985f3448dc4e098b4bee6`.
Its managed source-contract command checks the single private gateway declaration,
the borrowed `RandomService` accessor, and the existing Frameworks01 boundary
guard anchors, emitting
`RUNTIME01_RANDOM_HANDLE_GATEWAY_CURRENT_SOURCE_CONTRACT_PASS`. The ticket is
static-only (`staticParseOnly=true`, `upwardAcceptance=false`) and is queued;
no managed job or terminal result exists yet.

The dynamic Runtime01 `cargo check -p zircon_runtime --lib --locked` and the
Frameworks01 upward gate remain blocked by
`validation_ticket_external_worktree_dirty:E:/Git/zr_vm`. Review, canonical
fixed return, and closeout are still pending.

The static ticket executed as managed job `4aec6cef598b4748b3509ae33619255c`
/ run `7b36f08a5d4143fa9cdfee184a72a015`, exited `0`, and emitted the exact
marker `RUNTIME01_RANDOM_HANDLE_GATEWAY_CURRENT_SOURCE_CONTRACT_PASS`.
Coordinator cleanup completed. This validates only the transferred gateway
contract; the managed Runtime01 Cargo check, cross-owner construction gate,
review, canonical return, and closeout remain pending.

### 2026-09-21 independent source review (review-runtime01-random-gateway-r1)

- The transferred production and boundary-test paths match the passed ticket's immutable
  manifest `367caec4638655702a7dc95df3598c45a42c35bfad2985f3448dc4e098b4bee6`:
  `handle/mod.rs` `4cfb5ceec8e2bdaa7490714e690a2b23b854a6ecba4627c0212ea475ae04bc1c`,
  `handle/random.rs` `85bc1e4a03f1809d4dc789e1ed843a0d3ddf9976adb50bb0157366c822c432bd`, and
  `tools/tests/test_frameworks_01_random_contract_kernel_boundary.py`
  `28cba9e67463348b31a7516b9a4ab2331949ab40b879fbf6afb91409bbf38b67`. The failure-record
  hash has the expected receipt drift only.
- `handle/mod.rs` retains exactly one private `mod random;` declaration and exposes no framework
  implementation or public gateway module. `handle/random.rs` imports the runtime-owned kernel
  `RandomService`, exposes one shared-borrow `CoreHandle::random_service(&self) -> &RandomService`,
  and keeps the checkpoint/seed regressions on the runtime owner. No clone/copy/value/mutable
  authority escape or compatibility forwarding path was found in the attributed scope.
- The boundary guard's current 18-test Python suite passed (`Ran 18 tests ... OK`). It verifies
  the contract/kernel physical split, hidden assembly ownership, old-framework import exclusion,
  single-authority accessor surface, checkpoint restoration admission, no implicit stream copy,
  and adversarial scanner cases for aliases, glob imports, raw selectors, and escaping accessors.
  Scoped Rust 1.94.1 `rustfmt --check` and `git diff --check` passed for the owned Rust/Python
  paths.
- Independent review result: `Critical=0, Important=0, Moderate=0`. This is source and boundary
  evidence only. Managed Windows `cargo check -p zircon_runtime --lib --locked`, Runtime
  construction/state cross-owner reconciliation, Frameworks01 upward acceptance, canonical fixed
  return, and coordinator closeout remain pending; the external `E:\Git\zr_vm` dirty-worktree
  blocker is retained and no algorithm/performance claim is made.
