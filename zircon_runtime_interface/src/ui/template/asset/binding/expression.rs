//! 可序列化的绑定表达式语法树和有预算的文本解析器；资产验证/编译据此检查引用，运行时求值器再取值。
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::ui::component::UiValue;

#[path = "expression/evaluator.rs"]
mod evaluator;

pub use evaluator::UiBindingExpressionEvaluationError;

/// 最大 UTF-8 源文本字节数；在字符向量和 token 分配前检查。
pub const UI_BINDING_EXPRESSION_MAX_SOURCE_BYTES: usize = 16 * 1024;
/// 单次输入允许的最大词法 token 数。
pub const UI_BINDING_EXPRESSION_MAX_TOKENS: usize = 2_048;
/// AST 解析完成和运行时求值共同遵守的节点预算。
pub const UI_BINDING_EXPRESSION_MAX_NODES: usize = 1_024;
/// 递归解析和显式栈求值的最大表达式深度。
pub const UI_BINDING_EXPRESSION_MAX_DEPTH: usize = 64;
/// 求值帧和值栈在转用溢出 Vec 前保留的内联槽数。
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
    /// 跨控件只读当前树中 descriptor-owned 属性，编译前仍需解析 control id 和属性名。
    ControlPropRef {
        control_id: String,
        property: String,
    },
    /// 两侧值相等时为 true；解析器按同级左结合构造。
    Equals(Box<UiBindingExpression>, Box<UiBindingExpression>),
    /// 两侧值不等时为 true；解析器按同级左结合构造。
    NotEquals(Box<UiBindingExpression>, Box<UiBindingExpression>),
    /// 两侧都为布尔值且为 true 时成立；求值器短路 false 左侧。
    And(Box<UiBindingExpression>, Box<UiBindingExpression>),
    /// 任一侧布尔值为 true 即成立；求值器短路 true 左侧。
    Or(Box<UiBindingExpression>, Box<UiBindingExpression>),
    /// 对布尔子表达式取反；非布尔结果由求值器作为类型错误返回。
    Not(Box<UiBindingExpression>),
}

impl UiBindingExpression {
    /// 解析绑定表达式子集：`param`、`prop`、`control.X.prop.Y`、类型化字面量，以及逻辑非、与/或和相等/不等运算。
    /// 开头的 `=` 是资产表达式标记；源码字节、token、节点和嵌套深度均受预算限制。
    ///
    /// # 参数
    /// `input` 是资产中的表达式文本；前后空白与单个前导 `=` 按输入规范处理。
    ///
    /// # 返回值
    /// 返回供 Runtime binding compiler 继续编译的 AST；词法、语法、类型化字面量或预算错误均以 [`UiBindingExpressionParseError`] 返回。
    /// 返回 AST 只保证属于本解析器支持的语法，不会在解析阶段解析控件目标或读取运行时值。
    pub fn parse(input: &str) -> Result<Self, UiBindingExpressionParseError> {
        Parser::new(input)?.parse()
    }

    /// Reports whether the expression token stream contains a real component parameter reference.
    ///
    /// This intentionally tokenizes without requiring the whole expression dialect to parse, so
    /// editor-only functions can be preserved while quoted text such as `"param.title"` is ignored.
    /// 只探测 token 流中的独立 `param.<name>` 根，不要求整条表达式可解析；token 化失败时返回 false，且不证明参数已声明。
    pub fn contains_param_reference(input: &str) -> bool {
        Self::probe_param_reference(input).unwrap_or(false)
    }

    /// 只探测 token 中独立的 `param.<name>` 路径根；保留 token 化错误供验证器生成诊断。
    /// 探测不完整解析整条表达式，也不查询参数声明。
    ///
    /// # 参数
    /// `input` 是资产表达式文本；探测会按 parser 共享的规则移除首尾空白和可选前导 `=`。
    ///
    /// # 返回值
    /// `Ok(true/false)` 表示 token 流中是否找到该路径根；词法或源码/token 预算错误以 [`UiBindingExpressionParseError`] 返回。
    /// Runtime `binding_param_resolver::probe_param_reference` 将结果用于预筛，完整语法和参数解析仍由后续 binding compiler 处理。
    pub fn probe_param_reference(input: &str) -> Result<bool, UiBindingExpressionParseError> {
        probe_path_root(input, "param", 1)
    }

    /// Reports whether the token stream contains a real `control.X.prop.Y` reference.
    /// Quoted preview text is ignored even when the full editor expression dialect is unsupported.
    /// 只按词法路径根作布尔探测，不完整解析编辑器表达式；token 化失败时返回 false，不能据此证明目标属性存在。
    pub fn contains_control_reference(input: &str) -> bool {
        Self::probe_control_reference(input).unwrap_or(false)
    }

    /// 只探测形如 `control.<id>.prop.<name>` 的路径，不要求编辑器预览表达式整体可被运行时语法解析。
    ///
    /// # 实现边界
    /// 当前预筛只检查根后三个 `.标识符` 段，不校验中间段为 `prop`、控件是否存在或字段是否有效；契约确认事项见 `ZCA-TPL05-TODO-001`。
    /// 编辑器预览可含 Runtime 不支持的其他语法；返回 `Ok(true)` 只表示发现词法模式，完整合法性由后续解析和控件契约检查决定。
    ///
    /// # 参数
    /// `input` 是资产或编辑器预览表达式文本；探测遵守 parser 的源码字节和 token 预算。
    ///
    /// # 返回值
    /// `Ok(true/false)` 表示是否发现上述词法模式；token 化或预算错误以 [`UiBindingExpressionParseError`] 返回。
    /// Runtime `control_scope::probe_control_reference` 用命中结果预筛，之后仍须完整解析并按组件 scope 限定引用。
    pub fn probe_control_reference(input: &str) -> Result<bool, UiBindingExpressionParseError> {
        // TODO: [ZCA-TPL05-TODO-001] 确认预筛是否有意接受 `control.a.foo.b`：这里只检查根后 3 个 Dot+Ident，不要求中间段为 `prop`；scope 调用端会完整解析命中项。先核对契约并验证该路径、`control.a.prop.b` 与带引号文本，再决定是否收窄。
        probe_path_root(input, "control", 3)
    }
}

/// 在有 token 预算的输入中寻找独立路径根，并检查后续点分段数量。
/// 返回 token 化错误供验证器产生诊断；不解析或验证路径目标是否存在。
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

/// 为资产诊断提供稳定的人类可读语法/预算错误文本。
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
/// 解析器内部的词法单元；Unsupported 延迟表达式子集错误，Consumed 标记已移出的槽位。
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
    /// next 移出非 Copy token 后留下的哨兵，不会进入未消费 token 范围。
    Consumed,
}

/// 对有界 token 向量维护单调游标，并按优先级逐层构造 AST。
struct Parser {
    /// 已由源字节和 token 上限约束的词法结果。
    tokens: Vec<Token>,
    /// 下一个待消费 token 的位置；成功消费后只向前移动。
    index: usize,
}

impl Parser {
    /// 先完成有预算 token 化，再建立位于首 token 的游标。
    fn new(input: &str) -> Result<Self, UiBindingExpressionParseError> {
        Ok(Self {
            tokens: tokenize_with_budget(input)?,
            index: 0,
        })
    }

    /// 构造完整表达式，拒绝尾随 token，最后迭代检查 AST 节点数和深度。
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

    /// 在 parse_equality 之上折叠 `&&`，形成比 `||` 更紧的左结合层。
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

    /// 解析相等/不等链；关系运算与调用方传入的下一级优先级分离。
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

    /// 处理一元 `!` 并逐层检查递归深度，再交给 primary 解析。
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

    /// 读取字面量、限定路径、类型化构造器或括号表达式；其余标识符按不支持语法报错。
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

    /// 读取受支持的 `color`、资源/实例引用、enum、向量和 flags 构造值。
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

    /// 读取恰好一个字符串参数并消费构造器右括号。
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
        for (index, slot) in values.iter_mut().enumerate() {
            if index > 0 && !self.consume(|token| matches!(token, Token::Comma)) {
                return Err(UiBindingExpressionParseError::UnexpectedToken(format!(
                    "{constructor} requires {N} comma-separated numbers"
                )));
            }
            *slot = match self.next() {
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
    /// 测试基线：以临时 Vec 收集同一组向量参数后转数组。
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

    /// 读取零个或多个逗号分隔字符串，供 flags 构造器使用。
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

    /// 消费构造器终止符，阻止额外参数被静默忽略。
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
    /// 测试对照实现：克隆当前 token 后推进游标，用于与移动读取比较。
    fn next_cloned(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.index).cloned()?;
        self.index += 1;
        Some(token)
    }
}

/// 检查当前 1-based 深度，超出统一常量时返回结构化预算错误。
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

/// 将解析器各阶段的预算名和上限统一编码为公开错误变体。
fn binding_expression_budget_error(
    budget: &'static str,
    limit: usize,
) -> UiBindingExpressionParseError {
    UiBindingExpressionParseError::BudgetExceeded { budget, limit }
}

// 先按 UTF-8 源字节拒绝超限输入，再开始字符和 token 分配；tokenizer 在每个新增 token 后提前止损。
/// 先按 UTF-8 字节上限拒绝输入，再调用带 token 上限的词法器。
fn tokenize_with_budget(input: &str) -> Result<Vec<Token>, UiBindingExpressionParseError> {
    if input.len() > UI_BINDING_EXPRESSION_MAX_SOURCE_BYTES {
        return Err(binding_expression_budget_error(
            "source bytes",
            UI_BINDING_EXPRESSION_MAX_SOURCE_BYTES,
        ));
    }
    tokenize(input, UI_BINDING_EXPRESSION_MAX_TOKENS)
}

/// 将表达式子集切成有序 token；遇到预算越界立即返回，不交付部分 token 列表。
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
/// 测试对照路径：完整扫描后再比较 token 上限，用来验证生产词法器提前止损。
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
/// 去掉首尾空白和可选单个 `=` 标记，供 parser 与 reference probe 共用。
fn normalize_expression_input(input: &str) -> &str {
    let trimmed = input.trim();
    trimmed
        .strip_prefix('=')
        .map(str::trim_start)
        .unwrap_or(trimmed)
}

/// 从已消费开引号的位置读取字符串和转义；不闭合或转义不完整时返回 None。
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
/// 从游标读取四位 ASCII 十六进制转义；代理项和无效码点不会转成 char。
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
/// 测试对照实现：先收集四位字符到 String，再解析相同 Unicode 标量值。
fn parse_unicode_escape_allocating(chars: &[char], index: &mut usize) -> Option<char> {
    let end = index.checked_add(4)?;
    let digits = chars.get(*index..end)?.iter().collect::<String>();
    *index = end;
    u32::from_str_radix(&digits, 16)
        .ok()
        .and_then(char::from_u32)
}

/// 判断标识符是否属于 parser 可直接构造 UiValue 的固定名称集合。
fn is_typed_literal_constructor(value: &str) -> bool {
    matches!(
        value,
        "color" | "asset_ref" | "instance_ref" | "enum" | "vec2" | "vec3" | "vec4" | "flags"
    )
}

/// 扫描负号/数字前缀；溢出或格式不支持的数值保留为 Unsupported token。
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
/// 不分配中间字符串地累积有符号整数；checked 算术覆盖 i64 边界并拒绝溢出。
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
/// 测试对照实现：将数值字符先分配成 String，再委托标准解析器分类。
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
/// 扫描 ASCII 标识符并就地识别 true、false、null，其他拼写保留为 Ident。
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
/// 测试对照实现：先分配标识符 String，再按相同关键字规则分类。
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

/// 把词法阶段保留的 Unsupported 或普通 token 映射成公开 parse error。
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
#[path = "../../../asset/binding/tests/integer_token_perf.rs"]
mod integer_token_performance_tests;

#[cfg(test)]
#[path = "../../../asset/binding/tests/parser_token_perf.rs"]
mod parser_token_performance_tests;

#[cfg(test)]
#[path = "tests/expression.rs"]
mod tests;
