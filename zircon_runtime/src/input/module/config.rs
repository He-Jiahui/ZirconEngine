use serde::{Deserialize, Serialize};

use crate::core::framework::input::InputActionMap;

use super::super::runtime::{DefaultInputActionManager, InputActionEvaluator};

/// 输入服务的序列化启动配置；未启用时仍注册服务，但动作表对消费者呈现为空。
///
/// 拥有启用选项与原动作表的输入服务启动配置。
///
/// # 使用时机与约束
/// 在生成描述符或独立求值对象之前设置字段；`enabled` 决定这次构造采用原表的克隆还是空表。
/// 两个字段均公开，可直接构造和改写；构建方法不验证映射，也不限制名称、绑定或上下文的内容。
/// 派生 Clone 复制两字段；相等比较和 Serde 处理保存的原字段，而非经过启用选项过滤的有效映射。
/// 反序列化时缺失 `enabled` 使用 `false`，缺失整个 `action_map` 字段使用默认空表；给出嵌套表时仍遵循 InputActionMap 自身的字段规则。
///
/// # 生命周期
/// 配置拥有动作表；新的求值器或管理器拥有独立有效表。已经生成的工厂持有移入的配置，之后改写其它配置副本不会修改该工厂或已创建的服务。
/// 默认值禁用动作且保存空表；禁用不阻止模块声明或注册驱动与管理器，也不会持续限制后来显式安装的映射。
///
/// # 参考
/// 参见 [`Self::effective_action_map`]、[`Self::action_evaluator`]、[`Self::action_manager`] 与 [`super::module_descriptor_with_config`]。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputConfig {
    /// 决定新构造的动作对象是否取得保存映射的布尔选项。
    ///
    /// # 使用时机与约束
    /// 在读取有效动作表或生成描述符前设置；缺失字段和 Default 均为 `false`。
    /// 它不控制三个服务的声明、平台注册依赖、物理输入接收或 Core 激活；修改此字段也不向已创建的管理器发送配置更新。
    ///
    /// # 生命周期
    /// 值随配置保存、克隆或移入工厂，只在有效表被读取时生效；管理器后来调用 set_action_map 可安装映射，不再咨询此标志。
    ///
    /// # 参考
    /// 参见 [`Self::with_enabled`]、[`Self::effective_action_map`] 与 [`crate::core::framework::input::InputActionManager::set_action_map`]。
    #[serde(default)]
    pub enabled: bool,
    /// 配置拥有的原动作、上下文与物理绑定表。
    ///
    /// # 使用时机与约束
    /// 启动前通过公开字段或 with_action_map 替换；替换整表不会合并旧内容或执行配置验证。
    /// 缺失整个字段时使用三集合均为空的默认表；当字段存在时，嵌套反序列化由 InputActionMap 自身规则决定。
    /// 禁用选项只使这次有效表为空，不清除这里保存的配置；重新启用后可再次克隆保存表。
    ///
    /// # 生命周期
    /// 表随配置拥有、克隆和序列化；构造动作对象时移入的是独立的有效表，修改本字段不会回写既有求值器或管理器。
    ///
    /// # 参考
    /// 参见 [`InputActionMap`]、[`Self::with_action_map`] 与 [`Self::effective_action_map`]。
    #[serde(default)]
    pub action_map: InputActionMap,
}

/// 为默认模块入口提供禁用动作、保存空表的配置。
///
/// # 使用时机与生命周期
/// 生成默认描述符和构建链起点时使用；每次调用产生独立配置值，不拥有运行时或已注册服务。
/// 该默认实现只选择初始字段，不对任意公开构造的配置执行归一化。
///
/// # 参考
/// 参见 [`Self::default`]、[`InputActionMap`] 与 [`super::module_descriptor`]。
impl Default for InputConfig {
    /// 创建动作禁用且保存空表的独立启动配置。
    ///
    /// # 返回
    /// 返回 `enabled == false`、`action_map == InputActionMap::default()` 的 [`Self`]；默认动作表的上下文、动作与绑定均为空。
    ///
    /// # 使用时机与生命周期
    /// 用作默认模块描述符及值构建链的起点；返回值拥有表，不借用任何已有配置，也不注册服务。
    ///
    /// # 参考
    /// 参见 [`InputActionMap`] 与 [`super::module_descriptor`]。
    fn default() -> Self {
        Self {
            enabled: false,
            action_map: InputActionMap::default(),
        }
    }
}

/// 提供消费配置的值构建方法，以及按当前字段创建独立动作对象的入口。
///
/// # 使用时机与约束
/// 构建方法返回整份配置，便于在生成描述符前连续替换字段；它们不验证或合并映射。
/// 有效表入口只做本次启用投影；创建求值器或管理器不注册模块或服务。
///
/// # 生命周期
/// 构建方法消费并归还拥有的配置；其余入口仅在调用期间借用配置。`effective_action_map` 返回独立拥有的动作表；`action_evaluator` 与 `action_manager` 分别创建拥有独立动作表、编译代际和工作区的求值器或管理器，返回值不借用配置。
///
/// # 参考
/// 参见 [`Self::with_enabled`]、[`Self::with_action_map`]、[`Self::effective_action_map`] 与 [`Self::action_manager`]。
impl InputConfig {
    /// 替换配置的动作启用选项并返回配置。
    ///
    /// # 参数
    /// `self` 按值交出配置；`enabled` 是此后构造有效动作表时使用的布尔选项。
    ///
    /// # 返回
    /// 返回保存新选项的 [`Self`]；原 action_map 保持拥有的内容。
    ///
    /// # 使用时机与生命周期
    /// 用于生成描述符或动作对象前的构建链，消费并归还配置；不会修改既有工厂、求值器或服务，也不触发注册或激活。
    ///
    /// # 参考
    /// 参见 [`Self::enabled`]、[`Self::effective_action_map`] 与 [`Self::with_action_map`]。
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// 以给定动作表替换配置保存的整表。
    ///
    /// # 参数
    /// `self` 按值交出配置；`action_map` 按值移入并替换旧表，不合并、验证或归一化其内容。
    ///
    /// # 返回
    /// 返回拥有给定表的 [`Self`]；enabled 选项保留当前值。
    ///
    /// # 使用时机与生命周期
    /// 用于启动前配置及准备另一份重绑定配置。输入表由返回配置持有；已生成的工厂或动作对象不观察这次替换。
    ///
    /// # 参考
    /// 参见 [`InputActionMap`]、[`Self::action_map`] 与 [`Self::effective_action_map`]。
    pub fn with_action_map(mut self, action_map: InputActionMap) -> Self {
        self.action_map = action_map;
        self
    }

    /// 在模块工厂或独立求值器创建前读取；返回拥有所有权的表，后续重绑定须显式更新管理器。
    ///
    /// 读取当前启用选项并创建本次构造使用的独立动作表。
    ///
    /// # 参数
    /// `self` 在调用期间只读借用，提供 enabled 和保存的原动作表。
    ///
    /// # 返回
    /// 启用时返回原表的 Clone；禁用时返回默认空表。两分支都返回拥有数据的 [`InputActionMap`]，不借用或清空配置保存表。
    ///
    /// # 使用时机与生命周期
    /// 在创建求值器或管理器时读取；调用完成后返回表可独立移入另一对象。后续配置改写不更新已返回表、工厂捕获副本或已经创建的对象。
    /// 此方法不验证映射，也不控制服务注册、物理事件接收或运行期持续启用策略。
    ///
    /// # 参考
    /// 参见 [`Self::enabled`]、[`Self::action_evaluator`]、[`Self::action_manager`] 与 [`InputActionMap`]。
    pub fn effective_action_map(&self) -> InputActionMap {
        if self.enabled {
            self.action_map.clone()
        } else {
            InputActionMap::default()
        }
    }

    /// 按当前有效动作表创建独立的动作求值器。
    ///
    /// # 参数
    /// `self` 在构造期间只读借用；有效表由当前 enabled 选项决定。
    ///
    /// # 返回
    /// 返回拥有有效映射、新编译代际和独立工作区的 [`InputActionEvaluator`]；该对象尚未对任何帧求值。
    ///
    /// # 使用时机与生命周期
    /// 用于独立求值或从另一配置创建重绑定对象。映射移入返回值，不借用配置；之后改写配置不会回写此对象，已有对象的配置替换须走其 set_action_map。
    /// 构造不注册服务，也不把原 InputConfig 或其 enabled 选项保存为持续运行期开关。
    ///
    /// # 参考
    /// 参见 [`Self::effective_action_map`]、[`InputActionEvaluator::new`] 与 [`InputActionEvaluator::set_action_map`]。
    pub fn action_evaluator(&self) -> InputActionEvaluator {
        InputActionEvaluator::new(self.effective_action_map())
    }

    /// 按当前有效动作表创建独立的默认动作管理器值。
    ///
    /// # 参数
    /// `self` 仅在构造期间借用，提供 enabled 及保存表；有效表被克隆或按默认值建立后移入管理器。
    ///
    /// # 返回
    /// 返回拥有独立求值器和外层互斥锁的 [`DefaultInputActionManager`]；它不是已注册服务句柄。
    ///
    /// # 使用时机与生命周期
    /// 通常由模块动作工厂在服务初始化时调用；每次调用建立新管理器及其代际和工作区，不复用以前返回的状态。
    /// 返回值不借用配置，后续映射更改须显式调用管理器 trait 的 set_action_map；enabled 不作为持续运行期开关保存于管理器。
    /// 实际共享包装、注册和激活分别由工厂与 Core 推进。
    ///
    /// # 参考
    /// 参见 [`Self::effective_action_map`]、[`DefaultInputActionManager::new`] 与 [`crate::core::framework::input::InputActionManager::set_action_map`]。
    pub fn action_manager(&self) -> DefaultInputActionManager {
        DefaultInputActionManager::new(self.effective_action_map())
    }
}
