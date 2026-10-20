//! 通过 serde 回调写出排序、缩进的 JSON；数组直接输出，对象值先暂存再按键输出。
//! 每次顶层调用共享一份输出/资源计费状态和暂存尝试，嵌套序列化只重借用这些状态。
//! schema 封套由上层构造；本层不提供 sink 回滚或文件原子发布。

use std::collections::BTreeMap;
use std::fmt;
use std::io::Write;

use serde::ser::{
    SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant,
};
use serde::{Serialize, Serializer};

#[path = "canonical_writer/json_string.rs"]
mod json_string;
#[path = "canonical_writer/output.rs"]
mod output;

use self::json_string::{write_json_display, write_json_string, write_json_string_preaccounted};
use self::output::OutputBudget;
pub(super) use self::output::{CountingWriter, COPY_BUFFER_BYTES};
#[cfg(test)]
pub(in crate::serialization) use self::output::{
    MAX_CANONICAL_NESTING_DEPTH, MAX_CANONICAL_OBJECT_ENTRIES,
};
use super::super::write_error::CanonicalTextWriteError;
use super::super::SerializationBudget;
use super::canonical_map_key::{CanonicalMapKey, CanonicalMapKeySerializer};
use super::canonical_spool::TempSpool;
use super::wire::MAX_TEXT_DOCUMENT_BYTES;

/// serde_json 原始 JSON 包装的私有类型标记；本序列化器与键转换器据此拒绝绕过规范化。
pub(in crate::serialization) const SERDE_JSON_RAW_VALUE_TOKEN: &str =
    "$serde_json::private::RawValue";

/// Serializes canonical JSON directly to `sink` and appends exactly one newline.
///
/// Arrays are emitted directly. Object values are spooled while their keys are
/// collected, allowing lexical key ordering and duplicate-key replacement
/// without retaining a second complete document in memory.
///
/// 借用输入写出规范文本，供版本化文档包装调用。
///
/// # 参数
/// - `value`：本次通过 serde 遍历的值；本层不接管其所有权或添加封套。
/// - `sink`：独占借用的字节接收端，缓冲与持久化策略由调用方决定。
///
/// # 返回与错误
/// 成功返回包含末尾换行的字节数，并已请求 sink 刷新；使用版本化文本的固定输出上限。
/// 编码、资源或 I/O 错误会中止遍历，sink 可能已有前缀，不回滚已写字节。
///
/// # 生命周期与参考
/// 借用仅持续到本次调用返回；值暂存由本次尝试持有，清理语义见 `TempSpool`。
/// 自定义输出上限见 [`write_canonical_text_with_budget`]。
pub(in crate::serialization) fn write_canonical_text<T, W>(
    value: &T,
    sink: &mut W,
) -> Result<usize, CanonicalTextWriteError>
where
    T: ?Sized + Serialize,
    W: Write + ?Sized,
{
    write_canonical_text_with_limit(value, sink, MAX_TEXT_DOCUMENT_BYTES)
}

/// Streams canonical JSON under a caller-owned archive budget.
///
/// # 参数与返回
/// `value` 和 `sink` 的借用规则同 [`write_canonical_text`]；`budget` 指定含换行的输出字节上限。
/// 成功返回已写字节数，不额外套用版本化文本的固定字节上限。
///
/// # 约束
/// 深度、单对象键数、暂存工作量及文件数仍有独立上限；此值不是总内存或总 I/O 预算。
/// 失败可留下 sink 前缀；本层不校验领域 schema 或提供原子发布。
pub(in crate::serialization) fn write_canonical_text_with_budget<T, W>(
    value: &T,
    sink: &mut W,
    budget: SerializationBudget,
) -> Result<usize, CanonicalTextWriteError>
where
    T: ?Sized + Serialize,
    W: Write + ?Sized,
{
    write_canonical_text_with_limit(value, sink, budget.max_output_bytes())
}

/// 建立本次共享预算并从根深度 0 开始遍历；成功写完换行及刷新后才返回计费字节数。
/// 错误直接向上传播，当前 sink 和计费状态不作事务撤销。
fn write_canonical_text_with_limit<T, W>(
    value: &T,
    sink: &mut W,
    max_bytes: usize,
) -> Result<usize, CanonicalTextWriteError>
where
    T: ?Sized + Serialize,
    W: Write + ?Sized,
{
    let mut budget = OutputBudget::new(max_bytes);
    let mut output = CountingWriter::new(sink, &mut budget);
    {
        let mut serializer = CanonicalTextSerializer {
            output: &mut output,
            depth: 0,
        };
        value.serialize(&mut serializer)?;
    }
    output.write_counted(b"\n")?;
    output.flush()?;
    let bytes = output.budget.bytes;
    Ok(bytes)
}

#[cfg(test)]
/// 测试使用任意输出字节上限复用真实 writer；不改变生产资源上限或错误传播。
pub(in crate::serialization) fn write_canonical_text_with_limit_for_test<T, W>(
    value: &T,
    sink: &mut W,
    max_bytes: usize,
) -> Result<usize, CanonicalTextWriteError>
where
    T: ?Sized + Serialize,
    W: Write + ?Sized,
{
    write_canonical_text_with_limit(value, sink, max_bytes)
}

/// 一层 serde 遍历上下文；嵌套值重借用同一输出器和预算，不延长 sink 的外部借用期。
struct CanonicalTextSerializer<'output, 'sink, 'budget, W: Write + ?Sized> {
    /// 当前层独占的输出器借用；其内部仍分别借用 sink 和本次预算。
    output: &'output mut CountingWriter<'sink, 'budget, W>,
    /// 当前值的格式深度，根为 0；容器在进入子值时检查其目标深度。
    depth: usize,
}

impl<'output, 'sink, 'budget, W: Write + ?Sized>
    CanonicalTextSerializer<'output, 'sink, 'budget, W>
{
    /// 将目标深度转换为每层两个空格，并逐段计入输出预算。
    fn write_indent(&mut self, depth: usize) -> Result<(), CanonicalTextWriteError> {
        for _ in 0..depth {
            self.output.write_counted(b"  ")?;
        }
        Ok(())
    }

    /// 写入带引号及转义的字符串，实际输出字节由输出器计费。
    fn write_json_string(&mut self, value: &str) -> Result<(), CanonicalTextWriteError> {
        write_json_string(self.output, value)
    }

    /// 为已分派的数值等标量取得 JSON 拼写，再交由输出器计费写入。
    /// 浮点有限性与宽整数范围由调用此 helper 的 serde 分支先行检查。
    fn write_scalar<T>(&mut self, value: &T) -> Result<(), CanonicalTextWriteError>
    where
        T: ?Sized + Serialize,
    {
        let encoded = serde_json::to_string(value).map_err(|source| {
            CanonicalTextWriteError::PayloadEncode {
                reason: source.to_string(),
            }
        })?;
        self.output.write_counted(encoded.as_bytes())
    }
}

/// 数组/元组的流式状态；元素直接进入当前 sink，结束时消费状态并补齐右括号。
struct CanonicalArray<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized> {
    /// 本数组存续期间独占其父层上下文，元素通过短期重借用递归。
    serializer: &'serializer mut CanonicalTextSerializer<'output, 'sink, 'budget, W>,
    /// 尚未输出元素；控制首个换行与后续逗号分隔。
    first: bool,
}

impl<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized>
    CanonicalArray<'serializer, 'output, 'sink, 'budget, W>
{
    /// 先检查子值深度，再写分隔/缩进并序列化借用值；失败可能已输出本元素前缀。
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), CanonicalTextWriteError>
    where
        T: ?Sized + Serialize,
    {
        let depth = self.serializer.depth + 1;
        self.serializer.output.budget.ensure_nesting_depth(depth)?;
        if self.first {
            self.serializer.output.write_counted(b"\n")?;
            self.first = false;
        } else {
            self.serializer.output.write_counted(b",\n")?;
        }
        self.serializer.write_indent(depth)?;
        let output = &mut *self.serializer.output;
        let mut nested = CanonicalTextSerializer { output, depth };
        value.serialize(&mut nested)
    }

    /// 消费数组状态，非空时补换行和父层缩进；最终刷新由顶层调用负责。
    fn finish(self) -> Result<(), CanonicalTextWriteError> {
        if !self.first {
            self.serializer.output.write_counted(b"\n")?;
            self.serializer.write_indent(self.serializer.depth)?;
        }
        self.serializer.output.write_counted(b"]")
    }
}

/// 对象的排序状态：拥有规范键和各值的暂存，不在收集字段时输出对象正文。
/// 同键以后值覆盖前值；serde map 的键和值必须交替提交。
struct CanonicalObject<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized> {
    /// 本对象存续期间独占父层上下文；所有值暂存共用其预算与尝试。
    serializer: &'serializer mut CanonicalTextSerializer<'output, 'sink, 'budget, W>,
    /// 按规范键排序的已完成值；替换时释放旧值的输出计费。
    entries: BTreeMap<String, CanonicalObjectEntry>,
    /// map 已提交且尚待值的键；struct 字段入口直接成对处理，不使用此状态。
    pending_key: Option<CanonicalMapKey>,
}

/// 一个已序列化对象值的拥有者；随 entry 释放其暂存资源。
struct CanonicalObjectEntry {
    /// 复制输出时复用已计费内容，避免把同一值再次计入最终输出字节。
    value: TempSpool,
}

impl<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized>
    CanonicalObject<'serializer, 'output, 'sink, 'budget, W>
{
    /// 消费已准备的键并暂存借用值；`depth` 是子值深度。
    /// 重复键先移除旧值再开始新值，失败会终止本次写入，不恢复旧 entry。
    fn serialize_value_for_key<T>(
        &mut self,
        key: CanonicalMapKey,
        value: &T,
        depth: usize,
    ) -> Result<(), CanonicalTextWriteError>
    where
        T: ?Sized + Serialize,
    {
        // 重复键采用最后值；先退还旧值预算，避免替换操作被误计为两个输出字段。
        if let Some(previous) = self.entries.remove(key.value()) {
            self.serializer
                .output
                .budget
                .release(previous.value.accounted_bytes);
        }
        let (key, _) = key.into_parts();
        let spool = serialize_to_spool(self.serializer.output, depth, value)?;
        self.entries
            .insert(key, CanonicalObjectEntry { value: spool });
        Ok(())
    }

    /// 新键先检查本对象条目数，再预留带引号/转义的键字节；重复键保留原有键预算。
    fn prepare_key(
        &mut self,
        key: CanonicalMapKey,
    ) -> Result<CanonicalMapKey, CanonicalTextWriteError> {
        if !self.entries.contains_key(key.value()) {
            self.serializer
                .output
                .budget
                .ensure_object_entries(self.entries.len().saturating_add(1))?;
            self.serializer.output.budget.reserve(key.encoded_bytes())?;
        }
        Ok(key)
    }

    /// 消费收集结果并按键输出；若 map 仍有未配值的键，先返回编码协议错误。
    fn finish(self) -> Result<(), CanonicalTextWriteError> {
        if self.pending_key.is_some() {
            return Err(CanonicalTextWriteError::PayloadEncode {
                reason: "map ended without a value".to_string(),
            });
        }
        write_object(self.serializer.output, self.serializer.depth, self.entries)
    }
}

/// 外部标记的元组变体；先在本次尝试内暂存数组，再作为变体名对应的对象值输出。
struct CanonicalTupleVariant<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized> {
    /// 保留父层独占借用，暂存写入重借用同一预算。
    serializer: &'serializer mut CanonicalTextSerializer<'output, 'sink, 'budget, W>,
    /// serde 提供的静态变体名，结束时复制为拥有的对象键。
    variant: &'static str,
    /// 本变体数组的暂存；最终复制仍沿用已计费字节。
    spool: TempSpool,
    /// 数组是否尚无元素，用于选择分隔和结束缩进。
    first: bool,
}

impl<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized>
    CanonicalTupleVariant<'serializer, 'output, 'sink, 'budget, W>
{
    /// 在父深度加 2 的位置暂存字段：变体对象与内部数组各占一层；先校验深度。
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), CanonicalTextWriteError>
    where
        T: ?Sized + Serialize,
    {
        let depth = self.serializer.depth + 2;
        self.serializer.output.budget.ensure_nesting_depth(depth)?;
        {
            let mut output =
                CountingWriter::new_spool(&mut self.spool, &mut *self.serializer.output.budget);
            if self.first {
                output.write_counted(b"\n")?;
                self.first = false;
            } else {
                output.write_counted(b",\n")?;
            }
            write_indent(&mut output, depth)?;
            let mut nested = CanonicalTextSerializer {
                output: &mut output,
                depth,
            };
            value.serialize(&mut nested)?;
            output.flush()?;
        }
        Ok(())
    }

    /// 消费变体，先完成暂存数组，再输出单键外部标记对象；错误直接向上传播。
    fn finish(mut self) -> Result<(), CanonicalTextWriteError> {
        {
            let mut output =
                CountingWriter::new_spool(&mut self.spool, &mut *self.serializer.output.budget);
            if !self.first {
                output.write_counted(b"\n")?;
                write_indent(&mut output, self.serializer.depth + 1)?;
            }
            output.write_counted(b"]")?;
            output.flush()?;
        }
        write_single_object(
            self.serializer.output,
            self.serializer.depth,
            self.variant.to_string(),
            self.spool,
        )
    }
}

/// 外部标记的结构体变体；内部字段排序后再暂存为变体名对应的对象值。
struct CanonicalStructVariant<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized> {
    /// 保留父层独占上下文；内部字段和外部包装共享本次计费。
    serializer: &'serializer mut CanonicalTextSerializer<'output, 'sink, 'budget, W>,
    /// 静态变体名，完成时转为外部对象拥有的键。
    variant: &'static str,
    /// 内部结构体字段的排序表，各值暂存由 entry 拥有。
    entries: BTreeMap<String, CanonicalObjectEntry>,
}

impl<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized>
    CanonicalStructVariant<'serializer, 'output, 'sink, 'budget, W>
{
    /// 预留唯一键并在父深度加 2 处暂存字段；同键只退还旧值的输出计费。
    fn serialize_field<T>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), CanonicalTextWriteError>
    where
        T: ?Sized + Serialize,
    {
        let key = CanonicalMapKey::from_str(key, self.serializer.output.budget.max_bytes())?;
        if !self.entries.contains_key(key.value()) {
            self.serializer
                .output
                .budget
                .ensure_object_entries(self.entries.len().saturating_add(1))?;
            self.serializer.output.budget.reserve(key.encoded_bytes())?;
        }
        if let Some(previous) = self.entries.remove(key.value()) {
            self.serializer
                .output
                .budget
                .release(previous.value.accounted_bytes);
        }
        let (key, _) = key.into_parts();
        let spool = serialize_to_spool(self.serializer.output, self.serializer.depth + 2, value)?;
        self.entries
            .insert(key, CanonicalObjectEntry { value: spool });
        Ok(())
    }

    /// 先将已排序的内部对象写入共享尝试的暂存，再消费它输出变体包装。
    /// 暂存间复制不重复计最终输出字节，但仍累计暂存工作量。
    fn finish(self) -> Result<(), CanonicalTextWriteError> {
        let mut inner = TempSpool::new(self.serializer.output.budget.spool_attempt());
        {
            let mut output =
                CountingWriter::new_spool(&mut inner, &mut *self.serializer.output.budget);
            write_object(&mut output, self.serializer.depth + 1, self.entries)?;
            output.flush()?;
        }
        write_single_object(
            self.serializer.output,
            self.serializer.depth,
            self.variant.to_string(),
            inner,
        )
    }
}

/// 将 serde 值域映射到规范 JSON；容器状态均借用当前层，结束时消费各适配器。
/// 非有限浮点、超出 64 位对应范围的宽整数和原始 JSON 私有包装在相应分支拒绝。
impl<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized> Serializer
    for &'serializer mut CanonicalTextSerializer<'output, 'sink, 'budget, W>
{
    type Ok = ();
    type Error = CanonicalTextWriteError;
    type SerializeSeq = CanonicalArray<'serializer, 'output, 'sink, 'budget, W>;
    type SerializeTuple = CanonicalArray<'serializer, 'output, 'sink, 'budget, W>;
    type SerializeTupleStruct = CanonicalArray<'serializer, 'output, 'sink, 'budget, W>;
    type SerializeTupleVariant = CanonicalTupleVariant<'serializer, 'output, 'sink, 'budget, W>;
    type SerializeMap = CanonicalObject<'serializer, 'output, 'sink, 'budget, W>;
    type SerializeStruct = CanonicalObject<'serializer, 'output, 'sink, 'budget, W>;
    type SerializeStructVariant = CanonicalStructVariant<'serializer, 'output, 'sink, 'budget, W>;

    fn serialize_bool(self, value: bool) -> Result<Self::Ok, Self::Error> {
        self.output
            .write_counted(if value { b"true" } else { b"false" })
    }

    fn serialize_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
        self.write_scalar(&value)
    }
    fn serialize_i16(self, value: i16) -> Result<Self::Ok, Self::Error> {
        self.write_scalar(&value)
    }
    fn serialize_i32(self, value: i32) -> Result<Self::Ok, Self::Error> {
        self.write_scalar(&value)
    }
    fn serialize_i64(self, value: i64) -> Result<Self::Ok, Self::Error> {
        self.write_scalar(&value)
    }
    fn serialize_i128(self, value: i128) -> Result<Self::Ok, Self::Error> {
        i64::try_from(value)
            .map_err(|_| CanonicalTextWriteError::PayloadEncode {
                reason: "number out of range".to_string(),
            })
            .and_then(|value| self.write_scalar(&value))
    }
    fn serialize_u8(self, value: u8) -> Result<Self::Ok, Self::Error> {
        self.write_scalar(&value)
    }
    fn serialize_u16(self, value: u16) -> Result<Self::Ok, Self::Error> {
        self.write_scalar(&value)
    }
    fn serialize_u32(self, value: u32) -> Result<Self::Ok, Self::Error> {
        self.write_scalar(&value)
    }
    fn serialize_u64(self, value: u64) -> Result<Self::Ok, Self::Error> {
        self.write_scalar(&value)
    }
    fn serialize_u128(self, value: u128) -> Result<Self::Ok, Self::Error> {
        u64::try_from(value)
            .map_err(|_| CanonicalTextWriteError::PayloadEncode {
                reason: "number out of range".to_string(),
            })
            .and_then(|value| self.write_scalar(&value))
    }

    fn serialize_f32(self, value: f32) -> Result<Self::Ok, Self::Error> {
        if value.is_finite() {
            self.write_scalar(&value)
        } else {
            Err(CanonicalTextWriteError::NonFinite {
                value: value as f64,
            })
        }
    }

    fn serialize_f64(self, value: f64) -> Result<Self::Ok, Self::Error> {
        if value.is_finite() {
            self.write_scalar(&value)
        } else {
            Err(CanonicalTextWriteError::NonFinite { value })
        }
    }

    fn serialize_char(self, value: char) -> Result<Self::Ok, Self::Error> {
        self.write_json_string(&value.to_string())
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
        self.write_json_string(value)
    }

    /// 字节切片按元素序列输出为 JSON 数组，不使用编码后的单个字符串。
    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok, Self::Error> {
        let mut array = self.serialize_seq(Some(value.len()))?;
        for byte in value {
            array.serialize_element(byte)?;
        }
        SerializeSeq::end(array)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.output.write_counted(b"null")
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_none()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_none()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.write_json_string(variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        // 此私有标记可能绕过正常节点遍历；其余 newtype 继续沿用当前深度序列化内部值。
        if name == SERDE_JSON_RAW_VALUE_TOKEN {
            return Err(CanonicalTextWriteError::PayloadValidation {
                reason: "serde_json RawValue is outside the canonical versioned payload domain"
                    .to_string(),
            });
        }
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        // 外部标签增加一层；先暂存借用的内部值，再写拥有键的单字段对象。
        let spool = serialize_to_spool(self.output, self.depth + 1, value)?;
        write_single_object(self.output, self.depth, variant.to_string(), spool)
    }

    /// 长度提示不用于预分配或准入，元素逐个检查深度并直接写入。
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.output.write_counted(b"[")?;
        Ok(CanonicalArray {
            serializer: self,
            first: true,
        })
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(None)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(None)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        let mut spool = TempSpool::new(self.output.budget.spool_attempt());
        {
            let mut output = CountingWriter::new_spool(&mut spool, &mut *self.output.budget);
            output.write_counted(b"[")?;
            output.flush()?;
        }
        Ok(CanonicalTupleVariant {
            serializer: self,
            variant,
            spool,
            first: true,
        })
    }

    /// 忽略长度提示；唯一键的实际数量在逐键准备时检查。
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(CanonicalObject {
            serializer: self,
            entries: BTreeMap::new(),
            pending_key: None,
        })
    }

    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        if name == SERDE_JSON_RAW_VALUE_TOKEN {
            return Err(CanonicalTextWriteError::PayloadValidation {
                reason: "serde_json RawValue is outside the canonical versioned payload domain"
                    .to_string(),
            });
        }
        Ok(CanonicalObject {
            serializer: self,
            entries: BTreeMap::new(),
            pending_key: None,
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(CanonicalStructVariant {
            serializer: self,
            variant,
            entries: BTreeMap::new(),
        })
    }

    /// 将 Display 片段直接转义写入；编码或预算错误立即向上传播，不聚合完整显示文本。
    fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + fmt::Display,
    {
        write_json_display(self.output, value)
    }

    fn is_human_readable(&self) -> bool {
        true
    }
}

impl<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized> SerializeSeq
    for CanonicalArray<'serializer, 'output, 'sink, 'budget, W>
{
    type Ok = ();
    type Error = CanonicalTextWriteError;
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_element(value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish()
    }
}

impl<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized> SerializeTuple
    for CanonicalArray<'serializer, 'output, 'sink, 'budget, W>
{
    type Ok = ();
    type Error = CanonicalTextWriteError;
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_element(value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish()
    }
}

impl<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized> SerializeTupleStruct
    for CanonicalArray<'serializer, 'output, 'sink, 'budget, W>
{
    type Ok = ();
    type Error = CanonicalTextWriteError;
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_element(value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish()
    }
}

impl<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized> SerializeTupleVariant
    for CanonicalTupleVariant<'serializer, 'output, 'sink, 'budget, W>
{
    type Ok = ();
    type Error = CanonicalTextWriteError;
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_field(value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish()
    }
}

impl<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized> SerializeMap
    for CanonicalObject<'serializer, 'output, 'sink, 'budget, W>
{
    type Ok = ();
    type Error = CanonicalTextWriteError;
    /// 只允许一个待配值键；规范化后预留键预算，存入 pending 状态。
    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        if self.pending_key.is_some() {
            return Err(CanonicalTextWriteError::PayloadEncode {
                reason: "map key was not followed by a value".to_string(),
            });
        }
        let key = key.serialize(CanonicalMapKeySerializer::new(
            self.serializer.output.budget.max_bytes(),
        ))?;
        self.pending_key = Some(self.prepare_key(key)?);
        Ok(())
    }
    /// 取走此前提交的键并暂存值；没有 pending 键时返回编码错误。
    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let key =
            self.pending_key
                .take()
                .ok_or_else(|| CanonicalTextWriteError::PayloadEncode {
                    reason: "map value was emitted without a key".to_string(),
                })?;
        self.serialize_value_for_key(key, value, self.serializer.depth + 1)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish()
    }
}

impl<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized> SerializeStruct
    for CanonicalObject<'serializer, 'output, 'sink, 'budget, W>
{
    type Ok = ();
    type Error = CanonicalTextWriteError;
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let key = CanonicalMapKey::from_str(key, self.serializer.output.budget.max_bytes())?;
        let key = self.prepare_key(key)?;
        self.serialize_value_for_key(key, value, self.serializer.depth + 1)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish()
    }
}

impl<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized> SerializeStructVariant
    for CanonicalStructVariant<'serializer, 'output, 'sink, 'budget, W>
{
    type Ok = ();
    type Error = CanonicalTextWriteError;
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_field(key, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish()
    }
}

/// 将借用值序列化为本次尝试拥有的暂存，子值深度在遍历前检查。
///
/// # 参数与返回
/// `parent` 提供共享预算和尝试，`depth` 是该值深度，`value` 只在本次调用中借用。
/// 成功返回已完成写入的暂存；其计费增量用于以后替换同键值时退还最终输出预算。
///
/// # 生命周期与错误
/// 嵌套输出器先结束借用，再完成暂存写入；落盘暂存此时关闭写句柄，复制时重新读取。
/// 失败不撤销共享计费，返回途中按暂存所有权析构；清理错误的处理见 `TempSpool::drop`。
fn serialize_to_spool<T, W>(
    parent: &mut CountingWriter<'_, '_, W>,
    depth: usize,
    value: &T,
) -> Result<TempSpool, CanonicalTextWriteError>
where
    T: ?Sized + Serialize,
    W: Write + ?Sized,
{
    let before = parent.budget.bytes;
    parent.budget.ensure_nesting_depth(depth)?;
    let mut spool = TempSpool::new(parent.budget.spool_attempt());
    {
        let mut output = CountingWriter::new_spool(&mut spool, &mut *parent.budget);
        let mut serializer = CanonicalTextSerializer {
            output: &mut output,
            depth,
        };
        value.serialize(&mut serializer)?;
        output.flush()?;
    }
    spool.finish_write()?;
    spool.accounted_bytes = parent.budget.bytes.saturating_sub(before);
    Ok(spool)
}

/// 消费一个已计费的暂存值，预留拥有键的编码字节并输出单字段外部标记对象。
/// `depth` 表示外部对象深度；子值所在层仍须通过嵌套上限，复制值不再计最终输出字节。
fn write_single_object<W>(
    output: &mut CountingWriter<'_, '_, W>,
    depth: usize,
    key: String,
    value: TempSpool,
) -> Result<(), CanonicalTextWriteError>
where
    W: Write + ?Sized,
{
    output
        .budget
        .ensure_nesting_depth(depth.saturating_add(1))?;
    let key = CanonicalMapKey::from_string(key, output.budget.max_bytes())?;
    output.budget.reserve(key.encoded_bytes())?;
    let (key, _) = key.into_parts();
    let mut entries = BTreeMap::new();
    entries.insert(key, CanonicalObjectEntry { value });
    write_object(output, depth, entries)
}

/// 消费排序表并输出对象；键字节已在准备阶段计费，值内容已在序列化暂存时计费。
/// 此处只新增标点/缩进计费，按排序表顺序复制各值；错误可留下对象前缀。
fn write_object<W>(
    output: &mut CountingWriter<'_, '_, W>,
    depth: usize,
    mut entries: BTreeMap<String, CanonicalObjectEntry>,
) -> Result<(), CanonicalTextWriteError>
where
    W: Write + ?Sized,
{
    if entries.is_empty() {
        return output.write_counted(b"{}");
    }
    output.write_counted(b"{\n")?;
    let entry_count = entries.len();
    for (index, (key, entry)) in entries.iter_mut().enumerate() {
        write_indent(output, depth + 1)?;
        write_json_string_preaccounted(output, key)?;
        output.write_counted(b": ")?;
        entry.value.copy_to(output)?;
        if index + 1 != entry_count {
            output.write_counted(b",")?;
        }
        output.write_counted(b"\n")?;
    }
    write_indent(output, depth)?;
    output.write_counted(b"}")
}

/// 为指定深度写出每层两个空格；在暂存路径使用时也累计暂存工作量。
fn write_indent<W>(
    output: &mut CountingWriter<'_, '_, W>,
    depth: usize,
) -> Result<(), CanonicalTextWriteError>
where
    W: Write + ?Sized,
{
    for _ in 0..depth {
        output.write_counted(b"  ")?;
    }
    Ok(())
}
