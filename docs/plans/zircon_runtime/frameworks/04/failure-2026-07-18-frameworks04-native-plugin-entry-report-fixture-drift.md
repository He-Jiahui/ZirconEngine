---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: frameworks04-native-plugin-entry-report-fixture-drift
origin_plan: docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
fixing_plan: docs/plans/zircon_runtime/frameworks/04-plugin-dx-and-sdk-toolchain.md
origin_child_dir: docs/plans/zircon_runtime/runtime/12
fixing_child_dir: docs/plans/zircon_runtime/frameworks/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_abi.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests.rs
tests:
  - managed Windows job b683460bdf5045908517e328b85f962b / run c96ead9118594183a538a77ea88626ec
  - runtime_12_input_stack_mirror_docs_match_structure_audit_counts retry pending
---

# Frameworks04: native plugin entry-report fixture drift

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md`
- 来源执行切片：Runtime12 current-source mirror gate
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/04-plugin-dx-and-sdk-toolchain.md`
- 交接原因：失败位于 Frameworks04 所有的 native entry-report 测试 consumer；Runtime12 只消费编译结果，不拥有该 fixture shape。

## 失败现象与复现证据

Runtime12 current-source mirror job `b683460bdf5045908517e328b85f962b` / run
`c96ead9118594183a538a77ea88626ec` 以 exit 101 结束，目标测试未执行。lib-test 编译在
`native_plugin_live_host/tests.rs` 报告两个 E0063：本地 `NativePluginEntryReport` fixture
未初始化 Frameworks04 新增的 `missing_required_capabilities` 与 `denied_capabilities`。
同一快照中的 Text01 错误已由其 owner 判定为旧源污染，不属于本 lifecycle。

2026-07-30 fresh retry 使用 snapshot `1319`、job
`e83f2aa0784d45cab6526effd572d7a2` / run
`45760a71db784350a710e5b33d138fb4`。运行自然释放为 exit 101，目标 Runtime12 测试仍未执行；
编译先到达 Plugins01-owned availability projection 错误。该结果既不是 Runtime12 red，也不是
Frameworks04 fixture acceptance。

## 最低共享层根因

Frameworks04 扩展了 `NativePluginEntryReport` 的 capability outcome 合同，但两个同 owner
success fixture 仍构造旧字段集合。修复已在 current source 初始化为空列表，符合 fixture
descriptor 不请求 capability 的事实；剩余阻塞来自其他 owner，尚缺 source-valid managed compile。

## 架构修复验收

- 两个 success fixture 显式初始化 capability outcome 字段，不增加默认构造器、serde fallback 或兼容 shim。
- fresh source-valid Runtime lib-test compile 越过 fixture E0063，并实际执行 Runtime12 mirror target。
- Plugins01 availability projection failure 返回后重建 immutable source manifest；不得复用已污染的 b683/e83f 结果。

## 禁止临时方案

- 不得修改生产 capability negotiation、ABI layout 或 Runtime12 input contract 来绕过 test consumer。
- 不得把其他 owner 的 compile error 当作本 fixture 已通过，也不得提前生成 fixed return。
- 不得增加 optional field、默认兼容构造器或 test-only bypass。

## 修复结果与回传

Open state: `consumer_fix_applied_pending_fresh_source_valid_managed_compile`; no Runtime12 mirror pass or fixed return is claimed.

Plugins01 的并行 owner 记录见
[`runtime-profile-availability-rebuild`](../../../zircon_plugins/01/failure-2026-07-17-runtime-profile-availability-rebuild.md)。

## 2026-09-11 failure rolling repair

- Stable primary `failure-roll-01a084c8-frameworks04-entry-fixture-r1` owns the exact failure record and native live-host fixture consumer. Current source review confirms both success fixtures explicitly initialize `missing_required_capabilities` and `denied_capabilities` with empty lists; edition-2021 rustfmt and scoped diff checks pass.
- Snapshot 3421 freezes the failure record and `native_plugin_live_host/tests.rs`. Request `frameworks04-entry-fixture-runtime12-mirror-20260911-r1` submitted `cargo +1.94.1 test -p zircon_runtime --locked --lib runtime_12_input_stack_mirror_docs_match_structure_audit_counts -- --exact --test-threads=1 --nocapture`.
- Coordinator admission returned `validation_ticket_external_worktree_dirty` for `E:\\Git\\zr_vm`; no ticket or Cargo execution occurred. The lifecycle remains open and the session is `waiting_validation`; after a clean external revision, rerun the exact mirror target and verify it actually executes before any fixed return or closeout.

### 2026-09-19 rolling source-contract snapshot (Frameworks04 owner retry)

- Retry Session `failure-roll-01a084c8-frameworks04-entry-fixture-r2` transferred only this failure record and `zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests.rs`; production ABI declaration and Runtime12 mirror owner paths remain outside this scope.
- Both success fixtures explicitly initialize `missing_required_capabilities` and `denied_capabilities` as empty lists. Edition-2021 rustfmt check passed for the owned fixture file, and no default constructor, serde fallback, or capability-negotiation bypass was added. The owned source hash is `b37beb846c2744c7851ea69b29cf5facb5cae2a2c42d0a5872b0f52165cfe76d`.
- The 2026-09-11 managed mirror admission was rejected before ticket creation by `validation_ticket_external_worktree_dirty`; the prior b683/e83 compile-blocked results are not reused. A fresh managed Runtime12 exact test remains required once the external revision is clean.
- State remains `consumer_fix_applied / static_source_contract / managed_validation_pending`; no fixed/return or closeout is justified until the target test actually executes, unrelated owner errors are cleared, and independent Critical/Important/Moderate review is green.

### 2026-09-19 failed static ticket evidence (preserved)

- Retry ticket `73875bd9c7fe49d8b52b8316ae2d7ddc` (request `failure-roll-01a084c8-frameworks04-entry-fixture-20260919-r2`) was admitted and materialized, but ended `failed` with coordinator failure category and exit code 1.
- The command's source-contract assertions were not reached: the sealed manifest contained only this record and `native_plugin_live_host/tests.rs`, while `rustfmt --emit stdout` recursively required the unsealed module `zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/bridge_bindings.rs`. The managed copy therefore reported `os error 3` and `rustfmt parse failed`.
- This is validation-harness/source-manifest incompleteness, not evidence against the fixture repair. The failed result is excluded from reuse; a corrected ticket must seal the direct module dependencies (without absorbing another owner's edits) or use an equivalent scoped parser that can resolve them.

### 2026-09-19 corrected static validation

- Corrected ticket `466ab02b01a645b5b8740fae5ad71c5a` (request `failure-roll-01a084c8-frameworks04-entry-fixture-20260919-r3`) sealed the same two owned inputs and ran the root-only contract with `rustfmt +1.94.1 --edition 2021 --config skip_children=true --emit stdout`; this intentionally avoids traversing the separately attributed `bridge_bindings.rs` child.
- Coordinator job `8e9091a15a15486c8b174e97b2d81328` / run `466ab02b01a645b5b8740fae5ad71c5a` finished exit 0 with stdout `FRAMEWORKS04_NATIVE_ENTRY_FIXTURE_SOURCE_CONTRACT_PARSE_PASS`; cleanup completed. This is static source-contract evidence only; Runtime12 managed Cargo, product parity, review, and fixed return remain pending.

### 2026-09-20 independent source review r2

The independent reviewer inspected the two success fixtures in
`native_plugin_live_host/tests.rs` and the production report consumer.  Both
`NativePluginEntryReport` literals explicitly initialize
`missing_required_capabilities` and `denied_capabilities` to empty vectors,
matching a success fixture that negotiates no capability failure.  The review
also confirmed that `NativePluginEntryReportV3` decoding and
`capability_negotiation_details` remain production-owned and unchanged; no
default constructor, serde fallback, ABI-layout change, or capability bypass
was introduced.

Read-only checks:

- `rustfmt +1.94.1 --edition 2021 --config skip_children=true --check`
  on the owned fixture file: `FRAMEWORKS04_RUSTFMT_PASS`.
- `git diff --check` on that file: pass.

Current owned source hash inspected:

```text
zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests.rs b37beb846c2744c7851ea69b29cf5facb5cae2a2c42d0a5872b0f52165cfe76
```

Independent review result: `Critical=0 Important=0 Moderate=0`.  This receipt
does not claim Runtime12 managed Cargo execution, Plugins01 availability
repair, product parity, canonical fixed return, or closeout.
