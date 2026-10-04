---
handoff_kind: failure
status: open
created_at: 2026-08-31
summary_slug: runtime-interface-pointer-route-stale-serde-field-attributes
origin_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
fixing_plan: docs/plans/optimize/zircon_runtime_interface/12-ui-authoring-accessibility-input-diagnostic-status-operation-public-contract-current-source-review.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/01
fixing_child_dir: docs/plans/optimize/zircon_runtime_interface/12
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/ui/surface/pointer/route.rs
tests:
  - cargo build -p zr_resource --locked
  - cargo test -p zr_resource --locked --lib projection_snapshot_
---

# RuntimeInterface12: pointer route keeps serde field attributes after manual serde hard cut

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 来源执行切片：Frameworks01 managed Windows I1 support-layer validation
- 修复责任计划：`docs/plans/optimize/zircon_runtime_interface/12-ui-authoring-accessibility-input-diagnostic-status-operation-public-contract-current-source-review.md`
- 交接原因：失败位于 `zircon_runtime_interface` 的 pointer-route wire contract；Frameworks01 只消费该共享接口，最低共享原因与修复归 RuntimeInterface12。

## 失败现象与复现证据

Frameworks01 ran the managed Windows I1 support-layer validation from the shared current source:

```text
validate-matrix.ps1 -Package zr_resource -LibTests -TestFilter projection_snapshot_
  -TargetDir E:\cargo-targets\frameworks01-resource-identity-i1-green -VerboseOutput
```

The coordinator receipt is:

- job `c373ffe7a3164d06bd9eaabb1f75086b`;
- validation Session
  `validate-matrix:019ffe2b-296a-7023-9433-8654b9ea8f18:successor:814a10c25ca4470f8d8b98bae4f78982`;
- start `2026-08-31T07:39:50.117852+08:00`;
- finish `2026-08-31T07:41:59.762994+08:00`;
- release `2026-08-31T07:42:02.649508+08:00`, exit `1`;
- target and scratch remained under `E:`; no artifact was written to `C:`.

Cargo stopped while compiling `zircon_runtime_interface`, before `zr_resource` was compiled. Rustc
reported five `cannot find attribute serde in this scope` errors at `route.rs:53`, `55`, `66`, `68`
and `70`.

## 最低共享层根因

Current `route.rs` SHA-256 is
`a75a6782f5b3baaca2246b19092a132ae3240039647121c929765e2faa26c18d`. Its shared-worktree diff
hard-cuts `UiPointerRoute` from derived serde to manual `Serialize`/`Deserialize`, but leaves
field-level `#[serde(default)]` attributes on the production struct. Those attributes are valid only
inside a serde derive input; the manual wire DTO already owns the required defaults.

Coordinator ownership matrix request `ee816b4bb31848dba6e77dae30e83a0b` reports the exact path as
`modified / unowned / attribution_missing`, with no live lease. Frameworks01 did not claim or edit
the file. The failure was routed to RuntimeInterface owner task
`01a00797-56e0-70f1-a57c-dc3fb65263e8` with the exact hash, diagnostics and job receipt.

## 架构修复验收

- Establish one legal owner for the whole current pointer-route blob; do not split or reattribute
  only the five lines.
- Remove the stale production-struct serde field attributes while preserving the manual wire DTO's
  defaulting semantics and pointer-route serialization compatibility required by the owning plan.
- Run the owner-focused pointer route serde tests and compile `zircon_runtime_interface`.
- Return the final file hash and integration receipt. Frameworks01 then reruns the exact managed
  `zr_resource` command above.

## 禁止临时方案

- Frameworks01 must not claim, rewrite or commit this foreign mixed blob.
- Do not restore derived serde merely to make the attributes compile; the current manual serializer
  intentionally projects the shared routing path back to the wire `bubbled` field.
- This failure does not invalidate the I1 static GREEN. It blocks managed compile/test evidence only,
  so Frameworks01 continues non-validation Resource support work.

## 修复结果与回传

Open state: `源码契约修复已确认 / immutable managed Cargo 与来源计划复验待完成`。

RuntimeInterface03 now owns the complete current `UiPointerRoute` blob under lease request
`bb5441736be9431ba4c6665fcdf0ce27`. The manual serde hard cut is internally consistent:

- `UiPointerRoute` has no field-level `#[serde(...)]` attributes;
- the derived `WirePointerRoute` deserialization DTO retains the six required `#[serde(default)]`
  defaults for backward-compatible wire input;
- serialization still emits the stable `bubbled` route projection and preserves borrowed route
  traversal without an allocation in the hot path.

Current source SHA-256:

`zircon_runtime_interface/src/ui/surface/pointer/route.rs`
`1b784b39603ed6eb8c8670e84c0a21708d3c10c202c42e3078ba3b8dde985ea2`

Scoped static contract, rustfmt, and diff checks are green. A fresh managed Windows Rust 1.94.1
`--locked --release` `zircon_runtime_interface` gate is queued; this record remains `open` until
that gate and the originating Frameworks01 `zr_resource` rerun both pass.

The shared batch is ticket `e5ad38bacd664fdb87ad7d4fa9acb22c`, submitted by request
`92e7978469ca4402831ac77ddef98c80` (receipt `fc62c260f6254b9aafcc6702cba4f517`).

### 2026-09-01 immutable-copy retry

Fresh static evidence remains green on the current route bytes: the input-routing receipt contract
passed `9/9`, Rust 1.94.1 rustfmt passed for the exact input batch, and scoped diff-check passed.
Validation-copy job `cf49ab8dbb064c079878a00d9f61427f` was then accepted with only
`route.rs` as an explicit overlay, but Cargo did not start. Artifact governance removed the copy
during materialization because the concurrently introduced shared path
`E:\cargo-targets\zircon-engine\cache\cargo-metadata-home` was classified as unmanaged.

This is not a pointer-route compile result. The copy is terminal and cannot be reused; no retry is
authorized until the Coordinator-owned metadata-home lifecycle is committed and loaded.

### 2026-09-01 pinned baseline source-read regression

After the metadata-home path became governed, successor copy
`1b7b951a2c9c4a1f8a12c04e290100c5` passed artifact preflight but failed in
`closure_planning` before Cargo. Durable code
`validation_copy_compile_time_source_git_failed` records operation
`git_cat_file_compile_time_sources` and stderr
`fatal: not a git repository (or any of the parent directories): .git`.

The failure is in the Coordinator-owned pinned planner view: baseline Git object reads must use
the logical repository, not the ephemeral non-Git metadata view. This copy is terminal and no
pointer-route result is claimed; replay waits for that active Tooling repair and daemon alignment.

### 2026-09-01 current-baseline owner reconciliation

Current `main` already contains the structurally correct pointer-route repair in commit
`5798051603e7f7f565538125c9aba96d5beabae2`. The source remains byte-identical at SHA-256
`1b784b39603ed6eb8c8670e84c0a21708d3c10c202c42e3078ba3b8dde985ea2`: production
`UiPointerRoute` has no field-level serde attributes, while the nested derived `WirePointerRoute`
retains the six compatibility defaults.

The dedicated owner Session `root-runtime-interface12-pointer-route-return-r2-20260901` reclaimed
the exact source and this canonical record under lease request
`23e50785b0b0417e875759b43da6a0b3`; attribution request
`db31f23ee6d24c5f9584b7a6197ddc7a` succeeded. Immutable snapshot `2749` was created by request
`4f2c31f7175d49ce9911c4e88b0db38e`. The combined route-sharing and input-routing static suites
pass `16/16`; scoped Rust 1.94.1 rustfmt and diff integrity also pass.

One managed Windows batch was submitted as request
`runtime-interface12-pointer-route-serde-return-20260901-r1` to run release library tests for both
`zircon_runtime_interface` and `zr_resource`. Admission rejected it before ticket creation with
`validation_ticket_external_worktree_dirty` for external worktree `E:\\Git\\zr_vm`. No Cargo
process started and no dynamic compile/test evidence is claimed. The external worktree was not
modified. This Failure remains open pending an admitted managed current-source batch.

## 2026-09-02 coordinated current-source batch receipt

RuntimeInterface03 renewed the copy-complete lease for `route.rs`, this canonical Failure, and the
companion interface failures under request `437bd9bac8194aaf9eaff5849b4da574`; attribution request
`cd08b4bd45e24aa19c63fcb63a67cf33` accepted the exact current hashes. `route.rs` remains
byte-identical at SHA-256
`1B784B39603ED6EB8C8670E84C0A21708D3C10C202C42E3078BA3B8DDE985EA2`: the production route has no
field-level serde attributes, while the manual wire DTO retains its compatibility defaults. The
focused input-routing contract remains `9/9` GREEN.

Combined managed request `runtime-interface03-runtime200-current-source-20260902-r1` was submitted
for Windows Rust 1.94.1 release tests of `zircon_runtime_interface` and `zircon_runtime`, but
admission rejected it before ticket creation and before Cargo execution with
`validation_ticket_external_worktree_dirty` for external worktree `E:\\Git\\zr_vm`. No compile,
test, performance, integration, or fixed-return receipt is claimed. The external worktree remains
untouched, and this Failure remains `open` pending an admitted current-source managed batch and the
originating Frameworks01 rerun.

After this receipt was appended, document-only lease request
`134b23729daf4ba39e17f989d6773017` and attribution request
`cde75b5e97f64013bb50d587aa61c7c7` refreshed all three canonical Failure artifacts.

## 2026-09-08 Focused Owner Validation Admission

Stable fixing Session `failure-roll-01a07160-interface12`, baseline `601`,
continues this lifecycle. Registration request
`fd446ff9d53e48a8aefe163ea9e648c4` was accepted and reconciled as completed;
it was not resubmitted after the client timeout. Transfer
`16caf52babdd4ba2bcc4bd012b6f648e` preserves the exact archived
`root-runtime-interface03-activate-link-failure-20260831` source and record
hashes. Snapshot `3106`, request `c5927e63b46b464f9e8ddef584024248`, freezes
the unchanged `route.rs` at
`1b784b39603ed6eb8c8670e84c0a21708d3c10c202c42e3078ba3b8dde985ea2`
and the previous record at
`9cd4ca318bdd7c97516c9e176adaa716036e150dc7c0c165b374f1e99b8d60e7`.
There is no source diff against HEAD. Current tracing confirms the manual
serializer's `bubbled` wire projection, six deserializer defaults and existing
roundtrip assertions in
`ui_binding_update_contract_represents_attribute_state_and_ecs_domains`.

Request `interface12-pointer-serde-and-resource-20260908-r1` submitted the
owned source hash with Windows Rust/Cargo 1.94.1 and command:

```text
cargo +1.94.1 test -p zircon_runtime_interface -p zr_resource --locked --release --lib
```

This would cover the interface library, pointer-route roundtrip and originating
resource projection tests. Admission returned
`validation_ticket_external_worktree_dirty` for `E:/Git/zr_vm` before ticket
creation or Cargo execution. No dynamic acceptance is claimed. The user has
explicitly excluded `zr_vm`; that repository is untouched, this item is
suspended, and independent failures continue without polling or resubmission.

## 2026-09-11 rolling repair admission

- Stable fixing session `failure-roll-01a084c8-interface12-pointer-serde` reclaimed
  this canonical record and `zircon_runtime_interface/src/ui/surface/pointer/route.rs`;
  lease heartbeat and source ownership were renewed at HEAD
  `c37155ba304740b3762b20585f77fb53a6da47fb`.
- Snapshot `3410` sealed the exact current bytes. The route source hash is
  `1b784b39603ed6eb8c8670e84c0a21708d3c10c202c42e3078ba3b8dde985ea2`; the failure
  record hash at submission was
  `85efaa215635f72841b87ba59be087754f572b29029ddf77dc2913f403c068f4`.
- Owner/originating batch request `interface12-pointer-serde-resource-20260911-r1`
  requested Windows Rust/Cargo 1.94.1:
  `cargo +1.94.1 test -p zircon_runtime_interface -p zr_resource --locked --release --lib`.
  Admission rejected it before ticket creation with
  `validation_ticket_external_worktree_dirty` for `E:\Git\zr_vm`; no Cargo process,
  dynamic test, upward acceptance, review, fixed return, or closeout evidence exists.
- The external repository was not modified. This lifecycle remains open and the
  session is waiting for the external owner to restore a clean validation state.

## 2026-09-19 rolling successor source-contract admission

- Successor Session `failure-roll-01a084c8-interface12-pointer-serde-r4` reclaimed
  the canonical failure record and `zircon_runtime_interface/src/ui/surface/pointer/route.rs`
  under coordinator transfer fingerprint
  `ea421400f3c32b13e32dbd5938f643a67cf964302a3389668340acbf39af8df4` at baseline
  epoch `611`; the source remains byte-identical at SHA-256
  `1b784b39603ed6eb8c8670e84c0a21708d3c10c202c42e3078ba3b8dde985ea2`.
- Static source-contract request
  `failure-roll-01a084c8-interface12-pointer-serde-20260919-r1` admitted ticket
  `a2452872d29245c1b102ca5020def5d6` with sealed manifest hash
  `85c09f3830e7252a20909aa5614e7e4de998f5ff5820f581647a170ddfac6677`; status is
  `queued` pending the coordinator terminal result.
- The ticket checks the manual `UiPointerRoute` serde boundary, stable `bubbled`
  projection, nested `WirePointerRoute` defaults, and rustfmt. It is static-only;
  fresh managed Interface/Resource Cargo gates, the Frameworks01 originating rerun,
  independent C/I/M review, fixed return, closeout, and the external
  `E:\Git\zr_vm` clean-worktree prerequisite remain deferred.

### 2026-09-19 source-contract terminal result

- The coordinator terminalized ticket `a2452872d29245c1b102ca5020def5d6` as
  `passed` at `2026-09-19T06:17:53.473444Z`; managed validation job
  `4fa65a8fba79405398bc259771277e4a` exited `0` with stdout marker
  `RUNTIMEINTERFACE12_POINTER_ROUTE_SERDE_SOURCE_CONTRACT_PARSE_PASS` and an
  empty stderr tail. Cleanup completed in ticket event `10869`.
- This remains a static source-contract result only. Fresh managed
  `zircon_runtime_interface`/`zr_resource` Cargo gates, the Frameworks01
  `projection_snapshot_` rerun, independent C/I/M review, canonical fixed
  return, and failure closeout remain pending; the external `E:\Git\zr_vm`
  dirty-worktree blocker is unchanged.

## 2026-09-21 independent source review receipt

- Reviewer Session `review-interface12-pointer-serde-r4` inspected the current
  owned route source without editing it. The reviewed `route.rs` SHA-256 is
  `1b784b39603ed6eb8c8670e84c0a21708d3c10c202c42e3078ba3b8dde985ea2`.
- The review re-ran `rustfmt +1.94.1 --edition 2021 --config
  skip_children=true --check` and scoped `git diff --check`; both passed with
  markers `INTERFACE12_RUSTFMT_PASS` and `INTERFACE12_DIFF_CHECK_PASS`.
- The source-contract probe passed as
  `INTERFACE12_POINTER_ROUTE_SOURCE_REVIEW_PASS`. It verified that production
  `UiPointerRoute` has no stale serde field attributes, uses manual
  `Serialize`/`Deserialize`, projects the stable `bubbled` field through the
  reverse route wrapper, and keeps exactly six `#[serde(default)]` fields on
  the nested `WirePointerRoute` DTO (`modifiers`, `activation_phase`,
  `hit_path`, `pressed`, `click_target`, and `release_inside_pressed`). The
  serialized field count remains `19`.
- Independent review result: **Critical=0 / Important=0 / Moderate=0**. No
  source change was needed and no foreign owner scope was absorbed.
- This receipt does not claim Cargo or product acceptance. Fresh managed
  `zircon_runtime_interface`/`zr_resource` Cargo validation and the Frameworks01
  `projection_snapshot_` consumer rerun remain pending because the external
  `E:\Git\zr_vm` worktree is dirty. Canonical `fixed-*` return, closeout, and
  WeCom notification remain pending until those gates produce matching,
  source-bound evidence.
