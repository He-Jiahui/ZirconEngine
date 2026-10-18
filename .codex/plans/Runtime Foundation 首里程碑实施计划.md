# Runtime Foundation 首里程碑实施计划

## Summary
- 目标：先完成 `runtime foundation`，把基础数学、Entity 层级、变换传播、Active、渲染 Layer、静态物件语义定成统一规范，并落成可运行实现。
- 范围：本轮只做 `zircon_math + zircon_scene` 主体，以及保证 `zircon_graphics/zircon_editor` 持续可编译的最小兼容适配。
- 明确后移：`MeshFilter/MeshRenderer` 新组件拆分、hierarchy inspector UI、RTS 视口控制重写，留到下一里程碑。
- 参考基线：Godot `Node3D` 的 local/global dirty 传播，Fyrox 的 local enabled/global enabled 语义，Piccolo 的 TRS 到 `Matrix4x4` 运行时模型。

## Public APIs / Runtime Spec
- `zircon_math`
  - 继续以 `Vec2/Vec3/Vec4/Quat/Mat4/Transform` 作为公共数学表面，底层后端保持 `glam` SIMD。
  - 明确全仓库规范：`f32`、右手坐标、`+Y` 向上、`-Z` 向前、列主序 `Mat4`、`parent_world * local_matrix` 为层级组合顺序。
  - 由 `zircon_math` 提供统一 TRS 组合、仿射逆、点/向量变换、有限值检查等 helper；其他 crate 不再直接依赖 `glam`。
- `zircon_scene`
  - 运行时术语正式以 `Entity` 为准；现有 `NodeId/SceneNode/NodeRecord/NodeKind` 仅作为兼容别名保留到 editor/rendering 里程碑。
  - 公共组件/类型定稿为：
    - `LocalTransform { transform: Transform }`
    - `WorldMatrix(Mat4)`，作为权威世界变换缓存
    - `ActiveSelf(bool)`
    - `ActiveInHierarchy(bool)`，派生只读
    - `RenderLayerMask(u32)`，默认 `0x0000_0001`
    - `Mobility::{Dynamic, Static}`
  - 世界查询/修改接口定稿为：
    - `world_matrix`
    - `active_self` / `set_active_self`
    - `active_in_hierarchy`
    - `render_layer_mask` / `set_render_layer_mask`
    - `mobility` / `set_mobility`
  - 保留兼容查询给现有下游使用，但兼容返回值不再是权威存储；权威只认 `WorldMatrix`。
- 序列化规范
  - `SceneAsset` 中的 `active` 映射到 `ActiveSelf`。
  - 新增可选 `render_layer_mask` 与 `mobility` 字段；旧资源缺省时回退到 `layer=1`、`Dynamic`。
  - `ActiveInHierarchy`、`WorldMatrix` 不序列化。

## Implementation Changes
- Phase 1：数学层规范化
  - 收拢所有 scene/runtime 数学调用到 `zircon_math`，把当前散落的直接矩阵/向量操作封装成统一 helper。
  - 为 `Transform` 明确“本地 TRS 权威”角色，不在本轮引入 Euler 编辑面。
  - 增补一份运行时数学规范文档，记录坐标系、矩阵顺序、SIMD 后端与“只经由 `zircon_math` 访问数学类型”的规则。
- Phase 2：`World` 数据模型升级
  - 在 `World` 内部加入 `active_self`、`active_in_hierarchy`、`render_layer_mask`、`mobility`、`world_matrices` 存储。
  - 保持层级缓存稳定顺序：父在前、子在后、同级按当前稳定顺序输出，便于未来 hierarchy inspector 直接复用。
  - 现有 selection/editor 过渡状态先保留，不在本轮继续把 editor 语义下沉进 world。
- Phase 3：层级传播与约束
  - 引入内部 dirty 传播，统一重建：
    - `WorldMatrix`
    - `ActiveInHierarchy`
    - 兼容节点缓存
  - 规则固定为：
    - 本地变换修改只写 `LocalTransform`
    - 世界矩阵按 `parent_world * local_matrix` 派生
    - `active_in_hierarchy = parent.active_in_hierarchy && active_self`
    - `RenderLayerMask` 不继承，只是本地渲染过滤标记
    - `Mobility::Static` 禁止常规运行时 `update_transform`/`set_parent_checked` 之类会改变结果世界变换的操作
    - `Static` 实体挂到 `Dynamic` 父节点下直接拒绝
  - bootstrap/import 路径提供专用插入流程，允许静态实体在加载阶段被初始化，但不放宽运行时约束。
- Phase 4：兼容层与编译门槛
  - 让现有 `SceneNode`/`NodeRecord`/`nodes()`/`find_node()` 继续可用，但底层读取新 runtime 模型。
  - 把 `zircon_graphics` 适配到当前嵌套的 `SceneViewportRenderPacket { scene, overlays, preview }`，不在本轮顺手扩展渲染功能，只做最小消费改造与测试修正。
  - `zircon_editor` 依赖兼容查询继续工作，避免首里程碑把下游长期打断。
- Phase 5：规范清理
  - 移除或降级旧的“世界 TRS 即权威”路径。
  - 更新架构文档，正式写清 `Entity / WorldMatrix / ActiveSelf+ActiveInHierarchy / RenderLayerMask / Mobility` 基础规范。

## Test Plan
- `zircon_math`
  - TRS 组合矩阵与输入平移/旋转/缩放一致。
  - 父子矩阵相乘得到正确世界位置与方向。
  - 仿射逆与点变换往返在 epsilon 内成立。
- `zircon_scene`
  - 新实体默认 `ActiveSelf(true)`、`ActiveInHierarchy(true)`、`RenderLayerMask(1)`、`Mobility::Dynamic`。
  - 父关闭后子 `ActiveInHierarchy(false)`，但子 `ActiveSelf` 保持原值。
  - 重新激活父节点能恢复子树有效激活。
  - reparent 后世界矩阵按稳定顺序重算。
  - `Static` 实体拒绝运行时变换修改与动态父节点挂接。
  - `RenderLayerMask` set/get/serialize/deserialize 正常。
  - 旧场景资源不带新字段时仍按默认值加载。
  - 兼容 `nodes()` / `find_node()` / `world_transform()` 仍满足 editor 当前调用。
- `zircon_graphics`
  - 现有 renderer/tests 改到 `scene/overlays/preview` 新 packet 结构后重新通过。
  - grid、fallback light、clear color 仍能输出非背景帧。
- 验收门槛
  - `cargo test -p zircon_math --locked`
  - `cargo test -p zircon_scene --locked`
  - `cargo test -p zircon_graphics --locked`
  - `cargo test -p zircon_editor --locked`
  - `cargo test --workspace --locked`

## Assumptions / Defaults
- 本轮不落 `MeshFilter/MeshRenderer` 新公共组件，只先把运行时基础打稳。
- 本轮不迁移 hierarchy inspector 与 RTS 交互，只保证未来可接入。
- `Transform` 名称继续保留为本地 TRS 类型，避免第一轮无意义命名震荡。
- `WorldMatrix` 是权威；任何世界 TRS 分解结果都只是兼容视图。
- `RenderLayerMask` 只代表渲染/查询过滤，不代表物理层或层级继承。
- `Mobility::Static` 在首版语义上定义为“运行时不可改本地/结构性变换 + 为后续优化预留契约”，不等同于本轮就完成静态批处理。
