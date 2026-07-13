---
handoff_kind: fixed
status: fixed
created_at: 2026-07-12
summary_slug: component-registry-typed-contract-test
origin_plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
fixing_plan: docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
origin_child_dir: docs/plans/zircon_editor/editor/14
fixing_child_dir: docs/plans/zircon_editor/editor_layout/15
related_code:
  - zircon_editor/src/ui/component_registry.rs
  - zircon_editor/src/ui/component_registry.rs
tests:
  - cargo test -p zircon_editor --lib --locked --jobs 1 core::jobs::tests -- --test-threads=1 --nocapture
resolved_at: 2026-07-12
---


# Editor Layout 15：组件注册表测试仍按字符串比较 typed 契约

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md`
- 来源执行切片：Editor14 M2 线程所有权与终态资源合同最终重编译
- 修复责任计划：`docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md`
- 交接原因：失败位于组件标准化注册表的 typed category/layout-role 契约测试，归属组件标准化计划，不属于调度系统。

## 失败现象与复现证据

受管 Windows job `8a813aefe7294e93981b3925466f08ed` 在测试体启动前于 `zircon_editor/src/ui/component_registry.rs:32-33` 产生 E0308×2：测试把 `UiComponentCategory` 与字符串 `"input"` 比较，并把 `UiComponentLayoutRole` 与字符串 `"leaf"` 比较。原命令为 `cargo test -p zircon_editor --lib --locked --jobs 1 core::jobs::tests -- --test-threads=1 --nocapture`，stderr 为 `D:/cargo-targets/editor14-final-focused-20260712.err.log`。

## 最低共享层根因

组件注册表生产契约已硬切为 typed `UiComponentCategory/UiComponentLayoutRole`，但同文件测试仍保留旧字符串断言，形成测试与新版唯一类型真源不一致。最低修复应更新测试消费 typed 值，并删除任何旧字符串兼容比较预期。

## 架构修复验收

- `component_registry.rs` 测试只与 typed enum/新类型值比较，不增加 `PartialEq<str>`、字符串转换 shim 或旧字段。
- component registry focused 测试通过并证明 category/layout role 的 typed 语义。
- 原 Editor14 `core::jobs::tests` 重编译并自然结束。
- `cargo test -p zircon_editor --lib --locked --jobs 1` 能继续进入测试体。

## 禁止临时方案

- 禁止为 typed 值实现对 `str` 的兼容比较、恢复字符串字段或在测试中转回旧架构真源。
- 禁止 ignore/注释断言或弱化全量门禁。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| Editor Layout 15 / Editor14 M2 | component registry typed 合同测试 | `open-待对应功能修复` | 2026-07-12 | job `8a813aefe7294e93981b3925466f08ed`；`component_registry.rs:32-33` E0308×2；日志 `D:/cargo-targets/editor14-final-focused-20260712.err.log`。 |

## 修复结果与回传

- 根因：The registry production contract had converged to typed category/layout-role values while its test still compared legacy string literals.
- 架构修复：The shared retained component registry remains the single owner and the focused test consumes typed values through their explicit as_str contract; no PartialEq string shim or legacy field was introduced.
- 验证：Current zircon_editor test binary: component_registry exact 1/1 passed. Editor14 managed rerun log editor14-m3-focused-rerun.log: core::jobs::tests 36/36 passed, exit 0; E0308 did not recur.
- 回传：Typed component-registry test contract is fixed and the originating Editor14 focused suite now completes successfully.
