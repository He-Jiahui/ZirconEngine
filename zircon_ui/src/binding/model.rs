use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UiEventPath {
    pub view_id: String,
    pub control_id: String,
    pub event_kind: UiEventKind,
}

impl UiEventPath {
    pub fn new(
        view_id: impl Into<String>,
        control_id: impl Into<String>,
        event_kind: UiEventKind,
    ) -> Self {
        Self {
            view_id: view_id.into(),
            control_id: control_id.into(),
            event_kind,
        }
    }

    pub fn native_prefix(&self) -> String {
        format!(
            "{}/{}:{}",
            self.view_id,
            self.control_id,
            self.event_kind.native_name()
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum UiEventKind {
    Click,
    DoubleClick,
    Hover,
    Press,
    Release,
    Change,
    Submit,
    Toggle,
    Focus,
    Blur,
    Scroll,
    Resize,
    DragBegin,
    DragUpdate,
    DragEnd,
}

impl UiEventKind {
    pub fn native_name(self) -> &'static str {
        match self {
            Self::Click => "onClick",
            Self::DoubleClick => "onDoubleClick",
            Self::Hover => "onHover",
            Self::Press => "onPress",
            Self::Release => "onRelease",
            Self::Change => "onChange",
            Self::Submit => "onSubmit",
            Self::Toggle => "onToggle",
            Self::Focus => "onFocus",
            Self::Blur => "onBlur",
            Self::Scroll => "onScroll",
            Self::Resize => "onResize",
            Self::DragBegin => "onDragBegin",
            Self::DragUpdate => "onDragUpdate",
            Self::DragEnd => "onDragEnd",
        }
    }
}

impl fmt::Display for UiEventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.native_name())
    }
}

impl FromStr for UiEventKind {
    type Err = UiBindingParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "onClick" => Ok(Self::Click),
            "onDoubleClick" => Ok(Self::DoubleClick),
            "onHover" => Ok(Self::Hover),
            "onPress" => Ok(Self::Press),
            "onRelease" => Ok(Self::Release),
            "onChange" => Ok(Self::Change),
            "onSubmit" => Ok(Self::Submit),
            "onToggle" => Ok(Self::Toggle),
            "onFocus" => Ok(Self::Focus),
            "onBlur" => Ok(Self::Blur),
            "onScroll" => Ok(Self::Scroll),
            "onResize" => Ok(Self::Resize),
            "onDragBegin" => Ok(Self::DragBegin),
            "onDragUpdate" => Ok(Self::DragUpdate),
            "onDragEnd" => Ok(Self::DragEnd),
            other => Err(UiBindingParseError::UnknownEventKind(other.to_string())),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UiBindingValue {
    String(String),
    Unsigned(u64),
    Signed(i64),
    Float(f64),
    Bool(bool),
    Null,
    Array(Vec<UiBindingValue>),
}

impl UiBindingValue {
    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    pub fn unsigned(value: u32) -> Self {
        Self::Unsigned(value as u64)
    }

    pub fn array(values: impl Into<Vec<UiBindingValue>>) -> Self {
        Self::Array(values.into())
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_u32(&self) -> Option<u32> {
        match self {
            Self::Unsigned(value) => (*value).try_into().ok(),
            Self::Signed(value) if *value >= 0 => (*value as u64).try_into().ok(),
            _ => None,
        }
    }

    fn native_repr(&self) -> String {
        match self {
            Self::String(value) => format!("\"{}\"", escape_string(value)),
            Self::Unsigned(value) => value.to_string(),
            Self::Signed(value) => value.to_string(),
            Self::Float(value) => {
                let mut rendered = value.to_string();
                if !rendered.contains('.') && !rendered.contains('e') && !rendered.contains('E') {
                    rendered.push_str(".0");
                }
                rendered
            }
            Self::Bool(value) => value.to_string(),
            Self::Null => "null".to_string(),
            Self::Array(values) => format!(
                "[{}]",
                values
                    .iter()
                    .map(Self::native_repr)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiBindingCall {
    pub symbol: String,
    pub arguments: Vec<UiBindingValue>,
}

impl UiBindingCall {
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            arguments: Vec::new(),
        }
    }

    pub fn with_argument(mut self, argument: UiBindingValue) -> Self {
        self.arguments.push(argument);
        self
    }

    pub fn argument(&self, index: usize) -> Option<&UiBindingValue> {
        self.arguments.get(index)
    }

    pub fn native_repr(&self) -> String {
        format!(
            "{}({})",
            self.symbol,
            self.arguments
                .iter()
                .map(UiBindingValue::native_repr)
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiEventBinding {
    pub path: UiEventPath,
    pub action: Option<UiBindingCall>,
}

impl UiEventBinding {
    pub fn new(path: UiEventPath, action: UiBindingCall) -> Self {
        Self {
            path,
            action: Some(action),
        }
    }

    pub fn without_action(path: UiEventPath) -> Self {
        Self { path, action: None }
    }

    pub fn native_binding(&self) -> String {
        let prefix = self.path.native_prefix();
        match &self.action {
            Some(action) => format!("{prefix}({})", action.native_repr()),
            None => prefix,
        }
    }

    pub fn parse_native_binding(input: &str) -> Result<Self, UiBindingParseError> {
        BindingParser::new(input).parse_binding()
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum UiBindingParseError {
    #[error("binding is missing event separator ':'")]
    MissingEventSeparator,
    #[error("binding is missing control separator '/'")]
    MissingControlSeparator,
    #[error("binding contains empty view or control id")]
    EmptyPathSegment,
    #[error("binding contains unexpected trailing input")]
    TrailingInput,
    #[error("unknown ui event kind {0}")]
    UnknownEventKind(String),
    #[error("unexpected end of input")]
    UnexpectedEnd,
    #[error("expected '{expected}' but found '{found}'")]
    ExpectedCharacter { expected: char, found: char },
    #[error("invalid binding call symbol")]
    InvalidCallSymbol,
    #[error("invalid numeric literal")]
    InvalidNumber,
    #[error("unterminated string literal")]
    UnterminatedString,
    #[error("invalid escape sequence")]
    InvalidEscape,
}

fn escape_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

struct BindingParser<'a> {
    input: &'a str,
    index: usize,
}

impl<'a> BindingParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, index: 0 }
    }

    fn parse_binding(mut self) -> Result<UiEventBinding, UiBindingParseError> {
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
                        })
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
        match self.peek_char() {
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
            Some(_) => Err(UiBindingParseError::InvalidNumber),
            None => Err(UiBindingParseError::UnexpectedEnd),
        }
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
                        })
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
        let mut output = String::new();
        while let Some(ch) = self.peek_char() {
            self.index += ch.len_utf8();
            match ch {
                '"' => return Ok(output),
                '\\' => {
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
                }
                other => output.push(other),
            }
        }
        Err(UiBindingParseError::UnterminatedString)
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
