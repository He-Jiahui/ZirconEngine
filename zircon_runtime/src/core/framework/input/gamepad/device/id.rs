//! 设备编号纯值及其派生值语义。
//!
//! # 职责、使用时机与生命周期
//! 宿主事件和请求使用本类型配对目标；本文件仅定义 u64 包装，没有设备分配、查找或连接订阅。
//!
//! # 参考
//! [`GamepadId`]、[`crate::core::framework::input::InputEvent`]。
use serde::{Deserialize, Serialize};

/// 携带事件生产者提供的手柄编号，供连接、按钮、轴状态和震动请求关联同一目标。
///
/// # 使用时机
///
/// 从当前宿主的设备枚举或输入事件取得编号后，用它作为设备状态键或输出请求目标。
///
/// # 字段
///
/// - `0`：原始 `u64` 编号，公开构造允许零值，也不检查是否对应已连接设备。
///
/// # 约束
///
/// 相等、排序与散列只依据完整数字。当前原生 App 将 Gilrs 的 usize 编号转为 u64；本类型没有宿主命名空间或设备世代，因此跨会话稳定性和断连后编号重用需由生产者与消费方约定。
///
/// # 所有权与生命周期
///
/// 值可 Copy、Clone 和序列化，不借用或保活设备。旧值及旧快照可继续存在，使用点仍需按当前设备映射判断目标是否可执行。
///
/// # 参考
///
/// [`crate::core::framework::input::GamepadConnectionInfo`]、[`crate::core::framework::input::InputEvent`]、[`crate::core::framework::input::InputFrameSnapshot`]、[`crate::core::framework::input::GamepadRumbleRequest`]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GamepadId(pub u64);
