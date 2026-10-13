//! 定义按会话订阅的插件事件 JSON 页、身份与 ABI 槽。
//!
//! Runtime 先准备待交付页，输出登记成功后才提交序列和队列消费；Host 有界解码后释放
//! 原分配，再交给 typed consumer。订阅句柄本身不持有会话或自动退订。
//! # 参考
//! [`ZrRuntimePluginEventDeliveryBatchV1`]、[`ZrRuntimeDrainPluginEventsFnV2`]、
//! `RuntimeDynamicSession::commit_plugin_event_output`、`unsubscribe_consumer`。

use serde::{Deserialize, Serialize};
use serde_json::value::{to_raw_value, RawValue};

use crate::{ZrByteSlice, ZrOwnedResultV2, ZrRuntimeSessionHandle, ZrStatus};

/// 单页投递条数上限，Runtime 准备页和 Host 解码后计数共同使用。
/// Maximum event deliveries returned by one V1 plugin-event drain page.
pub const ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1: usize = 64;

/// 单页完整 JSON 的字节上限，包含信封、描述符、payload 与积压字段。
/// Maximum JSON-encoded bytes returned by one V1 plugin-event drain page.
pub const ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1: usize = 256 * 1024;

/// 原 Runtime 会话发出的订阅身份，公开元组字段 `.0` 保存 wire ID。
///
/// # 生命周期与约束
/// 零为占位；非零与反序列化不证明订阅仍存活。跨传输使用时须同时保存发行会话身份，
/// 复制 ID 不会持有 Runtime 端订阅，正常退订成功后不得继续排出该订阅。
/// # 参考
/// [`ZrRuntimeSubscribePluginEventFnV1`]、[`ZrRuntimeUnsubscribePluginEventFnV1`]、`QualifiedSubscription`。
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ZrRuntimePluginEventSubscriptionHandle(pub u64);

impl ZrRuntimePluginEventSubscriptionHandle {
    /// 包装 raw ID，不验证原会话的订阅注册表。
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// 返回零占位，供 subscribe 成功状态与有效输出都确认前使用。
    pub const fn invalid() -> Self {
        Self(0)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }

    /// 仅检查 ID 非零，不能确认发行会话或订阅的远端存活状态。
    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }
}

/// 订阅注册事件和 payload schema 的 V1 JSON 请求，字段使用 camelCase。
///
/// # 使用时机与生命周期
/// Host 在同步 subscribe 中借用编码字节；当前 Runtime 复制并有界解码，随后交给
/// 世界的事件镜像注册表匹配 event_id/payload_schema，失败不交付新句柄。
/// # 参考
/// [`ZrRuntimeSubscribePluginEventFnV1`]、`RuntimeDynamicSession::subscribe_plugin_event`。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZrRuntimePluginEventSubscribeRequestV1 {
    /// 当前 subscribe 消费路径要求的 V1 版本。
    pub abi_version: u32,
    /// 世界事件镜像登记的事件键，构造器不校验或改写。
    pub event_id: String,
    /// 该事件的 payload schema 身份，后续 typed consumer 还会核对。
    pub payload_schema: String,
}

impl ZrRuntimePluginEventSubscribeRequestV1 {
    /// 取得事件键和 schema 字符串的所有权并保存版本，不进行远端注册。
    ///
    /// # 参数与返回值
    /// 三个输入组成拥有者请求；返回值尚未编码、预算检查或被 Runtime 接受。
    /// # 参考
    /// [`ZrRuntimeSubscribePluginEventFnV1`]、`RuntimeDynamicSession::subscribe_plugin_event`。
    pub fn new(
        abi_version: u32,
        event_id: impl Into<String>,
        payload_schema: impl Into<String>,
    ) -> Self {
        Self {
            abi_version,
            event_id: event_id.into(),
            payload_schema: payload_schema.into(),
        }
    }
}

/// 一条已从订阅页解码的投递，payload 保留为拥有者 RawValue。
///
/// # 使用时机与生命周期
/// 解码后的字段可以在 ABI 字节释放后保留；交给 consumer 前还须核对会话、订阅、
/// 事件、schema 与序列。RawValue 保留 JSON 文本，正常转发不需中间 Value 树。
/// # 参考
/// [`ZrRuntimePluginEventDeliveryBatchV1`]、`validate_delivery`、`SessionGateway::drain_plugin_events`。
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZrRuntimePluginEventDeliveryV1 {
    /// 当前动态 producer 填写原 session 的 raw ID，consumer 核对播放会话归属。
    pub play_session_id: u64,
    /// 本条投递所属订阅，必须与 drain 请求匹配。
    pub subscription: ZrRuntimePluginEventSubscriptionHandle,
    /// 订阅的事件键，consumer 与贡献清单核对。
    pub event_id: String,
    /// 订阅的 schema 身份，consumer 在类型化消费前核对。
    pub payload_schema: String,
    /// 同订阅递增序列；Runtime 提交已交付前缀后才推进，consumer 拒绝陈旧序列。
    pub sequence: u64,
    /// 拥有合法 raw JSON 文本；ABI allocation 释放不会撤销此 Box 的存活期。
    /// Owned JSON bytes forwarded to the typed consumer without an intermediate value tree.
    pub payload: Box<RawValue>,
}

impl ZrRuntimePluginEventDeliveryV1 {
    /// 从 Value 构造投递，并把 payload 序列化成拥有者 raw JSON。
    ///
    /// # 参数与返回值
    /// 会话、订阅、事件、schema、序列原样组成信封，不进行归属或序列校验；payload
    /// 由 to_raw_value 编码后移入返回值，此构造不使用运行期页预算。
    /// # 参考
    /// `plugin_event_mirror_contract_preserves_session_schema_sequence_and_payload`、[`ZrRuntimePluginEventDeliveryBatchV1`]。
    pub fn new(
        play_session_id: u64,
        subscription: ZrRuntimePluginEventSubscriptionHandle,
        event_id: impl Into<String>,
        payload_schema: impl Into<String>,
        sequence: u64,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            play_session_id,
            subscription,
            event_id: event_id.into(),
            payload_schema: payload_schema.into(),
            sequence,
            payload: to_raw_value(&payload)
                .expect("serde_json::Value always serializes to raw JSON payloads"),
        }
    }
}

// 相等比较先比身份字段，仅在身份一致时把两段 raw JSON 解析为 Value 后比较语义。
// 正常转发路径保留 RawValue；这个比较路径会额外构造两棵 Value 树。
impl PartialEq for ZrRuntimePluginEventDeliveryV1 {
    /// 比较五项信封字段与解析后的 payload 值，不要求原始 JSON 文本或空白一致。
    fn eq(&self, other: &Self) -> bool {
        self.play_session_id == other.play_session_id
            && self.subscription == other.subscription
            && self.event_id == other.event_id
            && self.payload_schema == other.payload_schema
            && self.sequence == other.sequence
            // TODO: [ZCA-OPS-001] RawValue 的合法 raw JSON 不等于 Value 的数值/递归限制可表示性；
            // 当前证据只覆盖普通 Value 产物和空白差异，公共直接构造是否满足重解析前提待确认。
            // 下一步由 Jenkins 核对当前 serde_json 版本/feature 与 Runtime 消费预算，并隔离验证
            // 非 Value 可表示数值和深嵌套 RawValue 的 eq 路径；目前无实际复现或完整生产触发链。
            && serde_json::from_str::<serde_json::Value>(self.payload.get())
                .expect("RawValue always contains valid JSON")
                == serde_json::from_str::<serde_json::Value>(other.payload.get())
                    .expect("RawValue always contains valid JSON")
    }
}

/// 单个订阅的有界 JSON 交付页，字段使用 camelCase。
///
/// # 生命周期与约束
/// 当前 Runtime 输出登记成功后提交已交付前缀；剩余计数包含仍保留的积压，Host
/// 解码后核对版本、数量和每条订阅。构造此值不执行预算检查，也不推进远端队列。
/// # 参考
/// [`ZrRuntimeDrainPluginEventsFnV2`]、`RuntimeDynamicSession::prepare_plugin_event_output`、
/// `SessionGateway::drain_plugin_events`。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// 一次订阅 drain 的有界页；剩余数量和最老待取年龄描述运行时仍保留的积压。
/// 网关须核对版本、页大小和每条投递的订阅句柄，再交给 typed consumer。
pub struct ZrRuntimePluginEventDeliveryBatchV1 {
    /// 页信封的 V1 版本，Host 验证后才接受投递。
    pub abi_version: u32,
    /// 本页拥有的投递序列，数量须满足单页上限。
    pub deliveries: Vec<ZrRuntimePluginEventDeliveryV1>,
    /// 提交本页前缀后 Runtime 仍保留的数量；默认零兼容缺少此字段的旧 JSON。
    /// Events still retained by the Runtime subscription after this page commits.
    #[serde(default)]
    pub remaining_deliveries: u32,
    /// 最老待取事件的单调经过毫秒数，默认零；不是 wall-clock 时间戳。
    /// Monotonic elapsed age of the oldest event still retained by the Runtime subscription.
    #[serde(default)]
    pub oldest_pending_age_millis: u64,
}

impl ZrRuntimePluginEventDeliveryBatchV1 {
    /// 构造零积压的拥有者页；abi_version 和 deliveries 原样保存，不限制页大小。
    /// # 参考
    /// [`Self::with_runtime_backlog`]、`plugin_event_mirror_contract_preserves_session_schema_sequence_and_payload`。
    pub fn new(abi_version: u32, deliveries: Vec<ZrRuntimePluginEventDeliveryV1>) -> Self {
        Self {
            abi_version,
            deliveries,
            remaining_deliveries: 0,
            oldest_pending_age_millis: 0,
        }
    }

    /// 消费 self 并写入 Runtime 报告的积压数量和毫秒年龄，返回更新后的页。
    ///
    /// 不重新读取远端队列，也不自行校验数值；用于编码已有的排出快照。
    /// # 参考
    /// [`Self::new`]、`encode_plugin_event_batch`。
    pub const fn with_runtime_backlog(
        mut self,
        remaining_deliveries: u32,
        oldest_pending_age_millis: u64,
    ) -> Self {
        self.remaining_deliveries = remaining_deliveries;
        self.oldest_pending_age_millis = oldest_pending_age_millis;
        self
    }

    /// 构造零投递、零积压的页值，版本原样保存。
    ///
    /// 当前 Runtime 编码辅助把零投递页表示为规范空 ABI 输出；此值本身不分配 ABI 存储。
    /// # 参考
    /// `encode_plugin_event_batch`、[`ZrRuntimeDrainPluginEventsFnV2`]。
    pub const fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            deliveries: Vec::new(),
            remaining_deliveries: 0,
            oldest_pending_age_millis: 0,
        }
    }
}

/// 在原会话创建事件镜像订阅的同步入口。
///
/// # 参数与返回值
/// 参数依次是会话、V1 JSON 字节借用和对齐可写句柄位置；返回 status，成功才写非零订阅。
/// # 约束与生命周期
/// provider/session 与回调在调用期存活；请求是已初始化、只读、完整有效区域，输出可写。
/// 当前 Runtime 复制输入并验证版本/预算和注册匹配；调用者不能把非零 ID 当作会话租约。
/// # 参考
/// [`ZrRuntimePluginEventSubscribeRequestV1`]、`subscribe_plugin_event_ffi`、`QualifiedSubscription`。
pub type ZrRuntimeSubscribePluginEventFnV1 = unsafe extern "C" fn(
    ZrRuntimeSessionHandle,
    ZrByteSlice,
    *mut ZrRuntimePluginEventSubscriptionHandle,
) -> ZrStatus;

/// 撤销原会话中的订阅；参数依次是会话和该会话发出的订阅句柄。
///
/// # 返回值与生命周期
/// 返回 status。当前 Runtime 拒绝零 ID、未知订阅或 output in-flight；世界真正断开成功
/// 才移除订阅。Host 在成功前仍须保留原端点供错误处理，不能仅丢弃 ID 声称退订完成。
/// # 参考
/// `unsubscribe_plugin_event_ffi`、`RuntimeDynamicSession::unsubscribe_plugin_event`、`unsubscribe_consumer`。
pub type ZrRuntimeUnsubscribePluginEventFnV1 = unsafe extern "C" fn(
    ZrRuntimeSessionHandle,
    ZrRuntimePluginEventSubscriptionHandle,
) -> ZrStatus;

/// 排出原会话订阅的一页投递，并转移 Runtime allocation 的一次释放权。
///
/// # 参数与返回值
/// 参数依次是会话、订阅和对齐可写输出位置；Host 先置空，成功才消费输出，
/// 无投递可返回规范空结果。失败时也按实际收到的输出检查并清理。
/// # 约束与生命周期
/// provider/session、回调和输出位置在调用期间有效；非空 JSON 在读取期保持可读不变，
/// 随后经同 provider/session 的 release_allocation 归还。解码/预算/身份失败不撤销释放责任。
/// # 参考
/// [`ZrRuntimePluginEventDeliveryBatchV1`]、`drain_plugin_events_ffi`、`RuntimeOwnedOutputReleaser`。
/// 返回拥有者 JSON 缓冲区；即使解码或验证失败，宿主也须通过同一会话的释放槽归还它。
pub type ZrRuntimeDrainPluginEventsFnV2 = unsafe extern "C" fn(
    ZrRuntimeSessionHandle,
    ZrRuntimePluginEventSubscriptionHandle,
    *mut ZrOwnedResultV2,
) -> ZrStatus;
