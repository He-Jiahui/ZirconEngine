//! 为单帧拾取管线报告定义五个阶段标签。
//!
//! [`PickingScheduleLabel`] 按值保存于 [`super::PickingPipelineStageReport`]，
//! 与阶段启用标记和计数一起解释。标签本身不执行调度或维护交互状态；
//! 实际调用次序和禁用行为由 [`super::run_picking_pipeline`] 的控制流决定。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// 标识拾取管线报告中的输入、射线、后端、悬停或事件阶段。
///
/// # 报告边界
/// 同一标签也可出现在禁用报告中；是否启用与计数应读取对应
/// [`super::PickingPipelineStageReport`]。计数的单位随阶段变化，标签不能单独证明已执行查询。
///
/// # 值与执行顺序
/// 枚举是可复制、可比较的无状态值。派生的 `Ord` 按变体声明次序比较，
/// 实际执行次序由 [`super::run_picking_pipeline`] 显式构造阶段记录的顺序决定。
///
/// # 参考
/// 参见 [`super::PickingPipelineStageReport`] 与 [`super::run_picking_pipeline`]。
pub enum PickingScheduleLabel {
    /// 输入接收阶段的报告标签。
    ///
    /// 管线开启时，输入计数为位置样本数与输入动作数之和，输出计数为位置样本数；
    /// 总开关关闭时仍记录接收数量，但阶段禁用且输出为零。
    /// 具体记录见 [`super::PickingPipelineStageReport`] 与 [`super::run_picking_pipeline`]。
    Input,
    /// 从位置样本与相机输入构建本帧 [`super::RayMap`] 的阶段标签。
    ///
    /// 管线开启时，输入计数为位置数乘相机数，包含尚未过滤的组合；输出计数为实际射线数。
    /// 射线开关关闭时阶段标记禁用、射线输出为零，后续悬停与事件仍可能运行。
    /// 阶段记录本身不保留上帧射线。
    RayMap,
    /// 后端收集命中批次的阶段标签。
    ///
    /// 阶段启用时按输入后端次序调用 [`super::PickingBackend::collect_hits`]，
    /// 输入计数为后端数量，输出计数为返回的 [`super::PointerHits`] 批次数。
    /// 每个批次可包含多个命中，此阶段的输出计数不展开批次内容。
    /// 射线开关关闭时不调用后端；射线集合为空但开关开启时仍会调用。
    Backend,
    /// 合并排序并按遮挡与可悬停策略生成 [`super::PickingHoverMap`] 的阶段标签。
    ///
    /// 输入计数是所有后端批次的原始命中总数，输出计数是筛选后的悬停命中总数。
    /// 总开关开启时此阶段继续执行，射线与后端被禁用时也会解析空输出。
    Hover,
    /// 用悬停快照和本帧输入推进 [`super::PickingEventState`] 的阶段标签。
    ///
    /// 输入计数为动作数加当前悬停命中数，输出计数为产生的指针事件数；
    /// 事件还可来自先前帧状态，因此两个计数不是一一对应关系。
    /// 总开关开启且射线阶段关闭时仍派发事件，具体行为见 [`super::PickingEventState::dispatch_frame`]。
    Events,
}
