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
