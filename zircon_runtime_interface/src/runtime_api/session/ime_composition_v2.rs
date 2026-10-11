//! IME 组合 V2 的有界字节编解码；头和分段均按小端逐字段读写。
//! 文本、游标和分段范围采用 UTF-8 字节偏移，原生 UTF-16 索引由宿主先转换。
//! 载荷复用固定事件信封的状态 10，发送与消费前须完成 V2 能力协商。
//! 当前窗口及组合代数的新旧检查由 Runtime 会话路由执行。

use crate::ui::dispatch::{
    UiImePreeditClause, UiImePreeditClauseError, UiImePreeditClauseKind, UiTextByteRange,
};

pub const ZR_RUNTIME_IME_COMPOSITION_V2_MAGIC: [u8; 4] = *b"ZIME";
pub const ZR_RUNTIME_IME_COMPOSITION_V2_SCHEMA: u16 = 2;
pub const ZR_RUNTIME_IME_COMPOSITION_V2_HEADER_BYTES: usize = 52;
pub const ZR_RUNTIME_IME_COMPOSITION_V2_CLAUSE_BYTES: usize = 12;
/// 单个组合载荷最多包含的样式分段数；完整载荷仍须满足事件总字节预算。
/// 分段表按每项 12 字节计入同一预算，不能因数量未超限而跳过总长度检查。
pub const ZR_RUNTIME_IME_COMPOSITION_V2_MAX_CLAUSES: usize = 4_096;

/// Empty clauses can mean either "the platform had no attributes" or "the platform reported
/// an empty composition". Keep that distinction in the negotiated envelope instead of guessing.
/// # 载荷含义
///
/// `Unavailable` 表示平台未提供可用属性，预编辑分段列表必须为空。
/// `Available` 表示平台确实提供了属性；列表仍可为空，因此不能只凭 `clauses.is_empty()` 推断属性不可用。
/// 提交和取消操作都要求 `Unavailable`；整数解析由 [`Self::try_from`] 拒绝未知标记。
/// 参见 [`ZrRuntimeImeCompositionV2`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ZrRuntimeImePreeditClauseAvailabilityV2 {
    Unavailable = 0,
    Available = 1,
}

impl TryFrom<u32> for ZrRuntimeImePreeditClauseAvailabilityV2 {
    type Error = ZrRuntimeImeCompositionV2Error;

    /// 将头部属性可用性整数解析为已知协议标记。
    ///
    /// # 参数与返回
    ///
    /// `value` 为已读出的小端 `u32` 字段；0 返回 `Unavailable`，1 返回 `Available`。
    /// 其他值返回 [`ZrRuntimeImeCompositionV2Error::UnknownClauseAvailability`]，不按非零值猜测可用性。
    /// 本转换不检查分段列表与操作的关系；完整约束由 [`ZrRuntimeImeCompositionV2::decode`] 校验。
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Unavailable),
            1 => Ok(Self::Available),
            _ => Err(ZrRuntimeImeCompositionV2Error::UnknownClauseAvailability),
        }
    }
}

/// 标识组合载荷是更新预编辑、提交文本还是取消组合。
///
/// # 操作与生命周期
///
/// `Preedit` 可携带文本、可选游标和样式分段；`Commit` 只携带提交文本。
/// `Cancel` 的文本必须为空；后两者都不得携带游标/分段，且属性标记为不可用。
/// 这些是载荷内容约束，实际是否属于当前活动组合由 Runtime 会话路由的代数门决定。
///
/// # 参考
///
/// 参见 [`ZrRuntimeImeCompositionV2`] 和 [`ZrRuntimeImeCompositionV2::decode`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum ZrRuntimeImeCompositionOperationV2 {
    Preedit = 1,
    Commit = 2,
    Cancel = 3,
}

impl TryFrom<u16> for ZrRuntimeImeCompositionOperationV2 {
    type Error = ZrRuntimeImeCompositionV2Error;

    /// 将头部操作整数解析为组合 V2 已定义的操作。
    ///
    /// # 参数与返回
    ///
    /// `value` 为已读出的小端 `u16` 字段；1、2、3 分别返回预编辑、提交、取消。
    /// 未知值返回 [`ZrRuntimeImeCompositionV2Error::UnknownOperation`]，不会回退为旧 IME 事件。
    /// 该转换只解析标签，文本/游标/分段关系由 [`ZrRuntimeImeCompositionV2::decode`] 检查。
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Preedit),
            2 => Ok(Self::Commit),
            3 => Ok(Self::Cancel),
            _ => Err(ZrRuntimeImeCompositionV2Error::UnknownOperation),
        }
    }
}

/// 宿主分配的窗口、焦点与组合生命周期代数，三者均须非零。
/// `Default` 的零值尚不能编码；事件新旧仍须由所属运行时会话的代数门核对。
/// # 使用时机与责任边界
///
/// 宿主为窗口、焦点和组合的生命周期分配代数，并随每个 V2 操作发送当前三元组。
/// 结构按值保存数字，不持有窗口或焦点资源；公开字段和零值 `Default` 本身没有验证保证。
/// 编解码只检查三个数字非零；Runtime 路由还要求窗口与当前绑定一致，并比较焦点/组合代数及活动状态。
/// 因此成功编码或解码不证明事件仍属于当前组合，窗口代数的分配仍属于宿主窗口生命周期。
///
/// # 参考
///
/// 参见 [`ZrRuntimeImeCompositionV2::encode`]、[`ZrRuntimeImeCompositionV2::decode`]。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ZrRuntimeImeCompositionContextV2 {
    pub window_generation: u64,
    pub focus_generation: u64,
    pub composition_generation: u64,
}

impl ZrRuntimeImeCompositionContextV2 {
    fn is_valid(self) -> bool {
        self.window_generation != 0
            && self.focus_generation != 0
            && self.composition_generation != 0
    }
}

/// 拥有组合文本及样式分段；游标和分段端点均为 `text` 内的 UTF-8 字节边界。
/// 预编辑分段按文本顺序排列且不重叠，属性不可用时分段为空。
/// 提交不携带游标或分段，取消还须清空文本；这两种终结操作的属性标记均为不可用。
/// # 使用时机与所有权
///
/// 宿主组装一次组合操作后调用 [`Self::encode`]；Runtime 从事件载荷调用 [`Self::decode`] 得到同一数据模型。
/// `text` 与 `clauses` 由该值拥有，解码结果不借用输入字节；公开字段允许构造尚未验证的值。
/// 游标和分段范围都相对于本次 `text`，不是周围文档偏移、字符序号或 UTF-16 单元索引；原生索引由宿主先转换。
///
/// # 约束与生命周期
///
/// 分段可以为空或留有文本空隙，但必须按文本顺序且不重叠，端点都位于合法 UTF-8 字节边界。
/// 总线格式为固定头、定长分段表和 UTF-8 正文，分段数量与完整载荷字节数分别有上限。
/// 编解码不执行能力协商或会话代数路由；调用方需在提交前完成协商，并由 Runtime 会话判断新旧事件。
///
/// # 参考
///
/// 参见 [`ZrRuntimeImeCompositionContextV2`]、[`ZrRuntimeImePreeditClauseAvailabilityV2`] 和
/// [`UiImePreeditClause::validate_preedit_payload`]。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZrRuntimeImeCompositionV2 {
    pub operation: ZrRuntimeImeCompositionOperationV2,
    pub context: ZrRuntimeImeCompositionContextV2,
    pub text: String,
    pub cursor_range: Option<UiTextByteRange>,
    pub clauses: Vec<UiImePreeditClause>,
    pub clause_availability: ZrRuntimeImePreeditClauseAvailabilityV2,
}

/// 描述组合 V2 字节格式或操作内容验证失败，供编码和解码调用方定位载荷问题。
///
/// # 错误范围
///
/// 格式错误包括超出总预算、短头、magic/schema/标签不支持、长度溢出或不精确、非法 UTF-8 及非零分段保留字段。
/// 语义错误包括上下文代数为零、分段数量超限、操作字段组合不合法，以及携带具体共享范围错误的 `InvalidPreeditRange`。
/// 错误按当前处理顺序返回首个失败条件，修正数据后可以重新编解码；两种入口都不会改变调用方的原值。
/// 该枚举不表示能力已协商、代数仍新或事件已派发成功；这些由 Runtime 路由及事件入口另行报告。
/// 内存分配失败也不是此枚举列举的可返回错误。
///
/// # 参考
///
/// 参见 [`ZrRuntimeImeCompositionV2::encode`]、[`ZrRuntimeImeCompositionV2::decode`] 和 [`UiImePreeditClauseError`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZrRuntimeImeCompositionV2Error {
    PayloadTooLarge,
    HeaderTruncated,
    BadMagic,
    UnsupportedSchema,
    UnknownOperation,
    InvalidContext,
    CountLimitExceeded,
    LengthOverflow,
    LengthMismatch,
    InvalidUtf8,
    UnknownClauseKind,
    UnknownClauseAvailability,
    NonZeroReservedClauseField,
    InvalidOperationPayload,
    InvalidPreeditRange(UiImePreeditClauseError),
}

impl ZrRuntimeImeCompositionV2 {
    /// 验证操作字段后返回完整的小端字节载荷，总长度遵守事件负载预算。
    /// 宿主借用这些字节派发事件时，须保持它们可读直到同步 `handle_event` 返回。
    /// # 输入约束
    ///
    /// 只读 `self`；先检查非零上下文和分段数量，再按操作检查文本、游标、分段及属性标记的关系。
    /// 预编辑范围由 [`UiImePreeditClause::validate_preedit_payload`] 检查 UTF-8 字节边界、顺序与不重叠。
    /// 原生 UTF-16 游标或分段索引必须由宿主在组装该值之前转换为本次文本的字节偏移。
    ///
    /// # 返回与错误
    ///
    /// 成功返回拥有全部字节的 `Vec<u8>`，布局为 52 字节头、每项 12 字节的分段表及 UTF-8 文本。
    /// 总长度为 `52 + 12 * clauses.len() + text.len()`，受 [`crate::ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1`] 约束。
    /// 非零上下文、分段数、操作关系和范围错误先于编码返回；整数转换或长度计算溢出返回 `LengthOverflow`，总预算超限返回 `PayloadTooLarge`。
    /// 函数成功仅证明生成了合法载荷，不证明完成协商、当前代数被路由接受或事件已送达。
    ///
    /// # 所有权与派发
    ///
    /// 游标为 `None` 时写入一对 `u32::MAX`；分段保留字段写零。
    /// 调用方拥有返回向量；若将其地址放入借用型事件载体，须保持该字节区域可读、内容稳定且地址不变，直到同步 `handle_event` 返回。
    /// Runtime 入口复制借用字节之后的后续处理不延长宿主原缓冲区的借用。
    /// 参见 [`Self::decode`] 和 [`ZrRuntimeImeCompositionV2Error`]。
    pub fn encode(&self) -> Result<Vec<u8>, ZrRuntimeImeCompositionV2Error> {
        self.validate()?;
        let text_len = u32::try_from(self.text.len())
            .map_err(|_| ZrRuntimeImeCompositionV2Error::LengthOverflow)?;
        let clause_count = u32::try_from(self.clauses.len())
            .map_err(|_| ZrRuntimeImeCompositionV2Error::LengthOverflow)?;
        let clause_bytes = self
            .clauses
            .len()
            .checked_mul(ZR_RUNTIME_IME_COMPOSITION_V2_CLAUSE_BYTES)
            .ok_or(ZrRuntimeImeCompositionV2Error::LengthOverflow)?;
        let total_bytes = ZR_RUNTIME_IME_COMPOSITION_V2_HEADER_BYTES
            .checked_add(clause_bytes)
            .and_then(|bytes| bytes.checked_add(self.text.len()))
            .ok_or(ZrRuntimeImeCompositionV2Error::LengthOverflow)?;
        if total_bytes > crate::ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1 {
            return Err(ZrRuntimeImeCompositionV2Error::PayloadTooLarge);
        }

        let mut output = Vec::with_capacity(total_bytes);
        output.extend_from_slice(&ZR_RUNTIME_IME_COMPOSITION_V2_MAGIC);
        output.extend_from_slice(&ZR_RUNTIME_IME_COMPOSITION_V2_SCHEMA.to_le_bytes());
        output.extend_from_slice(&(self.operation as u16).to_le_bytes());
        output.extend_from_slice(&self.context.window_generation.to_le_bytes());
        output.extend_from_slice(&self.context.focus_generation.to_le_bytes());
        output.extend_from_slice(&self.context.composition_generation.to_le_bytes());
        output.extend_from_slice(&text_len.to_le_bytes());
        let (cursor_start, cursor_end) = self
            .cursor_range
            .map(|range| (range.start_byte, range.end_byte))
            .unwrap_or((u32::MAX, u32::MAX));
        output.extend_from_slice(&cursor_start.to_le_bytes());
        output.extend_from_slice(&cursor_end.to_le_bytes());
        output.extend_from_slice(&clause_count.to_le_bytes());
        output.extend_from_slice(&(self.clause_availability as u32).to_le_bytes());
        for clause in &self.clauses {
            output.extend_from_slice(&clause.range.start_byte.to_le_bytes());
            output.extend_from_slice(&clause.range.end_byte.to_le_bytes());
            output.extend_from_slice(&clause_kind_to_wire(clause.kind).to_le_bytes());
            output.extend_from_slice(&0_u16.to_le_bytes());
        }
        output.extend_from_slice(self.text.as_bytes());
        debug_assert_eq!(output.len(), total_bytes);
        Ok(output)
    }

    /// 只接受头、分段与正文合成的精确长度，并在分配前核对总预算和分段数。
    /// 返回拥有文本的组合数据；能力协商与当前窗口、组合代数的检查由会话路由执行。
    /// # 参数
    ///
    /// `payload` 借用完整的一份组合 V2 字节载荷；调用期间须可读，不包含旧版文本事件或额外信封字节。
    /// 载荷按小端读字段；正文长度采用 UTF-8 字节数，游标和分段端点也采用正文内字节偏移。
    ///
    /// # 返回与错误
    ///
    /// 成功返回拥有文本和分段的值，不保留对 `payload` 的引用。
    /// 读取头前拒绝总预算超限或少于 52 字节；随后检查 magic、schema、操作/属性标签及非零上下文。
    /// 分段数最多为 [`ZR_RUNTIME_IME_COMPOSITION_V2_MAX_CLAUSES`]；计算 `52 + 12 * 分段数 + 正文字节数` 时检查溢出，
    /// 并要求结果精确等于 `payload.len()`，因此缺失字节或附加尾字节都会返回 `LengthMismatch`。
    /// 这些总量和计数检查完成后才为分段和正文分配；各分段的未知类型或非零保留字段会被拒绝。
    /// 一对 `u32::MAX` 表示无游标，只有一个端点为哨兵返回 `InvalidOperationPayload`。
    /// 正文不是 UTF-8 返回 `InvalidUtf8`；最后检查操作关系与预编辑字节范围，返回相应语义错误。
    ///
    /// # 调用时机与生命周期
    ///
    /// 本函数不修改输入、不执行能力协商，也不把任何事件写入 UI 或核心输入队列。
    /// Runtime 会话在调用前核对协商能力，成功解码后仍须核对当前窗口、焦点、组合代数和活动状态。
    /// 调用方可在返回后释放输入字节，返回值仍独立有效；参见 [`Self::encode`]、[`ZrRuntimeImeCompositionContextV2`] 和 [`ZrRuntimeImeCompositionV2Error`]。
    pub fn decode(payload: &[u8]) -> Result<Self, ZrRuntimeImeCompositionV2Error> {
        if payload.len() > crate::ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1 {
            return Err(ZrRuntimeImeCompositionV2Error::PayloadTooLarge);
        }
        if payload.len() < ZR_RUNTIME_IME_COMPOSITION_V2_HEADER_BYTES {
            return Err(ZrRuntimeImeCompositionV2Error::HeaderTruncated);
        }
        if payload.get(..4) != Some(&ZR_RUNTIME_IME_COMPOSITION_V2_MAGIC[..]) {
            return Err(ZrRuntimeImeCompositionV2Error::BadMagic);
        }
        if read_u16(payload, 4)? != ZR_RUNTIME_IME_COMPOSITION_V2_SCHEMA {
            return Err(ZrRuntimeImeCompositionV2Error::UnsupportedSchema);
        }
        let operation = ZrRuntimeImeCompositionOperationV2::try_from(read_u16(payload, 6)?)?;
        let context = ZrRuntimeImeCompositionContextV2 {
            window_generation: read_u64(payload, 8)?,
            focus_generation: read_u64(payload, 16)?,
            composition_generation: read_u64(payload, 24)?,
        };
        if !context.is_valid() {
            return Err(ZrRuntimeImeCompositionV2Error::InvalidContext);
        }
        let text_len = read_u32(payload, 32)? as usize;
        let cursor_start = read_u32(payload, 36)?;
        let cursor_end = read_u32(payload, 40)?;
        let clause_count = read_u32(payload, 44)? as usize;
        let clause_availability =
            ZrRuntimeImePreeditClauseAvailabilityV2::try_from(read_u32(payload, 48)?)?;
        if clause_count > ZR_RUNTIME_IME_COMPOSITION_V2_MAX_CLAUSES {
            return Err(ZrRuntimeImeCompositionV2Error::CountLimitExceeded);
        }
        let clause_bytes = clause_count
            .checked_mul(ZR_RUNTIME_IME_COMPOSITION_V2_CLAUSE_BYTES)
            .ok_or(ZrRuntimeImeCompositionV2Error::LengthOverflow)?;
        let text_start = ZR_RUNTIME_IME_COMPOSITION_V2_HEADER_BYTES
            .checked_add(clause_bytes)
            .ok_or(ZrRuntimeImeCompositionV2Error::LengthOverflow)?;
        let expected_len = text_start
            .checked_add(text_len)
            .ok_or(ZrRuntimeImeCompositionV2Error::LengthOverflow)?;
        if expected_len != payload.len() {
            return Err(ZrRuntimeImeCompositionV2Error::LengthMismatch);
        }

        // 一对 u32::MAX 表示没有游标；半个哨兵须拒绝，防止它被解释为实际字节范围。
        let cursor_range = match (cursor_start, cursor_end) {
            (u32::MAX, u32::MAX) => None,
            (u32::MAX, _) | (_, u32::MAX) => {
                return Err(ZrRuntimeImeCompositionV2Error::InvalidOperationPayload);
            }
            (start_byte, end_byte) => Some(UiTextByteRange::new(start_byte, end_byte)),
        };
        let mut clauses = Vec::with_capacity(clause_count);
        for index in 0..clause_count {
            let offset = ZR_RUNTIME_IME_COMPOSITION_V2_HEADER_BYTES
                + index * ZR_RUNTIME_IME_COMPOSITION_V2_CLAUSE_BYTES;
            let kind = wire_to_clause_kind(read_u16(payload, offset + 8)?)?;
            if read_u16(payload, offset + 10)? != 0 {
                return Err(ZrRuntimeImeCompositionV2Error::NonZeroReservedClauseField);
            }
            clauses.push(UiImePreeditClause::new(
                UiTextByteRange::new(read_u32(payload, offset)?, read_u32(payload, offset + 4)?),
                kind,
            ));
        }
        let text = String::from_utf8(payload[text_start..].to_vec())
            .map_err(|_| ZrRuntimeImeCompositionV2Error::InvalidUtf8)?;
        let decoded = Self {
            operation,
            context,
            text,
            cursor_range,
            clauses,
            clause_availability,
        };
        decoded.validate()?;
        Ok(decoded)
    }

    /// 检查组合值的上下文和操作内字段关系，供编码前和解码后共用。
    ///
    /// # 返回与错误
    ///
    /// 成功返回 `Ok(())`，不修改 `self`；上下文含零返回 `InvalidContext`，分段数超限返回 `CountLimitExceeded`。
    /// 预编辑在属性不可用时不得携带分段，且游标/分段范围须通过共享范围验证；范围错误包装为 `InvalidPreeditRange`。
    /// 提交不得携带游标、分段或可用属性；取消还要求文本为空，违例返回 `InvalidOperationPayload`。
    /// 当前规则允许空文本的提交及可用但为空的预编辑分段，不额外推断平台状态。
    ///
    /// # 边界与参考
    ///
    /// 本函数不计算完整编码字节预算，不检查当前会话的能力或代数新旧。
    /// [`Self::encode`] 另检查编码长度；[`Self::decode`] 在调用本函数前已验证字节布局与总预算。
    /// 参见 [`UiImePreeditClause::validate_preedit_payload`]、[`ZrRuntimeImeCompositionV2Error`]。
    fn validate(&self) -> Result<(), ZrRuntimeImeCompositionV2Error> {
        if !self.context.is_valid() {
            return Err(ZrRuntimeImeCompositionV2Error::InvalidContext);
        }
        if self.clauses.len() > ZR_RUNTIME_IME_COMPOSITION_V2_MAX_CLAUSES {
            return Err(ZrRuntimeImeCompositionV2Error::CountLimitExceeded);
        }
        match self.operation {
            ZrRuntimeImeCompositionOperationV2::Preedit => {
                if self.clause_availability == ZrRuntimeImePreeditClauseAvailabilityV2::Unavailable
                    && !self.clauses.is_empty()
                {
                    return Err(ZrRuntimeImeCompositionV2Error::InvalidOperationPayload);
                }
                UiImePreeditClause::validate_preedit_payload(
                    &self.text,
                    self.cursor_range,
                    &self.clauses,
                )
                .map_err(ZrRuntimeImeCompositionV2Error::InvalidPreeditRange)?;
            }
            ZrRuntimeImeCompositionOperationV2::Commit => {
                if self.cursor_range.is_some()
                    || !self.clauses.is_empty()
                    || self.clause_availability
                        != ZrRuntimeImePreeditClauseAvailabilityV2::Unavailable
                {
                    return Err(ZrRuntimeImeCompositionV2Error::InvalidOperationPayload);
                }
            }
            ZrRuntimeImeCompositionOperationV2::Cancel => {
                if !self.text.is_empty()
                    || self.cursor_range.is_some()
                    || !self.clauses.is_empty()
                    || self.clause_availability
                        != ZrRuntimeImePreeditClauseAvailabilityV2::Unavailable
                {
                    return Err(ZrRuntimeImeCompositionV2Error::InvalidOperationPayload);
                }
            }
        }
        Ok(())
    }
}

/// 从借用字节序列的指定偏移读取一个小端 `u16` 字段，供组合头和分段解析使用。
///
/// # 参数
///
/// - `bytes`：包含待解析字段的字节序列，仅在调用期间借用。
/// - `offset`：字段起始字节偏移，不要求原生整数对齐。
///
/// # 返回与错误
///
/// 成功返回整数值，不返回借用引用；偏移加字段宽度溢出返回 `LengthOverflow`，对应字节区间不足返回 `HeaderTruncated`。
/// 函数逐字节构造整数，不转换裸指针，也不校验该字段的协议含义；参见 [`ZrRuntimeImeCompositionV2::decode`]。
fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, ZrRuntimeImeCompositionV2Error> {
    let end = offset
        .checked_add(2)
        .ok_or(ZrRuntimeImeCompositionV2Error::LengthOverflow)?;
    let field = bytes
        .get(offset..end)
        .ok_or(ZrRuntimeImeCompositionV2Error::HeaderTruncated)?;
    Ok(u16::from_le_bytes([field[0], field[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, ZrRuntimeImeCompositionV2Error> {
    let end = offset
        .checked_add(4)
        .ok_or(ZrRuntimeImeCompositionV2Error::LengthOverflow)?;
    let field = bytes
        .get(offset..end)
        .ok_or(ZrRuntimeImeCompositionV2Error::HeaderTruncated)?;
    Ok(u32::from_le_bytes([field[0], field[1], field[2], field[3]]))
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64, ZrRuntimeImeCompositionV2Error> {
    let end = offset
        .checked_add(8)
        .ok_or(ZrRuntimeImeCompositionV2Error::LengthOverflow)?;
    let field = bytes
        .get(offset..end)
        .ok_or(ZrRuntimeImeCompositionV2Error::HeaderTruncated)?;
    Ok(u64::from_le_bytes([
        field[0], field[1], field[2], field[3], field[4], field[5], field[6], field[7],
    ]))
}

/// 把共享 UI 分段样式转换为组合 V2 的固定整数标签，供编码写入分段表。
///
/// # 参数与返回
///
/// `kind` 为已知 [`UiImePreeditClauseKind`]；四种样式依次映射为协议整数 1 至 4。
/// 此转换没有错误分支，也不验证分段范围；范围由组合值的语义验证负责。
/// 参见 [`wire_to_clause_kind`] 和 [`ZrRuntimeImeCompositionV2::encode`]。
fn clause_kind_to_wire(kind: UiImePreeditClauseKind) -> u16 {
    match kind {
        UiImePreeditClauseKind::Input => 1,
        UiImePreeditClauseKind::Converted => 2,
        UiImePreeditClauseKind::TargetConverted => 3,
        UiImePreeditClauseKind::TargetNotConverted => 4,
    }
}

/// 把分段表的整数标签解析为共享 UI 分段样式，拒绝协议尚未定义的值。
///
/// # 参数与返回
///
/// `value` 为从小端 `u16` 字段读出的标签；1 至 4 返回相应 [`UiImePreeditClauseKind`]。
/// 其他值返回 [`ZrRuntimeImeCompositionV2Error::UnknownClauseKind`]；不会把未知样式降级为普通输入。
/// 转换不检查范围端点或顺序；参见 [`clause_kind_to_wire`] 和 [`ZrRuntimeImeCompositionV2::decode`]。
fn wire_to_clause_kind(
    value: u16,
) -> Result<UiImePreeditClauseKind, ZrRuntimeImeCompositionV2Error> {
    match value {
        1 => Ok(UiImePreeditClauseKind::Input),
        2 => Ok(UiImePreeditClauseKind::Converted),
        3 => Ok(UiImePreeditClauseKind::TargetConverted),
        4 => Ok(UiImePreeditClauseKind::TargetNotConverted),
        _ => Err(ZrRuntimeImeCompositionV2Error::UnknownClauseKind),
    }
}

#[cfg(test)]
#[path = "tests/ime_composition_v2.rs"]
mod tests;
