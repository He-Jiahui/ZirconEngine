use zircon_runtime_interface::ui::template::UiAssetError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiSelector {
    pub segments: Vec<UiSelectorSegment>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiSelectorSegment {
    pub combinator: Option<UiSelectorCombinator>,
    pub tokens: Vec<UiSelectorToken>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiSelectorCombinator {
    Descendant,
    Child,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UiSelectorToken {
    Type(String),
    Class(String),
    Id(String),
    State(String),
    Part(String),
    Host,
}

#[derive(Clone, Copy, Debug)]
pub struct UiSelectorMatchNode<'a> {
    pub component: &'a str,
    pub control_id: Option<&'a str>,
    pub classes: &'a [String],
    pub is_host: bool,
    pub states: &'a [&'a str],
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

        Ok(Self { segments })
    }

    pub fn specificity(&self) -> usize {
        self.segments
            .iter()
            .flat_map(|segment| segment.tokens.iter())
            .map(|token| match token {
                UiSelectorToken::Id(_) => 100,
                UiSelectorToken::Class(_)
                | UiSelectorToken::State(_)
                | UiSelectorToken::Part(_)
                | UiSelectorToken::Host => 10,
                UiSelectorToken::Type(_) => 1,
            })
            .sum()
    }

    pub fn matches_path(&self, path: &[UiSelectorMatchNode<'_>]) -> bool {
        if path.is_empty() || self.segments.is_empty() {
            return false;
        }

        let mut path_index = path.len() - 1;
        let mut selector_index = self.segments.len() - 1;

        if !matches_segment(&self.segments[selector_index], &path[path_index]) {
            return false;
        }

        while selector_index > 0 {
            let combinator = self.segments[selector_index].combinator;
            selector_index -= 1;
            match combinator {
                Some(UiSelectorCombinator::Child) => {
                    if path_index == 0 {
                        return false;
                    }
                    path_index -= 1;
                    if !matches_segment(&self.segments[selector_index], &path[path_index]) {
                        return false;
                    }
                }
                Some(UiSelectorCombinator::Descendant) => {
                    let mut matched = None;
                    let mut candidate = path_index;
                    while candidate > 0 {
                        candidate -= 1;
                        if matches_segment(&self.segments[selector_index], &path[candidate]) {
                            matched = Some(candidate);
                            break;
                        }
                    }
                    let Some(found) = matched else {
                        return false;
                    };
                    path_index = found;
                }
                None => return false,
            }
        }

        true
    }
}

fn matches_segment(segment: &UiSelectorSegment, node: &UiSelectorMatchNode<'_>) -> bool {
    segment.tokens.iter().all(|token| match token {
        UiSelectorToken::Type(component) => node.component == component,
        UiSelectorToken::Class(class_name) => node.classes.iter().any(|class| class == class_name),
        UiSelectorToken::Id(control_id) => node.control_id == Some(control_id.as_str()),
        UiSelectorToken::State(state) => node.states.iter().any(|value| value == state),
        UiSelectorToken::Part(_) => false,
        UiSelectorToken::Host => node.is_host,
    })
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
