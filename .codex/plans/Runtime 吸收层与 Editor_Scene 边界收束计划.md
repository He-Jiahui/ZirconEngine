# Runtime 吸收层与 Editor/Scene 边界收束计划

## Summary

- 最终架构收束为：
  - `zircon_app`：进程入口、profile 选择、主循环宿主
  - `zircon_core`：生命周期、依赖、注册、调度内核
  - `zircon_manager`：统筹层，负责 service name、resolver、handle、激活与访问入口
  - `zircon_framework`：纯抽象契约层，只定义 render/input/scene/ui/script 等框架接口与共享数据，不实现具体业务
  - `zircon_runtime`：运行时吸收层，物理吸收高层业务实现
  - `zircon_editor`：编辑器宿主与作者态逻辑
- `server` 只保留给真正服务端语义；非网络架构层统一不用 `server` 命名。
- 本轮不保留兼容 shim、alias crate、旧路径 re-export；每个迁移阶段完成时都直接切到新结构。

## Key Changes

### Workspace 收束

- `zircon_entry` 重命名并收束为 `zircon_app`，同时吸收 `zircon_builtin_modules`，不再保留 `module_set -> builtin_registry -> builtin_modules` 三层转发。
- 新建 `zircon_framework`，吸收当前这些“契约但非实现”的内容：
  - `zircon_render_server`
  - `zircon_scene_protocol`
  - `zircon_input_protocol`
  - `zircon_manager` 里现有的 manager trait 定义
- 保留 `zircon_manager`，但职责收窄为统筹访问层：
  - 保留 `ManagerResolver`
  - 保留 `*_MANAGER_NAME`
  - 保留 handle/resolver 入口
  - 所有 trait/DTO/协议类型改为引用 `zircon_framework::*`
- 新建 `zircon_runtime` 作为吸收层，物理吸收高层运行时业务 crate：
  - `zircon_foundation`
  - `zircon_platform`
  - `zircon_input`
  - `zircon_asset`
  - `zircon_graphics` 的高层业务部分
  - `zircon_scene`
  - `zircon_ui`
  - `zircon_script`
- 本轮不吸收底层基础设施 crate；它们继续独立：
  - `zircon_core`
  - `zircon_math`
  - `zircon_module`
  - `zircon_manager`
  - `zircon_framework`
  - `zircon_resource`
  - `zircon_rhi`
  - `zircon_rhi_wgpu`
  - `zircon_render_graph`
- 可选扩展功能 crate 暂不并入 `zircon_runtime`：
  - `zircon_physics`
  - `zircon_sound`
  - `zircon_texture`
  - `zircon_net`
  - `zircon_navigation`
  - `zircon_particles`
  - `zircon_animation`

### Scene / Editor 边界重建

- `zircon_runtime::scene` 只保留运行时世界权威模型：
  - entity/hierarchy/transform/renderable/light/camera/serialization/render extract
- 从运行时世界移除 editor 态数据与 editor 视图生成逻辑：
  - `selected_entity`
  - `selected_node()/set_selected()`
  - `SceneViewportSettings`
  - `SceneViewportTool`
  - `TransformSpace`
  - `DisplayMode`
  - `GridMode`
  - `ViewOrientation`
  - gizmo provider/registry/build logic
  - selection highlight/anchor/handle overlay 生成
  - preview lighting/skybox authoring开关
- `zircon_editor::scene` 接管作者态场景逻辑：
  - 选择集
  - viewport 工具状态
  - camera override
  - handle/gizmo/pointer route
  - scene overlay 生成
  - scene inspector 与 scene 编辑命令
- `zircon_framework::render` 只保留中性渲染契约数据：
  - viewport camera snapshot
  - base render extract packet
  - overlay packet DTO
  - render viewport descriptor / quality / stats / handles
- `zircon_runtime::scene` 只产生基础 scene extract。
- `zircon_editor::scene` 在 editor 模式下基于 runtime world + editor state 生成 overlay packet，再交给 graphics。
- Runtime preview 若需要“当前选中对象/相机聚焦”，状态不再写回 world，而是归 `zircon_app` 或 `zircon_runtime` 的宿主控制器局部持有。

### Editor 收束

- `zircon_editor_ui` 全量并入 `zircon_editor`，独立 crate 删除。
- `zircon_editor` 顶层目录固定收束为 `zircon_editor/src/core/`、`zircon_editor/src/scene/`、`zircon_editor/src/ui/`。
- `core/` 负责：
  - editor command/history/intent
  - editor event runtime/journal/replay
  - project/session/startup
  - 跨 UI/scene 的纯编辑器状态机
- `scene/` 负责：
  - world slot
  - scene selection
  - viewport controller/handles/pointer
  - scene snapshot projection
  - 与 `zircon_runtime::scene` 的交互桥
- `ui/` 负责：
  - 原 `zircon_editor_ui` binding/template/reflection/control
  - host/slint/template runtime
  - workbench/layout/view/snapshot
  - binding dispatch 与 UI projection
- `zircon_editor/src/lib.rs` 只保留结构性 wiring 和对外 re-export，不再承载混合实现。

### App / Runtime 主链

- `zircon_app` 只做宿主：
  - 创建 `CoreRuntime`
  - 注册 `zircon_runtime` 与扩展模块
  - 根据 profile 激活 `zircon_editor`
  - 驱动 editor/runtime/headless 主循环
- `zircon_runtime` 负责“主运行链业务”的统一注册面，而不是继续分散在多个高层 crate。
- `zircon_app` 的 editor 启动路径固定为：
  - core
  - manager
  - runtime
  - editor
- `zircon_app` 的 runtime 启动路径固定为：
  - core
  - manager
  - runtime
- 不再保留当前 `zircon_entry` 内部直接操纵旧 `Scene` 选择态与 render extract 的路径；这些调用统一改走新 runtime/editor 分层。

## Public APIs / Interfaces

- 新增 crate：
  - `zircon_app`
  - `zircon_framework`
  - `zircon_runtime`
- 删除 crate：
  - `zircon_builtin_modules`
  - `zircon_editor_ui`
- 高层业务 crate 最终并入 `zircon_runtime` 后不再保留独立 Cargo package。
- `zircon_manager` 对外保留稳定入口，但返回/引用的 trait 与共享 DTO 全部改为 `zircon_framework::*` 所定义。
- `zircon_framework` 统一提供这些域：
  - `framework::input`
  - `framework::scene`
  - `framework::render`
  - `framework::ui`
  - `framework::script`
- `zircon_runtime::scene::World` 不再暴露 selection API，也不序列化 selection。
- Editor viewport authoring API 固定放在 `zircon_editor::scene`，不再从 runtime world 导出。
- Graphics 只消费 `zircon_framework::render` 的中性 render packet，不依赖 editor crate。

## Migration Order

- 第 1 阶段：先建立 `zircon_framework` 并迁走 trait/protocol/render contract；同步改造 `zircon_manager` 为 resolver-only 统筹层。
- 第 2 阶段：建立 `zircon_runtime`，先吸收 foundation/platform/input/asset/scene/script/ui 的高层业务注册面，再拆 `zircon_graphics` 的高层业务到 `zircon_runtime::graphics`。
- 第 3 阶段：重建 scene/editor 边界；从 world 删除 selection 与 viewport authoring 状态，把 overlay/gizmo/handle 逻辑移到 `zircon_editor::scene`。
- 第 4 阶段：把 `zircon_editor_ui` 并入 `zircon_editor`，完成 `core/scene/ui` 三分目录并迁完所有引用。
- 第 5 阶段：把 `zircon_entry` 和 `zircon_builtin_modules` 收束为 `zircon_app`，切换 editor/runtime/headless 启动链到新结构。
- 每一阶段结束时直接删旧路径与旧 crate 引用，不保留过渡 alias。

## Test Plan

- Workspace 验收：
  - `cargo check --workspace`
  - `cargo test --workspace`
  - workspace 中不再存在 `zircon_entry`、`zircon_builtin_modules`、`zircon_editor_ui`
- 边界验收：
  - `zircon_runtime` 不依赖 `zircon_editor`
  - `zircon_framework` 不包含任何具体业务实现
  - `zircon_manager` 只保留统筹/resolver/handle/service-name 角色
  - 非网络架构层不存在 `server` 命名残留
- Scene/Editor 验收：
  - `World` 序列化结果不再包含 selection
  - editor selection、viewport tool、camera override 仅存在于 `zircon_editor::scene`
  - gizmo/handle/selection overlay 仍能完整显示并参与 pointer route
- Graphics/Render 验收：
  - graphics 仅通过 `zircon_framework::render` 消费 render packet
  - editor 和 runtime 两条路径都能正常渲染同一 world
- Bootstrap 验收：
  - `zircon_app` 的 editor profile 能启动 editor
  - `zircon_app` 的 runtime profile 能启动 runtime preview
  - 旧 `module_set/builtin_registry` 转发链已删除
- 目录结构验收：
  - `zircon_editor` 根下只保留 `core/`、`scene/`、`ui/` 三个主域
  - root wiring 文件保持结构性，不再承载混合实现

## Assumptions

- `manager` 是统筹层，`framework` 是其下属抽象契约层；`framework` 不实现业务。
- `runtime` 是吸收层，但本轮只吸收高层业务，不吞并底层 graphics 技术基础设施。
- 可选扩展模块先保持独立 crate，等主运行链收束稳定后再决定是否吸收。
- 本计划默认允许大规模 crate 重命名与物理迁移，并以最终落点为准，不为旧名称保留长期兼容。
