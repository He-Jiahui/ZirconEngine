//! 动态 Runtime 发往平台宿主的批次请求，以及 Gameplay 场景切换意图载体。
//!
//! `ZrRuntimeHostRequestV1` 由动态会话排队、分页编码并经 FFI drain；场景切换结构另由 Gameplay
//! host 写入当前 World 的 typed resource，不会因为定义在本文件中就进入该批次。

use crate::handles::ZrRuntimeViewportHandle;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU64;

use super::{
    ZrRuntimeClipboardHostRequestV1, ZrRuntimeUiActionHostRequestV1, ZrRuntimeUiHostRequestV1,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 运行时经拥有者缓冲区批量交给宿主的 JSON 请求。
/// 宿主先验证批次版本和输出预算，再按各变体路由到平台服务。
///
/// # 使用时机与生命周期
/// 动态会话从 UI、输入管理器收集并排空待处理队列，在有界输出调用中编码请求前缀；成功提交
/// 后移除该页，失败回滚时保留待发请求。App 在帧内 drain 后依序路由每个变体。
///
/// # 参数
/// `abi_version` 由 Runtime 填为当前 ABI 版本；`requests` 保留收集顺序并携带各服务的 typed 请求。
///
/// # 约束
/// 批次只约束 JSON 编码预算和 ABI 版本，不验证各载荷的业务字段。场景切换 request/result
/// 不是此批次的成员。
///
/// # 参考
/// `zircon_runtime::dynamic_api::session::state::prepare_host_request_output`、
/// `zircon_app::entry::runtime_entry_app::host_requests::drain::apply_runtime_host_requests`。
pub struct ZrRuntimeHostRequestBatchV1 {
    /// Runtime ABI 版本；App 的 RuntimeLibrary drain 在返回请求前检查该字段。
    pub abi_version: u32,
    /// 按 Runtime 收集顺序待宿主处理的 typed 请求列表。
    pub requests: Vec<ZrRuntimeHostRequestV1>,
}

impl ZrRuntimeHostRequestBatchV1 {
    /// 建立指定 ABI 版本与请求列表的 batch 值，不检查版本或编码预算。
    ///
    /// # 参数
    /// `abi_version` 标识载荷版本，`requests` 是已收集的宿主请求。
    ///
    /// # 返回值
    /// 返回待动态会话分页编码的 batch。
    pub fn new(abi_version: u32, requests: Vec<ZrRuntimeHostRequestV1>) -> Self {
        Self {
            abi_version,
            requests,
        }
    }

    /// 建立无请求的 batch；由调用者提供 ABI 版本。
    ///
    /// # 返回值
    /// `requests` 为空的 batch，仍需由 Runtime 输出边界编码或提交。
    pub fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            requests: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// Runtime 动态会话可交给 App 的宿主请求；震动的新旧 wire variant 显式区分。
///
/// # 使用时机与生命周期
/// 该 enum 是 [`ZrRuntimeHostRequestBatchV1::requests`] 的元素类型；App drain 后按 variant 分派。
/// UI IME/clipboard 仍使用专用 request 载体，`UiAction` 和 `UiHost` 是两个不同的 UI 通道。
///
/// # 约束
/// enum 仅承载数据。App 当前执行 IME、gamepad、cursor、clipboard 分支；UI Action/Host 分支先核对
/// viewport，再记录未处理诊断。各请求由哪个 Runtime 队列生成见 `collect_host_requests`。
pub enum ZrRuntimeHostRequestV1 {
    /// 输入法窗口能力、光标区域或 surrounding text 更新。
    Ime(ZrRuntimeImeHostRequestV1),
    /// Legacy unversioned rumble. Retained for decoding; current hosts reject it without effects.
    GamepadRumble(ZrRuntimeGamepadRumbleRequestV1),
    /// Rumble bound to the connection captured at input enqueue time.
    GamepadRumbleV2(ZrRuntimeGamepadRumbleRequestV2),
    /// 光标可见性、抓取、命中测试或位置请求。
    Cursor(ZrRuntimeCursorHostRequestV1),
    /// 带 surface/transfer 身份的剪贴板事务。
    Clipboard(ZrRuntimeClipboardHostRequestV1),
    /// Runtime UI 组件 action/route 派发请求。
    UiAction(ZrRuntimeUiActionHostRequestV1),
    /// 不属于 IME/clipboard 专用通道的通用 UI effect 请求。
    UiHost(ZrRuntimeUiHostRequestV1),
}

impl ZrRuntimeHostRequestV1 {
    /// 将 IME 载体包装为批次 enum variant。
    pub fn ime(request: ZrRuntimeImeHostRequestV1) -> Self {
        Self::Ime(request)
    }

    /// 将 gamepad rumble 载体包装为批次 enum variant。
    pub fn gamepad_rumble(request: ZrRuntimeGamepadRumbleRequestV1) -> Self {
        Self::GamepadRumble(request)
    }

    /// Preserve the captured generation in a distinct serde variant. No legacy fallback is safe.
    pub fn gamepad_rumble_v2(request: ZrRuntimeGamepadRumbleRequestV2) -> Self {
        Self::GamepadRumbleV2(request)
    }

    /// 将 cursor 载体包装为批次 enum variant。
    pub fn cursor(request: ZrRuntimeCursorHostRequestV1) -> Self {
        Self::Cursor(request)
    }

    /// 将 clipboard 事务包装为批次 enum variant。
    pub fn clipboard(request: ZrRuntimeClipboardHostRequestV1) -> Self {
        Self::Clipboard(request)
    }

    /// 将 Runtime UI action 包装为批次 enum variant。
    pub fn ui_action(request: ZrRuntimeUiActionHostRequestV1) -> Self {
        Self::UiAction(request)
    }

    /// 将通用 Runtime UI effect 包装为批次 enum variant。
    pub fn ui_host(request: ZrRuntimeUiHostRequestV1) -> Self {
        Self::UiHost(request)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
/// Gameplay producer 写入场景请求的策略标签；当前仅有且仅由该 producer 使用 `ReplaceActive`。
///
/// 该值只表达请求意图；实际资源加载、切换和结果生成由后续消费流程决定。
pub enum ZrRuntimeProjectSceneTransitionPolicyV1 {
    #[default]
    /// 请求替换活动场景；该值不加载资源，何时完成由消费流程决定。
    ReplaceActive,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 游戏逻辑发起的项目场景切换意图；构造器仅校验 `res://` URI 的词法形式。
/// 构造检查不探测目标资源或执行切换。动态会话在成功 tick 后或 host-request drain 边界消费请求。
///
/// 该类型不是 [`ZrRuntimeHostRequestV1`] 的 variant；Gameplay host 将它写入当前 World 的 typed
/// resource。`try_new` 只做 URI 词法检查，Serde 反序列化也不会调用该构造器。
///
/// `zircon_runtime::dynamic_api::session::scene_transition` 加载并替换活动 World，随后将
/// [`ZrRuntimeProjectSceneTransitionResultV1`] 写入当前 World；加载失败保留原场景。
pub struct ZrRuntimeProjectSceneTransitionRequestV1 {
    /// Gameplay producer 分配的请求 ID；直接构造时不检查范围或唯一性。
    pub request_id: u64,
    /// `res://` 开头的项目相对资源路径；构造器只检查词法形式，不探测资源是否存在。
    pub scene_uri: String,
    /// 交由后续场景切换流程解释的策略。
    pub policy: ZrRuntimeProjectSceneTransitionPolicyV1,
}

impl ZrRuntimeProjectSceneTransitionRequestV1 {
    /// 转成 owned URI 并验证项目资源路径的词法约束。
    ///
    /// # 参数
    /// `request_id` 被原样保存；`scene_uri` 转换为 `String` 后检查；`policy` 被原样保存。
    ///
    /// # 返回值
    /// 合法词法路径返回 request；否则返回相应 [`ZrRuntimeProjectSceneTransitionRequestErrorV1`]。
    ///
    /// # 约束
    /// 不检查 `request_id`、目标资源存在性或切换能力。类型派生的 `Deserialize` 可直接构造值，
    /// 不经过此检查。Gameplay producer 在成功后才推进其 per-World 请求序号并写入 request resource。
    ///
    /// # 参考
    /// `zircon_runtime::script::vm::gameplay_host::scene_transition::request_scene_transition`。
    pub fn try_new(
        request_id: u64,
        scene_uri: impl Into<String>,
        policy: ZrRuntimeProjectSceneTransitionPolicyV1,
    ) -> Result<Self, ZrRuntimeProjectSceneTransitionRequestErrorV1> {
        let scene_uri = scene_uri.into();
        validate_project_scene_uri(&scene_uri)?;
        Ok(Self {
            request_id,
            scene_uri,
            policy,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 场景切换结果中的状态分类标签；本接口类型本身不执行状态转换。
pub enum ZrRuntimeProjectSceneTransitionStatusV1 {
    /// 结果生产者用此标签报告请求成功。
    Succeeded,
    /// 结果生产者用此标签报告请求失败。
    Failed,
    /// 结果生产者用此标签说明请求被较新的请求取代；该标签不实施取代动作。
    Superseded,
    /// 结果生产者用此标签说明请求被拒绝。
    Rejected,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 场景切换请求的最近一次完成结果，由动态会话写入活动 World 的 typed resource。
///
/// `request_id` 与 URI 可关联来源；status 和可选 diagnostic 描述结果。该结构没有构造器或自动事件发送逻辑。
/// Gameplay 脚本用 `scene_transition_result_json()` 读取（无结果时返回 JSON null）；读取不消费结果。
pub struct ZrRuntimeProjectSceneTransitionResultV1 {
    /// 与原 request 相同的请求关联 ID。
    pub request_id: u64,
    /// 与原 request 相同的项目资源 URI。
    pub scene_uri: String,
    /// 完成、失败、被替代或被拒绝的状态分类。
    pub status: ZrRuntimeProjectSceneTransitionStatusV1,
    /// 可选诊断文本；缺省时 Serde 解码为 `None`，编码时省略该键。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostic: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
/// [`ZrRuntimeProjectSceneTransitionRequestV1::try_new`] 的 URI 词法校验失败原因。
pub enum ZrRuntimeProjectSceneTransitionRequestErrorV1 {
    #[error("project scene URI must use the res:// scheme")]
    /// URI 未以 `res://` scheme 开头。
    MissingResourceScheme,
    #[error("project scene URI must name a project-relative asset")]
    /// `res://` 后没有路径。
    EmptyResourcePath,
    #[error("project scene URI must use canonical forward-slash segments")]
    /// 整个 URI 首尾含空白，或路径含控制字符、非规范斜杠、`?`/`#`、空段或 `.`/`..` 段。
    NonCanonicalResourcePath,
}

/// 只校验 `try_new` 所需的项目资源 URI 词法形式，不查资源注册表或文件系统。
///
/// # 返回值
/// 合法时返回 `Ok(())`；失败分类区分缺 scheme、空路径和非规范路径。
fn validate_project_scene_uri(
    scene_uri: &str,
) -> Result<(), ZrRuntimeProjectSceneTransitionRequestErrorV1> {
    let Some(path) = scene_uri.strip_prefix("res://") else {
        return Err(ZrRuntimeProjectSceneTransitionRequestErrorV1::MissingResourceScheme);
    };
    if path.is_empty() {
        return Err(ZrRuntimeProjectSceneTransitionRequestErrorV1::EmptyResourcePath);
    }
    // 按原始字符串拒绝所列斜杠、路径段、分隔符和控制字符；不解码 URI，也不查资源或文件系统。
    if scene_uri.trim() != scene_uri
        || path.starts_with('/')
        || path.ends_with('/')
        || path.contains('\\')
        || path.contains('?')
        || path.contains('#')
        || path.chars().any(char::is_control)
        || path
            .split('/')
            .any(|segment| segment.is_empty() || matches!(segment, "." | ".."))
    {
        return Err(ZrRuntimeProjectSceneTransitionRequestErrorV1::NonCanonicalResourcePath);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
/// 单个 cursor 平台更新；kind 决定 `grab_mode`、`position` 与 `value` 中哪些字段有效。
///
/// Runtime producer 使用下方 helpers 生成一致组合。直接构造或反序列化可以产生缺少所需 option 的值；
/// App 对缺少 grab/position 的分支只警告并跳过该项。
pub struct ZrRuntimeCursorHostRequestV1 {
    /// 选择 App 要执行的 cursor 操作。
    pub kind: ZrRuntimeCursorHostRequestKindV1,
    /// `SetGrabMode` 专用载荷；其他 helper 留为 `None`。
    pub grab_mode: Option<ZrRuntimeCursorGrabModeV1>,
    /// `SetPosition` 的逻辑窗口坐标；其他 helper 留为 `None`。
    pub position: Option<ZrRuntimeCursorPositionV1>,
    /// `SetVisible` 的可见值或 `SetHitTest` 的命中测试值；其他种类使用固定 false 占位。
    pub value: bool,
}

impl ZrRuntimeCursorHostRequestV1 {
    /// 构造 SetVisible 请求载体。
    /// # 参数与返回值
    /// visible 是窗口可见目标；返回携带该值的载体。
    /// # 约束
    /// 仅组装请求，窗口状态由 App 消费时修改。
    pub const fn set_visible(visible: bool) -> Self {
        Self {
            kind: ZrRuntimeCursorHostRequestKindV1::SetVisible,
            grab_mode: None,
            position: None,
            value: visible,
        }
    }
    /// 构造 SetGrabMode 请求载体。
    /// # 参数与返回值
    /// grab_mode 选择 None/Confined/Locked；返回携带该模式的请求。
    /// # 约束
    /// App 消费时按平台能力尝试替代模式；本方法不执行抓取或探测支持。
    pub const fn set_grab_mode(grab_mode: ZrRuntimeCursorGrabModeV1) -> Self {
        Self {
            kind: ZrRuntimeCursorHostRequestKindV1::SetGrabMode,
            grab_mode: Some(grab_mode),
            position: None,
            value: false,
        }
    }
    /// 构造 SetHitTest 请求载体。
    /// # 参数与返回值
    /// hit_test 原样交给 Winit；返回携带该值的请求。
    /// # 约束
    /// Winit 调用发生在 App 消费时，本方法不修改窗口。
    pub const fn set_hit_test(hit_test: bool) -> Self {
        Self {
            kind: ZrRuntimeCursorHostRequestKindV1::SetHitTest,
            grab_mode: None,
            position: None,
            value: hit_test,
        }
    }
    /// 构造 SetPosition 请求载体。
    /// # 参数与返回值
    /// position 是窗口逻辑坐标；返回携带该值的请求。
    /// # 约束
    /// 本方法不检查有限性，App 调用 Winit 前验证。
    pub const fn set_position(position: ZrRuntimeCursorPositionV1) -> Self {
        Self {
            kind: ZrRuntimeCursorHostRequestKindV1::SetPosition,
            grab_mode: None,
            position: Some(position),
            value: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// Cursor 宿主请求的操作标签；与请求 payload 字段配对。
pub enum ZrRuntimeCursorHostRequestKindV1 {
    /// 调整 cursor visible 标志。
    SetVisible,
    /// 调整窗口的 cursor grab 模式。
    SetGrabMode,
    /// 调整窗口 cursor hit-test 标志。
    SetHitTest,
    /// 调整窗口光标位置。
    SetPosition,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 传给 App 窗口的 pointer grab 目标；受平台能力限制时 App 可尝试相邻模式。
pub enum ZrRuntimeCursorGrabModeV1 {
    /// 不抓取指针。
    None,
    /// 首选限制在窗口内；App 在平台拒绝时尝试 Locked。
    Confined,
    /// 首选锁定指针；App 在平台拒绝时尝试 Confined。
    Locked,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
/// App 以 Winit `LogicalPosition` 提交的二维窗口坐标。
pub struct ZrRuntimeCursorPositionV1 {
    /// 横向逻辑坐标；App 会拒绝非有限值。
    pub x: f32,
    /// 纵向逻辑坐标；App 会拒绝非有限值。
    pub y: f32,
}

impl ZrRuntimeCursorPositionV1 {
    /// # 参数与返回值
    /// x、y 是窗口相对逻辑坐标；返回保存该坐标的值。
    /// # 约束
    /// 不验证有限性或窗口范围，后者由 App 消费路径处理。
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
/// Legacy unversioned rumble wire value. Current hosts reject it; its serde shape stays intact.
pub struct ZrRuntimeGamepadRumbleRequestV1 {
    /// Runtime 输入层使用的 gamepad 身份值。
    pub gamepad_id: u64,
    /// `Add` 提交一次定时效果，`Stop` 停止该 gamepad 当前效果。
    pub kind: ZrRuntimeGamepadRumbleRequestKindV1,
    /// Add 请求的强马达强度；Runtime producer 通过源 intensity 的 `clamped()` 生成。
    pub strong_motor: f32,
    /// Add 请求的弱马达强度；Runtime producer 通过源 intensity 的 `clamped()` 生成。
    pub weak_motor: f32,
    /// Add 请求的效果时长，单位毫秒；Stop helper 置零。
    pub duration_millis: u32,
}

impl ZrRuntimeGamepadRumbleRequestV1 {
    /// 创建提交马达振动的 typed request。
    ///
    /// # 参数
    /// `gamepad_id` 标识设备；`strong_motor`、`weak_motor` 与 `duration_millis` 来自 Runtime 振动强度和时长。
    ///
    /// # 返回值
    /// 返回 `kind = Add` 的值；该构造器不 clamp 强度，动态 Runtime producer 会在调用前 clamp。
    pub const fn add(
        gamepad_id: u64,
        strong_motor: f32,
        weak_motor: f32,
        duration_millis: u32,
    ) -> Self {
        Self {
            gamepad_id,
            kind: ZrRuntimeGamepadRumbleRequestKindV1::Add,
            strong_motor,
            weak_motor,
            duration_millis,
        }
    }
    /// 为 gamepad_id 生成 Stop 请求。
    /// # 参数与返回值
    /// ID 标识设备；返回强弱马达强度和时长均为零的载体。
    /// # 约束
    /// 仅组装旧载体；当前 App 明确拒绝没有连接代际的震动请求。
    pub const fn stop(gamepad_id: u64) -> Self {
        Self {
            gamepad_id,
            kind: ZrRuntimeGamepadRumbleRequestKindV1::Stop,
            strong_motor: 0.0,
            weak_motor: 0.0,
            duration_millis: 0,
        }
    }
}

/// Connection-bound rumble JSON payload. This is not passed by value through the C API.
///
/// The generation is mandatory and nonzero, encoded as an integer. The V8 C table,
/// drain function and owned-result layouts remain unchanged; the outer V1 JSON batch uses
/// the distinct GamepadRumbleV2 tag. Older hosts reject that unknown tag rather than execute
/// an unversioned fallback. Compatible execution requires a matched host/runtime build.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZrRuntimeGamepadRumbleRequestV2 {
    pub gamepad_id: u64,
    pub generation: NonZeroU64,
    pub kind: ZrRuntimeGamepadRumbleRequestKindV1,
    pub strong_motor: f32,
    pub weak_motor: f32,
    pub duration_millis: u32,
}

impl ZrRuntimeGamepadRumbleRequestV2 {
    pub const fn add(
        gamepad_id: u64,
        generation: NonZeroU64,
        strong_motor: f32,
        weak_motor: f32,
        duration_millis: u32,
    ) -> Self {
        Self {
            gamepad_id,
            generation,
            kind: ZrRuntimeGamepadRumbleRequestKindV1::Add,
            strong_motor,
            weak_motor,
            duration_millis,
        }
    }

    pub const fn stop(gamepad_id: u64, generation: NonZeroU64) -> Self {
        Self {
            gamepad_id,
            generation,
            kind: ZrRuntimeGamepadRumbleRequestKindV1::Stop,
            strong_motor: 0.0,
            weak_motor: 0.0,
            duration_millis: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// Gamepad rumble 请求的操作类别。
pub enum ZrRuntimeGamepadRumbleRequestKindV1 {
    /// 添加/提交一次有时长的马达振动效果。
    Add,
    /// 停止指定 gamepad 的振动效果。
    Stop,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// IME 平台操作的序列化载体；当前生产者总是附带目标视口。
/// 缺省视口只用于读取旧格式，宿主仍须在应用操作前核对归属。
///
/// `enable`、`disable`、`set_cursor_area` 和 `set_surrounding_text` 先生成未路由载体，Runtime
/// producer 随后调用 `with_target_viewport`。直接调用 helper 后输出时仍可能保留 `None`；App 只允许
/// 该缺省值落到默认 viewport。每个 kind 对应的其他 Option 字段由 App 在执行前检查。
///
/// # 参考
/// `zircon_runtime::dynamic_api::session::host_requests::runtime_ime_host_request`；
/// `zircon_app::entry::runtime_entry_app::host_requests::routing::ime_request_targets_viewport`。
pub struct ZrRuntimeImeHostRequestV1 {
    /// 选择 IME 操作；`SetCursorArea` 使用 `cursor_area`，`SetSurroundingText` 使用 `surrounding_text`。
    /// App 执行对应更新时检查对应载荷是否存在；缺失时记录警告并跳过该项。
    pub kind: ZrRuntimeImeHostRequestKindV1,
    /// The host viewport that owns this IME session. Missing only decodes older
    /// serialized output; current runtime producers always set this target.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_viewport: Option<ZrRuntimeViewportHandle>,
    /// `SetCursorArea` 的 window-logical 矩形；其他 kind 通常为 `None`。
    pub cursor_area: Option<ZrRuntimeImeCursorAreaV1>,
    /// `SetSurroundingText` 的文本窗口与 caret offsets；Winit V1 adapter 当前只提交 value/cursor/anchor。
    pub surrounding_text: Option<ZrRuntimeImeSurroundingTextV1>,
}

impl ZrRuntimeImeHostRequestV1 {
    /// 返回未绑定 viewport 的 Enable 载体；producer 应先绑定目标，App 消费时才启用 IME。
    pub fn enable() -> Self {
        Self {
            kind: ZrRuntimeImeHostRequestKindV1::Enable,
            target_viewport: None,
            cursor_area: None,
            surrounding_text: None,
        }
    }
    /// 返回未绑定 viewport 的 Disable 载体；producer 应先绑定目标，App 消费时才停用 IME。
    pub fn disable() -> Self {
        Self {
            kind: ZrRuntimeImeHostRequestKindV1::Disable,
            target_viewport: None,
            cursor_area: None,
            surrounding_text: None,
        }
    }
    /// 构造未绑定 viewport 的 SetCursorArea 载体。
    /// # 参数与返回值
    /// area 是窗口逻辑矩形；返回保存该值的请求。
    /// # 约束
    /// producer 应先绑定目标，App 消费时才更新平台。
    pub fn set_cursor_area(area: ZrRuntimeImeCursorAreaV1) -> Self {
        Self {
            kind: ZrRuntimeImeHostRequestKindV1::SetCursorArea,
            target_viewport: None,
            cursor_area: Some(area),
            surrounding_text: None,
        }
    }
    /// 构造未绑定 viewport 的 SetSurroundingText 载体。
    /// # 参数与返回值
    /// text 的位置为 UTF-8 byte offset；返回保存该窗口文本的请求。
    /// # 约束
    /// producer 应先绑定目标；本方法不校验偏移，App 消费时才更新平台。
    pub fn set_surrounding_text(text: ZrRuntimeImeSurroundingTextV1) -> Self {
        Self {
            kind: ZrRuntimeImeHostRequestKindV1::SetSurroundingText,
            target_viewport: None,
            cursor_area: None,
            surrounding_text: Some(text),
        }
    }
    /// # 参数与返回值
    /// target_viewport 是后续路由核对的句柄；返回写入该目标的更新载体。
    /// # 约束
    /// 只写路由字段，不校验句柄归属或执行 IME 操作。
    pub const fn with_target_viewport(mut self, target_viewport: ZrRuntimeViewportHandle) -> Self {
        self.target_viewport = Some(target_viewport);
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// IME 宿主请求的操作类别；App 根据 kind 读取相应可选字段并调用 Winit。
pub enum ZrRuntimeImeHostRequestKindV1 {
    /// 启用窗口 IME 能力。
    Enable,
    /// 停用窗口 IME 能力。
    Disable,
    /// 更新候选窗口使用的光标几何。
    SetCursorArea,
    /// 更新 IME surrounding text 窗口。
    SetSurroundingText,
}

/// IME cursor rectangles are window-relative logical pixels. The app submits
/// these values to winit's logical position and size APIs without DPI scaling.
/// IME 矩形使用窗口相对逻辑像素；App 直接传给 Winit 逻辑位置/尺寸接口，不再乘 DPI 比例。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZrRuntimeImeCoordinateSpaceV1 {
    #[default]
    /// 以窗口为参照的逻辑像素；App 直接使用 Winit logical position/size，不再乘 DPI 比例。
    WindowLogical,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
/// 输入法插入点或候选窗口矩形；坐标以窗口为参照，单位为逻辑像素。
///
/// App 的普通窗口更新直接转换为 Winit logical position/size；native candidate 路径另行验证几何范围。
pub struct ZrRuntimeImeCursorAreaV1 {
    /// 矩形左边界的窗口逻辑坐标。
    pub x: f32,
    /// 矩形上边界的窗口逻辑坐标。
    pub y: f32,
    /// 逻辑像素宽度；App 路径允许零宽 caret，native candidate 路径要求有限且非负。
    pub width: f32,
    /// 逻辑像素高度；App 普通窗口更新允许零，高级 native candidate 路径要求大于零。
    pub height: f32,
    /// 坐标系；旧 JSON 缺字段时按 `WindowLogical` 解码。
    #[serde(default)]
    pub coordinate_space: ZrRuntimeImeCoordinateSpaceV1,
}

impl ZrRuntimeImeCursorAreaV1 {
    /// # 参数与返回值
    /// x/y 是左上点，width/height 是窗口逻辑像素；返回 WindowLogical 矩形。
    /// # 约束
    /// 不验证几何值，App 各消费路径调用平台 API 前检查。
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            coordinate_space: ZrRuntimeImeCoordinateSpaceV1::WindowLogical,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// IME surrounding text 窗口；所有光标/选区位置均以该 `value` 的 UTF-8 字节偏移计。
pub struct ZrRuntimeImeSurroundingTextV1 {
    /// caret 周围发送给 IME 的文本窗口。
    pub value: String,
    /// caret 位置在 `value` 中的 UTF-8 byte offset。
    pub cursor: usize,
    /// selection anchor 在 `value` 中的 UTF-8 byte offset。
    pub anchor: usize,
    /// 可选的活动 composition 范围；字段缺省时解码为 `None`，为空时 JSON 不输出。
    ///
    /// TODO: [ZCA-HRQ-002] Runtime producer 会把内部 surrounding text 中的可选范围搬运到此字段，但当前 App V1 `SetSurroundingText` adapter
    /// 只向 Winit 传 `value`、`cursor`、`anchor`。确认此字段供其他 host/version 使用，还是本平台应转交；
    /// 下一步核对生成 request → 平台适配 → Runtime 回传的集成契约。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub composition_range: Option<ZrRuntimeImeTextRangeV1>,
}

impl ZrRuntimeImeSurroundingTextV1 {
    /// # 参数与返回值
    /// value 转为 owned string，cursor/anchor 为 UTF-8 byte offset；返回无 composition range 的载体。
    /// # 约束
    /// 不校验偏移范围或字符边界；Runtime producer 另行裁剪文本窗口。
    pub fn new(value: impl Into<String>, cursor: usize, anchor: usize) -> Self {
        Self {
            value: value.into(),
            cursor,
            anchor,
            composition_range: None,
        }
    }
    /// # 参数与返回值
    /// composition_range 边界为文本 UTF-8 byte offset；返回设置为该 Some/None 值的载体。
    /// # 约束
    /// 不校验顺序、字符串长度或 UTF-8 字符边界。
    pub fn with_composition_range(
        mut self,
        composition_range: Option<ZrRuntimeImeTextRangeV1>,
    ) -> Self {
        self.composition_range = composition_range;
        self
    }
}

/// UTF-8 byte offsets into the surrounding-text window.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// `value` surrounding window 中的一段 UTF-8 字节范围。
pub struct ZrRuntimeImeTextRangeV1 {
    /// 范围起始字节偏移。
    pub start: usize,
    /// 范围结束字节偏移。
    pub end: usize,
}

impl ZrRuntimeImeTextRangeV1 {
    /// # 参数与返回值
    /// start/end 是父文本窗口中的 UTF-8 byte offset；返回保存这两个边界的值。
    /// # 约束
    /// 不验证顺序、长度或字符边界，有效性由具体 producer/consumer 契约决定。
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[cfg(test)]
#[path = "tests/host_requests.rs"]
mod tests;
