//! 定义单帧拾取管线的总开关与射线、后端阶段开关。
//!
//! 配置通过 [`super::PickingPipelineInput`] 按值传给 [`super::run_picking_pipeline`]。
//! 设置值本身不修改交互状态；实际清理或推进由运行管线完成。
//! 总开关关闭会清空事件状态；仅关闭射线开关仍处理空命中的悬停与事件。
//! [`PickingSettings::DEFAULT`] 提供两个开关均开启的默认值。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 控制整条选取管线与射线阶段；关闭总开关会清空事件状态，关闭射线阶段则不调用后端。
///
/// 按值存入 [`super::PickingPipelineInput`] 的运行配置，可复制并用于后续调用。
///
/// # 开关关系
/// [`Self::enabled`] 关闭时，管线先清空 [`super::PickingEventState`]，并返回空交互结果；
/// [`Self::ray_map_enabled`] 在总开关开启时只控制射线重建和后端收集，
/// 悬停解析及事件状态仍继续推进，可能产生基于先前帧状态的事件。
///
/// # 生效时机与生命周期
/// 构造或修改配置不立即清理状态；[`super::run_picking_pipeline`] 读取该次输入时才执行分支。
/// 配置只包含布尔值，不借用或拥有射线、后端及事件资源；它不取消具体后端此前启动的工作。
///
/// # 参考
/// 参见 [`Self::DEFAULT`]、[`super::PickingPipelineInput::with_settings`] 与 [`super::run_picking_pipeline`]。
pub struct PickingSettings {
    /// 是否执行本次完整拾取管线调用。
    ///
    /// `false` 在运行时立即清空已有悬停和按键状态，返回空射线、命中、悬停与事件结果，
    /// 五个阶段记录均标记禁用。清理不会派发离开或取消事件。
    /// 再次开启后从已清空的交互状态开始；具体行为见 [`super::run_picking_pipeline`]。
    pub enabled: bool,
    /// 总开关开启时，是否重建本帧射线并调用后端收集。
    ///
    /// `false` 跳过这两步，使本帧 [`super::RayMap`] 与后端输出为空；
    /// 悬停解析和 [`super::PickingEventState::dispatch_frame`] 仍运行，并可使用旧帧状态派发退出或释放事件。
    /// 此开关不等同于清空全部按键状态，也不取消后端已有工作；总开关关闭时由其分支优先处理。
    pub ray_map_enabled: bool,
}

impl Default for PickingSettings {
    /// 生成两个拾取开关均开启的默认配置值。
    ///
    /// # 返回
    /// 返回与 [`Self::DEFAULT`] 相同的 [`Self`]，总管线和射线、后端收集均启用。
    /// 返回值可独立复制或修改，不与先前生成的配置共享可变状态。
    ///
    /// # 生效时机与生命周期
    /// 本方法只返回配置值，不运行拾取或清理交互状态，也不持有后端和事件资源。
    /// 将配置传入 [`super::PickingPipelineInput::with_settings`] 后，
    /// 下一次 [`super::run_picking_pipeline`] 才按该值选择执行分支。
    ///
    /// # 参考
    /// 参见 [`Self::DEFAULT`] 与 [`super::PickingPipelineInput::new`]。
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl PickingSettings {
    /// 默认的全开配置：总管线与射线、后端收集均启用。
    ///
    /// [`Default::default`] 返回同一组值，[`super::PickingPipelineInput::new`] 也按值采用此常量。
    /// 可复制后覆盖 [`Self::enabled`] 或 [`Self::ray_map_enabled`]；实际效果在下一次运行管线时生效。
    pub const DEFAULT: Self = Self {
        enabled: true,
        ray_map_enabled: true,
    };
}
