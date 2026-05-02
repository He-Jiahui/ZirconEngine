use std::fmt;

use serde::{Deserialize, Serialize};

use crate::ui::component::UiValue;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UiBindingExpression {
    Literal(UiValue),
    ParamRef(String),
    PropRef(String),
    Equals(Box<UiBindingExpression>, Box<UiBindingExpression>),
    NotEquals(Box<UiBindingExpression>, Box<UiBindingExpression>),
    And(Box<UiBindingExpression>, Box<UiBindingExpression>),
    Or(Box<UiBindingExpression>, Box<UiBindingExpression>),
    Not(Box<UiBindingExpression>),
}

impl UiBindingExpression {
    pub fn parse(input: &str) -> Result<Self, UiBindingExpressionParseError> {
        Parser::new(input).parse()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UiBindingExpressionParseError {
    Empty,
    UnsupportedOperator(String),
    UnexpectedToken(String),
    UnterminatedString,
}

impl fmt::Display for UiBindingExpressionParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("binding expression is empty"),
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
    fn new(input: &str) -> Self {
        Self {
            tokens: tokenize(input),
            index: 0,
        }
    }

    fn parse(mut self) -> Result<UiBindingExpression, UiBindingExpressionParseError> {
        if self.tokens.is_empty() {
            return Err(UiBindingExpressionParseError::Empty);
        }
        let expression = self.parse_or()?;
        if let Some(token) = self.peek() {
            return Err(parse_error_from_token(token));
        }
        Ok(expression)
    }

    fn parse_or(&mut self) -> Result<UiBindingExpression, UiBindingExpressionParseError> {
        let mut expression = self.parse_and()?;
        while self.consume(|token| matches!(token, Token::Or)) {
            let rhs = self.parse_and()?;
            expression = UiBindingExpression::Or(Box::new(expression), Box::new(rhs));
        }
        Ok(expression)
    }

    fn parse_and(&mut self) -> Result<UiBindingExpression, UiBindingExpressionParseError> {
        let mut expression = self.parse_equality()?;
        while self.consume(|token| matches!(token, Token::And)) {
            let rhs = self.parse_equality()?;
            expression = UiBindingExpression::And(Box::new(expression), Box::new(rhs));
        }
        Ok(expression)
    }

    fn parse_equality(&mut self) -> Result<UiBindingExpression, UiBindingExpressionParseError> {
        let mut expression = self.parse_unary()?;
        loop {
            if self.consume(|token| matches!(token, Token::Equals)) {
                let rhs = self.parse_unary()?;
                expression = UiBindingExpression::Equals(Box::new(expression), Box::new(rhs));
            } else if self.consume(|token| matches!(token, Token::NotEquals)) {
                let rhs = self.parse_unary()?;
                expression = UiBindingExpression::NotEquals(Box::new(expression), Box::new(rhs));
            } else {
                break;
            }
        }
        Ok(expression)
    }

    fn parse_unary(&mut self) -> Result<UiBindingExpression, UiBindingExpressionParseError> {
        if self.consume(|token| matches!(token, Token::Not)) {
            let nested = self.parse_unary()?;
            return Ok(UiBindingExpression::Not(Box::new(nested)));
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<UiBindingExpression, UiBindingExpressionParseError> {
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
            Token::Ident(value) => Err(UiBindingExpressionParseError::UnexpectedToken(value)),
            Token::LeftParen => {
                let expression = self.parse_or()?;
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
            value.push(match escaped {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                other => other,
            });
        } else {
            value.push(ch);
        }
    }
    None
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
