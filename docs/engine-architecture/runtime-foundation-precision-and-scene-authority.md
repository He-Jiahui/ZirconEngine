---
related_code:
  - zircon_math/src/lib.rs
  - zircon_math/tests/precision_contract.rs
  - zircon_asset/src/assets/scene.rs
  - zircon_asset/src/tests/assets/scene.rs
  - zircon_scene/src/components.rs
  - zircon_scene/src/lib.rs
  - zircon_scene/src/module.rs
  - zircon_scene/src/module/core_error.rs
  - zircon_scene/src/module/default_level_manager.rs
  - zircon_scene/src/module/level_display_name.rs
  - zircon_scene/src/module/level_manager_facade.rs
  - zircon_scene/src/module/level_manager_lifecycle.rs
  - zircon_scene/src/module/level_manager_project_io.rs
  - zircon_scene/src/module/manager_access.rs
  - zircon_scene/src/module/module_descriptor.rs
  - zircon_scene/src/module/service_names.rs
  - zircon_scene/src/module/world_driver.rs
  - zircon_scene/src/world.rs
  - zircon_scene/src/world/world.rs
  - zircon_scene/src/world/bootstrap.rs
  - zircon_scene/src/world/derived_state.rs
  - zircon_scene/src/world/hierarchy.rs
  - zircon_scene/src/world/project_io.rs
  - zircon_scene/src/world/query.rs
  - zircon_scene/src/world/records.rs
  - zircon_scene/src/world/render.rs
  - zircon_scene/tests/runtime_foundation.rs
  - zircon_graphics/src/scene/resources/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/mod.rs
  - zircon_graphics/src/scene/scene_renderer/primitives/mod.rs
  - zircon_graphics/src/tests/project_render.rs
  - zircon_graphics/src/tests/scene_overlay.rs
implementation_files:
  - zircon_math/src/lib.rs
  - zircon_asset/src/assets/scene.rs
  - zircon_scene/src/components.rs
  - zircon_scene/src/module.rs
  - zircon_scene/src/module/core_error.rs
  - zircon_scene/src/module/default_level_manager.rs
  - zircon_scene/src/module/level_display_name.rs
  - zircon_scene/src/module/level_manager_facade.rs
  - zircon_scene/src/module/level_manager_lifecycle.rs
  - zircon_scene/src/module/level_manager_project_io.rs
  - zircon_scene/src/module/manager_access.rs
  - zircon_scene/src/module/module_descriptor.rs
  - zircon_scene/src/module/service_names.rs
  - zircon_scene/src/module/world_driver.rs
  - zircon_scene/src/world.rs
  - zircon_scene/src/world/world.rs
  - zircon_scene/src/world/bootstrap.rs
  - zircon_scene/src/world/derived_state.rs
  - zircon_scene/src/world/hierarchy.rs
  - zircon_scene/src/world/project_io.rs
  - zircon_scene/src/world/query.rs
  - zircon_scene/src/world/records.rs
  - zircon_scene/src/world/render.rs
  - zircon_graphics/src/scene/resources/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/mod.rs
  - zircon_graphics/src/scene/scene_renderer/primitives/mod.rs
plan_sources:
  - user: 2026-04-15 implement the f64-ready runtime foundation plan with math/scene/asset/graphics boundaries
  - user: 2026-04-16 全仓库模块边界拆分与根入口去逻辑化
  - .codex/plans/全系统重构方案.md
tests:
  - zircon_math/tests/precision_contract.rs
  - zircon_scene/tests/runtime_foundation.rs
  - zircon_asset/src/tests/assets/scene.rs
  - zircon_graphics/src/tests/project_render.rs
  - zircon_graphics/src/tests/scene_overlay.rs
  - cargo test -p zircon_scene --offline
  - cargo test -p zircon_math --locked
  - cargo test -p zircon_asset --locked
  - cargo test -p zircon_scene --locked
  - cargo test -p zircon_graphics --locked
  - cargo test -p zircon_editor --locked
  - cargo test --workspace --locked
doc_type: module-detail
---

# Runtime Foundation Precision And Scene Authority

## Purpose

这份文档定义 `runtime foundation` 首里程碑的最终约束：

- `zircon_math` 成为唯一精度 seam
- `zircon_scene::World` 以 local authoring state + derived runtime state 为权威
- `zircon_asset` 只持久化 authoring/runtime 输入态，不持久化派生态
- `zircon_graphics` 明确承担 `runtime precision -> render precision` 的降级边界

这轮实现仍然是 `f32 + glam` 后端，但边界已经按未来可切 `f64` 的方向收口。

## Precision Contract

`zircon_math` 现在不再把“直接 re-export `glam`”当作最终公共契约，而是先定义自己的精度边界：

- `Real`：当前是 `f32`
- `Vec2/Vec3/Vec4/Quat/Mat4`：runtime/backend alias
- `RenderScalar`：固定为 `f32`
- `RenderVec* / RenderMat4`：render seam 专用 alias

统一 helper 负责两个职责：

- runtime 构造与校验：`compose_trs`、`transform_to_mat4`、`affine_inverse`、`is_finite_*`
- render 降级：`to_render_scalar`、`to_render_vec*`、`to_render_mat4`

因此未来如果 runtime 切到 `f64`，主入口应只在 `zircon_math` backend alias 与 helper，而不是在 `zircon_scene`、`zircon_asset`、`zircon_graphics` 里散落改类型。

## Scene Runtime Authority

`zircon_scene::World` 现在把运行时 authority 固定为下列组件集合：

- `LocalTransform`
- `WorldMatrix`
- `ActiveSelf`
- `ActiveInHierarchy`
- `RenderLayerMask`
- `Mobility`

对应规则：

- local TRS 是唯一可写 authoring/runtime 输入态
- `WorldMatrix = parent_world * local_matrix`
- `ActiveInHierarchy = parent.ActiveInHierarchy && ActiveSelf`
- `RenderLayerMask` 不继承
- `Mobility::Static` 禁止常规 runtime transform 修改
- `Mobility::Static` 禁止常规 runtime reparent
- `Static` 初始化允许通过 scene/project restore 直接建立，但初始化完成后进入同一套约束

派生状态重建顺序固定为：

1. hierarchy validity
2. `ActiveInHierarchy`
3. `WorldMatrix`
4. compatibility node cache

这里的 “hierarchy validity” 负责在 derived rebuild 前清掉缺失父节点、自指和环路链。

为了让这套 authority 在工程规模继续扩大时不再退化成单文件实现，当前代码树还新增了两个边界约束：

- `zircon_scene/src/world.rs` 现在只作为 world 子系统入口；`World` 结构定义独立放到 `zircon_scene/src/world/world.rs`
- `zircon_scene/src/module.rs` 现在只作为 scene module 导出层；`DefaultLevelManager` 生命周期、project I/O、facade 适配和 descriptor 组装拆到 `zircon_scene/src/module/`

## Compatibility Layer

editor 当前还没有完成 hierarchy inspector/runtime 分离，所以兼容查询层保留：

- `NodeId`
- `SceneNode`
- `NodeRecord`
- `world_transform()`
- `nodes()`
- `find_node()`

但这些都不再是权威存储：

- `SceneNode.transform` 继续投影 local TRS，供 editor 现有编辑流复用
- `world_transform()` 由 `WorldMatrix` 反投影出来
- `NodeRecord.active` 对应 `ActiveSelf`
- `NodeRecord.render_layer_mask` / `NodeRecord.mobility` 作为新的持久化字段参与 roundtrip

## Asset Boundary

`zircon_asset::SceneAsset` 的数值字段现在统一走 `zircon_math::Real`：

- `TransformAsset`
- `SceneCameraAsset`
- `SceneDirectionalLightAsset`

同时 `SceneEntityAsset` 新增：

- `render_layer_mask`
- `mobility`

它们都带默认值，因此旧 TOML scene 文件仍可直接读取：

- `active` 缺省回退 `true`
- `render_layer_mask` 缺省回退 `0x0000_0001`
- `mobility` 缺省回退 `Dynamic`

不会落盘的字段：

- `WorldMatrix`
- `ActiveInHierarchy`

因此 scene 文件依然只描述 authoring/runtime 输入态，而不是运行中缓存。

## Graphics Precision Seam

`zircon_graphics` 现在显式把 renderer 当作精度降级边界处理：

- scene extract 继续使用 runtime alias 类型
- uniform、clear color、overlay line vertex、model matrix 打包前统一调用 `to_render_*`
- GPU/WGSL 侧继续固定 `f32`

renderer 现在也已经从单文件实现整理成目录化子树：

- `scene/resources/mod.rs` 负责 scene 资源入口，具体 streamer / GPU resource / fallback 逻辑下沉到 `scene/resources/*`
- `scene_renderer/core/mod.rs` 负责 render core 入口，具体 scene uniform / history / target / render orchestration 下沉到 `scene_renderer/core/*`
- `scene_renderer/mesh.rs` 负责 mesh draw 构建与 pipeline cache
- `scene_renderer/overlay.rs` 负责 grid、selection、scene gizmo、icon overlay pass
- `scene_renderer/primitives/mod.rs` 负责 primitive 入口，具体 packing / vertex / fallback / geometry helper 下沉到 `scene_renderer/primitives/*`

这意味着未来 runtime 升成 `f64` 时：

- CPU scene/world/transform 可以逐步升精度
- render extract 和 GPU 上传继续在 renderer 边界显式 downcast
- 不需要要求 WGSL、纹理、颜色缓冲、uniform layout 跟着升精度

## Validation Shape

这轮实现新增或收紧了以下验证面：

- `zircon_math/tests/precision_contract.rs`
  - precision alias
  - TRS helper
  - affine inverse
  - finite / render conversion
- `zircon_scene/tests/runtime_foundation.rs`
  - runtime default components
  - active propagation
  - world matrix rebuild
  - static mutation constraints
  - render layer + mobility roundtrip
- `zircon_asset` scene tests
  - scene asset roundtrip
  - 旧文档缺省字段回退
- `zircon_graphics` render tests
  - nested viewport packet 继续可消费
  - wire-only 与 shaded 输出继续可区分

## Future f64 Switch Boundary

如果后续真的切 runtime `f64`，本轮实现希望把主改动尽量压缩到下面几处：

- `zircon_math` backend alias
- `zircon_math` render conversion helper
- `zircon_math` / `zircon_scene` / `zircon_asset` 中依赖容差的测试

不应该再把精度切换扩散成：

- scene 内部重新定义另一套 transform 规则
- asset 文件格式升级
- renderer/WGSL 全链路升精度
- editor 兼容查询层重新做一轮大拆
