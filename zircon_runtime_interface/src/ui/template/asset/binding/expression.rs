use std::fmt;

use serde::{Deserialize, Serialize};

use crate::ui::component::UiValue;

mod evaluator;

pub use evaluator::UiBindingExpressionEvaluationError;

pub const UI_BINDING_EXPRESSION_MAX_SOURCE_BYTES: usize = 16 * 1024;
pub const UI_BINDING_EXPRESSION_MAX_TOKENS: usize = 2_048;
pub const UI_BINDING_EXPRESSION_MAX_NODES: usize = 1_024;
pub const UI_BINDING_EXPRESSION_MAX_DEPTH: usize = 64;
pub const UI_BINDING_EXPRESSION_INLINE_STACK_CAPACITY: usize = 8;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UiBindingExpression {
    Literal(UiValue),
    ParamRef(String),
    PropRef(String),
    /// References a descriptor-owned property on another control in the current tree.
    ControlPropRef {
        control_id: String,
        property: String,
    },
    Equals(Box<UiBindingExpression>, Box<UiBindingExpression>),
    NotEquals(Box<UiBindingExpression>, Box<UiBindingExpression>),
    And(Box<UiBindingExpression>, Box<UiBindingExpression>),
    Or(Box<UiBindingExpression>, Box<UiBindingExpression>),
    Not(Box<UiBindingExpression>),
}

impl UiBindingExpression {
    pub fn parse(input: &str) -> Result<Self, UiBindingExpressionParseError> {
        Parser::new(input)?.parse()
    }

    /// Reports whether the expression token stream contains a real component parameter reference.
    ///
    /// This intentionally tokenizes without requiring the whole expression dialect to parse, so
    /// editor-only functions can be preserved while quoted text such as `"param.title"` is ignored.
    pub fn contains_param_reference(input: &str) -> bool {
        Self::probe_param_reference(input).unwrap_or(false)
    }

    pub fn probe_param_reference(input: &str) -> Result<bool, UiBindingExpressionParseError> {
        probe_path_root(input, "param", 1)
    }

    /// Reports whether the token stream contains a real `control.X.prop.Y` reference.
    /// Quoted preview text is ignored even when the full editor expression dialect is unsupported.
    pub fn contains_control_reference(input: &str) -> bool {
        Self::probe_control_reference(input).unwrap_or(false)
    }

    pub fn probe_control_reference(input: &str) -> Result<bool, UiBindingExpressionParseError> {
        probe_path_root(input, "control", 3)
    }
}

fn probe_path_root(
    input: &str,
    root: &str,
    trailing_segments: usize,
) -> Result<bool, UiBindingExpressionParseError> {
    let tokens = tokenize_with_budget(input)?;
    Ok(tokens.iter().enumerate().any(|(index, token)| {
        let is_path_root = index == 0 || !matches!(tokens.get(index - 1), Some(Token::Dot));
        if !is_path_root || !matches!(token, Token::Ident(candidate) if candidate == root) {
            return false;
        }
        (0..trailing_segments).all(|segment| {
            matches!(tokens.get(index + 1 + segment * 2), Some(Token::Dot))
                && matches!(tokens.get(index + 2 + segment * 2), Some(Token::Ident(_)))
        })
    }))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UiBindingExpressionParseError {
    Empty,
    BudgetExceeded { budget: &'static str, limit: usize },
    UnsupportedOperator(String),
    UnexpectedToken(String),
    UnterminatedString,
}

impl fmt::Display for UiBindingExpressionParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("binding expression is empty"),
            Self::BudgetExceeded { budget, limit } => {
                write!(f, "binding expression exceeds {budget} budget of {limit}")
            }
            Self::UnsupportedOperator(operator) => {
                write!(f, "binding expression uses unsupported operator {operator}")
            }
            Self::UnexpectedToken(token) => {
                write!(f, "binding expression has unexpected token {token}")
            }
            Self::UnterminatedString => f.write_str("binding expression has unterminated string"),
        }
    }
}

impl std::error::Error for UiBindingExpressionParseError {}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Ident(String),
    String(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
    Null,
    Dot,
    Comma,
    LeftParen,
    RightParen,
    Equals,
    NotEquals,
    And,
    Or,
    Not,
    Unsupported(String),
}

struct Parser {
    tokens: Vec<Token>,
    index: usize,
}

impl Parser {
    fn new(input: &str) -> Result<Self, UiBindingExpressionParseError> {
        Ok(Self {
            tokens: tokenize_with_budget(input)?,
            index: 0,
        })
    }

    fn parse(mut self) -> Result<UiBindingExpression, UiBindingExpressionParseError> {
        if self.tokens.is_empty() {
            return Err(UiBindingExpressionParseError::Empty);
        }
        let expression = self.parse_or(1)?;
        if let Some(token) = self.peek() {
            return Err(parse_error_from_token(token));
        }
        validate_expression_budget(&expression)?;
        Ok(expression)
    }

    fn parse_or(
        &mut self,
        depth: usize,
    ) -> Result<UiBindingExpression, UiBindingExpressionParseError> {
        ensure_expression_depth(depth)?;
        let mut expression = self.parse_and(depth)?;
        while self.consume(|token| matches!(token, Token::Or)) {
            let rhs = self.parse_and(depth)?;
            expression = UiBindingExpression::Or(Box::new(expression), Box::new(rhs));
        }
        Ok(expression)
    }

    fn parse_and(
        &mut self,
        depth: usize,
    ) -> Result<UiBindingExpression, UiBindingExpressionParseError> {
        let mut expression = self.parse_equality(depth)?;
        while self.consume(|token| matches!(token, Token::And)) {
            let rhs = self.parse_equality(depth)?;
            expression = UiBindingExpression::And(Box::new(expression), Box::new(rhs));
        }
        Ok(expression)
    }

    fn parse_equality(
        &mut self,
        depth: usize,
    ) -> Result<UiBindingExpression, UiBindingExpressionParseError> {
        let mut expression = self.parse_unary(depth)?;
        loop {
            if self.consume(|token| matches!(token, Token::Equals)) {
                let rhs = self.parse_unary(depth)?;
                expression = UiBindingExpression::Equals(Box::new(expression), Box::new(rhs));
            } else if self.consume(|token| matches!(token, Token::NotEquals)) {
                let rhs = self.parse_unary(depth)?;
                expression = UiBindingExpression::NotEquals(Box::new(expression), Box::new(rhs));
            } else {
                break;
            }
        }
        Ok(expression)
    }

    fn parse_unary(
        &mut self,
        depth: usize,
    ) -> Result<UiBindingExpression, UiBindingExpressionParseError> {
        ensure_expression_depth(depth)?;
        if self.consume(|token| matches!(token, Token::Not)) {
            let nested = self.parse_unary(depth + 1)?;
            return Ok(UiBindingExpression::Not(Box::new(nested)));
        }
        self.parse_primary(depth)
    }

    fn parse_primary(
        &mut self,
        depth: usize,
    ) -> Result<UiBindingExpression, UiBindingExpressionParseError> {
        let Some(token) = self.next() else {
            return Err(UiBindingExpressionParseError::Empty);
        };
        match token {
            Token::String(value) => Ok(UiBindingExpression::Literal(UiValue::String(value))),
            Token::Integer(value) => Ok(UiBindingExpression::Literal(UiValue::Int(value))),
            Token::Float(value) => Ok(UiBindingExpression::Literal(UiValue::Float(value))),
            Token::Bool(value) => Ok(UiBindingExpression::Literal(UiValue::Bool(value))),
            Token::Null => Ok(UiBindingExpression::Literal(UiValue::Null)),
            Token::Ident(value) if value == "param" || value == "prop" => {
                self.expect_dot()?;
                let name = self.expect_ident()?;
                if value == "param" {
                    Ok(UiBindingExpression::ParamRef(name))
                } else {
                    Ok(UiBindingExpression::PropRef(name))
                }
            }
            Token::Ident(value) if value == "control" => {
                self.expect_dot()?;
                let control_id = self.expect_ident()?;
                self.expect_dot()?;
                let segment = self.expect_ident()?;
                if segment != "prop" {
                    return Err(UiBindingExpressionParseError::UnexpectedToken(segment));
                }
                self.expect_dot()?;
                let property = self.expect_ident()?;
                Ok(UiBindingExpression::ControlPropRef {
                    control_id,
                    property,
                })
            }
            Token::Ident(value) if is_typed_literal_constructor(&value) => {
                self.parse_typed_literal(&value)
            }
            Token::Ident(value) => Err(UiBindingExpressionParseError::UnexpectedToken(value)),
            Token::LeftParen => {
                let expression = self.parse_or(depth + 1)?;
                if !self.consume(|token| matches!(token, Token::RightParen)) {
                    return Err(UiBindingExpressionParseError::UnexpectedToken(
                        "missing ')'".to_string(),
                    ));
                }
                Ok(expression)
            }
            Token::Unsupported(operator) => {
                Err(UiBindingExpressionParseError::UnsupportedOperator(operator))
            }
            other => Err(parse_error_from_token(&other)),
        }
    }

    fn expect_dot(&mut self) -> Result<(), UiBindingExpressionParseError> {
        if self.consume(|token| matches!(token, Token::Dot)) {
            Ok(())
        } else {
            Err(UiBindingExpressionParseError::UnexpectedToken(
                "expected '.'".to_string(),
            ))
        }
    }

    fn parse_typed_literal(
        &mut self,
        constructor: &str,
    ) -> Result<UiBindingExpression, UiBindingExpressionParseError> {
        if !self.consume(|token| matches!(token, Token::LeftParen)) {
            return Err(UiBindingExpressionParseError::UnexpectedToken(format!(
                "{constructor} requires '('"
            )));
        }

        let value = match constructor {
            "color" => UiValue::Color(self.expect_string_argument(constructor)?),
            "asset_ref" => UiValue::AssetRef(self.expect_string_argument(constructor)?),
            "instance_ref" => UiValue::InstanceRef(self.expect_string_argument(constructor)?),
            "enum" => UiValue::Enum(self.expect_string_argument(constructor)?),
            "vec2" => UiValue::Vec2(self.expect_float_arguments::<2>(constructor)?),
            "vec3" => UiValue::Vec3(self.expect_float_arguments::<3>(constructor)?),
            "vec4" => UiValue::Vec4(self.expect_float_arguments::<4>(constructor)?),
            "flags" => UiValue::Flags(self.expect_string_arguments(constructor)?),
            _ => unreachable!("constructor is guarded by is_typed_literal_constructor"),
        };
        Ok(UiBindingExpression::Literal(value))
    }

    fn expect_string_argument(
        &mut self,
        constructor: &str,
    ) -> Result<String, UiBindingExpressionParseError> {
        let value = match self.next() {
            Some(Token::String(value)) => value,
            Some(token) => return Err(parse_error_from_token(&token)),
            None => {
                return Err(UiBindingExpressionParseError::UnexpectedToken(format!(
                    "{constructor} requires a string argument"
                )));
            }
        };
        self.expect_right_paren(constructor)?;
        Ok(value)
    }

    fn expect_float_arguments<const N: usize>(
        &mut self,
        constructor: &str,
    ) -> Result<[f64; N], UiBindingExpressionParseError> {
        let mut values = Vec::with_capacity(N);
        for index in 0..N {
            if index > 0 && !self.consume(|token| matches!(token, Token::Comma)) {
                return Err(UiBindingExpressionParseError::UnexpectedToken(format!(
                    "{constructor} requires {N} comma-separated numbers"
                )));
            }
            values.push(match self.next() {
                Some(Token::Integer(value)) => value as f64,
                Some(Token::Float(value)) => value,
                Some(token) => return Err(parse_error_from_token(&token)),
                None => {
                    return Err(UiBindingExpressionParseError::UnexpectedToken(format!(
                        "{constructor} requires {N} numbers"
                    )));
                }
            });
        }
        self.expect_right_paren(constructor)?;
        Ok(values
            .try_into()
            .expect("typed vector literal length is fixed by N"))
    }

    fn expect_string_arguments(
        &mut self,
        constructor: &str,
    ) -> Result<Vec<String>, UiBindingExpressionParseError> {
        let mut values = Vec::new();
        if self.consume(|token| matches!(token, Token::RightParen)) {
            return Ok(values);
        }
        loop {
            values.push(match self.next() {
                Some(Token::String(value)) => value,
                Some(token) => return Err(parse_error_from_token(&token)),
                None => {
                    return Err(UiBindingExpressionParseError::UnexpectedToken(format!(
                        "{constructor} requires string arguments"
                    )));
                }
            });
            if self.consume(|token| matches!(token, Token::RightParen)) {
                return Ok(values);
            }
            if !self.consume(|token| matches!(token, Token::Comma)) {
                return Err(UiBindingExpressionParseError::UnexpectedToken(format!(
                    "{constructor} requires comma-separated strings"
                )));
            }
        }
    }

    fn expect_right_paren(
        &mut self,
        constructor: &str,
    ) -> Result<(), UiBindingExpressionParseError> {
        if self.consume(|token| matches!(token, Token::RightParen)) {
            Ok(())
        } else {
            Err(UiBindingExpressionParseError::UnexpectedToken(format!(
                "{constructor} requires ')'"
            )))
        }
    }

    fn expect_ident(&mut self) -> Result<String, UiBindingExpressionParseError> {
        match self.next() {
            Some(Token::Ident(value)) => Ok(value),
            Some(token) => Err(parse_error_from_token(&token)),
            None => Err(UiBindingExpressionParseError::UnexpectedToken(
                "expected identifier".to_string(),
            )),
        }
    }

    fn consume(&mut self, matches: impl FnOnce(&Token) -> bool) -> bool {
        if self.peek().is_some_and(matches) {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.index)
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.index).cloned()?;
        self.index += 1;
        Some(token)
    }
}

fn ensure_expression_depth(depth: usize) -> Result<(), UiBindingExpressionParseError> {
    if depth > UI_BINDING_EXPRESSION_MAX_DEPTH {
        Err(binding_expression_budget_error(
            "depth",
            UI_BINDING_EXPRESSION_MAX_DEPTH,
        ))
    } else {
        Ok(())
    }
}

fn validate_expression_budget(
    root: &UiBindingExpression,
) -> Result<(), UiBindingExpressionParseError> {
    let mut pending = vec![(root, 1usize)];
    let mut node_count = 0usize;
    while let Some((expression, depth)) = pending.pop() {
        ensure_expression_depth(depth)?;
        node_count += 1;
        if node_count > UI_BINDING_EXPRESSION_MAX_NODES {
            return Err(binding_expression_budget_error(
                "nodes",
                UI_BINDING_EXPRESSION_MAX_NODES,
            ));
        }
        match expression {
            UiBindingExpression::Equals(lhs, rhs)
            | UiBindingExpression::NotEquals(lhs, rhs)
            | UiBindingExpression::And(lhs, rhs)
            | UiBindingExpression::Or(lhs, rhs) => {
                pending.push((lhs, depth + 1));
                pending.push((rhs, depth + 1));
            }
            UiBindingExpression::Not(value) => pending.push((value, depth + 1)),
            UiBindingExpression::Literal(_)
            | UiBindingExpression::ParamRef(_)
            | UiBindingExpression::PropRef(_)
            | UiBindingExpression::ControlPropRef { .. } => {}
        }
    }
    Ok(())
}

fn binding_expression_budget_error(
    budget: &'static str,
    limit: usize,
) -> UiBindingExpressionParseError {
    UiBindingExpressionParseError::BudgetExceeded { budget, limit }
}

fn tokenize_with_budget(input: &str) -> Result<Vec<Token>, UiBindingExpressionParseError> {
    if input.len() > UI_BINDING_EXPRESSION_MAX_SOURCE_BYTES {
        return Err(binding_expression_budget_error(
            "source bytes",
            UI_BINDING_EXPRESSION_MAX_SOURCE_BYTES,
        ));
    }
    let tokens = tokenize(input);
    if tokens.len() > UI_BINDING_EXPRESSION_MAX_TOKENS {
        return Err(binding_expression_budget_error(
            "tokens",
            UI_BINDING_EXPRESSION_MAX_TOKENS,
        ));
    }
    Ok(tokens)
}

fn tokenize(input: &str) -> Vec<Token> {
    let trimmed = normalize_expression_input(input);
    let chars = trimmed.chars().collect::<Vec<_>>();
    let mut tokens = Vec::new();
    let mut index = 0usize;
    while index < chars.len() {
        let ch = chars[index];
        if ch.is_whitespace() {
            index += 1;
            continue;
        }
        match ch {
            '.' => {
                tokens.push(Token::Dot);
                index += 1;
            }
            ',' => {
                tokens.push(Token::Comma);
                index += 1;
            }
            '(' => {
                tokens.push(Token::LeftParen);
                index += 1;
            }
            ')' => {
                tokens.push(Token::RightParen);
                index += 1;
            }
            '!' if chars.get(index + 1) == Some(&'=') => {
                tokens.push(Token::NotEquals);
                index += 2;
            }
            '!' => {
                tokens.push(Token::Not);
                index += 1;
            }
            '=' if chars.get(index + 1) == Some(&'=') => {
                tokens.push(Token::Equals);
                index += 2;
            }
            '=' => {
                tokens.push(Token::Unsupported("=".to_string()));
                index += 1;
            }
            '&' if chars.get(index + 1) == Some(&'&') => {
                tokens.push(Token::And);
                index += 2;
            }
            '&' => {
                tokens.push(Token::Unsupported("&".to_string()));
                index += 1;
            }
            '|' if chars.get(index + 1) == Some(&'|') => {
                tokens.push(Token::Or);
                index += 2;
            }
            '|' => {
                tokens.push(Token::Unsupported("|".to_string()));
                index += 1;
            }
            '>' | '<' | '+' | '*' | '/' | '%' => {
                tokens.push(Token::Unsupported(ch.to_string()));
                index += 1;
            }
            '"' | '\'' => match parse_string(&chars, &mut index, ch) {
                Some(value) => tokens.push(Token::String(value)),
                None => {
                    tokens.push(Token::Unsupported("unterminated string".to_string()));
                    break;
                }
            },
            '-' | '0'..='9' => tokens.push(parse_number_or_ident(&chars, &mut index)),
            _ => tokens.push(parse_ident(&chars, &mut index)),
        }
    }
    tokens
}

fn normalize_expression_input(input: &str) -> &str {
    let trimmed = input.trim();
    trimmed
        .strip_prefix('=')
        .map(str::trim_start)
        .unwrap_or(trimmed)
}

fn parse_string(chars: &[char], index: &mut usize, quote: char) -> Option<String> {
    *index += 1;
    let mut value = String::new();
    while *index < chars.len() {
        let ch = chars[*index];
        *index += 1;
        if ch == quote {
            return Some(value);
        }
        if ch == '\\' {
            let escaped = *chars.get(*index)?;
            *index += 1;
            match escaped {
                'n' => value.push('\n'),
                'r' => value.push('\r'),
                't' => value.push('\t'),
                'b' => value.push('\u{0008}'),
                'f' => value.push('\u{000c}'),
                'u' => value.push(parse_unicode_escape(chars, index)?),
                other => value.push(other),
            }
        } else {
            value.push(ch);
        }
    }
    None
}

fn parse_unicode_escape(chars: &[char], index: &mut usize) -> Option<char> {
    let end = index.checked_add(4)?;
    let digits = chars.get(*index..end)?.iter().collect::<String>();
    *index = end;
    u32::from_str_radix(&digits, 16)
        .ok()
        .and_then(char::from_u32)
}

fn is_typed_literal_constructor(value: &str) -> bool {
    matches!(
        value,
        "color" | "asset_ref" | "instance_ref" | "enum" | "vec2" | "vec3" | "vec4" | "flags"
    )
}

fn parse_number_or_ident(chars: &[char], index: &mut usize) -> Token {
    let start = *index;
    if chars[*index] == '-' {
        *index += 1;
    }
    while *index < chars.len() && chars[*index].is_ascii_digit() {
        *index += 1;
    }
    if *index < chars.len() && chars[*index] == '.' {
        *index += 1;
        while *index < chars.len() && chars[*index].is_ascii_digit() {
            *index += 1;
        }
    }
    let text = chars[start..*index].iter().collect::<String>();
    if text == "-" {
        return Token::Unsupported("-".to_string());
    }
    if text.contains('.') {
        text.parse::<f64>()
            .map(Token::Float)
            .unwrap_or(Token::Unsupported(text))
    } else {
        text.parse::<i64>()
            .map(Token::Integer)
            .unwrap_or(Token::Unsupported(text))
    }
}

fn parse_ident(chars: &[char], index: &mut usize) -> Token {
    let start = *index;
    while *index < chars.len()
        && (chars[*index].is_ascii_alphanumeric() || chars[*index] == '_' || chars[*index] == '-')
    {
        *index += 1;
    }
    if start == *index {
        *index += 1;
    }
    let text = chars[start..*index].iter().collect::<String>();
    match text.as_str() {
        "true" => Token::Bool(true),
        "false" => Token::Bool(false),
        "null" => Token::Null,
        _ => Token::Ident(text),
    }
}

fn parse_error_from_token(token: &Token) -> UiBindingExpressionParseError {
    match token {
        Token::Unsupported(value) if value == "unterminated string" => {
            UiBindingExpressionParseError::UnterminatedString
        }
        Token::Unsupported(value) => {
            UiBindingExpressionParseError::UnsupportedOperator(value.clone())
        }
        other => UiBindingExpressionParseError::UnexpectedToken(format!("{other:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_binding_literal_parser_preserves_supported_value_kinds_and_escapes() {
        let cases = [
            (
                r#"color("quote\" slash\\ newline\n return\r tab\t back\b form\f unit\u001f")"#,
                UiValue::Color(
                    "quote\" slash\\ newline\n return\r tab\t back\u{0008} form\u{000c} unit\u{001f}"
                        .to_string(),
                ),
            ),
            (
                r#"asset_ref("asset://ui/status")"#,
                UiValue::AssetRef("asset://ui/status".to_string()),
            ),
            (
                r#"instance_ref("StatusRoot")"#,
                UiValue::InstanceRef("StatusRoot".to_string()),
            ),
            (
                r#"enum("layout.horizontal")"#,
                UiValue::Enum("layout.horizontal".to_string()),
            ),
            ("vec2(1, -2.5)", UiValue::Vec2([1.0, -2.5])),
            ("vec3(1, 2.25, -3)", UiValue::Vec3([1.0, 2.25, -3.0])),
            (
                "vec4(0, 0.5, 1, -4.75)",
                UiValue::Vec4([0.0, 0.5, 1.0, -4.75]),
            ),
            (
                r#"flags("read", "write")"#,
                UiValue::Flags(vec!["read".to_string(), "write".to_string()]),
            ),
            ("flags()", UiValue::Flags(Vec::new())),
        ];

        for (source, value) in cases {
            assert_eq!(
                UiBindingExpression::parse(source).unwrap(),
                UiBindingExpression::Literal(value),
                "typed literal source: {source}"
            );
        }
    }

    #[test]
    fn typed_binding_literal_param_probe_requires_a_path_root() {
        assert!(UiBindingExpression::contains_param_reference("param.title"));
        assert!(UiBindingExpression::contains_param_reference(
            "=concat(param.title, prop.text)"
        ));
        assert!(!UiBindingExpression::contains_param_reference(
            r#"=concat("param.title", prop.text)"#
        ));
        assert!(!UiBindingExpression::contains_param_reference(
            "control.param.prop.value"
        ));
    }

    #[test]
    fn control_reference_probe_ignores_quoted_preview_text() {
        assert!(UiBindingExpression::contains_control_reference(
            "control.Value.prop.text"
        ));
        assert!(UiBindingExpression::contains_control_reference(
            "=control.Value.prop.text == \"Ready\""
        ));
        assert!(!UiBindingExpression::contains_control_reference(
            r#"=concat("control.Value.prop.text", prop.text)"#
        ));
        assert!(!UiBindingExpression::contains_control_reference(
            "model.control.Value.prop.text"
        ));
    }

    #[test]
    fn binding_expression_parse_budgets_reject_oversized_or_deep_input() {
        let oversized = "x".repeat(UI_BINDING_EXPRESSION_MAX_SOURCE_BYTES + 1);
        assert_eq!(
            UiBindingExpression::parse(&oversized),
            Err(UiBindingExpressionParseError::BudgetExceeded {
                budget: "source bytes",
                limit: UI_BINDING_EXPRESSION_MAX_SOURCE_BYTES,
            })
        );
        assert_eq!(
            UiBindingExpression::probe_param_reference(&oversized),
            Err(UiBindingExpressionParseError::BudgetExceeded {
                budget: "source bytes",
                limit: UI_BINDING_EXPRESSION_MAX_SOURCE_BYTES,
            })
        );
        assert_eq!(
            UiBindingExpression::probe_control_reference(&oversized),
            Err(UiBindingExpressionParseError::BudgetExceeded {
                budget: "source bytes",
                limit: UI_BINDING_EXPRESSION_MAX_SOURCE_BYTES,
            })
        );

        let excessive_tokens = format!("{}true", "!".repeat(UI_BINDING_EXPRESSION_MAX_TOKENS));
        assert_eq!(
            UiBindingExpression::parse(&excessive_tokens),
            Err(UiBindingExpressionParseError::BudgetExceeded {
                budget: "tokens",
                limit: UI_BINDING_EXPRESSION_MAX_TOKENS,
            })
        );

        let excessive_depth = format!("{}true", "!".repeat(UI_BINDING_EXPRESSION_MAX_DEPTH));
        assert_eq!(
            UiBindingExpression::parse(&excessive_depth),
            Err(UiBindingExpressionParseError::BudgetExceeded {
                budget: "depth",
                limit: UI_BINDING_EXPRESSION_MAX_DEPTH,
            })
        );
    }
}
