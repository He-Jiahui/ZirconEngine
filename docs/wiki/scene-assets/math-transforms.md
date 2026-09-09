---
related_code:
  - zircon_runtime/src/core/math/mod.rs
  - zircon_runtime/src/scene/components/scene/transform.rs
  - zircon_runtime/src/scene/world/transform_validation.rs
implementation_files:
  - zircon_runtime/src/core/math/mod.rs
  - zircon_runtime/src/scene/world/transform_validation.rs
plan_sources:
  - user: 2026-09-09 数学与变换说明
tests:
  - zircon_runtime/src/asset/tests/assets/scene/foundation.rs
  - zircon_runtime/src/scene/world
doc_type: module-detail
---

# 数学、坐标约定与变换

## 统一类型

`core::math` 从 `zr_math` 导出 `Real`、`Vec2/3/4`、`Quat`、`Mat4`、`Transform`，并导出 `compose_trs`、`transform_to_mat4`、`view_matrix`、`perspective`、`try_affine_inverse` 等函数。渲染边界使用 `RenderScalar`/`RenderMat4` 和 `to_render_*` 窄化函数，避免把逻辑精度和 GPU 精度混用。

坐标与单位由 `CoordinateSchema`、`CoordinateHandedness`、`FrontFaceWinding`、`DepthDirection`、`ClipDepthRange`、`LengthUnit` 描述；仓库默认常量为 `ZIRCON_COORDINATE_SCHEMA`、`ZIRCON_UNIT_SCHEMA`、`ZIRCON_PRECISION_PROFILE`。

## 场景变换流程

`LocalTransform` 保存 authoring TRS；层级派生阶段按 parent 递归组合成 `WorldTransform`/`WorldMatrix`。写入前 `transform_validation::validate_transform_for_write` 检查有限数值，静态节点还会由 mobility 规则拒绝变更。相机视图使用 `view_matrix`，投影使用 `perspective`/`try_perspective`；求逆应使用返回 `AffineInverseError` 的 `try_affine_inverse`。

```rust
use zircon_runtime::core::math::{compose_trs, Quat, Transform, Vec3};
let local = Transform { translation: Vec3::new(0.0, 1.0, 0.0),
    rotation: Quat::IDENTITY, scale: Vec3::ONE };
let matrix = compose_trs(local.translation, local.rotation, local.scale);
world.update_transform(entity, local)?;
```

## 错误与限制/状态

NaN/Infinity、不可逆矩阵和非法透视参数分别返回 `NumericError`、`AffineInverseError`、`PerspectiveError`；`NumericPolicy` 提供阈值配置。渲染窄化可能返回 `RenderNarrowingError`，调用方应处理而非静默截断。基础向量、矩阵、四元数、TRS、坐标约定和验证 API 已实现。

## 源码与测试

[core/math/mod.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/core/math/mod.rs)、[transform.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/scene/components/scene/transform.rs)、[transform_validation.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/scene/world/transform_validation.rs)；场景基础测试见 [foundation.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/asset/tests/assets/scene/foundation.rs)。
