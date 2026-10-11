//! 把命名动作求值器接入输入动作服务，并以外层互斥锁串行化配置读取、替换和求值。
//!
//! # 入口与责任
//! [`DefaultInputActionManager`] 实现 [`InputActionManager`]；模块工厂按启动配置构造并包装服务，单独创建此类型不会注册服务。
//! 本模块的求值入口借助已锁定求值器的独占可变借用取得工作区，帧路径复用工作区存储。
//!
//! # 约束与生命周期
//! 输入帧和消费切片只在调用期间借用；返回的映射与 [`InputActionState`] 拥有各自的数据，可在释放锁后继续使用。
//! 中毒时取回 guard 使访问继续，但不清除中毒标志，也不校验或修复求值器状态。
//!
//! # 参考
//! 参见 [`DefaultInputActionManager::new`]、[`InputActionManager::set_action_map`] 与 [`InputActionEvaluator`]。
use std::sync::{Mutex, MutexGuard};

use crate::core::framework::input::InputActionManager;
use crate::input::{
    GamepadAxisInput, InputActionMap, InputActionState, InputButton, InputFrameSnapshot,
};

use super::InputActionEvaluator;

/// 注册表中的动作服务适配层；外层锁串行化配置替换与求值，并借用同一工作区避免重复加锁。
/// 持有一份动作映射、编译代际和可复用工作区的默认动作服务。
///
/// # 使用时机与约束
/// 模块工厂可把它放入共享服务包装；同一实例的配置访问和求值经外层互斥锁串行化。
/// 求值器内层工作区由独占可变借用取得，不重复获取该工作区的锁；外层中毒恢复不保证状态一致性。
///
/// # 生命周期
/// 派生的 `Default` 通过求值器默认实现创建空动作映射与空工作区；它不安装配置或注册服务。
/// 管理器拥有内部数据；每次返回的映射和动作状态独立拥有数据，后续替换不会改写已返回值。
///
/// # 参考
/// 参见 [`Self::new`]、[`InputActionManager`] 与 [`InputActionEvaluator`]。
#[derive(Debug, Default)]
pub struct DefaultInputActionManager {
    /// 由外层互斥锁保护的完整动作求值器，随管理器拥有配置、代际和工作区。
    ///
    /// # 访问约束与生命周期
    /// 配置读取、替换和求值共用这把锁；guard 存续期间其它服务调用可能等待。
    /// 求值时从已锁定求值器取得独占可变借用，再直接借用内层工作区；外层锁和工作区锁是不同对象。
    /// 中毒时取回 guard 不清除中毒状态，也不执行一致性检查。
    ///
    /// # 参考
    /// 参见 [`InputActionEvaluator`] 与 [`InputActionManager`]。
    evaluator: Mutex<InputActionEvaluator>,
}

/// 提供拥有配置的构造入口，并集中处理外层锁的获取。
///
/// # 使用约束与生命周期
/// 构造过程先建立求值器；服务操作由 [`InputActionManager`] 实现块提供。
/// 私有锁助手返回借用管理器的 guard，调用方负责把持锁范围限定在一次访问内。
///
/// # 参考
/// 参见 [`Self::new`] 与 [`InputActionEvaluator`]。
impl DefaultInputActionManager {
    /// 按给定动作映射创建一份独立的默认动作服务。
    ///
    /// # 参数
    /// `action_map` 按值移入求值器，提供上下文、动作和绑定；构造会为它编译查找代际。
    /// 管理器不验证或清理直接构造的配置，也不替调用方过滤空绑定。
    ///
    /// # 返回
    /// 返回拥有求值器及外层互斥锁的 [`Self`]；工作区初始为空，构造本身不注册服务。
    ///
    /// # 使用时机与生命周期
    /// 通常在模块服务工厂构造时调用；也可独立创建后通过 trait 使用。
    /// 返回值不借用原映射；后续配置变更须显式调用 [`InputActionManager::set_action_map`]。
    ///
    /// # 参考
    /// 参见 [`InputActionMap`] 与 [`InputActionEvaluator::new`]。
    pub fn new(action_map: InputActionMap) -> Self {
        Self {
            evaluator: Mutex::new(InputActionEvaluator::new(action_map)),
        }
    }

    /// 取得本管理器外层互斥锁的求值器 guard。
    ///
    /// # 参数
    /// `self` 提供要访问的管理器；已有 guard 持锁时不应再次获取同一锁，获取可能等待其它调用释放锁。
    ///
    /// # 返回
    /// 返回借用 `self` 的 `MutexGuard<'_, InputActionEvaluator>`。
    /// 正常取锁直接返回 guard；中毒时从错误中取回 guard，不把中毒错误向上返回。
    /// 此恢复不清除中毒标志，也不校验或修复求值器状态。
    ///
    /// # 生命周期
    /// guard 释放时解除外层锁；调用方在其范围内读取或可变借用求值器。
    /// 本助手不获取求值器内部的工作区锁。
    ///
    /// # 参考
    /// 参见 [`DefaultInputActionManager`]、[`InputActionManager`] 与 [`InputActionEvaluator`]。
    fn lock_evaluator(&self) -> MutexGuard<'_, InputActionEvaluator> {
        self.evaluator
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// 以同一外层互斥锁实现配置访问和每帧动作求值。
///
/// # 使用约束
/// 每个入口在操作完成前持有求值器 guard；求值走已持锁管理器的专用入口，以独占借用取得工作区。
/// 活动上下文和已消费输入由调用方逐次提供；省略的维度传入空切片，空活动列表表示全部启用上下文。
///
/// # 生命周期
/// 映射读取返回克隆，求值返回拥有数据的状态；两者可在 guard 释放后继续使用。
/// 中毒时继续取回 guard，不构成状态一致性验证。
///
/// # 参考
/// 参见 [`InputActionManager`] 与 [`InputActionEvaluator`]。
impl InputActionManager for DefaultInputActionManager {
    /// 读取当前服务配置并返回一份拥有所有权的映射副本。
    ///
    /// # 参数
    /// `self` 提供要读取的管理器；此调用取得外层锁，可能等待正在进行的替换或求值。
    ///
    /// # 返回
    /// 返回当前 [`InputActionMap`] 的完整克隆；不会返回锁内映射的借用或 guard。
    ///
    /// # 生命周期
    /// 克隆在持锁期间完成，返回前释放 guard；编辑返回副本不会改变服务。
    /// 安装修改须另行调用 [`InputActionManager::set_action_map`]；后续替换不会改写已返回副本。
    /// 取锁沿用管理器的中毒恢复规则，不另做状态检查。
    ///
    /// # 参考
    /// 参见 [`InputActionManager`] 与 [`InputActionEvaluator`]。
    fn action_map(&self) -> InputActionMap {
        self.lock_evaluator().action_map().clone()
    }

    /// 在持有外层锁时替换动作配置及其求值代际。
    ///
    /// # 参数
    /// `action_map` 按值移入求值器；先为新表编译代际，再替换表与代际并重置工作区。
    /// 此入口不验证配置，也不额外过滤已有空绑定。
    ///
    /// # 返回
    /// 返回 `()`；正常完成后，后续服务调用使用新映射与新代际。
    ///
    /// # 使用时机与生命周期
    /// 用于启动后重绑定或替换配置，与同一管理器的读取和求值串行化。
    /// 重置清空工作区逻辑内容并保留可复用容量；已返回的映射副本和动作状态保持各自的数据。
    /// 外层和工作区的中毒恢复均不执行一致性验证。
    ///
    /// # 参考
    /// 参见 [`InputActionManager::action_map`]、[`InputActionMap`] 与 [`InputActionEvaluator::set_action_map`]。
    fn set_action_map(&self, action_map: InputActionMap) {
        self.lock_evaluator().set_action_map(action_map);
    }

    /// 按全部启用上下文求值一份输入帧。
    ///
    /// # 参数
    /// `frame` 是本次只读物理输入快照；求值不修改它。
    /// 本入口传入空活动列表，使用全部启用上下文；没有命名上下文的动作直接参与求值。
    /// 本入口不提供已消费按钮，按钮过滤集合为空。
    /// 本入口不提供已消费轴，轴过滤集合为空。
    ///
    /// # 返回
    /// 返回拥有动作名称、按住状态、边沿和模拟值的 [`InputActionState`]；结果由当前映射、帧和本次过滤共同决定。
    /// 没有绑定的动作没有被汇总的输入；本方法不会据此验证或排除配置中的已有空绑定。
    ///
    /// # 使用时机与生命周期
    /// 调用方在准备本帧快照后使用；同一管理器的配置访问和其它求值在外层锁上串行化。
    /// 求值借用已锁定求值器的内层工作区，不再获取该工作区锁；临时索引和动作槽在下一次准备时重置，容量可复用。
    /// 返回值不借用帧、消费切片、guard 或工作区；返回前释放外层锁，中毒取回 guard 不执行状态修复。
    ///
    /// # 参考
    /// 参见 [`InputActionManager`]、[`InputActionEvaluator`] 与 [`InputActionManager::set_action_map`]。
    fn evaluate_actions(&self, frame: &InputFrameSnapshot) -> InputActionState {
        self.lock_evaluator()
            .evaluate_while_manager_locked(frame, &[], &[], &[])
    }

    /// 按全部启用上下文及已消费按钮求值一份输入帧。
    ///
    /// # 参数
    /// `frame` 是本次只读物理输入快照；求值不修改它。
    /// 本入口传入空活动列表，使用全部启用上下文；没有命名上下文的动作直接参与求值。
    /// `consumed_buttons` 提供已由 UI 或更高层处理的按钮；绑定含任一已消费按钮时，整条绑定跳过，包括该绑定的轴。
    /// 本入口不提供已消费轴，轴过滤集合为空。
    ///
    /// # 返回
    /// 返回拥有动作名称、按住状态、边沿和模拟值的 [`InputActionState`]；结果由当前映射、帧和本次过滤共同决定。
    /// 没有绑定的动作没有被汇总的输入；本方法不会据此验证或排除配置中的已有空绑定。
    ///
    /// # 使用时机与生命周期
    /// 调用方在准备本帧快照和消费记录后使用；同一管理器的配置访问和其它求值在外层锁上串行化。
    /// 求值借用已锁定求值器的内层工作区，不再获取该工作区锁；临时索引和动作槽在下一次准备时重置，容量可复用。
    /// 返回值不借用帧、消费切片、guard 或工作区；返回前释放外层锁，中毒取回 guard 不执行状态修复。
    ///
    /// # 参考
    /// 参见 [`InputActionManager`]、[`InputActionEvaluator`] 与 [`InputActionManager::set_action_map`]。
    fn evaluate_actions_with_consumed_buttons(
        &self,
        frame: &InputFrameSnapshot,
        consumed_buttons: &[InputButton],
    ) -> InputActionState {
        self.lock_evaluator()
            .evaluate_while_manager_locked(frame, &[], consumed_buttons, &[])
    }

    /// 按全部启用上下文和已消费按钮、轴求值一份输入帧。
    ///
    /// # 参数
    /// `frame` 是本次只读物理输入快照；求值不修改它。
    /// 本入口传入空活动列表，使用全部启用上下文；没有命名上下文的动作直接参与求值。
    /// `consumed_buttons` 提供已由 UI 或更高层处理的按钮；绑定含任一已消费按钮时，整条绑定跳过，包括该绑定的轴。
    /// `consumed_axes` 提供本次已消费的轴来源；轴求值逐项忽略这些来源，绑定的其余轴仍可参与。
    ///
    /// # 返回
    /// 返回拥有动作名称、按住状态、边沿和模拟值的 [`InputActionState`]；结果由当前映射、帧和本次过滤共同决定。
    /// 没有绑定的动作没有被汇总的输入；本方法不会据此验证或排除配置中的已有空绑定。
    ///
    /// # 使用时机与生命周期
    /// 调用方在准备本帧快照和消费记录后使用；同一管理器的配置访问和其它求值在外层锁上串行化。
    /// 求值借用已锁定求值器的内层工作区，不再获取该工作区锁；临时索引和动作槽在下一次准备时重置，容量可复用。
    /// 返回值不借用帧、消费切片、guard 或工作区；返回前释放外层锁，中毒取回 guard 不执行状态修复。
    ///
    /// # 参考
    /// 参见 [`InputActionManager`]、[`InputActionEvaluator`] 与 [`InputActionManager::set_action_map`]。
    fn evaluate_actions_with_consumed_input(
        &self,
        frame: &InputFrameSnapshot,
        consumed_buttons: &[InputButton],
        consumed_axes: &[GamepadAxisInput],
    ) -> InputActionState {
        self.lock_evaluator().evaluate_while_manager_locked(
            frame,
            &[],
            consumed_buttons,
            consumed_axes,
        )
    }

    /// 按本次活动上下文求值一份输入帧。
    ///
    /// # 参数
    /// `frame` 是本次只读物理输入快照；求值不修改它。
    /// `active_contexts` 提供本次活动名称；空切片表示全部启用上下文，未知名称忽略，配置禁用的上下文仍禁用。
    /// 没有命名上下文的动作不受活动列表限制；列表仅对本次调用有效。
    /// 本入口不提供已消费按钮，按钮过滤集合为空。
    /// 本入口不提供已消费轴，轴过滤集合为空。
    ///
    /// # 返回
    /// 返回拥有动作名称、按住状态、边沿和模拟值的 [`InputActionState`]；结果由当前映射、帧和本次过滤共同决定。
    /// 没有绑定的动作没有被汇总的输入；本方法不会据此验证或排除配置中的已有空绑定。
    ///
    /// # 使用时机与生命周期
    /// 调用方在准备本帧快照后使用；同一管理器的配置访问和其它求值在外层锁上串行化。
    /// 求值借用已锁定求值器的内层工作区，不再获取该工作区锁；临时索引和动作槽在下一次准备时重置，容量可复用。
    /// 返回值不借用帧、消费切片、guard 或工作区；返回前释放外层锁，中毒取回 guard 不执行状态修复。
    ///
    /// # 参考
    /// 参见 [`InputActionManager`]、[`InputActionEvaluator`] 与 [`InputActionManager::set_action_map`]。
    fn evaluate_actions_with_active_contexts(
        &self,
        frame: &InputFrameSnapshot,
        active_contexts: &[&str],
    ) -> InputActionState {
        self.lock_evaluator()
            .evaluate_while_manager_locked(frame, active_contexts, &[], &[])
    }

    /// 按本次活动上下文及已消费按钮求值一份输入帧。
    ///
    /// # 参数
    /// `frame` 是本次只读物理输入快照；求值不修改它。
    /// `active_contexts` 提供本次活动名称；空切片表示全部启用上下文，未知名称忽略，配置禁用的上下文仍禁用。
    /// 没有命名上下文的动作不受活动列表限制；列表仅对本次调用有效。
    /// `consumed_buttons` 提供已由 UI 或更高层处理的按钮；绑定含任一已消费按钮时，整条绑定跳过，包括该绑定的轴。
    /// 本入口不提供已消费轴，轴过滤集合为空。
    ///
    /// # 返回
    /// 返回拥有动作名称、按住状态、边沿和模拟值的 [`InputActionState`]；结果由当前映射、帧和本次过滤共同决定。
    /// 没有绑定的动作没有被汇总的输入；本方法不会据此验证或排除配置中的已有空绑定。
    ///
    /// # 使用时机与生命周期
    /// 调用方在准备本帧快照和消费记录后使用；同一管理器的配置访问和其它求值在外层锁上串行化。
    /// 求值借用已锁定求值器的内层工作区，不再获取该工作区锁；临时索引和动作槽在下一次准备时重置，容量可复用。
    /// 返回值不借用帧、消费切片、guard 或工作区；返回前释放外层锁，中毒取回 guard 不执行状态修复。
    ///
    /// # 参考
    /// 参见 [`InputActionManager`]、[`InputActionEvaluator`] 与 [`InputActionManager::set_action_map`]。
    fn evaluate_actions_with_active_contexts_and_consumed_buttons(
        &self,
        frame: &InputFrameSnapshot,
        active_contexts: &[&str],
        consumed_buttons: &[InputButton],
    ) -> InputActionState {
        self.lock_evaluator().evaluate_while_manager_locked(
            frame,
            active_contexts,
            consumed_buttons,
            &[],
        )
    }

    /// 按本次活动上下文和已消费按钮、轴求值一份输入帧。
    ///
    /// # 参数
    /// `frame` 是本次只读物理输入快照；求值不修改它。
    /// `active_contexts` 提供本次活动名称；空切片表示全部启用上下文，未知名称忽略，配置禁用的上下文仍禁用。
    /// 没有命名上下文的动作不受活动列表限制；列表仅对本次调用有效。
    /// `consumed_buttons` 提供已由 UI 或更高层处理的按钮；绑定含任一已消费按钮时，整条绑定跳过，包括该绑定的轴。
    /// `consumed_axes` 提供本次已消费的轴来源；轴求值逐项忽略这些来源，绑定的其余轴仍可参与。
    ///
    /// # 返回
    /// 返回拥有动作名称、按住状态、边沿和模拟值的 [`InputActionState`]；结果由当前映射、帧和本次过滤共同决定。
    /// 没有绑定的动作没有被汇总的输入；本方法不会据此验证或排除配置中的已有空绑定。
    ///
    /// # 使用时机与生命周期
    /// 调用方在准备本帧快照和消费记录后使用；同一管理器的配置访问和其它求值在外层锁上串行化。
    /// 求值借用已锁定求值器的内层工作区，不再获取该工作区锁；临时索引和动作槽在下一次准备时重置，容量可复用。
    /// 返回值不借用帧、消费切片、guard 或工作区；返回前释放外层锁，中毒取回 guard 不执行状态修复。
    ///
    /// # 参考
    /// 参见 [`InputActionManager`]、[`InputActionEvaluator`] 与 [`InputActionManager::set_action_map`]。
    fn evaluate_actions_with_active_contexts_and_consumed_input(
        &self,
        frame: &InputFrameSnapshot,
        active_contexts: &[&str],
        consumed_buttons: &[InputButton],
        consumed_axes: &[GamepadAxisInput],
    ) -> InputActionState {
        self.lock_evaluator().evaluate_while_manager_locked(
            frame,
            active_contexts,
            consumed_buttons,
            consumed_axes,
        )
    }
}

/// 仅在测试构建中挂载默认动作管理器的私有恢复测试。
///
/// # 约束与生命周期
/// 路径属性把测试文件挂载为本模块的子模块，使其可访问私有锁助手；生产构建不包含该测试。
/// 测试主动制造一次外层锁中毒后检查访问、替换和无绑定动作求值；它不证明任意 panic 后的状态一致性。
///
/// # 参考
/// 参见 [`DefaultInputActionManager`] 与 [`InputActionManager`]。
#[cfg(test)]
#[path = "tests/default_input_action_manager.rs"]
mod tests;
