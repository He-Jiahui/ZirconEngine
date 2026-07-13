---
handoff_kind: fixed
status: fixed
created_at: 2026-07-12
resolved_at: 2026-07-12
summary_slug: realtime-ibl-option-then-type-errors
origin_plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
fixing_plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
origin_child_dir: docs/plans/zircon_editor/editor/08
fixing_child_dir: docs/plans/zircon_runtime/shader/06
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
tests:
  - cargo check -p zircon_runtime --lib --no-default-features --features target-client --locked --offline --jobs 1
  - cargo test -p zircon_editor --lib --locked --jobs 1 --target-dir D:/cargo-targets/editor08-m1-20260712
---


# Shader 06：Realtime IBL prepared Option 链当前源码类型错误

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md`
- 来源执行切片：Editor08 M1 合一注册表与 when 统一测试阶段
- 修复责任计划：`docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md`
- 交接原因：Editor08 的完整 `zircon_editor --lib` 测试在进入 Editor 行为测试前，于 Realtime IBL prepared 集成点遇到两个同构 E0599。两个文件及帧准备语义属于 Shader06，Editor08 不越界修改；Text07 的更早 target-client 检查作为同一最低共享故障的补充复现保留在下文。

## 失败现象与复现证据

2026-07-12 使用协调器受管 target `D:/cargo-targets/zircon-editor-blend-space-20260712` 执行：

```text
cargo check -p zircon_runtime --lib --no-default-features --features target-client --locked --offline --jobs 1
```

出现：

- `scene_renderer_core_render_compiled_scene/render/render.rs:68`：`then_some(...).filter(...)` 已返回 `Option<T>`，继续调用只属于 `bool` 的 `.then(...)`，报 E0599。
- `scene_renderer_core_render_scene/render_scene.rs:25`：相同 Realtime IBL prepared 链以同样方式报 E0599。
- 同一次检查还复现既有 open handoff `failure-2026-07-12-realtime-ibl-graph-recorder-type-errors.md` 的 E0106/E0277；未出现 Text07 诊断。

协调器 job `d3733060775749f39bcc8f61746b3b2b` 已按失败退出码结束并释放。

Editor08 M1 统一测试阶段于同日用 Windows 受管 job `8e25dcad70cf433f847359490af0a9ee` 再次执行 `cargo test -p zircon_editor --lib --locked --jobs 1 --target-dir D:/cargo-targets/editor08-m1-20260712`，18m06s 后以 exit 101 结束并释放；同一次构建稳定复现两个 `E0599`，并同时复现 graph/recorder 交接中的 `E0106/E0277`，编译未进入 Editor08 行为测试。

## 最低共享层根因

Realtime IBL prepared 值已经进入 `Option` 组合阶段，但两个 scene renderer 入口仍使用 `bool::then` 风格闭包。修复必须由 Shader06 owner 统一两个入口的准备条件与副作用时机，不能由 Text07 在调用侧屏蔽编译。

## 架构修复验收

- 两个入口使用同一明确的 `Option` 变换语义，且仅在 procedural sky 有效并且 intensity 大于零时调用 `prepare_frame`。
- 不使用 `unwrap/expect`、伪默认 sky 或重复执行 `prepare_frame` 绕过类型错误。
- 上述 target-client production check 越过 E0599，并与既有 graph/recorder handoff 一起完成 Shader06 focused 验证。

## 禁止临时方案

- 禁止在 Text 或 Editor 入口 feature-gate、跳过或复制 Realtime IBL 准备路径。
- 禁止使用 `unwrap/expect`、伪默认 sky、重复执行 `prepare_frame` 或调用点特例掩盖类型错误。
- 禁止弱化来源计划及 Editor08 的原始 Cargo 验收命令。

## 修复结果与回传

- 根因：Both renderer entries called bool then after then_some and filter had already produced an Option of ProceduralSkyParams
- 架构修复：Unified both entries on Option map and passed the already filtered sky into prepare_frame exactly once
- 验证：zircon_runtime core-min cargo check passed; original editor no-run moved past both E0599 sites and next stopped on unrelated missing graphics/text/rich/bbcode_blocks module
- 回传：Shader 06 realtime IBL Option preparation type errors fixed and returned to Editor08; upper build now reaches the next Text owner
