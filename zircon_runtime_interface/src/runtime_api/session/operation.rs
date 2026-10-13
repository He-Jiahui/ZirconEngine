//! 定义会话操作的 JSON 提交/结果、固定布局轮询状态及三个 ABI 入口。
//!
//! Runtime 在帧推进中执行准备和应用；poll 只读取状态，harvest 消费可收获结果。
//! 句柄不持有会话，解码后的值与原 ABI 输出分配有不同的释放边界。
//! # 参考
//! [`ZrRuntimeOperationStatusV2`]、[`ZrRuntimeOperationResultV1`]、
//! `RuntimeOperationService::prepare_harvest`。

use serde::{Deserialize, Serialize};

use crate::{ZrByteSlice, ZrOwnedResultV2, ZrRuntimeSessionHandle, ZrStatus};

/// 一个 Runtime 会话操作的数值身份，零表示无效占位。
///
/// # 使用时机与生命周期
/// 提交成功后用于原会话的轮询和收获；私有元组字段 `.0` 保存原始 ID。
/// 复制、反序列化或非零检查不确认服务中仍有该任务，也不延长会话或结果的保留时间。
/// # 参考
/// [`ZrRuntimeSubmitOperationFnV1`]、[`ZrRuntimePollOperationFnV2`]、`RuntimeOperationService::poll`。
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ZrRuntimeOperationHandle(u64);

impl ZrRuntimeOperationHandle {
    /// 包装 `raw`，不查询操作注册表；实际 ABI 调用仍须核对会话归属。
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// 返回零占位，供提交调用方在状态与非零输出都确认前保存。
    pub const fn invalid() -> Self {
        Self(0)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }

    /// 仅检查 ID 非零；已过期、已收获或来自另一会话的 ID 仍可能通过此检查。
    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }
}

/// 提交给会话操作服务的自包含 JSON 请求，字段名保持 snake_case。
///
/// # 使用时机与生命周期
/// Host 编码后只在同步提交期间借用字节；当前 Runtime 复制并有界解码，再按
/// operation_id 查找处理器。构造值不等于操作已被接受。
/// # 参考
/// [`ZrRuntimeSubmitOperationFnV1`]、`RuntimeOperationService::submit_with_raw_admission`、
/// `runtime_operation_contracts_round_trip_without_transport_specific_types`。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZrRuntimeOperationSubmitRequestV1 {
    /// JSON 请求的 V1 版本，与 poll 状态的 V2 区分。
    pub abi_version: u32,
    /// 注册处理器的精确键；服务拒绝空白键和未注册键，构造器不规范化。
    pub operation_id: String,
    /// 处理器解释的拥有者 JSON 值；编码、图计数和保留字节在消费路径限额。
    pub payload: serde_json::Value,
}

impl ZrRuntimeOperationSubmitRequestV1 {
    /// 在编码前构造请求并取得操作键字符串的所有权。
    ///
    /// # 参数与返回值
    /// 版本、操作键和 payload 原样组成请求；不查处理器或验证预算。
    /// # 参考
    /// [`ZrRuntimeSubmitOperationFnV1`]、`RuntimeOperationService::submit_with_raw_admission`。
    pub fn new(
        abi_version: u32,
        operation_id: impl Into<String>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            abi_version,
            operation_id: operation_id.into(),
            payload,
        }
    }
}

/// 固定布局状态中的已知阶段编码。
///
/// # 约束
/// 终态分类用于停止等待；当前服务只有 Completed/Failed 且结果仍在位时允许收获。
/// Cancelled/Expired/Harvested 虽是终态，收获仍返回对应错误。
/// # 参考
/// [`Self::from_raw`]、[`Self::is_terminal`]、`validate_harvestable_task`。
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZrRuntimeOperationPhase {
    /// 已接纳并等待 owner 帧调度的任务。
    Queued = 1,
    /// owner 快照已交给准备工作，尚未进入应用队列。
    Preparing = 2,
    /// 准备结果已保留，等待 owner 应用到世界。
    ReadyToApply = 3,
    /// 应用成功，结果在保留期内可被收获。
    Completed = 4,
    /// 失败结果已保存；收获返回失败 outcome。
    Failed = 5,
    /// 任务已取消，当前服务不再提供可收获结果。
    Cancelled = 6,
    /// deadline 或终态保留期限已到，具体原因由 detail kind 区分。
    Expired = 7,
    /// 收获已提交，原结果已取走；重复收获不会再次交付。
    Harvested = 8,
}

impl ZrRuntimeOperationPhase {
    pub const fn raw(self) -> u32 {
        self as u32
    }

    /// 映射已知 wire 编码，未知值返回 None，供 Host 拒绝外来状态。
    pub const fn from_raw(value: u32) -> Option<Self> {
        match value {
            1 => Some(Self::Queued),
            2 => Some(Self::Preparing),
            3 => Some(Self::ReadyToApply),
            4 => Some(Self::Completed),
            5 => Some(Self::Failed),
            6 => Some(Self::Cancelled),
            7 => Some(Self::Expired),
            8 => Some(Self::Harvested),
            _ => None,
        }
    }

    /// 判断是否结束等待；此分类不担保存在可收获结果。
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Cancelled | Self::Expired | Self::Harvested
        )
    }
}

/// poll V2 的结构化原因码，须与 detail_value 配对解释。
///
/// # 约束
/// Host 拒绝未知编码；容量原因码的声明不意味着每一种接纳失败都会创建可轮询任务。
/// # 参考
/// [`ZrRuntimeOperationStatusV2`]、`ensure_operation_status`、`RuntimeOperationService::commit_harvest`。
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZrRuntimeOperationDetailKindV2 {
    /// 当前状态没有额外原因值。
    None = 0,
    /// 提交时记录的队列深度，detail_value 是当时的数量。
    QueueDepth = 1,
    /// 协议声明的任务数量容量原因码；具体失败输出由接纳路径决定。
    AdmissionCountLimit = 2,
    /// 保留字节预算导致的失败，当前 producer 写入所用预算上限。
    AdmissionByteLimit = 3,
    /// 任务 deadline 到期，当前 producer 写入经过的毫秒数。
    DeadlineElapsed = 4,
    /// 任务已取消，当前 producer 的 detail_value 为零。
    Cancelled = 5,
    /// 准备工作发生 panic，由失败完成路径转为状态。
    WorkerPanic = 6,
    /// owner 应用失败或 panic，错误文字保存在失败结果。
    OwnerApplyFailed = 7,
    /// 未收获终态达到保留期限，当前 producer 写入终态经过的毫秒数。
    TerminalTtlElapsed = 8,
    /// 结果已收获，当前 producer 写入收获前阶段的 raw 编码。
    Harvested = 9,
    /// 准备完成通道丢失，由完成消费路径报告。
    WorkerChannelLost = 10,
}

impl ZrRuntimeOperationDetailKindV2 {
    pub const fn raw(self) -> u32 {
        self as u32
    }

    /// 返回已知原因码或 None；未知值不等于编码零的 None 原因。
    pub const fn from_raw(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::QueueDepth),
            2 => Some(Self::AdmissionCountLimit),
            3 => Some(Self::AdmissionByteLimit),
            4 => Some(Self::DeadlineElapsed),
            5 => Some(Self::Cancelled),
            6 => Some(Self::WorkerPanic),
            7 => Some(Self::OwnerApplyFailed),
            8 => Some(Self::TerminalTtlElapsed),
            9 => Some(Self::Harvested),
            10 => Some(Self::WorkerChannelLost),
            _ => None,
        }
    }
}

/// 不含分配的操作状态快照，Host 在调用成功后才读取字段。
///
/// # 使用时机与生命周期
/// poll 不推进工作，帧 tick 才推进服务；快照不能替代同会话结果收获。
/// Host 核对 V2、reserved、请求句柄与已知阶段/原因；进度字段不包含结果。
/// # 参考
/// [`ZrRuntimePollOperationFnV2`]、[`ZrRuntimeOperationResultV1`]、
/// `RuntimeOperationTask::status`、`ensure_operation_status`。
/// 轮询状态只承载进度与终止原因；终态结果仍须用同一会话和句柄单独 harvest。
/// Fixed-layout, allocation-free operation status returned by the current poll ABI.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimeOperationStatusV2 {
    /// 固定状态布局的 V2 版本，与提交和结果 JSON 的 V1 不同。
    pub abi_version: u32,
    /// 阶段 raw 编码，须通过 phase() 检查是否已知。
    pub phase: u32,
    /// 原因 raw 编码，须通过 detail_kind() 检查是否已知。
    pub detail_kind: u32,
    /// 当前布局要求为零的保留字段，Host 消费时核对。
    pub reserved: u32,
    /// 本快照所属操作，必须与本次 poll 请求的句柄一致。
    pub handle: ZrRuntimeOperationHandle,
    /// producer 的完成工作量；当前任务快照以终态为一、非终态为零。
    pub completed_work: u64,
    /// 同一进度口径的总量；当前任务快照为一，构造器不校验两数关系。
    pub total_work: u64,
    /// 由 detail_kind 选择语义的数值，不能单独当作错误字符串或时间。
    pub detail_value: u64,
}

impl ZrRuntimeOperationStatusV2 {
    /// 构造 V2 状态并清零 reserved，其余数值由调用者提供。
    ///
    /// # 参数与返回值
    /// handle 标识任务；phase/detail_kind 转为 wire 编码；两个 work 值和 detail_value
    /// 原样保存。返回快照，不查询注册表或判定数值关系。
    /// # 参考
    /// `RuntimeOperationTask::status`、`runtime_operation_status_v2_is_fixed_layout_and_result_keeps_terminal_outcome`。
    pub const fn new(
        handle: ZrRuntimeOperationHandle,
        phase: ZrRuntimeOperationPhase,
        completed_work: u64,
        total_work: u64,
        detail_kind: ZrRuntimeOperationDetailKindV2,
        detail_value: u64,
    ) -> Self {
        Self {
            abi_version: crate::version::ZIRCON_RUNTIME_ABI_VERSION_V2,
            phase: phase.raw(),
            detail_kind: detail_kind.raw(),
            reserved: 0,
            handle,
            completed_work,
            total_work,
            detail_value,
        }
    }

    /// 只解码 phase 字段，不同时核对版本、句柄或 reserved。
    pub const fn phase(&self) -> Option<ZrRuntimeOperationPhase> {
        ZrRuntimeOperationPhase::from_raw(self.phase)
    }

    /// 只解码原因字段；返回 None 表示未知编码，与已知 None 原因不同。
    pub const fn detail_kind(&self) -> Option<ZrRuntimeOperationDetailKindV2> {
        ZrRuntimeOperationDetailKindV2::from_raw(self.detail_kind)
    }
}

/// 收获 JSON 的业务终态，使用 status 标签和 snake_case 变体名。
///
/// # 生命周期与约束
/// Succeeded.output 拥有成功 JSON 值，Failed.error 拥有诊断字符串；
/// 解码后的 Rust 值与承载它的原 ABI allocation 分别管理所有权。
/// # 参考
/// [`ZrRuntimeOperationResultV1`]、[`ZrRuntimeHarvestOperationFnV2`]。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ZrRuntimeOperationOutcomeV1 {
    /// 成功分支；内联 output 是处理器准备并由 owner 应用确认的结果值。
    Succeeded { output: serde_json::Value },
    /// 失败分支；内联 error 是结果级诊断，不替代 ABI 调用状态。
    Failed { error: String },
}

/// 从服务收获并有界解码的结果信封，JSON 字段名保持 snake_case。
///
/// # 使用时机与生命周期
/// Host 先核对调用状态，再验证 V1 与请求句柄；解码失败也须释放原 ABI allocation。
/// 收获提交后服务清除保留结果；此值拥有字符串和 JSON，可在释放字节后使用。
/// # 参考
/// [`ZrRuntimeOperationOutcomeV1`]、[`ZrRuntimeHarvestOperationFnV2`]、
/// `ensure_operation_output_handle`、`RuntimeOperationService::commit_harvest`。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 一次 harvest 的 JSON 结果；调用方必须将返回句柄与请求句柄核对后消费。
pub struct ZrRuntimeOperationResultV1 {
    /// 结果 JSON 的 V1 版本，由 Host 结果验证核对。
    pub abi_version: u32,
    /// 必须与本次 harvest 请求匹配的操作身份。
    pub handle: ZrRuntimeOperationHandle,
    /// 原任务的注册操作键，由结果构造时保留。
    pub operation_id: String,
    /// 成功输出或失败诊断，不表示 harvest ABI 调用本身失败。
    pub outcome: ZrRuntimeOperationOutcomeV1,
}

impl ZrRuntimeOperationResultV1 {
    /// 以拥有者 output 构造成功结果，字符串转换不查服务注册表。
    ///
    /// # 参数与返回值
    /// 版本、句柄、操作键组成信封，output 移入 Succeeded；结果尚未编码或被 Host 接受。
    /// # 参考
    /// [`Self::succeeded_output`]、`RuntimeOperationService` 的完成路径。
    pub fn succeeded(
        abi_version: u32,
        handle: ZrRuntimeOperationHandle,
        operation_id: impl Into<String>,
        output: serde_json::Value,
    ) -> Self {
        Self {
            abi_version,
            handle,
            operation_id: operation_id.into(),
            outcome: ZrRuntimeOperationOutcomeV1::Succeeded { output },
        }
    }

    /// 把错误转换为拥有者字符串并构造失败结果。
    ///
    /// # 参数与返回值
    /// 版本、句柄、操作键组成信封，error 移入 Failed；不验证调用状态或释放分配。
    /// # 参考
    /// [`Self::failure`]、`RuntimeOperationService` 的失败完成路径。
    pub fn failed(
        abi_version: u32,
        handle: ZrRuntimeOperationHandle,
        operation_id: impl Into<String>,
        error: impl Into<String>,
    ) -> Self {
        Self {
            abi_version,
            handle,
            operation_id: operation_id.into(),
            outcome: ZrRuntimeOperationOutcomeV1::Failed {
                error: error.into(),
            },
        }
    }

    /// 成功时借用内部 JSON，失败时返回 None；借用不能长于结果值的存活期。
    pub fn succeeded_output(&self) -> Option<&serde_json::Value> {
        match &self.outcome {
            ZrRuntimeOperationOutcomeV1::Succeeded { output } => Some(output),
            ZrRuntimeOperationOutcomeV1::Failed { .. } => None,
        }
    }

    /// 失败时借用诊断字符串，成功时返回 None；不产生 ABI diagnostics 借用。
    pub fn failure(&self) -> Option<&str> {
        match &self.outcome {
            ZrRuntimeOperationOutcomeV1::Succeeded { .. } => None,
            ZrRuntimeOperationOutcomeV1::Failed { error } => Some(error),
        }
    }
}

/// 向原会话提交 V1 JSON 请求的同步入口。
///
/// # 参数与返回值
/// 参数依次是会话、请求字节借用和可写句柄位置；当前 Runtime 仅在接纳成功后写非零句柄。
/// 返回 status；失败时不得将旧输出解释为本次提交结果。
/// # 约束与生命周期
/// 调用者保持 provider/session、请求内存和对齐可写输出位置在调用期间有效；输入是
/// 一段已初始化、可读且不并发改变的完整区域，形状和预算检查不能证明地址可读。
/// # 参考
/// [`ZrRuntimeOperationSubmitRequestV1`]、`submit_operation_ffi`、`RuntimeSession::submit_operation`。
/// 提交、轮询和收取结果共用会话归属；收取返回的拥有者缓冲区须由同一运行时释放。
pub type ZrRuntimeSubmitOperationFnV1 = unsafe extern "C" fn(
    ZrRuntimeSessionHandle,
    ZrByteSlice,
    *mut ZrRuntimeOperationHandle,
) -> ZrStatus;
/// 读取操作固定状态的同步入口，不分配 JSON，也不调度工作。
///
/// # 参数与返回值
/// 参数依次是原会话、操作句柄和对齐可写的完整状态位置；可提供未初始化存储，
/// 当前 Runtime 在成功时用 ptr::write 初始化。调用方先确认 status 成功，再读字段。
/// # 约束与生命周期
/// provider/session 与函数指针须持续存活；非零句柄不代替服务归属查询。
/// Host 随后核对 V2、reserved、句柄及已知阶段/原因码。
/// # 参考
/// [`ZrRuntimeOperationStatusV2`]、`poll_operation_ffi`、`RuntimeSession::poll_operation`。
pub type ZrRuntimePollOperationFnV2 = unsafe extern "C" fn(
    ZrRuntimeSessionHandle,
    ZrRuntimeOperationHandle,
    *mut ZrRuntimeOperationStatusV2,
) -> ZrStatus;
/// 消费原会话操作结果并交出 Runtime-owned JSON allocation 的同步入口。
///
/// # 参数与返回值
/// 参数依次是会话、操作句柄和对齐可写结果位置；当前 Host 先置空，成功后接收
/// [`ZrOwnedResultV2`]。仅 Completed/Failed 且结果仍在位时可收获，重复、取消、过期返回错误。
/// # 约束与生命周期
/// 非空输出取得一次释放权，须在同 provider/session 存活期间经 release_allocation 归还；
/// 解码、版本或句柄核对失败仍须清理。登记成功才提交服务收获，失败回滚收获标志。
/// # 参考
/// [`ZrRuntimeOperationResultV1`]、`harvest_operation_ffi`、`RuntimeOwnedOutputReleaser`。
pub type ZrRuntimeHarvestOperationFnV2 = unsafe extern "C" fn(
    ZrRuntimeSessionHandle,
    ZrRuntimeOperationHandle,
    *mut ZrOwnedResultV2,
) -> ZrStatus;
