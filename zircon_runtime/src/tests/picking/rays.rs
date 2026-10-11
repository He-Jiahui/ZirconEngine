//! 射线构造先验证指针、视口与相机准入；多指针和多相机映射只为有效组合产生射线。
//!
//! 本模块的五个原生测试在父测试条件成立时运行，复用 [`super::test_camera`] 的透视夹具，
//! 分别核对中心方向、视口与活动源准入、射线键组合和实际视口宽高比。
//! [`RayMap::rebuild`] 的键隔离与 [`ray_from_viewport_point`] 的数值构造分开断言；
//! 本模块未覆盖后端命中合并或事件路由，编号只是局部夹具标签。
use super::*;

#[test]
/// 验证透视相机在视口中心生成从相机位置沿前向的射线。
///
/// # 测试场景
///
/// 无参数；使用 [`super::test_camera`]，视口为 `1280×720`，指针位置为 `(640, 360)`。
///
/// # 返回
///
/// 射线存在、原点等于相机平移且方向与前向在 `0.0001` 容差内一致时返回 `()`；构造或断言失败会 panic。
///
/// # 约束与生命周期
///
/// 只核对此固定透视配置的中心点。相机和射线为本次用例的自有局部值，相机只在构建期间借用。
///
/// # 参考
///
/// 构造见 [`ray_from_viewport_point`]，前向定义见 [`crate::core::math::Transform::forward`]。
fn perspective_pointer_location_builds_center_camera_ray() {
    let camera = test_camera(ProjectionMode::Perspective);
    let viewport = UVec2::new(1280, 720);

    let ray = ray_from_viewport_point(&camera, viewport, Vec2::new(640.0, 360.0))
        .expect("center pointer should produce a camera ray");

    assert_eq!(ray.origin, camera.transform.translation);
    assert!(ray
        .direction
        .abs_diff_eq(camera.transform.forward(), 0.0001));
}

#[test]
/// 验证射线重建同时按指针视口配对和相机源活动标志筛选组合。
///
/// # 测试场景
///
/// 无参数；指针 `7` 属于视口 `2`，指针 `8` 属于视口 `3`；相机 `11`、`12` 均属于视口 `2`，后者停用。
///
/// # 返回
///
/// 仅存在相机 `11`、指针 `7`、视口 `2` 的键，且射线总数为一时返回 `()`；断言失败会 panic。
///
/// # 约束与生命周期
///
/// 两个位置均在固定 `640×360` 范围内。本例核对相机源的 `active` 标志，未验证快照的活动字段或异常坐标。
/// 快照、位置数组和射线图为局部值，数组只在 [`RayMap::rebuild`] 调用中借用。
///
/// # 参考
///
/// 停用来源见 [`CameraRaySource::inactive`]；具体键由 [`RayId::new`] 构造。
fn ray_map_respects_pointer_viewport_and_camera_activity() {
    let pointer = PointerId::new(7);
    let viewport = RenderViewportHandle::new(2);
    let other_viewport = RenderViewportHandle::new(3);
    let mut ray_map = RayMap::default();

    ray_map.rebuild(
        &[
            PointerLocation::new(pointer, viewport, Vec2::new(320.0, 180.0)),
            PointerLocation::new(PointerId::new(8), other_viewport, Vec2::new(320.0, 180.0)),
        ],
        &[
            CameraRaySource::new(
                11,
                viewport,
                UVec2::new(640, 360),
                test_camera(ProjectionMode::Perspective),
            ),
            CameraRaySource::new(
                12,
                viewport,
                UVec2::new(640, 360),
                test_camera(ProjectionMode::Perspective),
            )
            .inactive(),
        ],
    );

    assert_eq!(ray_map.len(), 1);
    assert!(ray_map.get(&RayId::new(11, pointer, viewport)).is_some());
    assert!(ray_map.get(&RayId::new(12, pointer, viewport)).is_none());
    assert!(ray_map
        .get(&RayId::new(11, PointerId::new(8), other_viewport))
        .is_none());
}

#[test]
/// 验证同一视口的两个指针与两个活动相机分别形成四个射线键。
///
/// # 测试场景
///
/// 无参数；两个指针在视口 `1` 的 `(25, 25)` 和 `(75, 75)`，两个透视相机使用 `100×100` 尺寸。
///
/// # 返回
///
/// 四个相机、指针和视口组合均可查询且总数为四时返回 `()`；断言失败会 panic。
///
/// # 约束与生命周期
///
/// 本例核对键的数量与存在性，不依赖图的遍历顺序，也不逐项比较射线方向。
/// 位置、快照和射线图由用例局部拥有，重建完成后无需保留输入数组借用。
///
/// # 参考
///
/// 组合构建见 [`RayMap::rebuild`]；键与查询见 [`RayId::new`]、[`RayMap::get`]。
fn ray_map_builds_rays_for_two_pointers_and_two_active_cameras() {
    let first_pointer = PointerId::new(1);
    let second_pointer = PointerId::new(2);
    let viewport = RenderViewportHandle::new(1);
    let mut ray_map = RayMap::default();

    ray_map.rebuild(
        &[
            PointerLocation::new(first_pointer, viewport, Vec2::new(25.0, 25.0)),
            PointerLocation::new(second_pointer, viewport, Vec2::new(75.0, 75.0)),
        ],
        &[
            CameraRaySource::new(
                11,
                viewport,
                UVec2::new(100, 100),
                test_camera(ProjectionMode::Perspective),
            ),
            CameraRaySource::new(
                12,
                viewport,
                UVec2::new(100, 100),
                test_camera(ProjectionMode::Perspective),
            ),
        ],
    );

    assert_eq!(ray_map.len(), 4);
    assert!(ray_map
        .get(&RayId::new(11, first_pointer, viewport))
        .is_some());
    assert!(ray_map
        .get(&RayId::new(12, first_pointer, viewport))
        .is_some());
    assert!(ray_map
        .get(&RayId::new(11, second_pointer, viewport))
        .is_some());
    assert!(ray_map
        .get(&RayId::new(12, second_pointer, viewport))
        .is_some());
}

#[test]
/// 验证同一指针在两个视口中的位置分别只与对应视口相机形成射线键。
///
/// # 测试场景
///
/// 无参数；指针 `1` 在视口 `1`、`2` 各有一个位置，相机 `11`、`12` 分别对应两视口。
///
/// # 返回
///
/// 两个正确配对的键存在、交换视口的键均不存在且总数为二时返回 `()`；断言失败会 panic。
///
/// # 约束与生命周期
///
/// 只核对 [`RayMap::rebuild`] 产生的射线键；本用例没有收集后端命中或推进事件，不能据此推断下游视口路由。
/// 所有身份、位置、快照和射线图都是局部夹具值，临时输入只在重建期间借用。
///
/// # 参考
///
/// 三维键见 [`RayId::new`]；存在性查询见 [`RayMap::get`]。
fn ray_map_keeps_same_pointer_locations_scoped_by_viewport() {
    let pointer = PointerId::new(1);
    let first_viewport = RenderViewportHandle::new(1);
    let second_viewport = RenderViewportHandle::new(2);
    let mut ray_map = RayMap::default();

    ray_map.rebuild(
        &[
            PointerLocation::new(pointer, first_viewport, Vec2::new(25.0, 25.0)),
            PointerLocation::new(pointer, second_viewport, Vec2::new(75.0, 75.0)),
        ],
        &[
            CameraRaySource::new(
                11,
                first_viewport,
                UVec2::new(100, 100),
                test_camera(ProjectionMode::Perspective),
            ),
            CameraRaySource::new(
                12,
                second_viewport,
                UVec2::new(100, 100),
                test_camera(ProjectionMode::Perspective),
            ),
        ],
    );

    // 这里只验证带视口的射线键；后端命中合并与事件路由仍需各自验证。
    assert_eq!(ray_map.len(), 2);
    assert!(ray_map
        .get(&RayId::new(11, pointer, first_viewport))
        .is_some());
    assert!(ray_map
        .get(&RayId::new(12, pointer, second_viewport))
        .is_some());
    assert!(ray_map
        .get(&RayId::new(11, pointer, second_viewport))
        .is_none());
    assert!(ray_map
        .get(&RayId::new(12, pointer, first_viewport))
        .is_none());
}

#[test]
/// 验证离中心射线的水平视场采用实际视口宽高比。
///
/// # 测试场景
///
/// 无参数；[`super::test_camera`] 的快照宽高比设为 `16/9`，实际视口为 `100×100`，点位于右边界 `(100, 50)`。
///
/// # 返回
///
/// 射线存在且方向与 `normalize(tan(fov_y/2), 0, -1)` 在 `0.0001` 容差内一致时返回 `()`；失败会 panic。
///
/// # 约束与生命周期
///
/// 方形视口的实际宽高比为一；夹具相机轴对齐看向 -Z，期望方向无需另加旋转。
/// 此固定右边界用例未覆盖 NaN、零尺寸或其他投影模式；快照改写和数值结果只在本次调用中拥有。
///
/// # 参考
///
/// NDC 与宽高比换算见 [`ray_from_viewport_point`]。
fn ray_generation_uses_actual_viewport_aspect_for_off_center_pointers() {
    let mut camera = test_camera(ProjectionMode::Perspective);
    camera.aspect_ratio = 16.0 / 9.0;
    let square_viewport = UVec2::new(100, 100);

    let ray = ray_from_viewport_point(&camera, square_viewport, Vec2::new(100.0, 50.0))
        .expect("right-edge pointer should produce a camera ray");

    let half_fov_tan = (camera.fov_y_radians * 0.5).tan();
    // 实际方形视口的宽高比为一，轴对齐夹具使此方向可直接与输出比较。
    let expected = Vec3::new(half_fov_tan, 0.0, -1.0).normalize();
    assert!(ray.direction.abs_diff_eq(expected, 0.0001));
}
