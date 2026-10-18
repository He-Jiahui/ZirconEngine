# Zircon Runtime 架构渐进式 Review 与优化计划

## Summary

- 采用分层硬切换，不做一次性全仓重写，也不保留旧路径 shim。当前固定目标是 `zircon_app` host、`zircon_runtime::core::{runtime,framework,manager,math,resource}` spine、`zircon_editor` authoring host。
- 当前审计结论：`zircon_app` 基本 converged；`zircon_runtime` 和 `zircon_editor` 因大文件、公开面过宽、runtime/editor 残留边界标为 `needs-refactor`。
- 参考策略：Bevy 约束 App/plugin/schedule/ECS，Fyrox 约束 Rust 引擎和 editor/runtime 分层，Unreal 约束大型 Runtime/Editor/Programs 和模块尺度。

## Key Decisions

- `CoreRuntime` 不重写为新内核；先保留现有 descriptor/context/handle/state 结构，优化注册、解析、调度和插件接入的边界与热路径。
- `zircon_app` 只选择 profile 和启动 host，不再直接拥有可选 runtime plugin 依赖扇出；插件组合、manifest、module 装配归 `zircon_runtime::plugin` 和 runtime-owned catalog。
- `zircon_runtime` crate root 收窄为模块入口和少量稳定 API；`zircon_runtime/src/lib.rs` 的 graphics 大量 re-export、`zircon_runtime/src/builtin/runtime_modules.rs` 的混合职责要拆成 folder-backed 装配层。
- 代码生成产物只能是 leaf binding/DTO/table，不允许持有业务规则、调度、状态突变或公共架构决策；重复的 generated behavior 迁回手写 owner。
- `zircon_runtime::scene` 只保留 runtime world、ECS、reflection、serialization、render extract；`scene/editor_projection` 改为中性 inspection/reflection 输出，editor 负责 hierarchy/inspector/viewport DTO。
- 插件主路径收束到 VM plugin lifecycle、stable host handles、slot hot reload；native plugin loader 从 runtime public surface 移除或隔离为工具/测试路径。
- 非网络 `server` 命名、legacy/compat/shim/bridge 迁移路径、旧 crate 文档引用都作为 review blocker。

## Milestones

1. **M0 Baseline And Coordination**
   - 保护当前 dirty worktree；读取活动 session note，冻结冲突区域的 owner。
   - 跑结构审计、旧命名扫描、大文件扫描、参考引擎 evidence matrix。
   - 产物：当前架构差距清单、冲突清单、review 顺序，不改代码。

2. **M1 Public Surface And Generated Boundary**
   - 收窄 `zircon_runtime`、`zircon_editor`、`zircon_app` root exports。
   - 建立 generated-code 规则：生成文件只导出数据或 adapter，业务逻辑迁入 owner module。
   - 删除旧路径文档和 re-export 依赖，不加兼容层。

3. **M2 Runtime Module Assembly**
   - 拆分 runtime module/profile/plugin assembly：core modules、profile manifest、linked plugin registration、feature reports 分离。
   - `zircon_app` 只调用 runtime-owned assembly API；可选插件依赖从 app 迁出或通过 runtime/plugin catalog 管理。
   - 加结构测试防止 app 重新依赖扩展 plugin crate。

4. **M3 Scene And Editor Boundary**
   - runtime 提供 neutral `WorldInspection`/reflection snapshot；editor 生成 `SceneEditModeProjection`、selection、overlay、gizmo、viewport state。
   - 移除 runtime 里的 editor 命名和 editor DTO re-export。
   - 验证 world serialization 不含 selection/viewport authoring state。

5. **M4 Plugin Lifecycle Convergence**
   - Core-level `PluginDescriptor`、`resolve_plugin`、VM slot lifecycle、hot reload state migration 补齐真实注册和失败路径测试。
   - native plugin loader 不再是 runtime 公共主路径；相关测试和文档迁到隔离 namespace。
   - 插件 framework DTO 保持中性，具体行为留在 `zircon_plugins/*/runtime`。

6. **M5 Performance Pass**
   - 优先处理热路径：service resolution、registry name parsing、ECS query/cache、schedule execution、render extract clone/alloc、plugin manifest lookup。
   - 优化原则：先加诊断或计数测试，再减少 per-frame string lookup、重复 Vec 构建、长锁持有和不必要 clone。
   - 不引入新 benchmark 依赖，先用现有 diagnostics 和 focused performance acceptance tests。

7. **M6 Graphics/RHI Integration Review**
   - 等当前 WGPU/RHI active session 稳定后再收 graphics public surface。
   - 保持 RHI/RenderGraph/graphics 分层：neutral descriptors in RHI/framework，WGPU backend in `rhi_wgpu`，render feature execution in graphics。
   - 删除 root-level graphics 内部 re-export，保留明确 public contract。

8. **M7 Editor/UI Hotspot Split**
   - 在 host editor UI active work 稳定后拆大文件：retained host painter、host lifecycle、pane conversion、workbench projection。
   - 只按行为 family 拆，不做视觉或功能重写。
   - 测试树同步 folder-backed，禁止回到巨型 `tests.rs` 或 root `mod.rs`。

## Review And Test Plan

- 每个 milestone 进入前必须回答：owner 是 app/runtime core/manager/framework/runtime module/editor/plugin 哪一个；参考 Bevy/Fyrox/Unreal 哪些文件；旧路径如何删除。
- 每个 milestone 结束跑：
  - `python .\.codex\skills\zircon-project-skills\zr-runtime-interface-convergence\scripts\audit_runtime_structure.py --json`
  - focused `cargo check` / `cargo test` for touched crate
  - shared API touched 后扩展到 `cargo build --workspace --locked --verbose` 和 `cargo test --workspace --locked --verbose`
  - plugin touched 后跑 `cargo check/build/test --manifest-path zircon_plugins/Cargo.toml --workspace --locked`
  - profile/export touched 后跑 `validate-matrix.ps1 -RunProfileFeatureContract` 和 export platform contract matrix
- 每个 milestone 同步更新 `docs/` 对应模块文档，删除旧架构文档中的 stale crate/path 引用。

## Assumptions

- 继续在当前 `main` checkout 工作，不创建 branch/worktree。
- 现有未提交改动视为用户或其他 active session 所有；实施时不得回滚，必须先协调。
- 允许破坏性改动，默认不保留兼容 API、alias crate、shim module 或旧 re-export。
- 旧《全系统重构方案》中独立 `zircon_core/manager/framework/server` crate 视为历史目标；当前计划以 runtime-internal core spine 为准。