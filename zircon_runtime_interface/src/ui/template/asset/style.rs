//! UI 模板选择器的数据结构、受限解析和级联优先级；运行时样式编译解析规则文本，再按树路径匹配和应用。
use serde::{Deserialize, Serialize};

use crate::ui::template::UiAssetError;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 按从左到右的 compound selector 序列保存路径选择器；每段的 combinator 描述它与前一段的关系。
/// 文本语法是 `parse` 实现的有限子集，不等同于完整 CSS；公开字段和反序列化可构造不满足该语法不变量的路径。
/// 对外部文本应先调用 `parse`，Runtime 样式匹配器再消费解析后的段和 token。
pub struct UiSelector {
    /// 从最左祖先到目标节点的复合选择器路径。
    pub segments: Vec<UiSelectorSegment>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 一个节点上的复合条件；tokens 必须同时匹配该节点，combinator 连接前一段与本段。
pub struct UiSelectorSegment {
    /// 本段相对前一段的关系；首段为 None。
    pub combinator: Option<UiSelectorCombinator>,
    /// 必须同时在本段节点满足的选择条件。
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
    /// Widget/组件类型名，`*` 表示通配类型。
    Type(String),
    /// 节点 class 列表中的名称。
    Class(String),
    /// 节点的 control id。
    Id(String),
    /// Runtime selector path 暴露的伪状态名。
    State(String),
    /// 组件公开部件名；由组件契约验证引用，不参与通用节点样式匹配。
    /// Runtime `zircon_runtime::ui::template::asset::style::matches_token` 对该 token 返回 false。
    Part(String),
    /// 组件宿主节点标记。
    Host,
}

/// 级联按三元组逐项比较，确保任意数量的 class-like 条件都不能超过一个 ID 条件。
/// CSS/USS cascade precedence ordered by ID, class-like selectors, and types.
///
/// This intentionally stays as a tuple rather than a weighted integer: any
/// number of class-like selectors must not tie an ID selector.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UiSelectorSpecificity {
    /// 全部段中的 ID token 数。
    id_count: usize,
    /// class、state、part 和 host token 总数。
    class_like_count: usize,
    /// 非通配类型 token 数。
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
    /// Editor `collect_matching_rules` 用此历史分值展示匹配规则；排序仍按 `(cascade_specificity, source_order)`。
    /// 公式使用普通 `usize` 乘加，因此调用方应只对实际解析所得、分值可表示的选择器用于展示；它不是级联排序键。
    pub const fn legacy_display_score(self) -> usize {
        const ID_WEIGHT: usize = 100;
        const CLASS_LIKE_WEIGHT: usize = 10;

        self.id_count * ID_WEIGHT + self.class_like_count * CLASS_LIKE_WEIGHT + self.type_count
    }
}

impl UiSelector {
    /// 解析受支持的类型、通配符、class、id、state、`:part(...)`、`:host` 及空白/`>` 组合器；拒绝空段和尾随组合器。
    ///
    /// # 参数
    /// `input` 是选择器文本；该文法是本 API 的受支持子集，不承诺完整 CSS 语法。
    ///
    /// # 返回值
    /// 返回首段无 combinator、各段有 token 的解析路径；语法无效时返回 `UiAssetError::InvalidSelector`。
    /// Runtime 文档验证与样式应用都使用该解析结果，避免把公开可构造/反序列化的任意段视为已验证。
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
    /// 返回值按 ID、class-like、type 计数作字典序比较；样式级联再以源码顺序作为同优先级的后续键。
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
/// 将一个不含组合器的片段拆为类型和限定 token；空 token 返回无效选择器。
/// 调用者已按空白或 `>` 切分路径段。
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

/// 去掉片段前导空白，并报告是否消费空白以区分后代组合器。
fn split_whitespace_prefix(input: &str) -> (&str, bool) {
    let remaining = input.trim_start_matches(char::is_whitespace);
    (remaining, remaining.len() != input.len())
}

#[cfg(test)]
/// 测试用字符迭代器版本，用来核对空白是否表示隐式后代关系。
fn skip_whitespace(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> bool {
    let mut saw_whitespace = false;
    while chars.peek().is_some_and(|ch| ch.is_whitespace()) {
        saw_whitespace = true;
        let _ = chars.next();
    }
    saw_whitespace
}

#[cfg(test)]
#[path = "../../asset/style/tests/selector_token_perf.rs"]
mod selector_token_performance_tests;

#[cfg(test)]
#[path = "../../asset/style/tests/selector_parse_perf.rs"]
mod selector_parse_performance_tests;

#[cfg(test)]
#[path = "../../asset/style/tests/selector_capacity_perf.rs"]
mod selector_capacity_performance_tests;
