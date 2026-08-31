---
handoff_kind: fixed
status: fixed
created_at: 2026-08-25
summary_slug: interface-spec-slot-list-lifetime
origin_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
fixing_plan: docs/plans/optimize/zircon_runtime_interface/08-runtime-dll-abi-ffi-version-handle-foreign-ownership-current-source-review.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/01
fixing_child_dir: docs/plans/optimize/zircon_runtime_interface/08
related_code:
  - zircon_runtime_interface/build.rs
tests:
  - coordinator managed Cargo job 673d51016b2d4679842de468abca4ec0
  - coordinator managed Cargo job 7f8adbb03fdf4616a9d6d887045c74a2
  - coordinator managed Cargo job 29a2e88837ab4890863b3b59ce7fd251
  - cargo test -p zircon_runtime_interface --locked
  - Frameworks01 focused zircon_runtime state lib-test gate
plan_link_mode: child_record_only
resolved_at: 2026-08-25
---

# Interface08: InterfaceSpec slot-list lifetime compile failure

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 来源执行切片：M1 state-transition retention focused managed Cargo validation
- 修复责任计划：`docs/plans/optimize/zircon_runtime_interface/08-runtime-dll-abi-ffi-version-handle-foreign-ownership-current-source-review.md`
- 交接原因：Runtime 状态测试尚未开始前，最低共享失败已定位到 Runtime Interface 的 InterfaceSpec build script。

## 失败现象与复现证据

Windows managed Cargo job `673d51016b2d4679842de468abca4ec0` 在通过 locked dependency resolution 后，
编译 `zircon_runtime_interface/build.rs` 时于原第 132 行返回 E0106。`slot_list` 同时接收 `spec` 和
`field` 两个借用，却返回 `Vec<&str>`，因此编译器无法判断返回值借用哪一个输入。该 job 在生成 Runtime
测试目标前退出 1；Frameworks01 state test 数为 0，不能记为行为失败或 GREEN。

## 最低共享层根因

返回的 slot 字符串只来自 `serde_json::Value` 中的数组元素，但函数签名没有把输出生命周期显式绑定到
`spec`。`field` 只参与查找与诊断格式化，不拥有任何返回数据，也不应被绑定到输出生命周期。

## 架构修复验收

- `slot_list<'a>(spec: &'a Value, field: &str) -> Result<Vec<&'a str>, String>` 编译通过，且不扩大 owned clone。
- Runtime Interface build-script 现有 InterfaceSpec generator 单元测试通过。
- 原 Frameworks01 focused Runtime state gate越过该 build-script 前沿并取得新终态，或记录下一条真实 foreign blocker。

## 禁止临时方案

- 不把 slot 转成 owned `String` 来掩盖借用声明错误。
- 不把 `field` 错误绑定到 `'a`，不扩大借用范围。
- 不添加 alias、compatibility shim、silent fallback、test-only bypass 或调用点特例。
- 不弱化 locked Cargo、InterfaceSpec schema 或 Frameworks01 state acceptance gate。

## 修复结果与回传

- 根因：slot_list returned string slices from InterfaceSpec spec without binding the output lifetime to spec; field only selected and diagnosed a key
- 架构修复：Bound slot_list output to spec through an explicit lifetime while keeping field independent and preserving borrowed zero-clone output; no compatibility wrapper or call-site exception
- 验证：Direct real build.rs rustc tests passed 5/5 in 0.10s on D drive; managed job 7f8adbb03fdf4616a9d6d887045c74a2 cargo build -p zircon_runtime_interface --locked passed in 3m12s, then its filtered lib-test stopped before execution on 9 foreign UI/Project test-source errors; rerun job 29a2e88837ab4890863b3b59ce7fd251 crossed the original E0106 and stopped at transient unowned ProjectGuid import drift; both jobs released with empty process trees
- 回传：InterfaceSpec build-script lifetime failure is fixed and its original compiler frontier is cleared; Frameworks01 resumes after Project06 current-source ownership converges
