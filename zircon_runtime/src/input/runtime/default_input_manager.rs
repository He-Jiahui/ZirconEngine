//! 把宿主提交的输入事件归入持久状态、帧内窗口和可选原始录制。
//!
//! # 入口与责任
//! [`DefaultInputManager`] 实现 [`InputManagerFacade`]；模块工厂将默认构造值包装为管理器服务，独立构造不会注册服务。事件采样、帧边界选择和宿主请求执行由调用方推进。
//!
//! # 状态与生命周期
//! 本实现的每次状态访问取得同一个状态锁。帧开始清理边沿、增量和通知，并更新请求视图起点；待交付请求与原始录制有各自的跨帧保留规则。返回快照和排空结果独立拥有数据。
//!
//! # 参考
//! 参见 [`InputState`]、[`InputFrameSnapshot`] 与 [`InputManagerFacade::submit_event`]。
use std::sync::{Mutex, MutexGuard};

use crate::core::framework::input::{
    GamepadInputRejection, InputBatchLimits, InputBatchRejection, InputBatchSize,
    InputEventAdmission, InputManager as InputManagerFacade, QueuedGamepadRumbleRequest,
};

use crate::input::{
    CursorHostRequest, GamepadAxisSettings, GamepadButtonAxisSettings, GamepadButtonSettings,
    ImeEvent, ImeHostRequest, InputButton, InputEvent, InputEventQueueStatus, InputEventRecord,
    InputEventRecordingConfig, InputEventRecordingStatus, InputFrameSnapshot, InputSnapshot,
    MouseScrollUnit, MouseWheelEvent, TouchPhase, TouchPoint,
};

use super::InputState;

#[path = "default_input_manager/batch.rs"]
mod batch;

#[path = "default_input_manager/apply_event.rs"]
mod apply_event;

use apply_event::apply_input_event;

/// 维护一份可通过共享引用访问的运行时输入状态。
///
/// # 构造与使用时机
/// 派生的 `Default` 创建空按钮、触摸、连接、请求与事件集合，光标和增量为零，IME 关闭，原始录制默认关闭。可在首次帧边界之前提交事件；服务工厂可另将实例包装进注册表。
///
/// # 访问与生命周期
/// [`InputManagerFacade`] 的每个具体入口独立取得状态锁，操作在持锁期间同步完成；多个调用之间可以插入其它线程的提交，不构成跨调用事务。锁中毒时取回 guard 继续访问，不清除中毒标志或修复部分更新。快照与排空结果不保留锁内借用。
///
/// # 接口边界
/// 本类型提供扩展帧快照、请求排空与录制支持；未覆盖的事件订阅沿用 trait 默认的 `None`。这些保留与锁策略只属于本实现。
///
/// # 参考
/// 参见 [`InputManagerFacade`]、`InputState`、[`InputSnapshot`] 与 [`InputFrameSnapshot`]。
#[derive(Debug, Default)]
pub struct DefaultInputManager {
    /// 同一实例全部输入状态、请求队列和两种事件缓冲的唯一互斥保护。
    /// 快照复制、排空、配置及提交共用此锁；guard 的存续时间就是这次访问的持锁范围。
    /// 中毒恢复只取回内部状态，不验证其一致性。
    state: Mutex<InputState>,
}

impl DefaultInputManager {
    /// 取得借用本管理器的输入状态 guard。
    ///
    /// # 调用时机与约束
    /// 在本实现的每次状态访问开始时调用；取得锁可能等待其它访问结束。持有本锁的调用路径不应再次获取同一锁。
    ///
    /// # 返回与生命周期
    /// 返回 `MutexGuard<'_, InputState>`，借用 `self`，释放 guard 时解除锁。中毒时从错误中取回 guard，不把中毒作为错误返回；该路径不清除中毒标志，也不检查或回滚已有状态。
    ///
    /// # 参考
    /// 参见 [`DefaultInputManager`] 与 [`InputState`]。
    fn lock_state(&self) -> MutexGuard<'_, InputState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl InputManagerFacade for DefaultInputManager {
    /// 在调用方选定的帧边界清理当前瞬时窗口。
    ///
    /// # 调用时机与生命周期
    /// 默认构造后即可提交；本方法不是初始化前提。调用方须先完成需要旧窗口的消费，再开启新窗口；动态会话在关卡 tick 后调用，回放入口可在提交一批记录前调用。
    ///
    /// # 状态与返回
    /// 在一次状态锁内清除按钮边沿、瞬时事件及合并数、滚轮和鼠标增量、IME 提交与删除通知、轴变化、窗口和拖放通知。按住、光标位置及内外标志、触摸、连接、轴与按钮幅值、IME 开关和预编辑继续保留。三类宿主请求只把本帧视图起点移到队尾，未交付请求和原始记录继续存在；返回 `()`。
    ///
    /// # 重复调用
    /// 每次调用都清理已有窗口，重复调用会丢弃尚未消费的瞬时信息；后续快照不改变之前返回的副本。
    ///
    /// # 回放入口的具体边界
    /// 本实例作为回放目标时，[`crate::input::InputReplayCursor::replay_next_frame`] 在提交该批事件前调用帧开始入口；
    /// [`crate::input::InputReplayCursor::submit_next_frame_events`] 则追加到已有窗口，不调用帧开始入口。
    ///
    /// # 参考
    /// 参见 [`InputManagerFacade::frame_snapshot`]、[`InputManagerFacade::drain_events`] 与 `InputState`。
    fn begin_frame(&self) {
        let mut state = self.lock_state();
        state.buttons.clear_transitions();
        state.frame_events.begin_frame();
        state.wheel_accumulator = 0.0;
        state.mouse_wheel_accumulator = [0.0, 0.0];
        state.mouse_wheel_unit = MouseScrollUnit::Line;
        state.mouse_wheel_events.clear();
        state.mouse_motion_accumulator = [0.0, 0.0];
        // 只移动待交付队列的本帧视图起点；旧请求留在队列内，由完整排空接口交给宿主。
        state.cursor_host_requests_frame_start = state.cursor_host_requests.len();
        state.ime_commits.clear();
        state.ime_delete_surrounding.clear();
        state.ime_host_requests_frame_start = state.ime_host_requests.len();
        state.clear_gamepad_axis_transitions();
        state.gamepad_rumble_requests_frame_start = state.gamepad_rumble_requests.len();
        state.window_status_events.clear();
        state.file_drag_drop_events.clear();
    }

    /// 同步消费一个输入事件，更新本实例的状态与事件保留器。
    ///
    /// # 参数
    /// `event` 按值传入；状态更新先借用其内容，必要时复制字符串或请求，随后原事件移入瞬时缓冲。普通坐标、滚轮、触摸及 IME 内容在这里不做统一有限性、文本边界或阶段历史验证；手柄幅值另经默认处理设置筛选。手柄样本必须携带生产端捕获的代际并匹配已打开的连接；旧的未连接样本行为已显式迁移。
    ///
    /// # 顺序与生命周期
    /// 整次调用持有一个状态锁：先更新持久或帧状态，再尝试记录未合并原事件，最后进入可合并相邻指针样本的瞬时缓冲。默认构造后即可调用；事件进入哪个窗口由调用方的帧边界决定。焦点丢失与断连在各自分支同步清理状态。
    ///
    /// # 返回与边界
    /// 返回代际准入回执；拒绝发生在任何状态、边沿、帧事件或录制变更之前。手柄幅值被过滤时可能不改变相应状态，原事件仍经过录制尝试与瞬时缓冲。请求只入队，窗口通知只保存，均不执行宿主动作；本调用也不表明 UI 已处理事件。
    ///
    /// # 参考
    /// 参见 [`InputEvent`]、[`GamepadAxisSettings::process_value`]、[`GamepadButtonAxisSettings::process_value`] 与 [`InputManagerFacade::drain_events`]。
    fn submit_event_with_admission(&self, event: InputEvent) -> InputEventAdmission {
        let mut state = self.lock_state();
        let admission = gamepad_input_admission(&state, &event);
        if matches!(admission, InputEventAdmission::Rejected(_)) {
            return admission;
        }
        apply_input_event(&mut state, event);
        InputEventAdmission::Accepted
    }

    fn submit_batch_with_admission(
        &self,
        events: Vec<InputEvent>,
        limits: InputBatchLimits,
    ) -> Result<InputBatchSize, InputBatchRejection> {
        let mut state = self.lock_state();
        let size = batch::preflight(&state, &events, limits)?;
        for event in events {
            apply_input_event(&mut state, event);
        }
        Ok(size)
    }

    /// 在一次状态锁内复制旧式输入视图。
    ///
    /// # 返回
    /// 返回拥有光标坐标、当前按住按钮和兼容纵向滚轮标量的 [`InputSnapshot`]；不包含边沿或扩展设备状态。按住列表来自内部有序集合的迭代。
    ///
    /// # 生命周期
    /// 查询不清理或排空状态；复制完成后释放锁，旧快照不随之后提交或帧清理而改变。需要边沿、设备与通知时使用完整帧入口。
    ///
    /// # 参考
    /// 参见 [`InputManagerFacade::frame_snapshot`] 与 [`InputSnapshot`]。
    fn snapshot(&self) -> InputSnapshot {
        let state = self.lock_state();
        InputSnapshot {
            cursor_position: state.cursor_position,
            pressed_buttons: state.buttons.pressed_inputs(),
            wheel_accumulator: state.wheel_accumulator,
        }
    }

    /// 直接在持锁的按住集合中查询一个按钮身份。
    ///
    /// # 参数
    /// `button` 只在调用期间共享借用；按 [`InputButton`] 的完整身份匹配，逻辑键名与物理键码分别查询，手柄须带设备和按钮。
    ///
    /// # 返回与生命周期
    /// 返回查询时该身份是否按住，不代表本帧边沿或设备仍连接。查询不修改状态，也不复制整个快照；这里覆盖 trait 通过旧式快照查找的默认路径。返回前释放状态锁。
    ///
    /// # 参考
    /// 参见 [`InputManagerFacade::button_pressed`]、[`InputButton`] 与 [`InputManagerFacade::snapshot`]。
    fn button_pressed(&self, button: &InputButton) -> bool {
        self.lock_state().buttons.pressed(button)
    }

    /// 在一次状态锁内复制当前完整输入视图。
    ///
    /// # 返回与窗口
    /// 返回拥有所有集合的 [`InputFrameSnapshot`]。持久字段与当前窗口的边沿、增量、通知一起复制；三类宿主请求仅复制各自视图起点之后的后缀，不含仍待交付的旧帧前缀。快照不代替排空。
    ///
    /// # 顺序
    /// 触摸、连接、轴和按钮幅值从各自有序容器迭代投影；轴变化直接复制现有向量。普通轴采样保持首次变化的槽位，断连可追加重复控件，不能把变化向量描述成全局唯一或按键排序。
    ///
    /// # 生命周期
    /// 查询不消耗任何状态，复制完成后释放锁。旧快照不借用管理器，后续提交、排空或帧清理不会修改它；它没有独立的帧编号或新鲜度保证。
    ///
    /// # 参考
    /// 参见 [`InputFrameSnapshot`]、`InputState::gamepad_axis_states` 与 [`InputManagerFacade::begin_frame`]。
    fn frame_snapshot(&self) -> InputFrameSnapshot {
        let state = self.lock_state();
        InputFrameSnapshot {
            cursor_position: state.cursor_position,
            cursor_inside_window: state.cursor_inside_window,
            cursor_host_requests: state.cursor_host_requests
                [state.cursor_host_requests_frame_start..]
                .to_vec(),
            buttons: state.buttons.clone(),
            wheel_accumulator: state.wheel_accumulator,
            mouse_wheel_accumulator: state.mouse_wheel_accumulator,
            mouse_wheel_unit: state.mouse_wheel_unit,
            mouse_wheel_events: state.mouse_wheel_events.clone(),
            mouse_motion_accumulator: state.mouse_motion_accumulator,
            active_touches: state.active_touches.values().copied().collect(),
            connected_gamepads: state.connected_gamepads.iter().copied().collect(),
            gamepad_axes: state.gamepad_axis_states(),
            gamepad_axis_transitions: state.gamepad_axis_transitions.clone(),
            gamepad_button_values: state.gamepad_button_value_states(),
            gamepad_rumble_requests: state.gamepad_rumble_requests
                [state.gamepad_rumble_requests_frame_start..]
                .iter()
                .map(QueuedGamepadRumbleRequest::request)
                .collect(),
            ime_enabled: state.ime_enabled,
            ime_preedit: state.ime_preedit.clone(),
            ime_commits: state.ime_commits.clone(),
            ime_delete_surrounding: state.ime_delete_surrounding.clone(),
            ime_host_requests: state.ime_host_requests[state.ime_host_requests_frame_start..]
                .to_vec(),
            window_status_events: state.window_status_events.clone(),
            file_drag_drop_events: state.file_drag_drop_events.clone(),
        }
    }

    /// 排空全部待交付 IME 宿主请求并转移所有权。
    ///
    /// # 返回
    /// 返回队列顺序的 [`ImeHostRequest`] 向量，包括快照已隐藏的旧帧请求；队列与本帧视图起点同步重置为空和零。没有后续提交时再次调用返回空向量。
    ///
    /// # 调用时机与生命周期
    /// 宿主消费路径调用；取出只完成管理器内排队交接，不证明平台已执行。焦点丢失会先撤销原队列并改为 Disable 请求。返回向量在释放状态锁后独立拥有内容。
    ///
    /// # 参考
    /// 参见 [`InputManagerFacade::frame_snapshot`] 与 [`InputManagerFacade::begin_frame`]。
    fn drain_ime_host_requests(&self) -> Vec<ImeHostRequest> {
        let mut state = self.lock_state();
        state.ime_host_requests_frame_start = 0;
        std::mem::take(&mut state.ime_host_requests)
    }

    /// 排空全部待交付手柄震动请求。
    ///
    /// # 返回
    /// 按保留顺序转移全部请求，包括已不在本帧快照后缀内的旧请求；队列变空，视图起点归零。返回的拥有式向量不是设备执行回执。
    ///
    /// # 调用时机与生命周期
    /// 宿主请求收集阶段使用；单次调用持有状态锁，返回后释放锁。帧边界保留请求，当前失焦与断连清理也没有取消此队列；实际设备处理由宿主完成。
    ///
    /// # 参考
    /// 参见 [`InputManagerFacade::drain_gamepad_rumble_requests`] 与 [`InputFrameSnapshot`]。
    fn drain_gamepad_rumble_requests(&self) -> Vec<QueuedGamepadRumbleRequest> {
        let mut state = self.lock_state();
        state.gamepad_rumble_requests_frame_start = 0;
        std::mem::take(&mut state.gamepad_rumble_requests)
    }

    /// 排空全部待交付窗口光标请求。
    ///
    /// # 返回
    /// 按队列顺序转移全部 [`CursorHostRequest`]，包括本帧快照隐藏的旧请求；队列变空，视图起点归零。无后续请求时再次排空为空。
    ///
    /// # 调用时机与生命周期
    /// 宿主收集反向请求时调用；取出不表明窗口已应用。焦点丢失会撤销原队列并保留解除捕获请求；帧边界只更新快照起点。向量在释放状态锁后独立拥有内容。
    ///
    /// # 参考
    /// 参见 [`InputManagerFacade::frame_snapshot`] 与 [`InputManagerFacade::begin_frame`]。
    fn drain_cursor_host_requests(&self) -> Vec<CursorHostRequest> {
        let mut state = self.lock_state();
        state.cursor_host_requests_frame_start = 0;
        std::mem::take(&mut state.cursor_host_requests)
    }

    /// 转移当前窗口尚未消费的瞬时事件。
    ///
    /// # 返回
    /// 返回缓冲顺序的 [`InputEvent`] 向量；相邻同类光标位置或鼠标位移可能已经合并。缓冲随即为空，合并计数保留到下次帧边界，返回结果不适合作为无损原始回放源。
    ///
    /// # 生命周期
    /// 单次调用在状态锁内排空；后续提交可继续进入同一窗口，下一次帧边界会丢弃尚未取出的事件。此操作不排空独立原始记录，也不撤销设备状态或宿主请求。
    ///
    /// # 参考
    /// 参见 [`InputManagerFacade::drain_event_records`] 与 [`InputManagerFacade::event_queue_status`]。
    fn drain_events(&self) -> Vec<InputEvent> {
        let mut state = self.lock_state();
        state.frame_events.drain()
    }

    /// 转移原始录制器当前保留的全部记录。
    ///
    /// # 返回
    /// 返回 [`InputEventRecord`] 的 FIFO 向量；未按时间戳或序号重新排序。为空不能证明此前没有提交或容量丢弃；关闭录制时也可能为空。
    ///
    /// # 生命周期
    /// 持锁排空后保留配置、序号游标和累计丢弃数，不影响瞬时缓冲或帧状态。需要与这些记录同一次事务的状态时使用联合入口，不能把稍后的独立查询当作同一观察。
    ///
    /// # 参考
    /// 参见 [`InputManagerFacade::drain_event_records_with_status`] 与 [`InputManagerFacade::event_recording_status`]。
    fn drain_event_records(&self) -> Vec<InputEventRecord> {
        let mut state = self.lock_state();
        state.event_recorder.drain()
    }

    /// 在同一次状态锁访问内排空记录并读取排空后的录制状态。
    ///
    /// # 返回与观察时点
    /// 返回拥有式记录向量和 [`InputEventRecordingStatus`]；保留数是排空后的零，取出的数量应读向量长度，配置和当前周期的累计丢弃数保持原值。状态不保证所有输入来源或完整历史已经被记录。
    ///
    /// # 调用时机与生命周期
    /// 捕获一批录制时使用；记录排空与状态读取之间不会插入本管理器其它持锁入口。返回后释放锁，之后提交或配置更新不改变已返回的记录与状态。
    ///
    /// # 参考
    /// 参见 [`InputManagerFacade::drain_event_records_with_status`] 与 [`crate::input::InputRecordingFrame::capture_from_manager`]。
    fn drain_event_records_with_status(
        &self,
    ) -> (Vec<InputEventRecord>, InputEventRecordingStatus) {
        let mut state = self.lock_state();
        let records = state.event_recorder.drain();
        (records, state.event_recorder.status())
    }

    /// 在状态锁内把新的原始录制配置交给记录器应用。
    ///
    /// # 参数
    /// `config` 按值传入，容量以记录条目计，允许零；不限制瞬时事件缓冲或事件负载总字节数。
    ///
    /// # 状态与返回
    /// 返回 `()`，同步保存配置并完成必要裁剪。关闭或从关闭切换为启用会清空周期记录、序号和丢弃数；持续启用保留周期，缩容淘汰队首旧记录并累计丢弃，增容保留记录。启用且零容量的后续提交推进序号并统计丢弃，但不保留记录。
    ///
    /// # 生命周期
    /// 不更改设备状态、帧窗口或宿主请求。每次配置更新与提交由同一个状态锁串行化。
    ///
    /// # 参考
    /// 参见 [`InputEventRecordingConfig`] 与 [`InputManagerFacade::event_recording_status`]。
    fn set_event_recording_config(&self, config: InputEventRecordingConfig) {
        self.lock_state().event_recorder.configure(config);
    }

    /// 在状态锁内读取当前录制周期的配置和计数。
    ///
    /// # 返回
    /// 返回拥有式 [`InputEventRecordingStatus`]；配置来自当前记录器，保留数是此刻 FIFO 长度，丢弃数是当前周期累计饱和值。独立查询不与另一次记录排空组成同一事务。
    ///
    /// # 生命周期
    /// 查询不排空或重置周期，返回值只描述此刻；后续提交、排空或配置变更不更新它。需要同次捕获时使用联合入口。
    ///
    /// # 参考
    /// 参见 [`InputManagerFacade::drain_event_records_with_status`] 与 [`InputEventRecordingStatus`]。
    fn event_recording_status(&self) -> InputEventRecordingStatus {
        self.lock_state().event_recorder.status()
    }

    /// 在状态锁内观察瞬时缓冲的保留与合并计数。
    ///
    /// # 返回
    /// 返回 [`InputEventQueueStatus`]；保留数是向量长度转换为 `u32`，超出表示范围会截断。合并数是构造或最近帧边界以来的累计饱和值，排空不清零；两数不能作为整个历史提交总数。
    ///
    /// # 生命周期
    /// 查询不消耗事件，后续提交、排空或帧边界可能改变缓冲。原始录制容量和丢弃周期与这里独立。
    ///
    /// # 参考
    /// 参见 [`InputManagerFacade::drain_events`] 与 [`InputManagerFacade::begin_frame`]。
    fn event_queue_status(&self) -> InputEventQueueStatus {
        self.lock_state().frame_events.status()
    }
}

#[cfg(test)]
#[path = "default_input_manager/tests/cases.rs"]
mod tests;
