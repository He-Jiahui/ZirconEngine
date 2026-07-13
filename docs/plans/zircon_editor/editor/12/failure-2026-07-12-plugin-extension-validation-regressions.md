---
handoff_kind: failure
status: open
created_at: 2026-07-12
summary_slug: plugin-extension-validation-regressions
origin_plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
fixing_plan: docs/plans/zircon_editor/editor/12-plugin-management.md
origin_child_dir: docs/plans/zircon_editor/editor/08
fixing_child_dir: docs/plans/zircon_editor/editor/12
related_code:
  - zircon_editor/src/core/editor_plugin.rs
  - zircon_editor/src/core/editor_event
  - zircon_editor/src/tests/editor_authoring_extension_descriptors.rs
  - zircon_editor/src/tests/editor_event/runtime
tests:
  - cargo test -p zircon_editor --lib --locked editor_authoring_extension_descriptors -- --test-threads=1
  - cargo test -p zircon_editor --lib --locked extensions_ -- --test-threads=1
---

# Editor 12：插件贡献注册与校验当前回归

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md`
- 来源执行切片：Editor08 M1 统一命令注册行为门
- 修复责任计划：`docs/plans/zircon_editor/editor/12-plugin-management.md`
- 交接原因：失败位于插件贡献校验、capability gate 与注册原子性，不能由 Editor08 恢复退役 operation registry 绕过。

## 失败现象与复现证据

受管全量 job `520d85713df249afae31661a7697ad07` 中以下三项失败：

- `authoring_registry_rejects_invalid_operation_payload_schema_ids`
- `editor_runtime_consumes_plugin_registration_reports_with_capability_gate`
- `editor_runtime_rejects_duplicate_extension_view_without_registering_operations`

同一 binary 的 Editor08 command registry hard-cut 与共享 palette/when 用例通过，说明本组信号位于插件贡献 validation、capability gate 和失败原子性，不应恢复旧 operation registry。原始日志为 `D:/cargo-targets/editor08-m1-rerun4-20260712.log`；全包停滞前未输出捕获的 panic 详情。

## 最低共享层根因

最低 owner 是 Editor12 的插件贡献物化与注册报告校验。功能 owner 须 exact 重跑取得实际 schema/capability/duplicate-view 差异，并检查失败时 command/operation/view contribution 是否保持原子不污染。

## 架构修复验收

- 三个 fully-qualified exact 单线程全绿，并覆盖无效 schema、capability 不足和重复 view 三类 typed rejection。
- 注册失败不得留下 command、view 或 operation 的部分贡献；rlib/cdylib 双轨必须遵循同一校验结果。
- 不恢复 `EditorOperationRegistry` 或 UI-host 私有命令注册表；插件 command 继续通过 Editor08 唯一 registry handle 物化。

## 禁止临时方案

- 禁止放宽 schema/capability 校验、删除原子性断言、吞掉 duplicate contribution。
- 禁止为测试重建退役 operation stack/registry、alias 或 compat registration path。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| Editor12 M1/M2 / Editor08 M1 | 插件贡献校验与失败原子性 | `open-待功能owner精确复现` | 2026-07-12 | job `520d85713df249afae31661a7697ad07` 复现 3 个插件注册/校验失败；完整失败名与运行上下文见 `D:/cargo-targets/editor08-m1-rerun4-20260712.log`。 |
| Editor12 M1/M2 / Editor09 M1 | 当前源码完整门停滞前复现 | `open-继续由功能owner处理` | 2026-07-13 | job `e81ed19d256f40c28ddb2437e9a18460` 在停滞前再次逐项记录同三项失败，证明该聚类未被 Editor09 asset registry hard-cut 掩盖；日志 `.codex/tmp/editor09-m1-full-lib-test-r2-20260713.log`。 |

## 修复结果与回传

- 状态：`open / 待修复`；Editor12 修复并复验后回传 Editor08，不在命令层添加兼容绕行。
