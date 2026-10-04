//! Native 插件 ABI 数据载体与序列化辅助函数。
//! Native-only 包可直接依赖本模块；runtime loader 在独立边界读取并校验导出的 ABI 数据。
use std::ffi::{c_char, c_void, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};

mod owned_buffers;

pub use owned_buffers::{NativePluginOwnedBytesError, NativePluginOwnedBytesErrorKind};

use serde::{Deserialize, Serialize};
pub use zircon_runtime_interface::{ZrByteBufferRef, ZrByteSlice, ZrStatus};

pub const ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3: u32 = 3;
pub const ZIRCON_NATIVE_PLUGIN_ABI_VERSION: u32 = ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3;
pub const ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH: u32 = 5;
pub const ZIRCON_NATIVE_PLUGIN_BEHAVIOR_ABI_VERSION_V4: u32 = 4;
pub const ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V3: &[u8] =
    b"zircon_native_plugin_descriptor_v3\0";
pub const ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL: &[u8] = ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V3;

pub const ZIRCON_NATIVE_PLUGIN_STATUS_OK: u32 = 0;
pub const ZIRCON_NATIVE_PLUGIN_STATUS_ERROR: u32 = 1;
pub const ZIRCON_NATIVE_PLUGIN_STATUS_DENIED: u32 = 2;
pub const ZIRCON_NATIVE_PLUGIN_STATUS_PANIC: u32 = 3;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// Native 描述符入口返回的包身份、清单文本、运行时/编辑器入口名与请求能力。
pub struct NativePluginAbiV3 {
    pub abi_version: u32,
    pub plugin_id: *const c_char,
    pub package_manifest_toml: *const c_char,
    pub runtime_entry_name: *const c_char,
    pub editor_entry_name: *const c_char,
    pub requested_capabilities: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// 声明行为载体引用的状态、命令、事件与注册清单 schema 版本。
pub struct NativePluginSchemaVersionsV3 {
    pub state_schema_version: u32,
    pub command_manifest_schema: *const c_char,
    pub event_manifest_schema: *const c_char,
    pub registration_manifest_schema: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// 入口协商结果；Host 通过该结构读取诊断、能力结果及行为和桥接表指针。
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

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// Host 在 Native 回调期间提供的 ABI 版本、句柄、能力集合与宿主回调。
pub struct NativePluginHostFunctionTableV3 {
    pub abi_version: u32,
    pub host_handle: u64,
    pub granted_capabilities: *const c_char,
    pub host_abi_version: Option<unsafe extern "C" fn() -> u32>,
    pub host_has_capability: Option<NativePluginHostHasCapabilityFnV3>,
    pub host_log: Option<NativePluginHostLogFnV3>,
    pub host_diagnostic: Option<NativePluginHostDiagnosticFnV3>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// 跨 ABI 传递的借用字节视图，由数据指针和长度组成。
pub struct NativePluginByteSliceV3 {
    pub data: *const u8,
    pub len: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// 跨 ABI 交接的 SDK 所有字节缓冲区；owned_bytes 同时填入释放回调与所有权令牌。
pub struct NativePluginOwnedByteBufferV3 {
    pub data: *mut u8,
    pub len: usize,
    pub capacity: usize,
    pub owner_token: u64,
    pub free: Option<NativePluginFreeBytesFnV3>,
}

impl NativePluginOwnedByteBufferV3 {
    pub const fn empty() -> Self {
        Self {
            data: std::ptr::null_mut(),
            len: 0,
            capacity: 0,
            owner_token: 0,
            free: None,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// 回调返回码与指向静态诊断文本的 C 字符串指针。
pub struct NativePluginCallbackStatusV3 {
    pub code: u32,
    pub diagnostics: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// Host 在命令回调期间提供的有界输出写入上下文。
pub struct NativePluginOutputSinkV4 {
    pub context: *mut c_void,
    pub max_output_bytes: usize,
    pub write: Option<NativePluginOutputWriteFnV4>,
}

impl NativePluginOutputSinkV4 {
    /// Writes one borrowed chunk into the host-owned command output.
    ///
    /// # Safety
    ///
    /// The sink must be the unmodified value supplied by the host for the active callback. Its
    /// context and writer are valid only for that callback's duration.
    pub unsafe fn write(self, bytes: &[u8]) -> NativePluginCallbackStatusV3 {
        if bytes.len() > self.max_output_bytes {
            return callback_status(
                ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
                NATIVE_OUTPUT_SINK_LIMIT_EXCEEDED_DIAGNOSTICS_V4,
            );
        }
        let Some(write) = self.write else {
            return callback_status(
                ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
                NATIVE_OUTPUT_SINK_MISSING_WRITER_DIAGNOSTICS_V4,
            );
        };
        unsafe {
            write(
                self.context,
                NativePluginByteSliceV3 {
                    data: bytes.as_ptr(),
                    len: bytes.len(),
                },
            )
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// Native 行为回调表及其状态、命令、事件和注册清单 schema 声明。
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

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// 桥接分发表头，描述 ABI 版本、连续方法数组和槽位数。
pub struct NativePluginBridgeMethodTableV3 {
    pub abi_version: u32,
    pub methods: *const NativePluginBridgeMethodV3,
    pub method_count: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// 单个桥接槽位的接口 ID、方法名、C 回调和不透明用户数据。
pub struct NativePluginBridgeMethodV3 {
    pub interface_id: *const c_char,
    pub method_name: *const c_char,
    pub method: Option<NativePluginBridgeMethodFnV3>,
    pub user_data: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// 单次桥接调用使用的接口/方法槽位、输入输出缓冲区与不透明用户数据。
pub struct NativePluginBridgeMethodCallV3 {
    pub interface_slot: u32,
    pub method_slot: u32,
    pub payload: ZrByteSlice,
    pub output: ZrByteBufferRef,
    pub user_data: u64,
}

// 以下函数指针别名固定缓冲区、命令、状态、桥接及 Host 回调的 C ABI 签名。
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

pub const NATIVE_COMMAND_MANIFEST_SCHEMA_V4: &[u8] = b"zircon.native.command-manifest/4\0";
pub const NATIVE_EVENT_MANIFEST_SCHEMA_V3: &[u8] = b"zircon.native.event-manifest/3\0";
pub const NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3: &[u8] =
    b"zircon.native.registration-manifest/3\0";
pub const NATIVE_EMPTY_CSTR: &[u8] = b"\0";
pub const NATIVE_FREE_OWNER_MISMATCH_DIAGNOSTICS: &[u8] =
    b"native plugin SDK allocation owner mismatch\0";
pub const NATIVE_FREE_INVALID_BUFFER_DIAGNOSTICS: &[u8] =
    b"native plugin SDK buffer descriptor is malformed\0";
pub const NATIVE_OUTPUT_SINK_LIMIT_EXCEEDED_DIAGNOSTICS_V4: &[u8] =
    b"native plugin command output exceeds the host-owned sink limit\0";
pub const NATIVE_OUTPUT_SINK_MISSING_WRITER_DIAGNOSTICS_V4: &[u8] =
    b"native plugin command output sink is missing its host writer\0";

pub const NATIVE_COMMAND_MANIFEST_SCHEMA_V4_TEXT: &str = "zircon.native.command-manifest/4";
pub const NATIVE_COMMAND_MAX_OUTPUT_BYTES_V4: usize = 256 * 1024 * 1024;
pub const NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3_TEXT: &str =
    "zircon.native.registration-manifest/3";
pub const NATIVE_SYSTEM_WORKER_SAFE_CAPABILITY_V3_TEXT: &str = "runtime.native.system.worker_safe";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Native 命令接口的版本化 TOML 数据；当前校验器要求槽位连续且名称唯一。
pub struct NativePluginCommandManifestV4 {
    pub schema: String,
    #[serde(default)]
    pub commands: Vec<NativePluginCommandV4>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// 一个命令槽位的名称、输入 schema 与 Host 允许的最大输出字节数。
pub struct NativePluginCommandV4 {
    pub name: String,
    pub slot: u32,
    pub payload_schema: String,
    pub max_output_bytes: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
/// 把模块、系统、资源、事件、扩展点及 capability 投影成 Native 可读取的声明清单。
pub struct NativePluginRegistrationManifestV3 {
    pub schema: String,
    #[serde(default)]
    pub modules: Vec<NativePluginRegistrationModuleV3>,
    #[serde(default)]
    pub systems: Vec<NativePluginRegistrationSystemV3>,
    #[serde(default)]
    pub resources: Vec<NativePluginRegistrationResourceV3>,
    #[serde(default)]
    pub events: Vec<NativePluginRegistrationEventV3>,
    #[serde(default)]
    pub extensions: Vec<NativePluginRegistrationExtensionV3>,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
/// Native 注册清单中的模块名称及模块种类。
pub struct NativePluginRegistrationModuleV3 {
    pub name: String,
    pub kind: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
/// 一个 Native runtime system 的调度、访问集、线程亲和性与可选桥接方法声明。
pub struct NativePluginRegistrationSystemV3 {
    pub id: String,
    pub module: String,
    pub stage: String,
    #[serde(default)]
    pub order: i32,
    #[serde(default)]
    pub sets: Vec<String>,
    #[serde(default)]
    pub before: Vec<String>,
    #[serde(default)]
    pub after: Vec<String>,
    #[serde(default)]
    pub access: Vec<String>,
    #[serde(default)]
    pub thread_affinity: NativePluginRegistrationThreadAffinityV3,
    #[serde(default)]
    pub bridge_interface: Option<String>,
    #[serde(default)]
    pub bridge_method: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
/// 描述 Native system 只能在主线程运行，还是可由 worker 执行。
pub enum NativePluginRegistrationThreadAffinityV3 {
    #[default]
    MainThreadOnly,
    WorkerSafe,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
/// Native runtime 注册资源及其可选所属模块和 schema。
pub struct NativePluginRegistrationResourceV3 {
    pub id: String,
    #[serde(default)]
    pub module: Option<String>,
    #[serde(default)]
    pub schema: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
/// Native runtime 事件的命名空间、名称、稳定哈希与可选 schema。
pub struct NativePluginRegistrationEventV3 {
    pub namespace: String,
    pub name: String,
    #[serde(default)]
    pub stable_hash: u64,
    #[serde(default)]
    pub schema: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
/// 声明一个扩展点贡献及其可选贡献值和 schema。
pub struct NativePluginRegistrationExtensionV3 {
    pub point: String,
    #[serde(default)]
    pub contribution: Option<String>,
    #[serde(default)]
    pub schema: Option<String>,
}

pub type NativePluginHostReadyFnV3 = fn(*const NativePluginHostFunctionTableV3);

/// Stores an audited immutable ABI carrier in generated plugin statics.
///
/// Arbitrary values are intentionally not `Sync` through this wrapper:
///
/// ```compile_fail
/// use std::cell::Cell;
/// use zircon_plugin_sdk::native::NativePluginStatic;
///
/// static INVALID: NativePluginStatic<Cell<u32>> = NativePluginStatic::new(Cell::new(0));
/// ```
#[repr(transparent)]
pub struct NativePluginStatic<T>(T);

// SAFETY: Implementations are restricted to immutable ABI tables whose raw pointers are data, not
// shared Rust ownership. The private trait prevents downstream crates from expanding this set.
unsafe trait NativePluginStaticValue {}

// SAFETY: These carrier values are generated once, never mutated, and only exposed by shared
// reference or raw const pointer. Callback targets and pointees own their separate synchronization.
unsafe impl NativePluginStaticValue for NativePluginAbiV3 {}
unsafe impl NativePluginStaticValue for NativePluginBehaviorV4 {}
unsafe impl NativePluginStaticValue for NativePluginEntryReportV3 {}
unsafe impl NativePluginStaticValue for NativePluginBridgeMethodTableV3 {}
unsafe impl NativePluginStaticValue for NativePluginBridgeMethodV3 {}
unsafe impl<T: NativePluginStaticValue, const N: usize> NativePluginStaticValue for [T; N] {}

// SAFETY: NativePluginStaticValue is a closed set of audited immutable ABI carriers.
unsafe impl<T: NativePluginStaticValue> Sync for NativePluginStatic<T> {}

impl<T> NativePluginStatic<T> {
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    pub const fn get(&self) -> &T {
        &self.0
    }

    pub const fn as_ptr(&self) -> *const T {
        &self.0 as *const T
    }
}

/// 按声明的能力条件选择成功或缺失宿主报告；成功时调用宿主就绪回调，无条件声明不校验宿主表兼容性。
pub struct NativePluginEntryPointV3 {
    report: &'static NativePluginStatic<NativePluginEntryReportV3>,
    missing_host_report: &'static NativePluginStatic<NativePluginEntryReportV3>,
    required_capabilities: &'static [&'static str],
    denied_capabilities: &'static [&'static str],
    on_host_ready: Option<NativePluginHostReadyFnV3>,
}

impl NativePluginEntryPointV3 {
    pub const fn new(
        report: &'static NativePluginStatic<NativePluginEntryReportV3>,
        missing_host_report: &'static NativePluginStatic<NativePluginEntryReportV3>,
        required_capabilities: &'static [&'static str],
        denied_capabilities: &'static [&'static str],
        on_host_ready: Option<NativePluginHostReadyFnV3>,
    ) -> Self {
        Self {
            report,
            missing_host_report,
            required_capabilities,
            denied_capabilities,
            on_host_ready,
        }
    }

    pub fn entry_report(
        &self,
        host_functions: *const NativePluginHostFunctionTableV3,
    ) -> *const NativePluginEntryReportV3 {
        let has_capability_constraints =
            !self.required_capabilities.is_empty() || !self.denied_capabilities.is_empty();
        let host_is_compatible =
            has_capability_constraints && host_functions_v3_are_compatible(host_functions);
        let supports_required = self.required_capabilities.is_empty()
            || (host_is_compatible
                && self.required_capabilities.iter().all(|capability| {
                    host_supports_capability_with_compatible_host_v3(host_functions, capability)
                }));
        let supports_denied = supports_required
            && host_is_compatible
            && self.denied_capabilities.iter().any(|capability| {
                host_supports_capability_with_compatible_host_v3(host_functions, capability)
            });
        if supports_required && !supports_denied {
            if let Some(on_host_ready) = self.on_host_ready {
                on_host_ready(host_functions);
            }
            self.report.as_ptr()
        } else {
            self.missing_host_report.as_ptr()
        }
    }
}

/// 将 SDK 状态码和静态诊断字节切片打包为 ABI 返回值。
pub fn callback_status(code: u32, diagnostics: &'static [u8]) -> NativePluginCallbackStatusV3 {
    NativePluginCallbackStatusV3 {
        code,
        diagnostics: diagnostics.as_ptr().cast(),
    }
}

/// 把 Vec 的分配所有权交给 ABI 描述符，供对应的 free 回调归还。
///
/// 登记成功后由当前 SDK image 保留真实 Vec，owner_token 是不复用的 opaque allocation ID。
/// 登记失败返回仍拥有原始 Vec 的错误，可用 into_bytes 收回；Native 生产者须返回 error.status。
/// 零容量空 Vec 返回 empty；有容量的空 Vec 仍须由同一 image 的 free 归还。
/// 描述符及其释放函数不得跨越该 DLL image 的存活期，也不得与载荷读取并发释放。
pub fn owned_bytes(
    bytes: Vec<u8>,
) -> Result<NativePluginOwnedByteBufferV3, NativePluginOwnedBytesError> {
    owned_buffers::register(bytes)
}

/// 将 Native 命令清单编码为供动态库消费者读取的 TOML。
pub fn command_manifest_v4_to_toml(
    manifest: &NativePluginCommandManifestV4,
) -> Result<String, toml::ser::Error> {
    toml::to_string(manifest)
}

/// 解析 Host 或插件提供的命令清单文本；schema/槽位语义由单独校验函数检查。
pub fn command_manifest_v4_from_toml(
    text: &str,
) -> Result<NativePluginCommandManifestV4, toml::de::Error> {
    toml::from_str(text)
}

/// 检查 schema 版本、连续槽位、非空字段、输出上限与唯一命令名。
pub fn command_manifest_v4_is_current_and_dense(manifest: &NativePluginCommandManifestV4) -> bool {
    if manifest.schema.trim() != NATIVE_COMMAND_MANIFEST_SCHEMA_V4_TEXT {
        return false;
    }
    let mut names = std::collections::BTreeSet::new();
    manifest
        .commands
        .iter()
        .enumerate()
        .all(|(index, command)| {
            command.slot == u32::try_from(index).unwrap_or(u32::MAX)
                && !command.name.is_empty()
                && !command.payload_schema.trim().is_empty()
                && command.max_output_bytes <= NATIVE_COMMAND_MAX_OUTPUT_BYTES_V4
                && names.insert(command.name.as_str())
        })
}

/// 将宏生成的 Native 注册声明编码为 ABI 约定的 TOML 文本。
pub fn registration_manifest_v3_to_toml(
    manifest: &NativePluginRegistrationManifestV3,
) -> Result<String, toml::ser::Error> {
    toml::to_string(manifest)
}

/// 将 ABI TOML 文本解析为 Native 注册清单 DTO。
pub fn registration_manifest_v3_from_toml(
    text: &str,
) -> Result<NativePluginRegistrationManifestV3, toml::de::Error> {
    toml::from_str(text)
}

/// 只判断注册清单 schema 标识是否属于当前 v3 版本。
pub fn registration_manifest_v3_schema_is_current(
    manifest: &NativePluginRegistrationManifestV3,
) -> bool {
    manifest.schema.trim() == NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3_TEXT
}

/// 释放由 SDK 转移给宿主的字节缓冲区，并返回 ABI 状态码。
///
/// # Safety
///
/// 非空缓冲区必须来自一次尚未释放的 owned_bytes 调用，字段未经修改且只归还一次；
/// 空缓冲区可使用 empty 构造。复制、伪造或重复归还非空缓冲区可能导致未定义行为。
///
/// SDK 登记表按 ID、地址、长度、容量核对并一次消费真实 Vec；不解引用来历不明的 data，
/// 不以外部字段重建 Vec。错配或已消费描述符返回 ERROR，错配不会移除仍存活的所有者。
/// 上述保守作者义务仍约束载荷读取与 image 生命周期；登记表不验证宿主读取的任意地址。
pub unsafe extern "C" fn free_owned_bytes_v3(
    buffer: NativePluginOwnedByteBufferV3,
) -> NativePluginCallbackStatusV3 {
    // The descriptor can cross an FFI boundary. Validate its shape before constructing a Vec.
    // Ownership stays in the registry; foreign metadata never constructs a Vec.
    owned_buffers::release(buffer)
}

/// 将 ABI 借用切片视图转换为 Rust 字节切片；空指针或零长度返回空切片。
///
/// # Safety
///
/// 对非空且非零长度的输入，data 必须在返回切片的整个使用期内指向至少 len 个
/// 连续且已初始化的可读字节，且 len 不超过 Rust 切片允许的长度。
/// 这些字节须位于同一分配内，在返回切片的借用期内不被修改，地址范围不得溢出。
pub unsafe fn bytes_from_slice<'a>(slice: NativePluginByteSliceV3) -> &'a [u8] {
    if slice.data.is_null() || slice.len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(slice.data, slice.len) }
    }
}

/// 捕获 Native 回调中的 Rust panic，并将其转换为约定的 PANIC 状态。
pub fn catch_native_callback_panic<F>(
    panic_diagnostics: &'static [u8],
    callback: F,
) -> NativePluginCallbackStatusV3
where
    F: FnOnce() -> NativePluginCallbackStatusV3,
{
    let result = catch_unwind(AssertUnwindSafe(callback));
    match result {
        Ok(status) => status,
        Err(_) => callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_PANIC, panic_diagnostics),
    }
}

/// 只有宿主 ABI 兼容且列表中的每项能力都获授时才返回 true；空列表为 true。
pub fn host_supports_all_capabilities_v3(
    host_functions: *const NativePluginHostFunctionTableV3,
    capabilities: &[&str],
) -> bool {
    if capabilities.is_empty() {
        return true;
    }
    if !host_functions_v3_are_compatible(host_functions) {
        return false;
    }
    capabilities.iter().all(|capability| {
        host_supports_capability_with_compatible_host_v3(host_functions, capability)
    })
}

/// 检查宿主是否授予列表中的至少一项能力；空列表返回 false。
pub fn host_supports_any_capability_v3(
    host_functions: *const NativePluginHostFunctionTableV3,
    capabilities: &[&str],
) -> bool {
    if capabilities.is_empty() || !host_functions_v3_are_compatible(host_functions) {
        return false;
    }
    capabilities.iter().any(|capability| {
        host_supports_capability_with_compatible_host_v3(host_functions, capability)
    })
}

/// 检查一个能力是否由版本兼容的宿主授予。
pub fn host_supports_capability_v3(
    host_functions: *const NativePluginHostFunctionTableV3,
    capability: &str,
) -> bool {
    host_functions_v3_are_compatible(host_functions)
        && host_supports_capability_with_compatible_host_v3(host_functions, capability)
}

// BUG: [CR-R02-public_sdk_reflect-0001] 安全公开入口会把原始宿主表指针传入此处；仅拒绝 null 后仍直接解引用，无效非空指针可由安全 Rust 调用触发未定义行为。
fn host_functions_v3_are_compatible(
    host_functions: *const NativePluginHostFunctionTableV3,
) -> bool {
    if host_functions.is_null() {
        return false;
    }
    let host_functions = unsafe { &*host_functions };
    let host_version = host_functions
        .host_abi_version
        .map(|host_abi_version| unsafe { host_abi_version() })
        .unwrap_or_default();
    if host_functions.abi_version != ZIRCON_NATIVE_PLUGIN_ABI_VERSION
        || host_version != ZIRCON_NATIVE_PLUGIN_ABI_VERSION
        || host_functions.host_handle == 0
    {
        return false;
    }
    true
}

fn host_supports_capability_with_compatible_host_v3(
    host_functions: *const NativePluginHostFunctionTableV3,
    capability: &str,
) -> bool {
    let host_functions = unsafe { &*host_functions };
    if let Some(host_has_capability) = host_functions.host_has_capability {
        let Ok(capability) = CString::new(capability) else {
            return false;
        };
        return unsafe { host_has_capability(host_functions, capability.as_ptr()) }
            == ZIRCON_NATIVE_PLUGIN_STATUS_OK;
    }
    capability_list_contains(host_functions.granted_capabilities, capability)
}

/// 在宿主提供的 NUL 终止能力串中查找以换行、逗号或分号分隔的能力名。
pub fn capability_list_contains(capabilities: *const std::ffi::c_char, capability: &str) -> bool {
    if capabilities.is_null() {
        return false;
    }
    // BUG: [CR-R02-public_sdk_reflect-0002] 此安全函数只过滤 null；非空悬空或未终止 C 字符串会令 from_ptr 越界读取。
    let Ok(capabilities) = unsafe { CStr::from_ptr(capabilities) }.to_str() else {
        return false;
    };
    capabilities
        .split(|character| matches!(character, '\n' | ',' | ';'))
        .map(str::trim)
        .any(|entry| entry == capability)
}

#[macro_export]
/// 导出固定的 v3 描述符符号，使 loader 能取得静态 Native ABI 描述符指针。
macro_rules! export_native_plugin_descriptor_v3 {
    ($descriptor:expr) => {
        #[no_mangle]
        pub extern "C" fn zircon_native_plugin_descriptor_v3(
        ) -> *const $crate::native::NativePluginAbiV3 {
            ($descriptor).as_ptr()
        }
    };
}

/// Internal implementation for downstream exported entry macros.
/// Ordinary unwind panics produce the existing V3 null-entry failure.
/// panic=abort, foreign faults and panicking payload destructors cannot be recovered.
#[doc(hidden)]
pub fn catch_native_entry_panic_v3<F>(entry: F) -> *const NativePluginEntryReportV3
where
    F: FnOnce() -> *const NativePluginEntryReportV3,
{
    match catch_unwind(AssertUnwindSafe(entry)) {
        Ok(report) => report,
        Err(payload) => {
            // Arbitrary panic_any payload destructors can panic again. Dispose
            // without allowing a second unwind across the extern-C boundary.
            match catch_unwind(AssertUnwindSafe(|| drop(payload))) {
                Ok(()) => std::ptr::null(),
                Err(_secondary_payload) => std::process::abort(),
            }
        }
    }
}

#[macro_export]
/// 导出宿主入口函数，并将 Host function table 交给声明的 SDK entry point。
macro_rules! export_native_plugin_entry_v3 {
    ($entry_fn:ident, $entry_point:expr) => {
        #[no_mangle]
        pub extern "C" fn $entry_fn(
            host_functions: *const $crate::native::NativePluginHostFunctionTableV3,
        ) -> *const $crate::native::NativePluginEntryReportV3 {
            $crate::native::catch_native_entry_panic_v3(|| {
                ($entry_point).entry_report(host_functions)
            })
        }
    };
}

#[macro_export]
/// 为无状态命令插件生成 descriptor、行为与入口报告静态值，并导出描述符和运行时入口。
macro_rules! native_command_plugin_v3 {
    (
        plugin_id: $plugin_id:expr,
        package_manifest: $package_manifest:expr,
        runtime_entry: $runtime_entry:ident,
        runtime_entry_name: $runtime_entry_name:expr,
        requested_capabilities: $requested_capabilities:expr,
        required_capabilities: [$($required_capability:literal),* $(,)?],
        negotiated_capabilities: $negotiated_capabilities:expr,
        diagnostics: $diagnostics:expr,
        missing_host_diagnostics: $missing_host_diagnostics:expr,
        command_manifest: $command_manifest:expr,
        event_manifest: $event_manifest:expr,
        invoke_command: $invoke_command:path $(,)?
    ) => {
        static __ZIRCON_NATIVE_SDK_DESCRIPTOR_V3: $crate::native::NativePluginStatic<
            $crate::native::NativePluginAbiV3,
        > = $crate::native::NativePluginStatic::new($crate::native::NativePluginAbiV3 {
            abi_version: $crate::native::ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
            plugin_id: ($plugin_id).as_ptr().cast(),
            package_manifest_toml: ($package_manifest).as_bytes().as_ptr().cast(),
            runtime_entry_name: ($runtime_entry_name).as_ptr().cast(),
            editor_entry_name: ::core::ptr::null(),
            requested_capabilities: ($requested_capabilities).as_ptr().cast(),
        });

        static __ZIRCON_NATIVE_SDK_BEHAVIOR_V4: $crate::native::NativePluginStatic<
            $crate::native::NativePluginBehaviorV4,
        > = $crate::native::NativePluginStatic::new($crate::native::NativePluginBehaviorV4 {
            abi_version: $crate::native::ZIRCON_NATIVE_PLUGIN_BEHAVIOR_ABI_VERSION_V4,
            is_stateless: 1,
            schema_versions: $crate::native::NativePluginSchemaVersionsV3 {
                state_schema_version: 0,
                command_manifest_schema: $crate::native::NATIVE_COMMAND_MANIFEST_SCHEMA_V4
                    .as_ptr()
                    .cast(),
                event_manifest_schema: $crate::native::NATIVE_EVENT_MANIFEST_SCHEMA_V3
                    .as_ptr()
                    .cast(),
                registration_manifest_schema: ::core::ptr::null(),
            },
            command_manifest: ($command_manifest).as_ptr().cast(),
            event_manifest: ($event_manifest).as_ptr().cast(),
            registration_manifest: ::core::ptr::null(),
            invoke_command: Some($invoke_command),
            save_state: None,
            restore_state: None,
            unload: None,
        });

        const __ZIRCON_NATIVE_SDK_REQUIRED_CAPABILITIES_TEXT_V3: &str =
            concat!($($required_capability, "\n",)* "\0");

        static __ZIRCON_NATIVE_SDK_REPORT_V3: $crate::native::NativePluginStatic<
            $crate::native::NativePluginEntryReportV3,
        > = $crate::native::NativePluginStatic::new($crate::native::NativePluginEntryReportV3 {
            layout_epoch: $crate::native::ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH,
            package_manifest_toml: ($package_manifest).as_bytes().as_ptr().cast(),
            diagnostics: ($diagnostics).as_ptr().cast(),
            negotiated_capabilities: ($negotiated_capabilities).as_ptr().cast(),
            required_capabilities: __ZIRCON_NATIVE_SDK_REQUIRED_CAPABILITIES_TEXT_V3
                .as_ptr()
                .cast(),
            denied_capabilities: $crate::native::NATIVE_EMPTY_CSTR.as_ptr().cast(),
            behavior: __ZIRCON_NATIVE_SDK_BEHAVIOR_V4.as_ptr(),
            bridge_methods: ::core::ptr::null(),
        });

        static __ZIRCON_NATIVE_SDK_MISSING_HOST_REPORT_V3: $crate::native::NativePluginStatic<
            $crate::native::NativePluginEntryReportV3,
        > = $crate::native::NativePluginStatic::new($crate::native::NativePluginEntryReportV3 {
            layout_epoch: $crate::native::ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH,
            package_manifest_toml: ($package_manifest).as_bytes().as_ptr().cast(),
            diagnostics: ($missing_host_diagnostics).as_ptr().cast(),
            negotiated_capabilities: $crate::native::NATIVE_EMPTY_CSTR.as_ptr().cast(),
            required_capabilities: __ZIRCON_NATIVE_SDK_REQUIRED_CAPABILITIES_TEXT_V3
                .as_ptr()
                .cast(),
            denied_capabilities: $crate::native::NATIVE_EMPTY_CSTR.as_ptr().cast(),
            behavior: ::core::ptr::null(),
            bridge_methods: ::core::ptr::null(),
        });

        const __ZIRCON_NATIVE_SDK_REQUIRED_CAPABILITIES_V3: &[&str] =
            &[$($required_capability),*];

        static __ZIRCON_NATIVE_SDK_ENTRY_POINT_V3: $crate::native::NativePluginEntryPointV3 =
            $crate::native::NativePluginEntryPointV3::new(
                &__ZIRCON_NATIVE_SDK_REPORT_V3,
                &__ZIRCON_NATIVE_SDK_MISSING_HOST_REPORT_V3,
                __ZIRCON_NATIVE_SDK_REQUIRED_CAPABILITIES_V3,
                &[],
                None,
            );

        $crate::export_native_plugin_descriptor_v3!(__ZIRCON_NATIVE_SDK_DESCRIPTOR_V3);
        $crate::export_native_plugin_entry_v3!(
            $runtime_entry,
            __ZIRCON_NATIVE_SDK_ENTRY_POINT_V3
        );
    };
}

#[cfg(test)]
#[path = "native/tests/cases.rs"]
mod tests;
