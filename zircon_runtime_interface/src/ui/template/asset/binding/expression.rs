//! 可序列化的绑定表达式语法树和有预算的文本解析器；资产验证/编译据此检查引用，运行时求值器再取值。
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
/// 绑定表达式的语法树：保存字面量、参数/属性引用、同树控制属性引用及其比较与逻辑组合。
/// 引用是否存在以及目标类型由组件描述符驱动的资产验证和编译阶段确认。
pub enum UiBindingExpression {
    /// 不再依赖外部状态的常量值。
    Literal(UiValue),
    /// 当前组件实例参数；编译器按组件参数表替换或解析该名称。
    ParamRef(String),
    /// 当前控件属性；编译器将名称解析为编译程序的属性索引。
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
    /// 解析绑定表达式子集：`param`、`prop`、`control.X.prop.Y`、类型化字面量，以及逻辑非、与/或和相等/不等运算。
    /// 开头的 `=` 是资产表达式标记；源码字节、token、节点和嵌套深度均受预算限制。
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

    /// 只探测 token 中独立的 `param.<name>` 路径根；保留 token 化错误供验证器生成诊断。
    pub fn probe_param_reference(input: &str) -> Result<bool, UiBindingExpressionParseError> {
        probe_path_root(input, "param", 1)
    }

    /// Reports whether the token stream contains a real `control.X.prop.Y` reference.
    /// Quoted preview text is ignored even when the full editor expression dialect is unsupported.
    pub fn contains_control_reference(input: &str) -> bool {
        Self::probe_control_reference(input).unwrap_or(false)
    }

    /// 只探测形如 `control.<id>.prop.<name>` 的路径，不要求编辑器预览表达式整体可被运行时语法解析。
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
/// 文本语法、token 化或输入预算错误；此处不表示参数/属性引用已经通过资产契约解析。
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
    Consumed,
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

    // 分层入口给出优先级：一元非、相等比较、&&、||；各层循环按左结合构造同级表达式。
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
            // 跨控件读取只接受 descriptor-owned prop 路径，不能把 state 等内部访问器带入表达式 AST。
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

    // vec2/vec3/vec4 的长度由构造子固定；直接写入定长数组，避免临时堆分配后再转数组。
    fn expect_float_arguments<const N: usize>(
        &mut self,
        constructor: &str,
    ) -> Result<[f64; N], UiBindingExpressionParseError> {
        let mut values = [0.0; N];
        for index in 0..N {
            if index > 0 && !self.consume(|token| matches!(token, Token::Comma)) {
                return Err(UiBindingExpressionParseError::UnexpectedToken(format!(
                    "{constructor} requires {N} comma-separated numbers"
                )));
            }
            values[index] = match self.next() {
                Some(Token::Integer(value)) => value as f64,
                Some(Token::Float(value)) => value,
                Some(token) => return Err(parse_error_from_token(&token)),
                None => {
                    return Err(UiBindingExpressionParseError::UnexpectedToken(format!(
                        "{constructor} requires {N} numbers"
                    )));
                }
            };
        }
        self.expect_right_paren(constructor)?;
        Ok(values)
    }

    #[cfg(test)]
    fn expect_float_arguments_allocating<const N: usize>(
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

    // token 内可能持有长字符串；推进索引后移出旧值，避免克隆，再用 sentinel 占住已消费槽位。
    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get_mut(self.index)?;
        self.index += 1;
        Some(std::mem::replace(token, Token::Consumed))
    }

    #[cfg(test)]
    fn next_cloned(&mut self) -> Option<Token> {
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

// 迭代遍历同时限制总节点数和树深；平坦布尔链可能很浅但节点很多，不能只依赖递归深度检查。
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

// 先按 UTF-8 源字节拒绝超限输入，再开始字符和 token 分配；tokenizer 在每个新增 token 后提前止损。
fn tokenize_with_budget(input: &str) -> Result<Vec<Token>, UiBindingExpressionParseError> {
    if input.len() > UI_BINDING_EXPRESSION_MAX_SOURCE_BYTES {
        return Err(binding_expression_budget_error(
            "source bytes",
            UI_BINDING_EXPRESSION_MAX_SOURCE_BYTES,
        ));
    }
    tokenize(input, UI_BINDING_EXPRESSION_MAX_TOKENS)
}

fn tokenize(input: &str, max_tokens: usize) -> Result<Vec<Token>, UiBindingExpressionParseError> {
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
                    index = chars.len();
                }
            },
            '-' | '0'..='9' => tokens.push(parse_number_or_ident(&chars, &mut index)),
            _ => tokens.push(parse_ident(&chars, &mut index)),
        }
        if tokens.len() > max_tokens {
            return Err(binding_expression_budget_error("tokens", max_tokens));
        }
    }
    Ok(tokens)
}

#[cfg(test)]
fn tokenize_with_budget_unbounded_scan(
    input: &str,
) -> Result<Vec<Token>, UiBindingExpressionParseError> {
    if input.len() > UI_BINDING_EXPRESSION_MAX_SOURCE_BYTES {
        return Err(binding_expression_budget_error(
            "source bytes",
            UI_BINDING_EXPRESSION_MAX_SOURCE_BYTES,
        ));
    }
    let tokens = tokenize(input, usize::MAX)?;
    if tokens.len() > UI_BINDING_EXPRESSION_MAX_TOKENS {
        return Err(binding_expression_budget_error(
            "tokens",
            UI_BINDING_EXPRESSION_MAX_TOKENS,
        ));
    }
    Ok(tokens)
}

// 资产中的 `=...` 表示动态表达式；去掉首尾空白和单个前缀标记后才进入同一套语法。
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

// Unicode 转义固定读取四个 ASCII 十六进制字符到栈上缓冲；无效码点（含代理项）直接作为字符串错误处理。
fn parse_unicode_escape(chars: &[char], index: &mut usize) -> Option<char> {
    let end = index.checked_add(4)?;
    let source = chars.get(*index..end)?;
    *index = end;
    let mut digits = [0_u8; 4];
    for (byte, digit) in digits.iter_mut().zip(source) {
        if !digit.is_ascii() {
            return None;
        }
        *byte = *digit as u8;
    }
    let digits = std::str::from_utf8(&digits).ok()?;
    u32::from_str_radix(digits, 16)
        .ok()
        .and_then(char::from_u32)
}

#[cfg(test)]
fn parse_unicode_escape_allocating(chars: &[char], index: &mut usize) -> Option<char> {
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
    let has_fraction = *index < chars.len() && chars[*index] == '.';
    if has_fraction {
        *index += 1;
        while *index < chars.len() && chars[*index].is_ascii_digit() {
            *index += 1;
        }
    }
    let number = &chars[start..*index];
    if !has_fraction {
        if let Some(value) = parse_i64_chars(number) {
            return Token::Integer(value);
        }
    }
    let text = chars[start..*index].iter().collect::<String>();
    if text == "-" {
        return Token::Unsupported("-".to_string());
    }
    if has_fraction {
        text.parse::<f64>()
            .map(Token::Float)
            .unwrap_or(Token::Unsupported(text))
    } else {
        Token::Unsupported(text)
    }
}

// 逐位用 checked 算术解析整数，负数从零向下累积以覆盖 i64::MIN，且不先构造临时字符串。
fn parse_i64_chars(chars: &[char]) -> Option<i64> {
    let (negative, digits) = match chars {
        ['-', digits @ ..] => (true, digits),
        digits => (false, digits),
    };
    if digits.is_empty() {
        return None;
    }

    let mut value = 0_i64;
    for digit in digits {
        let digit = i64::from(digit.to_digit(10)?);
        value = if negative {
            value.checked_mul(10)?.checked_sub(digit)?
        } else {
            value.checked_mul(10)?.checked_add(digit)?
        };
    }
    Some(value)
}

#[cfg(test)]
fn parse_number_or_ident_allocating(chars: &[char], index: &mut usize) -> Token {
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

// 关键字直接匹配字符切片可避免为 true/false/null 分配 String；普通标识符仍保留原拼写。
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
    match &chars[start..*index] {
        ['t', 'r', 'u', 'e'] => Token::Bool(true),
        ['f', 'a', 'l', 's', 'e'] => Token::Bool(false),
        ['n', 'u', 'l', 'l'] => Token::Null,
        identifier => Token::Ident(identifier.iter().collect::<String>()),
    }
}

#[cfg(test)]
fn parse_ident_allocating(chars: &[char], index: &mut usize) -> Token {
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
#[path = "expression/tests/integer_token_performance_tests.rs"]
mod integer_token_performance_tests;

#[cfg(test)]
#[path = "expression/tests/parser_token_performance_tests.rs"]
mod parser_token_performance_tests;

#[cfg(test)]
#[path = "tests/expression.rs"]
mod tests;
