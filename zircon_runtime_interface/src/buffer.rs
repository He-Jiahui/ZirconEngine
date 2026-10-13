//! 跨 ABI 的字节借用、输出分配载体及各入口的资源预算。
//!
//! 原始指针不由 Rust 借用关系或自动析构维持有效期，也不探测地址是否可读。Runtime 的有界 JSON
//! 编解码与 Host 的外来输出检查消费这些预算；分配释放分别遵循旧式回调协议和
//! [`ZrOwnedResultV2`] 的会话分配 ID 协议。

use core::ptr;

use crate::handles::ZrRuntimeAllocationId;
use crate::runtime_api::{
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1,
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1,
};
use crate::status::ZrStatus;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 字节视图在建立 Rust 借用前可检查出的形状或长度错误。
/// 地址映射、初始化和借用有效期仍由 [`ZrByteSlice::checked_slice`] 的调用方保证。
pub enum ZrByteSliceError {
    /// 非空视图没有数据地址；不进入原始指针读取。
    NullWithNonZeroLength,
    /// 长度超出 Rust 切片可表示的地址范围；`len` 是收到的长度。
    LengthExceedsAddressSpace { len: usize },
    /// 长度超过当前入口预算；`len` 是实际长度，`limit` 是该次读取的上限。
    LengthExceedsLimit { len: usize, limit: usize },
}

impl ZrByteSliceError {
    /// 供入口把资源超额映射为 [`crate::status::ZrStatusCode::LimitExceeded`]。
    /// 空指针和地址范围错误继续按无效参数处理。
    pub const fn is_limit_exceeded(self) -> bool {
        matches!(self, Self::LengthExceedsLimit { .. })
    }
}

// 以下预算由 Host 与 Runtime 共同执行，跨 ABI 解析和编码都必须选用对应入口的限额。
/// 有界 JSON 语法预检允许的最大容器嵌套层数。
pub const ZR_RUNTIME_JSON_MAX_NESTING_DEPTH_V1: usize = 128;
/// 状态诊断的同步读取上限；Runtime 的同线程诊断暂存区采用相同容量。
pub const ZR_RUNTIME_STATUS_DIAGNOSTICS_MAX_ENCODED_BYTES_V1: usize = 4 * 1024;
/// 创建会话时，配置中的 profile 名称字节上限。
pub const ZR_RUNTIME_SESSION_PROFILE_MAX_ENCODED_BYTES_V1: usize = 64;
/// 会话启动配置中各项目路径的独立字节上限。
pub const ZR_RUNTIME_PROJECT_PATH_MAX_ENCODED_BYTES_V1: usize = 32 * 1024;
/// 输入事件借用负载在复制到会话执行上下文前的字节上限。
pub const ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1: usize = 256 * 1024;
/// Maximum UTF-8 text body admitted by the clipboard host bridge.
///
/// This leaves deterministic room for the typed JSON request/result envelope under the
/// 256 KiB host-output and runtime-event ceilings.
/// 剪贴板正文先受此 UTF-8 字节预算约束，为外层请求和结果信封保留空间。
pub const ZR_RUNTIME_CLIPBOARD_TEXT_MAX_ENCODED_BYTES_V1: usize = 32 * 1024;
/// 原生插件注册适配器读取单个字符串时使用的字节上限。
pub const ZR_RUNTIME_NATIVE_STRING_MAX_ENCODED_BYTES_V1: usize = 256 * 1024;
/// 原生插件注册适配器读取字符串视图数组时使用的元素上限。
pub const ZR_RUNTIME_NATIVE_STRING_LIST_MAX_ITEMS_V1: usize = 16_384;

/// 帧请求和 RGBA 形状验证允许的单轴尺寸上限。
pub const ZR_RUNTIME_FRAME_MAX_DIMENSION_V1: u32 = 16_384;
/// 帧像素视图允许的字节上限；宽高检查还须核对每像素四字节的总量。
pub const ZR_RUNTIME_FRAME_MAX_RGBA_BYTES_V1: usize = 256 * 1024 * 1024;

/// 描述一次跨 ABI 负载允许的字节、元素、嵌套深度与处理时间预算。
/// 入口在读取外来字节前选择对应限额，返回值编码也遵循相应的输出限额。
///
/// # 参考
/// `bounded_json::decode` 先检查字节、嵌套与时间，再用入口提供的计数回调核对业务条目；
/// `bounded_json::encode` 和 `bounded_json::validate` 检查输出。此记录本身不执行检查。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimePayloadLimitV1 {
    /// 整个编码信封的字节上限，不等于业务正文长度。
    pub max_encoded_bytes: usize,
    /// 由入口的计数回调定义的业务条目上限，例如事件、请求或查询行。
    pub max_items: usize,
    /// JSON 编解码检查的嵌套层数上限。
    pub max_nesting_depth: usize,
    /// 编解码检查点使用的处理时间预算，单位为微秒；此值不创建抢占式定时器。
    pub max_processing_time_micros: u64,
    /// 消费端是否接受规范空负载；允许空负载仍不使残缺的 JSON 成为有效 JSON。
    pub allow_empty: bool,
}

impl ZrRuntimePayloadLimitV1 {
    /// 组合入口预算；参数依次为编码字节、业务条目与微秒处理时间上限。
    /// 返回值采用共享嵌套深度并默认拒绝空负载，实际检查由编解码消费端执行。
    ///
    /// # 参考
    /// 各请求和输出预算常量由此构造；[`Self::allow_empty`] 单独表达批次允许空值的协议。
    /// `bounded_json::decode`、`bounded_json::encode` 是实际消费预算的入口。
    pub const fn new(
        max_encoded_bytes: usize,
        max_items: usize,
        max_processing_time_micros: u64,
    ) -> Self {
        Self {
            max_encoded_bytes,
            max_items,
            max_nesting_depth: ZR_RUNTIME_JSON_MAX_NESTING_DEPTH_V1,
            max_processing_time_micros,
            allow_empty: false,
        }
    }

    /// 返回允许规范空负载的预算副本，供无待处理输出的批次入口使用。
    pub const fn allow_empty(mut self) -> Self {
        self.allow_empty = true;
        self
    }
}

/// profiling 控制请求的 JSON 预算；包括诊断和模块组合回执查询。
pub const ZR_RUNTIME_PROFILE_REQUEST_LIMIT_V1: ZrRuntimePayloadLimitV1 =
    ZrRuntimePayloadLimitV1::new(256 * 1024, 1_024, 25_000);
/// 插件事件订阅描述的 JSON 预算。
pub const ZR_RUNTIME_PLUGIN_EVENT_SUBSCRIBE_REQUEST_LIMIT_V1: ZrRuntimePayloadLimitV1 =
    ZrRuntimePayloadLimitV1::new(256 * 1024, 64, 10_000);
/// 提交运行时操作请求时采用的 JSON 预算。
pub const ZR_RUNTIME_OPERATION_REQUEST_LIMIT_V1: ZrRuntimePayloadLimitV1 =
    ZrRuntimePayloadLimitV1::new(1024 * 1024, 16_384, 25_000);
/// 世界查询请求的 JSON 预算；解码入口使用 `WorldQuery::request_item_count` 计数。
pub const ZR_RUNTIME_WORLD_QUERY_REQUEST_LIMIT_V1: ZrRuntimePayloadLimitV1 =
    ZrRuntimePayloadLimitV1::new(1024 * 1024, 16_384, 25_000);
/// 世界 watch 注册描述的 JSON 预算。
pub const ZR_RUNTIME_WORLD_WATCH_REQUEST_LIMIT_V1: ZrRuntimePayloadLimitV1 =
    ZrRuntimePayloadLimitV1::new(256 * 1024, 1_024, 10_000);
/// 无障碍动作事件请求的 JSON 预算。
pub const ZR_RUNTIME_ACCESSIBILITY_ACTION_REQUEST_LIMIT_V1: ZrRuntimePayloadLimitV1 =
    ZrRuntimePayloadLimitV1::new(256 * 1024, 4_096, 10_000);
/// 宿主剪贴板结果事件信封的 JSON 预算，正文另受文本字节上限约束。
pub const ZR_RUNTIME_CLIPBOARD_RESULT_REQUEST_LIMIT_V1: ZrRuntimePayloadLimitV1 =
    ZrRuntimePayloadLimitV1::new(256 * 1024, 1, 10_000);
/// 视口相机事件请求的 JSON 预算。
pub const ZR_RUNTIME_VIEWPORT_CAMERA_REQUEST_LIMIT_V1: ZrRuntimePayloadLimitV1 =
    ZrRuntimePayloadLimitV1::new(4 * 1024, 1, 1_000);

/// 无障碍树输出的 JSON 预算。
pub const ZR_RUNTIME_ACCESSIBILITY_TREE_OUTPUT_LIMIT_V1: ZrRuntimePayloadLimitV1 =
    ZrRuntimePayloadLimitV1::new(16 * 1024 * 1024, 65_536, 250_000);
/// Runtime 向 Host 排出的请求批次预算；没有请求时允许规范空输出。
pub const ZR_RUNTIME_HOST_REQUEST_OUTPUT_LIMIT_V1: ZrRuntimePayloadLimitV1 =
    ZrRuntimePayloadLimitV1::new(256 * 1024, 256, 10_000).allow_empty();
/// profiling 响应的 JSON 预算；允许协议规定的空输出。
pub const ZR_RUNTIME_PROFILE_RESPONSE_OUTPUT_LIMIT_V1: ZrRuntimePayloadLimitV1 =
    ZrRuntimePayloadLimitV1::new(16 * 1024 * 1024, 65_536, 250_000).allow_empty();
/// 收割运行时操作结果时的 JSON 输出预算。
pub const ZR_RUNTIME_OPERATION_RESULT_OUTPUT_LIMIT_V1: ZrRuntimePayloadLimitV1 =
    ZrRuntimePayloadLimitV1::new(1024 * 1024, 16_384, 25_000);
/// 插件事件交付页的输出预算，与页协议的字节数和交付条数保持一致。
pub const ZR_RUNTIME_PLUGIN_EVENT_OUTPUT_LIMIT_V1: ZrRuntimePayloadLimitV1 =
    ZrRuntimePayloadLimitV1::new(
        ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1,
        ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1,
        10_000,
    )
    .allow_empty();
/// 世界查询结果的 JSON 输出预算。
pub const ZR_RUNTIME_WORLD_QUERY_OUTPUT_LIMIT_V1: ZrRuntimePayloadLimitV1 =
    ZrRuntimePayloadLimitV1::new(1024 * 1024, 16_384, 25_000);
/// 世界失效通知批次的 JSON 输出预算；无通知时允许规范空输出。
pub const ZR_RUNTIME_WORLD_INVALIDATION_OUTPUT_LIMIT_V1: ZrRuntimePayloadLimitV1 =
    ZrRuntimePayloadLimitV1::new(1024 * 1024, 16_384, 25_000).allow_empty();

/// 借用型 C ABI 字节视图；数据仍由提供方持有，接收方不得跨其有效期保留引用。
/// checked_slice 只检查载体形状和长度，指针可读性与引用生命周期仍由调用方保证。
///
/// # 参考
/// [`Self::checked_slice`] 建立有界借用；`bounded_json::checked_bytes` 用入口预算调用它。
/// `read_utf8` 在插件注册适配器中同步读取并复制字符串，不保留提供方的字节借用。
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrByteSlice {
    /// 提供方持有的首字节地址；空视图不会解引用此字段。
    pub data: *const u8,
    /// 字节数；是否为空仅由此字段决定，不表示地址通过验证。
    pub len: usize,
}

impl ZrByteSlice {
    /// 构造空借用，供缺省字段或不带负载的调用使用。
    pub const fn empty() -> Self {
        Self {
            data: ptr::null(),
            len: 0,
        }
    }

    /// 借用静态字节，不分配或复制；返回视图保留输入的静态存储有效期。
    pub const fn from_static(bytes: &'static [u8]) -> Self {
        Self {
            data: bytes.as_ptr(),
            len: bytes.len(),
        }
    }

    /// 仅判断长度为零；非空视图仍须经过有界检查并满足指针借用前提。
    pub const fn is_empty(self) -> bool {
        self.len == 0
    }

    /// Validates and views this ABI slice as a Rust byte slice.
    ///
    /// # Safety
    ///
    /// After this method validates the carrier shape and length, a non-empty `data`
    /// pointer must still point to `len` initialized bytes that remain valid for the
    /// returned lifetime.
    ///
    /// # 参数与返回值
    /// `limit` 是当前入口允许读取的最大字节数；返回借用或具体的形状/预算错误。
    /// 零长度直接返回空切片，不读取 `data`，且不表示所在协议一定允许空负载。
    ///
    /// # 约束与生命周期
    /// 非空数据须处于单个分配内、已初始化，地址加长度不能回绕；返回借用存续期间
    /// 不得释放或修改这些字节。泛型生命周期 `'a` 不由载体自动建立，须由调用方
    /// 限制到实际提供方的有效期；同步请求通常只借用到该次调用返回。
    ///
    /// # 参考
    /// `bounded_json::checked_bytes` 只在数值检查后借用；`handle_event` 在会话执行前复制负载。
    /// `byte_slices_reject_invalid_shape_and_limits_before_slice_construction` 核对提前拒绝分支。
    pub unsafe fn checked_slice<'a>(self, limit: usize) -> Result<&'a [u8], ZrByteSliceError> {
        if self.len == 0 {
            return Ok(&[]);
        }
        if self.data.is_null() {
            return Err(ZrByteSliceError::NullWithNonZeroLength);
        }
        if self.len > isize::MAX as usize {
            return Err(ZrByteSliceError::LengthExceedsAddressSpace { len: self.len });
        }
        if self.len > limit {
            return Err(ZrByteSliceError::LengthExceedsLimit {
                len: self.len,
                limit,
            });
        }
        // SAFETY: 数值检查只确定形状；本 unsafe 方法的调用者须保证对应字节位于同一 allocation、已初始化，并在整个只读借用期间可读且保持有效。
        Ok(unsafe { core::slice::from_raw_parts(self.data, self.len) })
    }
}

/// 调用方借给同步回调的可写缓冲区；data 与 written 在回调期间必须有效。
/// 回调只能在 capacity 范围内写入，并通过 written 报告实际写入量。
/// 数据区和计数位置在写入时不得冲突；载体不分配存储，也不延长调用方的借用期。
///
/// # 参考
/// `native_host_bridge_call_v1` 将输出区传给已启用的桥接方法；`NativePluginBridgeMethodCallV3::output`
/// 保留同一同步输出协议。空构造器仅用于缺省或不会写出的调用路径。
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrByteBufferRef {
    /// 调用方提供的可写字节区，仅在同步回调约定的有效期内可用。
    pub data: *mut u8,
    /// `data` 最多可写的字节数，不随写入量改变。
    pub capacity: usize,
    /// 调用方提供的实际写入量输出位置；回调使用时须可写且对齐。
    pub written: *mut usize,
}

impl ZrByteBufferRef {
    /// 构造没有可写区或计数位置的载体；不会提供可直接写入的存储。
    pub const fn empty() -> Self {
        Self {
            data: ptr::null_mut(),
            capacity: 0,
            written: ptr::null_mut(),
        }
    }

    /// 仅检查空地址和零容量；不会检查 `written` 或完整回调前提。
    pub const fn is_empty(self) -> bool {
        self.data.is_null() && self.capacity == 0
    }
}

/// 由原分配方提供的释放入口；接收方不得用自己的分配器回收外来字节。
/// 参数是提供方交付的原载体，返回释放状态；调用时须保持回调所在库有效。
pub type ZrFreeBytesFn = unsafe extern "C" fn(ZrOwnedByteBuffer) -> ZrStatus;

/// 带分配方释放回调的旧式跨 ABI 输出；读取后由接收方调用配套 free。
/// free 缺失时接收方不能安全地自行释放 data。
/// 该载体可复制，但复制字段不新增释放权；原释放回调只应收到一次有效分配。
///
/// # 参考
/// [`crate::runtime_api::ZrRuntimeHostFetchFnV1`] 和 `ZrHostAssetApiV1::request`
/// 声明此载体的输出参数；[`ZrFreeBytesFn`] 定义归还原分配方的入口。类型声明不安装回调。
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ZrOwnedByteBuffer {
    /// 原分配方拥有的数据地址，接收方只在协议有效期内读取。
    pub data: *mut u8,
    /// 当前输出的有效字节数，不能据此推断分配大小。
    pub len: usize,
    /// 交还原释放回调的分配元数据，接收方不能据此重建本地容器。
    pub capacity: usize,
    /// 原分配方解释的不透明标识；接收方原样交还。
    pub owner_token: u64,
    /// 原分配方的回收函数；缺失时不授权接收方自行回收。
    pub free: Option<ZrFreeBytesFn>,
}

impl ZrOwnedByteBuffer {
    /// 构造没有数据、分配标识或释放回调的缺省输出。
    pub const fn empty() -> Self {
        Self {
            data: ptr::null_mut(),
            len: 0,
            capacity: 0,
            owner_token: 0,
            free: None,
        }
    }

    /// 只判断地址、长度和容量的空形状；不校验回调或分配标识。
    pub const fn is_empty(self) -> bool {
        self.data.is_null() && self.len == 0 && self.capacity == 0
    }
}

/// Immutable runtime-owned output whose allocation is released by opaque ID.
///
/// 非空结果由 Runtime 会话持有；Host 须在对应会话及动态库仍存活时调用
/// release_allocation，不能按本地 Vec 或其他分配器的规则释放 data。
///
/// # 使用时机与生命周期
/// Host 读取或复制输出后，通过原会话的释放入口交还分配 ID；释放后全部字节借用失效。
/// 此载体不自动回收，移动载体也不延长会话或动态库的有效期。
///
/// # 参考
/// `register_runtime_allocation_in_action` 保留字节所有者并发放 ID；`validate_owned_result`
/// 只核对结果形状。`RuntimeOwnedOutputReleaser::release` 经原会话调用统一释放槽，
/// `release_runtime_allocation` 核对归属并移除原分配，成功后不再允许使用旧字节视图。
#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub struct ZrOwnedResultV2 {
    /// Runtime 持有的不可变字节地址；非空输出须配有有效分配 ID。
    pub data: *const u8,
    /// 固定宽度的 ABI 字节数；Host 建立切片前须核对本地地址范围和入口预算。
    pub len: u64,
    /// 原会话的分配身份；接收方保留唯一释放权并原样交还。
    pub allocation: ZrRuntimeAllocationId,
}

impl ZrOwnedResultV2 {
    /// 构造空地址、零长度与无效分配 ID 的规范空输出；不需要释放。
    pub const fn empty() -> Self {
        Self {
            data: ptr::null(),
            len: 0,
            allocation: ZrRuntimeAllocationId::invalid(),
        }
    }

    /// 仅识别规范空三元组；部分字段为空的非规范载体仍须由消费端拒绝或清理。
    pub const fn is_empty(&self) -> bool {
        self.data.is_null() && self.len == 0 && !self.allocation.is_valid()
    }
}
