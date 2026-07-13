---
handoff_kind: fixed
status: fixed
created_at: 2026-07-12
resolved_at: 2026-07-12
summary_slug: realtime-ibl-graph-recorder-type-errors
origin_plan: docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
fixing_plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
origin_child_dir: docs/plans/zircon_editor/editor_layout/15
fixing_child_dir: docs/plans/zircon_runtime/shader/06
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_graph_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_wgpu_recorder.rs
tests:
  - cargo test -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:/cargo-targets/zircon-editor-blend-space-20260712 blend_space_workspace --no-run
---


# Shader 06：Realtime IBL graph/recorder 当前源码类型错误

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md`
- 来源执行切片：Blend Space 当前源码测试二进制构建、几何与截图验收
- 修复责任计划：`docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md`
- 交接原因：Physics 03 已补齐先前四处 `ColliderShape` 非穷尽消费端；同一 Editor 构建随后进入 Shader 06 活跃 owner 的未跟踪 Realtime IBL graph/recorder 文件，并被两个 Rust 类型错误阻断。这两个文件和语义属于 Shader 06 当前租约，Editor Layout 不越界修改。

## 失败现象与复现证据

2026-07-12 在受管 D 盘紧凑 target 增量构建 `zircon_editor`：

- `realtime_ibl_graph_plan.rs:312-316` 的 `select_slot` 接收两个借用并返回其中之一，返回引用缺少显式生命周期，报 `E0106`。
- `realtime_ibl_wgpu_recorder.rs:80` 在返回 `Result<RealtimeIblWgpuRecordReport, String>` 的方法中，对 `Option` 返回值直接使用 `?`，报 `E0277`；应由 Shader owner 定义缺失 mip resource 的 typed/error 映射语义。

复现命令：

```text
cargo test -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:/cargo-targets/zircon-editor-blend-space-20260712 blend_space_workspace --no-run
```

## 最低共享层根因

Shader 06 正在接入 Realtime IBL graph 与 WGPU recorder，但两个新 owner 文件尚未完成 Rust 借用返回关系和 `Option`→`Result` 错误边界。该失败发生在 `zircon_runtime` library 编译，所有上层 Editor test binary 均无法生成。

## 架构修复验收

- `select_slot` 的输入/输出生命周期关系显式且最小，不复制 slot resources 规避借用合同。
- recorder 对缺失 source/storage mip 使用 Shader 06 定义的真实错误语义转换，不使用 `unwrap/expect`、空字符串或静默跳过 dispatch。
- Shader 06 focused Realtime IBL graph/recorder 测试通过；原 Editor Blend Space `--no-run` 构建越过这两个错误后回传。

## 禁止临时方案

- 禁止在 Editor 侧 feature-gate、跳过或复制 Realtime IBL 模块。
- 禁止用 `unwrap/expect`、泄漏引用、全局静态或伪默认 resource 消除编译错误。
- 禁止弱化 Blend Space 当前源码几何和截图门禁。

## 修复结果与回传

- 根因：select_slot had two elided input lifetimes but one borrowed return; recorder propagated an Option PMREM slice command through a Result boundary without conversion
- 架构修复：Bound both slot references and the return to one explicit lifetime; added a recorder prefilter_command error boundary that reports mip and face range without fallback or unwrap
- 验证：zircon_runtime core-min cargo check passed; original zircon_editor no-run moved past both Shader errors and next stopped on unrelated missing graphics/text/rich/bbcode_blocks module
- 回传：Shader 06 realtime IBL graph and recorder type blockers fixed and returned to Editor Layout 15; upper build now reaches the next external owner
