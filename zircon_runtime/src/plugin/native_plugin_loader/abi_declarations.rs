//! 宿主与原生 SDK 共用的 C 布局边界；描述符和入口报告沿用 v3，行为回调另按 v4 协商。
//! 版本号只校验布局，不能验证插件提供的原始指针是否可读。

use std::ffi::{c_char, c_void};

use zircon_runtime_interface::{ZrByteBufferRef, ZrByteSlice, ZrStatus};

pub const ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3: u32 = 3;
pub const ZIRCON_NATIVE_PLUGIN_ABI_VERSION: u32 = ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3;
/// The descriptor and entry-point ABI remain v3. Behavior callbacks are a separately
/// versioned contract and deliberately hard-cut to v4.
pub const ZIRCON_NATIVE_PLUGIN_BEHAVIOR_ABI_VERSION_V4: u32 = 4;
pub const ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH: u32 = 5;
pub const ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V3: &[u8] =
    b"zircon_native_plugin_descriptor_v3\0";
pub const ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL: &[u8] = ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V3;

pub const ZIRCON_NATIVE_PLUGIN_STATUS_OK: u32 = 0;
pub const ZIRCON_NATIVE_PLUGIN_STATUS_ERROR: u32 = 1;
pub const ZIRCON_NATIVE_PLUGIN_STATUS_DENIED: u32 = 2;
pub const ZIRCON_NATIVE_PLUGIN_STATUS_PANIC: u32 = 3;

/// 固定导出符号返回的描述符；探测阶段复制文本，再按身份、ABI 与模块入口筛选。
/// 非空字符串指针须在探测期间指向可读、以 NUL 结尾的 UTF-8 文本。
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginAbiV3 {
    pub abi_version: u32,
    pub plugin_id: *const c_char,
    pub package_manifest_toml: *const c_char,
    pub runtime_entry_name: *const c_char,
    pub editor_entry_name: *const c_char,
    pub requested_capabilities: *const c_char,
}

/// 序列化载荷的版本声明；宿主据此选择命令、事件、注册解析和状态恢复策略。
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginSchemaVersionsV3 {
    pub state_schema_version: u32,
    pub command_manifest_schema: *const c_char,
    pub event_manifest_schema: *const c_char,
    pub registration_manifest_schema: *const c_char,
}

/// runtime/editor 入口的同步返回载体；宿主先读 epoch 再解释其余布局。
/// 文本会被复制，行为及桥回调地址则依赖加载代际保持有效。
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginEntryReportV3 {
    pub layout_epoch: u32,
    pub package_manifest_toml: *const c_char,
    pub diagnostics: *const c_char,
    pub negotiated_capabilities: *const c_char,
    pub required_capabilities: *const c_char,
    pub denied_capabilities: *const c_char,
    pub behavior: *const NativePluginBehaviorV4,
    pub bridge_methods: *const NativePluginBridgeMethodTableV3,
}

/// 本次入口调用借用的宿主表；栈上地址及授权列表在入口返回后失效。
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginHostFunctionTableV3 {
    pub abi_version: u32,
    pub host_handle: u64,
    pub granted_capabilities: *const c_char,
    pub host_abi_version: Option<unsafe extern "C" fn() -> u32>,
    pub host_has_capability: Option<NativePluginHostHasCapabilityFnV3>,
    pub host_log: Option<NativePluginHostLogFnV3>,
    pub host_diagnostic: Option<NativePluginHostDiagnosticFnV3>,
}

/// 本次命令或状态恢复借用的字节；插件不得在回调返回后继续引用。
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginByteSliceV3 {
    pub data: *const u8,
    pub len: usize,
}

/// 插件分配的状态快照；宿主读取后经同一加载代际的 free 回调归还。
/// data、len、capacity 与 owner_token 须保持分配时的配对关系。
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginOwnedByteBufferV3 {
    pub data: *mut u8,
    pub len: usize,
    pub capacity: usize,
    pub owner_token: u64,
    pub free: Option<NativePluginFreeBytesFnV3>,
}

impl NativePluginOwnedByteBufferV3 {
    pub(super) fn empty() -> Self {
        Self {
            data: std::ptr::null_mut(),
            len: 0,
            capacity: 0,
            owner_token: 0,
            free: None,
        }
    }
}

/// 回调的状态与诊断；诊断指针须在宿主完成本次读取前保持有效。
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginCallbackStatusV3 {
    pub code: u32,
    pub diagnostics: *const c_char,
}

/// 单次命令回调的宿主输出通道；context 和 write 仅在当前调用内有效。
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginOutputSinkV4 {
    /// Opaque host-owned context. It is valid only for the active command callback.
    pub context: *mut c_void,
    pub max_output_bytes: usize,
    pub write: Option<NativePluginOutputWriteFnV4>,
}

/// 可选的长期行为回调；其 v4 版本独立于描述符 v3，调用前需持有动态库代际。
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginBehaviorV4 {
    pub abi_version: u32,
    pub is_stateless: u32,
    pub schema_versions: NativePluginSchemaVersionsV3,
    pub command_manifest: *const c_char,
    pub event_manifest: *const c_char,
    pub registration_manifest: *const c_char,
    pub invoke_command: Option<NativePluginInvokeCommandFnV4>,
    pub save_state: Option<NativePluginSaveStateFnV3>,
    pub restore_state: Option<NativePluginRestoreStateFnV3>,
    pub unload: Option<NativePluginUnloadFnV3>,
}

/// 插件导出的候选桥方法；入口先复制名称，安装时再与包清单一一对应。
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginBridgeMethodTableV3 {
    pub abi_version: u32,
    pub methods: *const NativePluginBridgeMethodV3,
    pub method_count: usize,
}

/// 插件以稳定名称声明的方法绑定，user_data 在后续同步调用中原样回传。
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginBridgeMethodV3 {
    pub interface_id: *const c_char,
    pub method_name: *const c_char,
    pub method: Option<NativePluginBridgeMethodFnV3>,
    pub user_data: u64,
}

/// 宿主按清单槽位同步发起的桥调用；负载与输出只借用于本次调用。
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginBridgeMethodCallV3 {
    pub interface_slot: u32,
    pub method_slot: u32,
    pub payload: ZrByteSlice,
    pub output: ZrByteBufferRef,
    pub user_data: u64,
}

pub type NativePluginFreeBytesFnV3 =
    unsafe extern "C" fn(NativePluginOwnedByteBufferV3) -> NativePluginCallbackStatusV3;
pub type NativePluginOutputWriteFnV4 =
    unsafe extern "C" fn(*mut c_void, NativePluginByteSliceV3) -> NativePluginCallbackStatusV3;
pub type NativePluginInvokeCommandFnV4 = unsafe extern "C" fn(
    u32,
    NativePluginByteSliceV3,
    NativePluginOutputSinkV4,
) -> NativePluginCallbackStatusV3;
pub type NativePluginSaveStateFnV3 =
    unsafe extern "C" fn(*mut NativePluginOwnedByteBufferV3) -> NativePluginCallbackStatusV3;
pub type NativePluginRestoreStateFnV3 =
    unsafe extern "C" fn(NativePluginByteSliceV3) -> NativePluginCallbackStatusV3;
pub type NativePluginUnloadFnV3 = unsafe extern "C" fn() -> NativePluginCallbackStatusV3;
pub type NativePluginBridgeMethodFnV3 =
    unsafe extern "C" fn(NativePluginBridgeMethodCallV3) -> ZrStatus;
pub type NativePluginHostHasCapabilityFnV3 =
    unsafe extern "C" fn(*const NativePluginHostFunctionTableV3, *const c_char) -> u32;
pub type NativePluginHostLogFnV3 = unsafe extern "C" fn(
    *const NativePluginHostFunctionTableV3,
    u32,
    *const c_char,
    *const c_char,
) -> u32;
pub type NativePluginHostDiagnosticFnV3 = unsafe extern "C" fn(
    *const NativePluginHostFunctionTableV3,
    *const c_char,
    f64,
    *const c_char,
    *const c_char,
) -> u32;
