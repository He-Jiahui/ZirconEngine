//! 插件事件的同步 C 回调载体；声音适配器将交付记录投影为本次借用的请求。
//! 返回的调用状态先判定，再核对结果版本与业务状态；这些值不会管理插件代码的寿命。
use crate::buffer::ZrByteSlice;
use crate::status::{ZrStatus, ZrStatusCode};

/// Stable plugin-event callback ABI used by subsystem-specific adapters.
/// Payload semantics are carried by the namespace and schema strings.
/// # 参数与生命周期
/// 请求字节只读且仅在同步调用中有效；结果指针须对齐、可写并指向完整结果对象。
/// 回调代码须覆盖注册后的全部调用；失败诊断须至少活到宿主随后的同步复制。
/// # 返回值
/// 外层 [`ZrStatus`] 表示调用状态；外层成功后宿主才解释结果中的版本和业务状态。
/// # 参考
/// 声音插件的 `dynamic_event_abi/request.rs::sound_dynamic_event_callback_request` 借用交付记录；
/// `dynamic_event_abi/executor.rs::invoke_sound_dynamic_event_callback` 同步调用并检查两层状态。
pub type ZrPluginEventCallbackFnV1 = unsafe extern "C" fn(
    ZrPluginEventCallbackRequestV1,
    *mut ZrPluginEventCallbackResultV1,
) -> ZrStatus;

/// 请求中的字节视图借用投递对象；插件只可在同步回调期间读取，不可留存指针。
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
/// 只搬运身份与事件字节，不解析 payload、不持有交付对象；所有视图受同步调用期约束。
/// # 参考
/// 声音插件的 request adapter 逐字段投影，`tests/dynamic_events/abi/success.rs` 核对交付；
/// 本 crate 的 `tests/contracts.rs::plugin_event_callback_request_preserves_payload_slices` 核对字节视图。
pub struct ZrPluginEventCallbackRequestV1 {
    /// 请求载体版本；声音适配器填写 Runtime ABI V1。
    pub abi_version: u32,
    /// 借用的子系统事件名称空间。
    pub namespace: ZrByteSlice,
    /// 借用的目标处理器所属插件 ID。
    pub plugin_id: ZrByteSlice,
    /// 借用的目标处理器 ID。
    pub handler_id: ZrByteSlice,
    /// 借用的事件定义 ID。
    pub event_id: ZrByteSlice,
    /// 借用的来源路径；声音适配器对缺省来源填空文本。
    pub source_path: ZrByteSlice,
    /// 交付记录中的事件时间，载体不检验有限性。
    pub time_seconds: f32,
    /// 借用的 payload schema 标识，不在载体中解析。
    pub payload_schema: ZrByteSlice,
    /// 借用的事件字节；格式由名称空间和 schema 决定。
    pub payload: ZrByteSlice,
}

impl ZrPluginEventCallbackRequestV1 {
    /// 返回指定版本的空请求；不验证身份、schema 或时间。
    /// `abi_version` 原样保存，兼容性由调用方核对。
    pub const fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            namespace: ZrByteSlice::empty(),
            plugin_id: ZrByteSlice::empty(),
            handler_id: ZrByteSlice::empty(),
            event_id: ZrByteSlice::empty(),
            source_path: ZrByteSlice::empty(),
            time_seconds: 0.0,
            payload_schema: ZrByteSlice::empty(),
            payload: ZrByteSlice::empty(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 事件业务结果，与函数直接返回的调用状态分开；声音执行器检查两层状态和结果版本。
/// # 约束与参考
/// failed 的诊断仍借用提供方；须活到声音执行器 `status_detail` 完成有界复制。
/// 声音插件的 `tests/abi/support/callbacks/failure.rs` 演示外层成功、业务失败的写回。
pub struct ZrPluginEventCallbackResultV1 {
    /// 宿主在外层调用成功后核验的结果版本。
    pub abi_version: u32,
    /// 处理器业务状态；诊断仍借用提供方内存。
    pub status: ZrStatus,
}

impl ZrPluginEventCallbackResultV1 {
    /// 在调用回调前初始化结果槽为成功；初始化值本身不证明处理器已执行。
    /// `abi_version` 原样保存，兼容性由调用方核对。
    pub const fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            status: ZrStatus::ok(),
        }
    }

    /// 构造明确的业务成功结果，不影响回调函数外层返回状态。
    /// `abi_version` 原样保存，兼容性由调用方核对。
    pub const fn ok(abi_version: u32) -> Self {
        Self {
            abi_version,
            status: ZrStatus::ok(),
        }
    }

    /// 构造 Error 业务状态；diagnostics 不会被复制，须活到宿主读取完成。
    /// `abi_version` 原样保存，兼容性由调用方核对。
    pub const fn failed(abi_version: u32, diagnostics: ZrByteSlice) -> Self {
        Self {
            abi_version,
            status: ZrStatus::new(ZrStatusCode::Error, diagnostics),
        }
    }
}
