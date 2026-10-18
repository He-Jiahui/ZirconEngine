//! 保存默认输入管理器的持久样本、当前窗口和待交付请求。
//!
//! # 所有权与访问
//! [`InputState`] 自身不持有锁；默认管理器在其状态锁内借用，测试可独占构造和操作。方法以 `&mut self` 更新，返回的设备视图复制键和值。
//!
//! # 窗口与责任
//! 普通轴采样通过索引保留首次变化槽位；失焦与断连辅助有各自的追加规则。帧边界和队列排空由 [`super::DefaultInputManager`] 决定，本模块不驱动设备、平台请求或 UI。
//!
//! # 参考
//! 参见 [`InputState::record_gamepad_axis_transition`]、[`InputState::clear_active_input_for_focus_loss`] 与 [`InputState::append_disconnected_axis_transitions`]。
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::num::NonZeroU64;

use crate::core::framework::input::QueuedGamepadRumbleRequest;
use crate::input::{
    ButtonInputState, CursorGrabMode, CursorHostRequest, FileDragDropEvent, GamepadAxis,
    GamepadAxisState, GamepadAxisTransition, GamepadButton, GamepadButtonValueState, GamepadId,
    ImeDeleteSurrounding, ImeHostRequest, ImePreedit, InputButton, MouseScrollUnit,
    MouseWheelEvent, TouchPoint, WindowStatusEvent,
};

use super::event_buffer::{FrameEventBuffer, InputEventRecorder};

/// 把当前输入样本与不同保留周期的集合放在同一份状态中。
///
/// # 构造与生命周期
/// `Default` 允许立即接收输入：所有集合为空，坐标和增量为零，滚轮单位为行，IME 和原始录制关闭。持久样本跨帧保留；瞬时通知与边沿由管理器清理，待交付请求另由排空或失焦替换。
///
/// # 访问与不变量
/// 本类型没有内部同步。默认管理器持锁后取得可变或共享借用；轴变化向量与首槽索引必须同步维护，三类请求的视图起点须不超过对应队列长度。直接改写公开于 crate 的集合时也须维持这些关系。
///
/// # 参考
/// 参见 [`super::DefaultInputManager`]、[`FrameEventBuffer`] 与 [`InputEventRecorder`]。
#[derive(Debug)]
pub(crate) struct InputState {
    pub(crate) cursor_position: [f32; 2],
    pub(crate) cursor_inside_window: bool,
    /// 全部待交付光标请求；帧边界保留旧前缀，失焦改为解除捕获请求。
    /// 快照只读起点后的后缀，排空取走整队；保存请求不代表窗口已执行。
    pub(crate) cursor_host_requests: Vec<CursorHostRequest>,
    /// 当前帧快照后缀的起点，须不超过光标请求队列长度。
    /// 帧开始设为队尾；整队排空和失焦替换时归零。
    pub(crate) cursor_host_requests_frame_start: usize,
    pub(crate) buttons: ButtonInputState<InputButton>,
    pub(crate) wheel_accumulator: f32,
    pub(crate) mouse_wheel_accumulator: [f32; 2],
    pub(crate) mouse_wheel_unit: MouseScrollUnit,
    pub(crate) mouse_wheel_events: Vec<MouseWheelEvent>,
    pub(crate) mouse_motion_accumulator: [f32; 2],
    pub(crate) active_touches: BTreeMap<u64, TouchPoint>,
    pub(crate) connected_gamepads: BTreeSet<GamepadId>,
    /// Latest authoritative incarnation and its open/closed tombstone, retained across frames.
    pub(crate) gamepad_generations: BTreeMap<GamepadId, (NonZeroU64, bool)>,
    pub(crate) gamepad_axes: BTreeMap<(GamepadId, GamepadAxis), f32>,
    /// 当前窗口的轴变化记录；普通采样更新首次索引槽位，断连辅助可追加同身份记录。
    /// 向量保留这些槽位和追加位置，不保证全局唯一或按键排序。
    pub(crate) gamepad_axis_transitions: Vec<GamepadAxisTransition>,
    /// 设备与轴身份到已有首个变化槽位的索引。
    /// 索引与向量在帧边界一起清空；新增后缀用 `or_insert` 保留旧槽位，索引不是向量唯一性的证明。
    gamepad_axis_transition_indices: HashMap<(GamepadId, GamepadAxis), usize>,
    pub(crate) gamepad_button_values: BTreeMap<(GamepadId, GamepadButton), f32>,
    /// Host-delivery FIFO: generations are captured at acceptance and never rebound on drain.
    pub(crate) gamepad_rumble_requests: Vec<QueuedGamepadRumbleRequest>,
    /// 本帧震动请求快照的后缀起点，须不超过队列长度。
    /// 帧边界设为队尾，排空整队时归零；当前失焦与断连清理均不改此队列。
    pub(crate) gamepad_rumble_requests_frame_start: usize,
    pub(crate) ime_enabled: bool,
    pub(crate) ime_preedit: Option<ImePreedit>,
    pub(crate) ime_commits: Vec<String>,
    pub(crate) ime_delete_surrounding: Vec<ImeDeleteSurrounding>,
    pub(crate) ime_host_requests: Vec<ImeHostRequest>,
    /// 本帧 IME 请求快照的后缀起点，须不超过队列长度。
    /// 帧边界设为队尾；整队排空或失焦替换为 Disable 时归零。
    pub(crate) ime_host_requests_frame_start: usize,
    pub(crate) window_status_events: Vec<WindowStatusEvent>,
    pub(crate) file_drag_drop_events: Vec<FileDragDropEvent>,
    /// 可合并相邻指针样本的瞬时缓冲，帧边界清除尚未消费的事件及合并计数。
    /// 它不保存设备当前状态，也不是原始录制的副本。
    pub(super) frame_events: FrameEventBuffer,
    /// 可选、按条目容量保留的原始事件 FIFO；记录尝试位于瞬时事件合并之前。
    /// 配置周期和排空独立于帧边界，不能用它的状态替代瞬时缓冲计数。
    pub(super) event_recorder: InputEventRecorder,
}

impl Default for InputState {
    /// 构造可立即由调用方使用的空输入状态。
    ///
    /// # 返回与生命周期
    /// 返回独立拥有全部集合的 `Self`：视图起点为零，轴变化与索引同时为空，IME 关闭且无预编辑，帧缓冲为空，原始录制采用其默认关闭配置。构造不采样设备或注册服务。
    ///
    /// # 参考
    /// 参见 [`InputState`]、[`FrameEventBuffer`] 与 [`InputEventRecorder`]。
    fn default() -> Self {
        Self {
            cursor_position: [0.0, 0.0],
            cursor_inside_window: false,
            cursor_host_requests: Vec::new(),
            cursor_host_requests_frame_start: 0,
            buttons: ButtonInputState::default(),
            wheel_accumulator: 0.0,
            mouse_wheel_accumulator: [0.0, 0.0],
            mouse_wheel_unit: MouseScrollUnit::Line,
            mouse_wheel_events: Vec::new(),
            mouse_motion_accumulator: [0.0, 0.0],
            active_touches: BTreeMap::new(),
            connected_gamepads: BTreeSet::new(),
            gamepad_generations: BTreeMap::new(),
            gamepad_axes: BTreeMap::new(),
            gamepad_axis_transitions: Vec::new(),
            gamepad_axis_transition_indices: HashMap::new(),
            gamepad_button_values: BTreeMap::new(),
            gamepad_rumble_requests: Vec::new(),
            gamepad_rumble_requests_frame_start: 0,
            ime_enabled: false,
            ime_preedit: None,
            ime_commits: Vec::new(),
            ime_delete_surrounding: Vec::new(),
            ime_host_requests: Vec::new(),
            ime_host_requests_frame_start: 0,
            window_status_events: Vec::new(),
            file_drag_drop_events: Vec::new(),
            frame_events: FrameEventBuffer::default(),
            event_recorder: InputEventRecorder::default(),
        }
    }
}

impl InputState {
    /// 清空当前轴变化窗口及其首槽索引。
    ///
    /// # 调用时机与状态
    /// 帧开始时由管理器调用；一次独占借用内同时清空两个容器，避免后续采样指向旧槽位。该方法不清除当前轴样本，也不处理连接或按钮。
    ///
    /// # 返回
    /// 返回 `()`；后续普通采样会重新建立首次变化槽位。
    ///
    /// # 参考
    /// 参见 [`InputState::record_gamepad_axis_transition`]。
    pub(crate) fn clear_gamepad_axis_transitions(&mut self) {
        self.gamepad_axis_transitions.clear();
        self.gamepad_axis_transition_indices.clear();
    }

    /// 按完整轴身份记录普通采样的起始值和最终端点。
    ///
    /// # 参数与前提
    /// `gamepad` 与 `axis` 组成键；`previous_value` 只用于首次创建槽位，`value` 是本次最终端点。调用方负责幅值处理和是否变化的判断，本方法不验证连接、有限性或两值相等。
    ///
    /// # 状态与顺序
    /// 已有索引时只改该槽位的 `value`，保持原 `previous_value` 和位置；否则在尾部追加并索引。断连辅助可能另外追加同身份项，因此索引只指向首槽，不能证明整个向量唯一。
    ///
    /// # 返回与生命周期
    /// 返回 `()`，只在本次独占借用内修改。索引须指向有效槽位；帧清理应同时清空向量和索引。
    ///
    /// # 参考
    /// 参见 [`InputState::clear_gamepad_axis_transitions`] 与 [`GamepadAxisTransition`]。
    pub(crate) fn record_gamepad_axis_transition(
        &mut self,
        gamepad: GamepadId,
        axis: GamepadAxis,
        previous_value: f32,
        value: f32,
    ) {
        let key = (gamepad, axis);
        if let Some(index) = self.gamepad_axis_transition_indices.get(&key).copied() {
            self.gamepad_axis_transitions[index].value = value;
            return;
        }

        let index = self.gamepad_axis_transitions.len();
        self.gamepad_axis_transitions.push(GamepadAxisTransition {
            gamepad,
            axis,
            previous_value,
            value,
        });
        self.gamepad_axis_transition_indices.insert(key, index);
    }

    /// Losing interactive focus ends active controls but does not disconnect attached devices.
    /// 在失去交互焦点时同步结束当前活动控件。
    ///
    /// # 状态与顺序
    /// 先释放所有按住按钮，清除滚轮、鼠标位移、触点和按钮幅值，再关闭 IME 并撤销预编辑。已有 IME 与光标待交付队列被替换，分别保留 Disable 和解除捕获请求；起点归零。最后取走轴缓存，调整已有变化或追加归零端点，并索引新增后缀。
    ///
    /// # 保留与生命周期
    /// 连接记录、光标位置及内外标志、IME 提交和删除通知、震动请求、窗口与拖放通知继续存在。按钮刚按下边沿不会因释放而删去，可同时有本帧按下和释放边沿。反复失焦重新替换恢复请求；这里不执行平台动作。
    ///
    /// # 返回与边界
    /// 返回 `()`；独占借用不构成平台完成回执。已有重复轴身份只按辅助函数的首次匹配规则调整，不能承诺每个旧槽位都归零。
    ///
    /// # 参考
    /// 参见 [`reset_gamepad_axes_for_focus_loss`]、[`InputState::index_appended_gamepad_axis_transitions`] 与 [`ButtonInputState::release_all`]。
    pub(crate) fn clear_active_input_for_focus_loss(&mut self) {
        self.buttons.release_all();
        self.wheel_accumulator = 0.0;
        self.mouse_wheel_accumulator = [0.0, 0.0];
        self.mouse_wheel_unit = MouseScrollUnit::Line;
        self.mouse_wheel_events.clear();
        self.mouse_motion_accumulator = [0.0, 0.0];
        self.active_touches.clear();
        self.gamepad_button_values.clear();
        self.ime_enabled = false;
        self.ime_preedit = None;
        self.ime_host_requests.clear();
        self.ime_host_requests_frame_start = 0;
        self.ime_host_requests.push(ImeHostRequest::Disable);
        self.cursor_host_requests.clear();
        self.cursor_host_requests_frame_start = 0;
        self.cursor_host_requests
            .push(CursorHostRequest::set_grab_mode(CursorGrabMode::None));

        let appended_start = self.gamepad_axis_transitions.len();
        reset_gamepad_axes_for_focus_loss(
            std::mem::take(&mut self.gamepad_axes),
            &mut self.gamepad_axis_transitions,
        );
        self.index_appended_gamepad_axis_transitions(appended_start);
    }

    pub(crate) fn forget_gamepad_axis_transition_indices(&mut self, gamepad: GamepadId) {
        // Keep prior routing-aggregate records; a new incarnation gets a new last source item.
        self.gamepad_axis_transition_indices
            .retain(|(id, _), _| *id != gamepad);
    }

    /// 从删除前的轴缓存追加指定设备的归零变化并索引新后缀。
    ///
    /// # 参数与时机
    /// `disconnected_gamepad` 是完整设备身份；管理器在移除该设备轴和按钮幅值前调用。辅助函数只读取缓存，因此本方法调用后缓存仍在。
    ///
    /// # 状态与顺序
    /// 记录追加前向量长度，再追加该设备当前非零样本到零的记录。既有变化不改写，可能出现重复身份；新增索引只补缺失键，不覆盖已有首槽。
    ///
    /// # 返回与生命周期
    /// 返回 `()`；不移除连接记录、释放按钮或取消请求，后续清理由调用方执行。
    ///
    /// # 参考
    /// 参见 [`self::append_disconnected_axis_transitions`] 与 [`InputState::index_appended_gamepad_axis_transitions`]。
    pub(crate) fn append_disconnected_axis_transitions(&mut self, disconnected_gamepad: GamepadId) {
        let appended_start = self.gamepad_axis_transitions.len();
        append_disconnected_axis_transitions(
            &self.gamepad_axes,
            &mut self.gamepad_axis_transitions,
            disconnected_gamepad,
        );
        self.index_appended_gamepad_axis_transitions(appended_start);
    }

    /// 把当前轴缓存复制为拥有式样本列表。
    ///
    /// # 返回
    /// 返回 [`GamepadAxisState`] 向量，按内部 `BTreeMap` 当前键迭代；含已接受的零样本，不要求设备出现在连接集合。该顺序描述只属于此投影。
    ///
    /// # 生命周期
    /// 仅共享借用，不清理缓存或变化；返回后不保留对状态的借用，之后的提交与清理不改变旧列表。
    ///
    /// # 参考
    /// 参见 [`InputState::gamepad_button_value_states`]。
    pub(crate) fn gamepad_axis_states(&self) -> Vec<GamepadAxisState> {
        self.gamepad_axes
            .iter()
            .map(|((gamepad, axis), value)| GamepadAxisState {
                gamepad: *gamepad,
                axis: *axis,
                value: *value,
            })
            .collect()
    }

    /// 把当前处理后按钮幅值复制为拥有式样本列表。
    ///
    /// # 返回
    /// 返回 [`GamepadButtonValueState`] 向量，按内部 `BTreeMap` 当前键迭代；幅值与按住集合是不同状态，本方法不推导按钮是否按下，也不验证设备连接。
    ///
    /// # 生命周期
    /// 仅共享借用，键和值逐项复制；不排空缓存，返回后不借用原集合。
    ///
    /// # 参考
    /// 参见 [`InputState::gamepad_axis_states`]。
    pub(crate) fn gamepad_button_value_states(&self) -> Vec<GamepadButtonValueState> {
        self.gamepad_button_values
            .iter()
            .map(|((gamepad, button), value)| GamepadButtonValueState {
                gamepad: *gamepad,
                button: *button,
                value: *value,
            })
            .collect()
    }

    /// 只为追加后缀补齐尚未索引的轴身份。
    ///
    /// # 参数与前提
    /// `appended_start` 是调用方在追加前取得的向量长度；应位于当前长度以内。该方法不验证历史索引或修复直接改写过的向量。
    ///
    /// # 状态与返回
    /// 按后缀位置遍历，以完整设备/轴身份执行 `or_insert`，已有键继续指向原槽位，重复记录仍留在向量中。返回 `()`；没有新增后缀时不改索引。
    ///
    /// # 参考
    /// 参见 [`InputState::record_gamepad_axis_transition`] 与 [`InputState::append_disconnected_axis_transitions`]。
    fn index_appended_gamepad_axis_transitions(&mut self, appended_start: usize) {
        for index in appended_start..self.gamepad_axis_transitions.len() {
            let transition = self.gamepad_axis_transitions[index];
            self.gamepad_axis_transition_indices
                .entry((transition.gamepad, transition.axis))
                .or_insert(index);
        }
    }
}

/// 消费失焦前轴样本，调整已有变化并追加剩余归零记录。
///
/// # 参数与借用
/// `axes` 按值取得整个设备/轴缓存，调用方此前已从状态取走它；`transitions` 在调用期间独占借用。已有向量顺序和 `previous_value` 保持。
///
/// # 顺序与边界
/// 依次遍历已有记录，从缓存移除同身份键；若取出的缓存值非零，只把该记录最终值改零。缓存键只移除一次，同身份后续重复项不会再改写；缓存零也不改原端点。再按剩余 map 迭代顺序追加所有非零到零记录，不重排整个向量。
///
/// # 返回与生命周期
/// 返回 `()`；缓存被消费，向量修改保留。该函数不维护状态中的首槽索引，调用方只需为新增后缀补索引。
///
/// # 参考
/// 参见 [`InputState::clear_active_input_for_focus_loss`] 与 [`GamepadAxisTransition`]。
fn reset_gamepad_axes_for_focus_loss(
    mut axes: BTreeMap<(GamepadId, GamepadAxis), f32>,
    transitions: &mut Vec<GamepadAxisTransition>,
) {
    // 按原向量顺序消费缓存键；同身份只有首次匹配能改端点，后续重复项因键已移除而保留。
    for transition in transitions.iter_mut() {
        let Some(previous_value) = axes.remove(&(transition.gamepad, transition.axis)) else {
            continue;
        };
        if previous_value != 0.0 {
            transition.value = 0.0;
        }
    }
    transitions.extend(
        axes.into_iter()
            .filter_map(|((gamepad, axis), previous_value)| {
                (previous_value != 0.0).then_some(GamepadAxisTransition {
                    gamepad,
                    axis,
                    previous_value,
                    value: 0.0,
                })
            }),
    );
}

/// 依据缓存为指定断连设备追加非零到零的轴记录。
///
/// # 参数与借用
/// `axes` 在调用期间共享借用，不被删除；`transitions` 独占借用并在尾部扩展；`disconnected_gamepad` 用于完整设备身份匹配。
///
/// # 状态与顺序
/// 仅筛选该设备当前非零样本，按缓存迭代的筛选顺序追加，`previous_value` 取缓存当前值而 `value` 为零。已有记录不修改、不去重，反复调用可继续追加重复身份。
///
/// # 返回与生命周期
/// 返回 `()`；不清理连接、按钮、缓存或请求，也不维护索引。这些工作由 [`InputState::append_disconnected_axis_transitions`] 及其管理器调用方完成。
///
/// # 参考
/// 参见 [`GamepadAxisTransition`] 与 [`InputState::append_disconnected_axis_transitions`]。
fn append_disconnected_axis_transitions(
    axes: &BTreeMap<(GamepadId, GamepadAxis), f32>,
    transitions: &mut Vec<GamepadAxisTransition>,
    disconnected_gamepad: GamepadId,
) {
    transitions.extend(axes.iter().filter_map(|((gamepad, axis), value)| {
        (gamepad == &disconnected_gamepad && *value != 0.0).then_some(GamepadAxisTransition {
            gamepad: *gamepad,
            axis: *axis,
            previous_value: *value,
            value: 0.0,
        })
    }));
}

#[cfg(test)]
#[path = "tests/input_state.rs"]
mod tests;
