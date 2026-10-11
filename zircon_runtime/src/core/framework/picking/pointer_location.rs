//! 保存核心拾取使用的指针身份、渲染视口和局部像素位置。
//!
//! [`PointerLocation`] 由调用方提供；[`super::RayMap::rebuild`] 按相机视口配对后检查位置，
//! [`super::PointerInput`] 则保存动作发生时的位置。这里提供按值数据和范围判断。
use crate::core::framework::render::RenderViewportHandle;
use crate::core::math::{UVec2, Vec2};

use super::PointerId;

#[derive(Clone, Copy, Debug, PartialEq)]
/// 指针在某个渲染视口内的位置；射线构建要求 position 使用该视口的局部像素坐标。
///
/// # 使用时机与约束
///
/// 作为每帧指针采样或动作发生时的位置传入拾取流程。公开字段与 [`Self::new`] 均原样保存值，
/// 调用方负责身份、视口和坐标配对；构造时不验证句柄是否存续、坐标有限性或范围。
/// [`Self::is_inside_viewport`] 只判断给定尺寸下的范围；[`super::RayMap::rebuild`] 另行配对视口并构建射线。
///
/// # 所有权与生命周期
///
/// 实例拥有三个字段值，可以按值复制；句柄不会保活渲染视口，位置也不会自动刷新。
/// [`super::PickingEventState::dispatch_frame`] 可保存按压位置供后续帧计算拖动，使用历史值时应核对时效与坐标空间。
pub struct PointerLocation {
    /// 该位置所属的核心指针身份，供射线键和事件位置索引使用。
    ///
    /// 保存调用方提供的 [`super::PointerId`] 值，不分配编号或保活输入设备。
    pub pointer: PointerId,
    /// [`Self::position`] 采用局部像素坐标的渲染视口句柄。
    ///
    /// [`super::RayMap::rebuild`] 用它匹配相机视口；单独调用 [`Self::is_inside_viewport`] 时须自行传入对应尺寸。
    /// 这里只保存句柄值，不查询或保活视口。
    pub viewport: RenderViewportHandle,
    /// [`Self::viewport`] 内的像素位置；射线换算以左上角为原点，x 向右、y 向下。
    ///
    /// [`super::ray_from_viewport_point`] 按视口尺寸换算坐标，事件状态也用此值计算位置差。
    /// 字段可保存越界、NaN 或无穷值；生产方负责使用前所需的有效性检查。
    pub position: Vec2,
}

impl PointerLocation {
    /// 原样组合指针身份、视口句柄和局部像素位置，供采样或输入动作使用。
    ///
    /// # 参数
    ///
    /// - `pointer`：调用方提供的核心指针身份。
    /// - `viewport`：位置所属的渲染视口句柄。
    /// - `position`：该视口的局部像素位置，x 向右、y 向下。
    ///
    /// # 返回
    ///
    /// 返回拥有三个参数值的位置记录；不验证身份、视口存续、坐标配对、有限性或边界。
    ///
    /// # 所有权与生命周期
    ///
    /// 同步创建可复制的值，不借用或保活输入设备与视口，也不会自动更新位置。
    /// 调用方负责提供相容的坐标与视口，并在消费时判断采样是否仍适用。
    ///
    /// # 参考
    ///
    /// 范围判断见 [`Self::is_inside_viewport`]；视口配对及射线构建见 [`super::RayMap::rebuild`]。
    pub fn new(pointer: PointerId, viewport: RenderViewportHandle, position: Vec2) -> Self {
        Self {
            pointer,
            viewport,
            position,
        }
    }

    /// 判断位置是否在给定视口尺寸的闭区间范围内，供射线构建前筛选坐标。
    ///
    /// # 参数
    ///
    /// - `self`：按值读取其中的位置；本方法不核对指针身份或视口句柄。
    /// - `viewport_size`：与 [`Self::viewport`] 对应的像素尺寸；两维转成 `f32` 后作为上界，较大整数可能舍入。
    ///
    /// # 返回
    ///
    /// 两个分量分别在 `[0, width as f32]`、`[0, height as f32]` 内时返回 `true`，包含四条边界。
    /// NaN、正负无穷或越界坐标返回 `false`；负零可通过比较。零维允许该维坐标为零，零尺寸下原点也可返回 `true`。
    /// 结果不保证视口非零、句柄存续或相机和射线有效。
    ///
    /// # 所有权与生命周期
    ///
    /// 只读取自有字段，无资源查询或状态修改；位置是可复制的值，调用方可继续使用原副本。
    ///
    /// # 参考
    ///
    /// [`super::RayMap::rebuild`] 先匹配视口，再调用本方法；[`super::ray_from_viewport_point`] 另行检查零尺寸和射线构建条件。
    pub fn is_inside_viewport(self, viewport_size: UVec2) -> bool {
        let width = viewport_size.x as f32;
        let height = viewport_size.y as f32;
        self.position.x >= 0.0
            && self.position.y >= 0.0
            && self.position.x <= width
            && self.position.y <= height
    }
}
