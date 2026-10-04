---
handoff_kind: failure
status: open
created_at: 2026-09-08
summary_slug: runtime-v8-abi-inventory-test-drift
origin_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
fixing_plan: docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
origin_child_dir: docs/plans/optimize/zircon_runtime_interface/03
fixing_child_dir: docs/plans/zircon_runtime/runtime/10
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/tests/abi_safety_contracts.rs
  - zircon_runtime_interface/src/runtime_api/abi/api_table.rs
  - zircon_runtime_interface/src/runtime_build_set/interface_spec_v1.json
  - zircon_runtime_interface/build.rs
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/architecture/runtime-interface-cdylib-loader.md
  - docs/architecture/runtime-interface-convergence.md
tests:
  - cargo test -p zircon_runtime_interface --no-default-features --locked --lib tests::abi_safety_contracts::
  - cargo test -p zircon_runtime_interface --no-default-features --locked --lib runtime_api::abi::api_table::
---

# Runtime10: V8 ABI inventory test drift

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md`
- 来源执行切片：Interface03 full interface-library acceptance.
- 修复责任计划：`docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md`
- 交接原因：Runtime10 owns the dynamic API and ABI inventory.
- Session: `failure-roll-01a07160-runtime10`.
- The current frozen InterfaceSpec, generated version and production table are V8.
  This handoff repairs stale test inventory after that existing migration.

## 失败现象与复现证据

Windows locked/static job `dd18d653fe3c494c91fbe57f5959580a` executed the complete
interface library: 754 passed, 11 failed, 101 ignored. Its immutable input was
`interface-project-identity-3170-20260908`, manifest
`1f5a61e72075db714e26e348064c43abf268af11edf80465b15536ef40f639c9`.

`function_table_field_counts_match_runtime_10_inventory` observed 28 fields but
expected 25. `runtime_10_version_strategy_rejects_in_place_table_shape_changes`
expected generator version 7 while the frozen InterfaceSpec and generator use 8.

## 最低共享层根因

V8 adds request, poll and cancel viewport-pick slots. The explicit operation
inventory already lists all 26 function pointers plus two header fields, but the
field-count assertion and generator-version assertion retained V7 values. The
discovery helper also enumerated only ApiV1 through ApiV7, allowing unlisted V8 or
future function tables to escape the inventory comparison.
The version export assertion also retained `runtime_api`; the current generated
constant is owned and exported by `runtime_build_set`.

## 架构修复验收

- Match the frozen V8 table's 28 fields and generator version 8.
- Preserve the current `runtime_build_set` version-constant owner path. Refresh
  the stale Runtime10 ABI documentation before final acceptance.
- Discover numeric ApiV suffixes, including V8 and later versions; reject
  unversioned or metadata suffixes as function-table names.
- Execute the original ABI inventory tests, the discovery regression, and the
  producer's InterfaceSpec/table slot agreement tests on a frozen source input.
- Complete fixing-session managed validation, C0/I0/M0 independent review,
  canonical return and coordinator closeout before closing this lifecycle.

## 禁止临时方案

- Do not change production ABI layout, accept silent tail extension, remove
  frozen field-count checks, or reintroduce retired runtime versions.
- Do not treat another owner's layout assertion failure as passing acceptance.

## 修复结果与回传

State: `source_repair_complete_pending_managed_validation`.
Audited ownership transfer fingerprint:
`a2315cee3c93738e9a558b3fdda2e83ca707a829bacb55f1d7228ed22366b11b`.
Preimage snapshot 3185 retains source hash
`19622f09859ed18024211e9ceb861b6cd5a1744c926f55c8cd07a87c78ebc24f`.
The repair changes only the test inventory and discovery helper. No production
source, failure return, commit or WeCom notification is included yet.

Managed job `2c8027a3fc044b8e850562025ef0ad7f` executed ten ABI tests against source
3187, manifest `fa3c376d6d6138f75dd8641a23877e0b39cbb6a4a3a3662160fed9e7dbaf0406`:
9 passed and 1 failed. Field-count and discovery regressions passed; the remaining
version-strategy assertion still named the retired `runtime_api` export owner.
The next source snapshot corrects that exact path to `runtime_build_set`.

Formal request `failure-roll-20260908-runtime10-v8-abi-3186-r1` was rejected at
admission with `validation_ticket_external_worktree_dirty` for `E:/Git/zr_vm`.
No formal ticket was issued. Receipt:
`.codex/tmp/runtime10-v8-abi-3186-managed-submission-20260908.json`.
The user excluded `zr_vm`; its worktree is untouched and this prerequisite is
suspended while independent failures continue.

Source snapshot 3188 freezes the final test source at
`a549a918d0c8814d90c6b71d9c6dbecca85b0eed3ad5ab1fe4f7a019cb665111`.
Managed Windows job `711ebb7e29fe4fcb8075f3cb80a1f69d` used immutable input
`runtime10-v8-abi-owner-3188-20260908`, manifest
`ffd0d523c54a6e8732e2f1613bfedee4276a07b535736a198362a4aa264d412a`.
All ten ABI tests executed and passed, with zero ignored and zero failures.
Logs and source-bound receipt:
`results/runtime10-v8-abi-owner-3188.{json,log}` under that input directory.
This is operational managed evidence; formal admission, producer slot agreement,
stale ABI documentation, independent review and closeout remain open.

Producer gate job `7df84915b0db402cbef585c6206c48b0` used the same final immutable
input and passed `runtime_api::abi::api_table::tests::interface_spec_slot_partitions_match_the_abi_table_fields`
(1 passed, 0 failed, 0 ignored). Receipt and log:
`results/runtime10-v8-producer-slots-3188.{json,log}`. The lower producer and ten
original ABI regressions are now dynamically green. Formal admission, stale ABI
documentation, independent review and closeout remain pending.

The Runtime10 plan and cdylib-loader document were adopted through audited transfer
`2e80afacd91c8d2e60872c55898c229c67117a2bc0143cdf0d17dcf32532b153`;
snapshot 3194 retains their exact preimages. Their current sections now document
V8-only loading, 28 fields, 26 function pointers, 20 required and six optional
slots, viewport-pick operations, and the `runtime_build_set` version owner.
Historical V3/V7 evidence remains explicitly historical. No plan completion or
new full dynamic-library validation is asserted.

The broader `runtime-interface-convergence.md` still has an active owner,
`astra-full-domain-20260905`, and a V7 inventory in its Runtime10 section. That
cross-owner documentation update is retained as an acceptance prerequisite;
this Session did not overwrite or transfer the active owner's file. Current
Runtime10-owned documentation hashes are captured in the next snapshot. The
source-bound ABI and producer passes remain unchanged.

Direct App loader validation without features stopped before tests because
runtime_session.rs imports the diagnostic-log module. The existing independent
`diagnostic-log` feature enabled the actual upward test binary without zr_vm.
Job `3204ef6e6c1b46ffaee78707093ed4dd` used the same 3188 input and ran 26 passed,
2 failed, 1 ignored; the ignored project-capture test requires excluded zr_vm.
The source-bound output is `results/runtime10-v8-app-loader-diagnostics-3188.{json,log}`.
Both failures are retained under active App08 ownership:
[frame-demand assertion owner](../../../optimize/zircon_app/08/failure-2026-09-08-runtime-library-frame-demand-test-owner-drift.md)
and [junction fixture creation](../../../optimize/zircon_app/08/failure-2026-09-08-runtime-library-junction-fixture-creation.md).
No upward pass or source takeover is claimed. The related Interface03-owned
[V8 layout regression](../../../optimize/zircon_runtime_interface/03/fixed-2026-09-09-runtime-v8-layout-contract-test-drift.md)
also retains its own fixing identity and acceptance.

## 2026-09-19 rolling current-source contract snapshot

Successor Session `failure-roll-01a084c8-runtime10-v8-abi-r1` received the
audited ownership transfer for the complete Runtime10 inventory slice at
baseline epoch `611`. The current source-contract check is green for the
existing repair:

- `ZrRuntimeApiV8` is pinned to 28 fields in
  `src/tests/abi_safety_contracts.rs`;
- the version strategy assertion names the generated
  `runtime_build_set::ZIRCON_RUNTIME_API_VERSION_V8` owner and generator
  expectation `8`;
- API-table discovery extracts only numeric `ApiV<digits>` suffixes, covering
  current and future versions while rejecting metadata/unversioned names;
- the V8 API table, frozen InterfaceSpec JSON, and the producer slot-partition
  test remain present; Rust 2021 rustfmt checks pass for the focused test and
  table source.

Current immutable manifest hashes are:

```text
docs/plans/zircon_runtime/runtime/10/failure-2026-09-08-runtime-v8-abi-inventory-test-drift.md
  7f9b0d5930f622a4570fc9d5204cd4a456ba474bcd355c70f9a4dd3af842c535
zircon_runtime_interface/src/tests/abi_safety_contracts.rs
  a549a918d0c8814d90c6b71d9c6dbecca85b0eed3ad5ab1fe4f7a019cb665111
zircon_runtime_interface/src/runtime_api/abi/api_table.rs
  50412131e577acc590599dcbef9db86e19ca00621d160e2d6fc999fdbd2b11da
zircon_runtime_interface/build.rs
  1231e575ae17c8f6443e35af3a9548e650623b458f5058df258334c4fc911993
zircon_runtime_interface/src/runtime_build_set/interface_spec_v1.json
  db3c639db8e6a8385dad4959d178b0c95894b51ae87531f46d87ced6955e1fa4
```

The historical source-bound 10-test and producer slot receipts remain useful
lower-layer evidence, but formal current-source managed Cargo admission is
still blocked before execution by the excluded dirty `E:/Git/zr_vm` worktree.
The cross-owner convergence document and App08 upward failures remain outside
this Session. This lifecycle therefore stays `open`; no fixed return or
closeout is claimed.

### 2026-09-19 rolling static contract receipt

Successor ticket `5d55d81fdfe44987a1c7ee228df10375` (request
`failure-roll-01a084c8-runtime10-v8-abi-20260919-r1`) completed through the
managed coordinator. Job/run `1a904c4083374fc1b6182f0f06d0d3d3` returned exit
code 0 and emitted `RUNTIME10_V8_ABI_SOURCE_CONTRACT_PASS`; cleanup completed
without a receipt error. The sealed source-manifest hash was
`0c1813a4b51d067115f18c7d1fc66582590051ac25d273e7d2c98825e8aab091`.

This is a static current-source contract receipt (inventory count, generated
version owner, numeric suffix discovery, frozen table/spec presence and
rustfmt). It does not replace the deferred managed ABI/producer Cargo gates,
cross-owner convergence documentation, or App08 upward loader acceptance.
The lifecycle remains open and no fixed return or closeout is claimed.

### 2026-09-20 independent review

- Reviewer Session `review-runtime10-v8-abi-r1` inspected the source-sealed plan,
  failure record, ABI inventory test, API table, interface generator, and frozen InterfaceSpec.
  Current hashes were recorded as: `abi_safety_contracts.rs`
  `a549a918d0c8814d90c6b71d9c6dbecca85b0eed3ad5ab1fe4f7a019cb665111`, `api_table.rs`
  `50412131e577acc590599dcbef9db86e19ca00621d160e2d6fc999fdbd2b11da`, `build.rs`
  `1231e575ae17c8f6443e35af3a9548e650623b458f5058df258334c4fc911993`, and
  `interface_spec_v1.json` `db3c639db8e6a8385dad4959d178b0c95894b51ae87531f46d87ced6955e1fa4`.
  The reviewer held the failure-document lease while checking the exact Runtime10 scope.
- Review result: `Critical=0`, `Important=0`, `Moderate=0`. The inventory pins the V8 table to
  28 fields, keeps the V8 version owner in `runtime_build_set`, and compares required/optional
  slot partitions against the concrete table. Numeric-only `Api<digits>` discovery includes V8
  and future numeric versions while rejecting unversioned and metadata suffixes. The synthetic
  discovery regression, conservative version-rule assertions, and producer slot-partition guard
  are mutually consistent with the frozen InterfaceSpec.
- Scoped `rustfmt --check --edition 2021 --config skip_children=true` for the owned Rust files and
  `git diff --check` passed. No source or plan implementation bytes were edited by this review.
  Historical 10-test/producer receipts are retained only as supporting evidence; the excluded
  dirty `E:/Git/zr_vm` admission blocker, cross-owner V7 convergence-document drift, App08 loader
  failures, fresh managed Cargo acceptance, canonical fixed return, and closeout remain pending.
