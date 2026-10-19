//! 调用方配置的反射值遍历配额及其校验错误类型。
//!
//! 配额供 [crate::reflect::ReflectedValue::validate_with_budget] 检查已构造的反射值树；
//! 它不代替 schema、字段类型或权限校验。Runtime 入口策略定义在
//! zircon_runtime/src/scene/reflect/value_admission.rs。

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 一次反射值验证的调用方配额，覆盖反射容器及嵌入的 JSON 子树；入口按信任边界选择上限，本类型不提供隐式默认值。
///
/// # 使用时机与生命周期
/// 由入口按信任边界创建，再按值传给 [crate::reflect::ReflectedValue::validate_with_budget]；本类型实现 Copy，不持有借用。
///
/// # 约束
/// 根深度为 1；反射包装值和嵌入 JSON 节点都计入节点数；文本载荷和 Map/JSON 对象键共用 UTF-8 字节总额；
/// 容器条目上限逐容器比较。节点额度不承诺工作栈最大宽度或内存上限。对应验证器除配额外检查反射浮点有限性，
/// 不检查四元数归一化、资源 URI、schema、字段类型或权限；零值可表达。
///
/// # 参考
/// Runtime 上限定义在 zircon_runtime/src/scene/reflect/value_admission.rs；
/// 契约样例见 zircon_runtime_interface/src/tests/reflect_value_budget_contracts.rs。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReflectValueBudget {
    /// 单次遍历允许的最大一基深度；反射根深度为 1。
    max_depth: usize,
    /// ReflectedValue 包装和嵌入 JSON 子树合计可访问的节点上限。
    max_nodes: usize,
    /// 反射 String/Enum/Resource、JSON 字符串值及 Map/对象键共用的 UTF-8 字节上限。
    max_string_bytes: usize,
    /// 每个 List、Map、JSON 数组或对象各自允许的直接条目上限。
    max_container_entries: usize,
}

// 访问器返回原配置边界；遍历状态由验证器单独维护。

impl ReflectValueBudget {
    /// 原样保存调用方指定的四个上限，不替调用方选择验证策略。
    ///
    /// # 参数
    /// 参数依次为最大深度、节点数、字符串 UTF-8 字节数和单容器直接条目数。
    ///
    /// # 返回值
    /// 返回可复制并传给 [crate::reflect::ReflectedValue::validate_with_budget] 的配额值。
    ///
    /// # 约束
    /// 本构造器不验证或收窄上限；包括零值在内的 usize 参数都会原样保存。
    pub const fn new(
        max_depth: usize,
        max_nodes: usize,
        max_string_bytes: usize,
        max_container_entries: usize,
    ) -> Self {
        Self {
            max_depth,
            max_nodes,
            max_string_bytes,
            max_container_entries,
        }
    }

    /// 返回调用方设定的一基最大深度；根值按深度 1 计。
    ///
    /// # 返回值
    /// 返回此配额保存的 max_depth 原值。
    pub const fn max_depth(self) -> usize {
        self.max_depth
    }

    /// 返回整次遍历可访问的反射及 JSON 节点总上限。
    ///
    /// # 返回值
    /// 返回此配额保存的 max_nodes 原值。
    pub const fn max_nodes(self) -> usize {
        self.max_nodes
    }

    /// 返回字符串载荷和映射键共用的累计 UTF-8 字节上限。
    ///
    /// # 返回值
    /// 返回此配额保存的 max_string_bytes 原值。
    pub const fn max_string_bytes(self) -> usize {
        self.max_string_bytes
    }

    /// 返回每个容器单独采用的直接条目上限。
    ///
    /// # 返回值
    /// 返回此配额保存的 max_container_entries 原值。
    pub const fn max_container_entries(self) -> usize {
        self.max_container_entries
    }
}

/// 标识非有限错误来自哪一种反射浮点载荷；不表示向量或四元数归一化检查。

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReflectValueFloatKind {
    /// 标量；唯一分量的错误索引为 0。
    Scalar,
    /// 二维向量；错误索引从 0 开始。
    Vec2,
    /// 三维向量；错误索引从 0 开始。
    Vec3,
    /// 四维向量；错误索引从 0 开始。
    Vec4,
    /// 四元数；只检查分量有限性，错误索引从 0 开始。
    Quaternion,
}

/// 标识节点、深度或字符串累计发生算术溢出的配额维度。

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReflectValueBudgetDimension {
    /// 递增子节点深度时溢出。
    Depth,
    /// 递增节点累计数时溢出。
    Nodes,
    /// 累计 UTF-8 字节数时溢出。
    StringBytes,
}

/// 反射值遍历违反配额或包含非有限反射浮点时返回的错误；不判断字段类型、schema 或权限。

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReflectValueValidationError {
    #[error("reflected value {dimension:?} accounting overflowed")]
    /// 节点、深度或字符串累计的 checked_add 无法表示下一计数值。
    BudgetArithmeticOverflow {
        /// 标识发生计数溢出的预算维度。
        dimension: ReflectValueBudgetDimension,
    },
    #[error("reflected value depth {actual} exceeds the configured limit {maximum}")]
    /// 根或后代实际一基深度超限；actual 为首次越界深度，maximum 为调用方上限。
    DepthExceeded { actual: usize, maximum: usize },
    #[error("reflected value node count {actual} exceeds the configured limit {maximum}")]
    /// 计入下一节点后总数超限；actual 为尝试累计数，maximum 为调用方上限。
    NodeBudgetExceeded { actual: usize, maximum: usize },
    #[error("reflected value string bytes {actual} exceed the configured limit {maximum}")]
    /// 计入下一字符串后字节总额超限；actual 为尝试累计量，maximum 为调用方上限。
    StringBudgetExceeded { actual: usize, maximum: usize },
    #[error(
        "reflected value container has {actual} entries, exceeding the configured limit {maximum}"
    )]
    /// 当前单个容器直接条目数超限，不是整棵值树的总条目数。
    ContainerEntriesExceeded { actual: usize, maximum: usize },
    #[error("reflected {kind:?} component {component} is not finite")]
    /// 反射浮点指定分量非有限；component 使用从 0 开始的索引。
    NonFiniteFloat {
        kind: ReflectValueFloatKind,
        component: usize,
    },
}
