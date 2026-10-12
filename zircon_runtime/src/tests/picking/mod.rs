//! 拾取回归共享固定相机与命中夹具，子模块沿射线、后端输出、悬停归约和指针事件链验证；这里的句柄编号只标识夹具。
//!
//! # 入口与职责
//!
//! 本模块只在启用 `native-host` 的运行时测试构建中挂载，具体用例由五个子模块拥有。
//! [`hit`]、[`pointer_location`] 与 [`test_camera`] 每次创建自有夹具值，
//! [`event_labels`] 则为事件顺序断言提供标签投影；夹具编号不会注册真实相机、视口或设备。
use crate::core::framework::picking::{
    hovered_hits_for_pointer, ray_from_viewport_point, run_picking_pipeline,
    sorted_hits_for_pointer, CameraRaySource, HitData, HitRecord, HitTarget, Pickable, PickingAxis,
    PickingBackend, PickingDebugFeed, PickingDebugMetricKind, PickingEventKind, PickingEventLabel,
    PickingEventState, PickingHoverMap, PickingPipelineInput, PickingPipelineReport,
    PickingPrimitive, PickingScheduleLabel, PickingSettings, PointerAction, PointerButton,
    PointerHits, PointerId, PointerInput, PointerLocation, PointerScrollUnit,
    PrimitivePickingBackend, RayId, RayMap,
};
use crate::core::framework::render::{
    ProjectionMode, RenderViewportHandle, ViewportCameraSnapshot,
};
use crate::core::math::{Transform, UVec2, Vec2, Vec3};

/// 拾取报告和调试指标回归，覆盖命中归约与仅有射线的指针行。
#[path = "diagnostics.rs"]
mod diagnostics;
/// 命中排序、悬停策略、原语后端与共享投影存储的回归。
#[path = "hits_and_hover.rs"]
mod hits_and_hover;
/// 完整拾取阶段报告及总开关禁用后清空交互状态的回归。
#[path = "pipeline.rs"]
mod pipeline;
/// 按帧输入推进悬停、按压、拖拽、滚动与取消的事件顺序回归。
#[path = "pointer_events.rs"]
mod pointer_events;
/// 相机射线、指针与视口准入，以及多指针和多相机射线键的回归。
#[path = "rays.rs"]
mod rays;

/// 创建排序、悬停、诊断和事件用例共用的最小命中夹具。
///
/// # 参数
///
/// - `target`：用例提供的目标标签，原样写入结果。
/// - `depth`：用例提供的排序深度键，不检查有限性、正负或物理单位。
///
/// # 返回
///
/// 返回命中记录：相机标签固定为 `99`，位置和法线均为 `None`，
/// [`Pickable`] 沿用默认的可悬停且阻挡更低层目标的策略；用例可再改写策略。
///
/// # 所有权与生命周期
///
/// 每次创建自有记录与命中数据，不查询、注册或保活目标实体和相机；编号只是夹具值。
///
/// # 参考
///
/// 字段构造见 [`HitRecord::new`] 与 [`HitData::new`]。
fn hit(target: HitTarget, depth: f32) -> HitRecord {
    HitRecord::new(target, HitData::new(99, depth, None, None))
}

/// 为事件与管线用例创建固定视口 `1` 内的指针位置夹具。
///
/// # 参数
///
/// - `pointer`：调用方提供的核心指针身份。
/// - `x`：视口局部像素 x 坐标。
/// - `y`：视口局部像素 y 坐标。
///
/// # 返回
///
/// 返回 [`PointerLocation`]，原样保存指针与坐标，并使用句柄值 `1`；不检查范围、有限性或视口存续。
///
/// # 所有权与生命周期
///
/// 结果拥有可复制的字段值，不分配指针编号、注册视口或保活设备；用例负责位置与输入的配对。
///
/// # 参考
///
/// 原样构造行为见 [`PointerLocation::new`] 和 [`RenderViewportHandle::new`]。
fn pointer_location(pointer: PointerId, x: f32, y: f32) -> PointerLocation {
    PointerLocation::new(pointer, RenderViewportHandle::new(1), Vec2::new(x, y))
}

/// 按事件原有顺序提取标签，供管线和指针事件用例比较派发次序。
///
/// # 参数
///
/// - `events`：借用的事件切片，可以为空；调用期间不修改其中的事件。
///
/// # 返回
///
/// 返回新建的 [`PickingEventLabel`] 向量，与输入逐项对应并保留重复项；空输入返回空向量。
/// 标签只表达事件种类，目标、传播和几何属性应由用例另行断言。
///
/// # 所有权与生命周期
///
/// 切片只在本次调用中借用，结果拥有可复制的标签，不保留对输入事件的引用。
///
/// # 参考
///
/// 单项投影见 [`crate::core::framework::picking::PickingPointerEvent::label`]。
fn event_labels(
    events: &[crate::core::framework::picking::PickingPointerEvent],
) -> Vec<PickingEventLabel> {
    events.iter().map(|event| event.label()).collect()
}

/// 为射线、后端、诊断和管线用例创建固定配置的相机值快照。
///
/// # 参数
///
/// - `projection_mode`：用例明确选择的投影模式，原样写入快照。
///
/// # 返回
///
/// 返回 [`ViewportCameraSnapshot`]：相机从 `(0, 0, 8)` 看向原点，向上为 Y，
/// 垂直视场角为 60 度的弧度值，正交尺寸为 `5`，近远裁剪面为 `0.1` 和 `200`，宽高比为 `16/9`。
/// 未覆盖字段沿用 [`ViewportCameraSnapshot::default`]。实际射线宽高比由 [`ray_from_viewport_point`] 的视口尺寸确定。
///
/// # 所有权与生命周期
///
/// 每次返回独立的自有快照，可由用例局部修改；函数不注册场景相机或共享可变资源。
///
/// # 参考
///
/// 朝向构造见 [`Transform::looking_at`]；射线相关断言由 [`rays`] 子模块拥有。
fn test_camera(projection_mode: ProjectionMode) -> ViewportCameraSnapshot {
    ViewportCameraSnapshot {
        transform: Transform::looking_at(Vec3::new(0.0, 0.0, 8.0), Vec3::ZERO, Vec3::Y),
        projection_mode,
        fov_y_radians: 60.0_f32.to_radians(),
        ortho_size: 5.0,
        z_near: 0.1,
        z_far: 200.0,
        aspect_ratio: 16.0 / 9.0,
        ..ViewportCameraSnapshot::default()
    }
}
