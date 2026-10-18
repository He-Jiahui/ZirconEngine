# f64-Ready Runtime Foundation Implementation Plan

## Summary
- 目标：完成 `runtime foundation` 首里程碑，并把数学与运行时边界设计成“后续可切 `f64`”，但本轮实现仍使用 `f32` + `glam` 后端。
- 范围：主实现集中在 `zircon_math`、`zircon_scene`，并补足 `zircon_graphics` 的最小适配，保证全工作区持续可编译、可测试。
- 参考语义：
  - Godot `Node3D`：local TRS 为写入口，global transform 为 dirty-driven 派生缓存
  - Fyrox `Base`：local enabled 与 global enabled 分离，父状态传播到子树
  - Piccolo `Transform`：TRS 生成矩阵，运行时与渲染边界分层
- 明确后移：`MeshFilter/MeshRenderer` 新公共组件拆分、hierarchy inspector UI、RTS 视口控制重构。

## Public API And Precision Contract
- `zircon_math`
  - 停止直接 `pub use glam::{...}` 作为最终公共契约，改为由 `zircon_math` 自己定义精度边界。
  - 定稿公共精度契约：
    - `type Real = f32`（本轮）
    - `type Vec2/Vec3/Vec4/Quat/Mat4 = zircon_math backend aliases`
    - `type RenderScalar = f32`
  - 公共规则：
    - 右手坐标
    - `+Y` up，`-Z` forward
    - 列主序 `Mat4`
    - 层级组合顺序固定为 `parent_world * local_matrix`
  - 新增统一 helper：
    - `transform_to_mat4`
    - `compose_trs`
    - `affine_inverse`
    - `is_finite_*`
    - `to_render_vec* / to_render_mat4 / to_render_scalar`
- `zircon_scene`
  - 运行时公共组件/类型定稿为：
    - `LocalTransform { transform: Transform }`
    - `WorldMatrix(Mat4)`，权威世界变换缓存
    - `ActiveSelf(bool)`
    - `ActiveInHierarchy(bool)`，派生只读
    - `RenderLayerMask(u32)`，默认 `0x0000_0001`
    - `Mobility::{Dynamic, Static}`
  - 运行时查询/修改接口定稿为：
    - `world_matrix`
    - `active_self` / `set_active_self`
    - `active_in_hierarchy`
    - `render_layer_mask` / `set_render_layer_mask`
    - `mobility` / `set_mobility`
  - 兼容层继续保留：
    - `NodeId`
    - `SceneNode`
    - `NodeRecord`
    - `world_transform()`
    - `nodes()/find_node()`
    - 这些都改为从新 runtime 模型投影，不再是权威存储。
- `zircon_asset`
  - 场景序列化相关数值字段统一改用 `zircon_math::Real`，不再直接写死 `f32`。
  - 文件形状不变，仍是 TOML 浮点；本轮不做格式版本升级。
- `zircon_graphics`
  - 正式定义为精度降级边界：运行时/场景用 `Real`，render extract、CPU overlay 顶点、GPU uniform、WGSL 一律收口到 `RenderScalar = f32`。
  - 后续切 `f64` 时，不要求 WGSL 或 GPU 侧同步升精度。

## Implementation Changes
- 数学层收口
  - 在 [zircon_math/src/lib.rs](/E:/Git/ZirconEngine/zircon_math/src/lib.rs) 拆出 backend/precision seam，保留现有 API 名字，但来源统一改为 `zircon_math` 自有别名与 helper。
  - 所有 scene/runtime 代码禁止直接导入 `glam` 原生类型；只能经由 `zircon_math`。
  - 本轮 backend 仍绑定到 `glam` 的 `f32` 类型；后续切 `f64` 的唯一主入口是 `zircon_math` backend alias。
- 场景运行时模型升级
  - 在 [zircon_scene/src/components.rs](/E:/Git/ZirconEngine/zircon_scene/src/components.rs) 和 [zircon_scene/src/world/*](/E:/Git/ZirconEngine/zircon_scene/src/world.rs) 引入 `WorldMatrix`、`ActiveSelf`、`ActiveInHierarchy`、`RenderLayerMask`、`Mobility` 存储。
  - `LocalTransform` 保持本地 TRS 权威；世界态只认 `WorldMatrix`。
  - 内部重建顺序固定为：
    1. hierarchy validity
    2. `ActiveInHierarchy`
    3. `WorldMatrix`
    4. 兼容 node cache
  - 规则固定为：
    - 修改本地 TRS 只写 `LocalTransform`
    - `WorldMatrix = parent_world * local_matrix`
    - `active_in_hierarchy = parent.active_in_hierarchy && active_self`
    - `RenderLayerMask` 不继承
    - `Mobility::Static` 禁止常规运行时 transform/reparent 修改
    - `Static` 节点禁止挂到 `Dynamic` 父节点下
  - bootstrap/import 路径允许初始化 `Static` 实体，但初始化后进入相同约束。
- 序列化与 IO
  - 在 `zircon_asset` 的 scene asset 结构中把 transform/camera/light 的运行时标量改为 `Real`，project IO 通过 `zircon_math` helper 读写。
  - `ActiveInHierarchy`、`WorldMatrix` 不落盘。
  - `active` 映射到 `ActiveSelf`；新增可选 `render_layer_mask` 与 `mobility` 字段，缺省回退到默认值。
- 下游兼容与编译门槛
  - 在 [zircon_graphics/src/scene/scene_renderer.rs](/E:/Git/ZirconEngine/zircon_graphics/src/scene/scene_renderer.rs) 和相关资源代码中，把当前 renderer 完整适配到嵌套的 `SceneViewportRenderPacket { scene, overlays, preview }`。
  - 所有 renderer/overlay/uniform 构造通过 `to_render_*` helper 做显式 downcast，不允许散落的 `as f32` 成为长期路径。
  - `zircon_editor` 继续依赖兼容查询层，不提前推进第二里程碑的 inspector/runtime 分离改造。
- 文档与约束
  - 更新运行时规范文档，明确写清：
    - `runtime precision` 与 `render precision` 解耦
    - 未来切 `f64` 的唯一受控入口是 `zircon_math`
    - 其他 crate 不得直接把精度类型写死在公共 API 上

## Test Plan
- `zircon_math`
  - TRS 组合矩阵与输入平移/旋转/缩放一致。
  - 父子矩阵组合得到正确世界位置。
  - `affine_inverse` 往返误差在当前 `Real=f32` 容忍范围内。
  - `to_render_*` 对有限值转换正确，非有限值按约定拒绝或夹紧。
- `zircon_scene`
  - 新实体默认 `ActiveSelf(true)`、`ActiveInHierarchy(true)`、`RenderLayerMask(1)`、`Mobility::Dynamic`。
  - 父关闭后子树 `ActiveInHierarchy(false)`，但 `ActiveSelf` 保持原值。
  - 重新激活父节点后子树恢复。
  - `WorldMatrix` 在 reparent 与 local transform 更新后正确重建。
  - `Static` 实体拒绝运行时 transform 更新与非法挂接。
  - `render_layer_mask` set/get/serialize/deserialize 正常。
  - 旧场景资源缺少新字段时仍按默认值加载。
  - 兼容 `world_transform()` / `nodes()` / `find_node()` 继续满足 editor 当前使用。
- `zircon_asset`
  - scene asset roundtrip 在当前 `Real` 精度下保持稳定。
  - `Real` 改变时不需要文件格式迁移测试夹具重写，只需数值容忍区间调整。
- `zircon_graphics`
  - renderer 完整消费新 packet 结构。
  - 所有 render uniform 与 overlay 顶点来自显式 `to_render_*` 转换。
  - grid、fallback light、scene gizmo、selection overlay 仍输出非背景帧。
- 验收门槛
  - `cargo test -p zircon_math --locked`
  - `cargo test -p zircon_scene --locked`
  - `cargo test -p zircon_graphics --locked`
  - `cargo test -p zircon_editor --locked`
  - `cargo test --workspace --locked`

## Assumptions And Defaults
- 本轮 `Real = f32`，只做 `f64-ready` 架构，不在首里程碑直接把 runtime 后端切到 `f64`。
- 后续若切 `f64`，预期主改动集中在 `zircon_math` backend alias、少量 helper 与测试容忍区间；`zircon_graphics` 继续保持 `f32` render seam。
- `Material`、GPU 纹理、WGSL uniform、颜色缓冲等纯渲染域数据继续保持 `f32`。
- `Transform` 名称继续保留为本地 TRS 类型；`WorldMatrix` 是运行时权威世界态。
- 本轮不落新的 `MeshFilter/MeshRenderer` 公共组件，不动 hierarchy inspector 与 RTS 控制系统。
