//! 定义持续状态、帧边沿、扩展输入和新增宿主请求的拥有型帧视图。
//!
//! # 职责与使用时机
//! 22 个公开字段供动作求值和回放结果读取。DefaultInputManager 在一次状态锁定中复制完整视图，InputManager 的默认适配只补齐基本字段；本模块的 Default 构造提供空初始化值。
//!
//! # 边界与生命周期
//! 本模块不推进帧或执行请求，数据没有帧编号、采样时间或窗口身份。旧值不随后续管理器操作改变；请求字段的当前帧后缀视图与管理器保留的全部待发队列分别解释。
//!
//! # 参考
//! 参见 [InputFrameSnapshot](super::InputFrameSnapshot)、[InputManager::frame_snapshot()](super::InputManager::frame_snapshot)、[InputManager::begin_frame()](super::InputManager::begin_frame)。
use serde::{Deserialize, Serialize};

use super::{
    ButtonInputState, CursorHostRequest, FileDragDropEvent, GamepadAxisState,
    GamepadAxisTransition, GamepadButtonValueState, GamepadId, GamepadRumbleRequest,
    ImeDeleteSurrounding, ImeHostRequest, ImePreedit, InputButton, MouseScrollUnit,
    MouseWheelEvent, TouchPoint, WindowStatusEvent,
};

/// 保存一次查询得到的持续输入状态、当前帧瞬态和新增宿主请求视图。
///
/// # 使用时机与职责
/// 完整生产者 DefaultInputManager 在一次锁定中复制状态；trait 默认适配只从基本快照补出有限字段。动作求值与回放读取此拥有型数据。
///
/// # 约束
/// 持续状态与帧瞬态分别遵从字段说明。请求 Vec 只是当前帧新增的待发请求副本，读取它不消费队列、不确认宿主执行；旧帧未发请求可能已从当前视图隐藏但仍在排空队列。此类型不含帧编号、采样时间或窗口身份；调用方负责匹配管理器与帧阶段。公开字段和派生 Deserialize 不验证去重、数值、设备连接、边沿一致性或混合滚动单位。
///
/// # 生命周期
/// 返回值与各容器按值拥有；可跨帧保留，后续提交、切帧与排空都不改变旧副本。拥有数据不代表数据仍是当前帧。
///
/// # 参考
/// 参见 [InputManager::frame_snapshot()](super::InputManager::frame_snapshot)、[InputSnapshot](super::InputSnapshot)、[InputManager::begin_frame()](super::InputManager::begin_frame)、[InputEvent](super::InputEvent)。
/// 帧级状态视图：按住和连接状态跨帧持续，边沿、增量与通知由 `begin_frame` 划定。
/// 宿主请求字段只是本帧新增视图；实际交付应使用管理器的排空接口。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InputFrameSnapshot {
    /// 最后一次提交的二维指针位置。
    ///
    /// # 使用时机与职责
    /// 持续状态；切帧保留，CursorMoved 更新，CursorLeft 不清位置。
    ///
    /// # 约束
    /// 默认 [0,0]；不由 SetPosition 请求立即回写，不自行验证坐标空间或有限性。
    ///
    /// # 生命周期
    /// 字段数据归此快照拥有；后续管理器操作不更新旧副本，调用方可修改其本地值。
    ///
    /// # 参考
    /// 参见 [InputEvent::CursorMoved](super::InputEvent::CursorMoved)、[CursorHostRequest](super::CursorHostRequest)。
    pub cursor_position: [f32; 2],
    /// 是否已收到指针进入且尚未收到离开通知。
    ///
    /// # 使用时机与职责
    /// 持续状态；CursorEntered 与 CursorLeft 更新，切帧保留。
    ///
    /// # 约束
    /// 默认 false；有位置或按钮事件不自动变为 true，焦点丢失也不等同 CursorLeft。
    ///
    /// # 生命周期
    /// 字段数据归此快照拥有；后续管理器操作不更新旧副本，调用方可修改其本地值。
    ///
    /// # 参考
    /// 参见 [InputEvent::CursorEntered](super::InputEvent::CursorEntered)、[InputEvent::CursorLeft](super::InputEvent::CursorLeft)。
    pub cursor_inside_window: bool,
    /// 当前帧新增且尚待领取的光标宿主请求视图。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager复制待发队列的本帧后缀；切帧隐藏旧项，排空后当前视图为空。
    ///
    /// # 约束
    /// 默认空；旧项仍可在队列排空中领取。读取副本不执行或确认窗口操作；焦点丢失用释放抓取替换待发队列。
    ///
    /// # 生命周期
    /// 字段数据归此快照拥有；后续管理器操作不更新旧副本，调用方可修改其本地值。
    ///
    /// # 参考
    /// 参见 [CursorHostRequest](super::CursorHostRequest)、[InputManager::drain_cursor_host_requests()](super::InputManager::drain_cursor_host_requests)。
    pub cursor_host_requests: Vec<CursorHostRequest>,
    /// 持续按下状态及本帧按下、释放边沿的完整身份集合。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager切帧只清边沿，保留持续按下；重复按下不会再生成按下边沿。
    ///
    /// # 约束
    /// 默认三个集合均为空。一个按钮可同帧具有两种边沿；trait 基本快照适配仅生成按下集合、不补边沿。公开构造或反序列化不能保证边沿与持续态一致。
    ///
    /// # 生命周期
    /// 字段数据归此快照拥有；后续管理器操作不更新旧副本，调用方可修改其本地值。
    ///
    /// # 参考
    /// 参见 [ButtonInputState](super::ButtonInputState)、[InputButton](super::InputButton)、[InputManager::begin_frame()](super::InputManager::begin_frame)。
    pub buttons: ButtonInputState<InputButton>,
    /// 本帧兼容纵向滚动累计量。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager在两种滚动事件中累加，切帧及焦点丢失清零。
    ///
    /// # 约束
    /// 默认零；Pixel 按固定 0.1 比例换成旧式标量，无法恢复横向或逐条原始单位。
    ///
    /// # 生命周期
    /// 字段数据归此快照拥有；后续管理器操作不更新旧副本，调用方可修改其本地值。
    ///
    /// # 参考
    /// 参见 [MouseWheelEvent](super::MouseWheelEvent)、[InputEvent::WheelScrolled](super::InputEvent::WheelScrolled)。
    pub wheel_accumulator: f32,
    // TODO: [CR-FRAMEWORK-INPUT-0001] 确定混合行/像素滚动时聚合值的单位契约；当前管理器直接相加并仅保存最后一个事件的单位，需核实消费方应使用逐事件列表还是归一化聚合。
    /// 本帧滚动事件 x、y 原始数值的逐轴累加。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager逐条相加，切帧及焦点丢失清零。
    ///
    /// # 约束
    /// 默认 [0,0]；Line 与 Pixel 混用时仍直接相加，没有统一单位。最近的单位字段不能解释整个累计向量，精确消费应逐条读取带单位事件。
    ///
    /// # 生命周期
    /// 字段数据归此快照拥有；后续管理器操作不更新旧副本，调用方可修改其本地值。
    ///
    /// # 参考
    /// 参见 [MouseWheelEvent](super::MouseWheelEvent)、[InputFrameSnapshot](super::InputFrameSnapshot)。
    pub mouse_wheel_accumulator: [f32; 2],
    /// 本帧最近一次滚动输入的单位标记。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager每次滚动覆盖；切帧及焦点丢失恢复 Line。
    ///
    /// # 约束
    /// 默认 Line；它不意味着累计向量中所有数值使用该单位，未有滚动事件时也为 Line。
    ///
    /// # 生命周期
    /// 字段数据归此快照拥有；后续管理器操作不更新旧副本，调用方可修改其本地值。
    ///
    /// # 参考
    /// 参见 [MouseScrollUnit](super::MouseScrollUnit)、[MouseWheelEvent](super::MouseWheelEvent)。
    pub mouse_wheel_unit: MouseScrollUnit,
    /// 本帧按提交顺序保留的带单位滚轮事件。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager每次滚动追加；旧式 WheelScrolled 合成一条 Line 事件；切帧及焦点丢失清空。
    ///
    /// # 约束
    /// 默认空；每条保留自身单位，可用于正确处理混合 Line/Pixel。它是状态快照中的帧列表，不等同即时事件队列或跨帧原始录制流。
    ///
    /// # 生命周期
    /// 字段数据归此快照拥有；后续管理器操作不更新旧副本，调用方可修改其本地值。
    ///
    /// # 参考
    /// 参见 [MouseWheelEvent](super::MouseWheelEvent)、[InputManager::drain_events()](super::InputManager::drain_events)、[InputEventRecord](super::InputEventRecord)。
    pub mouse_wheel_events: Vec<MouseWheelEvent>,
    /// 本帧相对鼠标移动的逐轴累计量。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager对 MouseMotion 的 delta_x、delta_y 相加；切帧及焦点丢失清零。
    ///
    /// # 约束
    /// 默认 [0,0]；不等同绝对指针位置，数据类型不声明 DPI、缩放或有限性验证。
    ///
    /// # 生命周期
    /// 字段数据归此快照拥有；后续管理器操作不更新旧副本，调用方可修改其本地值。
    ///
    /// # 参考
    /// 参见 [InputEvent::MouseMotion](super::InputEvent::MouseMotion)、[InputEvent::CursorMoved](super::InputEvent::CursorMoved)。
    pub mouse_motion_accumulator: [f32; 2],
    /// 仍活动触点的最后阶段与位置样本。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager按触点 id 更新 Started/Moved，Ended/Cancelled 移除，切帧保留，焦点丢失清空。
    ///
    /// # 约束
    /// 默认空；DefaultInputManager按 id 有序复制，但公开 Vec 不验证唯一性或顺序。没有本帧 Touch 事件时仍可存在旧活动点，Started 不自动改成 Moved；终止边沿须从事件流读取。
    ///
    /// # 生命周期
    /// 字段数据归此快照拥有；后续管理器操作不更新旧副本，调用方可修改其本地值。
    ///
    /// # 参考
    /// 参见 [TouchPoint](super::TouchPoint)、[TouchPhase](super::TouchPhase)、[InputEvent::Touch](super::InputEvent::Touch)。
    pub active_touches: Vec<TouchPoint>,
    /// 由连接通知保留的手柄身份列表。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager收到连接时加入、断开时移除；切帧和焦点丢失保留连接集合。
    ///
    /// # 约束
    /// 默认空；DefaultInputManager按身份去重有序复制。轴/按钮提交不自动证明连接，身份值也不拥有或验证活设备对象；公开 Vec 可含重复项。
    ///
    /// # 生命周期
    /// 字段数据归此快照拥有；后续管理器操作不更新旧副本，调用方可修改其本地值。
    ///
    /// # 参考
    /// 参见 [GamepadId](super::GamepadId)、[InputEvent::GamepadConnection](super::InputEvent::GamepadConnection)。
    pub connected_gamepads: Vec<GamepadId>,
    /// 已发布并经具体实现处理的手柄轴持续值。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager使用默认死区、活区与变化阈值；切帧保留，断开或焦点丢失移除相应状态。
    ///
    /// # 约束
    /// 默认空；它可与原始提交值不同。DefaultInputManager按设备/轴身份有序复制；公开 Vec 和反序列化不验证范围、唯一性或连接关系。
    ///
    /// # 生命周期
    /// 字段数据归此快照拥有；后续管理器操作不更新旧副本，调用方可修改其本地值。
    ///
    /// # 参考
    /// 参见 [GamepadAxisState](super::GamepadAxisState)、[GamepadAxisSettings](super::GamepadAxisSettings)、[InputEvent::GamepadAxis](super::InputEvent::GamepadAxis)。
    pub gamepad_axes: Vec<GamepadAxisState>,
    /// 本帧已发布手柄轴值的变化记录。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager 一般采样按首次出现顺序保留首次 previous_value 与最新 value；切帧清空。断开会追加非零轴归零项，即使同一身份已有变化；焦点丢失可修改已有变化的最终值并追加尚未记录的归零项。
    ///
    /// # 约束
    /// 默认空；这是处理后状态边沿，不是每条原始样本。断开追加路径可与已有身份变化并存，类型不承诺全局去重或按身份排序；求值方应按实际生产口径读取。
    ///
    /// # 生命周期
    /// 字段数据归此快照拥有；后续管理器操作不更新旧副本，调用方可修改其本地值。
    ///
    /// # 参考
    /// 参见 [GamepadAxisTransition](super::GamepadAxisTransition)、[InputEvent::GamepadAxis](super::InputEvent::GamepadAxis)、[InputEvent::GamepadConnection](super::InputEvent::GamepadConnection)。
    pub gamepad_axis_transitions: Vec<GamepadAxisTransition>,
    /// 已发布的手柄按钮或模拟扳机持续幅值。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager使用默认幅值映射及变化阈值；切帧保留，断开或焦点丢失移除相应数据。
    ///
    /// # 约束
    /// 默认空；原始 value 与发布幅值可能不同。按钮按下状态还受宿主 pressed 和滞回阈值共同影响，不能仅用非零幅值当作按下。
    ///
    /// # 生命周期
    /// 字段数据归此快照拥有；后续管理器操作不更新旧副本，调用方可修改其本地值。
    ///
    /// # 参考
    /// 参见 [GamepadButtonValueState](super::GamepadButtonValueState)、[GamepadButtonAxisSettings](super::GamepadButtonAxisSettings)、[GamepadButtonSettings](super::GamepadButtonSettings)。
    pub gamepad_button_values: Vec<GamepadButtonValueState>,
    /// 当前帧新增且尚待领取的手柄震动请求视图。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager复制待发队列本帧后缀；切帧隐藏旧项，整体排空后当前视图为空。
    ///
    /// # 约束
    /// 默认空；读取不执行震动、不证明设备连接或操作完成，旧请求仍可从队列领取。此快照不提供请求回执。
    ///
    /// # 生命周期
    /// 字段数据归此快照拥有；后续管理器操作不更新旧副本，调用方可修改其本地值。
    ///
    /// # 参考
    /// 参见 [GamepadRumbleRequest](super::GamepadRumbleRequest)、[InputManager::drain_gamepad_rumble_requests()](super::InputManager::drain_gamepad_rumble_requests)。
    pub gamepad_rumble_requests: Vec<GamepadRumbleRequest>,
    /// 已观察到的 IME 启用状态。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager以 ImeEvent 的 Enabled/Disabled 更新；切帧保留，焦点丢失置 false。
    ///
    /// # 约束
    /// 默认 false；提交 Enable 宿主请求只排队，不立即改变此观察状态，也不确认系统已启用 IME。
    ///
    /// # 生命周期
    /// 字段数据归此快照拥有；后续管理器操作不更新旧副本，调用方可修改其本地值。
    ///
    /// # 参考
    /// 参见 [ImeEvent](super::ImeEvent)、[ImeHostRequest](super::ImeHostRequest)、[InputEvent::Ime](super::InputEvent::Ime)。
    pub ime_enabled: bool,
    /// 可选的当前 IME 预编辑状态。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager跨帧保留；空 Preedit、Commit、Disabled 或焦点丢失清除。
    ///
    /// # 约束
    /// 默认 None；不等同最终提交文本，公开值不验证 UTF-8 字节范围等内部语义；具体宿主 V2 验证另在入口执行。 DefaultInputManager 把空 Preedit 转为 None；公开构造或反序列化仍可提供包含空文本的 Some，类型不强制非空。
    ///
    /// # 生命周期
    /// 字段数据归此快照拥有；后续管理器操作不更新旧副本，调用方可修改其本地值。
    ///
    /// # 参考
    /// 参见 [ImePreedit](super::ImePreedit)、[ImeEvent](super::ImeEvent)、[InputEvent::Ime](super::InputEvent::Ime)。
    pub ime_preedit: Option<ImePreedit>,
    /// 本帧提交给管理器的 IME 已确认文本列表。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager每次 Commit 追加拥有型文本并清预编辑；切帧清列表。
    ///
    /// # 约束
    /// 默认空；可包含空文本，KeyboardInput.text 不会自动加入此列表。宿主 UI 已接管而未提交到管理器的内容不在这里。
    ///
    /// # 生命周期
    /// 字段数据归此快照拥有；后续管理器操作不更新旧副本，调用方可修改其本地值。
    ///
    /// # 参考
    /// 参见 [ImeEvent](super::ImeEvent)、[InputEvent::KeyboardInput](super::InputEvent::KeyboardInput)、[InputEvent::Ime](super::InputEvent::Ime)。
    pub ime_commits: Vec<String>,
    /// 本帧提交给管理器的周边文本删除请求列表。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager收到 ImeEvent::DeleteSurrounding 时追加，切帧清空。
    ///
    /// # 约束
    /// 默认空；此列表只描述请求，不替宿主或文本控件执行删除，数量/字节范围的业务有效性由消费方处理。
    ///
    /// # 生命周期
    /// 字段数据归此快照拥有；后续管理器操作不更新旧副本，调用方可修改其本地值。
    ///
    /// # 参考
    /// 参见 [ImeDeleteSurrounding](super::ImeDeleteSurrounding)、[ImeEvent](super::ImeEvent)、[InputEvent::Ime](super::InputEvent::Ime)。
    pub ime_delete_surrounding: Vec<ImeDeleteSurrounding>,
    /// 当前帧新增且尚待领取的反向 IME 宿主请求视图。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager复制待发队列本帧后缀；切帧隐藏旧项，整体排空后当前视图为空。
    ///
    /// # 约束
    /// 默认空；旧帧未发请求仍可排空，快照不消费或确认宿主执行。焦点丢失替换待发 IME 队列为 Disable。
    ///
    /// # 生命周期
    /// 字段数据归此快照拥有；后续管理器操作不更新旧副本，调用方可修改其本地值。
    ///
    /// # 参考
    /// 参见 [ImeHostRequest](super::ImeHostRequest)、[InputManager::drain_ime_host_requests()](super::InputManager::drain_ime_host_requests)。
    pub ime_host_requests: Vec<ImeHostRequest>,
    /// 本帧提交到管理器的窗口状态通知列表。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager逐条追加，切帧清空。
    ///
    /// # 约束
    /// 默认空；它不是完整持续窗口状态，也不因 Destroyed 自动调用管理器销毁或因任意通知触发 FocusLost。上层窗口处理与输入状态相互分离。
    ///
    /// # 生命周期
    /// 字段数据归此快照拥有；后续管理器操作不更新旧副本，调用方可修改其本地值。
    ///
    /// # 参考
    /// 参见 [WindowStatusEvent](super::WindowStatusEvent)、[InputEvent::WindowStatus](super::InputEvent::WindowStatus)、[InputEvent::FocusLost](super::InputEvent::FocusLost)。
    pub window_status_events: Vec<WindowStatusEvent>,
    /// 本帧提交到管理器的文件拖放通知列表。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager逐条追加，切帧清空。
    ///
    /// # 约束
    /// 默认空；路径文本是事件载荷，不验证路径存在、打开文件或导入资产；UI 与业务消费另行负责。
    ///
    /// # 生命周期
    /// 字段数据归此快照拥有；后续管理器操作不更新旧副本，调用方可修改其本地值。
    ///
    /// # 参考
    /// 参见 [FileDragDropEvent](super::FileDragDropEvent)、[InputEvent::FileDragDrop](super::InputEvent::FileDragDrop)。
    pub file_drag_drop_events: Vec<FileDragDropEvent>,
}

impl Default for InputFrameSnapshot {
    /// 构造不关联任何管理器采样的空输入帧值。
    ///
    /// # 使用时机与职责
    /// 作为新值初始化及 trait 基本快照适配的补齐基线。
    ///
    /// # 返回
    /// 新的拥有型 InputFrameSnapshot：位置和三种累计量为零，窗口内/IME 标志为 false，预编辑为 None，按钮三集合与所有 Vec 为空，滚轮单位为 Line。
    ///
    /// # 约束
    /// 空默认值不表示曾查询真实设备、不提供帧边沿或宿主能力证明。本类型的 Default 构造并非 Deserialize 的缺字段补齐规则，也不约束调用方之后写入公开字段。
    ///
    /// # 生命周期
    /// 每次创建独立容器和值；不持有管理器、宿主、设备或输入参数的借用。
    ///
    /// # 参考
    /// 参见 [InputFrameSnapshot](super::InputFrameSnapshot)、[InputManager::frame_snapshot()](super::InputManager::frame_snapshot)、[ButtonInputState](super::ButtonInputState)、[MouseScrollUnit](super::MouseScrollUnit)。
    fn default() -> Self {
        Self {
            cursor_position: [0.0, 0.0],
            cursor_inside_window: false,
            cursor_host_requests: Vec::new(),
            buttons: ButtonInputState::default(),
            wheel_accumulator: 0.0,
            mouse_wheel_accumulator: [0.0, 0.0],
            mouse_wheel_unit: MouseScrollUnit::Line,
            mouse_wheel_events: Vec::new(),
            mouse_motion_accumulator: [0.0, 0.0],
            active_touches: Vec::new(),
            connected_gamepads: Vec::new(),
            gamepad_axes: Vec::new(),
            gamepad_axis_transitions: Vec::new(),
            gamepad_button_values: Vec::new(),
            gamepad_rumble_requests: Vec::new(),
            ime_enabled: false,
            ime_preedit: None,
            ime_commits: Vec::new(),
            ime_delete_surrounding: Vec::new(),
            ime_host_requests: Vec::new(),
            window_status_events: Vec::new(),
            file_drag_drop_events: Vec::new(),
        }
    }
}
