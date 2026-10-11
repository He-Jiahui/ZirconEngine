//! 面向宿主的手柄震动命令值与纯构造入口。
//!
//! # 职责、使用时机与生命周期
//! 本文件只构造 Add、Stop 和读取目标。队列交付、设备查找、效果播放及停止由后续管理器与宿主承担。
//!
//! # 参考
//! [`GamepadRumbleRequest`]、[`crate::core::framework::input::InputEvent`]。
use serde::{Deserialize, Serialize};

use super::super::device::GamepadId;
use super::GamepadRumbleIntensity;

/// 动作侧发往宿主设备驱动的命令；管理器排空请求后才由宿主执行，不代表振动已生效。
///
/// 携带面向一个手柄的新增振动或停止意图，供支持请求的输入管理器交给宿主消费。
///
/// # 使用时机
///
/// 用 add 或 stop 构造后提交 InputEvent::GamepadRumbleRequest。当前默认输入实现按提交顺序保留请求；InputManager 的默认排空方法只返回空列表，支持程度须由具体实现确定。
///
/// # 约束
///
/// 公开变体、add 及 serde 还原都不校验目标、不归一化强度、不修改持续时间。出站转换归一化 Add 强度；当前 App 的 Gilrs 适配在执行点查找目标、将 Add 时长最多取 10,000 毫秒并限制每设备 32 个保留效果，可能拒绝请求或完成空执行。
///
/// # 所有权与生命周期
///
/// 请求拥有可复制字段，没有设备或效果引用。默认帧快照只复制本帧新增请求；begin_frame 隐藏旧请求而保留待交付队列，排空方法转移全部尚未消费请求。保存或复制快照不会消费队列，重复提交副本会成为新的请求。
///
/// # 参考
///
/// [`crate::core::framework::input::GamepadId`]、[`crate::core::framework::input::GamepadRumbleIntensity::clamped()`]、[`crate::core::framework::input::InputEvent::GamepadRumbleRequest`]、[`crate::core::framework::input::InputManager::drain_gamepad_rumble_requests()`]、[`crate::core::framework::input::InputFrameSnapshot::gamepad_rumble_requests`]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum GamepadRumbleRequest {
    /// 请求宿主为指定设备新增一个有时限的振动效果。
    ///
    /// # 约束
    ///
    /// 保留原始强度和 u32 毫秒时长；当前 Gilrs 宿主先解析目标设备；目标可用后，零时长或双电机归一化并量化后均为零可返回空执行结果。多次 Add 可保留多个效果，具体限额由宿主控制。
    ///
    /// # 所有权与生命周期
    ///
    /// 此变体只拥有参数；效果创建、播放与保留发生在宿主消费期间。
    ///
    /// # 参考
    ///
    /// [`crate::core::framework::input::GamepadRumbleRequest::add()`]、[`crate::core::framework::input::GamepadRumbleIntensity::clamped()`]
    Add {
        /// 新增效果的目标设备编号，采用当前宿主输入事件的同一身份域。
        ///
        /// # 约束
        ///
        /// 允许任意 GamepadId，构造阶段不查连接；消费方用当前映射判定可执行目标。
        ///
        /// # 所有权与生命周期
        ///
        /// 按值拥有，不保活设备。
        ///
        /// # 参考
        ///
        /// [`crate::core::framework::input::GamepadId`]、[`crate::core::framework::input::GamepadRumbleRequest::gamepad()`]
        gamepad: GamepadId,
        /// 新增效果希望使用的强、弱电机原始浮点强度。
        ///
        /// # 约束
        ///
        /// 该字段可直接含越界、NaN 或无穷值；公开变体和 add 不调用 clamped。当前出站 ABI 转换将有限值截到 [0,1]、非有限值改为零。
        ///
        /// # 所有权与生命周期
        ///
        /// 纯值复制；后续归一化产生新值，已保存的请求副本仍保留原值。
        ///
        /// # 参考
        ///
        /// [`crate::core::framework::input::GamepadRumbleIntensity`]、[`crate::core::framework::input::GamepadRumbleIntensity::clamped()`]
        intensity: GamepadRumbleIntensity,
        /// 新增效果请求的持续时间，单位为毫秒，取值由 u32 表示。
        ///
        /// # 约束
        ///
        /// add 原样保存 0 到 u32::MAX。当前 Gilrs 宿主先解析目标，再把非零时长限制到 10,000 毫秒；目标可用时，零时长不创建效果，也不代替 Stop。实际播放时序由宿主和设备决定。
        ///
        /// # 所有权与生命周期
        ///
        /// 数值随请求按值保存，不是等待完成的计时器；保留旧副本不会延长已播放效果。
        ///
        /// # 参考
        ///
        /// [`crate::core::framework::input::GamepadRumbleRequest::add()`]、[`crate::core::framework::input::GamepadRumbleRequest::Stop`]
        duration_millis: u32,
    },
    /// 请求宿主停止该设备当前保留的全部振动效果。
    ///
    /// # 约束
    ///
    /// 变体仅携带目标编号。当前 Gilrs 宿主先解析设备，再移除目标效果记录并逐个调用停止；停止失败记为告警，不能由这个值确认电机已停止。
    ///
    /// # 所有权与生命周期
    ///
    /// 构造或丢弃 Stop 值不会操作设备；需由请求消费方执行。
    ///
    /// # 参考
    ///
    /// [`crate::core::framework::input::GamepadRumbleRequest::stop()`]、[`crate::core::framework::input::GamepadId`]
    Stop {
        /// 停止操作的目标设备编号，停止范围是该宿主为此编号保留的全部效果。
        ///
        /// # 约束
        ///
        /// 停止请求自身不检验在线状态或设备世代；当前宿主消费时重新查找目标，查找失败可拒绝此操作。
        ///
        /// # 所有权与生命周期
        ///
        /// 只有可复制编号，未拥有任何效果句柄。
        ///
        /// # 参考
        ///
        /// [`crate::core::framework::input::GamepadId`]、[`crate::core::framework::input::GamepadRumbleRequest::stop()`]
        gamepad: GamepadId,
    },
}

impl GamepadRumbleRequest {
    /// 构造一个携带原始目标、双电机强度和毫秒时长的 Add 请求。
    ///
    /// # 使用时机
    ///
    /// 需要新增振动意图时先构造此值，再提交到支持请求的输入管理器；本调用可用于常量构造。
    ///
    /// # 参数
    ///
    /// - `gamepad`：按值传入目标编号，由消费宿主在自己的当前身份域内解析。
    /// - `intensity`：按值传入原始强度，允许任意 f32 内容；此函数不调用 clamped。
    /// - `duration_millis`：按值传入 u32 毫秒数，包含零，原样写入请求。
    ///
    /// # 返回
    ///
    /// 拥有型 Self::Add 值，字段与参数一致；返回值只证明完成构造，不证明排队、连接、准入或设备播放。
    ///
    /// # 约束
    ///
    /// 请求传给宿主前应执行强度归一化。当前原生宿主另对设备能力、时长和效果数施加限制，零时长不替代停止命令。
    ///
    /// # 所有权与生命周期
    ///
    /// 参数均为 Copy 值，不借用设备或强度存储；构造时没有需要取消的异步工作；提交和设备执行由后续拥有者承担。
    ///
    /// # 参考
    ///
    /// [`crate::core::framework::input::GamepadRumbleRequest::Add`]、[`crate::core::framework::input::GamepadRumbleIntensity::clamped()`]、[`crate::core::framework::input::InputEvent::GamepadRumbleRequest`]、[`crate::core::framework::input::InputManager::drain_gamepad_rumble_requests()`]
    pub const fn add(
        gamepad: GamepadId,
        intensity: GamepadRumbleIntensity,
        duration_millis: u32,
    ) -> Self {
        Self::Add {
            gamepad,
            intensity,
            duration_millis,
        }
    }

    /// 构造让宿主停止指定设备全部已保留振动效果的 Stop 请求。
    ///
    /// # 使用时机
    ///
    /// 需要结束该目标的振动时构造并提交请求，之后由支持请求的管理器排空给宿主。
    ///
    /// # 参数
    ///
    /// - `gamepad`：按值传入停止目标的当前宿主设备编号；构造不查设备或效果是否存在。
    ///
    /// # 返回
    ///
    /// 拥有型 Self::Stop 值；此函数没有执行停止，也没有设备完成确认。
    ///
    /// # 约束
    ///
    /// 当前 Gilrs 宿主消费时先重新解析设备，随后移除目标效果并尝试停止。平台停止失败可仅告警，调用者不可将构造成功或排队当作设备已停止。
    ///
    /// # 所有权与生命周期
    ///
    /// 请求只拥有 Copy 编号，不借用或保留驱动；丢弃未提交请求不会操作已有效果。
    ///
    /// # 参考
    ///
    /// [`crate::core::framework::input::GamepadRumbleRequest::Stop`]、[`crate::core::framework::input::GamepadId`]、[`crate::core::framework::input::InputManager::drain_gamepad_rumble_requests()`]
    pub const fn stop(gamepad: GamepadId) -> Self {
        Self::Stop { gamepad }
    }

    /// 从 Add 或 Stop 请求中按值取得相同的目标设备编号。
    ///
    /// # 使用时机
    ///
    /// 消费方需要按设备分组、路由或检查请求目标时调用；两种变体都可使用，且可在常量上下文求值。
    ///
    /// # 参数
    ///
    /// - `self`：按值接收可 Copy 的请求，访问的只是其中的 gamepad 字段。
    ///
    /// # 返回
    ///
    /// 请求中原样保存的 GamepadId；编号不包含当前在线状态、宿主世代或执行结果。
    ///
    /// # 约束
    ///
    /// 返回编号只表示请求目标；执行时仍需由当前宿主映射解析该编号。
    ///
    /// # 所有权与生命周期
    ///
    /// 返回拥有型 Copy 编号，没有对请求或设备的借用；方法不消费管理器队列，也不启动或停止效果。
    ///
    /// # 参考
    ///
    /// [`crate::core::framework::input::GamepadId`]、[`crate::core::framework::input::GamepadRumbleRequest::Add`]、[`crate::core::framework::input::GamepadRumbleRequest::Stop`]
    pub const fn gamepad(self) -> GamepadId {
        match self {
            Self::Add { gamepad, .. } | Self::Stop { gamepad } => gamepad,
        }
    }
}
