//! Bounded JSON decoding for foreign runtime payloads.
//!
//! 本模块先用无业务 DTO 分配的 Serde visitor 扫描 JSON 图，再用受截止时间限制的 reader 进行
//! 类型化反序列化，最后运行调用方的条目计数校验。profile 专属上下文跟踪 Interface DTO 的已知
//! 集合；未知字段仍计入 JSON 图上限，但不冒充这些类型化集合。

use std::io::{self, BufReader, Read};
use std::time::{Duration, Instant};

use serde::de::{
    Deserialize, DeserializeOwned, DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor,
};

use super::{
    budget::RuntimeForeignOutputPreflight, RuntimeForeignOutputBudget, RuntimeForeignOutputError,
    RUNTIME_FOREIGN_OUTPUT_JSON_MAX_NESTING_DEPTH,
};

const DECODE_READER_CHUNK_BYTES: usize = 4 * 1024;

/// 在字节、嵌套、类型化条目及时限约束内解码 JSON。
///
/// ## 使用时机与生命周期
///
/// 由 `RuntimeForeignOutputState::decode_json` 在检查 ABI 输出元数据和字节上限之后调用。
/// 输入切片必须在整个同步调用期间保持有效；其 owner 在该方法返回后才执行 ABI release。
///
/// ## 参数
///
/// `bytes` 是输出的只读字节，`budget` 选择共享上限，`operation` 用于错误诊断，`validate`
/// 在类型化反序列化成功后按具体 DTO 计算条目数。
///
/// ## 返回值
///
/// 返回解码结果和从预检开始计量的总耗时。任何预检、类型化解析、业务计数或时间检查失败都
/// 返回协议违规；成功时由外层状态在释放 ABI 分配后再决定是否接受结果。
pub(super) fn decode_bounded_json<T, E>(
    bytes: &[u8],
    budget: RuntimeForeignOutputBudget,
    operation: &'static str,
    validate: impl FnOnce(&T) -> Result<usize, E>,
) -> (Result<T, RuntimeForeignOutputError>, Duration)
where
    T: DeserializeOwned,
    E: std::fmt::Display,
{
    let decode_started = Instant::now();
    let Some(deadline) = decode_started.checked_add(budget.max_decode_time) else {
        return (
            Err(RuntimeForeignOutputError::protocol_violation(format!(
                "{operation} decode deadline exceeds the host clock range"
            ))),
            decode_started.elapsed(),
        );
    };
    // The interface item limit counts typed rows/deliveries, while the JSON graph also contains
    // envelopes and arbitrary payload values. Bound the allocation-free syntax pass by the wire
    // ceiling, then apply the exact typed item policy below.
    // 预检不构造业务 DTO；先阻止过深/过多 JSON 与已知 profile 集合进入类型化分配阶段。
    let json_value_limit = budget.max_encoded_bytes.saturating_add(1);
    if let Err(error) = preflight_json_graph(bytes, json_value_limit, budget, deadline, operation) {
        return (Err(error), decode_started.elapsed());
    }
    let preflight_time = decode_started.elapsed();
    if let Err(error) = budget.validate_decode_duration(preflight_time, operation) {
        return (Err(error), preflight_time);
    }
    let mut timed_out = false;
    // 使用同一绝对截止时间，避免预检和第二遍解析各自重新获得完整时限。
    let decoded = {
        let reader = DeadlineReader::new(bytes, deadline, &mut timed_out);
        serde_json::from_reader::<_, T>(BufReader::with_capacity(DECODE_READER_CHUNK_BYTES, reader))
    };
    let decoded = if timed_out {
        Err(RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} exceeded its decode time budget while parsing: maximum is {} microseconds",
            budget.max_decode_time.as_micros()
        )))
    } else {
        decoded.map_err(|error| {
            RuntimeForeignOutputError::protocol_violation(format!(
                "{operation} failed bounded JSON decode (maximum nesting depth {RUNTIME_FOREIGN_OUTPUT_JSON_MAX_NESTING_DEPTH}): {error}"
            ))
        })
    };
    let decoded = decoded.and_then(|decoded| {
        // 类型化 DTO 才能精确计数的集合在此验证；预检只提前限制会先触发分配的已知路径。
        let item_count = validate(&decoded)
            .map_err(|error| RuntimeForeignOutputError::protocol_violation(error.to_string()))?;
        budget.validate_decode_duration(decode_started.elapsed(), operation)?;
        budget.validate_item_count(item_count, operation)?;
        budget.validate_decode_duration(decode_started.elapsed(), operation)?;
        Ok(decoded)
    });
    let decode_time = decode_started.elapsed();
    let decoded = decoded.and_then(|decoded| {
        budget.validate_decode_duration(decode_time, operation)?;
        Ok(decoded)
    });
    (decoded, decode_time)
}

/// 在分配业务 DTO 前校验 JSON 语法图、嵌套深度和可识别的 profile 集合。
///
/// 遍历使用 `IgnoredAny` 与轻量 visitor，不保留 JSON 树；`max_json_values` 防止语法图节点过多，
/// `deadline` 与之后的类型化解析共用，`operation` 标识失败来源。
fn preflight_json_graph(
    bytes: &[u8],
    max_json_values: usize,
    budget: RuntimeForeignOutputBudget,
    deadline: Instant,
    operation: &'static str,
) -> Result<(), RuntimeForeignOutputError> {
    let mut timed_out = false;
    let profile_files_limit = match budget.preflight {
        RuntimeForeignOutputPreflight::None => None,
        RuntimeForeignOutputPreflight::ProfileFiles => Some(budget.max_items.saturating_sub(1)),
    };
    let profile_items_limit = profile_files_limit.map(|_| budget.max_items);
    let root_context = if profile_files_limit.is_some() {
        JsonItemContext::ProfileRoot
    } else {
        JsonItemContext::General
    };
    // counter 在遍历中逐节点累加；命中任一超限会让 Serde 停止，随后返回记录的具体计数。
    let mut counter =
        JsonItemCounter::new(max_json_values, profile_files_limit, profile_items_limit);
    let reader = DeadlineReader::new(bytes, deadline, &mut timed_out);
    let mut deserializer = serde_json::Deserializer::from_reader(BufReader::with_capacity(
        DECODE_READER_CHUNK_BYTES,
        reader,
    ));
    let result = JsonItemSeed {
        counter: &mut counter,
        context: root_context,
        profile_item: None,
    }
    .deserialize(&mut deserializer)
    .and_then(|()| deserializer.end());
    if timed_out {
        return Err(RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} exceeded its decode time budget during item preflight"
        )));
    }
    if let Some(observed) = counter.overflow_observed {
        return Err(RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} returned {observed} JSON values; syntax-graph maximum is {max_json_values}"
        )));
    }
    if let Some(observed) = counter.profile_files_overflow_observed {
        let maximum = counter.profile_files_limit.unwrap_or_default();
        return Err(RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} returned {observed} profile files; maximum is {maximum}"
        )));
    }
    if let Some(observed) = counter.profile_items_overflow_observed {
        let maximum = counter.profile_items_limit.unwrap_or_default();
        return Err(RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} returned {observed} profile items; maximum is {maximum}"
        )));
    }
    result.map_err(|error| {
        RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} failed JSON item preflight (maximum nesting depth {RUNTIME_FOREIGN_OUTPUT_JSON_MAX_NESTING_DEPTH}): {error}"
        ))
    })?;
    if let Some(maximum) = counter.profile_items_limit {
        if counter.profile_items_observed > maximum {
            return Err(RuntimeForeignOutputError::protocol_violation(format!(
                "{operation} returned {} profile items; maximum is {maximum}",
                counter.profile_items_observed
            )));
        }
    }
    Ok(())
}

/// JSON 语法节点以及 profile 已知类型化项目的有界累计数。
struct JsonItemCounter {
    /// 全部 JSON 值节点数。
    observed: usize,
    /// 全部 JSON 值节点上限。
    limit: usize,
    /// 首次越限时观察到的数量，用于错误消息。
    overflow_observed: Option<usize>,
    /// profile `files` 集合的元素累计数。
    profile_files_observed: usize,
    /// profile `files` 集合的专属上限。
    profile_files_limit: Option<usize>,
    /// 首次越过文件上限的数量。
    profile_files_overflow_observed: Option<usize>,
    /// profile DTO 外壳及已知集合的累计数。
    profile_items_observed: usize,
    /// profile DTO 项数上限。
    profile_items_limit: Option<usize>,
    /// 首次越过 profile 项上限的数量。
    profile_items_overflow_observed: Option<usize>,
}

impl JsonItemCounter {
    /// 建立语法图计数器；启用 profile 策略时把响应外壳先计作一个类型化项。
    fn new(
        limit: usize,
        profile_files_limit: Option<usize>,
        profile_items_limit: Option<usize>,
    ) -> Self {
        Self {
            observed: 0,
            limit,
            overflow_observed: None,
            profile_files_observed: 0,
            profile_files_limit,
            profile_files_overflow_observed: None,
            // The response envelope is one typed item in item_count.rs.
            profile_items_observed: if profile_items_limit.is_some() { 1 } else { 0 },
            profile_items_limit,
            profile_items_overflow_observed: None,
        }
    }

    /// 记录一个 JSON 值节点，越限后返回 Serde 错误停止遍历。
    fn observe<E: serde::de::Error>(&mut self) -> Result<(), E> {
        self.observed = self.observed.saturating_add(1);
        if self.observed <= self.limit {
            return Ok(());
        }
        self.overflow_observed = Some(self.observed);
        Err(E::custom("JSON item limit exceeded"))
    }

    /// 同时记录 profile 文件数；无专属上限时只累计文件而不拒绝。
    fn observe_profile_file<E: serde::de::Error>(&mut self) -> Result<(), E> {
        self.profile_files_observed = self.profile_files_observed.saturating_add(1);
        if self
            .profile_files_limit
            .is_none_or(|limit| self.profile_files_observed <= limit)
        {
            return Ok(());
        }
        self.profile_files_overflow_observed = Some(self.profile_files_observed);
        Err(E::custom("profile file limit exceeded"))
    }

    /// 记录已知 profile DTO 外壳或集合元素。
    fn observe_profile_item<E: serde::de::Error>(&mut self) -> Result<(), E> {
        self.profile_items_observed = self.profile_items_observed.saturating_add(1);
        if self
            .profile_items_limit
            .is_none_or(|limit| self.profile_items_observed <= limit)
        {
            return Ok(());
        }
        self.profile_items_overflow_observed = Some(self.profile_items_observed);
        Err(E::custom("profile item limit exceeded"))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
/// JSON visitor 当前所在的 Interface profile DTO 路径。
///
/// `General` 不识别业务集合；其余变体只沿已知 profile 结构传播，以在解析前为对应集合计数。
enum JsonItemContext {
    /// 一般 JSON 值或未知字段。
    General,
    /// ProfileControlResponse 外壳。
    ProfileRoot,
    /// ProfileControlResponse.files 数组。
    ProfileFiles,
    /// ProfileSnapshot 结构。
    ProfileSnapshot,
    /// Runtime diagnostics 结构。
    ProfileRuntimeDiagnostics,
    /// Runtime diagnostic series 结构。
    ProfileDiagnosticSeries,
    /// HotspotReport 结构。
    ProfileHotspotReport,
    /// CounterHotspotReport 结构。
    ProfileCounterHotspotReport,
    /// UiHotspotReport 结构。
    ProfileUiHotspotReport,
    /// 需要按条目数计数的已知集合。
    ProfileTypedArray,
    /// Runtime diagnostics 中的 diagnostic series 数组。
    ProfileDiagnosticSeriesArray,
    /// module composition receipt 结构。
    ProfileCompositionReceipt,
    /// HotspotReport.hints 数组。
    ProfileHotspotHints,
}

#[derive(Clone, Copy)]
/// Profile 预检识别出的集合元素类别。
enum ProfileItemKind {
    /// ProfileControlResponse.files 中的一项，同时计入文件和总 profile 项。
    File,
    /// 其他已知类型化 profile 集合中的一项。
    CollectionItem,
}

/// 让 Serde 递归遍历一个 JSON 值而不生成其业务 DTO。
struct JsonItemSeed<'a> {
    /// 当前预检共享的可变计数器。
    counter: &'a mut JsonItemCounter,
    /// 当前值所在的已识别 DTO 路径。
    context: JsonItemContext,
    /// 该值是否是某个类型化集合的元素。
    profile_item: Option<ProfileItemKind>,
}

impl<'de> DeserializeSeed<'de> for JsonItemSeed<'_> {
    /// visitor 只计数，因而返回单位值而不保留 JSON 子树。
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match self.profile_item {
            Some(ProfileItemKind::File) => {
                self.counter.observe_profile_file::<D::Error>()?;
                self.counter.observe_profile_item::<D::Error>()?;
            }
            Some(ProfileItemKind::CollectionItem) => {
                self.counter.observe_profile_item::<D::Error>()?;
            }
            None => {}
        }
        self.counter.observe::<D::Error>()?;
        deserializer.deserialize_any(JsonItemVisitor {
            counter: self.counter,
            context: self.context,
        })
    }
}

/// 递归访问 JSON 结构并只保留共享计数器。
struct JsonItemVisitor<'a> {
    /// 接收每个 JSON 值节点和已知 DTO 集合元素的累计器。
    counter: &'a mut JsonItemCounter,
    /// 决定子节点是否进入 profile 专用集合路径。
    context: JsonItemContext,
}

impl<'de> Visitor<'de> for JsonItemVisitor<'_> {
    /// visitor 不构建值，只用于验证和累计数量。
    type Value = ();

    /// 给 Serde 的类型错误提供 JSON 值说明。
    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a JSON value")
    }

    /// 布尔值是已由 `JsonItemSeed` 计数的叶节点。
    fn visit_bool<E>(self, _value: bool) -> Result<Self::Value, E> {
        Ok(())
    }

    /// 有符号整数是已计数的 JSON 叶节点。
    fn visit_i64<E>(self, _value: i64) -> Result<Self::Value, E> {
        Ok(())
    }

    /// 无符号整数是已计数的 JSON 叶节点。
    fn visit_u64<E>(self, _value: u64) -> Result<Self::Value, E> {
        Ok(())
    }

    /// 浮点数是已计数的 JSON 叶节点。
    fn visit_f64<E>(self, _value: f64) -> Result<Self::Value, E> {
        Ok(())
    }

    /// 借用字符串是已计数的 JSON 叶节点，不复制或保留文本。
    fn visit_str<E>(self, _value: &str) -> Result<Self::Value, E> {
        Ok(())
    }

    /// 输入生命周期内的借用字符串仍按叶节点处理。
    fn visit_borrowed_str<E>(self, _value: &'de str) -> Result<Self::Value, E> {
        Ok(())
    }

    /// Serde 提供的拥有字符串按叶节点处理，visitor 不保留它。
    fn visit_string<E>(self, _value: String) -> Result<Self::Value, E> {
        Ok(())
    }

    /// JSON null/option-none 值在 seed 中已计数。
    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(())
    }

    /// 单位值形式为空 JSON 叶值。
    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(())
    }

    /// 解开 Some 后沿当前 DTO 路径继续计数内部 JSON 值。
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        JsonItemSeed {
            counter: self.counter,
            context: self.context,
            profile_item: None,
        }
        .deserialize(deserializer)
    }

    /// 解开 newtype 后沿当前 DTO 路径继续计数内部 JSON 值。
    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        JsonItemSeed {
            counter: self.counter,
            context: self.context,
            profile_item: None,
        }
        .deserialize(deserializer)
    }

    /// 按序号识别 profile positional DTO 字段，其余数组元素作为普通子值递归计数。
    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let profile_struct = self.context.is_profile_struct();
        if self.context == JsonItemContext::ProfileCompositionReceipt {
            self.counter.observe_profile_item::<A::Error>()?;
        }
        let (context, profile_item) = match self.context {
            JsonItemContext::ProfileFiles => {
                (JsonItemContext::General, Some(ProfileItemKind::File))
            }
            JsonItemContext::ProfileDiagnosticSeriesArray => (
                JsonItemContext::ProfileDiagnosticSeries,
                Some(ProfileItemKind::CollectionItem),
            ),
            JsonItemContext::ProfileTypedArray | JsonItemContext::ProfileHotspotHints => (
                JsonItemContext::General,
                Some(ProfileItemKind::CollectionItem),
            ),
            _ => (JsonItemContext::General, None),
        };
        let mut index = 0;
        while sequence
            .next_element_seed(JsonItemSeed {
                counter: self.counter,
                context: if profile_struct {
                    self.context.position_context(index)
                } else {
                    context
                },
                profile_item: if profile_struct { None } else { profile_item },
            })?
            .is_some()
        {
            index += 1;
        }
        Ok(())
    }

    /// 已知 profile 对象解码字段键以选择子路径；一般对象跳过键并递归扫描值。
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        if self.context == JsonItemContext::ProfileCompositionReceipt {
            self.counter.observe_profile_item::<A::Error>()?;
        }
        if self.context.is_profile_map() {
            while let Some(key) = map.next_key::<ProfileFieldKey>()? {
                map.next_value_seed(JsonItemSeed {
                    counter: self.counter,
                    context: self.context.field_context(key),
                    profile_item: None,
                })?;
            }
        } else {
            while map.next_key::<IgnoredAny>()?.is_some() {
                map.next_value_seed(JsonItemSeed {
                    counter: self.counter,
                    context: JsonItemContext::General,
                    profile_item: None,
                })?;
            }
        }
        Ok(())
    }
}

impl JsonItemContext {
    /// profile map 和 receipt 都按 DTO 字段位置传播 profile 路径。
    fn is_profile_struct(self) -> bool {
        self.is_profile_map() || self == Self::ProfileCompositionReceipt
    }

    /// 判断该 context 是否使用字段名映射而非普通 map 遍历。
    fn is_profile_map(self) -> bool {
        matches!(
            self,
            Self::ProfileRoot
                | Self::ProfileSnapshot
                | Self::ProfileRuntimeDiagnostics
                | Self::ProfileDiagnosticSeries
                | Self::ProfileHotspotReport
                | Self::ProfileCounterHotspotReport
                | Self::ProfileUiHotspotReport
        )
    }

    /// 将 Interface DTO positional 序列的字段序号映射为子 context。
    fn position_context(self, index: usize) -> Self {
        use JsonItemContext as Context;

        // Serde's sequence form uses declaration order in the Interface profiling DTOs.
        match (self, index) {
            (Context::ProfileRoot, 2) => Context::ProfileSnapshot,
            (Context::ProfileRoot, 3) => Context::ProfileRuntimeDiagnostics,
            (Context::ProfileRoot, 4) => Context::ProfileCompositionReceipt,
            (Context::ProfileRoot, 5) => Context::ProfileHotspotReport,
            (Context::ProfileRoot, 6) => Context::ProfileCounterHotspotReport,
            (Context::ProfileRoot, 7) => Context::ProfileUiHotspotReport,
            (Context::ProfileRoot, 9) => Context::ProfileFiles,
            (Context::ProfileSnapshot, 5..=8) => Context::ProfileTypedArray,
            (Context::ProfileRuntimeDiagnostics, 8) => Context::ProfileDiagnosticSeriesArray,
            (Context::ProfileRuntimeDiagnostics, 10) => Context::ProfileSnapshot,
            (Context::ProfileDiagnosticSeries, 2 | 7) => Context::ProfileTypedArray,
            (Context::ProfileHotspotReport, 3) => Context::ProfileTypedArray,
            (Context::ProfileHotspotReport, 4) => Context::ProfileHotspotHints,
            (Context::ProfileCounterHotspotReport, 3 | 4) => Context::ProfileTypedArray,
            (Context::ProfileUiHotspotReport, 3 | 4) => Context::ProfileTypedArray,
            _ => Context::General,
        }
    }

    /// 根据已知字段名映射到子 context，未覆盖字段回退到 General。
    fn field_context(self, field: ProfileFieldKey) -> Self {
        use JsonItemContext as Context;
        use ProfileFieldKey as Field;

        // Follow only schema-known paths; matching names in unknown fields stay general JSON.
        match (self, field) {
            (Context::ProfileRoot, Field::Files) => Context::ProfileFiles,
            (Context::ProfileRoot, Field::Snapshot) => Context::ProfileSnapshot,
            (Context::ProfileRoot, Field::RuntimeDiagnostics) => Context::ProfileRuntimeDiagnostics,
            (Context::ProfileRoot, Field::ModuleCompositionReceipt) => {
                Context::ProfileCompositionReceipt
            }
            (Context::ProfileRoot, Field::HotspotReport) => Context::ProfileHotspotReport,
            (Context::ProfileRoot, Field::CounterHotspotReport) => {
                Context::ProfileCounterHotspotReport
            }
            (Context::ProfileRoot, Field::UiHotspotReport) => Context::ProfileUiHotspotReport,
            (
                Context::ProfileSnapshot,
                Field::Frames | Field::Spans | Field::Counters | Field::RecorderRetention,
            ) => Context::ProfileTypedArray,
            (Context::ProfileRuntimeDiagnostics, Field::DiagnosticSeries) => {
                Context::ProfileDiagnosticSeriesArray
            }
            (Context::ProfileRuntimeDiagnostics, Field::Profile) => Context::ProfileSnapshot,
            (Context::ProfileDiagnosticSeries, Field::SubsystemTags | Field::History) => {
                Context::ProfileTypedArray
            }
            (Context::ProfileHotspotReport, Field::Hotspots) => Context::ProfileTypedArray,
            (Context::ProfileHotspotReport, Field::Hints) => Context::ProfileHotspotHints,
            (Context::ProfileCounterHotspotReport, Field::Counters | Field::Hints) => {
                Context::ProfileTypedArray
            }
            (Context::ProfileUiHotspotReport, Field::Scenarios | Field::Alerts) => {
                Context::ProfileTypedArray
            }
            _ => Context::General,
        }
    }
}

#[derive(Clone, Copy)]
/// Profile 序列化对象内需识别的字段名；未知键使用 `Other`。
enum ProfileFieldKey {
    /// ProfileControlResponse.files 字段。
    Files,
    /// ProfileControlResponse.snapshot 字段。
    Snapshot,
    /// ProfileControlResponse.runtime_diagnostics 字段。
    RuntimeDiagnostics,
    /// ProfileControlResponse.module_composition_receipt 字段。
    ModuleCompositionReceipt,
    /// ProfileControlResponse.hotspot_report 字段。
    HotspotReport,
    /// ProfileControlResponse.counter_hotspot_report 字段。
    CounterHotspotReport,
    /// ProfileControlResponse.ui_hotspot_report 字段。
    UiHotspotReport,
    /// ProfileSnapshot.frames 集合。
    Frames,
    /// ProfileSnapshot.spans 集合。
    Spans,
    /// 快照或 CounterHotspotReport 中的 counters 集合。
    Counters,
    /// ProfileSnapshot.recorder_retention 集合。
    RecorderRetention,
    /// Runtime diagnostics 的 diagnostic_series 集合。
    DiagnosticSeries,
    /// Runtime diagnostics 的 profile 快照。
    Profile,
    /// DiagnosticSeries.subsystem_tags 集合。
    SubsystemTags,
    /// DiagnosticSeries.history 集合。
    History,
    /// HotspotReport.hotspots 集合。
    Hotspots,
    /// 报告中的 hints 映射字段。
    Hints,
    /// UI hotspot 报告的 scenarios 集合。
    Scenarios,
    /// UI hotspot 报告的 alerts 集合。
    Alerts,
    /// 任何不在预检 schema 映射中的字段。
    Other,
}

impl<'de> Deserialize<'de> for ProfileFieldKey {
    /// 将键解析委托给字段名 visitor。
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_identifier(ProfileFieldKeyVisitor)
    }
}

/// 将 profile map 键转换为已知 DTO 字段枚举。
struct ProfileFieldKeyVisitor;

impl Visitor<'_> for ProfileFieldKeyVisitor {
    /// visitor 输出用于选择嵌套路径的枚举值。
    type Value = ProfileFieldKey;

    /// 提供字段标识符解析失败时使用的 Serde 说明。
    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a profile response collection field")
    }

    /// 将序列化字段拼写映射到已知 profile DTO 字段，未知拼写归为 `Other`。
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(match value {
            "files" => ProfileFieldKey::Files,
            "snapshot" => ProfileFieldKey::Snapshot,
            "runtime_diagnostics" => ProfileFieldKey::RuntimeDiagnostics,
            "module_composition_receipt" => ProfileFieldKey::ModuleCompositionReceipt,
            "hotspot_report" => ProfileFieldKey::HotspotReport,
            "counter_hotspot_report" => ProfileFieldKey::CounterHotspotReport,
            "ui_hotspot_report" => ProfileFieldKey::UiHotspotReport,
            "frames" => ProfileFieldKey::Frames,
            "spans" => ProfileFieldKey::Spans,
            "counters" => ProfileFieldKey::Counters,
            "recorder_retention" => ProfileFieldKey::RecorderRetention,
            "diagnostic_series" => ProfileFieldKey::DiagnosticSeries,
            "profile" => ProfileFieldKey::Profile,
            "subsystem_tags" => ProfileFieldKey::SubsystemTags,
            "history" => ProfileFieldKey::History,
            "hotspots" => ProfileFieldKey::Hotspots,
            "hints" => ProfileFieldKey::Hints,
            "scenarios" => ProfileFieldKey::Scenarios,
            "alerts" => ProfileFieldKey::Alerts,
            _ => ProfileFieldKey::Other,
        })
    }

    /// 借用字段名复用同一映射，不延长输入字节生命周期。
    fn visit_borrowed_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
        self.visit_str(value)
    }
}

struct DeadlineReader<'a> {
    /// 尚未交给 Serde 的原始输出片段。
    remaining: &'a [u8],
    /// 预检和正式解析共享的绝对截止时刻。
    deadline: Instant,
    /// 供外层区分截止超时与普通 JSON 语法错误。
    timed_out: &'a mut bool,
}

impl<'a> DeadlineReader<'a> {
    /// 将原始字节、截止时刻和超时记录位绑定为一个同步 reader。
    fn new(remaining: &'a [u8], deadline: Instant, timed_out: &'a mut bool) -> Self {
        Self {
            remaining,
            deadline,
            timed_out,
        }
    }
}

impl Read for DeadlineReader<'_> {
    /// 按最多一个固定块提供输入；截止后先标记超时再返回 `TimedOut`。
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if self.remaining.is_empty() || output.is_empty() {
            return Ok(0);
        }
        if Instant::now() >= self.deadline {
            *self.timed_out = true;
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "foreign output decode deadline exceeded",
            ));
        }
        let count = output
            .len()
            .min(self.remaining.len())
            .min(DECODE_READER_CHUNK_BYTES);
        output[..count].copy_from_slice(&self.remaining[..count]);
        self.remaining = &self.remaining[count..];
        Ok(count)
    }
}
