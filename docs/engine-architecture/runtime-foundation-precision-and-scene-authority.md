---
related_code:
  - zircon_runtime/crates/zr_math/src/lib.rs
  - zircon_runtime_interface/src/math.rs
  - zircon_runtime/src/core/math/mod.rs
  - zircon_runtime/tests/math_transform_helpers.rs
  - zircon_runtime/src/asset/assets/scene/mod.rs
  - zircon_runtime/src/asset/tests/assets/scene.rs
  - zircon_runtime/src/scene/components/mod.rs
  - zircon_runtime/src/scene/components/scene/mod.rs
  - zircon_runtime/src/scene/components/scene/transform.rs
  - zircon_runtime/src/scene/components/scene/activation.rs
  - zircon_runtime/src/core/framework/scene/mobility.rs
  - zircon_runtime/src/scene/components/scene/node.rs
  - zircon_runtime/src/scene/ecs/mod.rs
  - zircon_runtime/src/scene/ecs/query/mod.rs
  - zircon_runtime/src/scene/ecs/schedule.rs
  - zircon_runtime/src/core/framework/scene/system_stage.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage/store.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage/table.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage/sparse.rs
  - zircon_runtime/src/scene/reflect/world_reflection.rs
  - zircon_runtime/src/scene/dynamic_scene/mod.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/module/mod.rs
  - zircon_runtime/src/scene/module/core_error.rs
  - zircon_runtime/src/scene/module/default_level_manager.rs
  - zircon_runtime/src/scene/module/level_display_name.rs
  - zircon_runtime/src/scene/module/level_manager_contract.rs
  - zircon_runtime/src/scene/module/level_manager_lifecycle.rs
  - zircon_runtime/src/scene/module/level_manager_project_io.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/world/mod.rs
  - zircon_runtime/src/scene/world/world.rs
  - zircon_runtime/src/scene/world/bootstrap.rs
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/world/hierarchy.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/scene/world/query.rs
  - zircon_runtime/src/scene/world/records.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/typed_api.rs
  - zircon_runtime/src/scene/world/typed_api/fixed_components.rs
  - zircon_runtime/src/scene/render_extract/mod.rs
  - zircon_runtime/src/scene/serializer/mod.rs
  - zircon_runtime/src/scene/tests/world_basics.rs
  - zircon_runtime/src/scene/tests/asset_scene.rs
  - zircon_runtime/src/scene/tests/ecs_performance_acceptance.rs
implementation_files:
  - zircon_runtime/crates/zr_math/src/lib.rs
  - zircon_runtime_interface/src/math.rs
  - zircon_runtime/src/core/math/mod.rs
  - zircon_runtime/src/asset/assets/scene/mod.rs
  - zircon_runtime/src/scene/components/mod.rs
  - zircon_runtime/src/scene/components/scene/mod.rs
  - zircon_runtime/src/scene/components/scene/transform.rs
  - zircon_runtime/src/scene/components/scene/activation.rs
  - zircon_runtime/src/core/framework/scene/mobility.rs
  - zircon_runtime/src/scene/components/scene/node.rs
  - zircon_runtime/src/scene/ecs/mod.rs
  - zircon_runtime/src/scene/ecs/query/mod.rs
  - zircon_runtime/src/scene/ecs/schedule.rs
  - zircon_runtime/src/core/framework/scene/system_stage.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage/store.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage/table.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage/sparse.rs
  - zircon_runtime/src/scene/reflect/world_reflection.rs
  - zircon_runtime/src/scene/dynamic_scene/mod.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/module/mod.rs
  - zircon_runtime/src/scene/module/core_error.rs
  - zircon_runtime/src/scene/module/default_level_manager.rs
  - zircon_runtime/src/scene/module/level_display_name.rs
  - zircon_runtime/src/scene/module/level_manager_contract.rs
  - zircon_runtime/src/scene/module/level_manager_lifecycle.rs
  - zircon_runtime/src/scene/module/level_manager_project_io.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/world/mod.rs
  - zircon_runtime/src/scene/world/world.rs
  - zircon_runtime/src/scene/world/bootstrap.rs
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/world/hierarchy.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/scene/world/query.rs
  - zircon_runtime/src/scene/world/records.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/typed_api.rs
  - zircon_runtime/src/scene/world/typed_api/fixed_components.rs
  - zircon_runtime/src/scene/render_extract/mod.rs
  - zircon_runtime/src/scene/serializer/mod.rs
plan_sources:
  - user: 2026-05-07 继续推进里程碑
  - .codex/plans/Material UI + .ui.toml 全链路 UI 系统推进计划.md
  - user: 2026-04-15 implement the f64-ready runtime foundation plan with math/scene/asset/graphics boundaries
  - user: 2026-04-16 全仓库模块边界拆分与根入口去逻辑化
  - .codex/plans/全系统重构方案.md
  - user: 2026-05-08 Bevy-grade ECS / Reflect / Scene / Transform roadmap implementation
  - .codex/plans/ZirconEngine Bevy-Grade ECS Reflect Scene Transform Roadmap.md
tests:
  - tools/tests/test_frameworks_01_math_crate_boundary.py
  - zircon_runtime/crates/zr_math/src/tests/
  - zircon_runtime_interface/tests/math_contract.rs
  - zircon_runtime/tests/math_transform_helpers.rs
  - rustfmt --edition 2021 --check zircon_runtime/src/core/math/mod.rs zircon_runtime_interface/src/math.rs
  - cargo test -p zircon_runtime --test math_transform_helpers --locked --jobs 1 --target-dir F:\cargo-targets\zircon-runtime-math-warning-cleanup --message-format short --color never
  - zircon_runtime/src/asset/tests/assets/scene.rs
  - zircon_runtime/src/scene/tests/world_basics.rs
  - zircon_runtime/src/scene/tests/asset_scene.rs
  - zircon_runtime/src/scene/tests/ecs_performance_acceptance.rs
  - cargo check -p zircon_runtime --lib --locked --offline --message-format short --jobs 1 --target-dir E:\cargo-targets\zircon-native-ecs-systems --color never
  - cargo test -p zircon_runtime --lib scene::tests::ecs --locked --offline --message-format short --jobs 1 --target-dir E:\cargo-targets\zircon-native-ecs-systems --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib scene::tests --locked --offline --message-format short --jobs 1 --target-dir E:\cargo-targets\zircon-native-ecs-systems --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --locked
  - cargo test --workspace --locked
doc_type: module-detail
---

# Runtime Foundation Precision And Scene Authority

## Purpose

这份文档定义 `runtime foundation` 首里程碑的最终约束，并记录 2026-08-24
`zr_math` 物理硬切后的当前归属：

- `zr_math` 是唯一数学实现 owner，拥有 `Real`、glam alias、数值策略、空间类型、有限值校验、render downcast helper 和 `Transform`
- `zircon_runtime_interface::math` 只拥有带 `SchemaId` 的版本化产品 DTO，并显式投影批准的 `zr_math` surface
- `zircon_runtime::core::math` 只作为运行时公开入口，显式投影 `zr_math` 与 Interface schema，不保留重复实现
- `zircon_runtime::scene` 以 local authoring state + derived runtime state 为权威
- graphics scene renderer 明确承担 `runtime precision -> render precision` 的降级边界

这轮实现仍然是 `f32 + glam` 后端。共享 seam 为未来的精度 profile 提供唯一的接口层
入口，但它本身不构成 `f64-ready` 或 large-world 能力声明。

## Precision Contract

`zr_math` 不把“直接 re-export `glam`”当作最终公共契约，而是定义低依赖、可独立验证的精度边界：

- `Real`：当前是 `f32`
- `Vec2/Vec3/Vec4/Quat/Mat4`：runtime/backend alias
- `RenderScalar`：固定为 `f32`
- `RenderVec* / RenderMat4`：render seam 专用 alias

当前 canonical `CoordinateSchema` 同时冻结右手系、`+Y` up、`-Z` forward、列向量/列主序、
`[0,1]` clip depth、near-to-far 深度和 CCW canonical front face。单独的 material、mirror 或
backend pipeline 可以在自己的 descriptor 中声明例外，但不能修改 scene/asset/renderer 共用的
基础坐标约定。

`ZIRCON_PRECISION_PROFILE` 是当前版本化精度身份：runtime scalar 与 render scalar 均为
`f32`。任何新的 profile 都必须使用新的 profile/schema 版本，并同时通过 ABI、持久化、
reflection、cache 与 render narrowing 的迁移验收；不得只改变 `Real` alias。

`ZIRCON_UNIT_SCHEMA` 固定 canonical runtime base units 为 meter、radian 和 second；Editor
显示单位与 importer source unit 必须通过各自的 conversion receipt 适配，不能改写此存储含义。

统一 helper 负责两个职责：

- runtime 构造与校验：`compose_trs`、`transform_to_mat4`、`affine_inverse`、`is_finite_*`
- render 降级：兼容 `to_render_scalar`、`to_render_vec*`、`to_render_mat4`，以及携带误差
  receipt 的 `try_to_render_scalar`

因此未来如果 runtime 切换精度 profile，`zr_math` 是唯一实现入口；
`zircon_runtime_interface::math` 与 `zircon_runtime::core::math` 是显式、可审计的产品投影，
避免 scene、asset serializer 和 graphics renderer 各自定义私有 numeric alias。
这并不意味着 profile 切换只修改 alias/helper：scene canonical world storage、asset 和
save schema、reflection、animation、plugin/wire ABI、BuildSet/cache identity，以及
runtime-to-render 的可表示范围和相对坐标提取都必须有显式迁移与验证。GPU 侧仍可保持
`f32`，但只能在具名、受检的 render boundary 进行降精度，不能把绝对 world matrix
直接当作完整 large-world 方案。

## 2026-05-07 Runtime Math Ownership Cutover

> Historical note: 本节记录的 Interface 单 owner 决策已被 2026-08-24 M1 `zr_math` 物理硬切取代，
> 不再描述当前实现归属。

runtime-interface 收敛后，数学 DTO 与 helper 的中立定义已经在 `zircon_runtime_interface/src/math.rs` 中成为跨 runtime/editor 的事实。`zircon_runtime/src/core/math/mod.rs` 保留为运行时侧的稳定导入面，继续允许现有代码使用 `zircon_runtime::core::math::{Transform, Vec3, Mat4, ...}`，但它只 re-export 接口层合同。

本轮删除了 `zircon_runtime/src/core/math/precision/*` 与 `zircon_runtime/src/core/math/transform/*` 两组运行时私有重复实现。这样做不改变公开数学 API，目的是避免同一批 alias/helper 出现两个维护源，并清掉 runtime 编译中由旧私有模块产生的 unused warning。

## 2026-08-24 M1 `zr_math` Physical Hard Cut

数学算法与纯值类型已经从 `zircon_runtime_interface` 物理迁入独立 workspace crate
`zircon_runtime/crates/zr_math`。旧的 Interface 实现文件已经删除，没有复制实现、兼容 module、
通配符 re-export 或双 owner 过渡期。依赖方向固定为：

1. `zr_math` 仅依赖 `glam`、`serde` 和 `thiserror`，禁止依赖 Runtime 或 Runtime Interface；
2. `zircon_runtime_interface::math::schema` 保留 `CoordinateSchema`、`UnitSchema`、
   `PrecisionProfile` 及其 `SchemaId`，并从 `zr_math` 导入纯 convention enum；
3. Runtime Interface 与 Runtime facade 均使用显式 symbol list 投影批准的数学 surface，新增
   `zr_math` 符号不会自动泄漏到产品 API；
4. 产品消费者继续使用稳定的 Runtime/Interface product path；基础 crate 内部实现可以直接依赖
   `zr_math`，但不得建立 `zr_math -> product facade` 反向边。

这个结构对应 Unreal Runtime Core Math 的基础 owner 与稳定公开投影，也对应 Bevy
`bevy_math` 的低依赖独立 crate；Zircon 额外保留 Interface 的版本化 ABI schema 责任。

## Scene Runtime Authority

`zircon_runtime::scene::World` 现在把运行时 authority 固定为下列组件集合：

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

为了让这套 authority 在工程规模继续扩大时不再退化成单文件实现，当前代码树还新增了三个边界约束：

- `zircon_runtime/src/scene/world/mod.rs` 只作为 world 子系统入口；`World` 结构定义独立放到 `zircon_runtime/src/scene/world/world.rs`
- `zircon_runtime/src/scene/mod.rs` 现在只作为 runtime scene 吸收层导出层；`LevelSystem`、`DefaultLevelManager` 生命周期、project I/O、framework service contract 实现和 world driver 组装拆到 `zircon_runtime/src/scene/` 与 `zircon_runtime/src/scene/module/`
- `zircon_runtime/src/scene/components/` 保留 scene-domain 组件与 `Mobility` glue；viewport request/render packet/overlay DTO 分别归运行时 framework render 类型与 `zircon_editor::scene::viewport::render_packet`

## 2026-05-25 M12 Scene Storage Cutover

M12 的目标是删掉不再拥有行为的重复路径，而不是把所有固定 `World` map 一次性清空。当前所有权边界是：

- 固定 `World` map 仍然拥有持久化 scene product：稳定实体列表、`NodeRecord` 兼容投影、serde/project load state、asset import/export、editor hierarchy 行、固定组件的 reflection 适配，以及 render extract 所需的产品数据。
- 派生 map 只作为 runtime cache：`world_matrices`、`active_in_hierarchy` 和 `node_cache` 由 hierarchy/local transform/active 输入重建，不作为独立 truth 落盘。
- typed ECS storage 拥有运行期 component identity/presence、change ticks、query/cache metadata、systems、resources、events/messages/observers 和 schedule conflict detection；新系统行为应走 typed API，不再增加新的固定 map 直读写路径。
- reflection 只拥有 editor/remote 字段路由，`WorldReflection` 必须调用正常 `World` API，不能成为第二套存储。

这也是本轮 M12 先删除 plain entity-id 查询 cache helper、把测试 introspection 收到 `#[cfg(test)]`，但暂不删除固定组件 map 的原因。`DynamicScene` 旧文档迁移、`WorldReflection` DTO 路由、render-layer legacy mask 和 fixed component maps 都还有明确产品责任；后续只有在替代所有权落地后才能硬删。

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

`zircon_runtime::asset::assets::SceneAsset` 的数值字段现在统一走 `zircon_runtime::core::math::Real`：

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

runtime framework render 现在显式把 renderer 当作精度降级边界处理：

- scene extract 继续使用 runtime alias 类型
- uniform、clear color、overlay line vertex、model matrix 打包前统一调用 `to_render_*`
- GPU/WGSL 侧继续固定 `f32`

scene/runtime 到 renderer 的当前入口由 `zircon_runtime/src/scene/render_extract/mod.rs` 和 `zircon_runtime/src/core/framework/render/*` 承担。`World::to_render_frame_extract()` 会通过 `RenderExtractProducer` 构建 frame extract，renderer-facing DTO 保留在 framework render 边界，避免 scene world 直接依赖具体 GPU 后端。

这意味着未来 runtime 升成 `f64` 时：

- CPU scene/world/transform 可以逐步升精度
- render extract 和 GPU 上传继续在 renderer 边界显式 downcast
- 不需要要求 WGSL、纹理、颜色缓冲、uniform layout 跟着升精度

## Validation Shape

这轮实现新增或收紧了以下验证面：

- `zircon_runtime/tests/math_transform_helpers.rs`
  - runtime 公开入口继续通过 `zircon_runtime::core::math` 提供 `Transform`、glam alias 和 TRS helper
  - helper 由 `zr_math` 拥有，Runtime/Interface 不保留重复实现
  - 2026-05-07 focused 验证通过：`cargo test -p zircon_runtime --test math_transform_helpers --locked --jobs 1 --target-dir F:\cargo-targets\zircon-runtime-math-warning-cleanup --message-format short --color never`，3 passed；编译输出剩余 warning 位于 graphics/ui 等既有区域，不再包含已删除的 runtime-local math owner warning 组
- `zircon_runtime/src/asset/tests/assets/scene.rs`
  - scene asset roundtrip
  - active/render layer/mobility 缺省字段回退
- `zircon_runtime/src/scene/tests/world_basics.rs` 与 broader `scene::tests`
  - runtime default components
  - active propagation
  - world matrix rebuild
  - static mutation constraints
  - render layer + mobility roundtrip
- `zircon_runtime/src/scene/tests/ecs_performance_acceptance.rs`
  - M11/M12 前置 hot path cache rebuild gate
  - transform projection 稳定性
  - changed-filter run-window 行为

2026-05-25 M11/M12 gate 已在当前 dirty workspace 下通过：`cargo test -p zircon_runtime --lib scene::tests::ecs --locked --offline --message-format short --jobs 1 --target-dir E:\cargo-targets\zircon-native-ecs-systems --color never -- --test-threads=1 --nocapture` 报告 `145 passed; 0 failed`，`cargo test -p zircon_runtime --lib scene::tests --locked --offline --message-format short --jobs 1 --target-dir E:\cargo-targets\zircon-native-ecs-systems --color never -- --test-threads=1 --nocapture` 报告 `179 passed; 0 failed`，`cargo check -p zircon_runtime --lib --locked --offline --message-format short --jobs 1 --target-dir E:\cargo-targets\zircon-native-ecs-systems --color never` 通过。该证据只接受 scene/ECS gate，不代表全 workspace CI 已在当前脏工作区通过。

2026-08-24 M1 gate 的当前证据为：math boundary guard `4/4`；`zr_math` locked build
与 lib tests 通过；Runtime Interface `math_contract` 6 个 public-contract tests 通过。
`zircon_runtime` product build 已成功编译 math/Interface 层，随后被 `zr_rhi_wgpu` 当前源中的
5 个外部错误阻塞，因此该证据不声明 Runtime、App、Editor 或 workspace 全绿。完整 job id、耗时
与阻塞指纹记录在
`docs/plans/zircon_runtime/frameworks/01/2026-08-24-m1-zr-math-physical-hard-cut.md`。

## Future f64 Switch Boundary

如果后续真的切 runtime `f64`，本轮实现希望把主改动尽量压缩到下面几处：

- `zr_math` backend alias
- `zr_math` render conversion helper
- `zircon_runtime_interface::math::schema` 的版本化 profile 与迁移规则
- runtime scene / asset serializer 中依赖容差的测试

不应该再把精度切换扩散成：

- scene 内部重新定义另一套 transform 规则
- asset 文件格式升级
- renderer/WGSL 全链路升精度
- editor 兼容查询层重新做一轮大拆
