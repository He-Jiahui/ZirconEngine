//! UI 模板选择器的数据结构、受限解析和级联优先级；运行时样式编译解析规则文本，再按树路径匹配和应用。
use serde::{Deserialize, Serialize};

use crate::ui::template::UiAssetError;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 按从左到右的 compound selector 序列保存路径选择器；每段的 combinator 描述它与前一段的关系。
pub struct UiSelector {
    pub segments: Vec<UiSelectorSegment>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 一个节点上的复合条件；tokens 必须同时匹配该节点，combinator 连接前一段与本段。
pub struct UiSelectorSegment {
    pub combinator: Option<UiSelectorCombinator>,
    pub tokens: Vec<UiSelectorToken>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 段间关系：Descendant 可跨越祖先节点，Child 只匹配直接父子关系。
pub enum UiSelectorCombinator {
    Descendant,
    Child,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 复合选择器解析出的条件 token：类型/通配符、class、id、state、part 或 host；`:part` 供公共部件契约校验但不匹配样式，`:host` 按节点的 host 标记匹配。
pub enum UiSelectorToken {
    Type(String),
    Class(String),
    Id(String),
    State(String),
    Part(String),
    Host,
}

/// CSS/USS cascade precedence ordered by ID, class-like selectors, and types.
///
/// This intentionally stays as a tuple rather than a weighted integer: any
/// number of class-like selectors must not tie an ID selector.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UiSelectorSpecificity {
    id_count: usize,
    class_like_count: usize,
    type_count: usize,
}

/// 选择器级联按 ID、class-like、type 的三元组词典序比较；旧显示分值不参与这个比较。
impl UiSelectorSpecificity {
    pub const fn new(id_count: usize, class_like_count: usize, type_count: usize) -> Self {
        Self {
            id_count,
            class_like_count,
            type_count,
        }
    }

    /// Returns the historical inspector display score, never cascade order.
    pub const fn legacy_display_score(self) -> usize {
        const ID_WEIGHT: usize = 100;
        const CLASS_LIKE_WEIGHT: usize = 10;

        self.id_count * ID_WEIGHT + self.class_like_count * CLASS_LIKE_WEIGHT + self.type_count
    }
}

impl UiSelector {
    /// 解析受支持的类型、通配符、class、id、state、`:part(...)`、`:host` 及空白/`>` 组合器；拒绝空段和尾随组合器。
    pub fn parse(input: &str) -> Result<Self, UiAssetError> {
        let mut remaining = input;
        let mut segments = Vec::new();
        let mut combinator = None;

        loop {
            (remaining, _) = split_whitespace_prefix(remaining);
            if remaining.is_empty() {
                break;
            }

            let compound_end = remaining
                .char_indices()
                .find_map(|(offset, character)| {
                    (character.is_whitespace() || character == '>').then_some(offset)
                })
                .unwrap_or(remaining.len());
            let compound = &remaining[..compound_end];
            if compound.is_empty() {
                return Err(UiAssetError::InvalidSelector(input.to_string()));
            }

            segments.push(UiSelectorSegment {
                combinator,
                tokens: parse_compound_tokens(compound)?,
            });

            remaining = &remaining[compound_end..];
            let saw_space;
            (remaining, saw_space) = split_whitespace_prefix(remaining);
            // 空白表示任意祖先关系，显式 `>` 表示直接父子；关系挂在右侧段上供路径匹配器回溯。
            combinator = match remaining.chars().next() {
                Some('>') => {
                    remaining = &remaining['>'.len_utf8()..];
                    Some(UiSelectorCombinator::Child)
                }
                Some(_) if saw_space => Some(UiSelectorCombinator::Descendant),
                Some(_) => {
                    return Err(UiAssetError::InvalidSelector(format!(
                        "{input}: expected whitespace or '>' between selector compounds"
                    )));
                }
                None => None,
            };
        }

        if segments.is_empty() {
            return Err(UiAssetError::InvalidSelector(input.to_string()));
        }

        if combinator.is_some() {
            return Err(UiAssetError::InvalidSelector(input.to_string()));
        }

        Ok(Self { segments })
    }

    /// 汇总全路径 token 的三元组优先级；通配符不加类型权重，state/part/host 计入 class-like。
    pub fn specificity(&self) -> UiSelectorSpecificity {
        let mut specificity = UiSelectorSpecificity::default();
        self.segments
            .iter()
            .flat_map(|segment| segment.tokens.iter())
            .for_each(|token| match token {
                UiSelectorToken::Id(_) => specificity.id_count += 1,
                UiSelectorToken::Class(_)
                | UiSelectorToken::State(_)
                | UiSelectorToken::Part(_)
                | UiSelectorToken::Host => specificity.class_like_count += 1,
                UiSelectorToken::Type(type_name) if type_name != "*" => specificity.type_count += 1,
                UiSelectorToken::Type(_) => {}
            });
        specificity
    }
}

// 每个点号、井号或冒号最多开启一个 token，再加可能的前置类型名；先预留上界避免解析长规则反复扩容。
fn parse_compound_tokens(input: &str) -> Result<Vec<UiSelectorToken>, UiAssetError> {
    let mut index = 0;
    let delimiter_count = input
        .bytes()
        .filter(|byte| matches!(byte, b'.' | b'#' | b':'))
        .count();
    let has_leading_type = input
        .as_bytes()
        .first()
        .is_some_and(|byte| !matches!(byte, b'.' | b'#' | b':'));
    let token_capacity = delimiter_count + usize::from(has_leading_type);
    let mut tokens = Vec::with_capacity(token_capacity);

    while index < input.len() {
        let prefix = input[index..]
            .chars()
            .next()
            .expect("selector index remains on a character boundary");
        let start = if matches!(prefix, '.' | '#' | ':') {
            index + prefix.len_utf8()
        } else {
            index
        };
        let end = input[start..]
            .char_indices()
            .find_map(|(offset, character)| {
                matches!(character, '.' | '#' | ':').then_some(start + offset)
            })
            .unwrap_or(input.len());
        let value = &input[start..end];
        if value.is_empty() {
            return Err(UiAssetError::InvalidSelector(input.to_string()));
        }

        match prefix {
            '.' => tokens.push(UiSelectorToken::Class(value.to_string())),
            '#' => tokens.push(UiSelectorToken::Id(value.to_string())),
            ':' if value == "host" => tokens.push(UiSelectorToken::Host),
            ':' if value.starts_with("part(") && value.ends_with(')') => {
                let part = value
                    .strip_prefix("part(")
                    .and_then(|value| value.strip_suffix(')'))
                    .unwrap_or_default();
                if part.is_empty() {
                    return Err(UiAssetError::InvalidSelector(input.to_string()));
                }
                tokens.push(UiSelectorToken::Part(part.to_string()));
            }
            ':' => tokens.push(UiSelectorToken::State(value.to_string())),
            _ => tokens.push(UiSelectorToken::Type(value.to_string())),
        }

        index = end;
    }

    Ok(tokens)
}

fn split_whitespace_prefix(input: &str) -> (&str, bool) {
    let remaining = input.trim_start_matches(char::is_whitespace);
    (remaining, remaining.len() != input.len())
}

#[cfg(test)]
fn skip_whitespace(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> bool {
    let mut saw_whitespace = false;
    while chars.peek().is_some_and(|ch| ch.is_whitespace()) {
        saw_whitespace = true;
        let _ = chars.next();
    }
    saw_whitespace
}

#[cfg(test)]
#[path = "style/tests/selector_token_performance_tests.rs"]
mod selector_token_performance_tests;

#[cfg(test)]
#[path = "style/tests/selector_parse_performance_tests.rs"]
mod selector_parse_performance_tests;

#[cfg(test)]
#[path = "style/tests/selector_capacity_performance_tests.rs"]
mod selector_capacity_performance_tests;
