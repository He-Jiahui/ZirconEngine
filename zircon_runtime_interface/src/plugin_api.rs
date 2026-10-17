//! 插件回调表与入口报告的 C ABI 载体。
//!
//! 当前原生插件加载路径使用独立布局，这些结构不可与其入口报告互换。

//! 宿主表按域提供可选函数指针；表值、句柄和 `Copy` 描述符均不持有注册 scope 或动态库。
//! V3 桥接表与 V4 注册表由各自宿主适配器构造；`Some` 入口仍可能返回不支持状态。
//! 本模块只定义边界数据，实际嵌套指针校验、授权和调度由宿主负责。
use crate::buffer::{ZrByteBufferRef, ZrByteSlice, ZrOwnedByteBuffer};
use crate::handles::ZrRuntimePluginHandle;
use crate::manifest::ZrPluginModuleDescriptorV1;
use crate::plugin_events::ZrPluginEventCallbackFnV1;
use crate::runtime_api::ZrHostApiV1;
use crate::status::ZrStatus;

/// 共享 V1 入口名称的 NUL 结尾字节；只声明符号，不执行动态加载。
pub const ZR_PLUGIN_ENTRY_SYMBOL_V1: &[u8] = b"zircon_plugin_entry_v1\0";
/// 共享 V3 入口名称；对应 [`ZrPluginEntryFnV3`]，不能替换原生 SDK 的另一套报告布局。
pub const ZR_PLUGIN_ENTRY_SYMBOL_V3: &[u8] = b"zircon_plugin_entry_v3\0";
/// 共享 V4 入口名称；布局契约见 [`ZrPluginEntryFnV4`]。
pub const ZR_PLUGIN_ENTRY_SYMBOL_V4: &[u8] = b"zircon_plugin_entry_v4\0";

/// V4 访问声明的只读模式；宿主把它映射为调度冲突契约的 read 标记。
pub const ZR_NATIVE_SYSTEM_ACCESS_MODE_READ_V1: u32 = 1;
/// V4 访问声明的写模式；对同一稳定 ID 的冲突由宿主访问计划判定。
pub const ZR_NATIVE_SYSTEM_ACCESS_MODE_WRITE_V1: u32 = 2;
/// 访问 ID 指向 ECS 组件域；宿主核对已注册组件及其插件归属。
pub const ZR_NATIVE_SYSTEM_ACCESS_DOMAIN_COMPONENT_V1: u32 = 1;
/// 访问 ID 指向外部资源域；宿主核对已知资源和对应授权。
pub const ZR_NATIVE_SYSTEM_ACCESS_DOMAIN_RESOURCE_V1: u32 = 2;
/// V4 系统限主线程调度的数值标记，也是空注册记录的默认值。
pub const ZR_NATIVE_SYSTEM_THREAD_AFFINITY_MAIN_THREAD_ONLY_V1: u32 = 0;
/// 申请 worker 调度的数值标记；宿主仍须核对相应 capability。
pub const ZR_NATIVE_SYSTEM_THREAD_AFFINITY_WORKER_SAFE_V1: u32 = 1;

/// 共享 V1 表到入口报告的 C 签名；当前原生加载器使用独立 SDK 签名。
/// 表指针仅供约定的调用期借用，返回报告及嵌套指针的可读期限须由提供者保证。
pub type ZrPluginEntryFnV1 =
    unsafe extern "C" fn(*const ZrHostApiV1) -> *const ZrPluginEntryReportV1;
/// 以 [`ZrHostApiV3`] 调用共享入口并借用其报告；不是当前 DLL 探测器的函数类型。
/// 调用者须保证输入表有效，报告提供者须保证返回布局及嵌套借用有效。
pub type ZrPluginEntryFnV3 =
    unsafe extern "C" fn(*const ZrHostApiV3) -> *const ZrPluginEntryReportV1;
/// 以 [`ZrHostApiV4`] 调用共享入口；返回值沿用 [`ZrPluginEntryReportV1`]。
/// 数字版本不会证明原始指针可读，输入表与输出报告的存活前提由双方建立。
pub type ZrPluginEntryFnV4 =
    unsafe extern "C" fn(*const ZrHostApiV4) -> *const ZrPluginEntryReportV1;
/// 对给定插件句柄执行提供方卸载回调并返回状态；共享 API 只保存该地址。
/// 调用前须停止相关在途调用，并在回调和诊断读取结束前保持提供方代码存活。
pub type ZrPluginUnloadFnV1 = unsafe extern "C" fn(ZrRuntimePluginHandle) -> ZrStatus;
/// 调度时传回注册句柄、`user_data` 与借用载荷；当前宿主适配器传空载荷。
/// 注册会保留函数指针供后续执行，调用方须维持代码寿命；当前适配器忽略返回状态。
pub type ZrNativeSystemInvokeFnV1 =
    unsafe extern "C" fn(ZrRuntimePluginHandle, u64, ZrByteSlice) -> ZrStatus;
/// 同步读取 V1 系统描述并返回注册状态；当前实现声明保守的全 World 访问。
/// 描述符和嵌套文本只在本次注册期借用，`invoke` 地址须覆盖后续调度。
/// # 返回值与参考
/// 返回本次注册状态；实际解析、复制及保守访问见 Runtime 的
/// `plugin/loader/host_api_adapter/ecs_registration/mod.rs::register_system_from_abi`。
/// # 安全前提
/// Runtime 的现有包装对空描述指针返回 `InvalidArgument`。非空 `registration` 须对齐并指向已初始化的完整记录。
/// 被读取的字节和非零计数数组须覆盖声明长度，在本次同步调用期间保持有效、可读且不被并发修改；
/// 每个切片须位于同一分配区。宿主的形状检查不能证明任意非空地址的存储有效。
pub type ZrHostRegisterSystemFnV1 =
    unsafe extern "C" fn(ZrRuntimePluginHandle, *const ZrSystemRegistrationV1) -> ZrStatus;
/// 同步读取 V2 描述，按 V4 策略核对访问 ID、授权及线程亲和后注册。
/// `handle` 须属于仍开放的注册 scope；非空描述符与数组须对齐、初始化且可读。
/// # 参数
/// 句柄选择仍开放的注册 scope；描述指针及其文本/数组仅借给本次同步解码。
/// # 返回值
/// 宿主返回注册状态；空描述拒绝为 InvalidArgument，失效句柄为 NotFound，接纳错误为 Error。
/// # 参考
/// Runtime 的 `host_api_adapter/registration_policy/scope.rs::api` 安装此入口；
/// `host_api_adapter/ecs_registration/mod.rs::register_system_from_abi_v4` 完成接纳。
pub type ZrHostRegisterSystemFnV2 =
    unsafe extern "C" fn(ZrRuntimePluginHandle, *const ZrSystemRegistrationV2) -> ZrStatus;
/// 读取组件 ID 与显示名并返回注册状态；当前适配器不消费 `schema` 或 `storage_kind`。
/// `handle` 定位注册 scope；描述符与被读取文本须在同步调用中有效。
/// # 参考
/// Runtime 的 `host_api_adapter/ecs_registration/mod.rs::register_component_from_abi`
/// 建立组件描述；`plugin/loader/ecs_registration/tests/cases.rs` 核对注册表结果。
/// # 安全前提
/// Runtime 的现有包装对空描述指针返回 `InvalidArgument`。非空 `descriptor` 须对齐并指向已初始化的完整记录。
/// 被读取的 `type_id`、`display_name` 字节须分别位于同一分配区并覆盖声明长度，
/// 在本次同步调用期间保持有效、可读且不被并发修改。
pub type ZrHostRegisterComponentFnV1 =
    unsafe extern "C" fn(ZrRuntimePluginHandle, *const ZrComponentDescV1) -> ZrStatus;
/// 以插件句柄和借用字节提交命令的槽位；当前注册适配器返回不支持且不消费载荷。
pub type ZrHostSpawnCommandFnV1 =
    unsafe extern "C" fn(ZrRuntimePluginHandle, *const u8, usize) -> ZrStatus;
/// 资产请求的槽位；当前注册适配器返回不支持，不写入 `output` 或转移分配。
pub type ZrHostAssetRequestFnV1 =
    unsafe extern "C" fn(ZrRuntimePluginHandle, ZrByteSlice, *mut ZrOwnedByteBuffer) -> ZrStatus;
/// 按事件类型提交借用载荷的槽位；当前注册适配器返回不支持。
pub type ZrHostEventEmitFnV1 =
    unsafe extern "C" fn(ZrRuntimePluginHandle, ZrEventTypeId, *const u8, usize) -> ZrStatus;
/// 把事件输出写到调用者缓冲区的槽位；当前注册适配器返回不支持且不写缓冲区。
pub type ZrHostEventDrainFnV1 =
    unsafe extern "C" fn(ZrRuntimePluginHandle, ZrEventTypeId, ZrByteBufferRef) -> ZrStatus;
/// 按冻结的接口槽、方法槽同步分派，返回方法状态；`handle` 须定位桥接 scope。
/// 输入载荷可读，输出数据区及 written 指针可写且不冲突；借用仅覆盖本次调用。
/// # 参数
/// interface_slot/method_slot 指向冻结分发表；payload/len 与 output 分别借用输入和输出内存。
/// # 返回值
/// 返回方法状态；接口禁用为 BridgeNotEnabled，句柄/方法缺席为 NotFound，注册句柄为不支持。
/// # 参考
/// Runtime 的 `host_api_adapter/bridge_scope/mod.rs::NativeHostBridgeCallScope::api`
/// 安装入口；同文件 `native_host_bridge_call_v1_inner` 在持有上下文及可选库世代租约时分派。
pub type ZrHostBridgeCallFnV1 = unsafe extern "C" fn(
    ZrRuntimePluginHandle,
    u32,
    u32,
    *const u8,
    usize,
    ZrByteBufferRef,
) -> ZrStatus;
/// 提交目标和日志文本的诊断槽位；当前 V3/V4 scope 的诊断表均不开放此入口。
pub type ZrHostDiagnosticsEmitFnV1 =
    unsafe extern "C" fn(ZrRuntimePluginHandle, ZrByteSlice, ZrByteSlice) -> ZrStatus;
/// 提交指标路径、数值及单位的槽位；当前 V3/V4 scope 的诊断表均不开放此入口。
pub type ZrHostDiagnosticsMetricFnV1 =
    unsafe extern "C" fn(ZrRuntimePluginHandle, ZrByteSlice, f64, ZrByteSlice) -> ZrStatus;
/// 把状态写入调用者缓冲区的共享回调签名；当前共享表没有生产调度者。
/// 缓冲区与 written 指针仅供同步写入，不能据此套用原生 SDK 的另一套状态协议。
pub type ZrPluginSnapshotSaveFnV1 =
    unsafe extern "C" fn(ZrRuntimePluginHandle, ZrByteBufferRef) -> ZrStatus;
/// 从同步借用的字节恢复状态并返回状态的共享签名；当前未接入实际原生恢复链。
pub type ZrPluginSnapshotRestoreFnV1 =
    unsafe extern "C" fn(ZrRuntimePluginHandle, *const u8, usize) -> ZrStatus;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// V3 宿主域表；当前桥接 scope 只开放 `bridge.call`，其余槽为空。
/// # 使用时机与生命周期
/// 表可复制但不拥有 scope；调用期须保持对应宿主句柄和回调来源存活。
/// # 参考
/// Runtime 的 `plugin/loader/host_api_adapter/bridge_scope/mod.rs::NativeHostBridgeCallScope::api`
/// 构造当前使用表；本 crate 的 `tests/plugin_api_contracts.rs` 固定布局与空表契约。
pub struct ZrHostApiV3 {
    /// 固定为 3 的共享宿主表版本。
    pub abi_version: u32,
    /// 本表的完整 C 布局字节数。
    pub size_bytes: usize,
    /// V1 ECS 注册槽。
    pub ecs: ZrHostEcsApiV1,
    /// 资产请求槽；存在槽不等于请求已实现。
    pub asset: ZrHostAssetApiV1,
    /// 宿主事件提交和读取槽。
    pub event: ZrHostEventApiV1,
    /// 冻结接口/方法槽分派入口。
    pub bridge: ZrHostBridgeApiV1,
    /// 可选日志与指标入口；当前 scope 保持为空。
    pub diagnostics: ZrHostDiagnosticsApiV1,
}

impl ZrHostApiV3 {
    /// 返回固定版本、完整 size 与空域槽的表；仍须由宿主填入可用回调。
    pub const fn empty() -> Self {
        Self {
            abi_version: 3,
            size_bytes: core::mem::size_of::<Self>(),
            ecs: ZrHostEcsApiV1::empty(),
            asset: ZrHostAssetApiV1::empty(),
            event: ZrHostEventApiV1::empty(),
            bridge: ZrHostBridgeApiV1::empty(),
            diagnostics: ZrHostDiagnosticsApiV1::empty(),
        }
    }
}

/// Versioned host table for exact native-system scheduling declarations.
///
/// `ZrHostApiV3` remains frozen so a V3 plugin always retains its conservative registration
/// semantics. V4 is the only table that may expose the structured access contract.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// V4 宿主域表，把 ECS 注册槽切换为 V2 描述；其他域布局保持独立版本。
/// # 约束
/// 注册 scope 填充的部分 `Some` 槽仍返回不支持；版本和 size 不能替代能力判定。
/// # 使用时机与生命周期
/// 注册阶段由宿主 scope 借用注册表并建立句柄；表的 Copy 不延长 scope 或插件代码寿命。
/// # 参考
/// Runtime 的 `host_api_adapter/registration_policy/scope.rs::NativeHostApiV4RegistrationScope::api`
/// 安装入口；`host_api_adapter/tests/cases.rs` 核对授权访问和空 diagnostics。
pub struct ZrHostApiV4 {
    /// 固定为 4 的共享宿主表版本。
    pub abi_version: u32,
    /// 本表的完整 C 布局字节数。
    pub size_bytes: usize,
    /// 接收 V2 系统描述的 ECS 注册槽。
    pub ecs: ZrHostEcsApiV2,
    /// 当前注册 scope 提供不支持请求的占位入口。
    pub asset: ZrHostAssetApiV1,
    /// 当前注册 scope 提供不支持的事件入口。
    pub event: ZrHostEventApiV1,
    /// 当前注册句柄不能用于桥接分派，返回不支持。
    pub bridge: ZrHostBridgeApiV1,
    /// 当前注册 scope 不开放任何诊断入口。
    pub diagnostics: ZrHostDiagnosticsApiV1,
}

impl ZrHostApiV4 {
    /// 返回固定版本、完整 size 与空域槽的表；仍须由宿主填入可用回调。
    pub const fn empty() -> Self {
        Self {
            abi_version: 4,
            size_bytes: core::mem::size_of::<Self>(),
            ecs: ZrHostEcsApiV2::empty(),
            asset: ZrHostAssetApiV1::empty(),
            event: ZrHostEventApiV1::empty(),
            bridge: ZrHostBridgeApiV1::empty(),
            diagnostics: ZrHostDiagnosticsApiV1::empty(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// 使用 V1 系统描述的可选 ECS 槽；当前 V3 桥接 scope 不开放这些槽。
pub struct ZrHostEcsApiV1 {
    /// 可选 V1 注册回调；后续调度采用保守访问。
    pub register_system: Option<ZrHostRegisterSystemFnV1>,
    /// 可选组件 ID/显示名注册回调。
    pub register_component: Option<ZrHostRegisterComponentFnV1>,
    /// 可选命令槽，当前适配器返回不支持。
    pub spawn_command: Option<ZrHostSpawnCommandFnV1>,
}

impl ZrHostEcsApiV1 {
    /// 返回所有可选槽为空的表；构造仅初始化值，不执行注册。
    pub const fn empty() -> Self {
        Self {
            register_system: None,
            register_component: None,
            spawn_command: None,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// V4 注册 scope 的 ECS 槽；系统访问与 worker 资格由宿主策略核验。
/// # 参考
/// [`ZrHostRegisterSystemFnV2`] 与 Runtime 的 V4 注册 scope。
pub struct ZrHostEcsApiV2 {
    /// V4 按访问计划及授权接纳 V2 系统描述的入口。
    pub register_system: Option<ZrHostRegisterSystemFnV2>,
    /// 同步复制组件 ID 和显示名的入口。
    pub register_component: Option<ZrHostRegisterComponentFnV1>,
    /// 当前存在但返回不支持的命令槽。
    pub spawn_command: Option<ZrHostSpawnCommandFnV1>,
}

impl ZrHostEcsApiV2 {
    /// 返回所有可选槽为空的表；构造仅初始化值，不执行注册。
    pub const fn empty() -> Self {
        Self {
            register_system: None,
            register_component: None,
            spawn_command: None,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// 资产请求的可选槽；是否能完成请求由回调状态判定。
pub struct ZrHostAssetApiV1 {
    /// 可选请求回调；当前实现不写输出。
    pub request: Option<ZrHostAssetRequestFnV1>,
}

impl ZrHostAssetApiV1 {
    /// 返回所有可选槽为空的表；构造仅初始化值，不执行注册。
    pub const fn empty() -> Self {
        Self { request: None }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// 事件提交与读取的可选槽，不是插件事件处理器的回调表。
pub struct ZrHostEventApiV1 {
    /// 可选提交槽，借用 payload 不转移内存所有权。
    pub emit: Option<ZrHostEventEmitFnV1>,
    /// 可选读取槽，输出缓冲区属于调用者。
    pub drain: Option<ZrHostEventDrainFnV1>,
}

impl ZrHostEventApiV1 {
    /// 返回所有可选槽为空的表；构造仅初始化值，不执行注册。
    pub const fn empty() -> Self {
        Self {
            emit: None,
            drain: None,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// 宿主桥接分派槽；V3 桥接句柄和 V4 注册句柄不能混用。
pub struct ZrHostBridgeApiV1 {
    /// 同步分派入口；按句柄、接口启用状态和方法槽核验。
    pub call: Option<ZrHostBridgeCallFnV1>,
}

impl ZrHostBridgeApiV1 {
    /// 返回所有可选槽为空的表；构造仅初始化值，不执行注册。
    pub const fn empty() -> Self {
        Self { call: None }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// 诊断可选槽；空表明确不提供日志或指标接收能力。
pub struct ZrHostDiagnosticsApiV1 {
    /// 可选目标/消息提交回调。
    pub emit: Option<ZrHostDiagnosticsEmitFnV1>,
    /// 可选路径/数值/单位提交回调。
    pub metric: Option<ZrHostDiagnosticsMetricFnV1>,
}

impl ZrHostDiagnosticsApiV1 {
    /// 返回所有可选槽为空的表；构造仅初始化值，不执行注册。
    pub const fn empty() -> Self {
        Self {
            emit: None,
            metric: None,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 事件的名称空间、名称及稳定哈希载体；切片不持有文本。
/// 当前事件适配器不消费此值；哈希的生成与冲突规则不由本类型验证。
pub struct ZrEventTypeId {
    /// 借用的事件名称空间文本。
    pub namespace: ZrByteSlice,
    /// 该名称空间内的事件名。
    pub name: ZrByteSlice,
    /// 提供者填写的稳定标识；本载体不计算或校验它。
    pub stable_hash: u64,
}

impl ZrEventTypeId {
    /// 返回空标识占位值；不分配文本或验证身份。
    pub const fn empty() -> Self {
        Self {
            namespace: ZrByteSlice::empty(),
            name: ZrByteSlice::empty(),
            stable_hash: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// V1 系统注册描述；文本在注册时复制，回调地址和 `user_data` 留到调度时使用。
/// # 约束
/// stage 是宿主 `SystemStage::ORDER` 的索引；当前路径采用保守 World 访问，忽略 invoke 状态。
/// # 参考
/// [`ZrHostRegisterSystemFnV1`] 同步复制名称，再把 invoke/user_data 放进系统执行闭包。
pub struct ZrSystemRegistrationV1 {
    /// 构造者指定的版本；当前 V1 解码路径不检查该字段。
    pub abi_version: u32,
    /// 描述布局字节数；当前 V1 解码路径不检查该字段。
    pub size_bytes: usize,
    /// 注册时复制并由注册表接纳的系统 ID。
    pub system_id: ZrByteSlice,
    /// 宿主阶段顺序表的索引，超范围会拒绝。
    pub stage: u32,
    /// 阶段内排序权重，交给系统 builder。
    pub order: i32,
    /// 同步借用的系统集合名称数组。
    pub set_names: *const ZrByteSlice,
    /// set_names 的元素数，零值表示无集合。
    pub set_count: usize,
    /// 同步借用的系统 ID 数组；本系统须先于所列系统运行。
    pub before: *const ZrByteSlice,
    /// before 的元素数。
    pub before_count: usize,
    /// 同步借用的系统 ID 数组；本系统须后于所列系统运行。
    pub after: *const ZrByteSlice,
    /// after 的元素数。
    pub after_count: usize,
    /// 留供后续调度的地址；None 时注册闭包不调用外部代码。
    pub invoke: Option<ZrNativeSystemInvokeFnV1>,
    /// 宿主不解释的 u64，后续原样回传给 invoke。
    pub user_data: u64,
}

impl ZrSystemRegistrationV1 {
    /// 构造无注册/回调的占位记录，填入本类型 size；不校验或取得外部资源。
    /// # 参数
    /// `abi_version` 原样保存，调用者按对应协议选择版本。
    pub const fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            size_bytes: core::mem::size_of::<Self>(),
            system_id: ZrByteSlice::empty(),
            stage: 0,
            order: 0,
            set_names: core::ptr::null(),
            set_count: 0,
            before: core::ptr::null(),
            before_count: 0,
            after: core::ptr::null(),
            after_count: 0,
            invoke: None,
            user_data: 0,
        }
    }
}

/// One exact component or external-resource access declaration for a V4 native system.
///
/// Raw numeric values keep the C ABI stable. The host validates all enum values, stable IDs,
/// capability grants, and duplicate/conflicting declarations before registering the system.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// V4 的单项访问声明；空构造值的 ID 仍为空，须补全才能注册。
/// 非空描述数组及稳定 ID 的内存须在同步解码期间对齐、初始化并保持可读。
/// # 参考
/// Runtime 的 `host_api_adapter/abi_decode/system.rs::read_v4_system_accesses` 校验数值和布局；
/// `ecs_registration/mod.rs::register_system_from_abi_v4` 继续核对授权。
pub struct ZrNativeSystemAccessV1 {
    /// 单项描述版本，当前宿主要求为 1。
    pub abi_version: u32,
    /// 当前宿主要求等于此结构的布局字节数。
    pub size_bytes: usize,
    /// read/write 数值之一，其他值会拒绝。
    pub mode: u32,
    /// component/resource 数值之一，其他值会拒绝。
    pub domain: u32,
    /// 同步借用的稳定 ID；后续访问计划核验格式和权限。
    pub stable_id: ZrByteSlice,
}

impl ZrNativeSystemAccessV1 {
    /// 返回 V1/read/component 默认值和空 ID；这是待补全的占位记录。
    pub const fn empty() -> Self {
        Self {
            abi_version: 1,
            size_bytes: core::mem::size_of::<Self>(),
            mode: ZR_NATIVE_SYSTEM_ACCESS_MODE_READ_V1,
            domain: ZR_NATIVE_SYSTEM_ACCESS_DOMAIN_COMPONENT_V1,
            stable_id: ZrByteSlice::empty(),
        }
    }
}

/// V4 registration descriptor with an explicit ECS conflict contract.
///
/// This record is intentionally separate from `ZrSystemRegistrationV1`: changing V1's layout
/// would silently corrupt existing native plugins instead of giving them a versioned opt-in.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// V4 系统描述，显式列出调度访问和线程资格；宿主接纳前校验而非自动授权。
/// # 使用时机与生命周期
/// 注册期借用数组与文本；之后仅保留复制的计划、回调地址和 `user_data`。
/// # 约束与参考
/// 当前接纳要求 abi_version=4、完整 size 和非空 accesses；空构造值须由调用方补全。
/// 见 [`ZrHostRegisterSystemFnV2`] 以及 Runtime 的
/// `plugin/loader/ecs_registration/tests/cases.rs::native_host_api_v4_system_enters_schedule_with_declared_access`。
pub struct ZrSystemRegistrationV2 {
    /// 当前宿主要求为 4，表示 V4 接纳契约。
    pub abi_version: u32,
    /// 当前宿主要求等于此结构的完整布局字节数。
    pub size_bytes: usize,
    /// 同步复制的系统注册 ID。
    pub system_id: ZrByteSlice,
    /// 宿主阶段顺序表的索引。
    pub stage: u32,
    /// 注册 builder 的阶段内排序权重。
    pub order: i32,
    /// 同步借用的集合名数组；非空时须有效且对齐。
    pub set_names: *const ZrByteSlice,
    /// 集合名元素数；宿主按列表预算校验。
    pub set_count: usize,
    /// 同步借用的系统 ID 数组；本系统须先于所列系统运行。
    pub before: *const ZrByteSlice,
    /// before 的元素数。
    pub before_count: usize,
    /// 同步借用的系统 ID 数组；本系统须后于所列系统运行。
    pub after: *const ZrByteSlice,
    /// after 的元素数。
    pub after_count: usize,
    /// 同步借用的访问描述数组；调用方保证对齐和可读性。
    pub accesses: *const ZrNativeSystemAccessV1,
    /// 访问项数量；当前 V4 必须非零且不超过宿主上限。
    pub access_count: usize,
    /// 主线程或 worker 数值；worker 还需相应 capability。
    pub thread_affinity: u32,
    /// 后续调度使用的地址，提供方须持续存活。
    pub invoke: Option<ZrNativeSystemInvokeFnV1>,
    /// 后续原样回传的插件自定义 u64。
    pub user_data: u64,
}

impl ZrSystemRegistrationV2 {
    /// 构造无注册/回调的占位记录，填入本类型 size；不校验或取得外部资源。
    /// # 参数
    /// `abi_version` 原样保存，调用者按对应协议选择版本。
    pub const fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            size_bytes: core::mem::size_of::<Self>(),
            system_id: ZrByteSlice::empty(),
            stage: 0,
            order: 0,
            set_names: core::ptr::null(),
            set_count: 0,
            before: core::ptr::null(),
            before_count: 0,
            after: core::ptr::null(),
            after_count: 0,
            accesses: core::ptr::null(),
            access_count: 0,
            thread_affinity: ZR_NATIVE_SYSTEM_THREAD_AFFINITY_MAIN_THREAD_ONLY_V1,
            invoke: None,
            user_data: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// 组件注册载体；当前宿主只复制 ID 与显示名，再由 scope 的模块归属确定插件 ID。
pub struct ZrComponentDescV1 {
    /// 构造者指定的版本；当前组件适配器不校验它。
    pub abi_version: u32,
    /// 声明布局长度；当前组件适配器不读取它。
    pub size_bytes: usize,
    /// 同步复制并接纳的组件稳定 ID。
    pub type_id: ZrByteSlice,
    /// 同步复制的组件显示名。
    pub display_name: ZrByteSlice,
    /// 预留 schema 字节视图，当前组件适配器不消费。
    pub schema: ZrByteSlice,
    /// 预留存储类别数值，当前组件适配器不消费。
    pub storage_kind: u32,
}

impl ZrComponentDescV1 {
    /// 构造无注册/回调的占位记录，填入本类型 size；不校验或取得外部资源。
    /// # 参数
    /// `abi_version` 原样保存，调用者按对应协议选择版本。
    pub const fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            size_bytes: core::mem::size_of::<Self>(),
            type_id: ZrByteSlice::empty(),
            display_name: ZrByteSlice::empty(),
            schema: ZrByteSlice::empty(),
            storage_kind: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// 共享状态回调的可选表；当前原生加载器另用 `NativePluginBehaviorV4`。
/// 本表不管理状态缓冲区或提供方寿命，空表也不声明保存/恢复能力。
pub struct ZrPluginStateSnapshotApiV1 {
    /// 构造者指定的共享表版本。
    pub abi_version: u32,
    /// 共享表布局字节数。
    pub size_bytes: usize,
    /// 可选状态保存回调；输出内存由调用者提供。
    pub save: Option<ZrPluginSnapshotSaveFnV1>,
    /// 可选状态恢复回调；输入仅同步借用。
    pub restore: Option<ZrPluginSnapshotRestoreFnV1>,
}

impl ZrPluginStateSnapshotApiV1 {
    /// 构造无注册/回调的占位记录，填入本类型 size；不校验或取得外部资源。
    /// # 参数
    /// `abi_version` 原样保存，调用者按对应协议选择版本。
    pub const fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            size_bytes: core::mem::size_of::<Self>(),
            save: None,
            restore: None,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// 共享插件的卸载与事件回调槽；复制表不会持有插件实例或动态库。
/// 当前共享入口报告没有生产加载调用；事件签名另由声音适配器直接使用。
/// # 参考
/// 本 crate 的 `tests/contracts.rs::plugin_api_table_records_optional_event_callback` 固定表布局；
/// [`ZrPluginEventCallbackFnV1`] 说明事件调用的借用期限和两层状态。
pub struct ZrPluginApiV1 {
    /// 构造者声明的共享回调表版本。
    pub abi_version: u32,
    /// 共享回调表完整布局字节数。
    pub size_bytes: usize,
    /// 可选卸载地址，不附带库或实例所有权。
    pub unload: Option<ZrPluginUnloadFnV1>,
    /// 可选同步事件地址；返回状态与结果内状态分开。
    pub invoke_event: Option<ZrPluginEventCallbackFnV1>,
}

impl ZrPluginApiV1 {
    /// 返回指定版本、两个回调均缺席的共享表；不取得插件或动态库所有权。
    /// # 参数
    /// `abi_version` 原样保存，未作兼容性检查。
    pub const fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            size_bytes: core::mem::size_of::<Self>(),
            unload: None,
            invoke_event: None,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// 共享入口的借用报告布局；当前原生加载器的 SDK 报告带 epoch，不能互换。
/// 本类型不复制清单、模块数组或 API 表，也不约束其内存寿命；解释者须先建立读取前提。
/// # 参考
/// Runtime 的 `plugin/loader/native_plugin_abi.rs::call_native_plugin_entry` 使用另一套 SDK 报告；
/// 本共享布局只由入口类型别名声明和本 crate 重导出，不应拿给该探测器解释。
pub struct ZrPluginEntryReportV1 {
    /// 共享报告版本，不能当作原生 SDK layout epoch。
    pub abi_version: u32,
    /// 借用的插件身份文本。
    pub plugin_id: ZrByteSlice,
    /// 借用的包清单字节。
    pub package_manifest: ZrByteSlice,
    /// 借用的模块描述数组，无 Rust 所有权。
    pub modules: *const ZrPluginModuleDescriptorV1,
    /// modules 指向的元素数量。
    pub module_count: usize,
    /// 借用的报告诊断字节。
    pub diagnostics: ZrByteSlice,
    /// 借用的共享插件回调表地址。
    pub api: *const ZrPluginApiV1,
}

impl ZrPluginEntryReportV1 {
    /// 构造所有借用为空的报告，不分配清单或模块数组。
    /// # 参数
    /// `abi_version` 原样保存，不表示入口已完成协商。
    pub const fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            plugin_id: ZrByteSlice::empty(),
            package_manifest: ZrByteSlice::empty(),
            modules: core::ptr::null(),
            module_count: 0,
            diagnostics: ZrByteSlice::empty(),
            api: core::ptr::null(),
        }
    }
}
