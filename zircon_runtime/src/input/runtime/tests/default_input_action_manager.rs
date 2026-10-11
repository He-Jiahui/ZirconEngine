//! 检查默认动作服务在一次主动制造的外层锁中毒后仍能访问和替换配置。
//!
//! # 责任与入口
//! 父模块以测试条件和路径属性挂载本文件；夹具可访问父类型的私有锁助手。
//! 测试在 guard 存续期间 panic，再调用配置设置、读取和默认帧求值。
//!
//! # 约束与生命周期
//! 捕获依赖可展开的 panic；闭包在 panic 前不改变求值器状态，测试不证明任意 panic 后的不变式。
//! 声明的动作没有任何绑定，也没有命名上下文；它与配置中已有空按钮、空轴绑定不同。
//! 本文件及夹具只在测试构建中挂载。
//!
//! # 参考
//! 参见 [`DefaultInputActionManager`]、[`InputActionManager`] 与 [`InputActionMap`]。
use std::panic::{self, AssertUnwindSafe};

use crate::core::framework::input::InputActionManager;
use crate::input::{InputAction, InputActionMap, InputFrameSnapshot};

use super::DefaultInputActionManager;

/// 检查外层求值器锁在一次主动中毒后仍允许配置访问和无绑定动作求值。
///
/// # 测试场景
/// 夹具使用默认管理器，在私有 guard 存续期间触发展开的 panic。
/// `AssertUnwindSafe` 只满足捕获边界；闭包在 panic 前不修改求值器，捕获结果被丢弃。
///
/// # 返回
/// 返回 `()`；断言配置克隆等于替换后的本地映射，且默认帧下 `gameplay.jump` 未按住。
/// 断言失败会使测试失败；测试不直接断言中毒标志、panic 内容或所有内部状态的一致性。
///
/// # 使用时机与生命周期
/// 用于支持可展开 panic 的测试构建，配置、帧和返回状态均由本夹具局部拥有。
/// 该动作只有声明，没有任何绑定且没有命名上下文；此断言不覆盖已有空按钮、空轴绑定的求值。
/// 测试不涉及并发调用，也不证明任意 panic 之后的状态修复。
///
/// # 参考
/// 参见 [`DefaultInputActionManager`]、[`InputActionManager::set_action_map`]、[`InputActionManager::action_map`] 与 [`InputActionManager::evaluate_actions`]。
#[test]
fn input_action_manager_accessors_recover_poisoned_evaluator_lock() {
    let manager = DefaultInputActionManager::default();
    let _ = panic::catch_unwind(AssertUnwindSafe(|| {
        let _guard = manager.lock_evaluator();
        panic!("poison input action evaluator");
    }));

    let mut action_map = InputActionMap::new();
    action_map.add_action(InputAction::new("gameplay.jump"));
    manager.set_action_map(action_map.clone());

    assert_eq!(manager.action_map(), action_map);
    assert!(!manager
        .evaluate_actions(&InputFrameSnapshot::default())
        .pressed("gameplay.jump"));
}
