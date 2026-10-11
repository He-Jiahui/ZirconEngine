//! 定义带位置的核心指针输入，以及动作载荷和滚动单位标签。
//!
//! 输入源显式创建 [`PointerInput`] 并保留动作顺序；[`PointerAction`] 的效果由
//! [`super::PickingEventState::dispatch_frame`] 处理，这里的构造与查询只操作值。
use crate::core::math::Vec2;

use super::{PointerButton, PointerId, PointerLocation};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// 标注二维滚动增量采用行单位还是像素单位。
///
/// # 使用约定
///
/// 与 [`PointerAction::Scroll`] 的 `delta` 配对，并原样传入 [`super::PickingEventKind::Scroll`]。
/// 输入源与消费方约定轴向、符号、行高和像素缩放；此枚举只保存标签，不提供单位换算。
///
/// # 所有权与生命周期
///
/// 标签可按值复制，随所属输入或事件保存；它不持有输入设备或视口资源。
pub enum PointerScrollUnit {
    /// `delta` 以行单位解释；具体行高和换算比例由输入源与消费方约定。
    Line,
    /// `delta` 以像素单位解释；逻辑或物理像素的缩放约定由输入源与消费方确定。
    Pixel,
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// 与动作发生时位置一起提交的核心指针动作。
///
/// # 使用时机与约束
///
/// 作为 [`PointerInput`] 的载荷，由 [`super::PickingEventState::dispatch_frame`] 按调用方给定的顺序消费。
/// 输入源负责按钮配对、增量单位和位置一致性；类型可保存未检查有限性或范围的二维增量。
///
/// # 所有权与生命周期
///
/// 动作拥有内嵌值并可按值复制；构造动作不会派发事件，跨帧按压和拖拽状态由派发器保存。
pub enum PointerAction {
    /// 按下内嵌的 [`super::PointerButton`] 标签，对当前悬停目标建立该指针和按钮的按压状态。
    ///
    /// 标签是自有可复制的值，由输入源提供；后续释放须用相同标签配对。
    /// 派发器可跨帧保存按压位置和命中，直到释放、取消或状态清空。
    Press(PointerButton),
    /// 释放内嵌的 [`super::PointerButton`] 标签，对该指针的对应按钮状态完成收尾。
    ///
    /// 标签是自有可复制的值，须与按压时配对；派发器用上一帧悬停处理释放和点击，
    /// 并取出该按钮的按压与拖拽内容处理拖拽结束，状态容器中的按钮键仍可保留。
    Release(PointerButton),
    /// 移动动作，拥有输入源提供的二维 `delta`，其坐标约定须与动作位置相容。
    ///
    /// 派发器在 `delta` 恰为零向量时跳过此移动处理；否则把它原样传入移动事件，
    /// 并从按压位置和最新位置另行计算拖拽距离与增量。载荷不检查有限性或自动换算坐标。
    Move {
        /// 输入源提供的二维移动量，随本次动作按值保存。
        ///
        /// # 使用约束与生命周期
        ///
        /// 坐标约定须与动作位置相容；类型不检查有限性或执行坐标换算。
        /// 派发器遇到零向量时跳过该移动处理，否则原样复制到移动事件；
        /// 拖拽距离与拖拽增量另从位置差计算。字段不借用或保活输入设备。
        ///
        /// # 参考
        ///
        /// 消费与零增量分支见 [`super::PickingEventState::dispatch_frame`]。
        delta: Vec2,
    },
    /// 滚动动作，拥有单位标签 `unit` 和按该单位解释的二维增量 `delta`。
    ///
    /// `unit` 使用 [`PointerScrollUnit`]；输入源与消费方约定 `delta` 的轴向、符号和比例。
    /// 派发器向当前悬停目标生成 [`super::PickingEventKind::Scroll`]，原样保留两个载荷，
    /// 不换算单位、过滤零值或检查有限性；该动作不更新按钮按压与拖拽状态。
    Scroll {
        /// 与 `delta` 配对的 [`PointerScrollUnit`] 标签，派发时按值原样保存。
        unit: PointerScrollUnit,
        /// 输入源提供的二维滚动量，与同一动作的 `unit` 标签配对。
        ///
        /// 派发到当前悬停目标时按值原样保存；字段不借用输入设备。
        /// 单位缩放、轴向和符号由生产方与消费方约定，类型及滚动分支不清洗非有限值。
        delta: Vec2,
    },
    /// 取消该指针的交互，向上一帧悬停目标派发取消并清除其悬停与全部按钮状态。
    ///
    /// [`super::PickingEventState::dispatch_frame`] 先从本帧悬停中剔除所有出现取消的指针，
    /// 再按输入顺序处理动作；首次取消后，同一指针的后续输入不再派发。
    /// 取消前的输入仍按已过滤的悬停处理，后续输入位置也仍参与预先构建的位置索引。
    Cancel,
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// 单次输入动作与当时的视口局部位置绑定；同帧输入顺序会影响按下、释放与拖拽状态。
///
/// # 使用时机与约束
///
/// 输入源配对动作与位置后，按发生顺序提交给 [`super::PickingEventState::dispatch_frame`]。
/// 公开字段和 [`Self::new`] 均不校验位置、按钮、单位或顺序；同指针最后一条输入的
/// [`Self::location`] 覆盖该帧采样位置，供悬停边界事件使用。取消的整帧影响见 [`PointerAction::Cancel`]。
///
/// # 所有权与生命周期
///
/// 实例拥有可复制的位置和动作值，不借用或保活设备、视口，也不提供自动采样更新。
/// 派发器可保存其中的按压位置供后续帧使用，调用方须保持历史与当前坐标约定相容。
pub struct PointerInput {
    /// 动作发生时的 [`PointerLocation`] 值，拥有指针身份、视口句柄和局部像素位置。
    ///
    /// [`super::PickingEventState::dispatch_frame`] 用本条位置处理动作；同指针最后一条输入位置
    /// 还会覆盖采样位置，供本帧悬停边界事件使用。字段不检查位置配对或时效。
    pub location: PointerLocation,
    /// 在 [`Self::location`] 发生的 [`PointerAction`] 自有值，其载荷由输入源明确提供。
    ///
    /// 创建记录只保存动作；事件效果和跨帧状态由消费该记录的派发器处理。
    pub action: PointerAction,
}

impl PointerInput {
    /// 原样组合动作发生时的位置和动作载荷，创建一条核心指针输入记录。
    ///
    /// # 参数
    ///
    /// - `location`：动作所属的指针、视口和局部像素位置。
    /// - `action`：输入源提供的动作，包含相应按钮或增量载荷。
    ///
    /// # 返回
    ///
    /// 返回拥有两个参数值的记录；不校验位置、按钮配对、滚动单位、有限性或同帧顺序。
    ///
    /// # 所有权与生命周期
    ///
    /// 可在常量上下文中构造，调用只创建可复制的值，不借用或保活设备和视口。
    /// 调用方保留输入顺序并显式提交记录；函数不入队、派发或更新跨帧状态。
    ///
    /// # 参考
    ///
    /// 帧输入入口见 [`super::PickingPipelineInput`]；动作消费见 [`super::PickingEventState::dispatch_frame`]。
    pub const fn new(location: PointerLocation, action: PointerAction) -> Self {
        Self { location, action }
    }

    /// 读取输入位置中保存的指针身份，供位置索引和取消筛选使用。
    ///
    /// # 参数
    ///
    /// - `self`：按值读取的输入记录；动作种类和视口不参与此查询。
    ///
    /// # 返回
    ///
    /// 返回 [`Self::location`] 中的 [`PointerLocation::pointer`] 副本；不分配身份或验证其唯一性和存续。
    ///
    /// # 所有权与生命周期
    ///
    /// 可在常量上下文中调用，无外部状态修改；记录和身份均可复制，调用方可继续使用原记录。
    ///
    /// # 参考
    ///
    /// [`super::PickingEventState::dispatch_frame`] 用该身份索引位置，并筛选同指针的取消与后续输入。
    pub const fn pointer(self) -> PointerId {
        self.location.pointer
    }
}
