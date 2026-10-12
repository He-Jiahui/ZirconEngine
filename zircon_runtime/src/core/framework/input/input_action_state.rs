//! 保存命名动作的一次拥有型状态及其数值查询。
//!
//! # 责任
//!
//! 构造器接收调用方汇总的动作名称集合；查询只读取已有结果，不采样输入或重算动作。
//!
//! # 约束
//!
//! 三个布尔集合和数值表各自保存，类型不验证它们是否一致。数值清理只发生于相关构造器；派生反序列化直接恢复字段。
//!
//! # 使用时机与生命周期
//!
//! 状态拥有全部名称和值。后续求值或配置替换不会改写已有状态，销毁状态时释放其集合存储。
//!
//! # 参考
//!
//! [`InputActionState`]、[`InputActionState::from_sets()`]、[`InputActionState::from_sets_and_values()`]。
use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

/// 一次动作求值的按住、边沿与模拟值结果；构造器把值约束到负一到一，并从值表去除零与非有限值。
///
/// 拥有一份命名动作的布尔集合和模拟值结果。
///
/// # 责任
///
/// Serialize/Deserialize 派生直接处理字段。只有 values 在缺失时默认为空、为空时省略，其他三个集合没有缺失字段默认声明；反序列化不调用构造器，因此恢复值不自动清理或限幅，也不从 pressed 补出 1.0。
///
/// # 约束
///
/// 集合与数值表独立，构造器不检查激活/停用集合与按住集合之间的关系，也不核验动作是否已在映射声明。名称按 String 的大小写敏感顺序存储，允许空名称。
///
/// # 使用时机与生命周期
///
/// 派生 Default 得到空集合和空表，Clone 复制拥有的数据；后续输入帧或服务变化不会更新该值。PartialEq 使用字段和 f32 的普通相等语义。
///
/// # 参考
///
/// [`Self::from_sets()`]、[`Self::from_sets_and_values()`]、[`Self::pressed()`]、[`Self::value()`]。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct InputActionState {
    /// 拥有标记为按住的动作名称集合；去重并按名称有序。它与 values 及两个边沿集合独立，查询不由数值表推导。
    ///
    /// # 使用时机与生命周期
    ///
    /// 该集合或表随状态保存和释放，后续帧不会自动清空或更新。
    ///
    /// # 参考
    ///
    /// [`InputActionState::pressed()`]、[`InputActionState`]。
    pressed: BTreeSet<String>,
    /// 拥有本次结果标记为刚激活的名称；不在本类型内核验是否同时按住，也不保存上一帧用于推导边沿。
    ///
    /// # 使用时机与生命周期
    ///
    /// 该集合或表随状态保存和释放，后续帧不会自动清空或更新。
    ///
    /// # 参考
    ///
    /// [`InputActionState::just_activated()`]、[`InputActionState`]。
    just_activated: BTreeSet<String>,
    /// 拥有本次结果标记为刚停用的名称；与按住及激活集合独立，边沿窗口由产生状态的调用方决定。
    ///
    /// # 使用时机与生命周期
    ///
    /// 该集合或表随状态保存和释放，后续帧不会自动清空或更新。
    ///
    /// # 参考
    ///
    /// [`InputActionState::just_deactivated()`]、[`InputActionState`]。
    just_deactivated: BTreeSet<String>,
    /// 拥有按动作名称排序的数值表；布尔兼容构造器为按住项填 1.0，数值构造器移除零/非有限值并限幅。缺失字段反序列化为空表，派生反序列化不执行该清理；空表序列化时省略。
    ///
    /// # 使用时机与生命周期
    ///
    /// 该集合或表随状态保存和释放，后续帧不会自动清空或更新。
    ///
    /// # 参考
    ///
    /// [`InputActionState::value()`]、[`InputActionState`]。
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    values: BTreeMap<String, f32>,
}

impl InputActionState {
    /// 兼容仅有布尔动作状态的调用方：按住动作按满幅值解释。
    ///
    /// 从布尔动作集合构造状态，并为每个按住项提供满幅数值。
    ///
    /// # 参数
    ///
    /// - `pressed`：转交状态的按住名称集合。
    /// - `just_activated`：转交状态的刚激活名称集合。
    /// - `just_deactivated`：转交状态的刚停用名称集合。
    ///
    /// # 返回
    ///
    /// 返回拥有三个输入集合的状态；数值表包含 pressed 中全部名称，对应值为 1.0。
    ///
    /// # 约束
    ///
    /// 不核验三个集合的相互关系或动作声明。仅边沿集合中的名称不会因此获得模拟值。
    ///
    /// # 使用时机与生命周期
    ///
    /// 用于已有布尔动作结果的兼容构造。输入集合的所有权转入返回值；pressed 名称另外克隆到数值表，结果不借用调用方。
    ///
    /// # 参考
    ///
    /// [`Self::from_sets_and_values()`]、[`Self::value()`]、[`Self::pressed_actions()`]。
    pub fn from_sets(
        pressed: BTreeSet<String>,
        just_activated: BTreeSet<String>,
        just_deactivated: BTreeSet<String>,
    ) -> Self {
        let values = pressed
            .iter()
            .map(|action| (action.clone(), 1.0))
            .collect::<BTreeMap<_, _>>();
        Self {
            pressed,
            just_activated,
            just_deactivated,
            values,
        }
    }

    /// 从布尔集合与显式模拟值表构造拥有型状态。
    ///
    /// # 参数
    ///
    /// - `pressed`：转交状态的按住名称集合。
    /// - `just_activated`：转交状态的刚激活名称集合。
    /// - `just_deactivated`：转交状态的刚停用名称集合。
    /// - `values`：转交状态的名称到 f32 数值表；零和非有限值移除，有限非零值限幅到 [-1, 1]。
    ///
    /// # 返回
    ///
    /// 返回拥有三个原集合及清理后数值表的状态。
    ///
    /// # 约束
    ///
    /// 清理只作用于 values，不根据 pressed 补值、过滤名称或修改边沿集合；调用方负责它们之间的业务一致性。任意有限非零小值都会保留。
    ///
    /// # 使用时机与生命周期
    ///
    /// 用于带模拟量的求值结果。全部容器转移所有权，返回值不借用输入；此清理路径不包含派生反序列化。
    ///
    /// # 参考
    ///
    /// [`Self::from_sets()`]、[`Self::value()`]、[`Self::valued_actions()`]。
    pub fn from_sets_and_values(
        pressed: BTreeSet<String>,
        just_activated: BTreeSet<String>,
        just_deactivated: BTreeSet<String>,
        values: BTreeMap<String, f32>,
    ) -> Self {
        Self {
            pressed,
            just_activated,
            just_deactivated,
            values: normalized_action_values(values),
        }
    }

    /// 查询名称是否位于保存的按住集合。
    ///
    /// # 参数
    ///
    /// - `self`：共享借用已有状态。
    /// - `action`：按值传入的 AsRef<str> 名称提供者；使用其借出的字符串精确匹配。
    ///
    /// # 返回
    ///
    /// 返回该集合是否包含名称的布尔副本；未保存名称时为 false。
    ///
    /// # 使用时机与生命周期
    ///
    /// 只查询该状态已有结果，不推进帧或重新求值。名称借用仅用于本次查找；若传入拥有型名称提供者，其值按签名移入调用。
    ///
    /// # 参考
    ///
    /// [`Self::from_sets()`]、[`Self::from_sets_and_values()`]。
    pub fn pressed(&self, action: impl AsRef<str>) -> bool {
        self.pressed.contains(action.as_ref())
    }

    /// 查询名称是否位于保存的刚激活集合。
    ///
    /// # 参数
    ///
    /// - `self`：共享借用已有状态。
    /// - `action`：按值传入的 AsRef<str> 名称提供者；使用其借出的字符串精确匹配。
    ///
    /// # 返回
    ///
    /// 返回该集合是否包含名称的布尔副本；未保存名称时为 false。
    ///
    /// # 使用时机与生命周期
    ///
    /// 只查询该状态已有结果，不推进帧或重新求值。名称借用仅用于本次查找；若传入拥有型名称提供者，其值按签名移入调用。
    ///
    /// # 参考
    ///
    /// [`Self::from_sets()`]、[`Self::from_sets_and_values()`]。
    pub fn just_activated(&self, action: impl AsRef<str>) -> bool {
        self.just_activated.contains(action.as_ref())
    }

    /// 查询名称是否位于保存的刚停用集合。
    ///
    /// # 参数
    ///
    /// - `self`：共享借用已有状态。
    /// - `action`：按值传入的 AsRef<str> 名称提供者；使用其借出的字符串精确匹配。
    ///
    /// # 返回
    ///
    /// 返回该集合是否包含名称的布尔副本；未保存名称时为 false。
    ///
    /// # 使用时机与生命周期
    ///
    /// 只查询该状态已有结果，不推进帧或重新求值。名称借用仅用于本次查找；若传入拥有型名称提供者，其值按签名移入调用。
    ///
    /// # 参考
    ///
    /// [`Self::from_sets()`]、[`Self::from_sets_and_values()`]。
    pub fn just_deactivated(&self, action: impl AsRef<str>) -> bool {
        self.just_deactivated.contains(action.as_ref())
    }

    /// 读取数值表中保存的动作值。
    ///
    /// # 参数
    ///
    /// - `self`：共享借用已有状态。
    /// - `action`：按值传入的 AsRef<str> 名称提供者，用其字符串精确查找。
    ///
    /// # 返回
    ///
    /// 存在条目时返回存储 f32 的副本，缺失时返回 0.0。读取不再清理或限幅，也不从 pressed 推导满幅值。
    ///
    /// # 约束
    ///
    /// 构造器产生的数值受各自构造规则限制；派生反序列化得到的字段值可以绕过这些规则。
    ///
    /// # 使用时机与生命周期
    ///
    /// 查询不采样输入或改变状态，返回值不保留名称或状态借用。
    ///
    /// # 参考
    ///
    /// [`Self::from_sets()`]、[`Self::from_sets_and_values()`]、[`Self::valued_actions()`]。
    pub fn value(&self, action: impl AsRef<str>) -> f32 {
        self.values.get(action.as_ref()).copied().unwrap_or(0.0)
    }

    /// 复制保存的按住动作名称列表。
    ///
    /// # 参数
    ///
    /// - `self`：共享借用已有状态。
    ///
    /// # 返回
    ///
    /// 返回按 String 顺序排列且去重的 Vec<String>，每个名称均为拥有型副本；空集合返回空向量。
    ///
    /// # 使用时机与生命周期
    ///
    /// 用于枚举该状态的按住结果。结果不借用集合，不会随状态释放或后续求值改变。
    ///
    /// # 参考
    ///
    /// [`Self::pressed()`]、[`Self::from_sets()`]、[`Self::from_sets_and_values()`]。
    pub fn pressed_actions(&self) -> Vec<String> {
        self.pressed.iter().cloned().collect()
    }

    /// 复制保存的刚激活动作名称列表。
    ///
    /// # 参数
    ///
    /// - `self`：共享借用已有状态。
    ///
    /// # 返回
    ///
    /// 返回按 String 顺序排列且去重的 Vec<String>，每个名称均为拥有型副本；空集合返回空向量。
    ///
    /// # 使用时机与生命周期
    ///
    /// 用于枚举该状态的刚激活结果。结果不借用集合，不会随状态释放或后续求值改变。
    ///
    /// # 参考
    ///
    /// [`Self::just_activated()`]、[`Self::from_sets()`]、[`Self::from_sets_and_values()`]。
    pub fn just_activated_actions(&self) -> Vec<String> {
        self.just_activated.iter().cloned().collect()
    }

    /// 复制保存的刚停用动作名称列表。
    ///
    /// # 参数
    ///
    /// - `self`：共享借用已有状态。
    ///
    /// # 返回
    ///
    /// 返回按 String 顺序排列且去重的 Vec<String>，每个名称均为拥有型副本；空集合返回空向量。
    ///
    /// # 使用时机与生命周期
    ///
    /// 用于枚举该状态的刚停用结果。结果不借用集合，不会随状态释放或后续求值改变。
    ///
    /// # 参考
    ///
    /// [`Self::just_deactivated()`]、[`Self::from_sets()`]、[`Self::from_sets_and_values()`]。
    pub fn just_deactivated_actions(&self) -> Vec<String> {
        self.just_deactivated.iter().cloned().collect()
    }

    /// 复制数值表中全部名称和值。
    ///
    /// # 参数
    ///
    /// - `self`：共享借用已有状态。
    ///
    /// # 返回
    ///
    /// 返回按名称 String 顺序排列的 Vec<(String, f32)>；名称克隆、值复制，空表返回空向量。
    ///
    /// # 约束
    ///
    /// 枚举保存的表本身，不与 pressed 取交集，也不重新限幅或移除异常值。
    ///
    /// # 使用时机与生命周期
    ///
    /// 返回向量拥有每个名称，不保留状态借用，后续状态或服务变化不会回写此向量。
    ///
    /// # 参考
    ///
    /// [`Self::value()`]、[`Self::from_sets_and_values()`]。
    pub fn valued_actions(&self) -> Vec<(String, f32)> {
        self.values
            .iter()
            .map(|(action, value)| (action.clone(), *value))
            .collect()
    }
}

/// 筛除零与非有限模拟值，并限制有限非零值的幅度。
///
/// # 参数
///
/// - `value`：按值输入的 f32 模拟量。
///
/// # 返回
///
/// 正零、负零、NaN 及正负无穷返回 None；其他有限值返回限幅到 [-1, 1] 的 Some 值。
///
/// # 约束
///
/// 不设置死区或近零阈值，任意有限非零小值保留；符号在限幅后保留。
///
/// # 使用时机与生命周期
///
/// 由数值构造器的条目清理及测试基准使用，不读取或改变帧、映射及其他动作状态。
///
/// # 参考
///
/// [`InputActionState::from_sets_and_values()`]、[`normalized_action_values()`]。
fn normalized_action_value(value: f32) -> Option<f32> {
    if !value.is_finite() || value == 0.0 {
        None
    } else {
        Some(value.clamp(-1.0, 1.0))
    }
}

/// 在转交的数值树内删除无效项并覆盖保留值。
///
/// # 参数
///
/// - `values`：转交所有权的 BTreeMap<String, f32>；名称不改写，只按数值规则处理条目。
///
/// # 返回
///
/// 返回该拥有型数值表，零和非有限条目已移除，保留条目值已限幅。
///
/// # 约束
///
/// 复用输入树进行筛选，不创建用于重新收集条目的替换 BTreeMap；这不构成树内部重排、释放或实际分配次数的保证。
///
/// # 使用时机与生命周期
///
/// 由显式模拟值构造器调用；闭包借入键和可变值，不把借用带到返回结果。移除项的键值被丢弃；树节点存储的释放不由本方法逐项保证。
///
/// # 参考
///
/// [`normalized_action_value()`]、[`InputActionState::from_sets_and_values()`]。
fn normalized_action_values(mut values: BTreeMap<String, f32>) -> BTreeMap<String, f32> {
    values.retain(|_, value| {
        let Some(normalized) = normalized_action_value(*value) else {
            return false;
        };
        *value = normalized;
        true
    });
    values
}

/// 在测试构建挂载数值清理等价、源码形状和默认忽略的计时夹具。
///
/// # 使用时机与生命周期
///
/// 测试子模块可访问本模块私有清理函数；生产构建不包含这些夹具，源码形状检查和计时样例各有独立限制。
///
/// # 参考
///
/// [`InputActionState`]、[`normalized_action_value()`]、[`normalized_action_values()`]。
#[cfg(test)]
#[path = "tests/input_action_state.rs"]
mod tests;
