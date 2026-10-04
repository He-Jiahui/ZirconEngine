use std::{collections::BTreeMap, str::FromStr};

use super::{
    UiBindingAssetReference, UiBindingCall, UiBindingCollectionView, UiBindingEntityReference,
    UiBindingEnumValue, UiBindingMapKey, UiBindingParseError, UiBindingValue, UiEventBinding,
    UiEventKind, UiEventPath, UiModelProviderId, UiModelProviderKey, UiModelProviderVersion,
    UiModelSchemaId, UiModelSchemaKey, UiModelSchemaVersion,
};

pub(super) struct BindingParser<'a> {
    input: &'a str,
    index: usize,
}

impl<'a> BindingParser<'a> {
    pub(super) fn new(input: &'a str) -> Self {
        Self { input, index: 0 }
    }

    // 只有完整读完路径、可选动作和尾部空白后才构造结果，避免调用方接收部分匹配。
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

    fn skip_ws(&mut self) {
        while matches!(self.peek_char(), Some(ch) if ch.is_whitespace()) {
            self.index += 1;
        }
    }

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

    fn peek_char(&self) -> Option<char> {
        self.input[self.index..].chars().next()
    }

    fn remaining(&self) -> &str {
        &self.input[self.index..]
    }

    fn is_eof(&self) -> bool {
        self.index >= self.input.len()
    }
}

#[cfg(test)]
#[path = "parser/tests/string_performance_tests.rs"]
mod string_performance_tests;

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
#[path = "parser/tests/constructed_entry_performance_tests.rs"]
mod constructed_entry_performance_tests;

#[cfg(test)]
#[path = "parser/tests/collection_view_performance_tests.rs"]
mod collection_view_performance_tests;

fn invalid_constructor(constructor: &str, reason: &str) -> UiBindingParseError {
    UiBindingParseError::InvalidValueConstructor {
        constructor: constructor.to_string(),
        reason: reason.to_string(),
    }
}
