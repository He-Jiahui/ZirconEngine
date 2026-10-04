---
handoff_kind: fixed
failure_scope: local
status: fixed
created_at: 2026-09-08
summary_slug: runtime-v8-layout-contract-test-drift
origin_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
fixing_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
origin_child_dir: docs/plans/optimize/zircon_runtime_interface/03
fixing_child_dir: docs/plans/optimize/zircon_runtime_interface/03
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime_interface/src/runtime_api/abi/api_table.rs
tests:
  - cargo test -p zircon_runtime_interface --no-default-features --locked --lib tests::contracts::runtime_api_table_records_size_and_version
  - cargo test -p zircon_runtime_interface --no-default-features --locked --lib tests::contracts::
resolved_at: 2026-09-09
---

# Interface03: V8 layout contract test drift

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md`
- 来源执行切片：Full interface-library acceptance of the existing Interface03 snapshot.
- 修复责任计划：`docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md`
- 交接原因：This local regression retains the existing Interface03 owner of contracts.rs.
- Related ABI inventory owner: [Runtime10](../../../zircon_runtime/runtime/10/failure-2026-09-08-runtime-v8-abi-inventory-test-drift.md).

## 失败现象与复现证据

Windows managed job `dd18d653fe3c494c91fbe57f5959580a` ran the full interface
library: 754 passed, 11 failed, 101 ignored. Input
`interface-project-identity-3170-20260908` has manifest
`1f5a61e72075db714e26e348064c43abf268af11edf80465b15536ef40f639c9`.
Its `results/interface-library-3170.log` records the original
`tests::contracts::runtime_api_table_records_size_and_version` failure:
the actual table size is 224 bytes while the test expects 200.

## 最低共享层根因

The frozen V8 producer adds request, poll and cancel viewport-pick function
pointers after the V7 table. The existing layout regression retained the V7 size
and omitted the three new fields' empty values and offsets. Production uses the
V8 table and InterfaceSpec; no production layout change is needed.

## 架构修复验收

- Pin the existing 64-bit V8 layout to 224 bytes and retain exact version checking.
- Assert all three picking slots are empty and contiguous after the preceding
  world-invalidation slot, preserving the previous slot-order assertions.
- Execute the original test and the surrounding contracts suite on frozen input.
- Keep Runtime10 inventory/producer evidence and ownership separate; complete
  formal validation, C0/I0/M0 review, canonical return and coordinator closeout.

## 禁止临时方案

- Do not relax the size check, reintroduce a V7 table, or edit the producer to fit
  stale assertions. Do not claim the full library passed from a filtered result.

## 修复结果与回传

- 根因：The V8 runtime API table added three viewport-pick function-pointer slots, so the authoritative 64-bit layout is 224 bytes; the old test asserted the stale V7 200-byte size.
- 架构修复：Updated the Interface03 layout contract to assert 224 bytes, exact V8 version, and empty contiguous request/poll/cancel slots after world invalidation without changing the producer.
- 验证：Managed Windows job 07c2ecb073c0493cb16fd4c4a95744aa on input interface03-v8-layout-3199-20260908, manifest 58164b9667051219bec66342bcb79d9325648eb2cfcea1f3ad307ebc743c1abd: 51/51 surrounding contract tests passed including tests::contracts::runtime_api_table_records_size_and_version; receipt and log retained in results/interface03-v8-contracts-3199.{json,log}.
- 回传：Canonical child-only fixed artifact and return receipt generated under Interface03 child directory; original failure evidence is preserved and this lifecycle is ready for coordinator closeout review.
