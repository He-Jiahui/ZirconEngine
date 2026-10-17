//! 原生绑定文本的借用游标与拥有值构造器。
//!
//! 公共绑定入口先读取路由头，再解析可选动作与参数，最后拒绝尾随内容。
//! 游标以字节计数；字符读取和字符串切片依赖 UTF-8 边界，成功结果不借用输入。
//! 参数在构造后逐值准入，解析过程本身没有提前限制所有嵌套分配；相关预算待办位于值模块。
//! 事件种类与值构造器使用静态分派，不在这里注册或执行 Runtime 处理器。

use std::{collections::BTreeMap, str::FromStr};

use super::{
    UiBindingAssetReference, UiBindingCall, UiBindingCollectionView, UiBindingEntityReference,
    UiBindingEnumValue, UiBindingMapKey, UiBindingParseError, UiBindingValue, UiEventBinding,
    UiEventKind, UiEventPath, UiModelProviderId, UiModelProviderKey, UiModelProviderVersion,
    UiModelSchemaId, UiModelSchemaKey, UiModelSchemaVersion,
};

/// 在一次文本解析期间独占推进的借用游标。
///
/// 输入在游标存活期间保持有效；正常字符消费须按字符字节长度推进，才能继续借用后缀。
pub(super) struct BindingParser<'a> {
    /// 解析期间只读借用的完整 UTF-8 文本。
    input: &'a str,
    /// 当前字节偏移；字符后缀访问要求它位于输入的字符边界。
    index: usize,
}

impl<'a> BindingParser<'a> {
    /// 借用输入并从字节零开始；不预先解析、分配路径或校验资源预算。
    pub(super) fn new(input: &'a str) -> Self {
        Self { input, index: 0 }
    }

    // 只有完整读完路径、可选动作和尾部空白后才构造结果，避免调用方接收部分匹配。
    /// 消费整个游标，返回拥有内容的绑定或首个解析错误。
    ///
    /// 先解析头部，仅在存在外层动作括号时读取调用；尾部仅允许空白，不返回部分匹配。
    pub(super) fn parse_binding(mut self) -> Result<UiEventBinding, UiBindingParseError> {
        let (view_id, control_id, event_kind, has_action) = self.parse_header()?;
        let action = if has_action {
            let call = self.parse_call()?;
            self.skip_ws();
            self.expect(')')?;
            Some(call)
        } else {
            None
        };
        self.skip_ws();
        if !self.is_eof() {
            return Err(UiBindingParseError::TrailingInput);
        }
        Ok(UiEventBinding {
            path: UiEventPath::new(view_id, control_id, event_kind),
            action,
        })
    }

    /// 读取路径分隔、非空标识及静态事件名称，并记录是否消费了外层动作括号。
    ///
    /// 返回的两个标识拥有内容；路径分隔使用首次匹配，未在这里处理转义。
    fn parse_header(&mut self) -> Result<(String, String, UiEventKind, bool), UiBindingParseError> {
        let Some((path, event_and_payload)) = self.input.split_once(':') else {
            return Err(UiBindingParseError::MissingEventSeparator);
        };
        let Some((view_id, control_id)) = path.split_once('/') else {
            return Err(UiBindingParseError::MissingControlSeparator);
        };
        if view_id.is_empty() || control_id.is_empty() {
            return Err(UiBindingParseError::EmptyPathSegment);
        }
        let event_end = event_and_payload
            .find('(')
            .unwrap_or(event_and_payload.len());
        let event_kind = UiEventKind::from_str(event_and_payload[..event_end].trim())?;
        self.index = path.len() + 1 + event_end;
        let has_action = event_end < event_and_payload.len();
        if has_action {
            self.expect('(')?;
        }
        Ok((
            view_id.to_string(),
            control_id.to_string(),
            event_kind,
            has_action,
        ))
    }

    /// 在外层动作括号已打开后读取动作符号与内层参数表。
    ///
    /// 空参数表直接结束；否则逐值读取并要求逗号或右括号，错误不回滚当前游标。
    fn parse_call(&mut self) -> Result<UiBindingCall, UiBindingParseError> {
        self.skip_ws();
        let symbol = self.parse_symbol()?;
        self.skip_ws();
        self.expect('(')?;
        let mut arguments = Vec::new();
        self.skip_ws();
        if self.peek_char() != Some(')') {
            loop {
                arguments.push(self.parse_value()?);
                self.skip_ws();
                match self.peek_char() {
                    Some(',') => {
                        self.index += 1;
                        self.skip_ws();
                    }
                    Some(')') => break,
                    Some(found) => {
                        return Err(UiBindingParseError::ExpectedCharacter {
                            expected: ')',
                            found,
                        });
                    }
                    None => return Err(UiBindingParseError::UnexpectedEnd),
                }
            }
        }
        self.expect(')')?;
        Ok(UiBindingCall { symbol, arguments })
    }

    /// 按当前起始字符解析一个拥有值，随后校验该值的完整子树。
    ///
    /// 数组与构造器递归进入同一入口；校验针对当前参数，不累加兄弟参数的总预算。
    fn parse_value(&mut self) -> Result<UiBindingValue, UiBindingParseError> {
        self.skip_ws();
        let value = match self.peek_char() {
            Some('"') => self.parse_string().map(UiBindingValue::String),
            Some('[') => self.parse_array(),
            Some('-') | Some('0'..='9') => self.parse_number(),
            Some('t') if self.remaining().starts_with("true") => {
                self.index += 4;
                Ok(UiBindingValue::Bool(true))
            }
            Some('f') if self.remaining().starts_with("false") => {
                self.index += 5;
                Ok(UiBindingValue::Bool(false))
            }
            Some('n') if self.remaining().starts_with("null") => {
                self.index += 4;
                Ok(UiBindingValue::Null)
            }
            Some('a'..='z' | 'A'..='Z' | '_') => self.parse_constructed_value(),
            Some(_) => Err(UiBindingParseError::InvalidNumber),
            None => Err(UiBindingParseError::UnexpectedEnd),
        }?;
        value.validate()?;
        Ok(value)
    }

    // 各构造器在此统一分派；单个构造器负责校验参数数量、类型和领域对象不变量。
    /// 读取构造器名和参数表，交给固定名称对应的领域构造逻辑；未知名称返回构造器错误。
    fn parse_constructed_value(&mut self) -> Result<UiBindingValue, UiBindingParseError> {
        let constructor = self.parse_symbol()?;
        self.skip_ws();
        self.expect('(')?;
        let arguments = self.parse_value_list(')')?;
        match constructor.as_str() {
            "record" => self.construct_record(&constructor, arguments),
            "map" => self.construct_map(&constructor, arguments),
            "enum" => self.construct_enum(&constructor, arguments),
            "asset" => self.construct_asset(&constructor, arguments),
            "entity" => self.construct_entity(&constructor, arguments),
            "optional" => self.construct_optional(&constructor, arguments),
            "collection_view" => self.construct_collection_view(&constructor, arguments),
            _ => Err(invalid_constructor(&constructor, "unknown constructor")),
        }
    }

    /// 读取允许空表的逗号分隔值序列，消耗关闭括号并返回拥有的参数向量。
    fn parse_value_list(
        &mut self,
        closing: char,
    ) -> Result<Vec<UiBindingValue>, UiBindingParseError> {
        let mut values = Vec::new();
        self.skip_ws();
        if self.peek_char() != Some(closing) {
            loop {
                values.push(self.parse_value()?);
                self.skip_ws();
                match self.peek_char() {
                    Some(',') => {
                        self.index += 1;
                        self.skip_ws();
                    }
                    Some(found) if found == closing => break,
                    Some(found) => {
                        return Err(UiBindingParseError::ExpectedCharacter {
                            expected: closing,
                            found,
                        });
                    }
                    None => return Err(UiBindingParseError::UnexpectedEnd),
                }
            }
        }
        self.expect(closing)?;
        Ok(values)
    }

    /// 把偶数项参数按字符串字段名和值配对移动到有序字段表。
    ///
    /// 先检查数量，再拒绝错误键类型或重复字段，最后让值类型校验所得记录。
    fn construct_record(
        &self,
        constructor: &str,
        arguments: Vec<UiBindingValue>,
    ) -> Result<UiBindingValue, UiBindingParseError> {
        require_even_arity(constructor, &arguments)?;
        let mut fields = BTreeMap::new();
        let mut arguments = arguments.into_iter();
        while let Some(field) = arguments.next() {
            let value = arguments
                .next()
                .expect("even record arity was validated before pair consumption");
            let UiBindingValue::String(field) = field else {
                return Err(invalid_constructor(
                    constructor,
                    "record field names must be strings",
                ));
            };
            if fields.insert(field, value).is_some() {
                return Err(invalid_constructor(
                    constructor,
                    "record field names must be unique",
                ));
            }
        }
        UiBindingValue::record(fields).map_err(Into::into)
    }

    /// 把偶数项参数移动为键值对；键只接受字符串、有符号或无符号整数。
    ///
    /// 领域映射负责后续键准入，值入口负责整个映射的资源与嵌套值校验。
    fn construct_map(
        &self,
        constructor: &str,
        arguments: Vec<UiBindingValue>,
    ) -> Result<UiBindingValue, UiBindingParseError> {
        require_even_arity(constructor, &arguments)?;
        let mut entries = Vec::with_capacity(arguments.len() / 2);
        let mut arguments = arguments.into_iter();
        while let Some(key) = arguments.next() {
            let value = arguments
                .next()
                .expect("even map arity was validated before pair consumption");
            let key = match key {
                UiBindingValue::String(value) => UiBindingMapKey::String(value),
                UiBindingValue::Unsigned(value) => UiBindingMapKey::Unsigned(value),
                UiBindingValue::Signed(value) => UiBindingMapKey::Signed(value),
                UiBindingValue::Bool(value) => UiBindingMapKey::Bool(value),
                _ => {
                    return Err(invalid_constructor(
                        constructor,
                        "map keys must be string, unsigned, signed, or bool scalars",
                    ));
                }
            };
            entries.push((key, value));
        }
        UiBindingValue::map(entries).map_err(Into::into)
    }

    /// 仅测试保留的借用后复制版本，用作记录构造结果与错误的比较基准。
    #[cfg(test)]
    fn construct_record_cloning(
        &self,
        constructor: &str,
        arguments: Vec<UiBindingValue>,
    ) -> Result<UiBindingValue, UiBindingParseError> {
        require_even_arity(constructor, &arguments)?;
        let mut fields = BTreeMap::new();
        for pair in arguments.chunks_exact(2) {
            let UiBindingValue::String(field) = &pair[0] else {
                return Err(invalid_constructor(
                    constructor,
                    "record field names must be strings",
                ));
            };
            if fields.insert(field.clone(), pair[1].clone()).is_some() {
                return Err(invalid_constructor(
                    constructor,
                    "record field names must be unique",
                ));
            }
        }
        UiBindingValue::record(fields).map_err(Into::into)
    }

    /// 仅测试保留的借用后复制版本，用作映射构造结果与错误的比较基准。
    #[cfg(test)]
    fn construct_map_cloning(
        &self,
        constructor: &str,
        arguments: Vec<UiBindingValue>,
    ) -> Result<UiBindingValue, UiBindingParseError> {
        require_even_arity(constructor, &arguments)?;
        let entries = arguments
            .chunks_exact(2)
            .map(|pair| {
                let key = match &pair[0] {
                    UiBindingValue::String(value) => UiBindingMapKey::String(value.clone()),
                    UiBindingValue::Unsigned(value) => UiBindingMapKey::Unsigned(*value),
                    UiBindingValue::Signed(value) => UiBindingMapKey::Signed(*value),
                    UiBindingValue::Bool(value) => UiBindingMapKey::Bool(*value),
                    _ => {
                        return Err(invalid_constructor(
                            constructor,
                            "map keys must be string, unsigned, signed, or bool scalars",
                        ));
                    }
                };
                Ok((key, pair[1].clone()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        UiBindingValue::map(entries).map_err(Into::into)
    }

    /// 接受类型名、变体名及可选载荷，按领域枚举构造器校验身份。
    fn construct_enum(
        &self,
        constructor: &str,
        mut arguments: Vec<UiBindingValue>,
    ) -> Result<UiBindingValue, UiBindingParseError> {
        if !(2..=3).contains(&arguments.len()) {
            return Err(invalid_constructor(
                constructor,
                "expected enum(type_id, variant[, payload])",
            ));
        }
        let payload = (arguments.len() == 3).then(|| arguments.pop().unwrap());
        let variant = take_string_argument(constructor, arguments.pop().unwrap(), 1)?;
        let type_id = take_string_argument(constructor, arguments.pop().unwrap(), 0)?;
        Ok(UiBindingValue::Enum(UiBindingEnumValue::try_new(
            type_id, variant, payload,
        )?))
    }

    /// 接受唯一字符串定位符，由资源引用构造器检查身份约束。
    fn construct_asset(
        &self,
        constructor: &str,
        mut arguments: Vec<UiBindingValue>,
    ) -> Result<UiBindingValue, UiBindingParseError> {
        require_arity(constructor, &arguments, 1)?;
        let locator = take_string_argument(constructor, arguments.pop().unwrap(), 0)?;
        Ok(UiBindingValue::Asset(UiBindingAssetReference::try_new(
            locator,
        )?))
    }

    /// 接受两个无符号参数并交给实体引用构造器检查代次约束。
    fn construct_entity(
        &self,
        constructor: &str,
        arguments: Vec<UiBindingValue>,
    ) -> Result<UiBindingValue, UiBindingParseError> {
        require_arity(constructor, &arguments, 2)?;
        let entity_id = unsigned_argument(constructor, &arguments, 0)?;
        let generation = unsigned_argument(constructor, &arguments, 1)?;
        Ok(UiBindingValue::Entity(UiBindingEntityReference::try_new(
            entity_id, generation,
        )?))
    }

    /// 接受零或一个拥有值，分别构造空可选值或装箱载荷；其他数量返回错误。
    fn construct_optional(
        &self,
        constructor: &str,
        mut arguments: Vec<UiBindingValue>,
    ) -> Result<UiBindingValue, UiBindingParseError> {
        if arguments.len() > 1 {
            return Err(invalid_constructor(
                constructor,
                "expected optional() or optional(value)",
            ));
        }
        Ok(UiBindingValue::Optional(arguments.pop().map(Box::new)))
    }

    /// 读取八项集合窗口参数，先按固定顺序检查数量、数值和字符串类型，再移动身份字符串。
    ///
    /// 两个身份键与窗口均须通过其领域构造器；错误检查顺序与测试中的历史实现保持一致。
    fn construct_collection_view(
        &self,
        constructor: &str,
        arguments: Vec<UiBindingValue>,
    ) -> Result<UiBindingValue, UiBindingParseError> {
        require_arity(constructor, &arguments, 8)?;
        let total_length = unsigned_argument(constructor, &arguments, 7)?;
        let length = unsigned_argument(constructor, &arguments, 6)?
            .try_into()
            .map_err(|_| invalid_constructor(constructor, "window length exceeds u32"))?;
        let offset = unsigned_argument(constructor, &arguments, 5)?;
        let revision = unsigned_argument(constructor, &arguments, 4)?;
        let item_schema_version =
            UiModelSchemaVersion::try_new(unsigned_argument(constructor, &arguments, 3)?)
                .map_err(|error| invalid_constructor(constructor, &error.to_string()))?;
        if !matches!(arguments.get(2), Some(UiBindingValue::String(_))) {
            return Err(invalid_constructor(
                constructor,
                "argument 2 must be a string",
            ));
        }
        let provider_version =
            UiModelProviderVersion::try_new(unsigned_argument(constructor, &arguments, 1)?)
                .map_err(|error| invalid_constructor(constructor, &error.to_string()))?;
        let mut arguments = arguments.into_iter();
        let provider_id = arguments
            .next()
            .expect("collection-view arity was validated before argument consumption");
        let _ = arguments.next();
        let item_schema_id = arguments
            .next()
            .expect("collection-view arity was validated before argument consumption");
        let item_schema_id = take_string_argument(constructor, item_schema_id, 2)?;
        let provider_id = take_string_argument(constructor, provider_id, 0)?;
        let provider = UiModelProviderKey {
            id: UiModelProviderId::try_new(provider_id)
                .map_err(|error| invalid_constructor(constructor, &error.to_string()))?,
            version: provider_version,
        };
        let item_schema = UiModelSchemaKey {
            id: UiModelSchemaId::try_new(item_schema_id)
                .map_err(|error| invalid_constructor(constructor, &error.to_string()))?,
            version: item_schema_version,
        };
        Ok(UiBindingValue::CollectionView(
            UiBindingCollectionView::try_new(
                provider,
                item_schema,
                revision,
                offset,
                length,
                total_length,
            )?,
        ))
    }

    /// 递归读取允许空表的方括号值序列；父值入口随后校验整个数组的预算。
    fn parse_array(&mut self) -> Result<UiBindingValue, UiBindingParseError> {
        self.expect('[')?;
        let mut values = Vec::new();
        self.skip_ws();
        if self.peek_char() != Some(']') {
            loop {
                values.push(self.parse_value()?);
                self.skip_ws();
                match self.peek_char() {
                    Some(',') => {
                        self.index += 1;
                        self.skip_ws();
                    }
                    Some(']') => break,
                    Some(found) => {
                        return Err(UiBindingParseError::ExpectedCharacter {
                            expected: ']',
                            found,
                        });
                    }
                    None => return Err(UiBindingParseError::UnexpectedEnd),
                }
            }
        }
        self.expect(']')?;
        Ok(UiBindingValue::Array(values))
    }

    /// 读取引号内文本并返回拥有字符串，关闭引号之后的内容留给外层解析。
    ///
    /// 无转义时一次复制借用片段；遇到转义后按片段追加，仅支持本分支列出的五种转义。
    /// 缺少关闭引号或遇到不支持的转义时返回相应错误。
    fn parse_string(&mut self) -> Result<String, UiBindingParseError> {
        self.expect('"')?;
        let value_start = self.index;
        let remaining = self.remaining();
        let Some(first_special_offset) = remaining.find(['"', '\\']) else {
            self.index = self.input.len();
            return Err(UiBindingParseError::UnterminatedString);
        };
        let first_special_index = value_start + first_special_offset;
        if self.input.as_bytes()[first_special_index] == b'"' {
            let value_end = first_special_index;
            self.index = value_end + 1;
            return Ok(self.input[value_start..value_end].to_string());
        }

        let mut output = String::with_capacity(first_special_offset.saturating_add(16));
        let mut chunk_start = value_start;
        let mut special_index = first_special_index;
        loop {
            output.push_str(&self.input[chunk_start..special_index]);
            self.index = special_index + 1;
            match self.input.as_bytes()[special_index] {
                b'"' => return Ok(output),
                b'\\' => {
                    let escaped = self.peek_char().ok_or(UiBindingParseError::InvalidEscape)?;
                    self.index += escaped.len_utf8();
                    output.push(match escaped {
                        '"' => '"',
                        '\\' => '\\',
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        _ => return Err(UiBindingParseError::InvalidEscape),
                    });
                    chunk_start = self.index;
                }
                _ => unreachable!("binding string delimiter scan returns only quote or escape"),
            }

            let Some(next_special_offset) = self.remaining().find(['"', '\\']) else {
                self.index = self.input.len();
                return Err(UiBindingParseError::UnterminatedString);
            };
            special_index = self.index + next_special_offset;
        }
    }

    /// 扫描可选符号、十进制小数和指数，再按是否包含小数或指数选择数值类型。
    ///
    /// 指数必须含数字；转换失败返回数字错误，浮点有限性由外层值校验继续检查。
    fn parse_number(&mut self) -> Result<UiBindingValue, UiBindingParseError> {
        let start = self.index;
        if self.peek_char() == Some('-') {
            self.index += 1;
        }
        while matches!(self.peek_char(), Some('0'..='9')) {
            self.index += 1;
        }
        let mut is_float = false;
        if self.peek_char() == Some('.') {
            is_float = true;
            self.index += 1;
            while matches!(self.peek_char(), Some('0'..='9')) {
                self.index += 1;
            }
        }
        if matches!(self.peek_char(), Some('e' | 'E')) {
            is_float = true;
            self.index += 1;
            if matches!(self.peek_char(), Some('+' | '-')) {
                self.index += 1;
            }
            let exponent_start = self.index;
            while matches!(self.peek_char(), Some('0'..='9')) {
                self.index += 1;
            }
            if self.index == exponent_start {
                return Err(UiBindingParseError::InvalidNumber);
            }
        }
        let literal = &self.input[start..self.index];
        if is_float {
            literal
                .parse::<f64>()
                .map(UiBindingValue::Float)
                .map_err(|_| UiBindingParseError::InvalidNumber)
        } else if literal.starts_with('-') {
            literal
                .parse::<i64>()
                .map(UiBindingValue::Signed)
                .map_err(|_| UiBindingParseError::InvalidNumber)
        } else {
            literal
                .parse::<u64>()
                .map(UiBindingValue::Unsigned)
                .map_err(|_| UiBindingParseError::InvalidNumber)
        }
    }

    /// 读取连续的 ASCII 字母、数字、下划线和点作为拥有符号，空串返回错误。
    fn parse_symbol(&mut self) -> Result<String, UiBindingParseError> {
        let start = self.index;
        while matches!(
            self.peek_char(),
            Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.')
        ) {
            self.index += 1;
        }
        if self.index == start {
            return Err(UiBindingParseError::InvalidCallSymbol);
        }
        Ok(self.input[start..self.index].to_string())
    }

    /// 跳过字符分类为空白的输入；每次循环均会再次读取当前字节后缀。
    fn skip_ws(&mut self) {
        while matches!(self.peek_char(), Some(ch) if ch.is_whitespace()) {
            // BUG: [CR-ZCA-R067-0001] 事件外层括号后的 U+3000 空白到达这里时，单字节推进会落入字符内部；
            // 下一次读取后缀时会 panic，无法返回解析错误。此为静态路径证明，未运行复现。
            self.index += 1;
        }
    }

    /// 消费一个期望字符并按其字节长度推进；不匹配或已结束时返回结构化错误。
    fn expect(&mut self, expected: char) -> Result<(), UiBindingParseError> {
        match self.peek_char() {
            Some(found) if found == expected => {
                self.index += expected.len_utf8();
                Ok(())
            }
            Some(found) => Err(UiBindingParseError::ExpectedCharacter { expected, found }),
            None => Err(UiBindingParseError::UnexpectedEnd),
        }
    }

    /// 借用当前后缀读取下一个字符；偏移必须是 UTF-8 边界，否则切片会 panic。
    fn peek_char(&self) -> Option<char> {
        self.input[self.index..].chars().next()
    }

    /// 借用尚未消费的后缀；有效期受游标输入借用限制，并要求当前偏移是字符边界。
    fn remaining(&self) -> &str {
        &self.input[self.index..]
    }

    /// 只比较偏移与字节总长，不替代字符边界检查。
    fn is_eof(&self) -> bool {
        self.index >= self.input.len()
    }
}

#[cfg(test)]
#[path = "parser/tests/string_perf.rs"]
mod string_performance_tests;

/// 检查精确参数数目，失败诊断同时拥有构造器名称、期望数与实际数。
fn require_arity(
    constructor: &str,
    arguments: &[UiBindingValue],
    expected: usize,
) -> Result<(), UiBindingParseError> {
    if arguments.len() == expected {
        Ok(())
    } else {
        Err(invalid_constructor(
            constructor,
            &format!(
                "expected {expected} arguments but received {}",
                arguments.len()
            ),
        ))
    }
}

/// 检查键值对参数可整齐配对，失败时返回该构造器的数量诊断。
fn require_even_arity(
    constructor: &str,
    arguments: &[UiBindingValue],
) -> Result<(), UiBindingParseError> {
    if arguments.len() % 2 == 0 {
        Ok(())
    } else {
        Err(invalid_constructor(
            constructor,
            "expected alternating key and value arguments",
        ))
    }
}

/// 消费一个参数并取出字符串；类型不符时按原参数位置返回拥有诊断。
fn take_string_argument(
    constructor: &str,
    argument: UiBindingValue,
    index: usize,
) -> Result<String, UiBindingParseError> {
    match argument {
        UiBindingValue::String(value) => Ok(value),
        _ => Err(invalid_constructor(
            constructor,
            &format!("argument {index} must be a string"),
        )),
    }
}

/// 按索引借用参数并复制无符号数；缺项或其他值种类返回同一位置的类型诊断。
fn unsigned_argument(
    constructor: &str,
    arguments: &[UiBindingValue],
    index: usize,
) -> Result<u64, UiBindingParseError> {
    match arguments.get(index) {
        Some(UiBindingValue::Unsigned(value)) => Ok(*value),
        _ => Err(invalid_constructor(
            constructor,
            &format!("argument {index} must be unsigned"),
        )),
    }
}

#[cfg(test)]
#[path = "../../model/parser/tests/constructed_entry_perf.rs"]
mod constructed_entry_performance_tests;

#[cfg(test)]
#[path = "../../model/parser/tests/collection_view_perf.rs"]
mod collection_view_performance_tests;

/// 统一把借用的构造器名称与原因复制到拥有错误中。
fn invalid_constructor(constructor: &str, reason: &str) -> UiBindingParseError {
    UiBindingParseError::InvalidValueConstructor {
        constructor: constructor.to_string(),
        reason: reason.to_string(),
    }
}
