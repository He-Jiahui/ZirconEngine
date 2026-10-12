//! 定义提交给输入管理器的拥有型设备事件、窗口通知和反向宿主请求。
//!
//! # 职责与使用时机
//! 20 个事件分支由宿主适配、调用方或回放构造，经 InputManager::submit_event 交给实现。本模块表达载荷，具体实现再分别更新持续状态、帧通知、即时队列及可选原始记录。
//!
//! # 边界与生命周期
//! 枚举及 serde 不统一验证数值、设备连接或触点顺序，也不提供接收或硬件执行回执。String 等载荷由事件拥有；宿主 ABI 的临时字节须在构造拥有型值前验证或复制。
//!
//! # 参考
//! 参见 [InputEvent](super::InputEvent)、[InputManager::submit_event()](super::InputManager::submit_event)、[InputFrameSnapshot](super::InputFrameSnapshot)、[InputEventRecord](super::InputEventRecord)。
use serde::{Deserialize, Serialize};

use super::{
    CursorHostRequest, FileDragDropEvent, GamepadAxis, GamepadButton, GamepadConnectionInfo,
    GamepadId, GamepadRumbleRequest, ImeEvent, ImeHostRequest, InputButton, MouseWheelEvent,
    TouchPhase, WindowStatusEvent,
};

/// 用共享分支表达已提交的设备输入、窗口通知和反向宿主请求。
///
/// # 使用时机与职责
/// 宿主适配、调用方或回放构造后交给 InputManager。DefaultInputManager 先更新状态，再按录制开关和容量尝试保留提交事件，最后更新可合并相邻移动的即时队列；快照是处理后的状态视图。
///
/// # 约束
/// 拥有型枚举和派生 Deserialize 只表达载荷，不统一验证数值、设备连接、触点阶段顺序、键名映射或请求有效性。宿主 ABI 的验证不覆盖直接构造或反序列化后提交。UI 有独立传播：物理键/按钮通常先提交，文本和 IME 内容可先由聚焦 UI 接管；不能把此事件流称为全部宿主原始事件。
///
/// # 生命周期
/// 字符串等载荷由值拥有，不借用宿主临时字节。克隆和录制副本的存活不等同设备、窗口或请求执行状态的存活。
///
/// # 参考
/// 参见 [InputManager](super::InputManager)、[InputFrameSnapshot](super::InputFrameSnapshot)、[InputEventRecord](super::InputEventRecord)、[InputButton](super::InputButton)。
/// 宿主归一化后提交给输入管理器的事件边界；管理器同时更新设备状态、即时帧队列及可选录制。
/// UI 分发可有独立路由，提交这里并不表示 UI 已处理该事件。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum InputEvent {
    /// 提交二维绝对指针位置。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager替换持续位置；即时队列只在相邻同分支时保留最新事件。
    ///
    /// # 约束
    /// x、y 由提交方的坐标约定解释，不自行缩放或检查有限性；不自动设置窗口内标志。
    ///
    /// # 生命周期
    /// 事件按值拥有载荷；提交后由实现消费，克隆或录制副本的保留与输入状态更新独立。
    ///
    /// # 参考
    /// 参见 [InputFrameSnapshot](super::InputFrameSnapshot)、[InputEvent::MouseMotion](super::InputEvent::MouseMotion)。
    CursorMoved {
        /// 绝对指针位置的横向分量。
        ///
        /// # 使用时机与职责
        /// 随所属事件分支按值提交；处理时机和状态保留规则参见该分支。
        ///
        /// # 约束
        /// DefaultInputManager原样替换位置 x；坐标空间、有限性由提交方负责。
        ///
        /// # 生命周期
        /// 字段按值随事件保存，不借用管理器或宿主输入缓冲。
        ///
        /// # 参考
        /// 参见 [InputEvent::CursorMoved](super::InputEvent::CursorMoved)、[InputFrameSnapshot](super::InputFrameSnapshot)。
        x: f32,
        /// 绝对指针位置的纵向分量。
        ///
        /// # 使用时机与职责
        /// 随所属事件分支按值提交；处理时机和状态保留规则参见该分支。
        ///
        /// # 约束
        /// DefaultInputManager原样替换位置 y；不由此推断窗口内或焦点状态。
        ///
        /// # 生命周期
        /// 字段按值随事件保存，不借用管理器或宿主输入缓冲。
        ///
        /// # 参考
        /// 参见 [InputEvent::CursorMoved](super::InputEvent::CursorMoved)、[InputFrameSnapshot](super::InputFrameSnapshot)。
        y: f32,
    },
    /// 通知指针进入窗口。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager将窗口内状态设为 true 并跨帧保留。
    ///
    /// # 约束
    /// 不携带位置，也不代表按键按下或焦点取得。
    ///
    /// # 生命周期
    /// 事件按值拥有载荷；提交后由实现消费，克隆或录制副本的保留与输入状态更新独立。
    ///
    /// # 参考
    /// 参见 [InputFrameSnapshot](super::InputFrameSnapshot)、[InputEvent::CursorLeft](super::InputEvent::CursorLeft)。
    CursorEntered,
    /// 通知指针离开窗口。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager将窗口内状态设为 false，持续位置仍保留。
    ///
    /// # 约束
    /// 不等同 FocusLost，也不自动释放所有按钮或终止触点；宿主需要另行提交相关事件。
    ///
    /// # 生命周期
    /// 事件按值拥有载荷；提交后由实现消费，克隆或录制副本的保留与输入状态更新独立。
    ///
    /// # 参考
    /// 参见 [InputFrameSnapshot](super::InputFrameSnapshot)、[InputEvent::FocusLost](super::InputEvent::FocusLost)。
    CursorLeft,
    /// 提交二维相对鼠标移动。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager累加本帧移动；即时队列只将相邻此分支逐轴相加。
    ///
    /// # 约束
    /// 增量不是绝对坐标，类型不验证数值或统一设备/DPI 约定；浮点相加顺序可能影响舍入。
    ///
    /// # 生命周期
    /// 事件按值拥有载荷；提交后由实现消费，克隆或录制副本的保留与输入状态更新独立。
    ///
    /// # 参考
    /// 参见 [InputFrameSnapshot](super::InputFrameSnapshot)、[InputEvent::CursorMoved](super::InputEvent::CursorMoved)。
    MouseMotion {
        /// 本次相对移动的横向增量。
        ///
        /// # 使用时机与职责
        /// 随所属事件分支按值提交；处理时机和状态保留规则参见该分支。
        ///
        /// # 约束
        /// DefaultInputManager累加到本帧 x；不是绝对坐标，非有限值不在类型中验证。
        ///
        /// # 生命周期
        /// 字段按值随事件保存，不借用管理器或宿主输入缓冲。
        ///
        /// # 参考
        /// 参见 [InputEvent::MouseMotion](super::InputEvent::MouseMotion)、[InputFrameSnapshot](super::InputFrameSnapshot)。
        delta_x: f32,
        /// 本次相对移动的纵向增量。
        ///
        /// # 使用时机与职责
        /// 随所属事件分支按值提交；处理时机和状态保留规则参见该分支。
        ///
        /// # 约束
        /// DefaultInputManager累加到本帧 y；邻接合并和浮点相加不恢复每个原始样本。
        ///
        /// # 生命周期
        /// 字段按值随事件保存，不借用管理器或宿主输入缓冲。
        ///
        /// # 参考
        /// 参见 [InputEvent::MouseMotion](super::InputEvent::MouseMotion)、[InputFrameSnapshot](super::InputFrameSnapshot)。
        delta_y: f32,
    },
    /// 提交一个完整按钮身份的按下通知。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager更新持续按下和本帧边沿；重复相同身份不会新生按下边沿。
    ///
    /// # 约束
    /// 载荷 .0 为拥有型 InputButton；它不验证设备已连接，不把物理键码与逻辑键名合成同一身份。
    ///
    /// # 生命周期
    /// 事件按值拥有载荷；提交后由实现消费，克隆或录制副本的保留与输入状态更新独立。
    ///
    /// # 参考
    /// 参见 [InputButton](super::InputButton)、[ButtonInputState](super::ButtonInputState)、[InputEvent::ButtonReleased](super::InputEvent::ButtonReleased)。
    ButtonPressed(InputButton),
    /// 提交一个完整按钮身份的释放通知。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager移除该身份的持续按下并产生本帧释放边沿；未按下身份不产生边沿。
    ///
    /// # 约束
    /// 载荷 .0 为拥有型 InputButton；需要与此前按下的分支及载荷完整匹配。
    ///
    /// # 生命周期
    /// 事件按值拥有载荷；提交后由实现消费，克隆或录制副本的保留与输入状态更新独立。
    ///
    /// # 参考
    /// 参见 [InputButton](super::InputButton)、[ButtonInputState](super::ButtonInputState)、[InputEvent::ButtonPressed](super::InputEvent::ButtonPressed)。
    ButtonReleased(InputButton),
    /// 提交旧式行单位纵向滚动增量。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager累加兼容标量与原始 y 累计，并追加一条 Line 事件。
    ///
    /// # 约束
    /// delta 是本次增量而非总量；不提供横向或 Pixel 单位，也不自行过滤非有限值。
    ///
    /// # 生命周期
    /// 事件按值拥有载荷；提交后由实现消费，克隆或录制副本的保留与输入状态更新独立。
    ///
    /// # 参考
    /// 参见 [MouseWheelEvent](super::MouseWheelEvent)、[InputEvent::MouseWheel](super::InputEvent::MouseWheel)、[InputFrameSnapshot](super::InputFrameSnapshot)。
    WheelScrolled {
        /// 旧式本次纵向行单位滚动增量。
        ///
        /// # 使用时机与职责
        /// 随所属事件分支按值提交；处理时机和状态保留规则参见该分支。
        ///
        /// # 约束
        /// DefaultInputManager累加兼容纵向值，并合成 Line 滚轮事件；非有限性不在本类型验证。
        ///
        /// # 生命周期
        /// 字段按值随事件保存，不借用管理器或宿主输入缓冲。
        ///
        /// # 参考
        /// 参见 [InputEvent::WheelScrolled](super::InputEvent::WheelScrolled)、[MouseWheelEvent](super::MouseWheelEvent)。
        delta: f32,
    },
    /// 提交携带 Line/Pixel 单位的二维滚轮事件。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager保留逐条原始滚轮事件及 x/y 累计，另为兼容标量换算纵向值。
    ///
    /// # 约束
    /// 载荷 .0 为 MouseWheelEvent。类型和直接提交不验证有限性；动态宿主新式路径另行拒绝非有限增量。混合单位累计未统一尺度。
    ///
    /// # 生命周期
    /// 事件按值拥有载荷；提交后由实现消费，克隆或录制副本的保留与输入状态更新独立。
    ///
    /// # 参考
    /// 参见 [MouseWheelEvent](super::MouseWheelEvent)、[InputFrameSnapshot](super::InputFrameSnapshot)、[InputEvent::WheelScrolled](super::InputEvent::WheelScrolled)。
    MouseWheel(MouseWheelEvent),
    /// 提交物理键码、可选逻辑键名及文本的键盘输入样本。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager依据 pressed 更新 KeyCode，并在 logical_key 为 Some 时单独更新 Key；text 和 repeat 不直接生成 IME 或额外按下边沿。
    ///
    /// # 约束
    /// 按下与释放使用各自提供的逻辑键名，不在本类型中记忆/修正布局映射。文本与重复字段仍随原始记录保留；宿主文本路径可能先被 UI 接管。
    ///
    /// # 生命周期
    /// 事件按值拥有载荷；提交后由实现消费，克隆或录制副本的保留与输入状态更新独立。
    ///
    /// # 参考
    /// 参见 [InputButton](super::InputButton)、[InputFrameSnapshot](super::InputFrameSnapshot)、[InputEvent::Ime](super::InputEvent::Ime)。
    KeyboardInput {
        /// 本次物理键码身份值。
        ///
        /// # 使用时机与职责
        /// 随所属事件分支按值提交；处理时机和状态保留规则参见该分支。
        ///
        /// # 约束
        /// DefaultInputManager独立作为 InputButton::KeyCode 更新；数字约定来自提交方，不等同逻辑字符串。
        ///
        /// # 生命周期
        /// 字段按值随事件保存，不借用管理器或宿主输入缓冲。
        ///
        /// # 参考
        /// 参见 [InputEvent::KeyboardInput](super::InputEvent::KeyboardInput)、[InputButton](super::InputButton)。
        key_code: u32,
        /// 可选的拥有型逻辑键名。
        ///
        /// # 使用时机与职责
        /// 随所属事件分支按值提交；处理时机和状态保留规则参见该分支。
        ///
        /// # 约束
        /// Some 时 DefaultInputManager独立更新 InputButton::Key；None 时只更新物理码。按放载荷的名称需由提交方保持匹配，类型不折叠大小写。
        ///
        /// # 生命周期
        /// 拥有型 String 及其 Option 随事件保存，不借用宿主临时缓冲。
        ///
        /// # 参考
        /// 参见 [InputEvent::KeyboardInput](super::InputEvent::KeyboardInput)、[InputButton](super::InputButton)。
        logical_key: Option<String>,
        /// 本次可选的拥有型键盘文本。
        ///
        /// # 使用时机与职责
        /// 随所属事件分支按值提交；处理时机和状态保留规则参见该分支。
        ///
        /// # 约束
        /// DefaultInputManager不将它自动追加到 IME commits；启用且保留策略允许时原始记录保存该载荷，宿主文本可先由 UI 接管。
        ///
        /// # 生命周期
        /// 拥有型 String 及其 Option 随事件保存，不借用宿主临时缓冲。
        ///
        /// # 参考
        /// 参见 [InputEvent::KeyboardInput](super::InputEvent::KeyboardInput)、[InputEventRecord](super::InputEventRecord)。
        text: Option<String>,
        /// 本次键盘样本的按下/释放标志。
        ///
        /// # 使用时机与职责
        /// 随所属事件分支按值提交；处理时机和状态保留规则参见该分支。
        ///
        /// # 约束
        /// DefaultInputManager以 true 按下、false 释放各键身份；重复按下由集合幂等处理，不代表本帧边沿本身。
        ///
        /// # 生命周期
        /// 字段按值随事件保存，不借用管理器或宿主输入缓冲。
        ///
        /// # 参考
        /// 参见 [InputEvent::KeyboardInput](super::InputEvent::KeyboardInput)、[ButtonInputState](super::ButtonInputState)。
        pressed: bool,
        /// 提交方提供的键盘重复标志。
        ///
        /// # 使用时机与职责
        /// 随所属事件分支按值提交；处理时机和状态保留规则参见该分支。
        ///
        /// # 约束
        /// DefaultInputManager不凭此字段产生额外边沿或文本，当前动态宿主输入路径填写 false；直接构造或记录可保留其他值。
        ///
        /// # 生命周期
        /// 字段按值随事件保存，不借用管理器或宿主输入缓冲。
        ///
        /// # 参考
        /// 参见 [InputEvent::KeyboardInput](super::InputEvent::KeyboardInput)、[InputEventRecord](super::InputEventRecord)。
        repeat: bool,
    },
    /// 提交窗口状态通知。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager只将该通知追加到本帧窗口通知列表。
    ///
    /// # 约束
    /// 载荷 .0 为 WindowStatusEvent；窗口业务、销毁与时钟变更由宿主或上层处理，本分支不代替 FocusLost 状态重置。
    ///
    /// # 生命周期
    /// 事件按值拥有载荷；提交后由实现消费，克隆或录制副本的保留与输入状态更新独立。
    ///
    /// # 参考
    /// 参见 [WindowStatusEvent](super::WindowStatusEvent)、[InputEvent::FocusLost](super::InputEvent::FocusLost)、[InputFrameSnapshot](super::InputFrameSnapshot)。
    WindowStatus(WindowStatusEvent),
    /// 提交文件拖放通知。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager只追加到本帧文件通知列表。
    ///
    /// # 约束
    /// 载荷 .0 为 FileDragDropEvent；路径是拥有型文字，不验证文件存在、打开文件或导入资源。
    ///
    /// # 生命周期
    /// 事件按值拥有载荷；提交后由实现消费，克隆或录制副本的保留与输入状态更新独立。
    ///
    /// # 参考
    /// 参见 [FileDragDropEvent](super::FileDragDropEvent)、[InputFrameSnapshot](super::InputFrameSnapshot)。
    FileDragDrop(FileDragDropEvent),
    /// 提交已进入输入管理器的 IME 状态或文本通知。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager更新启用/预编辑持续状态，并追加本帧提交文本或周边删除请求。
    ///
    /// # 约束
    /// 载荷 .0 为 ImeEvent；宿主可让 UI 先接管内容，管理器不必收到全部系统 IME 事件。本分支不会执行反向宿主命令。
    ///
    /// # 生命周期
    /// 事件按值拥有载荷；提交后由实现消费，克隆或录制副本的保留与输入状态更新独立。
    ///
    /// # 参考
    /// 参见 [ImeEvent](super::ImeEvent)、[InputEvent::ImeHostRequest](super::InputEvent::ImeHostRequest)、[InputFrameSnapshot](super::InputFrameSnapshot)。
    Ime(ImeEvent),
    /// 提交待宿主执行的反向 IME 命令。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager追加到待发 IME 队列；帧快照只展示本帧新增后缀。
    ///
    /// # 约束
    /// 载荷 .0 为 ImeHostRequest；提交 Enable/Disable 不直接改变已观察的 ime_enabled。需由宿主排空并执行，没有执行回执。
    ///
    /// # 生命周期
    /// 事件按值拥有载荷；提交后由实现消费，克隆或录制副本的保留与输入状态更新独立。
    ///
    /// # 参考
    /// 参见 [ImeHostRequest](super::ImeHostRequest)、[InputManager::drain_ime_host_requests()](super::InputManager::drain_ime_host_requests)、[InputFrameSnapshot](super::InputFrameSnapshot)。
    ImeHostRequest(ImeHostRequest),
    /// 提交待窗口宿主执行的反向光标命令。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager追加到待发光标队列，宿主经排空领取。
    ///
    /// # 约束
    /// 载荷 .0 为 CursorHostRequest；SetPosition 不在此直接改写已观察指针位置，命令不证明窗口能力或执行成功。
    ///
    /// # 生命周期
    /// 事件按值拥有载荷；提交后由实现消费，克隆或录制副本的保留与输入状态更新独立。
    ///
    /// # 参考
    /// 参见 [CursorHostRequest](super::CursorHostRequest)、[InputManager::drain_cursor_host_requests()](super::InputManager::drain_cursor_host_requests)、[InputFrameSnapshot](super::InputFrameSnapshot)。
    CursorHostRequest(CursorHostRequest),
    /// 通知实现输入焦点丢失以清理容易卡住的状态。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager释放全部按下身份、清触点与模拟状态、清移动/滚动并关闭 IME/预编辑，替换待发 IME 与光标队列为 Disable 和释放抓取。
    ///
    /// # 约束
    /// DefaultInputManager 保留连接设备、指针位置/窗口内标志、已产生的 IME 提交/周边删除、窗口/文件通知、待发震动请求及事件缓冲；已有轴变化可被改写为归零。按下与释放边沿可以同帧共存。此事件不等同设备断开或全部宿主操作完成。
    ///
    /// # 生命周期
    /// 事件按值拥有载荷；提交后由实现消费，克隆或录制副本的保留与输入状态更新独立。
    ///
    /// # 参考
    /// 参见 [InputFrameSnapshot](super::InputFrameSnapshot)、[InputEvent::GamepadConnection](super::InputEvent::GamepadConnection)、[ButtonInputState](super::ButtonInputState)。
    FocusLost,
    /// 提交一个触点身份、阶段和坐标样本。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager Started/Moved 对 id 写入当前样本，Ended/Cancelled 从活动集合移除；切帧保留活动点。
    ///
    /// # 约束
    /// 类型不验证阶段先后或坐标有限性，未知 id 的 Moved 也可直接成为活动点。终止边沿从事件流而非活动触点快照读取。
    ///
    /// # 生命周期
    /// 事件按值拥有载荷；提交后由实现消费，克隆或录制副本的保留与输入状态更新独立。
    ///
    /// # 参考
    /// 参见 [TouchPhase](super::TouchPhase)、[TouchPoint](super::TouchPoint)、[InputFrameSnapshot](super::InputFrameSnapshot)。
    Touch {
        /// 提交方提供的触点身份键。
        ///
        /// # 使用时机与职责
        /// 随所属事件分支按值提交；处理时机和状态保留规则参见该分支。
        ///
        /// # 约束
        /// DefaultInputManager仅按此整数区分活动点；没有设备/窗口世代验证，不拥有触点对象。
        ///
        /// # 生命周期
        /// 字段按值随事件保存，不借用管理器或宿主输入缓冲。
        ///
        /// # 参考
        /// 参见 [InputEvent::Touch](super::InputEvent::Touch)、[TouchPoint](super::TouchPoint)。
        id: u64,
        /// 本次触点阶段。
        ///
        /// # 使用时机与职责
        /// 随所属事件分支按值提交；处理时机和状态保留规则参见该分支。
        ///
        /// # 约束
        /// Started/Moved 写入最后样本，Ended/Cancelled 移除；DefaultInputManager 不验证阶段先后，终止边沿需读事件流。
        ///
        /// # 生命周期
        /// 字段按值随事件保存，不借用管理器或宿主输入缓冲。
        ///
        /// # 参考
        /// 参见 [InputEvent::Touch](super::InputEvent::Touch)、[TouchPhase](super::TouchPhase)。
        phase: TouchPhase,
        /// 本次触点位置的横向分量。
        ///
        /// # 使用时机与职责
        /// 随所属事件分支按值提交；处理时机和状态保留规则参见该分支。
        ///
        /// # 约束
        /// DefaultInputManager写入该 id 的样本，不自行检查有限性或换算坐标空间。
        ///
        /// # 生命周期
        /// 字段按值随事件保存，不借用管理器或宿主输入缓冲。
        ///
        /// # 参考
        /// 参见 [InputEvent::Touch](super::InputEvent::Touch)、[TouchPoint](super::TouchPoint)。
        x: f32,
        /// 本次触点位置的纵向分量。
        ///
        /// # 使用时机与职责
        /// 随所属事件分支按值提交；处理时机和状态保留规则参见该分支。
        ///
        /// # 约束
        /// DefaultInputManager写入该 id 的样本，Ended/Cancelled 的载荷仍可从原始事件读取而不留在活动列表。
        ///
        /// # 生命周期
        /// 字段按值随事件保存，不借用管理器或宿主输入缓冲。
        ///
        /// # 参考
        /// 参见 [InputEvent::Touch](super::InputEvent::Touch)、[TouchPoint](super::TouchPoint)。
        y: f32,
    },
    /// 提交手柄连接/断开通知及描述元数据。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager连接时加入身份；断开时移除身份及其模拟状态，释放其按钮并可追加非零轴归零变化。
    ///
    /// # 约束
    /// 载荷 .0 为 GamepadConnectionInfo；完整事件保留元数据，而 connected_gamepads 快照只含 id。身份值不拥有或验证活设备。
    ///
    /// # 生命周期
    /// 事件按值拥有载荷；提交后由实现消费，克隆或录制副本的保留与输入状态更新独立。
    ///
    /// # 参考
    /// 参见 [GamepadConnectionInfo](super::GamepadConnectionInfo)、[GamepadId](super::GamepadId)、[InputFrameSnapshot](super::InputFrameSnapshot)。
    GamepadConnection(GamepadConnectionInfo),
    /// 提交指定手柄按钮的原始幅值及宿主按下标志。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager先应用默认幅值映射和变化阈值，再结合宿主 pressed 与滞回决定按钮状态。
    ///
    /// # 约束
    /// 幅值过滤为 None 时保持上次幅值和按钮状态；pressed 不是唯一决定因素。类型不验证连接、值域或一致性，启用且保留策略允许时，原始录制仍保存提交值。
    ///
    /// # 生命周期
    /// 事件按值拥有载荷；提交后由实现消费，克隆或录制副本的保留与输入状态更新独立。
    ///
    /// # 参考
    /// 参见 [GamepadButtonAxisSettings](super::GamepadButtonAxisSettings)、[GamepadButtonSettings](super::GamepadButtonSettings)、[InputButton](super::InputButton)、[InputFrameSnapshot](super::InputFrameSnapshot)。
    GamepadButton {
        /// 本次按钮样本的手柄身份。
        ///
        /// # 使用时机与职责
        /// 随所属事件分支按值提交；处理时机和状态保留规则参见该分支。
        ///
        /// # 约束
        /// 与 button 一起构成完整状态键；类型及 DefaultInputManager 的按钮路径不验证连接集合中已经有该身份。
        ///
        /// # 生命周期
        /// 字段按值随事件保存，不借用管理器或宿主输入缓冲。
        ///
        /// # 参考
        /// 参见 [InputEvent::GamepadButton](super::InputEvent::GamepadButton)、[GamepadId](super::GamepadId)。
        gamepad: GamepadId,
        /// 指定手柄上的按钮身份。
        ///
        /// # 使用时机与职责
        /// 随所属事件分支按值提交；处理时机和状态保留规则参见该分支。
        ///
        /// # 约束
        /// 与 gamepad 联合匹配；同名按钮在不同手柄上是不同状态，不提供业务动作含义。
        ///
        /// # 生命周期
        /// 字段按值随事件保存，不借用管理器或宿主输入缓冲。
        ///
        /// # 参考
        /// 参见 [InputEvent::GamepadButton](super::InputEvent::GamepadButton)、[GamepadButton](super::GamepadButton)。
        button: GamepadButton,
        /// 提交的原始按钮或扳机幅值。
        ///
        /// # 使用时机与职责
        /// 随所属事件分支按值提交；处理时机和状态保留规则参见该分支。
        ///
        /// # 约束
        /// DefaultInputManager先过滤非有限值、映射到默认范围并检查变化阈值；发布幅值与原始录制值可以不同。
        ///
        /// # 生命周期
        /// 字段按值随事件保存，不借用管理器或宿主输入缓冲。
        ///
        /// # 参考
        /// 参见 [InputEvent::GamepadButton](super::InputEvent::GamepadButton)、[GamepadButtonAxisSettings](super::GamepadButtonAxisSettings)。
        value: f32,
        /// 宿主提供的按钮按下标志。
        ///
        /// # 使用时机与职责
        /// 随所属事件分支按值提交；处理时机和状态保留规则参见该分支。
        ///
        /// # 约束
        /// DefaultInputManager将此标志与处理后幅值和滞回共同使用；当幅值未发布变化时不会仅凭该标志更新状态。
        ///
        /// # 生命周期
        /// 字段按值随事件保存，不借用管理器或宿主输入缓冲。
        ///
        /// # 参考
        /// 参见 [InputEvent::GamepadButton](super::InputEvent::GamepadButton)、[GamepadButtonSettings](super::GamepadButtonSettings)。
        pressed: bool,
    },
    /// 提交指定手柄轴的原始模拟值。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager 用默认死区、活区与变化阈值处理。返回 Some 时写入持续值，仅在处理值与先前值（缺省为零）不同的情况下记录帧变化；首次接受零值可以建立持续状态而不产生变化边沿。
    ///
    /// # 约束
    /// 非有限输入或阈值内变化可以不改变状态；原始记录仍可能保留事件。类型不验证连接、范围或身份，处理后快照值可与原值不同。
    ///
    /// # 生命周期
    /// 事件按值拥有载荷；提交后由实现消费，克隆或录制副本的保留与输入状态更新独立。
    ///
    /// # 参考
    /// 参见 [GamepadAxisSettings](super::GamepadAxisSettings)、[GamepadAxisState](super::GamepadAxisState)、[GamepadAxisTransition](super::GamepadAxisTransition)。
    GamepadAxis {
        /// 本次轴样本的手柄身份。
        ///
        /// # 使用时机与职责
        /// 随所属事件分支按值提交；处理时机和状态保留规则参见该分支。
        ///
        /// # 约束
        /// 与 axis 联合匹配持续值和边沿；不拥有设备，也不自动加入连接集合。
        ///
        /// # 生命周期
        /// 字段按值随事件保存，不借用管理器或宿主输入缓冲。
        ///
        /// # 参考
        /// 参见 [InputEvent::GamepadAxis](super::InputEvent::GamepadAxis)、[GamepadId](super::GamepadId)。
        gamepad: GamepadId,
        /// 指定手柄上的轴身份。
        ///
        /// # 使用时机与职责
        /// 随所属事件分支按值提交；处理时机和状态保留规则参见该分支。
        ///
        /// # 约束
        /// 与 gamepad 构成完整键；方向、业务动作与消费上下文由上层绑定解释。
        ///
        /// # 生命周期
        /// 字段按值随事件保存，不借用管理器或宿主输入缓冲。
        ///
        /// # 参考
        /// 参见 [InputEvent::GamepadAxis](super::InputEvent::GamepadAxis)、[GamepadAxisState](super::GamepadAxisState)。
        axis: GamepadAxis,
        /// 提交的原始手柄轴值。
        ///
        /// # 使用时机与职责
        /// 随所属事件分支按值提交；处理时机和状态保留规则参见该分支。
        ///
        /// # 约束
        /// DefaultInputManager使用默认死区、活区、夹紧与变化阈值后才发布状态；启用且保留策略允许时，原始记录仍保存提交值，不替换成处理后状态。
        ///
        /// # 生命周期
        /// 字段按值随事件保存，不借用管理器或宿主输入缓冲。
        ///
        /// # 参考
        /// 参见 [InputEvent::GamepadAxis](super::InputEvent::GamepadAxis)、[GamepadAxisSettings](super::GamepadAxisSettings)。
        value: f32,
    },
    /// 提交待宿主执行的反向手柄震动命令。
    ///
    /// # 使用时机与职责
    /// DefaultInputManager仅追加待发队列，由宿主排空领取；快照展示本帧新增后缀。
    ///
    /// # 约束
    /// 载荷 .0 为 GamepadRumbleRequest；提交不执行硬件操作或证明设备连接。ABI 适配可能另行夹紧强度，排空不是执行回执。
    ///
    /// # 生命周期
    /// 事件按值拥有载荷；提交后由实现消费，克隆或录制副本的保留与输入状态更新独立。
    ///
    /// # 参考
    /// 参见 [GamepadRumbleRequest](super::GamepadRumbleRequest)、[InputManager::drain_gamepad_rumble_requests()](super::InputManager::drain_gamepad_rumble_requests)。
    GamepadRumbleRequest(GamepadRumbleRequest),
}
