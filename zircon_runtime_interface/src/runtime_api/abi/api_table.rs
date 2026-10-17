//! Runtime V8 固定函数表与 Host V1 回调表的 C ABI 载体。
//!
//! # 调用约束与生命周期
//! 宿主先证明提供方、完整表地址和所需槽可用，并在全部调用及输出释放期间持有提供方。
//! 参数中的非空借用区须已初始化并在本次同步调用内可读；输出位置须对齐、可写且独占。
//! 调用方先将拥有分配的输出载体初始化为空，并按返回状态检查和清理输出。
//! [`ZrOwnedResultV2`] 的字节属于原会话，读取结束后只交给该会话的释放入口。
//! 返回状态的诊断仍是同步借用；Rust panic 包装不建立外来地址的内存安全性。

use crate::buffer::{ZrByteSlice, ZrOwnedByteBuffer, ZrOwnedResultV2};
use crate::handles::{ZrRuntimeAllocationId, ZrRuntimeSessionHandle, ZrRuntimeViewportHandle};
pub use crate::profiling::ZrRuntimeProfileControlFnV2;
use crate::status::ZrStatus;
use crate::world_sync::WatchToken;

use super::super::{
    frame::{
        ZrRuntimeFrameDemandV1, ZrRuntimeHighlightSetV1, ZrRuntimeViewportPickRequestV1,
        ZrRuntimeViewportPickResultV1, ZrRuntimeViewportPickTicket,
    },
    session::{
        ZrRuntimeAccessibilityTreeRequestV1, ZrRuntimeAppSessionConfigurationV2,
        ZrRuntimeBindViewportSurfaceRequestV1, ZrRuntimeDrainPluginEventsFnV2, ZrRuntimeEventV1,
        ZrRuntimeFrameRequestV1, ZrRuntimeFrameV2, ZrRuntimeHarvestOperationFnV2,
        ZrRuntimeHostFetchRequestV1, ZrRuntimePollOperationFnV2, ZrRuntimeSessionConfigV3,
        ZrRuntimeSubmitOperationFnV1, ZrRuntimeSubscribePluginEventFnV1,
        ZrRuntimeUnsubscribePluginEventFnV1,
    },
};

/// 获取运行时静态函数表的动态入口；返回指针借用动态库内存。
/// 宿主须在读取表字段及调用函数期间保持库已加载，并先完成表形状与必需槽校验。
///
/// # 参数与返回值
/// 参数是本次调用内可读的完整 Host V1 表，也可为空以表示没有回调。
/// Runtime 当前入口检查 Host 表头后返回静态 V8 表地址，拒绝表头或捕获 panic 时返回空值。
/// 该入口不保存传入的临时 Host 表地址；返回表及槽的有效期依赖动态库仍被持有。
pub type ZrRuntimeGetApiFnV8 = unsafe extern "C" fn(*const ZrHostApiV1) -> *const ZrRuntimeApiV8;
/// 创建会话并把新身份写入调用方的输出位置。
///
/// # 参数与返回值
/// 参数依次是 V3 启动配置和可写的会话身份位置；返回创建状态。
/// 配置中的路径与 profile 字节仅借用本次调用；wake 回调须在会话可能回调的期间有效。
/// 当前入口先将非空输出位置置为无效身份，再验证配置；成功后发布可供后续入口使用的身份。
pub type ZrRuntimeCreateSessionFnV3 =
    unsafe extern "C" fn(ZrRuntimeSessionConfigV3, *mut ZrRuntimeSessionHandle) -> ZrStatus;
/// Standalone AppSession V2 configuration operation; kept outside frozen V8.
/// 为现有会话协商 AppSession V2 配置；宿主按独立符号解析此入口，不增加 V8 表字段。
/// 参数依次为原会话和 V2 描述，返回协商/配置状态；当前入口先验证描述再改变会话路由状态。
pub type ZrRuntimeConfigureAppSessionFnV2 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrRuntimeAppSessionConfigurationV2) -> ZrStatus;
/// 请求销毁参数所指的会话，返回清理状态；调用方须先归还其输出分配。
/// 清理失败的状态需要继续处理，不能仅因发出了请求就卸载仍有活动责任的提供方。
pub type ZrRuntimeDestroySessionFnV1 = unsafe extern "C" fn(ZrRuntimeSessionHandle) -> ZrStatus;
/// 向会话提交 V1 输入事件，参数依次为会话和事件载体，返回处理状态。
/// 当前入口在进入会话执行上下文前按事件字节预算复制 payload，宿主借用只需维持本次调用。
pub type ZrRuntimeHandleEventFnV1 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrRuntimeEventV1) -> ZrStatus;
/// 捕获指定会话/视口的帧，参数依次为会话、V1 帧请求和可写 V2 帧输出位置。
/// 返回捕获状态；成功输出中的 RGBA 通过 [`ZrOwnedResultV2`] 持有分配 ID，读取后须归还。
/// 当前入口核对请求版本、视口和尺寸，并在发布输出前检查 RGBA 字节预算。
pub type ZrRuntimeCaptureFrameFnV2 = unsafe extern "C" fn(
    ZrRuntimeSessionHandle,
    ZrRuntimeFrameRequestV1,
    *mut ZrRuntimeFrameV2,
) -> ZrStatus;
/// 捕获会话/视口的无障碍树，参数依次为会话、树请求和有界 JSON 输出位置。
/// 返回捕获状态；成功的非空结果仍由原会话持有，按分配 ID 释放。
pub type ZrRuntimeCaptureAccessibilityTreeFnV2 = unsafe extern "C" fn(
    ZrRuntimeSessionHandle,
    ZrRuntimeAccessibilityTreeRequestV1,
    *mut ZrOwnedResultV2,
) -> ZrStatus;
/// 归还一次原会话的输出分配；参数依次为发放会话和分配 ID，返回释放状态。
/// 当前注册表核对归属后移除分配；零 ID、错误会话或已释放 ID 返回未找到。
/// 成功释放后所有指向该输出的借用失效，复制 ID 不产生第二份释放权。
///
/// # 约束与生命周期
/// 对有效分配 ID，先检查会话释放准入，再查找分配；会话处于 `Closing` 时也返回
/// `NotFound`，不能仅凭此码推断分配已经回收。销毁失败转入 `TeardownRetryPending`
/// 后仍允许通过原会话归还输出，调用方须继续持有原提供方及未完成的释放责任。
///
/// # 参考
/// `release_runtime_allocation` 先调用 `begin_session_release_action`；后者经
/// `SessionSlot::begin_release_action` 区分关闭阶段和允许释放的待重试阶段。
pub type ZrRuntimeReleaseAllocationFnV2 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrRuntimeAllocationId) -> ZrStatus;
/// 将宿主原生表面绑定到会话/视口，参数依次为会话和绑定请求，返回绑定状态。
/// 宿主维持原生表面有效期，使用相应解绑入口结束绑定后再结束表面所有权。
pub type ZrRuntimeBindViewportSurfaceFnV1 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrRuntimeBindViewportSurfaceRequestV1) -> ZrStatus;
/// 结束参数所指会话与视口的原生表面绑定，返回解绑状态。
/// Host 与编辑器共享绑定登记；会话清理方仍须处理实际成功建立的绑定。
pub type ZrRuntimeUnbindViewportSurfaceFnV1 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrRuntimeViewportHandle) -> ZrStatus;
/// 向已绑定视口呈现帧，参数依次为会话和帧请求，返回呈现状态。
/// 本入口不交付拥有分配的像素输出，截图使用 [`ZrRuntimeCaptureFrameFnV2`]。
pub type ZrRuntimePresentViewportFnV1 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrRuntimeFrameRequestV1) -> ZrStatus;
/// 向会话提交视口高亮集合，参数依次为会话和带实体 ID 借用区的请求。
/// 当前入口先复制实体 ID，再验证并写入会话高亮状态；返回处理状态。
pub type ZrRuntimeSubmitHighlightSetFnV1 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrRuntimeHighlightSetV1) -> ZrStatus;
/// 发起视口拾取，参数依次为会话、拾取请求和可写票据位置，返回提交状态。
/// 当前入口先置无效票据，成功后返回同会话轮询/取消使用的身份；提交成功不等于拾取已完成。
pub type ZrRuntimeRequestViewportPickFnV1 = unsafe extern "C" fn(
    ZrRuntimeSessionHandle,
    ZrRuntimeViewportPickRequestV1,
    *mut ZrRuntimeViewportPickTicket,
) -> ZrStatus;
/// 轮询同一会话的拾取票据，参数依次为会话、票据和可写结果位置。
/// 返回调用状态；拾取是否终结由结果的 disposition 表达，不能只依据状态码判断命中。
pub type ZrRuntimePollViewportPickFnV1 = unsafe extern "C" fn(
    ZrRuntimeSessionHandle,
    ZrRuntimeViewportPickTicket,
    *mut ZrRuntimeViewportPickResultV1,
) -> ZrStatus;
/// 取消参数所指会话的拾取票据，返回取消或未找到等状态。
/// 票据的后续状态由原会话拾取注册表决定，不能转交另一个会话轮询。
pub type ZrRuntimeCancelViewportPickFnV1 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrRuntimeViewportPickTicket) -> ZrStatus;
/// 推进会话一帧并写出调度需求，参数依次为会话和可写需求位置。
/// 返回推进状态；成功需求用于决定下次调度时机，本入口不直接交付像素分配。
pub type ZrRuntimeTickFrameFnV2 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, *mut ZrRuntimeFrameDemandV1) -> ZrStatus;
/// 排出会话发给 Host 的请求批次，参数依次为会话和有界 JSON 输出位置。
/// 返回调用状态；非空输出归原会话分配协议管理，无待处理请求可返回规范空输出。
pub type ZrRuntimeDrainHostRequestsFnV2 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, *mut ZrOwnedResultV2) -> ZrStatus;
/// Host 资源获取回调的类型，参数依次为获取请求和调用方提供的旧式输出位置。
/// 返回获取状态；输出读取后遵循 [`ZrOwnedByteBuffer`] 的原分配方回调协议。
/// 当前 Runtime 动态入口仅检查 Host 表头，不在该入口调用此槽。
pub type ZrRuntimeHostFetchFnV1 =
    unsafe extern "C" fn(ZrRuntimeHostFetchRequestV1, *mut ZrOwnedByteBuffer) -> ZrStatus;
/// 查询会话世界，参数依次为会话、有界 WorldQuery JSON 借用和结果输出位置。
/// 返回查询状态；请求在本次调用内解析，非空 JSON 结果按原会话分配 ID 释放。
pub type ZrRuntimeQueryWorldFnV2 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrByteSlice, *mut ZrOwnedResultV2) -> ZrStatus;
/// 注册世界 watch，参数依次为会话、有界注册 JSON 和可写 WatchToken 位置。
/// 返回注册状态；成功 token 供同会话 [`ZrRuntimeUnwatchWorldFnV1`] 使用。
pub type ZrRuntimeWatchWorldFnV1 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrByteSlice, *mut WatchToken) -> ZrStatus;
/// 撤销参数所指会话的 watch，第三个参数是可写的移除结果字节。
/// 返回调用状态；成功后输出 1 表示移除、0 表示该会话没有该 watch，零 token 被拒绝。
pub type ZrRuntimeUnwatchWorldFnV1 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, WatchToken, *mut u8) -> ZrStatus;
/// 排出会话世界的失效通知，参数依次为会话和有界 JSON 批次输出位置。
/// 返回调用状态；当前入口先准备批次，输出分配成功才提交排出，失败回滚待交付通知。
pub type ZrRuntimeDrainWorldInvalidationsFnV2 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, *mut ZrOwnedResultV2) -> ZrStatus;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// 宿主交给动态入口的回调表；V1 的槽均可为空，能力依赖由后续握手判定。
/// 当前 Runtime 动态入口只借用表来核对元数据；不会在该入口保存或调用回调槽。
pub struct ZrHostApiV1 {
    /// Host V1 载体版本，由 Host 表形状检查核对。
    pub abi_version: u32,
    /// 本地完整表大小；同版本只接受精确大小。
    pub size_bytes: usize,
    /// 可选同步诊断回调，参数是提供方的字节借用。
    pub diagnostics_sink: Option<unsafe extern "C" fn(ZrByteSlice)>,
    /// 可选资源获取回调，使用旧式输出释放协议。
    pub fetch_resource: Option<ZrRuntimeHostFetchFnV1>,
}

impl ZrHostApiV1 {
    /// 返回无回调的表骨架；`abi_version` 原样写入，不代表版本已被接受。
    pub const fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            size_bytes: core::mem::size_of::<Self>(),
            diagnostics_sink: None,
            fetch_resource: None,
        }
    }
}

/// The immutable runtime API V8 table.
///
/// This shape is frozen. Any future field addition requires a new table
/// version and a coordinated hard cutover of all dynamic hosts.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// 表头只描述固定布局；槽是否可用还须按宿主所需能力逐项验证。
/// 复制表只复制函数地址，宿主和编辑器仍须独立持有原提供方及会话生命周期。
pub struct ZrRuntimeApiV8 {
    /// 冻结 V8 函数表版本，区别于各请求 DTO 的版本。
    pub abi_version: u32,
    /// 提供方声明的完整表大小，用于精确形状检查。
    pub size_bytes: usize,
    /// 创建入口，见 [`ZrRuntimeCreateSessionFnV3`] 的配置借用和输出前提。
    pub create_session: Option<ZrRuntimeCreateSessionFnV3>,
    /// 会话清理入口；发出销毁请求不等于清理已成功。
    pub destroy_session: Option<ZrRuntimeDestroySessionFnV1>,
    /// 原会话输出分配的唯一释放入口。
    pub release_allocation: Option<ZrRuntimeReleaseAllocationFnV2>,
    /// 同步提交事件；外来 payload 在会话执行前复制。
    pub handle_event: Option<ZrRuntimeHandleEventFnV1>,
    /// 交付带分配 ID 的 V2 RGBA 帧输出。
    pub capture_frame: Option<ZrRuntimeCaptureFrameFnV2>,
    /// 可选无障碍树捕获入口，输出遵循会话分配协议。
    pub capture_accessibility_tree: Option<ZrRuntimeCaptureAccessibilityTreeFnV2>,
    /// 可选原生表面绑定入口；由宿主持有原生表面。
    pub bind_viewport_surface: Option<ZrRuntimeBindViewportSurfaceFnV1>,
    /// 可选原生表面解绑入口。
    pub unbind_viewport_surface: Option<ZrRuntimeUnbindViewportSurfaceFnV1>,
    /// 可选直接呈现入口，不交付像素输出分配。
    pub present_viewport: Option<ZrRuntimePresentViewportFnV1>,
    /// 同步提交高亮实体集合。
    pub submit_highlight_set: Option<ZrRuntimeSubmitHighlightSetFnV1>,
    /// 可选 profiling/诊断控制入口，使用有界 JSON 请求和会话输出分配。
    pub profile_control: Option<ZrRuntimeProfileControlFnV2>,
    /// 推进会话并返回下次帧调度需求。
    pub tick_frame: Option<ZrRuntimeTickFrameFnV2>,
    /// 可选 Host 请求排出入口；宿主负责执行这些请求。
    pub drain_host_requests: Option<ZrRuntimeDrainHostRequestsFnV2>,
    /// 创建会话内插件事件订阅的入口。
    pub subscribe_plugin_event: Option<ZrRuntimeSubscribePluginEventFnV1>,
    /// 撤销同会话插件事件订阅的入口。
    pub unsubscribe_plugin_event: Option<ZrRuntimeUnsubscribePluginEventFnV1>,
    /// 排出订阅交付页，结果按会话分配协议释放。
    pub drain_plugin_events: Option<ZrRuntimeDrainPluginEventsFnV2>,
    /// 提交操作并返回同会话操作身份。
    pub submit_operation: Option<ZrRuntimeSubmitOperationFnV1>,
    /// 查询操作阶段与结果是否可收割。
    pub poll_operation: Option<ZrRuntimePollOperationFnV2>,
    /// 收割操作 JSON 结果，结果按会话分配协议释放。
    pub harvest_operation: Option<ZrRuntimeHarvestOperationFnV2>,
    /// 有界世界查询入口。
    pub query_world: Option<ZrRuntimeQueryWorldFnV2>,
    /// 注册世界 watch 并返回 token。
    pub watch_world: Option<ZrRuntimeWatchWorldFnV1>,
    /// 撤销同会话世界 watch。
    pub unwatch_world: Option<ZrRuntimeUnwatchWorldFnV1>,
    /// 有界失效通知排出入口。
    pub drain_world_invalidations: Option<ZrRuntimeDrainWorldInvalidationsFnV2>,
    /// 提交拾取请求并返回同会话票据。
    pub request_viewport_pick: Option<ZrRuntimeRequestViewportPickFnV1>,
    /// 轮询拾取票据并写出 disposition。
    pub poll_viewport_pick: Option<ZrRuntimePollViewportPickFnV1>,
    /// 取消同会话拾取票据。
    pub cancel_viewport_pick: Option<ZrRuntimeCancelViewportPickFnV1>,
}

impl ZrRuntimeApiV8 {
    /// 返回精确版本/大小但所有槽为空的表骨架，供构造和契约夹具使用。
    /// 它能通过表头检查，调用方仍须填充并核对实际所需槽后才能使用。
    pub const fn empty() -> Self {
        Self {
            abi_version: crate::version::ZIRCON_RUNTIME_API_VERSION_V8,
            size_bytes: core::mem::size_of::<Self>(),
            create_session: None,
            destroy_session: None,
            release_allocation: None,
            handle_event: None,
            capture_frame: None,
            capture_accessibility_tree: None,
            bind_viewport_surface: None,
            unbind_viewport_surface: None,
            present_viewport: None,
            submit_highlight_set: None,
            profile_control: None,
            tick_frame: None,
            drain_host_requests: None,
            subscribe_plugin_event: None,
            unsubscribe_plugin_event: None,
            drain_plugin_events: None,
            submit_operation: None,
            poll_operation: None,
            harvest_operation: None,
            query_world: None,
            watch_world: None,
            unwatch_world: None,
            drain_world_invalidations: None,
            request_viewport_pick: None,
            poll_viewport_pick: None,
            cancel_viewport_pick: None,
        }
    }
}

// 这些清单由本模块的契约测试与生成的 InterfaceSpec 槽清单核对，包含必需表头字段。
// 运行时形状检查不会消费此清单，宿主仍在加载边界逐槽建立实际调用能力。
/// 冻结表的必需字段清单，供 InterfaceSpec 与源码字段测试交叉核对。
#[cfg(test)]
const ZR_RUNTIME_API_V8_REQUIRED_FIELD_NAMES: &[&str] = &[
    "abi_version",
    "size_bytes",
    "create_session",
    "destroy_session",
    "release_allocation",
    "handle_event",
    "capture_frame",
    "submit_highlight_set",
    "tick_frame",
    "subscribe_plugin_event",
    "unsubscribe_plugin_event",
    "drain_plugin_events",
    "submit_operation",
    "poll_operation",
    "harvest_operation",
    "query_world",
    "watch_world",
    "unwatch_world",
    "drain_world_invalidations",
    "request_viewport_pick",
    "poll_viewport_pick",
    "cancel_viewport_pick",
];
// 可选槽的缺失不由表头检查拒绝；使用相关功能的宿主另行检查能力。
/// 可选功能槽清单，按顺序与生成目录比较，不安装或调用任何函数。
#[cfg(test)]
const ZR_RUNTIME_API_V8_OPTIONAL_FIELD_NAMES: &[&str] = &[
    "capture_accessibility_tree",
    "bind_viewport_surface",
    "unbind_viewport_surface",
    "present_viewport",
    "profile_control",
    "drain_host_requests",
];
// Host V1 保留的两个可选回调槽，供生成目录与具体布局的契约测试比较。
/// Host 表两个可选回调的名称清单，不代表当前动态入口实际使用它们。
#[cfg(test)]
const ZR_HOST_API_V1_OPTIONAL_FIELD_NAMES: &[&str] = &["diagnostics_sink", "fetch_resource"];

#[cfg(test)]
#[path = "tests/api_table.rs"]
mod tests;
