use std::str::FromStr;

use super::{
    UiBindingCall, UiBindingParseError, UiBindingValue, UiEventBinding, UiEventKind, UiEventPath,
};

pub(super) struct BindingParser<'a> {
    input: &'a str,
    index: usize,
}

impl<'a> BindingParser<'a> {
    pub(super) fn new(input: &'a str) -> Self {
        Self { input, index: 0 }
    }

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
