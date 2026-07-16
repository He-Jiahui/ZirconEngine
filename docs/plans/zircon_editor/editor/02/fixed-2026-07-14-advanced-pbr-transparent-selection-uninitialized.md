---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
summary_slug: advanced-pbr-transparent-selection-uninitialized
origin_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
fixing_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
origin_child_dir: docs/plans/zircon_editor/editor/02
fixing_child_dir: docs/plans/zircon_runtime/render/18
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
tests:
  - cargo check -p zircon_runtime --lib --locked
  - cargo test -p zircon_editor --lib --locked
resolved_at: 2026-07-14
---


# Render 18：advanced PBR transparent command selection 可能未初始化

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 来源执行切片：Editor02 M1 最终 editor consumer 门禁
- 修复责任计划：`docs/plans/zircon_runtime/render/18-advanced-lighting-features.md`
- 交接原因：错误由 Render18 正在接入的 late-forward opaque/transmission command selection 产生；Editor02 不拥有 renderer graph execution，也不得用默认空列表绕过渲染阶段语义。

## 失败现象与复现证据

受管 Windows job `62cbbd3e48ff4406a6b6b05364f7797b` 执行：

```powershell
cargo test -p zircon_editor --lib --locked
```

编译 `zircon_runtime` 时以 exit `101` 停止：

```text
error[E0381]: used binding `selected_transparent_commands` is possibly-uninitialized
zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/
render_pass_execution_context/gpu.rs:694:17
```

`selected_transparent_commands` 只在 `RenderPassStage::Transparent3d` 分支初始化，但后续
`mixes_transparent_sprites` 路径无条件借用它；Rust 无法从当前控制流证明该借用只发生在初始化分支。
完整日志：`E:\ZirconBuilds\editor02-m1-editor-consumer-final-20260714.log`。

## 最低共享层根因

最低已证明边界是 Render18 的 advanced PBR pass selection 与既有 transparent sprite 混合路径没有形成
同一个类型安全的 command-stream owner。新逻辑把临时 `Vec<MeshDrawCommand>` 的生命周期绑定在一个条件分支，
而旧 sprite replay 接口仍直接读取该临时列表，导致非透明 stage 的控制流存在未初始化借用。

## 架构修复验收

- command selection 的所有分支都返回已初始化且生命周期明确的 stream/selection owner。
- `Main` 与 `LateForwardOpaque` 的透明命令过滤语义保持互斥，transmission 与 sprite 混合顺序不被默认空列表掩盖。
- 不复制 draw command、不恢复旧 pass 路径、不为通过编译而跳过 transparent sprite replay。
- `cargo check -p zircon_runtime --lib --locked` 与原 `cargo test -p zircon_editor --lib --locked` 均不再出现 E0381。

## 禁止临时方案

- 不在 Editor02 增加 cfg、feature 或 consumer 侧绕过。
- 不使用未初始化内存、`unsafe`、全局 mutable scratch 或虚假默认命令来压制借用检查。
- 不修改 Shader04 文件，也不恢复任何旧架构兼容层。

## 修复结果与回传

- 根因：late-forward opaque reused the transparent stage through a branch-local filtered Vec, while sprite replay borrowed that temporary outside the compiler-provable initialization branch
- 架构修复：partitioned advanced forward-only opaque commands into a dedicated MeshPassCommandBuffers and indirect stream; typed MeshStageCommandSource now selects stable borrowed streams, keeps standard transparent sprite replay isolated, and preserves transmission ordering without draw-command copies or default bypasses
- 验证：git diff --check passed; managed Windows zircon_runtime build compiled beyond gpu.rs with no E0381 and then stopped on foreign LevelManager CoreWeak/type-inference errors tracked by Frameworks05; Editor02 managed consumer validation was requeued after Render18 and no longer reports the original uninitialized binding
- 回传：Render18 command selection owner is initialized on every branch. Main late-forward opaque and transmission streams are distinct; transparent sprite replay remains on the standard transparent stream. Returned to Editor02 for consumer-gate continuation.
