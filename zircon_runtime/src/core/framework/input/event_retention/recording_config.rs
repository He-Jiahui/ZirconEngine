//! 定义可选原始输入录制的配置值、默认条目容量和常量构造入口。
//!
//! # 使用时机与生命周期
//! 本文件只生成拥有 `bool` 和 `u32` 字段的值。管理器应用配置时才处理录制开关、
//! 已有记录和周期计数；构造或修改副本不会更新管理器。
//! 容量以原始记录条目计，允许零，不限制即时帧队列，也不限制事件负载的总字节数。
//!
//! # 参考
//! [`InputEventRecordingConfig`]、[`DEFAULT_INPUT_EVENT_RECORDING_CAPACITY`]、
//! [`crate::core::framework::input::InputManager::set_event_recording_config`]。
use serde::{Deserialize, Serialize};

/// 关闭构造器与默认构造器携带的原始录制容量：8,192 条记录。
///
/// # 使用时机与边界
/// [`InputEventRecordingConfig::disabled()`] 和 `Default` 使用本值；
/// [`InputEventRecordingConfig::enabled()`] 原样保存调用者提供的容量。
/// 这是配置默认值，构造时不会预分配记录，也不表示即时帧队列或总内存的上限。
///
/// # 参考
/// [`InputEventRecordingConfig`]、
/// [`crate::core::framework::input::InputManager::set_event_recording_config`]。
pub const DEFAULT_INPUT_EVENT_RECORDING_CAPACITY: u32 = 8_192;

/// 控制独立于帧事件队列的诊断录制；默认关闭，启用后容量限制保留的原始事件记录。
/// 容量为零仍统计丢弃数，重新启用会开始新的序列。
///
/// 以开关和条目容量表达原始输入录制策略的配置值。
///
/// # 构造与使用时机
/// `Default` 等价于 [`Self::disabled()`]，开关为 `false`、容量为 8,192。
/// [`Self::enabled()`] 原样保存指定的 `u32` 容量，包括零；构造器、公开字段及 Serde
/// 均不执行额外校验。应将配置交给管理器的配置入口，由具体实现应用。
///
/// # 默认实现的周期与容量规则
/// 默认输入管理器将配置原样交给记录器。关闭时清空记录、序号和丢弃数；
/// 从关闭切换到开启时开始新周期。持续开启的更新保持同一周期，缩容从队首淘汰最旧记录
/// 并累计丢弃；增容保留已有记录。零容量启用后仍推进序号、统计丢弃，但不保留记录。
/// 容量按条目计，不限制帧队列或事件负载字节。
///
/// # 生命周期与接口边界
/// 配置独立拥有两个标量，`Clone`/`Copy` 复制数值，不持有管理器、队列或锁。
/// 修改副本不会自行应用配置。接口默认的配置方法为空，具体实现是否支持录制应由调用者确认。
///
/// # 参考
/// [`Self::disabled()`]、[`Self::enabled()`]、[`DEFAULT_INPUT_EVENT_RECORDING_CAPACITY`]、
/// [`crate::core::framework::input::InputManager::set_event_recording_config`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputEventRecordingConfig {
    /// 请求是否开启原始事件录制。
    ///
    /// # 应用时机与生命周期
    /// 此布尔值只有交给支持录制的管理器后才生效。默认记录器应用 `false` 时清空周期状态；
    /// 此前关闭而新值为 `true` 时开始新周期，持续开启时保留序号和累计丢弃数。
    /// 开启且零容量仍统计丢弃；关闭时记录尝试不推进序号或计数。字段不借用记录器。
    ///
    /// # 参考
    /// [`Self::disabled()`]、
    /// [`crate::core::framework::input::InputManager::set_event_recording_config`]。
    pub enabled: bool,
    /// 请求保留的原始记录条目数上限；零是有效容量。
    ///
    /// # 应用时机与约束
    /// 默认记录器应用容量后裁剪超额的最旧记录；持续开启时缩容会累计丢弃，
    /// 后续满载写入也淘汰最旧条目。容量按条目数计，不按帧数或负载字节计。
    /// 构造与反序列化原样保存 `u32`，修改配置副本不会更新已有记录器。
    ///
    /// # 参考
    /// [`Self::enabled()`]、[`DEFAULT_INPUT_EVENT_RECORDING_CAPACITY`]、
    /// [`crate::core::framework::input::InputManager::set_event_recording_config`]。
    pub capacity: u32,
}

/// 提供可在常量上下文使用的录制配置构造器。
///
/// # 约束与生命周期
/// 两个入口只生成配置值。应用配置、重置周期及裁剪记录由管理器实现完成；
/// 构造不借用服务或分配记录容量，零容量按原值保留。
///
/// # 参考
/// [`Self::disabled()`]、[`Self::enabled()`]。
impl InputEventRecordingConfig {
    /// 构造关闭录制并携带默认条目容量的配置。
    ///
    /// # 参数
    /// 无参数，可在常量上下文调用。
    ///
    /// # 返回
    /// 返回独立的配置值：`enabled = false`，`capacity = DEFAULT_INPUT_EVENT_RECORDING_CAPACITY`。
    ///
    /// # 使用时机与生命周期
    /// 用于关闭策略或默认初始化。构造本值不清理已有记录；默认输入管理器应用此配置时
    /// 才清空记录、序号和累计丢弃数。返回值不借用记录器。
    ///
    /// # 参考
    /// [`DEFAULT_INPUT_EVENT_RECORDING_CAPACITY`]、
    /// [`crate::core::framework::input::InputManager::set_event_recording_config`]。
    pub const fn disabled() -> Self {
        Self {
            enabled: false,
            capacity: DEFAULT_INPUT_EVENT_RECORDING_CAPACITY,
        }
    }

    /// 构造开启录制并携带指定条目容量的配置。
    ///
    /// # 参数
    ///
    /// - `capacity`：原始记录的 `u32` 条目容量，原样保存；允许零。
    ///
    /// # 返回
    /// 返回独立的配置值：`enabled = true`，容量等于参数。
    ///
    /// # 使用时机、约束与生命周期
    /// 可在常量上下文调用。返回值不借用记录器，构造时不启动录制或裁剪已有记录。
    /// 默认管理器应用此值时，此前关闭会开始新周期，持续开启则保持周期并按容量裁剪；
    /// 零容量仍推进录制序号并累计丢弃，但不保留记录。
    ///
    /// # 参考
    /// [`Self::disabled()`]、
    /// [`crate::core::framework::input::InputManager::set_event_recording_config`]。
    pub const fn enabled(capacity: u32) -> Self {
        Self {
            enabled: true,
            capacity,
        }
    }
}

/// 把默认初始化定义为关闭录制并携带默认条目容量。
///
/// # 使用时机与生命周期
/// 默认记录器初始化采用此配置。默认构造只生成数值，不调用管理器；
/// 状态类型的全零 `Default` 是另一种数据合同，其容量为零。
///
/// # 参考
/// [`Self::disabled()`]、[`crate::core::framework::input::InputEventRecordingStatus`]。
impl Default for InputEventRecordingConfig {
    /// 生成与 [`Self::disabled()`] 相同的默认录制配置。
    ///
    /// # 参数
    /// 无参数。
    ///
    /// # 返回
    /// 返回开关关闭、容量为 8,192 的独立配置值。
    ///
    /// # 使用时机与生命周期
    /// 用于记录器等拥有者的默认值初始化。只构造数值，不读取管理器、分配记录或修改已有录制周期。
    /// 返回值没有外部借用；应用配置应另行调用管理器接口。
    ///
    /// # 参考
    /// [`Self::disabled()`]、[`DEFAULT_INPUT_EVENT_RECORDING_CAPACITY`]、
    /// [`crate::core::framework::input::InputManager::set_event_recording_config`]。
    fn default() -> Self {
        Self::disabled()
    }
}
