use serde::{Deserialize, Serialize};

use crate::ui::template::UiAssetError;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiSelector {
    pub segments: Vec<UiSelectorSegment>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiSelectorSegment {
    pub combinator: Option<UiSelectorCombinator>,
    pub tokens: Vec<UiSelectorToken>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiSelectorCombinator {
    Descendant,
    Child,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
    pub fn parse(input: &str) -> Result<Self, UiAssetError> {
        let mut chars = input.chars().peekable();
        let mut segments = Vec::new();
        let mut combinator = None;

        loop {
            skip_whitespace(&mut chars);
            if chars.peek().is_none() {
                break;
            }

            let mut compound = String::new();
            while let Some(&ch) = chars.peek() {
                if ch.is_whitespace() || ch == '>' {
                    break;
                }
                compound.push(ch);
                let _ = chars.next();
            }

            if compound.is_empty() {
                return Err(UiAssetError::InvalidSelector(input.to_string()));
            }

            segments.push(UiSelectorSegment {
                combinator,
                tokens: parse_compound_tokens(&compound)?,
            });

            let saw_space = skip_whitespace(&mut chars);
            combinator = match chars.peek().copied() {
                Some('>') => {
                    let _ = chars.next();
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
                UiSelectorToken::Type(_) => specificity.type_count += 1,
            });
        specificity
    }
}

fn parse_compound_tokens(input: &str) -> Result<Vec<UiSelectorToken>, UiAssetError> {
    let chars: Vec<char> = input.chars().collect();
    let mut index = 0;
    let mut tokens = Vec::new();

    while index < chars.len() {
        let prefix = chars[index];
        let start = if matches!(prefix, '.' | '#' | ':') {
            index + 1
        } else {
            index
        };
        let mut end = start;
        while end < chars.len() && !matches!(chars[end], '.' | '#' | ':') {
            end += 1;
        }
        let value: String = chars[start..end].iter().collect();
        if value.is_empty() {
            return Err(UiAssetError::InvalidSelector(input.to_string()));
        }

        match prefix {
            '.' => tokens.push(UiSelectorToken::Class(value)),
            '#' => tokens.push(UiSelectorToken::Id(value)),
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
            ':' => tokens.push(UiSelectorToken::State(value)),
            _ => tokens.push(UiSelectorToken::Type(value)),
        }

        index = end;
    }

    Ok(tokens)
}

fn skip_whitespace(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> bool {
    let mut saw_whitespace = false;
    while chars.peek().is_some_and(|ch| ch.is_whitespace()) {
        saw_whitespace = true;
        let _ = chars.next();
    }
    saw_whitespace
}
