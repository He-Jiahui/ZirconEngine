//! 声明输入服务的事件提交、拥有型查询、队列排空和可选录制入口。
//!
//! # 职责与使用时机
//! 宿主或回放通过 InputManager 提交事件，消费方查询快照并领取事件或反向请求。16 个方法中 5 个必须由实现提供，11 个带有兼容默认体；服务工厂和解析器只注册、解析实际实现，不由此创建设备线程。
//!
//! # 边界与生命周期
//! 本模块不持有设备或输入状态，也不规定全局调用顺序。帧边界、验证、保留及宿主执行由实现与调用方负责；查询和排空按值返回数据，多次调用不自动组成一个事务。
//!
//! # 参考
//! 参见 [InputManager](super::InputManager)、[InputSnapshot](super::InputSnapshot)、[InputFrameSnapshot](super::InputFrameSnapshot)、[InputEventRecord](super::InputEventRecord)。
use std::num::NonZeroU64;

use crate::core::framework::channel::ChannelReceiver;

use super::{
    ButtonInputState, CursorHostRequest, GamepadId, GamepadRumbleRequest, ImeHostRequest, InputButton,
    InputEvent, InputEventQueueStatus, InputEventRecord, InputEventRecordingConfig,
    InputEventRecordingStatus, InputFrameSnapshot, InputSnapshot, MouseWheelEvent,
};

/// Result of atomically admitting an event into input state and its retained event streams.
/// Accepted generation admission does not imply a numeric value survived existing filtering,
/// that a recorder retained it, or that any UI/host callback has run.
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputEventAdmission {
    Accepted,
    Rejected(GamepadInputRejection),
}

/// A gamepad rejection never changes samples, edges, frame events or recording buffers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GamepadInputRejection {
    GenerationRequired { gamepad: GamepadId },
    /// Rumble intents can only bind to a currently observed, open connection.
    RumbleTargetUnavailable { gamepad: GamepadId },
    ConnectionNotObserved { gamepad: GamepadId, submitted: NonZeroU64 },
    StaleGeneration { gamepad: GamepadId, submitted: NonZeroU64, current: NonZeroU64 },
    FutureGeneration { gamepad: GamepadId, submitted: NonZeroU64, current: NonZeroU64 },
    ConnectionClosed { gamepad: GamepadId, submitted: NonZeroU64, current: NonZeroU64 },
}

/// A rumble intent bound to the connection observed atomically at enqueue time.
///
/// This owned value must retain its generation through drains, output-page retries and host
/// delivery. Consumers must never replace it with a generation observed at drain time.
/// The stamp proves observed input ownership, not physical device availability or completion.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QueuedGamepadRumbleRequest {
    request: GamepadRumbleRequest,
    generation: NonZeroU64,
}

impl QueuedGamepadRumbleRequest {
    /// Construct only while the input owner admits this intent against its open connection.
    pub const fn new(request: GamepadRumbleRequest, generation: NonZeroU64) -> Self {
        Self { request, generation }
    }

    pub const fn request(&self) -> GamepadRumbleRequest {
        self.request
    }

    pub const fn generation(&self) -> NonZeroU64 {
        self.generation
    }
}

/// 通过共享借用提供输入状态查询及事件消费的线程可共享服务契约。
///
/// # 使用时机与职责
/// 宿主提交拥有型事件；调用方在约定帧边界读取状态、消费事件或领取反向宿主请求。服务解析器借注册的实现提供同一服务，接口本身不创建设备线程或事件循环。
///
/// # 约束
/// 实现必须提供 submit_event_with_admission、snapshot、drain_events、drain_event_records 和 drain_event_records_with_status 共 5 个方法。其余 11 个方法具有兼容默认体；空操作、空向量、零状态和无订阅分别表示默认适配行为，不能据此推断实现具备这些能力。Send 与 Sync 不规定调用顺序、执行线程或全局帧事务；记录与状态联合排空另有单事务要求。
///
/// # 生命周期
/// 各查询结果按值返回；查询与排空结果不借用本服务。事件及请求的保留期限由实现负责，跨多次查询的状态不自动属于同一时刻。
///
/// # 参考
/// 参见 [InputSnapshot](super::InputSnapshot)、[InputFrameSnapshot](super::InputFrameSnapshot)、[InputEventRecord](super::InputEventRecord)、[InputEventRecordingStatus](super::InputEventRecordingStatus)。
/// 宿主事件进入运行时输入栈的服务边界；一帧先 `begin_frame`、再提交事件、再读取快照或排空队列。
/// 完整实现同时维护设备持久状态、帧内通知和可选原始记录，调用方须区分这些生命周期。
pub trait InputManager: Send + Sync {
    /// 通知实现切换输入帧的边界。
    ///
    /// # 使用时机与职责
    /// 由宿主帧循环调用；当前动态会话在关卡 tick 读取输入后调用此方法，为后续提交准备新帧。
    ///
    /// # 返回
    /// 不返回值。
    ///
    /// # 约束
    /// 本 trait 默认不执行任何操作。DefaultInputManager 清除按键边沿、即时事件、滚动/移动累积和若干帧通知，保留按下、触点、连接设备、模拟状态和 IME 持续状态；待发宿主请求和原始记录不因切帧自动排空。 每次调用都会丢弃未读的本帧即时事件；重复调用也建立新窗口。DefaultInputManager 默认构造后即可提交，首次 begin_frame 不是事件接收的初始化前提。
    ///
    /// # 生命周期
    /// 已返回的拥有型快照不随本次切帧改变；调用方负责安排提交和查询的相对时机。
    ///
    /// # 参考
    /// 参见 [InputManager::frame_snapshot()](super::InputManager::frame_snapshot)、[InputManager::drain_events()](super::InputManager::drain_events)、[InputManager::drain_event_records()](super::InputManager::drain_event_records)。
    /// 支持帧状态的实现应清除瞬时边沿与增量，同时保留按住和连接状态。
    fn begin_frame(&self) {}
    /// 将一个拥有型输入事件交给实现处理。
    ///
    /// # 使用时机与职责
    /// 设备或宿主适配先构造共享事件，再同步调用本兼容入口；默认体委托必需的有类型准入方法。
    ///
    /// # 参数
    /// event：本次提交的事件及其拥有型载荷，所有权交给实现。
    ///
    /// # 返回
    /// 兼容入口不返回值并委托 submit_event_with_admission；需要准入结果的宿主使用有类型回执。
    ///
    /// # 约束
    /// 本 trait 不提供数值验证、合并或保留策略。DefaultInputManager 在同一锁内先验证手柄代际，拒绝时不改变任何输入保留器；通过后更新状态，按配置录制已准入的原始事件，再更新可合并的即时队列；录制关闭、容量为零或容量淘汰均可能不保留记录。具体宿主适配的验证与 UI 传播顺序按事件分支执行。
    ///
    /// # 生命周期
    /// 字符串等载荷由事件拥有，不借用宿主 ABI 的临时字节视图；实现可以消费或保留数据。
    ///
    /// # 参考
    /// 参见 [InputEvent](super::InputEvent)、[InputManager::drain_events()](super::InputManager::drain_events)、[InputManager::drain_event_records()](super::InputManager::drain_event_records)。
    fn submit_event(&self, event: InputEvent) {
        let _ = self.submit_event_with_admission(event);
    }

    /// Admit and consume through the same implementation as the compatibility unit entry.
    /// Implementations must check a gamepad's producer generation before any state, edge,
    /// frame-event or recording mutation. Generic unstamped gamepad button events reject.
    /// Closed generations remain fenced across frames, drains and focus loss.
    fn submit_event_with_admission(&self, event: InputEvent) -> InputEventAdmission;
    /// 取得指针位置、持续按下按钮和兼容纵向滚动量的基本快照。
    ///
    /// # 使用时机与职责
    /// 在调用时查询实现的基本状态；这是 trait 的必需实现方法。
    ///
    /// # 返回
    /// 拥有型 InputSnapshot；不携带按下/释放边沿、帧编号或扩展设备状态。
    ///
    /// # 约束
    /// 本 trait 不指定按钮向量的顺序、去重规则或并发快照策略。DefaultInputManager 在单次锁定中复制状态，按钮按其有序集合的迭代顺序克隆。
    ///
    /// # 生命周期
    /// 返回值可由调用方保留；后续提交、切帧或排空不会更新这份副本。
    ///
    /// # 参考
    /// 参见 [InputSnapshot](super::InputSnapshot)、[InputManager::frame_snapshot()](super::InputManager::frame_snapshot)。
    fn snapshot(&self) -> InputSnapshot;

    /// 查询一个完整按钮身份是否处于持续按下状态。
    ///
    /// # 参数
    /// button：只在本次调用中借用的按钮身份；物理键码、逻辑键名及手柄设备/按钮身份分别匹配。
    ///
    /// # 返回
    /// 存在相同身份的按下按钮时为 true；不是仅限本帧的按下边沿。
    ///
    /// # 约束
    /// 默认适配取得完整基本快照后在按钮向量中查找。DefaultInputManager 覆写为对内部集合直接查询；脚本输入入口经服务解析调用该查询。接口不保证恒定时间，也不把未连接设备身份自动判为无效。
    ///
    /// # 生命周期
    /// 不持有 button 借用；返回布尔值与临时快照的生命周期独立。
    ///
    /// # 参考
    /// 参见 [InputButton](super::InputButton)、[InputManager::snapshot()](super::InputManager::snapshot)、[ButtonInputState](super::ButtonInputState)。
    fn button_pressed(&self, button: &InputButton) -> bool {
        self.snapshot().pressed_buttons.contains(button)
    }

    /// 取得供帧边沿和扩展输入消费方使用的拥有型快照。
    ///
    /// # 使用时机与职责
    /// 调用方应在约定帧阶段读取；返回类型本身没有帧编号或时效标记。
    ///
    /// # 返回
    /// 拥有型 InputFrameSnapshot。默认适配从一次基本快照复制位置、按下列表和纵向量；其余字段沿用 InputFrameSnapshot::default，包括 Line 单位、false 标志、None 预编辑以及空容器。
    ///
    /// # 约束
    /// 默认适配从按下列表构造按钮状态，去重但不生成按下/释放边沿；纵向量不等于零时合成一条行单位滚轮事件和 [0,纵向量]，不恢复原始滚动单位或历史。非有限量没有在此过滤。DefaultInputManager 另行覆写，在一次锁定中复制完整状态；不能把兼容适配的空扩展字段当作设备能力证明。 比较 == 0.0 同时把正零和负零判为空事件；NaN 会进入非零分支，合成的行单位事件仍携带 NaN。
    ///
    /// # 生命周期
    /// 返回值按值拥有容器与数据，不借用管理器；后续事件、切帧或宿主请求排空不会修改旧副本。
    ///
    /// # 参考
    /// 参见 [InputFrameSnapshot](super::InputFrameSnapshot)、[InputManager::snapshot()](super::InputManager::snapshot)、[ButtonInputState](super::ButtonInputState)、[MouseWheelEvent](super::MouseWheelEvent)。
    /// 返回完整帧视图；仅实现 `snapshot` 的简化管理器会缺少边沿和设备扩展状态。
    fn frame_snapshot(&self) -> InputFrameSnapshot {
        let snapshot = self.snapshot();
        let buttons = ButtonInputState::from_pressed(snapshot.pressed_buttons);
        InputFrameSnapshot {
            cursor_position: snapshot.cursor_position,
            buttons,
            wheel_accumulator: snapshot.wheel_accumulator,
            mouse_wheel_accumulator: [0.0, snapshot.wheel_accumulator],
            mouse_wheel_events: if snapshot.wheel_accumulator == 0.0 {
                Vec::new()
            } else {
                vec![MouseWheelEvent::lines(0.0, snapshot.wheel_accumulator)]
            },
            ..InputFrameSnapshot::default()
        }
    }

    /// 领取待处理的反向 IME 宿主请求。
    ///
    /// # 使用时机与职责
    /// 宿主消费方向调用；读取帧快照中的请求列表不会替代本次领取。
    ///
    /// # 返回
    /// 拥有型 IME 请求向量；默认适配返回空向量。
    ///
    /// # 约束
    /// DefaultInputManager 取走全部待发队列并复位帧请求索引，包括前帧未领取但当前帧视图已隐藏的请求；再次排空不会重复返回已取走项。动态会话另行保留待输出 ABI 页，返回此向量并不证明宿主已执行请求。
    ///
    /// # 生命周期
    /// 返回值归消费方拥有；此前快照中克隆的请求仍保留原值。
    ///
    /// # 参考
    /// 参见 [ImeHostRequest](super::ImeHostRequest)、[InputFrameSnapshot](super::InputFrameSnapshot)、[InputEvent::ImeHostRequest](super::InputEvent::ImeHostRequest)。
    /// 将运行时请求交给宿主执行；排空后本次请求不应再被其他宿主批次发送。
    fn drain_ime_host_requests(&self) -> Vec<ImeHostRequest> {
        Vec::new()
    }

    /// 领取待处理的反向手柄震动请求。
    ///
    /// # 使用时机与职责
    /// 宿主按队列请求执行震动操作；帧快照中的请求是观察副本。
    ///
    /// # 返回
    /// 拥有型、带入队时连接代际的震动请求向量；默认适配返回空向量。
    ///
    /// # 约束
    /// DefaultInputManager 仅在同一状态锁内观察到已连接目标时接收意图，并在入队时固定其代际。排空取走全部待发队列并复位帧请求索引，旧帧请求仍可被领取；取走项不在下一次排空中重复返回。宿主必须核对原代际，不能用当前代际重标。此观察不确认物理设备存在、操作成功或震动完成；动态会话可将原值保留在有界输出页中。
    ///
    /// # 生命周期
    /// 消费方拥有结果；已生成快照与后续请求队列独立。
    ///
    /// # 参考
    /// 参见 [GamepadRumbleRequest](super::GamepadRumbleRequest)、[InputFrameSnapshot](super::InputFrameSnapshot)、[InputEvent::GamepadRumbleRequest](super::InputEvent::GamepadRumbleRequest)。
    fn drain_gamepad_rumble_requests(&self) -> Vec<QueuedGamepadRumbleRequest> {
        Vec::new()
    }

    /// 领取待处理的反向光标宿主请求。
    ///
    /// # 使用时机与职责
    /// 宿主消费显示、抓取、命中测试或位置命令时调用；快照不执行窗口操作。
    ///
    /// # 返回
    /// 拥有型光标请求向量；默认适配返回空向量。
    ///
    /// # 约束
    /// DefaultInputManager 取走全部待发队列并复位帧请求索引，包含旧帧未消费项；取走项不在下一次排空中重复返回。焦点丢失会用释放抓取请求替换其待发光标队列。返回请求不确认窗口操作成功。
    ///
    /// # 生命周期
    /// 结果及旧快照独立拥有请求；消费方负责实际宿主操作。
    ///
    /// # 参考
    /// 参见 [CursorHostRequest](super::CursorHostRequest)、[InputFrameSnapshot](super::InputFrameSnapshot)、[InputEvent::CursorHostRequest](super::InputEvent::CursorHostRequest)。
    fn drain_cursor_host_requests(&self) -> Vec<CursorHostRequest> {
        Vec::new()
    }

    /// 领取并移出实现当前保留的即时输入事件。
    ///
    /// # 使用时机与职责
    /// 这是 trait 的必需实现方法；消费方应按实现的帧与排空规则及时读取。
    ///
    /// # 返回
    /// 拥有型 InputEvent 向量，允许返回空向量。
    ///
    /// # 约束
    /// 即时队列可以合并事件，不保证逐条原始流。DefaultInputManager 只合并相邻光标移动和相邻相对移动；按钮、触摸等事件形成合并边界。排空不清除状态或原始记录，切帧会清除未读即时事件。
    ///
    /// # 生命周期
    /// 取走事件由调用方拥有；不借用队列，也不会使旧快照失效。
    ///
    /// # 参考
    /// 参见 [InputEvent](super::InputEvent)、[InputManager::drain_event_records()](super::InputManager::drain_event_records)、[InputManager::event_queue_status()](super::InputManager::event_queue_status)。
    /// 排空用于即时消费的帧事件；它可能合并相邻移动事件，不适合作无损回放源。
    fn drain_events(&self) -> Vec<InputEvent>;
    /// 领取并移出实现当前保留的原始输入记录。
    ///
    /// # 使用时机与职责
    /// 这是 trait 的必需实现方法；需要录制是否启用和丢弃信息时使用联合排空。
    ///
    /// # 返回
    /// 拥有型 InputEventRecord 向量；关闭录制或没有保留记录时可为空。
    ///
    /// # 约束
    /// DefaultInputManager 按录制 FIFO 顺序取走记录，保持录制配置、周期序号和累计丢弃计数；不清除即时队列或输入状态。原始记录受容量限制，不能由空结果单独判定本帧没有事件或录制完整。
    ///
    /// # 生命周期
    /// 调用方拥有记录及事件载荷；不借用管理器。
    ///
    /// # 参考
    /// 参见 [InputEventRecord](super::InputEventRecord)、[InputManager::drain_event_records_with_status()](super::InputManager::drain_event_records_with_status)、[InputManager::set_event_recording_config()](super::InputManager::set_event_recording_config)。
    fn drain_event_records(&self) -> Vec<InputEventRecord>;

    /// 在同一管理器事务中领取原始记录并取得关联录制状态。
    ///
    /// # 使用时机与职责
    /// 这是 trait 的必需实现方法；录制捕获调用它以避免分别排空和查询造成状态错配。
    ///
    /// # 返回
    /// 二元组：(取走的拥有型原始记录向量，同一事务内取得的录制状态)。
    ///
    /// # 约束
    /// 实现必须将排空与状态获取放在同一事务边界；不能简单拼接两次可被并发提交穿插的独立调用。DefaultInputManager 持一次锁先排空再取状态，因此返回状态的 retained_records 为零，discarded_records 仍是当前启用周期累计值；它不是本次向量长度或本帧损失数。
    ///
    /// # 生命周期
    /// 两个结果均按值返回；不保留管理器借用。
    ///
    /// # 参考
    /// 参见 [InputEventRecord](super::InputEventRecord)、[InputEventRecordingStatus](super::InputEventRecordingStatus)、[InputManager::drain_event_records()](super::InputManager::drain_event_records)。
    /// Drains records and observes their retention status under one manager transaction.
    /// 在同一管理器事务中取得原始记录及丢弃状态，避免将不完整录制当作无损回放。
    fn drain_event_records_with_status(&self)
        -> (Vec<InputEventRecord>, InputEventRecordingStatus);

    /// 向实现提供可选原始事件录制配置。
    ///
    /// # 参数
    /// _config：要提供给实现的录制开关与容量，按值传入。
    ///
    /// # 返回
    /// 不返回值；默认适配忽略配置，不提供启用成功回执。
    ///
    /// # 约束
    /// DefaultInputManager 覆写并应用配置：关闭清空记录与周期统计，关闭后再启用开始新周期；已启用时调整容量保留序号与丢弃统计，并淘汰队首超额项。零容量仍可启用，提交只消耗周期序号并累计丢弃。
    ///
    /// # 生命周期
    /// 默认体不保留参数；具体实现自行持有其配置值。
    ///
    /// # 参考
    /// 参见 [InputEventRecordingConfig](super::InputEventRecordingConfig)、[InputManager::event_recording_status()](super::InputManager::event_recording_status)、[InputEventRecord](super::InputEventRecord)。
    fn set_event_recording_config(&self, _config: InputEventRecordingConfig) {}

    /// 查询可选原始事件录制的当前状态。
    ///
    /// # 返回
    /// 按值返回 InputEventRecordingStatus；默认适配返回该类型的零值默认状态。
    ///
    /// # 约束
    /// 默认状态为关闭、容量零及零计数，不能证明实现具有录制支持。DefaultInputManager 的初始录制器关闭但配置容量为 8192；具体查询返回保留数与当前启用周期累计丢弃数，排空不重置累计丢弃。查询与其他独立调用不组成事务。
    ///
    /// # 生命周期
    /// 返回状态是调用时的值副本，后续配置、提交和排空不会更新它。
    ///
    /// # 参考
    /// 参见 [InputEventRecordingStatus](super::InputEventRecordingStatus)、[InputEventRecordingConfig](super::InputEventRecordingConfig)、[InputManager::drain_event_records_with_status()](super::InputManager::drain_event_records_with_status)。
    fn event_recording_status(&self) -> InputEventRecordingStatus {
        InputEventRecordingStatus::default()
    }

    /// 查询可选即时事件队列的当前统计。
    ///
    /// # 返回
    /// 按值返回 InputEventQueueStatus；默认适配返回零值默认状态。
    ///
    /// # 约束
    /// 默认零值是适配占位，不能证明存在队列或当前已排空。DefaultInputManager 返回即时保留数和当前帧相邻合并计数；排空清事件但保留合并计数，切帧才清合并计数。具体保留数由 usize 转 u32，接口未承诺饱和长度计数。
    ///
    /// # 生命周期
    /// 返回统计不借用队列，后续变化不更新此副本。
    ///
    /// # 参考
    /// 参见 [InputEventQueueStatus](super::InputEventQueueStatus)、[InputManager::drain_events()](super::InputManager::drain_events)、[InputManager::begin_frame()](super::InputManager::begin_frame)。
    fn event_queue_status(&self) -> InputEventQueueStatus {
        InputEventQueueStatus::default()
    }

    /// 尝试取得实现提供的原始事件订阅接收端。
    ///
    /// # 返回
    /// 支持订阅时为拥有型 ChannelReceiver 的 Some；默认适配返回 None。
    ///
    /// # 约束
    /// None 表示此默认体未提供订阅通道；DefaultInputManager 也没有覆写本方法。接口不规定通道容量、丢弃策略、重放、广播或接收者克隆的分发规则，不能将可选签名视为已经支持订阅。接收端的具体行为遵从[ChannelReceiver](crate::core::framework::channel::ChannelReceiver) 别名所指类型及实现。
    ///
    /// # 生命周期
    /// Some 中的接收端按值返回，不借用 &self；通道存活和断开由实现持有的发送端等资源决定。
    ///
    /// # 参考
    /// 参见 [InputEventRecord](super::InputEventRecord)、[InputManager::drain_event_records()](super::InputManager::drain_event_records)。
    fn subscribe_events(&self) -> Option<ChannelReceiver<InputEventRecord>> {
        None
    }
}
